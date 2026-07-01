#!/usr/bin/env bash
#
# Create the per-service test databases and apply each service's migrations.
#
# The backend runs one database per microservice (see
# database/init/01-init-databases.sql), and every service owns the migrations
# under backend/<service>/migrations. CI previously started Postgres but never
# applied any migrations, so a build with a broken or missing schema could pass
# `cargo test` (which touched no tables) and only fail at runtime with
# `relation "..." does not exist`.
#
# This script closes that gap: it creates each service DB, installs the shared
# extensions, and runs `sqlx migrate run` against it. A broken migration now
# fails CI here, before any test runs.
#
# Configuration (defaults match .github/workflows/rust.yml's postgres service):
#   PGHOST     (default: localhost)
#   PGPORT     (default: 5432)
#   PGUSER     (default: test_user)
#   PGPASSWORD (default: test_password)
#
# It also writes a `.ci-test-db-env` file with the per-service DATABASE_URLs so
# later steps can `source` it.
set -euo pipefail

PGHOST="${PGHOST:-localhost}"
PGPORT="${PGPORT:-5432}"
PGUSER="${PGUSER:-test_user}"
PGPASSWORD="${PGPASSWORD:-test_password}"
export PGPASSWORD

# Resolve the repo's backend directory relative to this script so the script
# works regardless of the caller's working directory.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BACKEND_DIR="${REPO_ROOT}/backend"

# service_directory:database_name
SERVICES=(
  "api-gateway:verdyx_gateway"
  "user-service:verdyx_users"
  "analysis-engine:verdyx_analysis"
  "bounty-manager:verdyx_bounty"
  "submission-service:verdyx_submissions"
  "consensus-service:verdyx_consensus"
  "payment-service:verdyx_payments"
  "reputation-service:verdyx_reputation"
  "notification-service:verdyx_notifications"
)

psql_admin() {
  psql -v ON_ERROR_STOP=1 -h "${PGHOST}" -p "${PGPORT}" -U "${PGUSER}" "$@"
}

echo "==> Waiting for Postgres at ${PGHOST}:${PGPORT} ..."
for _ in $(seq 1 30); do
  if pg_isready -h "${PGHOST}" -p "${PGPORT}" -U "${PGUSER}" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
pg_isready -h "${PGHOST}" -p "${PGPORT}" -U "${PGUSER}"

ENV_FILE="${REPO_ROOT}/.ci-test-db-env"
: > "${ENV_FILE}"

for entry in "${SERVICES[@]}"; do
  service="${entry%%:*}"
  dbname="${entry##*:}"
  migrations_dir="${BACKEND_DIR}/${service}/migrations"

  if [ ! -d "${migrations_dir}" ]; then
    echo "!! No migrations directory for ${service} (${migrations_dir}); skipping"
    continue
  fi

  echo "==> [${service}] creating database ${dbname}"
  # CREATE DATABASE cannot run inside a transaction and has no IF NOT EXISTS,
  # so guard it with a catalog check (safe to re-run locally).
  if ! psql_admin -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname = '${dbname}'" | grep -q 1; then
    psql_admin -d postgres -c "CREATE DATABASE ${dbname};"
  fi

  echo "==> [${service}] installing extensions in ${dbname}"
  psql_admin -d "${dbname}" -c "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";" \
                             -c "CREATE EXTENSION IF NOT EXISTS \"pgcrypto\";" \
                             -c "CREATE EXTENSION IF NOT EXISTS \"pg_trgm\";"

  db_url="postgres://${PGUSER}:${PGPASSWORD}@${PGHOST}:${PGPORT}/${dbname}"
  echo "==> [${service}] applying migrations from ${migrations_dir}"
  sqlx migrate run --source "${migrations_dir}" --database-url "${db_url}"

  # Export a per-service DATABASE_URL for the test step, e.g.
  # USER_SERVICE_DATABASE_URL, SUBMISSION_SERVICE_DATABASE_URL.
  var_name="$(echo "${service}" | tr '[:lower:]-' '[:upper:]_')_DATABASE_URL"
  echo "${var_name}=${db_url}" >> "${ENV_FILE}"
done

echo "==> All service databases created and migrated."
echo "==> Per-service DATABASE_URLs written to ${ENV_FILE}"
