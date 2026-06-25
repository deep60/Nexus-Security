# Migration rollback strategy

This document describes how to roll back a SQL migration in any of the
9 Verdyx per-service databases. Migrations are managed by `sqlx::migrate!()`
embedded in each service binary and live under each service's
`migrations/<timestamp>_*.sql` folder.

## Why "down" migrations are not committed by default

`sqlx-cli` supports both `up` and `down` migration files. The Verdyx
codebase only commits `up` migrations because:

1. Most rollbacks in production are not "undo last migration" — they are
   "restore from a backup that pre-dates the bad change".
2. A reversible `down` script that loses data silently is more dangerous
   than no `down` at all. Forcing the operator to think reduces risk.

This document defines the explicit, manual procedure to use instead.

## Decision tree

```
Bad migration deployed?
├── Yes, and it dropped/renamed columns or destroyed data
│   └── Restore from backup (option A)
└── Yes, but it only added columns/tables/indexes and is safe to keep
    └── Forward-fix with a new migration (option B)
```

Avoid raw `DELETE FROM _sqlx_migrations` + manual `DROP` unless you have
verified the database state matches what you expect.

## Option A — Point-in-time restore from backup

Daily backups are produced by `scripts/maintenance/backup.sh` and contain
per-database `.sql` dumps. To restore:

```bash
# 1. Stop traffic to the affected service
docker compose stop <service>

# 2. Drop and recreate the affected database from the backup
./scripts/maintenance/restore.sh --backup backups/<TIMESTAMP> --yes

# 3. Replay only the migrations that were known-good at backup time.
#    sqlx::migrate!() is idempotent — re-starting the service does this
#    automatically.
docker compose up -d <service>
```

Because each service has its own database, only the affected service is
restored. Other services keep running unchanged.

## Option B — Forward-fix migration

Most schema mistakes are best fixed by adding a new migration that
corrects the previous one, rather than rolling back. This preserves a
clean linear history.

```bash
# 1. Add a new migration that reverses or repairs the bad change
cd backend/<service>
sqlx migrate add fix_<bad_migration_name>

# 2. Edit the new file. Examples:
#    - Drop a column that should not have been added
#    - Backfill a column with a default
#    - Recreate an index that was accidentally dropped

# 3. Test locally
sqlx migrate run --database-url "$DATABASE_URL"

# 4. Commit and deploy through the normal CI/CD path.
```

## Per-service migration directories

| Service              | Path                                                |
| -------------------- | --------------------------------------------------- |
| api-gateway          | `backend/api-gateway/migrations/`                   |
| user-service         | `backend/user-service/migrations/`                  |
| analysis-engine      | `backend/analysis-engine/migrations/`               |
| bounty-manager       | `backend/bounty-manager/migrations/`                |
| consensus-service    | `backend/consensus-service/migrations/`             |
| notification-service | `backend/notification-service/migrations/`          |
| payment-service      | `backend/payment-service/migrations/`               |
| reputation-service   | `backend/reputation-service/migrations/`            |
| submission-service   | `backend/submission-service/migrations/`            |

## Pre-deployment checklist for risky migrations

Before merging a migration that does any of the following, take a manual
backup and write a forward-fix plan:

- Drops a table or column
- Renames a column
- Adds a `NOT NULL` constraint without a default
- Changes a column type in a way that could fail to cast existing rows
- Adds a unique constraint to a column with potentially duplicate data

Manual pre-migration backup:

```bash
./scripts/maintenance/backup.sh
# Note the printed backup path. Keep it accessible during the rollout.
```

## Drift detection

To check that the DB schema matches what the binary expects:

```bash
docker compose exec <service> /usr/local/bin/app --check-migrations
```

(Service-specific flag — confirm in the service's `main.rs`.)

For services without that flag, compare `_sqlx_migrations` row count
against the number of `.sql` files under `migrations/`:

```bash
docker compose exec postgres \
  psql -U verdyx_user -d verdyx_<service> -c \
  "SELECT count(*) FROM _sqlx_migrations;"

ls backend/<service>/migrations/*.sql | wc -l
```

The two numbers should match.
