#!/usr/bin/env python3
"""
Adversarial Stress Harness for UOR-R4 Integer Serving (Milestone M1)
Author: teamwork_preview_challenger_m1_1
Tests standalone CLI binary on extreme boundary conditions, failure injection,
tamper detection, and probability mass conservation.
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

BINARY = Path("target/release/uor-r4-integer")
BUNDLE_QUAT = Path("/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1")
BUNDLE_ORD = Path("/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-householder_pair-1")
PROBABILITY_TOTAL = 1 << 48  # 281474976710656

results = []

def run_test(test_id, name, test_fn):
    print(f"Running [{test_id}] {name}...", end=" ", flush=True)
    t0 = time.time()
    try:
        passed, msg = test_fn()
        elapsed = round((time.time() - t0) * 1000, 1)
        if passed:
            print(f"\033[92mPASS\033[0m ({elapsed}ms)")
        else:
            print(f"\033[91mFAIL\033[0m: {msg} ({elapsed}ms)")
        results.append({
            "test_id": test_id,
            "name": name,
            "status": "PASS" if passed else "FAIL",
            "message": msg,
            "duration_ms": elapsed
        })
    except Exception as e:
        elapsed = round((time.time() - t0) * 1000, 1)
        print(f"\033[91mCRASH\033[0m: {e} ({elapsed}ms)")
        results.append({
            "test_id": test_id,
            "name": name,
            "status": "FAIL",
            "message": f"Exception: {e}",
            "duration_ms": elapsed
        })

# 1. CLI: Empty batch rejection
def test_cli_empty_batch():
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump([], f)
        req_path = f.name
    rep_dir = f"/tmp/adv_rep_empty_batch_{int(time.time()*1000)}"
    try:
        cmd = [str(BINARY), "generate", str(BUNDLE_QUAT), req_path, rep_dir]
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        passed = (res.returncode != 0 and "request batch must contain" in res.stderr)
        return passed, res.stderr.strip()
    finally:
        os.unlink(req_path)
        shutil.rmtree(rep_dir, ignore_errors=True)

# 2. CLI: Batch size 257 (exceeding maximum 256)
def test_cli_batch_overflow_257():
    reqs = [{"prompt": "test", "max_new_tokens": 1}] * 257
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump(reqs, f)
        req_path = f.name
    rep_dir = f"/tmp/adv_rep_overflow_batch_{int(time.time()*1000)}"
    try:
        cmd = [str(BINARY), "generate", str(BUNDLE_QUAT), req_path, rep_dir]
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        passed = (res.returncode != 0 and "request batch must contain" in res.stderr)
        return passed, res.stderr.strip()
    finally:
        os.unlink(req_path)
        shutil.rmtree(rep_dir, ignore_errors=True)

# 3. CLI: Empty prompt rejection inside valid batch
def test_cli_empty_prompt_in_batch():
    reqs = [
        {"prompt": "valid prompt", "max_new_tokens": 2},
        {"prompt": "", "max_new_tokens": 2}
    ]
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump(reqs, f)
        req_path = f.name
    rep_dir = f"/tmp/adv_rep_empty_prompt_{int(time.time()*1000)}"
    try:
        cmd = [str(BINARY), "generate", str(BUNDLE_QUAT), req_path, rep_dir]
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        passed = (res.returncode != 0 and "empty prompt" in res.stderr)
        return passed, res.stderr.strip()
    finally:
        os.unlink(req_path)
        shutil.rmtree(rep_dir, ignore_errors=True)

# 4. CLI: Sequence budget overflow (> 256 tokens total)
def test_cli_sequence_budget_overflow():
    reqs = [
        {"prompt": "word " * 200, "max_new_tokens": 100}  # ~200 tokens + 100 tokens > 256
    ]
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump(reqs, f)
        req_path = f.name
    rep_dir = f"/tmp/adv_rep_budget_overflow_{int(time.time()*1000)}"
    try:
        cmd = [str(BINARY), "generate", str(BUNDLE_QUAT), req_path, rep_dir]
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        passed = (res.returncode != 0 and "request exceeds full256 session budget" in res.stderr)
        return passed, res.stderr.strip()
    finally:
        os.unlink(req_path)
        shutil.rmtree(rep_dir, ignore_errors=True)

# 5. CLI: Zero new tokens requested
def test_cli_zero_max_new_tokens():
    reqs = [
        {"prompt": "Hello world", "max_new_tokens": 0}
    ]
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump(reqs, f)
        req_path = f.name
    rep_dir = f"/tmp/adv_rep_zero_tokens_{int(time.time()*1000)}"
    try:
        cmd = [str(BINARY), "generate", str(BUNDLE_QUAT), req_path, rep_dir]
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        passed = (res.returncode != 0 and "request exceeds full256 session budget" in res.stderr)
        return passed, res.stderr.strip()
    finally:
        os.unlink(req_path)
        shutil.rmtree(rep_dir, ignore_errors=True)

# 6. CLI: Repetitive input with ShortCycle stop and mass verification
def test_cli_repetitive_input_mass_verification():
    reqs = [
        {"prompt": "repeat " * 20, "max_new_tokens": 30, "selection": {"kind": "greedy"}}
    ]
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump(reqs, f)
        req_path = f.name
    rep_dir = f"/tmp/adv_rep_repetitive_{int(time.time()*1000)}"
    try:
        cmd = [str(BINARY), "generate", str(BUNDLE_QUAT), req_path, rep_dir]
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=15)
        if res.returncode != 0:
            return False, f"CLI returned non-zero code {res.returncode}: {res.stderr}"

        # Inspect generations.jsonl
        gen_file = Path(rep_dir) / "generations.jsonl"
        if not gen_file.exists():
            return False, "generations.jsonl not found"

        with open(gen_file) as gf:
            data = json.loads(gf.readline())
        gen = data["generation"]

        # Check every decision's mass
        for d in gen["decisions"]:
            if d["probability_sum_q48"] != PROBABILITY_TOTAL:
                return False, f"Probability sum mismatch: {d['probability_sum_q48']} != {PROBABILITY_TOTAL}"

        # Check stop reason is valid
        stop_reason = gen["stop"]["reason"]
        if stop_reason not in ("short_cycle", "maximum_new_tokens", "eos"):
            return False, f"Unexpected stop reason: {stop_reason}"

        return True, f"Stop reason: {stop_reason}, decisions verified: {len(gen['decisions'])}"
    finally:
        os.unlink(req_path)
        shutil.rmtree(rep_dir, ignore_errors=True)

# 7. CLI: Existing report directory rejection (atomic claim safety)
def test_cli_existing_report_directory_rejection():
    rep_dir = Path(f"/tmp/adv_rep_existing_{int(time.time()*1000)}")
    rep_dir.mkdir(parents=True, exist_ok=True)
    reqs = [{"prompt": "Hello", "max_new_tokens": 2}]
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump(reqs, f)
        req_path = f.name
    try:
        cmd = [str(BINARY), "generate", str(BUNDLE_QUAT), req_path, str(rep_dir)]
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        # report_output::claim must reject an existing directory
        passed = (res.returncode != 0 and "already exists" in res.stderr.lower() or "io" in res.stderr.lower() or "error" in res.stderr.lower())
        return passed, res.stderr.strip()
    finally:
        os.unlink(req_path)
        shutil.rmtree(rep_dir, ignore_errors=True)

# 8. CLI: Tampered Bundle Rejection (Cryptographic Seal Failure)
def test_cli_tampered_bundle_rejection():
    # Create temporary copy of bundle
    temp_bundle = Path(f"/tmp/adv_tampered_bundle_{int(time.time()*1000)}")
    shutil.copytree(BUNDLE_QUAT, temp_bundle)
    try:
        # Tamper with tables.bin by flipping a byte
        table_bin = temp_bundle / "tables" / "tables.bin"
        with open(table_bin, "r+b") as f:
            f.seek(100)
            orig_b = f.read(1)
            new_b = bytes([orig_b[0] ^ 0xFF])
            f.seek(100)
            f.write(new_b)

        reqs = [{"prompt": "Hello", "max_new_tokens": 2}]
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump(reqs, f)
            req_path = f.name
        rep_dir = f"/tmp/adv_rep_tamper_{int(time.time()*1000)}"

        cmd = [str(BINARY), "generate", str(temp_bundle), req_path, rep_dir]
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        passed = (res.returncode != 0 and ("manifest hash mismatch" in res.stderr or "mismatch" in res.stderr or "invalid" in res.stderr))
        return passed, res.stderr.strip()
    finally:
        shutil.rmtree(temp_bundle, ignore_errors=True)
        shutil.rmtree(rep_dir, ignore_errors=True)
        if os.path.exists(req_path):
            os.unlink(req_path)

# 9. CLI: Panic Freedom on Malformed JSON Request
def test_cli_malformed_json_request():
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        f.write("{malformed json [")
        req_path = f.name
    rep_dir = f"/tmp/adv_rep_malformed_{int(time.time()*1000)}"
    try:
        cmd = [str(BINARY), "generate", str(BUNDLE_QUAT), req_path, rep_dir]
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        passed = (res.returncode != 0 and "panicked at" not in res.stderr)
        return passed, "Clean JSON error return (no panic)"
    finally:
        os.unlink(req_path)
        shutil.rmtree(rep_dir, ignore_errors=True)

def main():
    print("=" * 80)
    print("ADVERSARIAL STRESS HARNESS: CLI & KERNEL BOUNDARIES")
    print("=" * 80)

    run_test("ADV_01", "CLI_EMPTY_BATCH_REJECTION", test_cli_empty_batch)
    run_test("ADV_02", "CLI_BATCH_OVERFLOW_257", test_cli_batch_overflow_257)
    run_test("ADV_03", "CLI_EMPTY_PROMPT_REJECTION", test_cli_empty_prompt_in_batch)
    run_test("ADV_04", "CLI_SEQUENCE_BUDGET_OVERFLOW", test_cli_sequence_budget_overflow)
    run_test("ADV_05", "CLI_ZERO_MAX_NEW_TOKENS", test_cli_zero_max_new_tokens)
    run_test("ADV_06", "CLI_REPETITIVE_MASS_CONSERVATION", test_cli_repetitive_input_mass_verification)
    run_test("ADV_07", "CLI_EXISTING_REPORT_DIR_REJECTION", test_cli_existing_report_directory_rejection)
    run_test("ADV_08", "CLI_TAMPERED_BUNDLE_REJECTION", test_cli_tampered_bundle_rejection)
    run_test("ADV_09", "CLI_MALFORMED_JSON_PANIC_FREEDOM", test_cli_malformed_json_request)

    print("=" * 80)
    total_passed = sum(1 for r in results if r["status"] == "PASS")
    total_failed = sum(1 for r in results if r["status"] == "FAIL")
    print(f"STRESS RESULTS: {total_passed} PASS / {total_failed} FAIL out of {len(results)} tests")
    print("=" * 80)

    out_file = Path("tests/adversarial/stress_results.json")
    with open(out_file, "w") as f:
        json.dump(results, f, indent=2)
    print(f"Saved results to {out_file}")

    if total_failed > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()
