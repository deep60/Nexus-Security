#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STRICT=0
FAILED=0

usage() {
  cat <<USAGE
Usage: scripts/testing/security-scan.sh [options]

Options:
  --strict            Fail if optional tools are missing
  -h, --help          Show this help message
USAGE
}

run_or_warn() {
  local label="$1"
  shift

  echo "[info] $label"
  if "$@"; then
    echo "[ok]   $label"
  else
    echo "[fail] $label"
    FAILED=1
  fi
}

require_or_skip() {
  local cmd="$1"
  if command -v "$cmd" >/dev/null 2>&1; then
    return 0
  fi

  if [[ "$STRICT" -eq 1 ]]; then
    echo "[fail] Missing required tool in strict mode: $cmd"
    FAILED=1
  else
    echo "[warn] Missing tool, skipped: $cmd"
  fi

  return 1
}

for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
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

if require_or_skip cargo; then
  run_or_warn "Rust audit (cargo audit)" bash -lc 'cd backend && cargo audit'
fi

if require_or_skip npm; then
  run_or_warn "Frontend npm audit" bash -lc 'cd frontend && npm audit --audit-level=high'
  run_or_warn "Blockchain npm audit" bash -lc 'cd blockchain && npm audit --audit-level=high'
fi

if require_or_skip trivy; then
  run_or_warn "Filesystem vulnerability scan (trivy fs .)" trivy fs .
fi

if [[ "$FAILED" -ne 0 ]]; then
  echo "[fail] Security scan completed with failures"
  exit 1
fi

echo "[ok] Security scan completed"
