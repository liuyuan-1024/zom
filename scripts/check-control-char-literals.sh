#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

matches_file="$(mktemp)"
cfg_test_file="$(mktemp)"
violations_file="$(mktemp)"

cleanup() {
  rm -f "$matches_file" "$cfg_test_file" "$violations_file"
}
trap cleanup EXIT

# 只扫描 crates/apps 的 Rust 源码，并排除白名单：
# 1) token 定义 crate（允许定义底层符号）
# 2) tests 目录与 tests.rs 文件
# 3) 快照目录/文件
rg -n --no-heading --color never '\\n|\\r|\\t' \
  "$ROOT/crates" "$ROOT/apps" \
  --glob '**/*.rs' \
  --glob '!crates/zom-text-tokens/**' \
  --glob '!**/tests/**' \
  --glob '!**/tests.rs' \
  --glob '!**/*_tests.rs' \
  --glob '!**/*_test.rs' \
  --glob '!**/*.snap' \
  --glob '!**/__snapshots__/**' \
  > "$matches_file" || true

# 记录每个文件首个 #[cfg(test)] 位置；该行及之后默认视为测试区域。
rg -n --no-heading --color never '^\s*#\[cfg\(test\)\]' \
  "$ROOT/crates" "$ROOT/apps" \
  --glob '**/*.rs' \
  --glob '!crates/zom-text-tokens/**' \
  > "$cfg_test_file" || true

awk -F: -v cfg_file="$cfg_test_file" '
BEGIN {
  while ((getline line < cfg_file) > 0) {
    split(line, parts, ":");
    file = parts[1];
    line_no = parts[2] + 0;
    if (!(file in cfg_cutoff) || line_no < cfg_cutoff[file]) {
      cfg_cutoff[file] = line_no;
    }
  }
}
{
  file = $1;
  line_no = $2 + 0;
  prefix = file ":" $2 ":";
  text = substr($0, length(prefix) + 1);

  if ((file in cfg_cutoff) && line_no >= cfg_cutoff[file]) {
    next;
  }
  if (text ~ /control-char-lint:[[:space:]]*allow/) {
    next;
  }
  if (index(text, "\"") == 0 && index(text, sprintf("%c", 39)) == 0) {
    next;
  }

  print file ":" line_no ":" text;
}
' "$matches_file" > "$violations_file"

if [[ -s "$violations_file" ]]; then
  echo "Detected bare control-char literals in production code."
  echo "Please use zom-text-tokens constants instead (or add 'control-char-lint: allow' when justified)."
  echo
  cat "$violations_file"
  exit 1
fi

echo "No bare control-char literals found in production code."
