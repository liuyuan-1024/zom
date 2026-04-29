#!/usr/bin/env bash
set -euo pipefail

# Ensure icon asset paths are centralized in src/icon.rs.
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TARGET_DIR="crates/zom-gpui/src"
ALLOWLIST_FILE="crates/zom-gpui/src/icon.rs"

if ! command -v rg >/dev/null 2>&1; then
  echo "ripgrep (rg) is required for check-icon-centralization.sh" >&2
  exit 2
fi

PATTERN='"icons/[^"]+"|\.path\("icons/|\.icon\("icons/'

violations="$(
  (
    rg -n --no-heading --glob '*.rs' "$PATTERN" "$TARGET_DIR" \
      | rg -v "^${ALLOWLIST_FILE}:"
  ) || true
)"

if [[ -n "$violations" ]]; then
  echo "Found non-centralized icon path usage. Move mappings to ${ALLOWLIST_FILE}." >&2
  echo "$violations" >&2
  exit 1
fi

echo "Icon centralization check passed."
