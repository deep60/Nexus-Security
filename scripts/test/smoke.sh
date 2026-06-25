#!/usr/bin/env bash
# Smoke-test every Verdyx service: spin up the compose stack, wait for each
# container to become healthy, and hit a known endpoint. Use this in CI for
# a "does the whole world come up?" check, and locally before merging risky
# branches.
#
# Usage:
#   scripts/test/smoke.sh                  # run against current docker-compose
#   BASE_URL=https://api.verdyx.io \
#     scripts/test/smoke.sh --remote       # test a deployed environment
#
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.yml}"
MODE="local"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-180}"

# Map service-name -> host:port for local checks.
declare -a LOCAL_TARGETS=(
  "api-gateway          http://localhost:8080/api/v1/health/live"
  "user-service         http://localhost:8081/health"
  "analysis-engine      http://localhost:8082/health"
  "bounty-manager       http://localhost:8083/health"
  "submission-service   http://localhost:8084/health"
  "consensus-service    http://localhost:8085/health"
  "payment-service      http://localhost:8086/health"
  "reputation-service   http://localhost:8087/health"
  "notification-service http://localhost:8088/health"
  "frontend             http://localhost:5001/"
)

# Endpoints to probe via the public gateway.
declare -a REMOTE_TARGETS=(
  "api-gateway  ${BASE_URL:-http://localhost:8080}/api/v1/health/live"
  "frontend     ${FRONTEND_URL:-http://localhost:5001}/"
)

log() { printf '[%s] %s\n' "$(date +%H:%M:%S)" "$*"; }

while [[ $# -gt 0 ]]; do
  case "$1" in
    --remote) MODE="remote"; shift ;;
    -h|--help)
      grep '^#' "$0" | sed -e 's/^# \?//' | head -30
      exit 0
      ;;
    *) echo "[error] Unknown flag: $1"; exit 1 ;;
  esac
done

cd "$ROOT_DIR"

if [[ "$MODE" == "local" ]]; then
  log "Bringing up the compose stack"
  docker compose -f "$COMPOSE_FILE" up -d --wait || {
    log "compose up --wait failed; falling back to plain up + manual wait"
    docker compose -f "$COMPOSE_FILE" up -d
  }
  TARGETS=("${LOCAL_TARGETS[@]}")
else
  TARGETS=("${REMOTE_TARGETS[@]}")
fi

failed=0
for entry in "${TARGETS[@]}"; do
  name="$(awk '{print $1}' <<<"$entry")"
  url="$(awk '{print $2}' <<<"$entry")"

  log "Probing $name -> $url"
  deadline=$((SECONDS + TIMEOUT_SECONDS))
  ok=0
  while [[ $SECONDS -lt $deadline ]]; do
    if curl -fsS --max-time 5 "$url" >/dev/null 2>&1; then
      ok=1; break
    fi
    sleep 2
  done

  if [[ $ok -eq 1 ]]; then
    log "  ✓ $name healthy"
  else
    log "  ✗ $name FAILED ($url)"
    failed=$((failed + 1))
  fi
done

if [[ $failed -gt 0 ]]; then
  log "Smoke test FAILED: $failed service(s) unhealthy"
  if [[ "$MODE" == "local" ]]; then
    log "Recent logs from failing containers:"
    docker compose -f "$COMPOSE_FILE" logs --tail=50 || true
  fi
  exit 1
fi

log "Smoke test PASSED"
