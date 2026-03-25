# Testing Guide

Use the scripts in `scripts/testing` for common workflows.

## 1. Full Test Run

```bash
scripts/testing/run-tests.sh
```

This runs:

- Backend workspace tests (`cargo test --workspace`)
- Frontend tests (`npm run test:run`)
- Blockchain tests (`npm test` in `blockchain`)

## 2. Targeted Runs

```bash
scripts/testing/run-tests.sh --backend-only
scripts/testing/run-tests.sh --frontend-only
scripts/testing/run-tests.sh --blockchain-only
```

## 3. Security Scan

```bash
scripts/testing/security-scan.sh
```

Strict mode fails when optional tools are missing:

```bash
scripts/testing/security-scan.sh --strict
```

## 4. Load Testing

```bash
scripts/testing/load-test.sh --url http://localhost:8080/api/v1/health --duration 30s --vus 10
```

`k6` is preferred. `ab` is used as fallback if installed.

## 5. Troubleshooting Test Failures

- Verify services are up: `scripts/deployment/health-check.sh`
- Reset DB state if needed: `scripts/development/reset-db.sh --with-seed`
- Re-run only the failing suite first to isolate scope.
