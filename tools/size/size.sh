#!/usr/bin/env bash
# Measures the release `kbx` binary and fails when it outgrows the budget in tools/size/budget.
# Writes the measurement as JSON when given a path.
# Usage: size.sh [out.json]   (build first: cargo build --release --locked -p kbx-app)
set -euo pipefail

binary=target/release/kbx
budget=$(tr -d '[:space:]' <tools/size/budget)
[ -f "$binary" ] || {
  echo "no $binary; run cargo build --release --locked -p kbx-app"
  exit 1
}
bytes=$(stat -f %z "$binary")

if [ -n "${1:-}" ]; then
  jq -n --arg date "$(date -u +%FT%TZ)" --arg commit "$(git describe --always --dirty --abbrev=40)" \
    --argjson bytes "$bytes" --argjson budget "$budget" \
    '{schema: 1, tier: 2, metric: "artifact-size", unit: "bytes", implementation: "rust", date: $date,
      repoCommit: $commit, bundle: "release binary, stripped, LTO", bytes: $bytes, budget: $budget}' >"$1"
fi

echo "kbx: $((bytes / 1048576)) MiB ($bytes bytes), budget $((budget / 1048576)) MiB"
if [ "$bytes" -gt "$budget" ]; then
  echo "    -> the binary outgrew its budget; find the cause with cargo bloat before raising tools/size/budget"
  exit 1
fi
