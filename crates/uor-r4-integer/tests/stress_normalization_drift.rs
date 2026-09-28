use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use uor_r4_integer::{
    bundle::Bundle,
    config::ReadMode,
    generation::{Request, Selection},
    model::PROBABILITY_TOTAL,
};

const BUNDLE_QUAT_PATH: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1";
const BUNDLE_ORD_PATH: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-householder_pair-1";
const EVAL_QUAT_PATH: &str =
    "/Users/casey.allard/uor-r4-investigations/full-context-integer-20260925/eval-quaternion-2";
const EVAL_ORD_PATH: &str =
    "/Users/casey.allard/uor-r4-investigations/full-context-integer-20260925/eval-householder_pair-2";

fn load_quat_bundle() -> Bundle {
    Bundle::load(Path::new(BUNDLE_QUAT_PATH)).expect("quaternion bundle failed to load")
}

fn load_ord_bundle() -> Bundle {
    Bundle::load(Path::new(BUNDLE_ORD_PATH)).expect("householder_pair bundle failed to load")
}

// -------------------------------------------------------------------------
// 1. Adversarial Full 256-Step Repeated Token 0 (BOS)
// -------------------------------------------------------------------------

#[test]
fn test_adversarial_full256_repeated_bos_quaternion() {
    let bundle = load_quat_bundle();
    let model = bundle.model();
    let mut session = model.new_session();

    let mut min_coord = i32::MAX;
    let mut max_coord = i32::MIN;

    for step_idx in 0..256 {
        let step = model
            .step(&mut session, 0, ReadMode::Enabled)
            .unwrap_or_else(|e| panic!("Step {step_idx} failed: {e}"));

        // Invariant 1: Exact sum 2^48 for probability mass
        let prob_sum: u128 = step.probabilities.iter().map(|&p| p as u128).sum();
        assert_eq!(
            prob_sum, PROBABILITY_TOTAL as u128,
            "Step {step_idx}: Probability sum != 2^48 (got {prob_sum})"
        );

        // Invariant 2: Attention mass + no_read_mass == 2^48
        let read_sum: u128 = step.read_masses.iter().map(|&m| m as u128).sum();
        let attn_total = read_sum + (step.no_read_mass as u128);
        assert_eq!(
            attn_total, PROBABILITY_TOTAL as u128,
            "Step {step_idx}: Attention total != 2^48 (got {attn_total})"
        );

        // Invariant 3: Exposed causal slots matches exact step index
        assert_eq!(
            step.read_masses.len(),
            step_idx,
            "Step {step_idx}: Causal slot count mismatch"
        );

        // Invariant 4: State vector bounded and valid
        assert_eq!(step.state.len(), 256);
        for &coord in &step.state {
            assert!(
                (-32767..=32767).contains(&coord),
                "Step {step_idx}: State coordinate out of range: {coord}"
            );
            min_coord = min_coord.min(coord);
            max_coord = max_coord.max(coord);
        }

        // Invariant 5: No probability is zero (uniform mixture floor ensures > 0)
        for (token_id, &p) in step.probabilities.iter().enumerate() {
            assert!(
                p > 0,
                "Step {step_idx}: Zero probability at token {token_id}"
            );
        }
    }

    // Ensure session is now at full capacity 256
    assert_eq!(session.len(), 256);
    // 257th step must be rejected
    assert!(
        model.step(&mut session, 0, ReadMode::Enabled).is_err(),
        "257th step must fail due to context exhaustion"
    );

    println!(
        "Repeated BOS (Quaternion): Min coord = {min_coord}, Max coord = {max_coord} (healthy dynamic range)"
    );
}

#[test]
fn test_adversarial_full256_repeated_bos_householder() {
    let bundle = load_ord_bundle();
    let model = bundle.model();
    let mut session = model.new_session();

    let mut min_coord = i32::MAX;
    let mut max_coord = i32::MIN;

    for step_idx in 0..256 {
        let step = model
            .step(&mut session, 0, ReadMode::Enabled)
            .unwrap_or_else(|e| panic!("Step {step_idx} failed: {e}"));

        let prob_sum: u128 = step.probabilities.iter().map(|&p| p as u128).sum();
        assert_eq!(
            prob_sum, PROBABILITY_TOTAL as u128,
            "Step {step_idx}: Probability sum != 2^48"
        );

        let read_sum: u128 = step.read_masses.iter().map(|&m| m as u128).sum();
        let attn_total = read_sum + (step.no_read_mass as u128);
        assert_eq!(
            attn_total, PROBABILITY_TOTAL as u128,
            "Step {step_idx}: Attention total != 2^48"
        );

        assert_eq!(step.read_masses.len(), step_idx);

        for &coord in &step.state {
            assert!((-32767..=32767).contains(&coord));
            min_coord = min_coord.min(coord);
            max_coord = max_coord.max(coord);
        }
    }

    assert_eq!(session.len(), 256);
    assert!(model.step(&mut session, 0, ReadMode::Enabled).is_err());
    println!(
        "Repeated BOS (Householder): Min coord = {min_coord}, Max coord = {max_coord}"
    );
}

// -------------------------------------------------------------------------
// 2. Adversarial Full 256-Step Max Token Index (4095)
// -------------------------------------------------------------------------

#[test]
fn test_adversarial_full256_max_token_index() {
    for (name, bundle) in [
        ("quaternion", load_quat_bundle()),
        ("householder_pair", load_ord_bundle()),
    ] {
        let model = bundle.model();
        let mut session = model.new_session();
        let max_token = 4095u32;

        for step_idx in 0..256 {
            let step = model
                .step(&mut session, max_token, ReadMode::Enabled)
                .unwrap_or_else(|e| panic!("{name} Step {step_idx} failed: {e}"));

            let prob_sum: u128 = step.probabilities.iter().map(|&p| p as u128).sum();
            assert_eq!(
                prob_sum, PROBABILITY_TOTAL as u128,
                "{name} Step {step_idx}: Probability sum != 2^48"
            );

            let read_sum: u128 = step.read_masses.iter().map(|&m| m as u128).sum();
            let attn_total = read_sum + (step.no_read_mass as u128);
            assert_eq!(
                attn_total, PROBABILITY_TOTAL as u128,
                "{name} Step {step_idx}: Attention total != 2^48"
            );

            for &coord in &step.state {
                assert!((-32767..=32767).contains(&coord));
            }
        }
    }
}

// -------------------------------------------------------------------------
// 3. Adversarial Full 256-Step Alternating Extreme Tokens [0, 4095, ...]
// -------------------------------------------------------------------------

#[test]
fn test_adversarial_full256_alternating_tokens_and_noread() {
    for (name, bundle) in [
        ("quaternion", load_quat_bundle()),
        ("householder_pair", load_ord_bundle()),
    ] {
        let model = bundle.model();

        // Arm 1: ReadMode::Enabled
        let mut session_enabled = model.new_session();
        for step_idx in 0..256 {
            let token = if step_idx % 2 == 0 { 0 } else { 4095 };
            let step = model
                .step(&mut session_enabled, token, ReadMode::Enabled)
                .unwrap_or_else(|e| panic!("{name} Enabled step {step_idx} failed: {e}"));

            let prob_sum: u128 = step.probabilities.iter().map(|&p| p as u128).sum();
            assert_eq!(prob_sum, PROBABILITY_TOTAL as u128);
            let read_sum: u128 = step.read_masses.iter().map(|&m| m as u128).sum();
            assert_eq!(read_sum + step.no_read_mass as u128, PROBABILITY_TOTAL as u128);
        }

        // Arm 2: ReadMode::NoRead
        let mut session_noread = model.new_session();
        for step_idx in 0..256 {
            let token = if step_idx % 2 == 0 { 0 } else { 4095 };
            let step = model
                .step(&mut session_noread, token, ReadMode::NoRead)
                .unwrap_or_else(|e| panic!("{name} NoRead step {step_idx} failed: {e}"));

            let prob_sum: u128 = step.probabilities.iter().map(|&p| p as u128).sum();
            assert_eq!(prob_sum, PROBABILITY_TOTAL as u128);

            // In NoRead, no_read_mass must be exactly PROBABILITY_TOTAL and read_masses all 0
            assert_eq!(step.no_read_mass, PROBABILITY_TOTAL);
            assert!(
                step.read_masses.iter().all(|&m| m == 0),
                "{name} NoRead step {step_idx}: Non-zero read masses in NoRead mode"
            );
        }
    }
}

// -------------------------------------------------------------------------
// 4. Adversarial Full 256-Step Pseudo-Random Token Sequences
// -------------------------------------------------------------------------

#[test]
fn test_adversarial_full256_pseudo_random_sequences() {
    let bundle = load_quat_bundle();
    let model = bundle.model();

    // SplitMix64 PRNG generator
    let mut seed = 0x123456789ABCDEF0u64;
    let next_u64 = |s: &mut u64| -> u64 {
        *s = s.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = *s;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    };

    for run_idx in 0..4 {
        let mut session = model.new_session();
        for step_idx in 0..256 {
            let token = (next_u64(&mut seed) % 4096) as u32;
            let step = model
                .step(&mut session, token, ReadMode::Enabled)
                .unwrap_or_else(|e| panic!("Run {run_idx} step {step_idx} failed: {e}"));

            let prob_sum: u128 = step.probabilities.iter().map(|&p| p as u128).sum();
            assert_eq!(
                prob_sum, PROBABILITY_TOTAL as u128,
                "Run {run_idx} Step {step_idx}: Prob sum mismatch"
            );

            let read_sum: u128 = step.read_masses.iter().map(|&m| m as u128).sum();
            assert_eq!(
                read_sum + step.no_read_mass as u128,
                PROBABILITY_TOTAL as u128
            );
        }
    }
}

// -------------------------------------------------------------------------
// 5. Autoregressive Generation Reaching Context Limit 256
// -------------------------------------------------------------------------

#[test]
fn test_max_generation_255_tokens_capacity() {
    for (name, bundle) in [
        ("quaternion", load_quat_bundle()),
        ("householder_pair", load_ord_bundle()),
    ] {
        // Prompt of 1 token ("A") + BOS (1) + 254 generated tokens = 256 exact capacity
        let prompt_tokens = bundle.tokenizer().encode("A");
        assert_eq!(prompt_tokens.len(), 1, "Single token prompt expected");

        let req = Request {
            prompt: "A".into(),
            max_new_tokens: 254,
            selection: Selection::Greedy,
            read_mode: ReadMode::Enabled,
            first_sentence: false,
        };

        let gen = bundle
            .generate(&req)
            .unwrap_or_else(|e| panic!("{name} generate failed: {e}"));

        assert_eq!(gen.prompt_token_ids.len(), 2); // [0 (BOS), token("A")]
        assert!(!gen.decisions.is_empty());

        for (dec_idx, decision) in gen.decisions.iter().enumerate() {
            assert_eq!(
                decision.probability_sum_q48, PROBABILITY_TOTAL,
                "{name} Decision {dec_idx}: probability sum != 2^48"
            );
            assert!(
                decision.no_read_mass_q48 <= PROBABILITY_TOTAL,
                "{name} Decision {dec_idx}: no_read_mass exceeds 2^48"
            );
            // Decision index corresponds to exposed causal slots
            assert_eq!(
                decision.exposed_causal_slots,
                dec_idx + 1,
                "{name} Decision {dec_idx}: exposed causal slots mismatch"
            );
        }

        println!(
            "{name}: Generated {} tokens, reached causal slots {}",
            gen.generated_token_ids.len(),
            gen.decisions.last().map(|d| d.exposed_causal_slots).unwrap_or(0)
        );
    }
}

// -------------------------------------------------------------------------
// 6. Empirical Drift Trajectory Analysis Over 256 Steps vs Float Reference
// -------------------------------------------------------------------------

#[test]
fn test_drift_trajectory_across_256_causal_steps() {
    for (name, eval_dir) in [
        ("quaternion", EVAL_QUAT_PATH),
        ("householder_pair", EVAL_ORD_PATH),
    ] {
        let targets_file = Path::new(eval_dir).join("targets-read.jsonl");
        let reader = BufReader::new(File::open(&targets_file).expect("Failed to open targets-read.jsonl"));

        let mut max_state_delta = 0.0f64;
        let mut max_prob_delta = 0.0f64;
        let mut max_tv = 0.0f64;
        let mut position_state_deltas = vec![0.0f64; 256];
        let mut position_counts = vec![0usize; 256];

        let mut total_rows = 0;
        for line in reader.lines() {
            let line = line.unwrap();
            let row: serde_json::Value = serde_json::from_str(&line).unwrap();

            let pos = row["position_in_block"].as_u64().unwrap() as usize;
            let state_delta = row["maximum_state_delta"].as_f64().unwrap();
            let prob_delta = row["maximum_probability_delta"].as_f64().unwrap();
            let tv = row["total_variation"].as_f64().unwrap();

            max_state_delta = max_state_delta.max(state_delta);
            max_prob_delta = max_prob_delta.max(prob_delta);
            max_tv = max_tv.max(tv);

            position_state_deltas[pos] = position_state_deltas[pos].max(state_delta);
            position_counts[pos] += 1;
            total_rows += 1;

            // Invariant: Prospective bounds <= 0.01 at EVERY SINGLE STEP
            assert!(
                state_delta <= 0.01,
                "{name} position {pos}: state delta {state_delta} > 0.01"
            );
            assert!(
                prob_delta <= 0.01,
                "{name} position {pos}: prob delta {prob_delta} > 0.01"
            );
        }

        assert_eq!(total_rows, 1024, "{name} expected 1024 targets across 4 windows");

        // Inspect drift at key horizons: pos 0, pos 50, pos 100, pos 200, pos 255
        println!(
            "{name} Step Drift Horizons (Max State Delta across 4 windows):"
        );
        for &p in &[0, 1, 10, 50, 100, 150, 200, 250, 255] {
            println!(
                "  Position {:3}: State Delta = {:.8}",
                p, position_state_deltas[p]
            );
        }
        println!(
            "{name} Global Peak State Delta: {:.8} (limit 0.01)",
            max_state_delta
        );
        println!(
            "{name} Global Peak Prob Delta:  {:.8} (limit 0.01)",
            max_prob_delta
        );
        println!(
            "{name} Global Peak Total Var:   {:.8}",
            max_tv
        );

        assert!(max_state_delta <= 0.01);
        assert!(max_prob_delta <= 0.01);
    }
}

// -------------------------------------------------------------------------
// 7. Memory Read & Copy-Gate Under Extreme Token Repetition
// -------------------------------------------------------------------------

#[test]
fn test_memory_read_copy_gate_extreme_repetition() {
    for (name, bundle) in [
        ("quaternion", load_quat_bundle()),
        ("householder_pair", load_ord_bundle()),
    ] {
        // Construct prompt with 80 repetitions of token " apple"
        let rep_prompt = "apple ".repeat(80);
        let token_count = bundle.tokenizer().encode(&rep_prompt).len();
        assert!(token_count >= 80 && token_count <= 200);

        let req = Request {
            prompt: rep_prompt,
            max_new_tokens: 15,
            selection: Selection::Greedy,
            read_mode: ReadMode::Enabled,
            first_sentence: false,
        };

        let gen = bundle
            .generate(&req)
            .unwrap_or_else(|e| panic!("{name} repeat generation failed: {e}"));

        assert!(!gen.decisions.is_empty());
        for (idx, d) in gen.decisions.iter().enumerate() {
            assert_eq!(
                d.probability_sum_q48, PROBABILITY_TOTAL,
                "{name} Step {idx}: Probability sum != 2^48 under massive repetition"
            );
            assert!(
                d.no_read_mass_q48 <= PROBABILITY_TOTAL,
                "{name} Step {idx}: no_read_mass invalid"
            );
            // Historical slots exposed at decision idx: BOS (1) + (token_count - 1) prior prompt tokens + idx generated tokens
            assert_eq!(d.exposed_causal_slots, token_count + idx);
        }
    }
}

// -------------------------------------------------------------------------
// 8. Multi-Turn Session Continuous State Evolution Up to Capacity
// -------------------------------------------------------------------------

#[test]
fn test_multiturn_drift_and_headroom_up_to_capacity() {
    let bundle = load_quat_bundle();
    let mut session = bundle
        .text_session(ReadMode::Enabled)
        .expect("text_session failed");

    let phrases = [
        "The quick brown fox jumps over the lazy dog.",
        "A quiet pond reflected the evening stars.",
        "Deep in the forest, ancient trees stood tall.",
        "The clock struck twelve in the quiet town.",
        "Morning light broke through the silver mist.",
        "A gentle breeze whispered through the open window.",
        "Leaves drifted silently onto the wet stone path.",
        "The travelers rested beside the flowing stream.",
    ];

    for (turn, phrase) in phrases.iter().enumerate() {
        let added = session.append(phrase).unwrap();
        let gen = session.generate(5, Selection::Greedy, false).unwrap();

        for d in &gen.decisions {
            assert_eq!(
                d.probability_sum_q48, PROBABILITY_TOTAL,
                "Turn {turn}: Probability sum != 2^48"
            );
            assert!(
                d.no_read_mass_q48 <= PROBABILITY_TOTAL,
                "Turn {turn}: Attention mass invalid"
            );
        }

        println!(
            "Turn {:2}: appended {:2} tokens, gen 5 tokens, session total = {:3}/256",
            turn + 1,
            added,
            session.len()
        );
    }

    assert!(session.len() <= 256);
}

