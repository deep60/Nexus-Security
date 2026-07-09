#!/usr/bin/env bash
#
# On-VM deploy entrypoint. Runs ON the server (/opt/verdyx), invoked over SSH by
# .github/workflows/deploy.yml and staging.yml. Keeping the deploy sequence in a
# committed script (instead of an inline SSH one-liner) makes it reviewable,
# testable, and consistent between staging and production.
#
# Sequence:
#   1. Record the currently-deployed git SHA (rollback target).
#   2. Pull the new revision.
#   3. Ensure per-service databases exist on RDS (ensure-databases.sh).
#   4. docker compose pull + up -d for the requested services.
#   5. Health check; on failure, automatically roll back to the previous
#      revision and restart, so a bad deploy never leaves prod down.
#
# Inputs (env):
#   SERVICES   space-separated service list, or empty for "all" (default "")
#   BRANCH     git branch to pull (default: main)
#   APP_DIR    deploy directory (default: /opt/verdyx)
#   .env in APP_DIR must define POSTGRES_HOST (RDS endpoint), POSTGRES_USER,
#   POSTGRES_PASSWORD, etc.
#
set -euo pipefail

APP_DIR="${APP_DIR:-/opt/verdyx}"
BRANCH="${BRANCH:-main}"
SERVICES="${SERVICES:-}"

# The app-tier services this VM runs. Production/staging use managed RDS +
# ElastiCache (set via POSTGRES_HOST / REDIS_HOST in .env), so the local
# `postgres`/`redis` compose services are intentionally NOT started here — we
# deploy an explicit app-service list with `--no-deps`. (Local dev still gets
# the full stack via a plain `docker compose up`.)
ALL_APP_SERVICES="api-gateway analysis-engine bounty-manager consensus-service notification-service payment-service reputation-service submission-service user-service frontend"

cd "$APP_DIR"

compose() {
  if command -v docker-compose >/dev/null 2>&1; then docker-compose "$@"; else docker compose "$@"; fi
}

# Poll the local api-gateway health endpoint. Returns non-zero if never healthy.
health_ok() {
  curl --fail --silent --retry 5 --retry-delay 6 http://localhost:8080/api/v1/health >/dev/null
}

# Empty SERVICES means "all app services" (e.g. staging deploys everything).
[ -z "$SERVICES" ] && SERVICES="$ALL_APP_SERVICES"

echo "==> [deploy] app_dir=$APP_DIR branch=$BRANCH services=$SERVICES"

# 1. Remember where we are so a later step can roll back to it.
PREV_SHA="$(git rev-parse HEAD)"
echo "==> [deploy] current revision: $PREV_SHA"
echo "$PREV_SHA" > .last-deployed-sha

# 2. Pull the new revision.
git fetch origin "$BRANCH"
git reset --hard "origin/${BRANCH}"
echo "==> [deploy] updated to: $(git rev-parse HEAD)"

# 3. Ensure per-service databases exist on RDS before services self-migrate.
#    Load the server env, then map POSTGRES_* -> libpq PG* for the script.
set -a; . ./.env; set +a
PGHOST="${POSTGRES_HOST:?POSTGRES_HOST (RDS endpoint) must be set in $APP_DIR/.env}" \
PGPORT="${POSTGRES_PORT:-5432}" \
PGUSER="${POSTGRES_USER:-verdyx}" \
PGPASSWORD="${POSTGRES_PASSWORD:?POSTGRES_PASSWORD must be set in $APP_DIR/.env}" \
PGADMINDB="${POSTGRES_ADMIN_DB:-verdyx}" \
  bash scripts/deployment/ensure-databases.sh

# 4. Pull images and (re)start the requested services. Each service applies its
#    own migrations at startup via sqlx::migrate!.
# shellcheck disable=SC2086  # word-splitting SERVICES is intentional
compose pull $SERVICES
# shellcheck disable=SC2086
compose up -d --no-deps $SERVICES

# 5. Health check (local, before external DNS/TLS). On failure, roll back.
echo "==> [deploy] waiting for services to become healthy"
sleep 20
if health_ok; then
  echo "==> [deploy] healthy. Pruning old images."
  docker system prune -af --filter until=24h || true
  echo "==> [deploy] done ($(git rev-parse --short HEAD))."
  exit 0
fi

# ── Rollback ────────────────────────────────────────────────────────────────
echo "::error::[deploy] health check FAILED at $(git rev-parse --short HEAD). Rolling back to ${PREV_SHA:0:7}."
git reset --hard "$PREV_SHA"
# shellcheck disable=SC2086
compose pull $SERVICES || true
# shellcheck disable=SC2086
compose up -d --no-deps $SERVICES

echo "==> [rollback] waiting for previous revision to become healthy"
sleep 20
if health_ok; then
  echo "::warning::[deploy] rolled back to ${PREV_SHA:0:7} and it is healthy. Deploy FAILED (investigate)."
  exit 1
fi

echo "::error::[deploy] rollback to ${PREV_SHA:0:7} is ALSO unhealthy — manual intervention required."
exit 1
