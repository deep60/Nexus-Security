# Frontend BFF (Backend-for-Frontend)

This directory hosts a thin Express layer in front of the React app. In
production it does two things only:

1. Serves the built static client (handled in `index.ts`).
2. Proxies every `/api/*` request to the Rust **api-gateway**
   (see `routes.ts`). The gateway is the single source of truth for data;
   **the BFF never reads or writes the database directly**.

## Files at a glance

| File | Role | Used at runtime? |
|------|------|------------------|
| `index.ts` | Express entrypoint, request logging, vite/serve-static glue | yes |
| `routes.ts` | `/api/*` and `/ws` proxy to the Rust gateway | yes |
| `vite.ts` | Dev-mode Vite middleware | dev only |
| `config.ts` | Env-var loader (`PORT`, `API_GATEWAY_URL`, etc.) | yes |
| `db.ts`, `pg-storage.ts`, `storage.ts` | **Test scaffolding only** — Drizzle/in-memory mock storage used by `frontend/tests/` so the test suite can run without a live gateway. Production never imports these. | **no** (tests only) |

## Why the Drizzle/Postgres code exists if the BFF doesn't use it

`frontend/tests/test-routes.ts` mounts a local Express app backed by
`MemStorage` (`server/storage.ts`) so API integration tests don't need the
Rust gateway running. The Drizzle `pg-storage.ts` is a parallel
implementation that exercises the same `IStorage` interface against a real
Postgres for end-to-end testing. Neither file is reachable from `index.ts`
or `routes.ts`.

If you remove or rewrite the test suite, these files can be deleted along
with `drizzle.config.ts` and the Drizzle dependencies in `package.json`.
