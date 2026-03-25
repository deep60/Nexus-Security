#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PROFILE="dev"
DETACH=1

usage() {
  cat <<USAGE
Usage: scripts/development/start-services.sh [options]

Options:
  --profile <dev|full|infra>   Service profile to start (default: dev)
  --no-detach                  Run in foreground
  -h, --help                   Show this help message
USAGE
}

compose_cmd() {
  if command -v docker-compose >/dev/null 2>&1; then
    echo "docker-compose"
  else
    echo "docker compose"
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      PROFILE="$2"
      shift 2
      ;;
    --no-detach)
      DETACH=0
      shift
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
UP_FLAGS=""
if [[ "$DETACH" -eq 1 ]]; then
  UP_FLAGS="-d"
fi

case "$PROFILE" in
  dev)
    echo "[info] Starting development stack (docker-compose.dev.yml)"
    $COMPOSE -f docker-compose.dev.yml up $UP_FLAGS
    ;;
  full)
    echo "[info] Starting full stack (docker-compose.yml)"
    $COMPOSE up $UP_FLAGS
    ;;
  infra)
    echo "[info] Starting infrastructure services only"
    $COMPOSE up $UP_FLAGS postgres redis minio clamav
    ;;
  *)
    echo "[error] Invalid profile: $PROFILE"
    usage
    exit 1
    ;;
esac

echo "[ok] Services started"
echo "[info] Common endpoints:"
echo "  frontend:         http://localhost:5000"
echo "  api-gateway:      http://localhost:8080"
echo "  user-service:     http://localhost:8081/health"
echo "  analysis-engine:  http://localhost:8082/health"
