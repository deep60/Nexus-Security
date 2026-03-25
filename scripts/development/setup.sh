#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SKIP_INSTALL=0
START_DOCKER=0

usage() {
  cat <<USAGE
Usage: scripts/development/setup.sh [options]

Options:
  --skip-install    Skip npm/cargo dependency setup
  --start-docker    Start postgres and redis after setup
  -h, --help        Show this help message
USAGE
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[error] Required command not found: $1"
    exit 1
  fi
}

compose_cmd() {
  if command -v docker-compose >/dev/null 2>&1; then
    echo "docker-compose"
  else
    echo "docker compose"
  fi
}

for arg in "$@"; do
  case "$arg" in
    --skip-install) SKIP_INSTALL=1 ;;
    --start-docker) START_DOCKER=1 ;;
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

require_cmd docker
require_cmd node
require_cmd npm
require_cmd cargo

if [[ ! -f .env ]]; then
  if [[ -f .env.example ]]; then
    cp .env.example .env
    echo "[ok] Created .env from .env.example"
  else
    echo "[warn] .env.example not found, skipped env bootstrap"
  fi
else
  echo "[ok] .env already exists"
fi

if [[ "$SKIP_INSTALL" -eq 0 ]]; then
  echo "[info] Installing frontend dependencies"
  (cd frontend && npm install)

  echo "[info] Installing blockchain dependencies"
  (cd blockchain && npm install)

  echo "[info] Fetching backend Rust dependencies"
  (cd backend && cargo fetch)
else
  echo "[info] Dependency installation skipped"
fi

if [[ "$START_DOCKER" -eq 1 ]]; then
  COMPOSE="$(compose_cmd)"
  echo "[info] Starting postgres and redis via docker-compose.dev.yml"
  $COMPOSE -f docker-compose.dev.yml up -d postgres redis
fi

echo "[ok] Development setup complete"
