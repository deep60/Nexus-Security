# ⚠️ LEGACY — NOT THE LIVE SCHEMA

These `00X_*.sql` files describe an earlier **monolithic single-database** design
and are **not executed at runtime** by any Verdyx service.

The live schema is owned per-service:

- `backend/api-gateway/migrations/`
- `backend/user-service/migrations/`
- `backend/analysis-engine/migrations/`
- `backend/bounty-manager/migrations/`
- `backend/submission-service/migrations/`
- `backend/consensus-service/migrations/`
- `backend/payment-service/migrations/`
- `backend/reputation-service/migrations/`
- `backend/notification-service/migrations/`

Each service runs its own migrations via `sqlx::migrate!` against its own
database (created by `database/init/01-init-databases.sql`).

Keep these files only for historical/domain reference. To change a table, edit
the owning service's `migrations/` folder instead. Do not add new migrations
here expecting them to run.
