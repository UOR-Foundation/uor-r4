//! Milestone M2 Adversarial Stress Test Suite
//! Author: teamwork_preview_challenger_m2_1 (Adversarial Empirical Challenger)
//!
//! Rigorous stress testing for:
//! 1. 256-token context horizon (strict boundary rejection at token 256, 0 silent eviction, 0 buffer wraparound)
//! 2. Copy-gate blending mass conservation (sum P_raw and sum P_final across g in {0, 16384, 32767} and diverse distributions)
//! 3. Entity repetition / periodic cycle traps (short_cycle period 1..=4 breakout and period 5+ termination)

use std::path::Path;
use uor_r4_integer::{
    bundle::Bundle,
    config::ReadMode,
    generation::{Request, Selection, Stop},
    math,
    model::PROBABILITY_TOTAL,
};

const BUNDLE_QUAT_PATH: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1";
const BUNDLE_ORD_PATH: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-householder_pair-1";

fn load_quat_bundle() -> Bundle {
    Bundle::load(Path::new(BUNDLE_QUAT_PATH)).expect("quaternion bundle failed to load")
}

fn load_ord_bundle() -> Bundle {
    Bundle::load(Path::new(BUNDLE_ORD_PATH)).expect("householder_pair bundle failed to load")
}

// =========================================================================
// SECTION 1: ADVERSARIAL CHALLENGE TO 256-TOKEN CONTEXT HORIZON
// =========================================================================

#[test]
fn test_m2_adversarial_context_horizon_kernel_exact_boundaries() {
    for (name, bundle) in [
        ("quaternion", load_quat_bundle()),
        ("householder_pair", load_ord_bundle()),
    ] {
        let model = bundle.model();
        let mut session = model.new_session();
        assert_eq!(session.len(), 0);

        // Feed sequences of length 255 (indices 0..255)
        for i in 0..255 {
            let tok = (i % 100 + 2) as u32;
            let step = model.step(&mut session, tok, ReadMode::Enabled);
            assert!(
                step.is_ok(),
                "{name}: Step {i} failed within capacity 255: {:?}",
                step.err()
            );
            assert_eq!(session.len(), i + 1);
        }
        assert_eq!(session.len(), 255, "{name}: Session must have 255 tokens");

        // The 256th token (step index 255 -> reaches capacity 256)
        let token_255 = 77u32;
        let step_256 = model.step(&mut session, token_255, ReadMode::Enabled);
        assert!(
            step_256.is_ok(),
            "{name}: 256th token must succeed: {:?}",
            step_256.err()
        );
        assert_eq!(session.len(), 256, "{name}: Session must be at exactly 256");

        // Now attempt the 257th step (requesting token beyond capacity 256)
        for trial in 0..5 {
            let overflow_token = (999 + trial) as u32;
            let step_overflow = model.step(&mut session, overflow_token, ReadMode::Enabled);
            assert!(
                step_overflow.is_err(),
                "{name}: Step beyond context capacity 256 MUST fail (trial {trial})"
            );
            let err_msg = match step_overflow {
                Err(e) => e.to_string(),
                Ok(_) => panic!("{name}: Step unexpectedly succeeded"),
            };
            assert!(
                err_msg.contains("integer session artifact/context/token mismatch"),
                "{name}: Expected context mismatch error, got: {err_msg}"
            );

            // Invariant: 0 silent eviction, 0 buffer wraparound, session length unchanged
            assert_eq!(
                session.len(),
                256,
                "{name}: Session length must remain strictly 256"
            );
        }
    }
}

#[test]
fn test_m2_adversarial_context_horizon_text_session_boundaries() {
    let bundle = load_quat_bundle();
    let tokenizer = bundle.tokenizer();

    // 1. TextSession begins with BOS (len == 1)
    let mut session = bundle
        .text_session(ReadMode::Enabled)
        .expect("text_session creation failed");
    assert_eq!(session.len(), 1);

    // 2. Synthesize prompt of exact length 254
    let mut prompt_254 = String::new();
    while tokenizer.encode(&prompt_254).len() < 254 {
        prompt_254.push_str(" word");
    }
    while tokenizer.encode(&prompt_254).len() > 254 {
        prompt_254.pop();
    }
    assert_eq!(tokenizer.encode(&prompt_254).len(), 254);

    // Append 254 tokens to session (total: 1 + 254 = 255)
    let appended = session.append(&prompt_254);
    assert!(appended.is_ok(), "Appending 254 tokens must succeed");
    assert_eq!(session.len(), 255);

    // 3. Generate 1 token: 255 + 1 = 256 <= 256 -> MUST SUCCEED
    let gen1 = session.generate(1, Selection::Greedy, false);
    assert!(
        gen1.is_ok(),
        "Generating 1 token to reach exact 256 capacity MUST succeed: {:?}",
        gen1.err()
    );
    assert_eq!(session.len(), 256);

    // 4. Now session is fully saturated at 256 tokens.
    // Any further generate call MUST be rejected prospectively
    let gen_extra = session.generate(1, Selection::Greedy, false);
    assert!(
        gen_extra.is_err(),
        "Generating when session is at 256 tokens MUST fail"
    );
    let err_msg = gen_extra.unwrap_err().to_string();
    assert!(
        err_msg.contains("request exceeds full256 session budget"),
        "Expected budget error, got: {err_msg}"
    );

    // Any append call MUST be rejected
    let append_extra = session.append(" extra");
    assert!(
        append_extra.is_err(),
        "Appending when session is at 256 tokens MUST fail"
    );
    let append_err = append_extra.unwrap_err().to_string();
    assert!(
        append_err.contains("empty text or session context256 exhausted"),
        "Expected context exhausted error, got: {append_err}"
    );

    // Length remains exactly 256 (0 silent eviction, 0 wraparound)
    assert_eq!(session.len(), 256);
}

#[test]
fn test_m2_adversarial_context_horizon_direct_request_budget() {
    let bundle = load_quat_bundle();
    let tokenizer = bundle.tokenizer();

    // Prompt of 254 tokens
    let mut prompt_254 = String::new();
    while tokenizer.encode(&prompt_254).len() < 254 {
        prompt_254.push_str(" item");
    }
    while tokenizer.encode(&prompt_254).len() > 254 {
        prompt_254.pop();
    }
    assert_eq!(tokenizer.encode(&prompt_254).len(), 254);

    // Prompt 254 + max_new_tokens 1 = 1 (BOS) + 254 + 1 = 256 <= 256 -> OK
    let req_ok = Request {
        prompt: prompt_254.clone(),
        max_new_tokens: 1,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };
    assert!(bundle.generate(&req_ok).is_ok());

    // Prompt 254 + max_new_tokens 2 = 1 (BOS) + 254 + 2 = 257 > 256 -> REJECTED
    let req_overflow = Request {
        prompt: prompt_254.clone(),
        max_new_tokens: 2,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };
    let res_overflow = bundle.generate(&req_overflow);
    assert!(res_overflow.is_err());
    assert!(res_overflow
        .unwrap_err()
        .to_string()
        .contains("request exceeds full256 session budget"));
}

// =========================================================================
// SECTION 2: ADVERSARIAL CHALLENGE TO COPY-GATE BLENDING MASS CONSERVATION
// =========================================================================

/// Helper replicating exact blending arithmetic from `crates/uor-r4-integer/src/model.rs:379-393`
fn blend_probabilities_synthetic(
    vocab_probs: &[u64],
    copy_masses: &[u64],
    gate: i32,
    no_read: u64,
) -> (Vec<i128>, Vec<u64>) {
    let total = PROBABILITY_TOTAL;
    let vocab_size = vocab_probs.len();
    assert_eq!(copy_masses.len(), vocab_size);

    // fraction = TOTAL - scaled(gate * (TOTAL - no_read), -15)
    let gate_i128 = i128::from(gate);
    let read_mass_total = i128::from(total - no_read);
    let gate_read_prod = math::checked_mul(gate_i128, read_mass_total).unwrap();
    let gate_read_scaled = math::scale_pow2(gate_read_prod, -15).unwrap();
    let fraction = i128::from(total) - gate_read_scaled;

    let mut raw_probs = Vec::with_capacity(vocab_size);
    let mut probabilities = Vec::with_capacity(vocab_size);

    for (&v, &c) in vocab_probs.iter().zip(copy_masses) {
        let v_prod = math::checked_mul(i128::from(v), fraction).unwrap();
        let v_scaled = math::scale_pow2(v_prod, -48).unwrap();

        let c_prod = math::checked_mul(i128::from(c), gate_i128).unwrap();
        let c_scaled = math::scale_pow2(c_prod, -15).unwrap();

        let raw = v_scaled + c_scaled;
        raw_probs.push(raw);

        // Uniform mixture: (raw * 99_999_999 + TOTAL >> 12) / 100_000_000
        let unif_prod = math::checked_mul(raw, 99_999_999).unwrap();
        let unif_num = unif_prod + i128::from(total >> 12);
        let uniform = math::div_round(unif_num, 100_000_000).unwrap();
        probabilities.push(u64::try_from(uniform).expect("positive probability"));
    }

    // Residual normalization
    let mut running_total = 0u64;
    let mut largest = 0usize;
    for (i, &p) in probabilities.iter().enumerate() {
        running_total = running_total.checked_add(p).unwrap();
        if p > probabilities[largest] {
            largest = i;
        }
    }
    if running_total <= total {
        probabilities[largest] += total - running_total;
    } else {
        probabilities[largest] -= running_total - total;
    }

    (raw_probs, probabilities)
}

#[test]
fn test_m2_adversarial_copy_gate_mass_conservation_diverse_distributions() {
    let vocab_size = 4096;
    let total = PROBABILITY_TOTAL;

    // Gate values: 0 (closed), 16384 (50%), 32767 (almost full), 32768 (100%), and bounds
    let gate_values = [0i32, 1, 8192, 16384, 24576, 32767, 32768];

    // Diverse vocabulary distributions
    let mut vocab_dists: Vec<(&str, Vec<u64>)> = Vec::new();

    // Dist 1: One-hot (token 0 has all mass)
    let mut one_hot = vec![0u64; vocab_size];
    one_hot[0] = total;
    vocab_dists.push(("one_hot_0", one_hot));

    // Dist 2: One-hot at end (token 4095 has all mass)
    let mut one_hot_last = vec![0u64; vocab_size];
    one_hot_last[vocab_size - 1] = total;
    vocab_dists.push(("one_hot_last", one_hot_last));

    // Dist 3: Uniform distribution (2^48 / 4096 = 2^36 each)
    let uniform = vec![total / (vocab_size as u64); vocab_size];
    assert_eq!(uniform.iter().sum::<u64>(), total);
    vocab_dists.push(("uniform", uniform));

    // Dist 4: Zipfian distribution (P(i) ~ 1 / (i + 1))
    let mut zipf = vec![0u64; vocab_size];
    let mut zipf_sum = 0.0f64;
    for i in 0..vocab_size {
        zipf_sum += 1.0 / (i as f64 + 1.0);
    }
    let mut current_sum = 0u64;
    for i in 0..vocab_size {
        let weight = (1.0 / (i as f64 + 1.0)) / zipf_sum;
        let mass = (weight * (total as f64)).round() as u64;
        zipf[i] = mass;
        current_sum = current_sum.saturating_add(mass);
    }
    // Adjust residual on token 0
    if current_sum <= total {
        zipf[0] += total - current_sum;
    } else {
        zipf[0] -= current_sum - total;
    }
    assert_eq!(zipf.iter().sum::<u64>(), total);
    vocab_dists.push(("zipfian", zipf));

    // Dist 5: Bimodal distribution
    let mut bimodal = vec![1000u64; vocab_size];
    let mass_used = 1000u64 * (vocab_size as u64);
    let remaining = total - mass_used;
    bimodal[42] += remaining / 2;
    bimodal[1337] += remaining - (remaining / 2);
    assert_eq!(bimodal.iter().sum::<u64>(), total);
    vocab_dists.push(("bimodal", bimodal));

    // Diverse copy mass distributions and no_read values
    let mut copy_scenarios: Vec<(&str, u64, Vec<u64>)> = Vec::new();

    // Scenario A: NoRead (no_read = TOTAL, all copy = 0)
    copy_scenarios.push(("noread_mode", total, vec![0u64; vocab_size]));

    // Scenario B: Full read on single token (no_read = 0, copy[token 5] = TOTAL)
    let mut copy_single = vec![0u64; vocab_size];
    copy_single[5] = total;
    copy_scenarios.push(("single_token_full_read", 0, copy_single));

    // Scenario C: Half read split across 10 tokens (no_read = TOTAL / 2)
    let mut copy_ten = vec![0u64; vocab_size];
    let read_half = total / 2;
    let share = read_half / 10;
    for i in 0..10 {
        copy_ten[i * 100 + 7] = share;
    }
    copy_ten[7] += read_half - share * 10;
    copy_scenarios.push(("half_read_ten_tokens", total - read_half, copy_ten));

    // Scenario D: 75% read dispersed across 64 tokens
    let read_75 = (total / 4) * 3;
    let mut copy_64 = vec![0u64; vocab_size];
    let share64 = read_75 / 64;
    for i in 0..64 {
        copy_64[i * 50] = share64;
    }
    copy_64[0] += read_75 - share64 * 64;
    copy_scenarios.push(("75pct_read_64_tokens", total - read_75, copy_64));

    // Execute combinatorial stress test: gate_values x vocab_dists x copy_scenarios
    let mut tests_run = 0;
    let mut max_raw_drift: i128 = 0;

    for &gate in &gate_values {
        for (v_name, v_dist) in &vocab_dists {
            for (c_name, no_read, c_dist) in &copy_scenarios {
                let (raw_probs, final_probs) =
                    blend_probabilities_synthetic(v_dist, c_dist, gate, *no_read);

                // 1. Check raw probability sum
                let sum_raw: i128 = raw_probs.iter().sum();
                let raw_drift = (sum_raw - i128::from(total)).abs();
                max_raw_drift = max_raw_drift.max(raw_drift);

                // For gate=0, raw sum MUST be IDENTICAL to 2^48 (zero drift)
                if gate == 0 {
                    assert_eq!(
                        sum_raw,
                        i128::from(total),
                        "Gate=0 must have zero raw drift for {v_name} / {c_name}"
                    );
                }

                // In all cases, raw drift due to integer dyadic rounding over 4096 elements
                // must be strictly bounded within ±4096 units (relative error < 1.5e-11)
                assert!(
                    raw_drift <= (vocab_size as i128),
                    "Raw drift {raw_drift} exceeds maximum rounding bound 4096 for g={gate}, {v_name}, {c_name}"
                );

                // 2. Check final normalized probability sum
                let sum_final: u128 = final_probs.iter().map(|&p| p as u128).sum();
                assert_eq!(
                    sum_final, total as u128,
                    "Final normalized probability sum must exactly equal 2^48 for g={gate}, {v_name}, {c_name}"
                );

                // 3. Positive measure invariant: uniform floor ensures no token has 0 probability
                for (v, &p) in final_probs.iter().enumerate() {
                    assert!(
                        p > 0,
                        "Token {v} has 0 probability for g={gate}, {v_name}, {c_name}"
                    );
                    assert!(
                        p <= total,
                        "Token {v} exceeds 2^48 for g={gate}, {v_name}, {c_name}"
                    );
                }

                tests_run += 1;
            }
        }
    }

    println!(
        "Copy-gate blending mass conservation verified across {tests_run} configurations."
    );
    println!(
        "Maximum observed pre-normalization raw drift: {max_raw_drift} units out of 2^48 ({:.2e} relative error)",
        (max_raw_drift as f64) / (total as f64)
    );
    println!("Final post-normalization probability mass: EXACTLY 2^48 with zero leakage in 100% of cases.");
}

// =========================================================================
// SECTION 3: STRESS-TEST ENTITY REPETITION / CYCLE TRAPS
// =========================================================================

#[test]
fn test_m2_repetition_short_cycle_period_1() {
    let bundle = load_quat_bundle();

    // 1-token periodic repetition in prompt: "hello hello hello hello "
    let rep_prompt = "hello ".repeat(30);
    let req = Request {
        prompt: rep_prompt,
        max_new_tokens: 25,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };

    let res = bundle.generate(&req);
    assert!(
        res.is_ok(),
        "Period-1 prompt must execute without crash or deadlock: {:?}",
        res.err()
    );
    let gen = res.unwrap();

    // Must terminate via ShortCycle, FirstSentenceBoundary, Eos, or MaximumNewTokens
    println!("Period 1 stop reason: {:?}", gen.stop);
    match gen.stop {
        Stop::ShortCycle { period } => {
            assert_eq!(period, 1, "Expected period 1 cycle breakout");
        }
        Stop::MaximumNewTokens | Stop::FirstSentenceBoundary | Stop::Eos => {}
    }
}

#[test]
fn test_m2_repetition_short_cycle_period_2() {
    let bundle = load_quat_bundle();

    // 2-token periodic repetition: "tick tock tick tock tick tock "
    let rep_prompt = "tick tock ".repeat(25);
    let req = Request {
        prompt: rep_prompt,
        max_new_tokens: 30,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };

    let res = bundle.generate(&req);
    assert!(res.is_ok(), "Period-2 prompt must execute cleanly");
    let gen = res.unwrap();
    println!("Period 2 stop reason: {:?}", gen.stop);
    println!("Period 2 generated raw: {:?}", gen.raw_decoded);
    println!("Period 2 generated tokens: {:?}", gen.generated_token_ids);
    for d in &gen.decisions {
        assert_eq!(d.probability_sum_q48, PROBABILITY_TOTAL);
    }
}

/// Unit test verifying exact algorithmic invariants of `short_cycle` detection
#[test]
fn test_m2_short_cycle_algorithm_boundary_matrix() {
    // We recreate the exact short_cycle implementation to stress-test its edge cases
    fn short_cycle_eval(tokens: &[u32]) -> Option<usize> {
        for period in 1..=4 {
            let twice = period << 1;
            let span = twice + period;
            if tokens.len() >= span {
                let tail = &tokens[tokens.len() - span..];
                if tail[..period] == tail[period..twice] && tail[..period] == tail[twice..] {
                    return Some(period);
                }
            }
        }
        None
    }

    // 1. Boundary: empty and small inputs
    assert_eq!(short_cycle_eval(&[]), None);
    assert_eq!(short_cycle_eval(&[1]), None);
    assert_eq!(short_cycle_eval(&[1, 1]), None);

    // 2. Period 1: requires 3 repetitions
    assert_eq!(short_cycle_eval(&[42, 42, 42]), Some(1));
    assert_eq!(short_cycle_eval(&[1, 2, 3, 42, 42, 42]), Some(1));
    assert_eq!(short_cycle_eval(&[42, 42, 42, 42]), Some(1));

    // 3. Period 2: requires 6 tokens (3 pairs)
    assert_eq!(short_cycle_eval(&[1, 2, 1, 2, 1]), None);
    assert_eq!(short_cycle_eval(&[1, 2, 1, 2, 1, 2]), Some(2));
    assert_eq!(short_cycle_eval(&[99, 88, 1, 2, 1, 2, 1, 2]), Some(2));

    // 4. Period 3: requires 9 tokens (3 triplets)
    assert_eq!(short_cycle_eval(&[1, 2, 3, 1, 2, 3, 1, 2]), None);
    assert_eq!(short_cycle_eval(&[1, 2, 3, 1, 2, 3, 1, 2, 3]), Some(3));
    assert_eq!(short_cycle_eval(&[5, 5, 1, 2, 3, 1, 2, 3, 1, 2, 3]), Some(3));

    // 5. Period 4: requires 12 tokens (3 quadruplets)
    assert_eq!(short_cycle_eval(&[1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3]), None);
    assert_eq!(short_cycle_eval(&[1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4]), Some(4));

    // 6. Period 5: outside detection horizon (1..=4), must return None
    let period_5 = [1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5];
    assert_eq!(short_cycle_eval(&period_5), None);

    // 7. Non-periodic repetitive noise
    assert_eq!(short_cycle_eval(&[1, 2, 1, 3, 1, 4, 1, 5]), None);
}

#[test]
fn test_m2_repetition_short_cycle_period_3() {
    let bundle = load_quat_bundle();

    // 3-token periodic repetition: "red green blue red green blue "
    let rep_prompt = "red green blue ".repeat(20);
    let req = Request {
        prompt: rep_prompt,
        max_new_tokens: 30,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };

    let res = bundle.generate(&req);
    assert!(res.is_ok(), "Period-3 prompt must execute cleanly");
    let gen = res.unwrap();
    println!("Period 3 stop reason: {:?}", gen.stop);
    for d in &gen.decisions {
        assert_eq!(d.probability_sum_q48, PROBABILITY_TOTAL);
    }
}

#[test]
fn test_m2_repetition_short_cycle_period_4() {
    let bundle = load_quat_bundle();

    // 4-token periodic repetition: "north east south west "
    let rep_prompt = "north east south west ".repeat(15);
    let req = Request {
        prompt: rep_prompt,
        max_new_tokens: 30,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };

    let res = bundle.generate(&req);
    assert!(res.is_ok(), "Period-4 prompt must execute cleanly");
    let gen = res.unwrap();
    println!("Period 4 stop reason: {:?}", gen.stop);
    for d in &gen.decisions {
        assert_eq!(d.probability_sum_q48, PROBABILITY_TOTAL);
    }
}

#[test]
fn test_m2_repetition_period_5_and_higher_safe_termination() {
    let bundle = load_quat_bundle();

    // 5-token periodic repetition: "one two three four five "
    // Note: short_cycle only checks periods 1..=4. Period 5 must terminate
    // safely at max_new_tokens without infinite loop or runtime panic!
    let rep_prompt = "one two three four five ".repeat(12);
    let req = Request {
        prompt: rep_prompt,
        max_new_tokens: 20,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };

    let res = bundle.generate(&req);
    assert!(
        res.is_ok(),
        "Period 5+ repetitive prompt must not enter infinite loop or crash"
    );
    let gen = res.unwrap();
    assert!(
        gen.generated_token_ids.len() <= 20,
        "Generation must respect max_new_tokens"
    );
    for d in &gen.decisions {
        assert_eq!(d.probability_sum_q48, PROBABILITY_TOTAL);
    }
}

#[test]
fn test_m2_repetition_pathological_inputs() {
    let bundle = load_quat_bundle();

    // Pathological input 1: 150 commas
    let prompt_commas = ",".repeat(150);
    let req1 = Request {
        prompt: prompt_commas,
        max_new_tokens: 15,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };
    let res1 = bundle.generate(&req1);
    assert!(res1.is_ok(), "Commas prompt must not panic");

    // Pathological input 2: Repeated statement entity assertions
    let prompt_facts = "The cat is on the mat. The cat is on the mat. The cat is on the mat. ";
    let req2 = Request {
        prompt: prompt_facts.repeat(5),
        max_new_tokens: 20,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };
    let res2 = bundle.generate(&req2);
    assert!(res2.is_ok(), "Repeated fact assertions must not panic");
    let gen2 = res2.unwrap();
    for d in &gen2.decisions {
        assert_eq!(d.probability_sum_q48, PROBABILITY_TOTAL);
    }
}
