#!/usr/bin/env bash
# pod.sh STEP: the on-pod recipe for claude-400 (D87), run inside the vLLM image on the
# leased Vast 4xH200 pod. Steps: setup | serve | wait-ready | smoke | run | status.
# The engine (vLLM) runs here, outside every compiled module (CHARTER 3.4); the client
# is the repository's Rust instrument, built here from the copied source.
set -euo pipefail
W=${M2_POD_W:-/workspace/m2}
MODEL_REPO=deepseek-ai/DeepSeek-V4.1-Flash-1017
MODEL_REV=08c8657f02291e11b6f6b63e0e16014f8e748518
MODEL_DIR=$W/donor
PORT=8000
SERVER=http://127.0.0.1:$PORT
BIN=$W/instrument/target/release/donor-corpus
mkdir -p "$W"

step=${1:?usage: pod.sh setup|serve|wait-ready|smoke|run|status}
shift
case $step in
setup)
  # 1. Client: Rust toolchain and the instrument copied to $W/instrument by the laptop.
  if ! [ -x "$HOME/.cargo/bin/cargo" ]; then
    curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal >/dev/null
  fi
  (cd "$W/instrument" && CARGO_INCREMENTAL=0 "$HOME/.cargo/bin/cargo" build --release 2>&1 | tail -2)
  # 2. Donor weights at the pinned revision, then every LFS file against HF's sha256.
  pip install -q 'huggingface_hub[hf_xet]' >/dev/null 2>&1 || true
  HF_XET_HIGH_PERFORMANCE=1 hf download "$MODEL_REPO" --revision "$MODEL_REV" --local-dir "$MODEL_DIR" >/dev/null
  curl -sf "https://huggingface.co/api/models/$MODEL_REPO/tree/$MODEL_REV?recursive=true" |
    jq -r '.[] | select(.lfs) | "\(.lfs.oid)  \(.path)"' > "$W/donor-lfs.sha256"
  # 16 parallel checkers; every line that is not "OK" is printed, and any one fails setup.
  rm -f "$W"/lfs-part.*
  split -n l/16 "$W/donor-lfs.sha256" "$W/lfs-part."
  (cd "$MODEL_DIR" && for p in "$W"/lfs-part.??; do sha256sum -c "$p" > "$p.out" 2>&1 & done; wait) || true
  if grep -h -v ': OK$' "$W"/lfs-part.*.out; then echo "donor LFS sha256 check FAILED (lines above)"; exit 1; fi
  echo "donor LFS sha256: all $(cat "$W"/lfs-part.*.out | grep -c ': OK$') of $(wc -l < "$W/donor-lfs.sha256") OK"
  nvidia-smi --query-gpu=index,name,uuid,driver_version,memory.total --format=csv,noheader > "$W/gpus.csv"
  nvidia-smi topo -m > "$W/gpu-topo.txt" 2>&1 || true   # NV# between all four GPUs = the NVLink bridge TP4 needs
  free -g | head -2 > "$W/host-mem.txt"; df -h /workspace | tail -1 >> "$W/host-mem.txt"
  vllm --version > "$W/vllm-version.txt" 2>&1
  echo "setup done"
  ;;
serve)
  # One endpoint, one TP4 replica per four GPUs; the recipe's H200 settings,
  # no speculative decoding (the recipe marks it for small batches).
  MAX_SEQS=${MAX_SEQS:-256}
  NG=$(nvidia-smi -L | wc -l); DP=$((NG / 4))
  [ "$DP" -ge 1 ] && [ $((DP * 4)) -eq "$NG" ] || { echo "serve: $NG GPUs is not a multiple of 4"; exit 1; }
  echo "serve: $NG GPUs, data-parallel $DP x tensor-parallel 4"
  nohup env VLLM_ENGINE_READY_TIMEOUT_S=7200 vllm serve "$MODEL_DIR" \
    --served-model-name donor --port $PORT \
    --tokenizer-mode deepseek_v41 \
    --data-parallel-size "$DP" --tensor-parallel-size 4 \
    --kv-cache-dtype fp8 \
    --engram-config '{"cpu_offload":true}' \
    --reasoning-parser deepseek_v41 \
    --language-model-only \
    --max-num-seqs "$MAX_SEQS" --max-num-batched-tokens 8192 \
    --gpu-memory-utilization 0.92 \
    > "$W/vllm.log" 2>&1 < /dev/null &
  echo $! > "$W/vllm.pid"
  echo "vllm pid $(cat "$W/vllm.pid"), log $W/vllm.log"
  ;;
wait-ready)
  for _ in $(seq 1 720); do
    if curl -sf "$SERVER/v1/models" >/dev/null; then echo ready; exit 0; fi
    kill -0 "$(cat "$W/vllm.pid")" 2>/dev/null || { echo "vllm exited"; tail -30 "$W/vllm.log"; exit 1; }
    sleep 10
  done
  echo "not ready after 2 h"; exit 1
  ;;
smoke)
  COMMIT=${1:?smoke needs the instrument commit}
  for f in model machine engine; do [ -s "$W/$f.json" ] || { echo "missing $W/$f.json"; exit 1; }; done
  head -32 "$W/bank.jsonl" > "$W/smoke-bank.jsonl"
  rm -rf "$W/smoke"
  "$BIN" generate --server "$SERVER" --prompts "$W/smoke-bank.jsonl" --out "$W/smoke" \
    --budget 100000000 --parallel 8 --limit 32 --instrument-commit "$COMMIT" ${EXTRA:+--extra-body "$EXTRA"}
  "$BIN" seal --dir "$W/smoke" --prompts "$W/smoke-bank.jsonl" --model "$W/model.json" --machine "$W/machine.json" --engine "$W/engine.json"
  "$BIN" verify --dir "$W/smoke" --prompts "$W/smoke-bank.jsonl"
  # The sheet's sampling, as recorded (temperature 1 and top_p 1 are fixed in request_body).
  jq -e 'select(.effort == 50 and .max_tokens == 16384 and .base_seed == 20261116 and .model_name == "donor")' \
    "$W/smoke/segments.jsonl" > /dev/null || { echo "smoke: recorded sampling differs from the sheet"; exit 1; }
  jq -c '{records: .records, kept_tokens, empty_reasoning_turns, dropped_by_reason}' "$W/smoke/metrics.json"
  echo "smoke OK"
  ;;
run)
  # run COMMIT PARALLEL MAX_SECONDS: one segment of the corpus run, detached; resumes.
  COMMIT=${1:?}; PAR=${2:?}; SECS=${3:?}
  mkdir -p "$W/out"
  # A restore from the laptop is incomplete: the guard does not sync during one, so refuse to run.
  [ ! -e "$W/out/.restoring" ] || { echo "run: $W/out/.restoring exists (restore incomplete): run drive.sh put again"; exit 1; }
  nohup "$BIN" generate --server "$SERVER" --prompts "$W/bank.jsonl" --out "$W/out" \
    --budget 1000000000 --parallel "$PAR" --max-seconds "$SECS" --instrument-commit "$COMMIT" \
    ${EXTRA:+--extra-body "$EXTRA"} > "$W/run-$(date -u +%Y%m%dT%H%M%SZ).log" 2>&1 < /dev/null &
  echo $! > "$W/run.pid"
  echo "run pid $(cat "$W/run.pid")"
  ;;
status)
  echo "run: $(kill -0 "$(cat "$W/run.pid" 2>/dev/null)" 2>/dev/null && echo alive || echo stopped)"
  echo "vllm: $(kill -0 "$(cat "$W/vllm.pid" 2>/dev/null)" 2>/dev/null && echo alive || echo stopped)"
  tail -3 "$W/out/rate.jsonl" 2>/dev/null || true
  echo "finished shards: $(find "$W/out" -maxdepth 1 -name 'shard-*.jsonl' 2>/dev/null | wc -l)"
  nvidia-smi --query-gpu=index,utilization.gpu,memory.used --format=csv,noheader
  ;;
*) echo "unknown step $step" >&2; exit 1;;
esac
