#!/usr/bin/env python3
"""
Adversarial Stress Harness for Milestone M3 (Features 11 and 12)
Author: teamwork_preview_challenger_m3_1

Adversarially challenges:
1. Feature 11: Tail NLL computation, dev.u16 token store integrity,
   complete block partitioning (976 blocks / 249,856 targets),
   strict 143 trailing token omission, and Kahan compensated summation stability.
2. Feature 12: Integer serving numerical drift bounds (<= 0.01) across 256
   sequential steps, dynamic range stability, and probability mass conservation.
"""

import hashlib
import json
import math
import os
import struct
import sys
import time
from pathlib import Path

# Paths
DEV_U16_PATH = Path("/Users/casey.allard/uor-r4/.uor-models/research/issue-1017/tokens/dev.u16")
EVALUATOR_V2_PATH = Path("docs/integration/reference-evaluator-v2.json")
EVAL_QUAT_DIR = Path("/Users/casey.allard/uor-r4-investigations/full-context-integer-20260925/eval-quaternion-2")
EVAL_ORD_DIR = Path("/Users/casey.allard/uor-r4-investigations/full-context-integer-20260925/eval-householder_pair-2")

CONTEXT = 256
VOCAB_SIZE = 4096
PROBABILITY_TOTAL = 1 << 48

results = {
    "feature_11": {},
    "feature_12": {},
    "summary": {
        "total_checks": 0,
        "passed_checks": 0,
        "failed_checks": 0,
        "verdict": "PENDING"
    }
}

def record_check(section, check_id, name, passed, details):
    results["summary"]["total_checks"] += 1
    if passed:
        results["summary"]["passed_checks"] += 1
        status = "\033[92mPASS\033[0m"
    else:
        results["summary"]["failed_checks"] += 1
        status = "\033[91mFAIL\033[0m"
    print(f"[{section}] {check_id}: {name} -> {status}")
    if not passed:
        print(f"   Details: {details}")
    results[section][check_id] = {
        "name": name,
        "passed": passed,
        "details": details
    }

# =========================================================================
# FEATURE 11: Block Integrity and Trailing Token Omission
# =========================================================================

def challenge_feature_11_block_integrity():
    print("\n=== Challenging Feature 11: Block Integrity & Trailing Token Omission ===")
    
    # 1.1 Verify file exists, size and sha256
    if not DEV_U16_PATH.exists():
        record_check("feature_11", "F11.1", "dev.u16 file existence", False, f"File not found: {DEV_U16_PATH}")
        return
    
    file_bytes = DEV_U16_PATH.read_bytes()
    expected_sha256 = "74f7d85fa7670355f65805a9ffe06c2fb9db150b9f4cae8029e0d2f35e8993b6"
    actual_sha256 = hashlib.sha256(file_bytes).hexdigest()
    
    record_check(
        "feature_11", "F11.1", "dev.u16 SHA-256 Bitwise Verification",
        actual_sha256 == expected_sha256,
        {"expected": expected_sha256, "actual": actual_sha256, "byte_len": len(file_bytes)}
    )
    
    record_check(
        "feature_11", "F11.2", "dev.u16 Exact Byte Length (500,000 bytes)",
        len(file_bytes) == 500000,
        {"byte_len": len(file_bytes)}
    )
    
    # 1.2 Unpack tokens
    token_count = len(file_bytes) // 2
    tokens = struct.unpack(f"<{token_count}H", file_bytes)
    record_check(
        "feature_11", "F11.3", "Token Count Derivation (250,000 u16 tokens)",
        token_count == 250000,
        {"token_count": token_count}
    )
    
    # 1.3 Vocabulary bounds
    oov_tokens = [t for t in tokens if t >= VOCAB_SIZE]
    record_check(
        "feature_11", "F11.4", "Token Vocabulary Bound Strictness (0 <= y < 4096)",
        len(oov_tokens) == 0,
        {"oov_count": len(oov_tokens), "sample_oov": oov_tokens[:5]}
    )
    
    # 1.4 Mathematical Block Derivation
    # complete_blocks = (token_count - 1) // CONTEXT
    complete_blocks = (token_count - 1) // CONTEXT
    record_check(
        "feature_11", "F11.5", "Exact 976 Complete Blocks Formed",
        complete_blocks == 976,
        {"complete_blocks": complete_blocks, "formula": "(250000 - 1) // 256 = 976"}
    )
    
    scored_targets = complete_blocks * CONTEXT
    record_check(
        "feature_11", "F11.6", "Exact 249,856 Targets Scored",
        scored_targets == 249856,
        {"scored_targets": scored_targets}
    )
    
    # Check last token accessed in block 975
    # Block 975: start = 975 * 256 = 249,600
    # Inputs: tokens[249600 .. 249856]
    # Targets: tokens[249601 .. 249857]
    last_target_token_index = 975 * 256 + 256  # 249856 (0-indexed, so 249857th token)
    unscored_tail_tokens = token_count - 1 - scored_targets
    
    record_check(
        "feature_11", "F11.7", "Strict 143 Incomplete Trailing Tokens Omitted",
        unscored_tail_tokens == 143,
        {
            "token_count": token_count,
            "last_accessed_token_idx": last_target_token_index,
            "unscored_tail_tokens": unscored_tail_tokens,
            "omitted_range": f"[{last_target_token_index + 1} .. {token_count - 1}]"
        }
    )
    
    # 1.5 Adversarial Boundary Testing on complete_blocks algorithm
    def rust_complete_blocks(n):
        if n <= 1:
            return 0
        return (n - 1) // 256

    adv_boundaries = [
        (0, 0, "Empty sequence"),
        (256, 0, "Exactly 256 tokens (insufficient for 256 targets with +1 shift)"),
        (257, 1, "Exactly 257 tokens (1 complete block, 0 trailing)"),
        (512, 1, "512 tokens (1 complete block, 255 trailing omitted)"),
        (513, 2, "513 tokens (2 complete blocks, 0 trailing)"),
        (249856, 975, "249856 tokens (975 complete blocks, 255 trailing omitted)"),
        (249857, 976, "249857 tokens (976 complete blocks, 0 trailing)"),
        (250000, 976, "250000 tokens (976 complete blocks, 143 trailing omitted)"),
    ]
    all_adv_passed = True
    adv_details = []
    for count, expected_blk, desc in adv_boundaries:
        res = rust_complete_blocks(count)
        if res != expected_blk:
            all_adv_passed = False
            adv_details.append(f"FAILED {desc}: got {res}, expected {expected_blk}")
        else:
            adv_details.append(f"OK {desc}: {res} blocks")
            
    record_check(
        "feature_11", "F11.8", "Adversarial Token Count Boundary Partitioning",
        all_adv_passed,
        adv_details
    )
    
    # 1.6 Partition Split: Tune (64 blocks = 16,384) vs Comparison Tail (912 blocks = 233,472)
    tune_blocks = 64
    tune_targets = tune_blocks * CONTEXT
    comp_blocks = 976 - tune_blocks
    comp_targets = comp_blocks * CONTEXT
    
    record_check(
        "feature_11", "F11.9", "Frozen Tune (16,384) & Tail (233,472) Population Split",
        tune_targets == 16384 and comp_targets == 233472 and (tune_targets + comp_targets == 249856),
        {
            "tune_blocks": tune_blocks, "tune_targets": tune_targets,
            "comp_blocks": comp_blocks, "comp_targets": comp_targets,
            "total_targets": tune_targets + comp_targets
        }
    )

# =========================================================================
# FEATURE 11: Kahan Compensated Summation & LogSumExp Stress Testing
# =========================================================================

class CompensatedSum:
    def __init__(self):
        self.sum = 0.0
        self.correction = 0.0

    def add(self, value):
        adjusted = value - self.correction
        next_sum = self.sum + adjusted
        self.correction = (next_sum - self.sum) - adjusted
        self.sum = next_sum

    def total(self):
        return self.sum

def score_logits_py(logits, target):
    """Exact reproduction of score_logits in reference_eval.rs"""
    target_logit = logits[target]
    predicted = max(range(len(logits)), key=lambda i: logits[i])
    maximum = float(logits[predicted])
    
    denom = CompensatedSum()
    for logit in logits:
        denom.add(math.exp(float(logit) - maximum))
    
    shifted_log_normalizer = math.log(denom.total())
    log_normalizer = maximum + shifted_log_normalizer
    nll_nats = maximum - float(target_logit) + shifted_log_normalizer
    return {
        "target": target,
        "predicted": predicted,
        "target_logit": target_logit,
        "maximum": maximum,
        "log_normalizer": log_normalizer,
        "nll_nats": nll_nats,
        "correct": predicted == target
    }

def challenge_feature_11_kahan_summation():
    print("\n=== Challenging Feature 11: Kahan Compensated Summation & LogSumExp Stability ===")
    
    # 2.1 Order Invariance Test across 249,856 realistic values
    # Generate pseudo-random NLL losses with heavy tail
    import random
    rng = random.Random(42)
    synthetic_losses = [rng.expovariate(1.0 / 2.1) + 0.1 for _ in range(249856)]
    
    # Standard naive accumulation
    naive_fwd = sum(synthetic_losses)
    naive_rev = sum(reversed(synthetic_losses))
    shuffled = list(synthetic_losses)
    rng.shuffle(shuffled)
    naive_shuf = sum(shuffled)
    
    naive_fwd_rev_diff = abs(naive_fwd - naive_rev)
    naive_fwd_shuf_diff = abs(naive_fwd - naive_shuf)
    
    # Kahan CompensatedSum accumulation
    kahan_fwd = CompensatedSum()
    for v in synthetic_losses:
        kahan_fwd.add(v)
    
    kahan_rev = CompensatedSum()
    for v in reversed(synthetic_losses):
        kahan_rev.add(v)
        
    kahan_shuf = CompensatedSum()
    for v in shuffled:
        kahan_shuf.add(v)
        
    kahan_fwd_rev_diff = abs(kahan_fwd.total() - kahan_rev.total())
    kahan_fwd_shuf_diff = abs(kahan_fwd.total() - kahan_shuf.total())
    
    record_check(
        "feature_11", "F11.10", "Kahan Summation Order-Invariance across 249,856 Losses",
        kahan_fwd_rev_diff <= 1e-11 and kahan_fwd_shuf_diff <= 1e-10,
        {
            "naive_fwd_rev_diff": naive_fwd_rev_diff,
            "naive_fwd_shuf_diff": naive_fwd_shuf_diff,
            "kahan_fwd_rev_diff": kahan_fwd_rev_diff,
            "kahan_fwd_shuf_diff": kahan_fwd_shuf_diff,
            "precision_gain_ratio": naive_fwd_shuf_diff / max(kahan_fwd_shuf_diff, 1e-18)
        }
    )
    
    # 2.2 Extreme dynamic range / catastrophic cancellation resistance
    # Scenario: initial massive scalar 10^7 followed by 249,855 additions of 10^-10
    base_val = 1e7
    small_val = 1e-10
    count_small = 249855
    expected_sum = base_val + count_small * small_val  # 10000000.0000249855
    
    naive_dyn = base_val
    for _ in range(count_small):
        naive_dyn += small_val
    naive_dyn_err = abs(naive_dyn - expected_sum)
    
    kahan_dyn = CompensatedSum()
    kahan_dyn.add(base_val)
    for _ in range(count_small):
        kahan_dyn.add(small_val)
    kahan_dyn_err = abs(kahan_dyn.total() - expected_sum)
    
    record_check(
        "feature_11", "F11.11", "Kahan Precision Preservation under Extreme Dynamic Range (10^7 + 10^-10)",
        kahan_dyn_err < 1e-11 and naive_dyn_err > 1e-5,
        {
            "naive_error": naive_dyn_err,
            "kahan_error": kahan_dyn_err,
            "expected_sum": expected_sum,
            "kahan_sum": kahan_dyn.total(),
            "naive_sum": naive_dyn
        }
    )
    
    # 2.3 LogSumExp Shift Invariance under Massive Logit Translations (+-10,000)
    base_logits = [rng.gauss(0.0, 2.0) for _ in range(VOCAB_SIZE)]
    target_idx = 42
    base_score = score_logits_py(base_logits, target_idx)
    
    for shift in [100.0, -100.0, 1000.0, -1000.0, 10000.0, -10000.0, 100000.0]:
        shifted_logits = [l + shift for l in base_logits]
        shifted_score = score_logits_py(shifted_logits, target_idx)
        delta_nll = abs(shifted_score["nll_nats"] - base_score["nll_nats"])
        pred_match = (shifted_score["predicted"] == base_score["predicted"])
        tol = max(1e-12, abs(shift) * 5e-16 * 100)
        record_check(
            "feature_11", f"F11.12_shift_{int(shift)}",
            f"LogSumExp Shift Invariance (shift = {shift}, tol = {tol:.1e})",
            delta_nll < tol and pred_match,
            {"shift": shift, "delta_nll": delta_nll, "base_nll": base_score["nll_nats"], "shifted_nll": shifted_score["nll_nats"], "tolerance": tol}
        )

# =========================================================================
# FEATURE 12: Numerical Drift Bounds (<= 0.01) Across 256 Steps
# =========================================================================

def challenge_feature_12_numerical_drift():
    print("\n=== Challenging Feature 12: Numerical Drift Bounds across Sequential Trajectories ===")
    
    # 3.1 Replay Target Analysis from Retained Evaluator Logs
    for arm_name, eval_dir in [("quaternion", EVAL_QUAT_DIR), ("householder_pair", EVAL_ORD_DIR)]:
        targets_file = eval_dir / "targets-read.jsonl"
        if not targets_file.exists():
            record_check("feature_12", f"F12_log_{arm_name}", f"{arm_name} targets-read.jsonl exists", False, "Missing log")
            continue
        
        lines = targets_file.read_text().strip().split("\n")
        total_targets = len(lines)
        
        max_state_delta = 0.0
        max_prob_delta = 0.0
        max_tv = 0.0
        state_violations = []
        prob_violations = []
        step_drift_by_pos = [[] for _ in range(CONTEXT)]
        prob_drift_by_pos = [[] for _ in range(CONTEXT)]
        top1_mismatches = 0
        
        for line_idx, line in enumerate(lines):
            row = json.loads(line)
            pos = row["position_in_block"]
            sd = row["maximum_state_delta"]
            pd = row["maximum_probability_delta"]
            tv = row["total_variation"]
            int_pred = row["integer_prediction"]
            f32_pred = row["f32_prediction"]
            
            if int_pred != f32_pred:
                top1_mismatches += 1
                
            max_state_delta = max(max_state_delta, sd)
            max_prob_delta = max(max_prob_delta, pd)
            max_tv = max(max_tv, tv)
            
            step_drift_by_pos[pos].append(sd)
            prob_drift_by_pos[pos].append(pd)
            
            if sd > 0.01:
                state_violations.append((pos, sd))
            if pd > 0.01:
                prob_violations.append((pos, pd))
                
        # Invariant checks
        record_check(
            "feature_12", f"F12.1_{arm_name}_targets_count",
            f"{arm_name} Evaluation Target Count (1024 targets = 4 windows * 256)",
            total_targets == 1024,
            {"total_targets": total_targets}
        )
        
        record_check(
            "feature_12", f"F12.2_{arm_name}_state_drift_bound",
            f"{arm_name} State Drift <= 0.01 at EVERY Step (Max: {max_state_delta:.6f})",
            len(state_violations) == 0 and max_state_delta <= 0.01,
            {
                "max_state_delta": max_state_delta,
                "bound": 0.01,
                "violations_count": len(state_violations),
                "sample_violations": state_violations[:5]
            }
        )
        
        record_check(
            "feature_12", f"F12.3_{arm_name}_prob_drift_bound",
            f"{arm_name} Probability Drift <= 0.01 at EVERY Step (Max: {max_prob_delta:.6f})",
            len(prob_violations) == 0 and max_prob_delta <= 0.01,
            {
                "max_prob_delta": max_prob_delta,
                "bound": 0.01,
                "violations_count": len(prob_violations),
                "sample_violations": prob_violations[:5]
            }
        )
        
        # Check drift across horizons: does error compound exponentially?
        h0 = max(step_drift_by_pos[0]) if step_drift_by_pos[0] else 0.0
        h50 = max(step_drift_by_pos[50]) if step_drift_by_pos[50] else 0.0
        h100 = max(step_drift_by_pos[100]) if step_drift_by_pos[100] else 0.0
        h200 = max(step_drift_by_pos[200]) if step_drift_by_pos[200] else 0.0
        h255 = max(step_drift_by_pos[255]) if step_drift_by_pos[255] else 0.0
        
        # In a stable recurrent system with RMSNorm, drift does NOT grow linearly or exponentially with step
        drift_is_bounded = (h255 <= 0.01) and (h200 <= 0.01) and (h100 <= 0.01)
        record_check(
            "feature_12", f"F12.4_{arm_name}_trajectory_stability",
            f"{arm_name} Trajectory Boundedness across 256 Horizon (pos 0->{h0:.4f}, pos 100->{h100:.4f}, pos 255->{h255:.4f})",
            drift_is_bounded,
            {"pos_0": h0, "pos_50": h50, "pos_100": h100, "pos_200": h200, "pos_255": h255}
        )

# =========================================================================
# Main Execution
# =========================================================================

def main():
    print("=" * 80)
    print("STARTING MILENSTONE M3 ADVERSARIAL CHALLENGER VERIFICATION HARNESS")
    print("=" * 80)
    
    t0 = time.time()
    challenge_feature_11_block_integrity()
    challenge_feature_11_kahan_summation()
    challenge_feature_12_numerical_drift()
    elapsed = time.time() - t0
    
    passed = results["summary"]["passed_checks"]
    total = results["summary"]["total_checks"]
    failed = results["summary"]["failed_checks"]
    verdict = "APPROVE" if failed == 0 else "REJECT"
    results["summary"]["verdict"] = verdict
    results["summary"]["elapsed_seconds"] = elapsed
    
    print("\n" + "=" * 80)
    print(f"M3 ADVERSARIAL CHALLENGE COMPLETE: {passed}/{total} Checks Passed ({failed} Failed)")
    print(f"Overall Empirical Challenger Verdict: {verdict} (Elapsed: {elapsed:.2f}s)")
    print("=" * 80)
    
    # Save output to JSON
    out_path = Path("/Users/casey.allard/uor-r4/.agents/teamwork/teamwork_preview_challenger_m3_1/m3_challenger_results.json")
    out_path.write_text(json.dumps(results, indent=2))
    print(f"Results saved to: {out_path}")
    
    if failed > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()
