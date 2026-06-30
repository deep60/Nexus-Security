#!/usr/bin/env bash
# Backup all Verdyx persistent state: every per-service Postgres database,
# the Redis snapshot, and (if present) MinIO contents.
#
# Output layout:
#   backups/<TIMESTAMP>/
#     postgres/<dbname>.sql        # one dump per service database
#     postgres/globals.sql         # roles, tablespaces (cluster-level)
#     redis.rdb
#     minio/                       # mirror of the MinIO data dir (if running)
#     manifest.json                # what was backed up + timestamps + checksums
#
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPOSE_FILE="docker-compose.yml"
BACKUP_ROOT="${BACKUP_ROOT:-$ROOT_DIR/backups}"
TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
BACKUP_DIR="$BACKUP_ROOT/$TIMESTAMP"

DB_USER="${POSTGRES_USER:-verdyx_user}"
INCLUDE_MINIO=1

# Redis runs with `--requirepass ${REDIS_PASSWORD}` in docker-compose, so the
# backup must authenticate or `SAVE` returns NOAUTH and we'd copy a stale RDB.
# Pull the value from the environment, falling back to the project .env file.
REDIS_PASSWORD="${REDIS_PASSWORD:-}"
if [[ -z "$REDIS_PASSWORD" && -f "$ROOT_DIR/.env" ]]; then
  REDIS_PASSWORD="$(grep -E '^REDIS_PASSWORD=' "$ROOT_DIR/.env" | tail -1 | cut -d= -f2- | sed -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")"
fi

# All per-service databases, kept in sync with database/init/01-init-databases.sql.
DATABASES=(
  verdyx
  verdyx_gateway
  verdyx_users
  verdyx_analysis
  verdyx_bounty
  verdyx_submissions
  verdyx_consensus
  verdyx_payments
  verdyx_reputation
  verdyx_notifications
)

usage() {
  cat <<USAGE
Usage: scripts/maintenance/backup.sh [options]

Options:
  --compose-file <file>   Compose file to use (default: docker-compose.yml)
  --backup-root <dir>     Output backup root (default: ./backups)
  --no-minio              Skip MinIO snapshot
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
    --compose-file) COMPOSE_FILE="$2"; shift 2 ;;
    --backup-root)  BACKUP_ROOT="$2"; BACKUP_DIR="$BACKUP_ROOT/$TIMESTAMP"; shift 2 ;;
    --no-minio)     INCLUDE_MINIO=0; shift ;;
    -h|--help)      usage; exit 0 ;;
    *)              echo "[error] Unknown option: $1"; usage; exit 1 ;;
  esac
done

cd "$ROOT_DIR"
COMPOSE="$(compose_cmd)"

mkdir -p "$BACKUP_DIR/postgres"

echo "[info] Ensuring postgres and redis are running"
$COMPOSE -f "$COMPOSE_FILE" up -d postgres redis >/dev/null

# ---------------------------------------------------------------- postgres
echo "[info] Dumping cluster globals"
$COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
  pg_dumpall -U "$DB_USER" --globals-only > "$BACKUP_DIR/postgres/globals.sql"

backed_up=()
for db in "${DATABASES[@]}"; do
  # Connect to the always-present `postgres` maintenance DB to query the
  # catalog. Without -d, psql would try to connect to a database named after
  # the user ("verdyx_user"), which doesn't exist, and every DB would be
  # wrongly skipped.
  exists=$($COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
    psql -U "$DB_USER" -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname='$db'" 2>/dev/null || true)
  if [[ "$exists" != "1" ]]; then
    echo "[warn] Skipping $db (does not exist on this server)"
    continue
  fi
  echo "[info] Dumping $db"
  $COMPOSE -f "$COMPOSE_FILE" exec -T postgres \
    pg_dump -U "$DB_USER" -d "$db" --format=plain --no-owner --no-privileges \
    > "$BACKUP_DIR/postgres/${db}.sql"
  backed_up+=("$db")
done

# ------------------------------------------------------------------- redis
echo "[info] Snapshotting redis"
# Authenticate when a password is configured; `--no-auth-warning` keeps the
# password out of stderr. Verify SAVE actually succeeded before copying the
# RDB so a silent NOAUTH/auth failure can't produce a stale/empty backup.
redis_auth=()
if [[ -n "$REDIS_PASSWORD" ]]; then
  redis_auth=(-a "$REDIS_PASSWORD" --no-auth-warning)
fi
if ! $COMPOSE -f "$COMPOSE_FILE" exec -T redis \
    redis-cli "${redis_auth[@]}" SAVE | grep -q OK; then
  echo "[error] redis SAVE failed (check REDIS_PASSWORD); aborting redis backup" >&2
  exit 1
fi
$COMPOSE -f "$COMPOSE_FILE" cp redis:/data/dump.rdb "$BACKUP_DIR/redis.rdb"

# ------------------------------------------------------------------- minio
if [[ "$INCLUDE_MINIO" -eq 1 ]] \
  && $COMPOSE -f "$COMPOSE_FILE" ps minio --status running 2>/dev/null | grep -q minio; then
  echo "[info] Copying MinIO data"
  mkdir -p "$BACKUP_DIR/minio"
  $COMPOSE -f "$COMPOSE_FILE" cp minio:/data/. "$BACKUP_DIR/minio/" || \
    echo "[warn] MinIO copy failed; skipping"
fi

# ----------------------------------------------------------------- snapshot
if [[ -f .env ]]; then
  cp .env "$BACKUP_DIR/.env.snapshot"
fi

# ----------------------------------------------------------------- manifest
if [[ ${#backed_up[@]} -eq 0 ]]; then
  echo "[error] No databases were backed up — check POSTGRES_USER and that the" >&2
  echo "        databases exist. Refusing to write an empty manifest." >&2
  exit 1
fi

{
  echo '{'
  echo "  \"timestamp\": \"$TIMESTAMP\","
  echo "  \"databases\": ["
  printf '    "%s"' "${backed_up[0]}"
  for db in "${backed_up[@]:1}"; do printf ',\n    "%s"' "$db"; done
  echo
  echo '  ],'
  echo "  \"redis_rdb\": $( [[ -f $BACKUP_DIR/redis.rdb ]] && echo true || echo false ),"
  echo "  \"minio_included\": $( [[ -d $BACKUP_DIR/minio ]] && echo true || echo false )"
  echo '}'
} > "$BACKUP_DIR/manifest.json"

echo "[ok] Backup complete: $BACKUP_DIR"
