#!/usr/bin/env bash
set -euo pipefail

TARGET_URL="http://localhost:8080/api/v1/health"
DURATION="30s"
VUS="10"

usage() {
  cat <<USAGE
Usage: scripts/testing/load-test.sh [options]

Options:
  --url <url>            Target URL (default: http://localhost:8080/api/v1/health)
  --duration <value>     Test duration, e.g. 30s, 2m (default: 30s)
  --vus <number>         Virtual users for k6 (default: 10)
  -h, --help             Show this help message
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --url)
      TARGET_URL="$2"
      shift 2
      ;;
    --duration)
      DURATION="$2"
      shift 2
      ;;
    --vus)
      VUS="$2"
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

if command -v k6 >/dev/null 2>&1; then
  TMP_SCRIPT="$(mktemp)"
  cat > "$TMP_SCRIPT" <<K6
import http from 'k6/http';
import { check } from 'k6';

export const options = {
  vus: ${VUS},
  duration: '${DURATION}',
};

export default function () {
  const res = http.get('${TARGET_URL}');
  check(res, {
    'status is < 500': (r) => r.status < 500,
  });
}
K6

  echo "[info] Running k6 load test against ${TARGET_URL}"
  k6 run "$TMP_SCRIPT"
  rm -f "$TMP_SCRIPT"
  echo "[ok] Load test complete"
  exit 0
fi

if command -v ab >/dev/null 2>&1; then
  echo "[warn] k6 not found, falling back to ApacheBench"
  echo "[info] ab -n 500 -c 20 ${TARGET_URL}"
  ab -n 500 -c 20 "$TARGET_URL"
  echo "[ok] Load test complete"
  exit 0
fi

echo "[error] Neither k6 nor ab is installed"
echo "Install one of:"
echo "  brew install k6"
echo "  brew install httpd"
exit 1
