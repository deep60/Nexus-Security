#!/bin/sh
# Nightly logical backup of the whole Postgres cluster (all Verdyx databases
# + roles) via pg_dumpall, gzipped, with local retention. Runs as a long-lived
# loop container; each cycle dumps, verifies, optionally ships off-box, then
# sleeps BACKUP_INTERVAL_SECONDS.
#
# Protects against the common data-loss causes: bad migration, accidental
# DROP, logical corruption. Local dumps do NOT survive VM/disk loss — set the
# S3/MinIO vars below to also push each backup off-box (that is what makes this
# a real disaster-recovery backup rather than a same-host convenience copy).
#
# Every dump is VERIFIED before it is kept: gzip integrity + a content sanity
# check that the expected cluster/database markers are present. A silently
# truncated or empty dump is treated as a failure, not a backup. (An untested
# backup is not a backup.)
#
# Env:
#   POSTGRES_HOST            (default: postgres)
#   POSTGRES_USER            (required)
#   POSTGRES_PASSWORD        (required)
#   BACKUP_INTERVAL_SECONDS  (default: 86400 = daily)
#   BACKUP_RETENTION_DAYS    (default: 7)
#   BACKUP_ON_START          (default: 1 — take one immediately on boot)
#
#   Off-site upload (all optional; upload is skipped unless BACKUP_S3_BUCKET is set):
#   BACKUP_S3_BUCKET         S3/MinIO bucket name, e.g. verdyx-backups
#   BACKUP_S3_PREFIX         key prefix within the bucket (default: pg)
#   BACKUP_S3_ENDPOINT       endpoint URL (MinIO, e.g. http://minio:9000).
#                            Omit for real AWS S3 (uses AWS_REGION).
#   AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY   credentials
#   AWS_REGION               (default: us-east-1) — AWS only
set -eu

HOST="${POSTGRES_HOST:-postgres}"
INTERVAL="${BACKUP_INTERVAL_SECONDS:-86400}"
RETENTION="${BACKUP_RETENTION_DAYS:-7}"
S3_BUCKET="${BACKUP_S3_BUCKET:-}"
S3_PREFIX="${BACKUP_S3_PREFIX:-pg}"
S3_ENDPOINT="${BACKUP_S3_ENDPOINT:-}"
DIR=/backups

log() { echo "[pg-backup] $(date -u +%Y-%m-%dT%H:%M:%SZ) $*"; }

# Verify a produced dump is well-formed. Returns non-zero if the file is
# truncated/corrupt or is missing the expected pg_dumpall markers.
verify_backup() {
  f="$1"
  if ! gzip -t "$f" 2>/dev/null; then
    log "VERIFY FAILED: gzip integrity check failed for $f"
    return 1
  fi
  # pg_dumpall emits this header; its absence means a partial/empty dump.
  if ! gzip -dc "$f" | grep -q "PostgreSQL database cluster dump"; then
    log "VERIFY FAILED: missing cluster-dump marker in $f (partial/empty dump?)"
    return 1
  fi
  # Sanity: at least one Verdyx database must appear in the dump.
  if ! gzip -dc "$f" | grep -qE "CREATE DATABASE verdyx"; then
    log "VERIFY FAILED: no 'CREATE DATABASE verdyx*' statements in $f"
    return 1
  fi
  log "verified: gzip ok + cluster/database markers present"
  return 0
}

# One-time MinIO-client (mc) setup for off-site upload. Installs the static
# binary if absent (image is postgres:16-alpine, which ships neither mc nor aws).
ensure_mc() {
  if command -v mc >/dev/null 2>&1; then return 0; fi
  log "installing mc (MinIO client) for off-site upload"
  arch="$(uname -m)"
  case "$arch" in
    x86_64) mcarch=amd64 ;;
    aarch64 | arm64) mcarch=arm64 ;;
    *) log "unsupported arch $arch for mc; skipping off-site upload"; return 1 ;;
  esac
  if wget -qO /usr/local/bin/mc "https://dl.min.io/client/mc/release/linux-${mcarch}/mc"; then
    chmod +x /usr/local/bin/mc
  else
    log "failed to download mc; skipping off-site upload"
    return 1
  fi
}

upload_offsite() {
  f="$1"
  [ -n "$S3_BUCKET" ] || return 0   # off-site disabled
  if [ -z "${AWS_ACCESS_KEY_ID:-}" ] || [ -z "${AWS_SECRET_ACCESS_KEY:-}" ]; then
    log "off-site: AWS_ACCESS_KEY_ID/SECRET not set; skipping upload"
    return 0
  fi
  ensure_mc || return 0
  endpoint="$S3_ENDPOINT"
  if [ -z "$endpoint" ]; then
    endpoint="https://s3.${AWS_REGION:-us-east-1}.amazonaws.com"
  fi
  if ! mc alias set verdyxbak "$endpoint" "$AWS_ACCESS_KEY_ID" "$AWS_SECRET_ACCESS_KEY" >/dev/null 2>&1; then
    log "off-site: failed to configure mc alias for $endpoint"
    return 1
  fi
  mc mb --ignore-existing "verdyxbak/${S3_BUCKET}" >/dev/null 2>&1 || true
  dest="verdyxbak/${S3_BUCKET}/${S3_PREFIX}/$(basename "$f")"
  if mc cp "$f" "$dest" >/dev/null 2>&1; then
    log "off-site: uploaded -> ${S3_BUCKET}/${S3_PREFIX}/$(basename "$f")"
  else
    log "off-site: upload FAILED for $f"
    return 1
  fi
}

take_backup() {
  ts="$(date -u +%Y%m%d-%H%M%S)"
  out="$DIR/verdyx-${ts}.sql.gz"
  log "dumping cluster from ${HOST} -> ${out}"
  if PGPASSWORD="$POSTGRES_PASSWORD" pg_dumpall -h "$HOST" -U "$POSTGRES_USER" --clean --if-exists \
      | gzip -6 > "${out}.tmp"; then
    mv "${out}.tmp" "$out"
    log "ok: $(du -h "$out" | cut -f1)"
  else
    log "FAILED"
    rm -f "${out}.tmp"
    return 1
  fi

  # A dump that does not verify is worthless — drop it and fail this cycle so
  # the failure is visible (monitoring/alerting) rather than silently retained.
  if ! verify_backup "$out"; then
    rm -f "$out"
    return 1
  fi

  upload_offsite "$out" || log "off-site upload failed (local copy retained)"

  # Prune old backups (keep at least the newest even if all are older).
  find "$DIR" -name 'verdyx-*.sql.gz' -type f -mtime "+${RETENTION}" -print -delete | while read -r f; do
    log "pruned $f"
  done
}

mkdir -p "$DIR"
log "starting; interval=${INTERVAL}s retention=${RETENTION}d off-site=${S3_BUCKET:-disabled}"

if [ "${BACKUP_ON_START:-1}" = "1" ]; then
  take_backup || log "initial backup failed (will retry next cycle)"
fi

while true; do
  sleep "$INTERVAL"
  take_backup || log "backup failed (will retry next cycle)"
done
