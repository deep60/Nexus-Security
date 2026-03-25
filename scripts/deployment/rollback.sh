#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET_REF=""
COMPOSE_FILE="docker-compose.yml"
ENV_FILE=".env"
SERVICES=""

usage() {
  cat <<USAGE
Usage: scripts/deployment/rollback.sh --to <git-ref> [options]

Options:
  --to <git-ref>          Git ref to rollback to (required)
  --compose-file <file>   Compose file in target ref (default: docker-compose.yml)
  --env-file <file>       Env file path in target ref (default: .env)
  --services "svc1 svc2"  Roll back only selected services
  -h, --help              Show this help message

Example:
  scripts/deployment/rollback.sh --to v0.1.0 --services "api-gateway user-service"
USAGE
}

compose_cmd() {
  if command -v docker-compose >/dev/null 2>&1; then
    echo "docker-compose"
  else
    echo "docker compose"
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --to)
      TARGET_REF="$2"
      shift 2
      ;;
    --compose-file)
      COMPOSE_FILE="$2"
      shift 2
      ;;
    --env-file)
      ENV_FILE="$2"
      shift 2
      ;;
    --services)
      SERVICES="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[error] Unknown option: $1"
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$TARGET_REF" ]]; then
  echo "[error] --to <git-ref> is required"
  usage
  exit 1
fi

cd "$ROOT_DIR"

if ! git rev-parse --verify "$TARGET_REF" >/dev/null 2>&1; then
  echo "[error] Invalid git ref: $TARGET_REF"
  exit 1
fi

WORKTREE_DIR="$(mktemp -d)"
cleanup() {
  git worktree remove "$WORKTREE_DIR" --force >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "[info] Creating temporary worktree for $TARGET_REF"
git worktree add --detach "$WORKTREE_DIR" "$TARGET_REF" >/dev/null

if [[ ! -f "$WORKTREE_DIR/$COMPOSE_FILE" ]]; then
  echo "[error] Compose file not found in target ref: $COMPOSE_FILE"
  exit 1
fi

COMPOSE="$(compose_cmd)"

echo "[info] Deploying rollback target: $TARGET_REF"
(
  cd "$WORKTREE_DIR"
  $COMPOSE --env-file "$ENV_FILE" -f "$COMPOSE_FILE" up -d --build $SERVICES
)

echo "[info] Running health checks after rollback"
bash "$ROOT_DIR/scripts/deployment/health-check.sh" --compose-file "$WORKTREE_DIR/$COMPOSE_FILE"

echo "[ok] Rollback completed to $TARGET_REF"
