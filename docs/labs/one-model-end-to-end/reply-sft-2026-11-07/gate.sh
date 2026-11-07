#!/usr/bin/env bash
# Static gates for the reply-SFT pod job and controller.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
JOB=${1:-"$ROOT/docs/labs/one-model-end-to-end/reply-sft-2026-11-07/job.sh"}
LAUNCHER=${2:-"$ROOT/docs/labs/one-model-end-to-end/reply-sft-2026-11-07/run-s1.sh"}
TRAINER="$ROOT/crates/uor-r4-float-train/src/in_space_long.rs"
checks=0

fail() {
  echo "FAIL: $*" >&2
  exit 1
}
require_line() {
  local text=$1 file=$2
  awk -v want="$text" '
    {
      line=$0
      sub(/^[ \t]+/, "", line)
      sub(/[ \t]+$/, "", line)
      if (line == want) found = 1
    }
    END { exit(found ? 0 : 1) }
  ' "$file" || fail "$file lacks complete line: $text"
  checks=$((checks + 1))
}

bash -n "$JOB"
bash -n "$LAUNCHER"
checks=$((checks + 2))
if command -v shellcheck >/dev/null; then
  shellcheck "$JOB" "$LAUNCHER"
  checks=$((checks + 1))
fi

require_line '2) echo "seed 2 is conditional and is not launched by this script" >&2; exit 2 ;;' "$JOB"
require_line '*) echo "SEED must be 1; seed 2 is a later decision" >&2; exit 2 ;;' "$JOB"
require_line 'got=$(sha256sum "$path" | cut -d'"'"' '"'"' -f1)' "$JOB"
require_line '[ "$got" = "$want" ] || die "$path hashes to $got, not $want"' "$JOB"
require_line 'OUT_STORE=${UOR_SFT_OUT_STORE:-results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/runs/s1}' "$JOB"
require_line 'if [ -d "$sealed" ] && (cd "$sealed" && sha256sum -c SHA256SUMS); then' "$JOB"
require_line 'if "$STORE" put "$RUN" "$OUT_STORE"; then' "$JOB"
require_line '"$STORE" put "$RUN.readback-status" "$OUT_STORE/READBACK" ||' "$JOB"

require_line 'fetch_file "results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/inputs/tokens.u16" \' "$JOB"
require_line 'fetch_file "results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/inputs/response_mask.u8" \' "$JOB"
require_line 'fetch_file "results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/inputs/manifest.json" \' "$JOB"
require_line 'fetch_file "results/claude/real-text-corpus-2026-11-05/oasst2/heldout/tokens.u16" \' "$JOB"
require_line 'fetch_file "results/claude/real-text-corpus-2026-11-05/oasst2/heldout/response_mask.u8" \' "$JOB"
require_line 'fetch_file "results/claude/real-text-corpus-2026-11-05/oasst2/heldout/manifest.json" \' "$JOB"
require_line 'fetch_file "results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/s1/d3-accum-s1.tar.gz" \' "$JOB"
require_line 'fetch_file "data/m2-fresh256/inputs.json" "$RUN/inputs/m2-fresh256/inputs.json"' "$JOB"
require_line 'fetch_file "data/m2-fresh256/labels.json" "$RUN/inputs/m2-fresh256/labels.json"' "$JOB"
require_line 'fetch_file "data/m2-fresh256/plan.json" "$RUN/inputs/m2-fresh256/plan.json"' "$JOB"
require_line 'fetch_file "data/m2-fresh256/manifest.json" "$RUN/inputs/m2-fresh256/manifest.json"' "$JOB"
require_line 'fetch_file "w2-inputs/2026-10-25/driver-data/tokenizer.json" "$RUN/inputs/data/tokenizer.json"' "$JOB"
require_line 'fetch_file "w2-inputs/2026-10-25/driver-data/ft/dev/tokens.u16" \' "$JOB"
require_line 'fetch_file "w2-inputs/2026-10-25/driver-data/ft/dev/response_mask.u8" \' "$JOB"

for hash in \
  3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3 \
  67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b \
  1c7aee4dfaffdde07f9c64a5b887b504ee21355038b6ae00897a8a97407d9bbe \
  02a04e97bbb8936c993bf5eff0f29c171ebeae88988217a4100fdc77701f169e \
  9c5bb6567c6a4cded2dfb2280ae4acdc798e283a3beae8aaebc695656e1b4833 \
  7474ba9331f242148dd9833d5cb085307c8e0d86314d13f285a0ab1c42b8b081 \
  f7852f2540e58c17b2311d39fb47f8e53471e2cda8d6136f81b034cdf6bf5bba \
  1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a \
  3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be \
  0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715 \
  7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964 \
  d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89 \
  0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
  a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
  6fd2d764171cc416d5ff272e19e3ebe0644183ab9c43ca0662706756ac75f5de \
  1045acb61d92d390f0f61617ffa4c1847437473c9694c09787b48bf46e761ed2 \
  672de97fcc329b10c3cde591f66047352851f9fe1d02df85a168190f5ba12ddd \
  6b2b84640e37d1f6322a5b4ab6db0d783469d7adf4db1c7e67b532d4310056f2
do
  require_line "check_file $hash \\" "$JOB"
done

grade_flag_lines=$(
  awk '
    {
      line=$0
      sub(/^[ \t]+/, "", line)
      sub(/[ \t]+$/, "", line)
      if (line == "--max-evaluation 512 --max-replies 1 --reply-tokens 1 \\") found++
    }
    END { print found }
  ' "$JOB"
)
[ "$grade_flag_lines" -eq 2 ] ||
  fail "$JOB does not pin both checkpoint and base-control grade flags"
checks=$((checks + 1))
require_line 'kill -STOP "$TRAIN_PID" 2>/dev/null || true' "$JOB"
require_line 'if [ -d "$checkpoint" ] && [ -f "$checkpoint/.checkpoint-ready" ]; then' "$JOB"
require_line '[ -f "$checkpoint/.checkpoint-ready" ] ||' "$JOB"
grep -Fq 'mark_checkpoint_ready(&directory)?;' "$TRAINER" ||
  fail "$TRAINER does not mark a checkpoint complete after saving it"
checks=$((checks + 1))
termination_count=$(
  awk '
  {
    line=$0
    sub(/^[ \t]+/, "", line)
    sub(/[ \t]+$/, "", line)
    if (previous == "kill -TERM \"$TRAIN_PID\" 2>/dev/null || true" &&
        line == "kill -CONT \"$TRAIN_PID\" 2>/dev/null || true") found++
    previous = line
  }
  END { print found }
' "$JOB"
)
[ "$termination_count" -eq 2 ] ||
  fail "$JOB does not continue both stopped-trainer SIGTERM paths"
checks=$((checks + 1))
require_line 'if [ "$guard" -lt 93 ]; then' "$JOB"
require_line 'elif [ "$step" -eq 512 ] && [ "$eos_top1" -eq 0 ] && [ "$eos_p50" -ge 7 ]; then' "$JOB"
require_line 'run_panel 0 "$BASE_ARTIFACT" one-user-turn/v1 1 base-one-user-teacher >/dev/null' "$JOB"
require_line '.eos_rank_min == 5 and .eos_rank_p50 == 7 and .eos_rank_max == 13 and' "$JOB"
require_line '.eos_rank_min == 18 and .eos_rank_p50 == 138 and .eos_rank_max == 873 and' "$JOB"
require_line '.complete == 0 and .eos_stops == 0 and .bos_stops == 0 and' "$JOB"
require_line '[ "$base_guard" -eq 132 ] ||' "$JOB"
require_line '--init-from "$B64L" \' "$JOB"
require_line '--train-split "$RUN/inputs/mix" \' "$JOB"
require_line '--train-sha256 3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3 \' "$JOB"
require_line '--train-mask-sha256 67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b \' "$JOB"
require_line '--batch 64 --micro-batch 16 --steps 1024 \' "$JOB"
require_line '--eval-every 256 --samples 16 --reply-tokens 64 --seed 1 \' "$JOB"
require_line 'CARGO_TARGET_DIR="$RUN/panel-target" \' "$JOB"
require_line 'PANEL_BIN="$RUN/panel-target/release/served-reply-panel"' "$JOB"
require_line 'READBACK_STATUS=verified' "$JOB"

require_line 'created=$("$POD_TOOL" up --lab deepseek --session "$SESSION" --gpu 5090 \' "$LAUNCHER"
require_line '[ "$(git rev-parse HEAD)" = "$PIN" ] ||' "$LAUNCHER"
require_line '[ -z "$(git status --porcelain --untracked-files=no)" ] ||' "$LAUNCHER"
require_line '--count 1 --hours 2 --ref "$PIN" \' "$LAUNCHER"
require_line 'if ! "$POD_TOOL" run "$POD" --lab deepseek --session "$SESSION" --gpu 0 -- \' "$LAUNCHER"
grep -Fqx "JOB_LOG=\$(awk '/Log: / {" "$LAUNCHER" ||
  fail "$LAUNCHER does not begin parsing uor-pod's job-log path"
checks=$((checks + 1))
grep -Fq 'for (i = 1; i <= NF; i++) if ($i == "Log:") { print $(i + 1); exit }' \
  "$LAUNCHER" || fail "$LAUNCHER does not parse the log path from uor-pod's one-line success message"
checks=$((checks + 1))
require_line '*) cat "$CONTROLLER/run.out" >&2; echo "uor-pod did not report a job log" >&2; exit 1 ;;' "$LAUNCHER"
require_line '[ -z "$job_exit" ] || break' "$LAUNCHER"
require_line '[ "$job_exit" -eq 0 ] || {' "$LAUNCHER"
if grep -Fq 'WRAPPER_PID' "$LAUNCHER"; then
  fail "$LAUNCHER still waits on the detached-job submission wrapper"
fi
checks=$((checks + 1))
require_line 'if ! "$POD_TOOL" renew "$POD" --lab deepseek --session "$SESSION" --hours 2 \' "$LAUNCHER"
require_line '"$STORE" get "$OUT_STORE/READBACK" "$CONTROLLER/readback" >/dev/null' "$LAUNCHER"
require_line '[ -f "$READBACK_FILE" ] || { echo "no READBACK file was returned" >&2; exit 1; }' "$LAUNCHER"
require_line '[ "$(cat "$READBACK_FILE")" = verified ] || {' "$LAUNCHER"
require_line '[ "$(cat "$CONTROLLER/readback/$OUT_STORE/EXIT")" = 0 ] || {' "$LAUNCHER"
require_line '--confirm-archived || rc=1' "$LAUNCHER"
require_line 'STATUS=SEALED' "$LAUNCHER"

echo "$checks checks passed"
