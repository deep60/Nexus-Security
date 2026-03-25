#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPOSE_FILE="docker-compose.yml"
ENV_FILE=".env"
SERVICES=""
NO_BUILD=0

usage() {
  cat <<USAGE
Usage: scripts/deployment/deploy.sh [options]

Options:
  --compose-file <file>   Compose file to deploy (default: docker-compose.yml)
  --env-file <file>       Env file for deployment (default: .env)
  --services "svc1 svc2"  Deploy only selected services
  --no-build              Do not build images before up
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

while [[ $# -gt 0 ]]; do
  case "$1" in
    --compose-file)
      COMPOSE_FILE="$2"
      shift 2
      ;;
    --env-file)
      ENV_FILE="$2"
      shift 2
      ;;
    --services)
      SERVICES="$2"
      shift 2
      ;;
    --no-build)
      NO_BUILD=1
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

if [[ ! -f "$COMPOSE_FILE" ]]; then
  echo "[error] Compose file not found: $COMPOSE_FILE"
  exit 1
fi

if [[ ! -f "$ENV_FILE" ]]; then
  echo "[warn] Env file not found: $ENV_FILE"
fi

if [[ "$NO_BUILD" -eq 0 ]]; then
  echo "[info] Building images"
  $COMPOSE --env-file "$ENV_FILE" -f "$COMPOSE_FILE" build $SERVICES
fi

echo "[info] Deploying services"
$COMPOSE --env-file "$ENV_FILE" -f "$COMPOSE_FILE" up -d $SERVICES

echo "[info] Running post-deploy health checks"
bash scripts/deployment/health-check.sh --compose-file "$COMPOSE_FILE"

echo "[ok] Deployment completed"
