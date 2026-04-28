#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

MODE="${1:-}"

if [[ "$MODE" == "--enforce" ]]; then
  cargo run --release -p zom-editor --example core_perf_baseline -- --enforce
else
  cargo run --release -p zom-editor --example core_perf_baseline --
fi
