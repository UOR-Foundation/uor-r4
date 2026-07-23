#!/bin/bash
# Gate C trend alarm (issue #77): run the Gate C measurement on the fixture
# corpus, compare against the pinned record, and fail on regression beyond
# the pinned thresholds. Writes the trend row to $TREND_OUT (JSONL) for the
# CI artifact upload.
set -euo pipefail

FIXTURE_META="${1:-crates/uor-r4-core/tests/fixtures/c_meta.bin}"
FIXTURE_RECS="${2:-crates/uor-r4-core/tests/fixtures/c_recs.bin}"
FIXTURE_ART="${3:-crates/uor-r4-core/tests/fixtures/tless_artifacts.bin}"
PINNED="docs/transformerless/gate_c_pinned.json"
OUT_DIR="${TREND_DIR:-/tmp/gate_c_trend}"
TREND_OUT="${TREND_OUT:-/tmp/gate_c_trend.jsonl}"

rm -rf "$OUT_DIR"
cargo run -q --release --bin r4 -- transformerless score \
  --corpus-meta "$FIXTURE_META" --corpus-recs "$FIXTURE_RECS" \
  --artifacts "$FIXTURE_ART" --out "$OUT_DIR" >/dev/null

python3 - "$PINNED" "$OUT_DIR/score_report.json" "$TREND_OUT" <<'PYEOF'
import json, sys, os, time
pinned = json.load(open(sys.argv[1]))
report = json.load(open(sys.argv[2]))
gate = report["gate_c"]
row = {
    "commit": os.environ.get("GITHUB_SHA", "local"),
    "run_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    "rule12_top1_agreement": gate["rule12_precedence"]["top1_agreement"],
    "rule12_bits_per_token": gate["rule12_precedence"]["bits_per_token"],
    "baseline_top1_agreement": gate["tla3_baseline"]["top1_agreement"],
    "baseline_bits_per_token": gate["tla3_baseline"]["bits_per_token"],
}
with open(sys.argv[3], "a") as f:
    f.write(json.dumps(row) + "\n")

failures = []
alarm = pinned["alarm"]
for label, cur, pin in [
    ("rule12_top1_agreement", row["rule12_top1_agreement"], pinned["rule12_top1_agreement"]),
    ("baseline_top1_agreement", row["baseline_top1_agreement"], pinned["baseline_top1_agreement"]),
]:
    if cur < pin - alarm["top1_drop_abs"]:
        failures.append(f"{label}: {cur:.4f} regressed below pinned {pin:.4f} - {alarm['top1_drop_abs']}")
for label, cur, pin in [
    ("rule12_bits_per_token", row["rule12_bits_per_token"], pinned["rule12_bits_per_token"]),
    ("baseline_bits_per_token", row["baseline_bits_per_token"], pinned["baseline_bits_per_token"]),
]:
    if cur > pin + alarm["bits_regress_abs"]:
        failures.append(f"{label}: {cur:.4f} regressed above pinned {pin:.4f} + {alarm['bits_regress_abs']}")

if failures:
    print("GATE C TREND ALARM:")
    for f_ in failures:
        print(f"  {f_}")
    sys.exit(1)
print(f"gate c trend ok: rule12 {row['rule12_top1_agreement']:.4f}/{row['rule12_bits_per_token']:.4f}, "
      f"baseline {row['baseline_top1_agreement']:.4f}/{row['baseline_bits_per_token']:.4f}")
PYEOF
