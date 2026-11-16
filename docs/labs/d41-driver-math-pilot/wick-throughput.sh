#!/usr/bin/env bash
# d41 donor throughput probe (claude-398), docs/harness/prep/d41-donor-throughput-2026-11-15.md.
# Runs inside Claude's wick container after the pilot (claude-390): it reuses the pilot's
# llama.cpp b12000 build, its GGUF and its fixed prompt set. The donor runs only in
# llama-server, outside the repository (CHARTER 3.4). Nothing here is corpus.
#
#   wick-throughput.sh ARMS_FILE OUT_NAME INSTRUMENT_COMMIT
#
# ARMS_FILE holds one arm per line: NAME|SERVER_ARGS|CLIENT_ARGS. Consecutive arms with the
# same SERVER_ARGS share one llama-server. An arm named smoke-* runs on 4 prompts with
# --parallel 1 and must pass the smoke check; every other arm is a timed window.
set -euo pipefail
ARMS=$1
NAME=$2
COMMIT=$3
W=/workspace/d41
IN=$W/in398
OUT=$W/tp/$NAME
SEED=20261115
PROMPTS=$W/prompts.jsonl
PROMPTS_SHA=6a6630b2eb0078c3c795b04e005da47c810113cf41d15888c7632bb96d37cca0
WINDOW=${WINDOW:-300}
PORT=8087
LS=$W/engine/llama.cpp/build/bin/llama-server
BASE="-m $W/gpt-oss-20b.gguf -ngl 99 --flash-attn on --jinja --host 127.0.0.1 --port $PORT"
export PATH=$HOME/.cargo/bin:$PATH
test ! -e "$OUT" || { echo "$OUT exists: one pass per name" >&2; exit 5; }
mkdir -p "$OUT"
cp "$ARMS" "$OUT/arms.txt"

echo "== instrument at $COMMIT"
(cd "$IN/instrument" && CARGO_TARGET_DIR=$W/target398 CARGO_INCREMENTAL=0 cargo build --release -q)
P=$W/target398/release/d41-pilot
test "$(sha256sum "$PROMPTS" | cut -d' ' -f1)" = "$PROMPTS_SHA"
head -4 "$PROMPTS" > "$OUT/smoke-prompts.jsonl"
nvidia-smi --query-gpu=name,uuid,driver_version,memory.used,memory.total --format=csv,noheader | tee "$OUT/gpu-before.txt"

SPID=""
GPID=""
CUR=""
stop_server() {
  if [ -n "$GPID" ]; then kill "$GPID" 2>/dev/null || true; wait "$GPID" 2>/dev/null || true; GPID=""; fi
  if [ -n "$SPID" ]; then kill "$SPID" 2>/dev/null || true; wait "$SPID" 2>/dev/null || true; SPID=""; fi
  CUR=""
}
trap stop_server EXIT

start_server() {  # $1 = server args, $2 = log name
  stop_server
  # shellcheck disable=SC2086
  "$LS" $BASE $1 < /dev/null > "$OUT/server-$2.log" 2>&1 &
  SPID=$!
  for _ in $(seq 1 120); do
    curl -sf "http://127.0.0.1:$PORT/health" >/dev/null && break
    kill -0 "$SPID" 2>/dev/null || { echo "server died: $1" >&2; tail -20 "$OUT/server-$2.log" >&2; return 1; }
    sleep 3
  done
  curl -sf "http://127.0.0.1:$PORT/health" >/dev/null
  CUR=$1
}

gpu_watch() {  # $1 = file; one line every 5 s: memory used MiB, utilization %
  ( while true; do nvidia-smi --query-gpu=memory.used,utilization.gpu --format=csv,noheader,nounits; sleep 5; done ) > "$1" &
  GPID=$!
}

while IFS='|' read -r -u 3 ARM SARGS CARGS; do
  case $ARM in ''|'#'*) continue;; esac
  echo "== arm $ARM | server: $SARGS | client: $CARGS"
  if [ "$SARGS" != "$CUR" ]; then
    start_server "$SARGS" "$ARM" || { echo "$ARM server-failed" >> "$OUT/failed.txt"; continue; }
  fi
  D=$OUT/$ARM
  gpu_watch "$OUT/$ARM.gpu"
  rc=0
  case $ARM in
    smoke-*)
      # shellcheck disable=SC2086
      "$P" generate --server "http://127.0.0.1:$PORT" --prompts "$OUT/smoke-prompts.jsonl" --out "$D" \
        --budget 1000000 --parallel 1 --seed $SEED --max-tokens 1024 --instrument-commit "$COMMIT" $CARGS > /dev/null || rc=$?
      ;;
    *)
      # shellcheck disable=SC2086
      "$P" generate --server "http://127.0.0.1:$PORT" --prompts "$PROMPTS" --out "$D" \
        --budget 1000000000 --seed $SEED --max-tokens 1024 --shard-records 512 --max-seconds "$WINDOW" \
        --instrument-commit "$COMMIT" $CARGS > /dev/null || rc=$?
      ;;
  esac
  kill "$GPID" 2>/dev/null || true; wait "$GPID" 2>/dev/null || true; GPID=""
  if [ $rc -ne 0 ]; then echo "$ARM rc=$rc" >> "$OUT/failed.txt"; continue; fi
  if jq -e '.kept_answers_counted == false' "$D/metrics.json" >/dev/null; then
    "$P" count --server "http://127.0.0.1:$PORT" --dir "$D" > /dev/null
  fi
  case $ARM in
    smoke-*)
      head -c 1500 "$D/shard-0000.jsonl"; echo
      jq -e '.markup_answers == 0 and .records_kept >= 3 and .errors == 0 and .answers_returned == 16' "$D/metrics.json" \
        || echo "$ARM smoke-check-failed" >> "$OUT/failed.txt"
      ;;
  esac
  jq -c '{wall_ms, prompts_dispatched, requests, answers_returned, generated_tokens, kept_answer_tokens,
          generated_millitokens_per_second, kept_answer_millitokens_per_second, truncated_answers, markup_answers,
          errors, tokenize_errors, request_ms}' "$D/metrics.json"
done 3< "$ARMS"
stop_server

echo "== summary (rates in thousandths of a token per second; gpu = mean utilization %, peak memory MiB)"
{
  printf 'arm\twall_ms\tprompts\tgenerated\tkept\tgen_mtps\tkept_mtps\tkept_per_1000_gen\tdropped\terrors\tgpu_util\tgpu_mib\n'
  for D in "$OUT"/*/; do
    A=$(basename "$D")
    [ -s "$D/metrics.json" ] || continue
    KEPT=$(jq '.kept_answer_tokens' "$D/metrics.json")
    [ -s "$D/count.json" ] && KEPT=$(jq '.kept_answer_tokens' "$D/count.json")
    G=$(awk -F', ' '{u+=$2; if ($1>m) m=$1; n++} END {if (n) printf "%d\t%d", u/n, m; else printf "-\t-"}' "$OUT/$A.gpu")
    jq -r --argjson kept "$KEPT" --arg g "$G" --arg a "$A" \
      '[$a, .wall_ms, .prompts_dispatched, .generated_tokens, $kept,
        (.generated_tokens * 1000000 / .wall_ms | floor), ($kept * 1000000 / .wall_ms | floor),
        ($kept * 1000 / ([.generated_tokens, 1] | max) | floor), (.truncated_answers + .markup_answers), .errors, $g] | @tsv' \
      "$D/metrics.json"
  done
} | tee "$OUT/summary.tsv"
test ! -s "$OUT/failed.txt" || { echo "== failed:"; cat "$OUT/failed.txt"; }
echo probe-done
