#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Load signing key from env or fall back to ~/.tauri/blank.key
if [ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" ]; then
  KEY_FILE="$HOME/.tauri/blank.key"
  if [ -f "$KEY_FILE" ]; then
    export TAURI_SIGNING_PRIVATE_KEY="$(cat "$KEY_FILE")"
    export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}"
  else
    echo "error: TAURI_SIGNING_PRIVATE_KEY is not set and $KEY_FILE does not exist." >&2
    exit 1
  fi
fi

export SDKROOT
SDKROOT="$(xcrun --show-sdk-path 2>/dev/null || true)"

CMD="${1:-}"

case "$CMD" in
  dev)
    npm run tauri dev
    ;;
  build)
    npm run tauri build
    ;;
  check)
    npm run check
    ;;
  "")
    echo "Usage: $0 {dev|build|check}" >&2
    exit 1
    ;;
  *)
    echo "Unknown command: $CMD" >&2
    echo "Usage: $0 {dev|build|check}" >&2
    exit 1
    ;;
esac
