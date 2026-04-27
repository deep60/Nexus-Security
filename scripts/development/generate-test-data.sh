#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPOSE_FILE="docker-compose.dev.yml"
SEED_FILE="database/postgres/seeds/test_data.sql"

DB_NAME="${DB_NAME:-verdyx}"
DB_USER="${DB_USER:-verdyx_user}"

usage() {
  cat <<USAGE
Usage: scripts/development/generate-test-data.sh [options]

Options:
  --compose-file <file>   Compose file to use (default: docker-compose.dev.yml)
  --seed-file <path>      Seed SQL file (default: database/postgres/seeds/test_data.sql)
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
    --seed-file)
      SEED_FILE="$2"
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

if [[ ! -f "$SEED_FILE" ]]; then
  echo "[error] Seed file not found: $SEED_FILE"
  exit 1
fi

$COMPOSE -f "$COMPOSE_FILE" up -d postgres

echo "[info] Loading seed data from $SEED_FILE"
$COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
  psql -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 < "$SEED_FILE"

echo "[ok] Test data generated"
