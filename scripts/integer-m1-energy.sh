#!/usr/bin/env bash
# scripts/integer-m1-energy.sh
# Measure Joules per token on Apple Silicon M1 (Track T3).
# Modeled on stage 8 of scripts/lut-m1-chat.sh and D0-b / D8 / D10 contracts.
#
# Usage:
#   ./scripts/integer-m1-energy.sh [--dry-run] [--sampler <macmon|powermetrics|dual>] [--out <DIR>] [--interval-ms <INT>]
#
# Samplers:
#   macmon (default): Non-root sampling via /opt/homebrew/bin/macmon measuring sys_power (whole-system Watts).
#   powermetrics    : Requires root/sudo. Samples SoC internal power.
#   dual            : Requires root/sudo. Samples both simultaneously for cross-validation.
#
# Requirements:
#   - Single-threaded evaluation: RAYON_NUM_THREADS=1.
#   - Dedicated model slot lock automatically claimed and released.
#   - Same-input stepping across sliding full256 context (no sampling/stop tokens).
#
set -euo pipefail

DRY_RUN=0
OUT=""
SAMPLER="macmon"
REPEATS=3
IDLE_SECONDS=8
INTERVAL_MS=100
K_LOW=8192
K_HIGH=32768

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --sampler)
      SAMPLER="$2"
      shift 2
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
    --interval-ms)
      INTERVAL_MS="$2"
      shift 2
      ;;
    --k-low)
      K_LOW="$2"
      shift 2
      ;;
    --k-high)
      K_HIGH="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

# Enforce non-root execution (EUID 0 refused)
if [[ "${EUID:-$(id -u)}" -eq 0 ]]; then
  echo "ERROR: integer-m1-energy.sh must NOT be run as root (EUID 0)." >&2
  echo "Run as normal user: macmon requires no sudo. Sudo is only used internally if powermetrics is selected." >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Target directory on SSD cache (renamed from anti-gravity-efficiency)
TARGET_DIR="${CARGO_TARGET_DIR:-/Volumes/UOR-Workspace/BuildCaches/anti-gravity}"
STEP_BIN="${STEP_BIN:-$TARGET_DIR/release/examples/same-input-step}"

TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
if [[ -z "$OUT" ]]; then
  OUT="/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-energy-${TIMESTAMP}"
fi
SUMMARY_JSON="$REPO_ROOT/docs/evidence/energy-summary-${TIMESTAMP}.json"

BUNDLE_DIR="${BUNDLE_DIR:-/Users/casey.allard/uor-r4/.uor-models/investigations/integer-serving-20260925/bundle-quaternion-1}"
if [[ ! -d "$BUNDLE_DIR" ]]; then
  BUNDLE_DIR="/Volumes/UOR-Workspace/Backups/language-continuation-20260926-1/canonical/.uor-models/investigations/integer-serving-20260925/bundle-quaternion-1"
fi

FF_CHECKPOINT="${FF_CHECKPOINT:-/Users/casey.allard/uor-r4-investigations/joint-recurrent-20260924/fit256-quaternion-3/checkpoint-final}"

TOKENS_FILE="${TOKENS_FILE:-/Users/casey.allard/uor-r4/.uor-models/research/issue-1017/tokens/dev.u16}"
if [[ ! -f "$TOKENS_FILE" ]]; then
  TOKENS_FILE="/Volumes/UOR-Workspace/Backups/language-continuation-20260926-1/canonical/.uor-models/research/issue-1017/tokens/dev.u16"
fi

LOCK_FILE="/Volumes/UOR-Workspace/locks/model-slot.json"

echo "================================================================================"
echo "Track T3: M1 Serving Energy Evaluation (J/token via Same-Input Stepping)"
echo "================================================================================"
echo "Dry Run         : $DRY_RUN"
echo "Sampler         : $SAMPLER"
echo "Output Directory: $OUT"
echo "Target Dir      : $TARGET_DIR"
echo "Step Runner     : $STEP_BIN"
echo "Bundle Path     : $BUNDLE_DIR"
echo "FF Checkpoint   : $FF_CHECKPOINT"
echo "Tokens File     : $TOKENS_FILE"
echo "Repeats         : $REPEATS"
echo "Sampling Rate   : $INTERVAL_MS ms"
echo "Step Horizons   : $K_LOW and $K_HIGH (ΔN = $((K_HIGH - K_LOW)))"
echo "Rayon Threads   : 1 (single-threaded benchmark)"
echo "================================================================================"

# Verify data files exist
if [[ ! -d "$BUNDLE_DIR" ]]; then
  echo "ERROR: Bundle directory $BUNDLE_DIR does not exist." >&2
  exit 1
fi

if [[ ! -d "$FF_CHECKPOINT" ]]; then
  echo "ERROR: FF checkpoint directory $FF_CHECKPOINT does not exist." >&2
  exit 1
fi

if [[ ! -f "$TOKENS_FILE" ]]; then
  echo "ERROR: Token file $TOKENS_FILE does not exist." >&2
  exit 1
fi

# Build rule: Builds happen ONLY in the dry run
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo ""
  echo "[DRY RUN: Compiling release binaries in isolated build cache]"
  CARGO_TARGET_DIR="$TARGET_DIR" cargo build --release -p uor-r4-training --example same-input-step -j 2
fi

if [[ ! -x "$STEP_BIN" ]]; then
  echo "ERROR: $STEP_BIN not found or not executable." >&2
  echo "Builds happen only in the dry run. Please run './scripts/integer-m1-energy.sh --dry-run' first." >&2
  exit 1
fi

# Dry-run execution test
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo ""
  echo "================================================================================"
  echo "[DRY RUN: Testing Functional Execution on $K_LOW and $K_HIGH Steps (No Power Sampler)]"
  echo "================================================================================"
  
  DRY_FAILURES=0
  for STEPS in "$K_LOW" "$K_HIGH"; do
    echo "--- Testing Integer Model ($STEPS steps) ---"
    T_START=$(python3 -c 'import time; print(time.time())')
    env RAYON_NUM_THREADS=1 "$STEP_BIN" \
      --model-type integer \
      --bundle "$BUNDLE_DIR" \
      --tokens-file "$TOKENS_FILE" \
      --tokens "$STEPS"
    T_END=$(python3 -c 'import time; print(time.time())')
    ELAPSED=$(python3 -c "print(f'{$T_END - $T_START:.2f}')")
    SAMPLES=$(python3 -c "import math; print(math.floor(float('$ELAPSED') * 1000 / $INTERVAL_MS))")
    echo "  Elapsed: ${ELAPSED}s (~${SAMPLES} samples at ${INTERVAL_MS}ms)"
    if (( $(python3 -c "print(1 if $ELAPSED >= 4.0 else 0)") )); then
      echo "  [PASS] Duration >= 4.0s (${ELAPSED}s, ~${SAMPLES} samples at ${INTERVAL_MS}ms interval)"
    else
      echo "  [FAIL] Duration < 4.0s (${ELAPSED}s, ~${SAMPLES} samples at ${INTERVAL_MS}ms interval)"
      DRY_FAILURES=$((DRY_FAILURES + 1))
    fi
    echo ""

    echo "--- Testing Continuous FF Parent ($STEPS steps) ---"
    T_START=$(python3 -c 'import time; print(time.time())')
    env RAYON_NUM_THREADS=1 "$STEP_BIN" \
      --model-type continuous-ff \
      --checkpoint "$FF_CHECKPOINT" \
      --tokens-file "$TOKENS_FILE" \
      --tokens "$STEPS"
    T_END=$(python3 -c 'import time; print(time.time())')
    ELAPSED=$(python3 -c "print(f'{$T_END - $T_START:.2f}')")
    SAMPLES=$(python3 -c "import math; print(math.floor(float('$ELAPSED') * 1000 / $INTERVAL_MS))")
    echo "  Elapsed: ${ELAPSED}s (~${SAMPLES} samples at ${INTERVAL_MS}ms)"
    if (( $(python3 -c "print(1 if $ELAPSED >= 4.0 else 0)") )); then
      echo "  [PASS] Duration >= 4.0s (${ELAPSED}s, ~${SAMPLES} samples at ${INTERVAL_MS}ms interval)"
    else
      echo "  [FAIL] Duration < 4.0s (${ELAPSED}s, ~${SAMPLES} samples at ${INTERVAL_MS}ms interval)"
      DRY_FAILURES=$((DRY_FAILURES + 1))
    fi
    echo ""
  done

  echo "================================================================================"
  if [[ "$DRY_FAILURES" -eq 0 ]]; then
    echo "[DRY RUN COMPLETE: PASS]"
    echo "Preflight execution verified cleanly across $K_LOW and $K_HIGH step horizons (all runs >= 4.0s)."
    echo "Note: Continuous FF checkpoint (fit256-quaternion-3) is the continuous floating-point ancestor"
    echo "of the served integer bundle (bundle-quaternion-1), not the same model."
  else
    echo "[DRY RUN COMPLETE: FAIL] $DRY_FAILURES preflight run(s) lasted < 4.0s."
    echo "Strict rule requires all benchmark runs to last >= 4.0s for power sample stability."
    exit 1
  fi
  echo "================================================================================"
  echo ""
  echo "Preconditions for energy measurement:"
  echo "  1. AC power connected (battery discharge power is invalid for benchmarking)."
  echo "  2. Display lid open."
  echo "  3. Machine idle (no other background builds, browsers, or heavy tasks)."
  echo "  4. Model slot lock must be free (the script claims and releases the slot itself)."
  echo ""
  echo "Owner Execution Command:"
  echo "  cd $REPO_ROOT && sudo -v && ./scripts/integer-m1-energy.sh"
  echo "================================================================================"
  exit 0
fi

# Acquire model-slot lock automatically (refuse if already claimed)
if [[ -f "$LOCK_FILE" ]]; then
  echo "NOTICE: Model slot lock file exists at $LOCK_FILE." >&2
  echo "Current slot holder:" >&2
  cat "$LOCK_FILE" >&2
  echo "" >&2
  echo "ERROR: Model slot is actively held by another job. Cannot run energy benchmarks concurrently." >&2
  echo "Wait until active job completes before running energy measurements." >&2
  exit 1
fi

mkdir -p "$(dirname "$LOCK_FILE")"
cat <<EOF > "$LOCK_FILE"
{
  "lab": "Anti-Gravity",
  "branch": "lab/anti-gravity/runtime-efficiency-t3",
  "pid": $$,
  "started_utc": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "expected_end_utc": "$(date -u -v+60M +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u +%Y-%m-%dT%H:%M:%SZ)",
  "threads": 1,
  "rss_cap_gb": 1.5,
  "job": "T3 marginal serving energy measurement"
}
EOF
trap 'rm -f "$LOCK_FILE"' EXIT INT TERM

mkdir -p "$OUT"
git rev-parse HEAD > "$OUT/source-revision.txt"
uname -a > "$OUT/machine.txt"
sysctl -n machdep.cpu.brand_string >> "$OUT/machine.txt" 2>/dev/null || true

echo ""
echo "Running Energy Benchmarks with sampler: $SAMPLER..."

SUDO_PREFIX=""
if [[ "$SAMPLER" == "powermetrics" || "$SAMPLER" == "dual" ]]; then
  SUDO_PREFIX="sudo "
fi

for STEPS in "$K_LOW" "$K_HIGH"; do
  for repeat in $(seq 1 "$REPEATS"); do
    echo "[Integer Model] Steps: $STEPS, Repeat: $repeat"
    ${SUDO_PREFIX}python3 "$SCRIPT_DIR/energy_per_token.py" \
      --label "integer-w256-k${STEPS}-rep${repeat}" \
      --sampler "$SAMPLER" \
      --idle-seconds "$IDLE_SECONDS" \
      --interval-ms "$INTERVAL_MS" \
      --tokens "$STEPS" \
      --raw-out "$OUT/raw-integer-k${STEPS}-rep${repeat}.jsonl" -- \
      env RAYON_NUM_THREADS=1 "$STEP_BIN" \
        --model-type integer \
        --bundle "$BUNDLE_DIR" \
        --tokens-file "$TOKENS_FILE" \
        --tokens "$STEPS" 2>&1 | tee "$OUT/energy-integer-k${STEPS}-rep${repeat}.txt"

    echo "[Continuous FF Parent] Steps: $STEPS, Repeat: $repeat"
    ${SUDO_PREFIX}python3 "$SCRIPT_DIR/energy_per_token.py" \
      --label "continuous-ff-k${STEPS}-rep${repeat}" \
      --sampler "$SAMPLER" \
      --idle-seconds "$IDLE_SECONDS" \
      --interval-ms "$INTERVAL_MS" \
      --tokens "$STEPS" \
      --raw-out "$OUT/raw-continuous-ff-k${STEPS}-rep${repeat}.jsonl" -- \
      env RAYON_NUM_THREADS=1 "$STEP_BIN" \
        --model-type continuous-ff \
        --checkpoint "$FF_CHECKPOINT" \
        --tokens-file "$TOKENS_FILE" \
        --tokens "$STEPS" 2>&1 | tee "$OUT/energy-continuous-ff-k${STEPS}-rep${repeat}.txt"
  done
done

echo ""
echo "================================================================================"
echo "Computing Marginal Energy per Token (ΔE/ΔN) and Evaluating Decision Rules..."
echo "================================================================================"
python3 "$SCRIPT_DIR/analyze_marginal_energy.py" "$OUT" \
  --repeats "$REPEATS" \
  --k-low "$K_LOW" \
  --k-high "$K_HIGH" \
  --bundle-dir "$BUNDLE_DIR" \
  --checkpoint-dir "$FF_CHECKPOINT" \
  --summary-out "$SUMMARY_JSON"

(cd "$OUT" && shasum -a 256 * > sha256sums.txt 2>/dev/null || true)

echo "Energy measurements complete."
echo "Raw files saved in: $OUT"
echo "Summary JSON saved in: $SUMMARY_JSON"
echo ""
echo "Filesystem Space:"
df -h / /Volumes/UOR-Workspace
echo "================================================================================"
