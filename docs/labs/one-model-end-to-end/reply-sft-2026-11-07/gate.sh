#!/usr/bin/env bash
# Static gates for the reply-SFT pod job and controller.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
JOB=${1:-"$ROOT/docs/labs/one-model-end-to-end/reply-sft-2026-11-07/job.sh"}
LAUNCHER=${2:-"$ROOT/docs/labs/one-model-end-to-end/reply-sft-2026-11-07/run-s1.sh"}
checks=0

fail() {
  echo "FAIL: $*" >&2
  exit 1
}
require() {
  local text=$1 file=$2
  rg -F --quiet -- "$text" "$file" || fail "$file lacks: $text"
  checks=$((checks + 1))
}

bash -n "$JOB"
bash -n "$LAUNCHER"
checks=$((checks + 2))
if command -v shellcheck >/dev/null; then
  shellcheck "$JOB" "$LAUNCHER"
  checks=$((checks + 1))
fi

require 'SEED must be 1; seed 2 is a later decision' "$JOB"
require 'results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/inputs/tokens.u16' "$JOB"
require 'f7852f2540e58c17b2311d39fb47f8e53471e2cda8d6136f81b034cdf6bf5bba' "$JOB"
require '--init-from "$B64L"' "$JOB"
require '--steps 1024' "$JOB"
require '--eval-every 256' "$JOB"
require 'kill -STOP "$TRAIN_PID"' "$JOB"
require 'if [ "$guard" -lt 93 ]' "$JOB"
require 'if [ "$step" -eq 512 ] && [ "$eos_top1" -eq 0 ] && [ "$eos_p50" -ge 7 ]' "$JOB"
require '--max-replies 1 --reply-tokens 1' "$JOB"
require '"$STORE" put "$RUN" "$OUT_STORE"' "$JOB"
require 'READBACK_STATUS=verified' "$JOB"
require '--gpu 5090' "$LAUNCHER"
require '--count 1 --hours 2 --ref "$PIN"' "$LAUNCHER"
require '"$POD_TOOL" renew "$POD"' "$LAUNCHER"
require 'READBACK_FILE' "$LAUNCHER"
require '--confirm-archived' "$LAUNCHER"

echo "$checks checks passed"
