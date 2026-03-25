#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPOSE_FILE="docker-compose.yml"
BACKUP_ROOT="${BACKUP_ROOT:-$ROOT_DIR/backups}"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
BACKUP_DIR="$BACKUP_ROOT/$TIMESTAMP"

DB_NAME="${DB_NAME:-nexus_security}"
DB_USER="${DB_USER:-nexus_user}"

usage() {
  cat <<USAGE
Usage: scripts/maintenance/backup.sh [options]

Options:
  --compose-file <file>   Compose file to use (default: docker-compose.yml)
  --backup-root <dir>     Output backup root (default: ./backups)
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
    --backup-root)
      BACKUP_ROOT="$2"
      shift 2
      BACKUP_DIR="$BACKUP_ROOT/$TIMESTAMP"
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

mkdir -p "$BACKUP_DIR"

echo "[info] Ensuring postgres and redis are running"
$COMPOSE -f "$COMPOSE_FILE" up -d postgres redis >/dev/null

echo "[info] Backing up postgres -> $BACKUP_DIR/postgres.sql"
$COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
  pg_dump -U "$DB_USER" -d "$DB_NAME" > "$BACKUP_DIR/postgres.sql"

echo "[info] Backing up redis -> $BACKUP_DIR/redis.rdb"
$COMPOSE -f "$COMPOSE_FILE" exec -T redis redis-cli SAVE >/dev/null
$COMPOSE -f "$COMPOSE_FILE" cp redis:/data/dump.rdb "$BACKUP_DIR/redis.rdb"

if [[ -f .env ]]; then
  cp .env "$BACKUP_DIR/.env.snapshot"
fi

echo "[ok] Backup complete: $BACKUP_DIR"
