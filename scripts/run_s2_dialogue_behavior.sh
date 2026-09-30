#!/bin/bash
set -euo pipefail

# Reproduction script for S2 Dialogue Codec Behavior and Distortion Analysis
# References #973 under #820.
# Laboratory: Anti-Gravity (Lab 3)

MODEL_DIR="${MODEL_DIR:-/Volumes/UOR-Workspace/uor-r4-lab/claude-s2-dialogue-baseline/dialogue-1/model}"
BASELINE_LUT="${BASELINE_LUT:-/Volumes/UOR-Workspace/uor-r4-lab/claude-s2-dialogue-baseline/export-1/model.lut}"
HELDOUT="${HELDOUT:-/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/prepared/heldout}"
REQUESTS="${REQUESTS:-tests/fixtures/development-requests.json}"
TOKENIZER="${TOKENIZER:-tests/fixtures/tokenizer.json}"
OUT_DIR="${OUT_DIR:-/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2-turn-optimization-2}"

echo "=== S2 Dialogue Codec Behavior & Turn Optimization ==="
echo "Model:        $MODEL_DIR"
echo "Baseline LUT: $BASELINE_LUT"
echo "Heldout:      $HELDOUT"
echo "Requests:     $REQUESTS"
echo "Tokenizer:    $TOKENIZER"
echo "Output:       $OUT_DIR"

# Run s2-behavior-codec
cargo run --release --bin s2-behavior-codec -- \
  model="$MODEL_DIR" \
  baseline_lut="$BASELINE_LUT" \
  heldout="$HELDOUT" \
  requests="$REQUESTS" \
  tokenizer="$TOKENIZER" \
  out="$OUT_DIR"

# Optional: Run s2-behavior-distortion
if [ "${RUN_DISTORTION:-0}" = "1" ]; then
  DIST_OUT="${DIST_OUT:-/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2-behavior-distortion-1}"
  echo "Running distortion analysis -> $DIST_OUT"
  cargo run --release --bin s2-behavior-distortion -- \
    model="$MODEL_DIR" \
    artifact="$BASELINE_LUT" \
    heldout="$HELDOUT/tokens.u16" \
    requests="$REQUESTS" \
    tokenizer="$TOKENIZER" \
    windows=32 \
    out="$DIST_OUT"
fi

echo "S2 dialogue behavior execution complete."
