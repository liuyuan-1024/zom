#!/usr/bin/env bash
set -euo pipefail

# Check rustdoc coverage rules for workspace Rust sources.
# Enforced rules:
# 1) Every non-empty module file starts with `//!` module doc.
# 2) Every `pub` / `pub(crate)` struct/enum/trait/fn/type/const is preceded by `///`.
#
# Usage:
#   ./scripts/check-rustdoc-coverage.sh

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v python3 >/dev/null 2>&1; then
  echo "error: python3 is required" >&2
  exit 2
fi

python3 - "$ROOT" <<'PY'
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1])
files = sorted(
    list((root / "crates").rglob("*.rs")) +
    list((root / "apps").rglob("*.rs"))
)

missing_module_docs = []
missing_public_docs = []

pub_pattern = re.compile(r'^\s*pub(?:\(crate\))?\s+(struct|enum|trait|fn|type|const)\b')

for path in files:
    lines = path.read_text(encoding="utf-8").splitlines()

    # Rule 1: module-level `//!`
    first_significant = None
    for line in lines:
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith("#!["):
            continue
        first_significant = stripped
        break

    if first_significant is not None and not first_significant.startswith("//!"):
        missing_module_docs.append((path, first_significant))

    # Rule 2: public item `///`
    for idx, line in enumerate(lines):
        if not pub_pattern.search(line):
            continue

        prev = idx - 1
        while prev >= 0 and not lines[prev].strip():
            prev -= 1
        prev_line = lines[prev].strip() if prev >= 0 else ""

        while prev_line.startswith("#["):
            prev -= 1
            while prev >= 0 and not lines[prev].strip():
                prev -= 1
            prev_line = lines[prev].strip() if prev >= 0 else ""

        if not prev_line.startswith("///"):
            missing_public_docs.append((path, idx + 1, line.strip()))

if missing_module_docs:
    print("rustdoc coverage violation: missing module `//!` docs")
    for path, first in missing_module_docs:
        print(f"- {path}: first significant line -> {first}")
    print()

if missing_public_docs:
    print("rustdoc coverage violation: missing `///` docs on public items")
    for path, line_no, decl in missing_public_docs:
        print(f"- {path}:{line_no}: {decl}")
    print()

if missing_module_docs or missing_public_docs:
    print("Rustdoc coverage check failed.")
    sys.exit(1)

print("Rustdoc coverage check passed.")
PY
