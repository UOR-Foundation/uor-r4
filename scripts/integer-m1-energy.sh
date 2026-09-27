#!/usr/bin/env bash
# scripts/integer-m1-energy.sh
# Measure Joules per token on Apple Silicon M1 (Track T3).
# Modeled on stage 8 of scripts/lut-m1-chat.sh.
#
# Usage:
#   ./scripts/integer-m1-energy.sh [--dry-run] [--out <DIR>]
#
# Requirements:
#   - RAYON_NUM_THREADS=1 for single-threaded evaluation
#   - model-slot lock at /Volumes/UOR-Workspace/locks/model-slot.json
#   - powermetrics requires sudo (exact command printed; run outside if unprivileged)
#
set -euo pipefail

DRY_RUN=0
OUT=""
REPEATS=3
IDLE_SECONDS=8

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --out)
      OUT="$2"
      shift 2
      ;;
    --repeats)
      REPEATS="$2"
      shift 2
      ;;
    --idle-seconds)
      IDLE_SECONDS="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [[ -z "$OUT" ]]; then
  OUT="$REPO_ROOT/docs/evidence/energy-measurement-$(date +%Y%m%d-%H%M%S)"
fi

TARGET_DIR="${CARGO_TARGET_DIR:-/Volumes/UOR-Workspace/BuildCaches/anti-gravity-efficiency}"
INTEGER_CHAT="${INTEGER_CHAT:-$TARGET_DIR/release/uor-chat}"
FF_GENERATE="${FF_GENERATE:-$TARGET_DIR/release/examples/continuous-ff-generate}"
PROMPT="Once upon a time there was a little girl who"

BUNDLE_DIR="${BUNDLE_DIR:-/Users/casey.allard/uor-r4/.uor-models/investigations/integer-serving-20260925/bundle-quaternion-1}"
if [[ ! -d "$BUNDLE_DIR" ]]; then
  BUNDLE_DIR="/Volumes/UOR-Workspace/Backups/language-continuation-20260926-1/canonical/.uor-models/investigations/integer-serving-20260925/bundle-quaternion-1"
fi

FF_CHECKPOINT="${FF_CHECKPOINT:-/Users/casey.allard/uor-r4-investigations/joint-recurrent-20260924/fit256-quaternion-3/checkpoint-final}"
FF_TOKENIZER="${FF_TOKENIZER:-$BUNDLE_DIR/tokenizer.json}"

LOCK_FILE="/Volumes/UOR-Workspace/locks/model-slot.json"

echo "================================================================================"
echo "Track T3: M1 Serving Energy Evaluation (J/token)"
echo "================================================================================"
echo "Dry Run         : $DRY_RUN"
echo "Output Directory: $OUT"
echo "Target Dir      : $TARGET_DIR"
echo "Integer Chat    : $INTEGER_CHAT"
echo "FF Generate     : $FF_GENERATE"
echo "Bundle Path     : $BUNDLE_DIR"
echo "FF Checkpoint   : $FF_CHECKPOINT"
echo "Repeats         : $REPEATS"
echo "Token Horizons  : 128 and 512"
echo "Rayon Threads   : 1"
echo "================================================================================"

# Verify prerequisites
if [[ ! -x "$INTEGER_CHAT" ]]; then
  echo "WARNING: $INTEGER_CHAT not found or not executable. Building..." >&2
  CARGO_TARGET_DIR="$TARGET_DIR" cargo build --release -p uor-r4-integer --bin uor-chat -j 2
fi

if [[ ! -x "$FF_GENERATE" ]]; then
  echo "WARNING: $FF_GENERATE not found or not executable. Building..." >&2
  CARGO_TARGET_DIR="$TARGET_DIR" cargo build --release -p uor-r4-training --example continuous-ff-generate -j 2
fi

if [[ ! -d "$BUNDLE_DIR" ]]; then
  echo "ERROR: Bundle directory $BUNDLE_DIR does not exist." >&2
  exit 1
fi

if [[ ! -d "$FF_CHECKPOINT" ]]; then
  echo "ERROR: FF checkpoint directory $FF_CHECKPOINT does not exist." >&2
  exit 1
fi

# Dry-run execution test
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo ""
  echo "================================================================================"
  echo "[DRY RUN: Testing Functional Execution on 128 and 512 Tokens Without Powermetrics]"
  echo "================================================================================"
  for TOKENS in 128 512; do
    echo "--- Testing Integer Model ($TOKENS tokens) ---"
    env RAYON_NUM_THREADS=1 "$INTEGER_CHAT" \
      --bundle "$BUNDLE_DIR" \
      --temperature 0.0 \
      --max-tokens "$TOKENS" \
      --prompt "$PROMPT"
    echo ""
    echo "--- Testing Continuous FF Parent ($TOKENS tokens) ---"
    env RAYON_NUM_THREADS=1 "$FF_GENERATE" \
      --checkpoint "$FF_CHECKPOINT" \
      --tokenizer "$FF_TOKENIZER" \
      --prompt "$PROMPT" \
      --tokens "$TOKENS"
    echo ""
    if [[ -n "${LLAMA_CLI:-}" ]] && [[ -n "${GGUF:-}" ]]; then
      echo "--- Testing llama.cpp SmolLM2 ($TOKENS tokens) ---"
      "$LLAMA_CLI" -m "$GGUF" -p "$PROMPT" -n "$TOKENS" -t 1 --temp 0 -no-cnv
      echo ""
    fi
  done
  echo "================================================================================"
  echo "[DRY RUN COMPLETE]"
  echo "Preflight execution verified cleanly across 128 and 512 token horizons."
  echo ""
  echo "To execute actual powermetrics energy measurement, run under sudo with the"
  echo "model slot lock claimed:"
  echo ""
  echo "  # Step 1: Claim model slot lock"
  echo "  cat <<EOF > $LOCK_FILE"
  echo "  {\"lab\": \"Anti-Gravity\", \"branch\": \"lab/anti-gravity/runtime-efficiency-t3\", \"pid\": \$\$, \"started_utc\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\", \"expected_end_utc\": \"$(date -u -v+30M +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u +%Y-%m-%dT%H:%M:%SZ)\", \"threads\": 1, \"rss_cap_gb\": 1.5}"
  echo "  EOF"
  echo ""
  echo "  # Step 2: Run integer energy measurement script"
  echo "  sudo ./scripts/integer-m1-energy.sh --out $OUT"
  echo ""
  echo "  # Step 3: Release model slot lock"
  echo "  rm -f $LOCK_FILE"
  echo "================================================================================"
  exit 0
fi

# Acquire model-slot lock
if [[ -f "$LOCK_FILE" ]]; then
  echo "ERROR: Model slot lock file exists at $LOCK_FILE." >&2
  echo "Locked by:" >&2
  cat "$LOCK_FILE" >&2
  echo "" >&2
  echo "Cannot run heavy model measurement while another job holds the slot." >&2
  exit 1
fi

mkdir -p "$(dirname "$LOCK_FILE")"
cat <<EOF > "$LOCK_FILE"
{
  "lab": "Anti-Gravity",
  "branch": "lab/anti-gravity/runtime-efficiency-t3",
  "pid": $$,
  "started_utc": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "expected_end_utc": "$(date -u -v+30M +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u +%Y-%m-%dT%H:%M:%SZ)",
  "threads": 1,
  "rss_cap_gb": 1.5
}
EOF
trap 'rm -f "$LOCK_FILE"' EXIT INT TERM

mkdir -p "$OUT"
git rev-parse HEAD > "$OUT/source-revision.txt"
uname -a > "$OUT/machine.txt"
sysctl -n machdep.cpu.brand_string >> "$OUT/machine.txt" 2>/dev/null || true

echo ""
echo "Running Energy Benchmarks..."

for TOKENS in 128 512; do
  for repeat in $(seq 1 "$REPEATS"); do
    echo "[Integer Model] Tokens: $TOKENS, Repeat: $repeat"
    sudo python3 "$SCRIPT_DIR/energy_per_token.py" \
      --label "integer-w256-k${TOKENS}-rep${repeat}" \
      --idle-seconds "$IDLE_SECONDS" -- \
      env RAYON_NUM_THREADS=1 "$INTEGER_CHAT" \
        --bundle "$BUNDLE_DIR" \
        --temperature 0.0 \
        --max-tokens "$TOKENS" \
        --prompt "$PROMPT" 2>&1 | tee "$OUT/energy-integer-k${TOKENS}-rep${repeat}.txt"

    echo "[Continuous FF Parent] Tokens: $TOKENS, Repeat: $repeat"
    sudo python3 "$SCRIPT_DIR/energy_per_token.py" \
      --label "continuous-ff-k${TOKENS}-rep${repeat}" \
      --idle-seconds "$IDLE_SECONDS" -- \
      env RAYON_NUM_THREADS=1 "$FF_GENERATE" \
        --checkpoint "$FF_CHECKPOINT" \
        --tokenizer "$FF_TOKENIZER" \
        --prompt "$PROMPT" \
        --tokens "$TOKENS" 2>&1 | tee "$OUT/energy-continuous-ff-k${TOKENS}-rep${repeat}.txt"

    if [[ -n "${LLAMA_CLI:-}" ]] && [[ -n "${GGUF:-}" ]]; then
      echo "[llama.cpp SmolLM2] Tokens: $TOKENS, Repeat: $repeat"
      sudo python3 "$SCRIPT_DIR/energy_per_token.py" \
        --label "llama-smollm2-k${TOKENS}-rep${repeat}" \
        --idle-seconds "$IDLE_SECONDS" \
        --tokens "$TOKENS" -- \
        "$LLAMA_CLI" -m "$GGUF" -p "$PROMPT" -n "$TOKENS" -t 1 --temp 0 -no-cnv 2>&1 | \
        tee "$OUT/energy-llama-smollm2-k${TOKENS}-rep${repeat}.txt"
    fi
  done
done

echo ""
echo "================================================================================"
echo "Computing Marginal Energy per Token (ΔE/ΔN) and Evaluating Decision Rules..."
echo "================================================================================"
python3 "$SCRIPT_DIR/analyze_marginal_energy.py" "$OUT" --repeats "$REPEATS"
echo "Energy measurements complete. Files and summary saved in $OUT"
echo "================================================================================"
