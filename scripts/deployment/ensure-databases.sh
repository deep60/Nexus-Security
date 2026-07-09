#!/usr/bin/env bash
#
# Ensure the per-service databases and their extensions exist on the target
# PostgreSQL instance, then verify each service's migrations are applied.
#
# WHY THIS EXISTS
#   Every backend service runs `sqlx::migrate!("./migrations")` at startup, so
#   the *schema* is applied automatically when a container boots. But sqlx only
#   creates tables inside an EXISTING database — it does not `CREATE DATABASE`.
#
#   In dev, the self-hosted Postgres creates the per-service databases via
#   database/init/01-init-databases.sql (a docker-entrypoint-initdb.d hook).
#   Managed RDS has NO such hook, so without this script the services would
#   crash-loop with `database "verdyx_gateway" does not exist`.
#
#   This script is idempotent — safe to run on every deploy. Run it BEFORE
#   `docker compose up` so the databases exist when the services start and
#   self-migrate.
#
# CONNECTION
#   Provide admin/superuser connection to the RDS instance's default database
#   via standard libpq env vars (matches the Terraform RDS module defaults):
#     PGHOST      RDS endpoint host          (required)
#     PGPORT      default 5432
#     PGUSER      RDS master username         (default: verdyx)
#     PGPASSWORD  RDS master password         (required)
#     PGADMINDB   admin/default database name (default: verdyx)
#
# USAGE
#   PGHOST=verdyx-postgres.xxxx.us-east-1.rds.amazonaws.com \
#   PGUSER=verdyx PGPASSWORD=... \
#   scripts/deployment/ensure-databases.sh
#
set -euo pipefail

PGHOST="${PGHOST:?PGHOST (RDS endpoint) is required}"
PGPORT="${PGPORT:-5432}"
PGUSER="${PGUSER:-verdyx}"
PGPASSWORD="${PGPASSWORD:?PGPASSWORD is required}"
PGADMINDB="${PGADMINDB:-verdyx}"
export PGPASSWORD

# Per-service databases — kept in sync with database/init/01-init-databases.sql
# and scripts/ci/setup-test-databases.sh.
DATABASES=(
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

EXTENSIONS=("uuid-ossp" "pg_trgm" "pgcrypto")

# Use a local psql if available; otherwise fall back to a throwaway container
# (the VM is guaranteed to have Docker, not necessarily postgresql-client).
if command -v psql >/dev/null 2>&1; then
  psql_run() { psql -v ON_ERROR_STOP=1 -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" "$@"; }
else
  echo "==> psql not found locally; using dockerized postgres:16-alpine client"
  psql_run() {
    docker run --rm -e PGPASSWORD="$PGPASSWORD" postgres:16-alpine \
      psql -v ON_ERROR_STOP=1 -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" "$@"
  }
fi

echo "==> Ensuring databases on ${PGHOST}:${PGPORT} (admin db: ${PGADMINDB})"

for db in "${DATABASES[@]}"; do
  # CREATE DATABASE cannot run in a transaction and has no IF NOT EXISTS, so
  # guard it with a catalog check.
  if psql_run -d "$PGADMINDB" -tAc "SELECT 1 FROM pg_database WHERE datname = '${db}'" | grep -q 1; then
    echo "==> [${db}] already exists"
  else
    echo "==> [${db}] creating"
    psql_run -d "$PGADMINDB" -c "CREATE DATABASE ${db} OWNER ${PGUSER};"
  fi

  # Extensions are per-database; installing is idempotent.
  for ext in "${EXTENSIONS[@]}"; do
    psql_run -d "$db" -c "CREATE EXTENSION IF NOT EXISTS \"${ext}\";" >/dev/null
  done
  echo "==> [${db}] extensions ready (${EXTENSIONS[*]})"
done

echo "==> All per-service databases and extensions are present."
echo "==> Services will apply their own migrations at startup (sqlx::migrate!)."
