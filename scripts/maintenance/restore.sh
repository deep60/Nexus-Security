#!/usr/bin/env bash
# Restore a Verdyx backup created by backup.sh.
#
# Restores every per-service Postgres database listed in manifest.json (or
# discovered as *.sql under postgres/), then restores Redis and MinIO if the
# backup contains them.
#
# Usage:
#   scripts/maintenance/restore.sh --backup backups/<TIMESTAMP> [--yes]
#
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPOSE_FILE="docker-compose.yml"
BACKUP_DIR=""
AUTO_YES=0

DB_USER="${POSTGRES_USER:-verdyx_user}"

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
    --backup)        BACKUP_DIR="$2"; shift 2 ;;
    --compose-file)  COMPOSE_FILE="$2"; shift 2 ;;
    -y|--yes)        AUTO_YES=1; shift ;;
    -h|--help)       usage; exit 0 ;;
    *)               echo "[error] Unknown option: $1"; usage; exit 1 ;;
  esac
done

if [[ -z "$BACKUP_DIR" || ! -d "$BACKUP_DIR" ]]; then
  echo "[error] --backup must point to an existing directory"
  usage
  exit 1
fi
if [[ ! -d "$BACKUP_DIR/postgres" ]]; then
  echo "[error] Backup is missing postgres/ subdirectory"
  exit 1
fi

cd "$ROOT_DIR"
COMPOSE="$(compose_cmd)"

if [[ "$AUTO_YES" -eq 0 ]]; then
  echo "This will OVERWRITE every Verdyx database, Redis, and (optionally) MinIO."
  read -r -p "Continue? [y/N]: " confirm
  [[ "$confirm" =~ ^[Yy]$ ]] || { echo "[info] Cancelled"; exit 0; }
fi

echo "[info] Ensuring postgres and redis are running"
$COMPOSE -f "$COMPOSE_FILE" up -d postgres redis >/dev/null

# Restore globals first (safe to skip on failure — usually identical roles).
if [[ -f "$BACKUP_DIR/postgres/globals.sql" ]]; then
  echo "[info] Restoring cluster globals"
  $COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
    psql -U "$DB_USER" -d postgres -v ON_ERROR_STOP=0 \
    < "$BACKUP_DIR/postgres/globals.sql" >/dev/null || \
    echo "[warn] globals.sql restore reported errors (often pre-existing roles); continuing"
fi

# Restore each per-service dump into its own database.
for sql in "$BACKUP_DIR"/postgres/*.sql; do
  fname="$(basename "$sql")"
  [[ "$fname" == "globals.sql" ]] && continue
  db="${fname%.sql}"

  echo "[info] Restoring $db"
  # Drop and recreate to guarantee a clean state.
  $COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
    psql -U "$DB_USER" -d postgres -v ON_ERROR_STOP=1 -c \
    "DROP DATABASE IF EXISTS \"$db\" WITH (FORCE);"
  $COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
    psql -U "$DB_USER" -d postgres -v ON_ERROR_STOP=1 -c \
    "CREATE DATABASE \"$db\" OWNER \"$DB_USER\";"
  $COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
    psql -U "$DB_USER" -d "$db" -v ON_ERROR_STOP=1 < "$sql" >/dev/null
done

# Redis
if [[ -f "$BACKUP_DIR/redis.rdb" ]]; then
  echo "[info] Restoring Redis"
  $COMPOSE -f "$COMPOSE_FILE" cp "$BACKUP_DIR/redis.rdb" redis:/data/dump.rdb
  $COMPOSE -f "$COMPOSE_FILE" restart redis >/dev/null
else
  echo "[warn] redis.rdb not found; skipped Redis restore"
fi

# MinIO (optional)
if [[ -d "$BACKUP_DIR/minio" ]] \
  && $COMPOSE -f "$COMPOSE_FILE" ps minio --status running 2>/dev/null | grep -q minio; then
  echo "[info] Restoring MinIO"
  $COMPOSE -f "$COMPOSE_FILE" cp "$BACKUP_DIR/minio/." minio:/data/ || \
    echo "[warn] MinIO restore copy failed; manual recovery may be needed"
fi

echo "[ok] Restore complete"
