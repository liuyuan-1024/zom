#!/usr/bin/env bash
set -euo pipefail

# Check workspace-internal crate dependencies against architecture boundary rules.
# Usage:
#   ./scripts/check-boundaries.sh

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required" >&2
  exit 2
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required (brew install jq)" >&2
  exit 2
fi

# Keep this mapping aligned with docs/开发规范手册.md -> "Crate 边界契约"
# Current enforced architecture:
#   zom-protocol(含 input) -> zom-text -> zom-editor -> zom-workspace -> zom-runtime -> zom-gpui -> apps/zom-desktop
# Optional crates (e.g. zom-workspace, zom-editor) must still obey single-direction dependencies.
allowed_deps_for() {
  case "$1" in
    zom-protocol) echo "" ;;
    zom-text) echo "zom-protocol" ;;
    zom-editor) echo "zom-protocol zom-text" ;;
    zom-workspace) echo "zom-protocol zom-text zom-editor" ;;
    zom-runtime) echo "zom-protocol zom-text zom-editor zom-workspace" ;;
    zom-gpui) echo "zom-protocol zom-runtime" ;;
    zom-desktop) echo "zom-runtime zom-gpui" ;;
    *) return 1 ;;
  esac
}

metadata="$(cargo metadata --format-version 1 --no-deps)"

workspace_pkgs="$(printf '%s' "$metadata" | jq -r '.packages[].name' | sort -u)"

errors=0

while IFS= read -r src; do
  [[ -z "$src" ]] && continue
  if ! allowed_deps_for "$src" >/dev/null; then
    echo "boundary error: crate '$src' has no boundary rule; add it in scripts/check-boundaries.sh" >&2
    errors=$((errors + 1))
  fi
done <<< "$workspace_pkgs"

internal_edges="$(
  printf '%s' "$metadata" \
    | jq -r '
      [.packages[].name] as $ws
      | .packages[] as $p
      | ($p.dependencies[]? | (.package // .name)) as $dep
      | select($ws | index($dep))
      | "\($p.name) \($dep)"
    ' \
    | sort -u
)"

is_allowed_dep() {
  local src="$1"
  local dst="$2"

  local allowed_list
  if ! allowed_list="$(allowed_deps_for "$src")"; then
    return 1
  fi
  for dep in $allowed_list; do
    if [[ "$dep" == "$dst" ]]; then
      return 0
    fi
  done
  return 1
}

while IFS= read -r edge; do
  [[ -z "$edge" ]] && continue
  src="${edge%% *}"
  dst="${edge##* }"

  if ! is_allowed_dep "$src" "$dst"; then
    echo "boundary violation: '$src' must not depend on '$dst'" >&2
    errors=$((errors + 1))
  fi
done <<< "$internal_edges"

if (( errors > 0 )); then
  echo >&2
  echo "Boundary check failed with $errors issue(s)." >&2
  echo "See: docs/开发规范手册.md (Crate 边界契约)" >&2
  exit 1
fi

echo "Boundary check passed."
