#!/usr/bin/env bash
# Verdyx preflight checks — run before `docker compose up` to catch the two
# silent failure modes that bit us:
#
#   1) postgres_data VOLUME drift — the volume was initialized with one
#      POSTGRES_PASSWORD, then .env was rotated. Postgres keeps the original
#      password forever and every service auth-errors with no hint of root
#      cause.
#
#   2) CONTAINER env drift — service containers created at first `compose up`
#      bake in the .env at-that-time. A later `compose up` (no --recreate)
#      restarts those containers with their original env, ignoring any .env
#      changes since then. Same auth-error symptom from a different cause.
#
# Usage:
#   scripts/preflight.sh             # check only, exit 1 on drift
#   scripts/preflight.sh --fix       # auto-recreate stale containers (safe);
#                                    # still refuses to touch the volume
#   scripts/preflight.sh --fix --force-volume
#                                    # NUCLEAR: also wipes postgres_data so
#                                    # postgres re-inits with current .env.
#                                    # YOU LOSE ALL DATA. Confirms first.
#
# Exit codes:
#   0  ok
#   1  drift detected (without --fix), or user declined a destructive fix
#   2  configuration error (missing .env, etc)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="${ENV_FILE:-$ROOT_DIR/.env}"
COMPOSE_PROJECT="${COMPOSE_PROJECT_NAME:-verdyx}"
PG_VOLUME="${COMPOSE_PROJECT}_postgres_data"
PG_CONTAINER="${COMPOSE_PROJECT}-postgres"
NETWORK_NAME="${COMPOSE_PROJECT}_${COMPOSE_PROJECT}-network"

FIX=0
FORCE_VOLUME=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --fix)          FIX=1; shift ;;
    --force-volume) FORCE_VOLUME=1; shift ;;
    -h|--help)
      sed -n '/^# Verdyx preflight/,/^set -e/p' "$0" | sed 's/^# \?//' | head -25
      exit 0 ;;
    *) echo "[preflight] unknown flag: $1" >&2; exit 2 ;;
  esac
done

log() { printf '[preflight] %s\n' "$*"; }
err() { printf '[preflight] ERROR: %s\n' "$*" >&2; }

# ── Step 0: .env present and required keys set ──────────────────────────────
if [[ ! -f "$ENV_FILE" ]]; then
  err ".env not found at $ENV_FILE"
  err "  cp .env.example .env  and fill in the secrets, then re-run."
  exit 2
fi

# Source the env file without polluting our shell — read the keys we care about.
read_env() {
  grep -E "^${1}=" "$ENV_FILE" | tail -1 | cut -d= -f2- | sed -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//"
}

POSTGRES_USER="$(read_env POSTGRES_USER)"
POSTGRES_PASSWORD="$(read_env POSTGRES_PASSWORD)"
POSTGRES_DB="$(read_env POSTGRES_DB)"
: "${POSTGRES_USER:=verdyx_user}"
: "${POSTGRES_DB:=verdyx}"

if [[ -z "$POSTGRES_PASSWORD" ]]; then
  err "POSTGRES_PASSWORD is empty in $ENV_FILE — set a value before bringing up the stack."
  exit 2
fi

# ── Step 1: postgres VOLUME drift detection ────────────────────────────────
# If the volume already exists, attempt a credential probe against the live
# postgres if it's running, otherwise spin up a throwaway postgres pointed at
# the volume and try.
log "checking postgres volume credential drift"
volume_exists=0
if docker volume inspect "$PG_VOLUME" >/dev/null 2>&1; then
  volume_exists=1
fi

# Returns 0 if creds in .env work against the existing volume, 1 if they
# don't (drift), 2 if the probe itself failed (don't make a determination).
#
# We probe via a SIDECAR psql container connected to the same docker network
# as the live postgres. Connecting from inside the postgres container itself
# would hit `host 127.0.0.1/32 trust` in the stock pg_hba.conf, which
# bypasses password auth and gives a false positive. From another container
# on the network, the connection goes through the IPv4 network → matches the
# `host all all all scram-sha-256` rule that actually checks the password.
probe_credentials() {
  local probe_id="verdyx-preflight-pgprobe-$$"
  local probe_out
  local probe_rc

  cleanup_probe() {
    docker rm -f "$probe_id" >/dev/null 2>&1 || true
  }
  trap cleanup_probe RETURN

  # If the project's postgres is running on its own network, probe through it.
  if docker ps --filter "name=^${PG_CONTAINER}$" --filter "status=running" -q | grep -q .; then
    if ! docker network inspect "$NETWORK_NAME" >/dev/null 2>&1; then
      return 2
    fi
    probe_out="$(docker run --rm \
        --name "$probe_id" \
        --network "$NETWORK_NAME" \
        -e PGPASSWORD="$POSTGRES_PASSWORD" \
        postgres:16-alpine \
        psql -h postgres -U "$POSTGRES_USER" -d "$POSTGRES_DB" -tAc "SELECT 1" 2>&1 || true)"
    probe_rc=$?
    if grep -q '^ *1 *$' <<<"$probe_out"; then
      return 0
    fi
    if grep -qi 'password authentication failed' <<<"$probe_out"; then
      return 1
    fi
    return 2
  fi

  # Pg not running: mount the volume into a one-shot pg, give it its own
  # ephemeral network, and probe the same way.
  local probe_net="verdyx-preflight-net-$$"
  docker network create "$probe_net" >/dev/null 2>&1 || true
  cleanup_probe() {
    docker rm -f "$probe_id" >/dev/null 2>&1 || true
    docker rm -f "${probe_id}-client" >/dev/null 2>&1 || true
    docker network rm "$probe_net" >/dev/null 2>&1 || true
  }

  if ! docker run -d --rm \
      --name "$probe_id" \
      --network "$probe_net" \
      --network-alias postgres \
      -v "${PG_VOLUME}:/var/lib/postgresql/data" \
      -e POSTGRES_PASSWORD="placeholder-unused" \
      postgres:16-alpine >/dev/null 2>&1; then
    return 2
  fi
  for _ in $(seq 1 30); do
    if docker exec "$probe_id" pg_isready -U "$POSTGRES_USER" >/dev/null 2>&1; then
      break
    fi
    sleep 1
  done

  probe_out="$(docker run --rm \
      --name "${probe_id}-client" \
      --network "$probe_net" \
      -e PGPASSWORD="$POSTGRES_PASSWORD" \
      postgres:16-alpine \
      psql -h postgres -U "$POSTGRES_USER" -d "$POSTGRES_DB" -tAc "SELECT 1" 2>&1 || true)"
  if grep -q '^ *1 *$' <<<"$probe_out"; then
    return 0
  fi
  if grep -qi 'password authentication failed' <<<"$probe_out"; then
    return 1
  fi
  return 2
}

volume_drift=0
if [[ $volume_exists -eq 1 ]]; then
  set +e
  probe_credentials
  probe_rc=$?
  set -e
  case $probe_rc in
    0) log "  ✓ postgres credentials in .env match the existing volume" ;;
    1) volume_drift=1
       err "  ✗ postgres VOLUME DRIFT detected."
       err "    The '$PG_VOLUME' volume was initialized with a DIFFERENT"
       err "    POSTGRES_PASSWORD than what is in .env right now. Postgres"
       err "    keeps the original password forever; every backend service"
       err "    will fail auth until this is reconciled."
       err ""
       err "    Pick one:"
       err "      a) Restore the original password in .env (preserves data)."
       err "      b) Re-init the volume (DESTROYS all DB data):"
       err "           scripts/preflight.sh --fix --force-volume"
       err "         or manually:"
       err "           docker compose down && docker volume rm $PG_VOLUME"
       ;;
    *) log "  ? could not probe postgres (skipping volume drift check)" ;;
  esac
else
  log "  · postgres volume not present yet; will be initialized fresh"
fi

# ── Step 2: per-container env drift detection ──────────────────────────────
# A container created N days ago has the .env from N days ago baked in. We
# spot this by comparing each container's env to the current .env values for
# the keys we control via env_file (POSTGRES_PASSWORD, REDIS_PASSWORD,
# MINIO_ROOT_PASSWORD).
log "checking container env drift"

# Build parallel arrays of (key, current value) for the env keys we
# control via env_file. We avoid associative arrays so this works on
# macOS's bundled bash 3.2.
watched_keys=(POSTGRES_PASSWORD REDIS_PASSWORD MINIO_ROOT_PASSWORD JWT_SECRET)
watched_values=()
for key in "${watched_keys[@]}"; do
  watched_values+=("$(read_env "$key")")
done

stale_containers=()
container_names=$(docker ps -a --filter "label=com.docker.compose.project=${COMPOSE_PROJECT}" \
                  --format '{{.Names}}')

for name in $container_names; do
  # Skip postgres itself (its password is locked in via the volume, not env).
  [[ "$name" == "$PG_CONTAINER" ]] && continue

  # Cache the container's env once per container rather than re-shelling
  # `docker inspect` per key.
  container_env=$(docker inspect "$name" --format \
    '{{range .Config.Env}}{{println .}}{{end}}' 2>/dev/null || true)

  drift_for_name=""
  for i in "${!watched_keys[@]}"; do
    key="${watched_keys[$i]}"
    want="${watched_values[$i]}"
    [[ -z "$want" ]] && continue
    have=$(printf '%s\n' "$container_env" | grep -E "^${key}=" | head -1 | cut -d= -f2- || true)
    # If the container doesn't have this env key, skip — it isn't expected to.
    [[ -z "$have" ]] && continue
    if [[ "$have" != "$want" ]]; then
      drift_for_name+="${key} "
    fi
  done

  if [[ -n "$drift_for_name" ]]; then
    stale_containers+=("$name")
    log "  ✗ ${name}: stale env vars [$(echo $drift_for_name | xargs)]"
  fi
done

if [[ ${#stale_containers[@]} -eq 0 ]]; then
  log "  ✓ no container env drift"
fi

# ── Step 3: decide & act ───────────────────────────────────────────────────
problems=0
[[ $volume_drift -eq 1 ]] && problems=$((problems + 1))
[[ ${#stale_containers[@]} -gt 0 ]] && problems=$((problems + 1))

if [[ $problems -eq 0 ]]; then
  log "preflight OK"
  exit 0
fi

if [[ $FIX -eq 0 ]]; then
  err ""
  err "preflight FAILED. Re-run with --fix to auto-recreate stale containers,"
  err "or with --fix --force-volume to also wipe the postgres volume."
  exit 1
fi

# ── --fix path ─────────────────────────────────────────────────────────────
if [[ ${#stale_containers[@]} -gt 0 ]]; then
  log "fixing: recreating stale containers"
  for name in "${stale_containers[@]}"; do
    svc="${name#${COMPOSE_PROJECT}-}"
    log "  recreating $svc"
    (cd "$ROOT_DIR" && docker compose up -d --no-deps --force-recreate "$svc" \
      >/dev/null 2>&1) || err "    failed to recreate $svc"
  done
fi

if [[ $volume_drift -eq 1 ]]; then
  if [[ $FORCE_VOLUME -ne 1 ]]; then
    err "postgres volume still has drift. Refusing to wipe without --force-volume."
    exit 1
  fi
  printf "\n[preflight] About to PERMANENTLY DELETE the postgres volume '%s'.\n" "$PG_VOLUME"
  printf "[preflight] All databases (verdyx, verdyx_users, verdyx_bounty, etc.) will be lost.\n"
  printf "[preflight] Type 'wipe' to confirm, anything else aborts: "
  read -r confirmation
  if [[ "$confirmation" != "wipe" ]]; then
    err "aborted by user"
    exit 1
  fi
  log "stopping stack"
  (cd "$ROOT_DIR" && docker compose down >/dev/null 2>&1) || true
  log "removing volume $PG_VOLUME"
  docker volume rm "$PG_VOLUME" >/dev/null
  log "starting stack fresh (postgres will re-init from current .env)"
  (cd "$ROOT_DIR" && docker compose up -d --wait --wait-timeout 300) || \
    err "stack did not become healthy within 300s; check 'docker compose logs'"
fi

log "preflight --fix done"
