#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUN_BACKEND=1
RUN_FRONTEND=1
RUN_BLOCKCHAIN=1

usage() {
  cat <<USAGE
Usage: scripts/testing/run-tests.sh [options]

Options:
  --backend-only         Run only backend (cargo) tests
  --frontend-only        Run only frontend (vitest) tests
  --blockchain-only      Run only blockchain (hardhat) tests
  --skip-backend         Skip backend tests
  --skip-frontend        Skip frontend tests
  --skip-blockchain      Skip blockchain tests
  -h, --help             Show this help message
USAGE
}

for arg in "$@"; do
  case "$arg" in
    --backend-only)
      RUN_BACKEND=1
      RUN_FRONTEND=0
      RUN_BLOCKCHAIN=0
      ;;
    --frontend-only)
      RUN_BACKEND=0
      RUN_FRONTEND=1
      RUN_BLOCKCHAIN=0
      ;;
    --blockchain-only)
      RUN_BACKEND=0
      RUN_FRONTEND=0
      RUN_BLOCKCHAIN=1
      ;;
    --skip-backend) RUN_BACKEND=0 ;;
    --skip-frontend) RUN_FRONTEND=0 ;;
    --skip-blockchain) RUN_BLOCKCHAIN=0 ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[error] Unknown option: $arg"
      usage
      exit 1
      ;;
  esac
done

cd "$ROOT_DIR"
FAILED=0

if [[ "$RUN_BACKEND" -eq 1 ]]; then
  echo "[info] Running backend tests (cargo test)"
  if ! (cd backend && cargo test --workspace); then
    echo "[fail] Backend tests failed"
    FAILED=1
  fi
fi

if [[ "$RUN_FRONTEND" -eq 1 ]]; then
  echo "[info] Running frontend tests (npm run test:run)"
  if ! (cd frontend && npm run test:run); then
    echo "[fail] Frontend tests failed"
    FAILED=1
  fi
fi

if [[ "$RUN_BLOCKCHAIN" -eq 1 ]]; then
  echo "[info] Running blockchain tests (npm test)"
  if ! (cd blockchain && npm test); then
    echo "[fail] Blockchain tests failed"
    FAILED=1
  fi
fi

if [[ "$FAILED" -ne 0 ]]; then
  echo "[fail] One or more test suites failed"
  exit 1
fi

echo "[ok] All selected test suites passed"
