#!/bin/sh
# Nightly logical backup of the whole Postgres cluster (all Verdyx databases
# + roles) via pg_dumpall, gzipped, with local retention. Runs as a long-lived
# loop container; each cycle dumps then sleeps BACKUP_INTERVAL_SECONDS.
#
# Protects against the common data-loss causes: bad migration, accidental
# DROP, logical corruption. It does NOT protect against VM/disk loss on its
# own — sync /backups off-box for that (see infrastructure notes / README).
#
# Env:
#   POSTGRES_HOST            (default: postgres)
#   POSTGRES_USER            (required)
#   POSTGRES_PASSWORD        (required)
#   BACKUP_INTERVAL_SECONDS  (default: 86400 = daily)
#   BACKUP_RETENTION_DAYS    (default: 7)
#   BACKUP_ON_START          (default: 1 — take one immediately on boot)
set -eu

HOST="${POSTGRES_HOST:-postgres}"
INTERVAL="${BACKUP_INTERVAL_SECONDS:-86400}"
RETENTION="${BACKUP_RETENTION_DAYS:-7}"
DIR=/backups

log() { echo "[pg-backup] $(date -u +%Y-%m-%dT%H:%M:%SZ) $*"; }

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
  # Prune old backups (keep at least the newest even if all are older).
  find "$DIR" -name 'verdyx-*.sql.gz' -type f -mtime "+${RETENTION}" -print -delete | while read -r f; do
    log "pruned $f"
  done
}

mkdir -p "$DIR"
log "starting; interval=${INTERVAL}s retention=${RETENTION}d"

if [ "${BACKUP_ON_START:-1}" = "1" ]; then
  take_backup || log "initial backup failed (will retry next cycle)"
fi

while true; do
  sleep "$INTERVAL"
  take_backup || log "backup failed (will retry next cycle)"
done
