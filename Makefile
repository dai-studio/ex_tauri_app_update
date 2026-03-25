.PHONY: dev build check

# Load signing key from env or fall back to ~/.tauri/blank.key
TAURI_SIGNING_PRIVATE_KEY ?= $(shell \
  if [ -n "$$TAURI_SIGNING_PRIVATE_KEY" ]; then \
    echo "$$TAURI_SIGNING_PRIVATE_KEY"; \
  elif [ -f "$$HOME/.tauri/blank.key" ]; then \
    cat "$$HOME/.tauri/blank.key"; \
  else \
    echo ""; \
  fi)

SDKROOT ?= $(shell xcrun --show-sdk-path 2>/dev/null || true)

export TAURI_SIGNING_PRIVATE_KEY
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD ?=
export SDKROOT

dev:
	npm run tauri dev

build:
	@if [ -z "$(TAURI_SIGNING_PRIVATE_KEY)" ]; then \
	  echo "error: TAURI_SIGNING_PRIVATE_KEY is not set and ~/.tauri/blank.key does not exist." >&2; \
	  exit 1; \
	fi
	npm run tauri build

check:
	npm run check
