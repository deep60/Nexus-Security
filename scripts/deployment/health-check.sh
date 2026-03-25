#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPOSE_FILE="docker-compose.yml"
TIMEOUT_SECONDS=5

usage() {
  cat <<USAGE
Usage: scripts/deployment/health-check.sh [options]

Options:
  --compose-file <file>   Compose file to inspect (default: docker-compose.yml)
  --timeout <seconds>     Curl timeout in seconds (default: 5)
  -h, --help              Show this help message
USAGE
}

compose_cmd() {
  if command -v docker-compose >/dev/null 2>&1; then
    echo "docker-compose"
  else
    echo "docker compose"
  fi
}

check_http() {
  local name="$1"
  local url="$2"

  if curl -fsS --max-time "$TIMEOUT_SECONDS" "$url" >/dev/null 2>&1; then
    echo "[ok]   $name -> $url"
    return 0
  fi

  echo "[fail] $name -> $url"
  return 1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --compose-file)
      COMPOSE_FILE="$2"
      shift 2
      ;;
    --timeout)
      TIMEOUT_SECONDS="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[error] Unknown option: $1"
      usage
      exit 1
      ;;
  esac
done

cd "$ROOT_DIR"
COMPOSE="$(compose_cmd)"

FAILED=0

echo "[info] Checking container status"
if ! $COMPOSE -f "$COMPOSE_FILE" ps >/dev/null; then
  echo "[fail] Could not query docker compose status"
  exit 1
fi

echo "[info] Checking HTTP health endpoints"
check_http "api-gateway" "http://localhost:8080/api/v1/health" || FAILED=1
check_http "user-service" "http://localhost:8081/health" || FAILED=1
check_http "analysis-engine" "http://localhost:8082/health" || FAILED=1
check_http "bounty-manager" "http://localhost:8083/health" || FAILED=1
check_http "submission-service" "http://localhost:8084/health" || FAILED=1
check_http "consensus-service" "http://localhost:8085/health" || FAILED=1
check_http "payment-service" "http://localhost:8086/health" || FAILED=1
check_http "reputation-service" "http://localhost:8087/health" || FAILED=1
check_http "notification-service" "http://localhost:8088/health" || FAILED=1
check_http "frontend" "http://localhost:5000" || FAILED=1

if [[ "$FAILED" -ne 0 ]]; then
  echo "[fail] One or more health checks failed"
  exit 1
fi

echo "[ok] All health checks passed"
