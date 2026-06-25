#!/usr/bin/env bash
# Compare the number of migration files on disk with the number of rows in
# each per-service _sqlx_migrations table. Drift between the two means a
# service started against an older or newer schema than it expects.
#
# Usage: scripts/maintenance/check-migrations.sh
#
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.yml}"
DB_USER="${POSTGRES_USER:-verdyx_user}"

# service-name : per-service database : path to migrations folder
declare -a SERVICES=(
  "api-gateway          verdyx_gateway        backend/api-gateway/migrations"
  "user-service         verdyx_users          backend/user-service/migrations"
  "analysis-engine      verdyx_analysis       backend/analysis-engine/migrations"
  "bounty-manager       verdyx_bounty         backend/bounty-manager/migrations"
  "submission-service   verdyx_submissions    backend/submission-service/migrations"
  "consensus-service    verdyx_consensus      backend/consensus-service/migrations"
  "payment-service      verdyx_payments       backend/payment-service/migrations"
  "reputation-service   verdyx_reputation     backend/reputation-service/migrations"
  "notification-service verdyx_notifications  backend/notification-service/migrations"
)

cd "$ROOT_DIR"
problems=0

printf '%-22s %-22s %8s %8s %s\n' "service" "database" "files" "applied" "status"
printf '%-22s %-22s %8s %8s %s\n' "-------" "--------" "-----" "-------" "------"

for entry in "${SERVICES[@]}"; do
  svc="$(awk '{print $1}' <<<"$entry")"
  db="$(awk '{print $2}' <<<"$entry")"
  dir="$(awk '{print $3}' <<<"$entry")"

  files=$(find "$dir" -maxdepth 1 -name '*.sql' 2>/dev/null | wc -l | tr -d ' ')
  applied=$(docker compose -f "$COMPOSE_FILE" exec -T postgres \
    psql -U "$DB_USER" -d "$db" -tAc \
    "SELECT count(*) FROM _sqlx_migrations" 2>/dev/null || echo "?")

  status="ok"
  if [[ "$applied" == "?" ]]; then
    status="unreachable"
    problems=$((problems + 1))
  elif [[ "$files" != "$applied" ]]; then
    status="DRIFT"
    problems=$((problems + 1))
  fi

  printf '%-22s %-22s %8s %8s %s\n' "$svc" "$db" "$files" "$applied" "$status"
done

if [[ $problems -gt 0 ]]; then
  echo
  echo "[error] Drift detected in $problems service(s). See docs/database/rollback-strategy.md."
  exit 1
fi
echo
echo "[ok] All services match their migration history."
