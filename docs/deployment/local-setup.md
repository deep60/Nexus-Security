# Local Setup

This guide uses the scripts under `scripts/development` and `scripts/deployment`.

## Prerequisites

- Docker and Docker Compose plugin
- Rust toolchain (for backend)
- Node.js 18+
- npm

## 1. Initial Bootstrap

From repo root:

```bash
scripts/development/setup.sh
```

Optional flags:

- `--skip-install`: skip npm/cargo install steps.
- `--start-docker`: start postgres/redis after setup.

## 2. Start Services

Development profile:

```bash
scripts/development/start-services.sh --profile dev
```

Full stack:

```bash
scripts/development/start-services.sh --profile full
```

Infra-only (postgres/redis/minio/clamav):

```bash
scripts/development/start-services.sh --profile infra
```

## 3. Verify Health

```bash
scripts/deployment/health-check.sh
```

## 4. Reset and Seed Database (Optional)

```bash
scripts/development/reset-db.sh --with-seed
```

Generate seed data only:

```bash
scripts/development/generate-test-data.sh
```

## 5. Run Test Suites

```bash
scripts/testing/run-tests.sh
```

Target one suite:

```bash
scripts/testing/run-tests.sh --frontend-only
```

## Common Local Endpoints

- Frontend: `http://localhost:5000`
- API Gateway: `http://localhost:8080/api/v1`
- API Health: `http://localhost:8080/api/v1/health`
