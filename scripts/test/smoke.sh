#!/usr/bin/env bash
# Smoke-test every Verdyx service: spin up the compose stack, wait for each
# container to become healthy, and hit liveness, readiness, and metrics
# endpoints. Use this in CI for a "does the whole world come up?" check, and
# locally before merging risky branches.
#
# What it verifies for each backend service:
#   - liveness  (process is up)
#   - readiness (DB + dependencies reachable)
#   - metrics   (Prometheus text format served, exposes verdyx_service_up)
#
# Usage:
#   scripts/test/smoke.sh                  # run against current docker-compose
#   BASE_URL=https://api.verdyx.io \
#     scripts/test/smoke.sh --remote       # test a deployed environment
#
# Exit code 0 on success, 1 on any failure.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.yml}"
MODE="local"
# Per-target deadline. ClamAV has a 300s start_period; this needs to be
# generous enough to cover slow first boots without dragging CI forever.
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-300}"

# service-name host:port for each backend in local mode. Each row maps to
# three probes: liveness, readiness, metrics.
declare -a LOCAL_BACKENDS=(
  "api-gateway          http://localhost:8080"
  "user-service         http://localhost:8081"
  "analysis-engine      http://localhost:8082"
  "bounty-manager       http://localhost:8083"
  "submission-service   http://localhost:8084"
  "consensus-service    http://localhost:8085"
  "payment-service      http://localhost:8086"
  "reputation-service   http://localhost:8087"
  "notification-service http://localhost:8088"
)

# Frontend exposes only "/" so we treat it specially.
FRONTEND_URL_LOCAL="http://localhost:5001/"

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
  # Catch postgres-password drift (volume or stale-container) BEFORE bringing
  # the stack up — otherwise services crashloop with a misleading
  # "password authentication failed" error and no hint of root cause. See
  # scripts/preflight.sh for full detail. Skip with SKIP_PREFLIGHT=1.
  if [[ "${SKIP_PREFLIGHT:-0}" != "1" ]] && [[ -x scripts/preflight.sh ]]; then
    log "Running preflight (postgres volume + container env drift check)"
    if ! scripts/preflight.sh; then
      log "Preflight reported drift. Re-run 'scripts/preflight.sh --fix' to heal,"
      log "or set SKIP_PREFLIGHT=1 to bypass."
      exit 1
    fi
  fi

  log "Bringing up the compose stack (this can take several minutes on first boot)"
  docker compose -f "$COMPOSE_FILE" up -d --wait || {
    log "compose up --wait failed; falling back to plain up + manual wait"
    docker compose -f "$COMPOSE_FILE" up -d
  }
fi

failed=0

# Wait for $url to return HTTP 2xx, retrying every 2s until $TIMEOUT_SECONDS.
# Returns 0 on success, 1 on timeout. Stderr is captured so probe noise does
# not pollute the smoke output.
probe() {
  local url="$1"
  local deadline=$((SECONDS + TIMEOUT_SECONDS))
  while [[ $SECONDS -lt $deadline ]]; do
    if curl -fsS --max-time 5 "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  return 1
}

# Stronger metrics check: must respond 200 AND include verdyx_service_up
# (lets us catch a service that returns 200 from an error handler).
probe_metrics() {
  local url="$1"
  local deadline=$((SECONDS + TIMEOUT_SECONDS))
  while [[ $SECONDS -lt $deadline ]]; do
    if body="$(curl -fsS --max-time 5 "$url" 2>/dev/null)"; then
      if printf '%s' "$body" | grep -q '^verdyx_service_up'; then
        return 0
      fi
    fi
    sleep 2
  done
  return 1
}

check_backend() {
  local name="$1"
  local base="$2"
  local live="$base/health"
  local ready="$base/ready"
  local metrics="$base/metrics"
  # api-gateway exposes liveness at the legacy path too; accept either.
  if [[ "$name" == "api-gateway" ]]; then
    live="$base/health"
  fi

  log "Probing $name liveness  -> $live"
  if probe "$live"; then log "  ✓ $name liveness"; else log "  ✗ $name liveness FAILED"; failed=$((failed + 1)); fi

  log "Probing $name readiness -> $ready"
  if probe "$ready"; then log "  ✓ $name readiness"; else log "  ✗ $name readiness FAILED"; failed=$((failed + 1)); fi

  log "Probing $name metrics   -> $metrics"
  if probe_metrics "$metrics"; then log "  ✓ $name metrics"; else log "  ✗ $name metrics FAILED"; failed=$((failed + 1)); fi
}

if [[ "$MODE" == "local" ]]; then
  for entry in "${LOCAL_BACKENDS[@]}"; do
    name="$(awk '{print $1}' <<<"$entry")"
    base="$(awk '{print $2}' <<<"$entry")"
    check_backend "$name" "$base"
  done

  log "Probing frontend -> $FRONTEND_URL_LOCAL"
  if probe "$FRONTEND_URL_LOCAL"; then log "  ✓ frontend"; else log "  ✗ frontend FAILED"; failed=$((failed + 1)); fi
else
  # Remote mode: only the api-gateway and frontend are externally reachable;
  # downstream services live on the internal network.
  GATEWAY_BASE="${BASE_URL:-http://localhost:8080}"
  FRONTEND_URL="${FRONTEND_URL:-http://localhost:5001}/"

  log "Probing api-gateway liveness  -> $GATEWAY_BASE/health"
  if probe "$GATEWAY_BASE/health"; then log "  ✓ gateway liveness"; else log "  ✗ gateway liveness FAILED"; failed=$((failed + 1)); fi

  log "Probing api-gateway readiness -> $GATEWAY_BASE/ready"
  if probe "$GATEWAY_BASE/ready"; then log "  ✓ gateway readiness"; else log "  ✗ gateway readiness FAILED"; failed=$((failed + 1)); fi

  log "Probing frontend -> $FRONTEND_URL"
  if probe "$FRONTEND_URL"; then log "  ✓ frontend"; else log "  ✗ frontend FAILED"; failed=$((failed + 1)); fi
fi

if [[ $failed -gt 0 ]]; then
  log "Smoke test FAILED: $failed check(s) failed"
  if [[ "$MODE" == "local" ]]; then
    log "Recent logs from compose:"
    docker compose -f "$COMPOSE_FILE" logs --tail=50 || true
  fi
  exit 1
fi

log "Smoke test PASSED"
