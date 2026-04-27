#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPOSE_FILE="docker-compose.yml"
BACKUP_DIR=""
AUTO_YES=0

DB_NAME="${DB_NAME:-verdyx}"
DB_USER="${DB_USER:-verdyx_user}"

usage() {
  cat <<USAGE
Usage: scripts/maintenance/restore.sh --backup <dir> [options]

Options:
  --backup <dir>          Backup directory created by backup.sh (required)
  --compose-file <file>   Compose file to use (default: docker-compose.yml)
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
    --backup)
      BACKUP_DIR="$2"
      shift 2
      ;;
    --compose-file)
      COMPOSE_FILE="$2"
      shift 2
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

if [[ -z "$BACKUP_DIR" ]]; then
  echo "[error] --backup <dir> is required"
  usage
  exit 1
fi

if [[ ! -d "$BACKUP_DIR" ]]; then
  echo "[error] Backup directory not found: $BACKUP_DIR"
  exit 1
fi

if [[ ! -f "$BACKUP_DIR/postgres.sql" ]]; then
  echo "[error] postgres.sql missing from backup directory"
  exit 1
fi

cd "$ROOT_DIR"
COMPOSE="$(compose_cmd)"

if [[ "$AUTO_YES" -eq 0 ]]; then
  echo "This will overwrite live postgres and redis data."
  read -r -p "Continue? [y/N]: " confirm
  if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    echo "[info] Cancelled"
    exit 0
  fi
fi

echo "[info] Ensuring postgres and redis are running"
$COMPOSE -f "$COMPOSE_FILE" up -d postgres redis >/dev/null

echo "[info] Restoring postgres"
$COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
  psql -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 \
  -c "DROP SCHEMA IF EXISTS public CASCADE; CREATE SCHEMA public;"
$COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
  psql -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 < "$BACKUP_DIR/postgres.sql"

if [[ -f "$BACKUP_DIR/redis.rdb" ]]; then
  echo "[info] Restoring redis"
  $COMPOSE -f "$COMPOSE_FILE" cp "$BACKUP_DIR/redis.rdb" redis:/data/dump.rdb
  $COMPOSE -f "$COMPOSE_FILE" restart redis >/dev/null
else
  echo "[warn] redis.rdb not found; skipped redis restore"
fi

echo "[ok] Restore complete"
