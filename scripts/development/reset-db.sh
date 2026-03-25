#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPOSE_FILE="docker-compose.dev.yml"
AUTO_YES=0
WITH_SEED=0

DB_NAME="${DB_NAME:-nexus_security}"
DB_USER="${DB_USER:-nexus_user}"
DB_PASSWORD="${DB_PASSWORD:-nexus_password}"

usage() {
  cat <<USAGE
Usage: scripts/development/reset-db.sh [options]

Options:
  --compose-file <file>   Compose file to use (default: docker-compose.dev.yml)
  --with-seed             Load database/postgres/seeds/test_data.sql after reset
  -y, --yes               Skip confirmation prompt
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
    --with-seed)
      WITH_SEED=1
      shift
      ;;
    -y|--yes)
      AUTO_YES=1
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

if [[ "$AUTO_YES" -eq 0 ]]; then
  echo "This will drop and recreate the '$DB_NAME' database schema."
  read -r -p "Continue? [y/N]: " confirm
  if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    echo "[info] Cancelled"
    exit 0
  fi
fi

$COMPOSE -f "$COMPOSE_FILE" up -d postgres

export PGPASSWORD="$DB_PASSWORD"

echo "[info] Recreating public schema"
$COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
  psql -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 \
  -c "DROP SCHEMA IF EXISTS public CASCADE; CREATE SCHEMA public;"

echo "[info] Applying migrations"
for migration in $(find database/postgres/migrations -type f -name '*.sql' | sort); do
  echo "[info] -> $migration"
  $COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
    psql -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 < "$migration"
done

if [[ "$WITH_SEED" -eq 1 && -f database/postgres/seeds/test_data.sql ]]; then
  echo "[info] Loading seed data"
  $COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
    psql -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 < database/postgres/seeds/test_data.sql
fi

echo "[ok] Database reset complete"
