# Postgres password rotation runbook

## TL;DR

```bash
# After editing POSTGRES_PASSWORD in .env, run:
scripts/preflight.sh --fix              # heals stale containers, refuses to wipe the DB
scripts/preflight.sh --fix --force-volume   # also wipes postgres_data — DESTROYS ALL DATA
```

If you only want to **detect** drift without changing anything:

```bash
scripts/preflight.sh
```

## Why this exists

Docker postgres has two silent failure modes that look identical from the
backend (`password authentication failed for user "verdyx_user"` crash loop):

### 1. Volume drift

The `postgres_data` named volume locks in `POSTGRES_PASSWORD` on **first
init**. Rotating the value in `.env` does **not** update the password inside
the volume — postgres keeps the original forever. Every backend service then
tries the new password and gets rejected.

Symptoms: postgres itself is healthy, but every other service crash-loops with
auth errors.

### 2. Container env drift

`env_file:` in docker-compose is read **when the container is created**, not
on every restart. A container created on day 0 keeps day-0 secrets across
restarts. Rotating `.env` after that point has no effect on the existing
container — the container needs to be `--force-recreate`'d for the new env to
take effect.

Symptoms: some services authenticate fine, others don't, with no obvious
pattern. The "others" are the ones whose containers were created earlier than
the most recent `.env` change.

## What `preflight.sh` does

| Phase | Check | Action on failure |
|-------|-------|-------------------|
| Volume drift | Probe the existing `postgres_data` volume from a sidecar `psql` container on the verdyx network using `.env` creds | Print actionable error; never auto-wipe without `--force-volume` |
| Container env drift | `docker inspect` each `verdyx-*` container, compare `POSTGRES_PASSWORD`, `REDIS_PASSWORD`, `MINIO_ROOT_PASSWORD`, `JWT_SECRET` against current `.env` | With `--fix`: `docker compose up -d --force-recreate` each stale container |

The probe runs from a sidecar specifically because Postgres's stock
`pg_hba.conf` includes `host all all 127.0.0.1/32 trust`, so probing from
inside the postgres container would always succeed (a false negative).

## Common scenarios

### "I just rotated POSTGRES_PASSWORD in `.env`"

```bash
scripts/preflight.sh --fix
```

This recreates every stale service container against the new `.env`. If your
postgres volume was already initialized with the **old** password, you'll see
a "VOLUME DRIFT detected" error and the script will refuse to proceed. You
have two options:

- Restore the old password in `.env` (preserves data).
- Wipe the volume and re-init from scratch (loses data — use only in dev):

  ```bash
  scripts/preflight.sh --fix --force-volume
  ```

  This prompts for confirmation (`type 'wipe' to confirm`) and then runs
  `docker compose down` + `docker volume rm verdyx_postgres_data` +
  `docker compose up -d --wait`.

### "I want to keep the data but change the password"

The volume's password is just `verdyx_user`'s row in `pg_authid`. Rotate it
in-place rather than wiping:

```bash
# 1. With the OLD password still in .env so services can connect:
docker compose exec postgres psql -U verdyx_user -d verdyx -c \
  "ALTER USER verdyx_user WITH PASSWORD 'new-strong-pw-here';"

# 2. Update .env to the new value:
sed -i.bak 's/^POSTGRES_PASSWORD=.*/POSTGRES_PASSWORD=new-strong-pw-here/' .env

# 3. Recreate stale containers so they pick up the new env:
scripts/preflight.sh --fix
```

### "CI is failing with auth errors after a `.env.example` change"

Most likely a test runner is using a stale postgres volume from a previous
CI run. In CI, always start with a clean volume:

```bash
docker compose down -v
docker compose up -d --wait
```

`down -v` removes named volumes. Safe in CI; dangerous on a real machine
(loses local dev data).

## Integration with smoke tests

`scripts/test/smoke.sh` runs `preflight.sh` automatically before `docker
compose up`. To skip preflight (e.g., when you've already validated manually):

```bash
SKIP_PREFLIGHT=1 scripts/test/smoke.sh
```

## Prevention

- Never edit `.env` and leave the stack running. Either restart the affected
  services (`docker compose up -d --force-recreate <svc>`) or run preflight.
- For real password rotations on a populated database, prefer the
  in-place `ALTER USER` path above. Wiping the volume is for dev only.
- In production we don't use the `postgres_data` named volume at all —
  postgres is managed (RDS / Cloud SQL / etc.). The rotation procedure there
  is whatever the managed service offers; this runbook is for the docker
  compose local/staging stack.
