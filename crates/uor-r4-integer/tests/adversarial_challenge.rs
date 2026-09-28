use std::path::Path;
use uor_r4_integer::{
    bundle::Bundle,
    config::ReadMode,
    generation::{Request, Selection, Stop},
    math,
    model::PROBABILITY_TOTAL,
    sampling::{SamplePolicy, Sampler},
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

// -------------------------------------------------------------------------
// 1. Boundary Inputs: Empty Prompt, Whitespace, Null Bytes, Unicode
// -------------------------------------------------------------------------

#[test]
fn test_boundary_empty_and_whitespace_prompts() {
    let bundle = load_quat_bundle();

    // 1. Completely empty prompt -> rejected by validate_budget
    let req_empty = Request {
        prompt: "".into(),
        max_new_tokens: 10,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };
    let res_empty = bundle.generate(&req_empty);
    assert!(
        res_empty.is_err(),
        "Empty prompt must be rejected by validate_budget"
    );
    let err_str = match res_empty {
        Err(e) => e.to_string(),
        Ok(_) => unreachable!(),
    };
    assert!(
        err_str.contains("empty prompt"),
        "Expected 'empty prompt', got: {err_str}"
    );

    // 2. Whitespace prompt: "   "
    // Does tokenizer produce tokens or empty?
    let ws_tokens = bundle.tokenizer().encode("   ");
    let req_ws = Request {
        prompt: "   ".into(),
        max_new_tokens: 5,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };
    let res_ws = bundle.generate(&req_ws);
    if ws_tokens.is_empty() {
        assert!(res_ws.is_err(), "If tokenizer produces 0 tokens, must error");
    } else {
        assert!(res_ws.is_ok(), "If whitespace has tokens, must generate without panic");
        let gen = res_ws.unwrap();
        for d in &gen.decisions {
            assert_eq!(d.probability_sum_q48, PROBABILITY_TOTAL);
        }
    }

    // 3. Null bytes in prompt
    let req_null = Request {
        prompt: "Hello\0World".into(),
        max_new_tokens: 4,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };
    let res_null = bundle.generate(&req_null);
    assert!(
        res_null.is_ok(),
        "Prompt containing null bytes must not crash: {:?}",
        res_null.err()
    );

    // 4. Multibyte UTF-8 and Emoji
    let req_unicode = Request {
        prompt: "The quick brown 🦊 jumps over 123 🌟!".into(),
        max_new_tokens: 8,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };
    let res_unicode = bundle.generate(&req_unicode);
    assert!(
        res_unicode.is_ok(),
        "Unicode prompt must succeed: {:?}",
        res_unicode.err()
    );
    let gen_u = res_unicode.unwrap();
    assert!(!gen_u.generated_token_ids.is_empty());
}

// -------------------------------------------------------------------------
// 2. Token ID Overflow and Out-of-Bounds Boundaries
// -------------------------------------------------------------------------

#[test]
fn test_token_id_overflow_boundary() {
    let bundle = load_quat_bundle();
    let model = bundle.model();
    let vocab_size = model.config().vocab_size;
    assert_eq!(vocab_size, 4096, "Expected vocab size 4096");

    let mut session = model.new_session();

    // Valid tokens: 0 (BOS), 1 (EOS), 4095 (last valid)
    assert!(model.step(&mut session, 0, ReadMode::Enabled).is_ok());
    assert!(model.step(&mut session, 1, ReadMode::Enabled).is_ok());
    assert!(model.step(&mut session, 4095, ReadMode::Enabled).is_ok());

    let state_before = session.len();

    // Out-of-bounds tokens: 4096 (exact boundary), 4097, 65535, u32::MAX
    let oob_tokens = [4096u32, 4097u32, 65535u32, 100000u32, u32::MAX];
    for &oob in &oob_tokens {
        let res = model.step(&mut session, oob, ReadMode::Enabled);
        assert!(
            res.is_err(),
            "Token {oob} is out of bounds (vocab_size={vocab_size}) and must error"
        );
        let err_msg = match res {
            Err(e) => e.to_string(),
            Ok(_) => panic!("Token {oob} unexpectedly succeeded"),
        };
        assert!(
            err_msg.contains("integer session artifact/context/token mismatch"),
            "Expected token mismatch error for {oob}, got: {err_msg}"
        );
        // Session state must NOT be modified or corrupted
        assert_eq!(
            session.len(),
            state_before,
            "Failed step must not mutate session state"
        );
    }
}

// -------------------------------------------------------------------------
// 3. Sequence Length Boundaries: 254, 255, 256, 257
// -------------------------------------------------------------------------

#[test]
fn test_sequence_length_kernel_step_boundaries() {
    let bundle = load_quat_bundle();
    let model = bundle.model();
    let mut session = model.new_session();

    // Ingest tokens one by one up to context limit 256
    for i in 0..256 {
        assert_eq!(session.len(), i);
        let tok = (i % 100 + 2) as u32; // Normal tokens
        let step = model.step(&mut session, tok, ReadMode::Enabled);
        assert!(
            step.is_ok(),
            "Step {i} must succeed within context 256"
        );
        let s = match step {
            Ok(s) => s,
            Err(e) => panic!("Step {i} failed: {e}"),
        };

        // Invariant check at EVERY single step:
        let total_prob: u64 = s.probabilities.iter().sum();
        assert_eq!(
            total_prob, PROBABILITY_TOTAL,
            "Step {i}: Total probability must exactly equal 2^48"
        );

        let total_read: u64 = s.no_read_mass + s.read_masses.iter().sum::<u64>();
        assert_eq!(
            total_read, PROBABILITY_TOTAL,
            "Step {i}: Total attention read mass must exactly equal 2^48"
        );
        assert_eq!(
            s.read_masses.len(),
            i,
            "Step {i}: Read masses count must match previous session length"
        );
    }

    assert_eq!(session.len(), 256, "Session must now have exactly 256 tokens");

    // Attempting step 257: MUST fail with context mismatch, MUST NOT PANIC
    let overflow_step = model.step(&mut session, 2, ReadMode::Enabled);
    assert!(
        overflow_step.is_err(),
        "Step 257 must be rejected when context=256 is full"
    );
    let err_msg = match overflow_step {
        Err(e) => e.to_string(),
        Ok(_) => panic!("Step 257 unexpectedly succeeded"),
    };
    assert!(
        err_msg.contains("integer session artifact/context/token mismatch"),
        "Expected context mismatch error, got: {err_msg}"
    );
    assert_eq!(
        session.len(),
        256,
        "Session length must remain 256 after rejected step"
    );
}

#[test]
fn test_sequence_length_budget_validation_boundaries() {
    let bundle = load_quat_bundle();

    let mut text_session = bundle
        .text_session(ReadMode::Enabled)
        .expect("text_session failed");
    // TextSession starts with observe(0) [BOS], so session.len() == 1.
    assert_eq!(text_session.len(), 1);

    // Can we generate 256 tokens from initial state? 1 + 256 = 257 > 256 -> MUST error
    let res_overflow = text_session.generate(256, Selection::Greedy, false);
    assert!(
        res_overflow.is_err(),
        "Initial session (len=1) + max_new_tokens=256 exceeds context 256 and must fail"
    );
    assert!(res_overflow
        .unwrap_err()
        .to_string()
        .contains("request exceeds full256 session budget"));
    assert_eq!(
        text_session.len(),
        1,
        "Rejected generate must not alter session length"
    );

    // Zero tokens generation: max_new_tokens = 0 -> MUST error
    let res_zero = text_session.generate(0, Selection::Greedy, false);
    assert!(
        res_zero.is_err(),
        "max_new_tokens = 0 must fail budget validation"
    );
    assert_eq!(
        text_session.len(),
        1,
        "Rejected zero budget must not alter session length"
    );

    // Valid boundary: generate 10 tokens
    let gen_10 = text_session.generate(10, Selection::Greedy, false);
    assert!(
        gen_10.is_ok(),
        "Generating 10 tokens must succeed: {:?}",
        gen_10.err()
    );
    let g10 = gen_10.unwrap();
    assert_eq!(g10.decisions.len(), 10);
    // After generating 10 tokens, session has 1 (BOS) + 10 = 11 tokens (10 observed + 1 pending)
    assert_eq!(text_session.len(), 11);

    // Now remaining capacity is 256 - 11 = 245.
    // Generating 246 tokens must fail:
    let res_exceed_rem = text_session.generate(246, Selection::Greedy, false);
    assert!(
        res_exceed_rem.is_err(),
        "Generating 246 tokens with 245 capacity must fail"
    );

    // Generating up to 245 tokens: 11 + 245 = 256 -> Budget validation MUST succeed!
    let gen_fill = text_session.generate(245, Selection::Greedy, false);
    assert!(
        gen_fill.is_ok(),
        "Filling with max_new_tokens=245 within budget must succeed: {:?}",
        gen_fill.err()
    );
    let g_fill = gen_fill.unwrap();
    println!("Fill generation stopped at {} tokens with reason: {:?}", g_fill.generated_token_ids.len(), g_fill.stop);
    let current_len = text_session.len();
    assert_eq!(current_len, 11 + g_fill.generated_token_ids.len());

    // Test remaining budget enforcement
    let remaining = 256 - current_len;
    if remaining > 0 {
        // Exceeding remaining budget by 1 must fail:
        let res_exceed = text_session.generate(remaining + 1, Selection::Greedy, false);
        assert!(
            res_exceed.is_err(),
            "Request exceeding remaining capacity by 1 must be rejected"
        );
        // Generating within remaining budget must succeed:
        let res_within = text_session.generate(remaining, Selection::Greedy, false);
        assert!(
            res_within.is_ok(),
            "Request within remaining capacity must succeed: {:?}",
            res_within.err()
        );
    }
}


// -------------------------------------------------------------------------
// 4. Extreme Repetitive Inputs & Adversarial Cycle Traps
// -------------------------------------------------------------------------

#[test]
fn test_extreme_repetitive_inputs_period1() {
    let bundle = load_quat_bundle();

    // Repetition of 1 single word: "hello hello hello hello hello "
    let rep_prompt = "hello ".repeat(30);
    let req = Request {
        prompt: rep_prompt,
        max_new_tokens: 30,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };

    let res = bundle.generate(&req);
    assert!(
        res.is_ok(),
        "Repetitive prompt must generate without panic/deadlock: {:?}",
        res.err()
    );
    let gen = res.unwrap();

    // Verify all decisions preserve 2^48 probability mass
    for d in &gen.decisions {
        assert_eq!(d.probability_sum_q48, PROBABILITY_TOTAL);
    }

    // Verify stop condition is valid (either short_cycle, eos, or maximum_new_tokens)
    match gen.stop {
        Stop::ShortCycle { period } => {
            assert!(
                (1..=4).contains(&period),
                "Period must be 1..=4, got {period}"
            );
        }
        Stop::MaximumNewTokens | Stop::Eos | Stop::FirstSentenceBoundary => {}
    }
}

#[test]
fn test_extreme_repetitive_inputs_period2() {
    let bundle = load_quat_bundle();

    // Period 2 repetition in prompt: "tick tock tick tock tick tock "
    let rep_prompt = "tick tock ".repeat(25);
    let req = Request {
        prompt: rep_prompt,
        max_new_tokens: 30,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };

    let res = bundle.generate(&req);
    assert!(res.is_ok(), "Period 2 prompt must generate cleanly");
    let gen = res.unwrap();
    for d in &gen.decisions {
        assert_eq!(d.probability_sum_q48, PROBABILITY_TOTAL);
    }
}

#[test]
fn test_extreme_repetitive_inputs_single_character() {
    let bundle = load_quat_bundle();

    // 100 repetitions of single character "a"
    let rep_prompt = "a".repeat(100);
    let req = Request {
        prompt: rep_prompt,
        max_new_tokens: 30,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };

    let res = bundle.generate(&req);
    assert!(res.is_ok(), "100x 'a' prompt must generate cleanly");
    let gen = res.unwrap();
    for d in &gen.decisions {
        assert_eq!(d.probability_sum_q48, PROBABILITY_TOTAL);
    }
}

// -------------------------------------------------------------------------
// 5. Invariant Verification: Panic Safety, Deadlock Freedom, Mass Conservation
// -------------------------------------------------------------------------

#[test]
fn test_probability_mass_conservation_categorical_and_greedy() {
    let bundle = load_quat_bundle();

    // Test with categorical sampling with various top_k values
    let top_k_values = [0, 1, 2, 5, 20, 50, 4096, 10000];
    for &k in &top_k_values {
        let req = Request {
            prompt: "The universe is".into(),
            max_new_tokens: 15,
            selection: Selection::Categorical {
                top_k: k,
                seed: 42 + k as u64,
            },
            read_mode: ReadMode::Enabled,
            first_sentence: false,
        };
        let res = bundle.generate(&req);
        assert!(
            res.is_ok(),
            "Categorical sampling top_k={k} must succeed: {:?}",
            res.err()
        );
        let gen = res.unwrap();
        for d in &gen.decisions {
            assert_eq!(
                d.probability_sum_q48, PROBABILITY_TOTAL,
                "Probability total must equal 2^48 for top_k={k}"
            );
            assert!(
                d.probability_q48 > 0,
                "Individual selected probability must be > 0"
            );
            assert!(
                d.probability_q48 <= PROBABILITY_TOTAL,
                "Individual probability must not exceed total"
            );
        }
    }
}

#[test]
fn test_sampler_stress_10000_draws() {
    let mut sampler = Sampler::new(12345);

    // Uniform-ish distribution of 4096 elements
    let vocab_size = 4096;
    let base = PROBABILITY_TOTAL / (vocab_size as u64);
    let mut probs = vec![base; vocab_size];
    let rem = PROBABILITY_TOTAL - base * (vocab_size as u64);
    probs[0] += rem;

    assert_eq!(probs.iter().sum::<u64>(), PROBABILITY_TOTAL);

    // Draw 10,000 times across policies: greedy, categorical top_k=1, top_k=5, top_k=4096
    for _i in 0..2500 {
        let sel_greedy = sampler.select(&probs, SamplePolicy::Greedy).unwrap();
        assert_eq!(sel_greedy, 0); // token 0 has highest mass

        let sel_top1 = sampler
            .select(&probs, SamplePolicy::Categorical { top_k: 1 })
            .unwrap();
        assert_eq!(sel_top1, 0);

        let sel_top5 = sampler
            .select(&probs, SamplePolicy::Categorical { top_k: 5 })
            .unwrap();
        assert!(sel_top5 < 5, "top_k=5 must pick from top 5");

        let sel_all = sampler
            .select(&probs, SamplePolicy::Categorical { top_k: 0 })
            .unwrap();
        assert!(sel_all < vocab_size);
    }
}

#[test]
fn test_read_mode_noread_exact_invariants() {
    let bundle = load_quat_bundle();
    let mut session = bundle
        .text_session(ReadMode::NoRead)
        .expect("NoRead session failed");

    session.append("Once upon a time").unwrap();
    let gen = session.generate(20, Selection::Greedy, false).unwrap();

    for d in &gen.decisions {
        assert_eq!(
            d.no_read_mass_q48, PROBABILITY_TOTAL,
            "In NoRead mode, no_read_mass must be 100% of 2^48"
        );
        assert_eq!(
            d.probability_sum_q48, PROBABILITY_TOTAL,
            "Probability total must equal 2^48 in NoRead mode"
        );
    }
}

// -------------------------------------------------------------------------
// 6. Householder-Pair Arm Parity
// -------------------------------------------------------------------------

#[test]
fn test_householder_pair_arm_boundaries() {
    let bundle = load_ord_bundle();

    // Verify token ID overflow
    let model = bundle.model();
    let mut session = model.new_session();
    assert!(model.step(&mut session, 0, ReadMode::Enabled).is_ok());
    assert!(model.step(&mut session, 4096, ReadMode::Enabled).is_err());

    // Verify boundary generation
    let req = Request {
        prompt: "The experiment shows".into(),
        max_new_tokens: 15,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };
    let gen = bundle.generate(&req).unwrap();
    for d in &gen.decisions {
        assert_eq!(d.probability_sum_q48, PROBABILITY_TOTAL);
    }
}

// -------------------------------------------------------------------------
// 7. Math Kernel Extremal Stress: Checked Primitives
// -------------------------------------------------------------------------

#[test]
fn test_math_kernel_extremes() {
    // scale_pow2
    assert_eq!(math::scale_pow2(0, 10).unwrap(), 0);
    assert_eq!(math::scale_pow2(1, 0).unwrap(), 1);
    assert_eq!(math::scale_pow2(-1, 127).unwrap(), i128::MIN);
    assert!(math::scale_pow2(1, 127).is_err()); // 2^127 overflows positive i128::MAX
    assert!(math::scale_pow2(1, 126).is_ok());
    assert!(math::scale_pow2(1, 128).is_err());
    assert!(math::scale_pow2(1, -128).is_ok());
    assert!(math::scale_pow2(1, -129).is_err());

    // div_round
    assert_eq!(math::div_round(10, 3).unwrap(), 3);
    assert_eq!(math::div_round(11, 3).unwrap(), 4); // 3.666 -> 4
    assert!(math::div_round(10, 0).is_err()); // Division by zero

    // checked_mul
    assert_eq!(math::checked_mul(0, i128::MAX).unwrap(), 0);
    assert!(math::checked_mul(i128::MAX, 2).is_err());
    assert!(math::checked_mul(i128::MIN, -1).is_err());

    // isqrt
    assert_eq!(math::isqrt(0), 0);
    assert_eq!(math::isqrt(1), 1);
    assert_eq!(math::isqrt(4), 2);
    assert_eq!(math::isqrt(u128::MAX), u64::MAX as u128);
}

// -------------------------------------------------------------------------
// 8. Exact Token-Length Prompt Boundaries (253, 254, 255, 256)
// -------------------------------------------------------------------------

#[test]
fn test_exact_token_length_prompts_253_254_255_256() {
    let bundle = load_quat_bundle();
    let tokenizer = bundle.tokenizer();

    let find_prompt_with_tokens = |target_len: usize| -> String {
        let mut text = String::new();
        while tokenizer.encode(&text).len() < target_len {
            let mut stepped = false;
            for candidate in [" 1", " a", " !", " x", " z", " \n", " 2", " 3"] {
                let mut trial = text.clone();
                trial.push_str(candidate);
                let l = tokenizer.encode(&trial).len();
                if l <= target_len {
                    text = trial;
                    stepped = true;
                    break;
                }
            }
            if !stepped {
                for c in 'a'..='z' {
                    let mut trial = text.clone();
                    trial.push(c);
                    let l = tokenizer.encode(&trial).len();
                    if l <= target_len {
                        text = trial;
                        stepped = true;
                        break;
                    }
                }
            }
            if !stepped {
                panic!("stuck searching for prompt of length {target_len}");
            }
        }
        text
    };

    let prompt_253 = find_prompt_with_tokens(253);
    assert_eq!(tokenizer.encode(&prompt_253).len(), 253);

    let prompt_254 = find_prompt_with_tokens(254);
    assert_eq!(tokenizer.encode(&prompt_254).len(), 254);

    let prompt_255 = find_prompt_with_tokens(255);
    assert_eq!(tokenizer.encode(&prompt_255).len(), 255);

    // Case A: Prompt 254 tokens + max_new_tokens 1 = 1 (BOS) + 254 + 1 = 256 <= 256 -> MUST SUCCEED
    let req_254_1 = Request {
        prompt: prompt_254.clone(),
        max_new_tokens: 1,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };
    let res_254_1 = bundle.generate(&req_254_1);
    assert!(
        res_254_1.is_ok(),
        "Prompt 254 tokens + 1 max_new_tokens = exact 256 capacity MUST succeed: {:?}",
        res_254_1.err()
    );
    let g = res_254_1.unwrap();
    assert_eq!(g.prompt_token_ids.len(), 255); // BOS (1) + prompt (254)
    assert_eq!(g.generated_token_ids.len(), 1);
    assert_eq!(g.decisions[0].probability_sum_q48, PROBABILITY_TOTAL);

    // Case B: Prompt 254 tokens + max_new_tokens 2 = 1 (BOS) + 254 + 2 = 257 > 256 -> MUST FAIL
    let req_254_2 = Request {
        prompt: prompt_254.clone(),
        max_new_tokens: 2,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };
    let res_254_2 = bundle.generate(&req_254_2);
    assert!(
        res_254_2.is_err(),
        "Prompt 254 tokens + 2 max_new_tokens = 257 tokens MUST fail"
    );

    // Case C: Prompt 255 tokens + max_new_tokens 1 = 1 (BOS) + 255 + 1 = 257 > 256 -> MUST FAIL
    let req_255_1 = Request {
        prompt: prompt_255.clone(),
        max_new_tokens: 1,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };
    let res_255_1 = bundle.generate(&req_255_1);
    assert!(
        res_255_1.is_err(),
        "Prompt 255 tokens + 1 max_new_tokens = 257 tokens MUST fail"
    );

    // Case D: Prompt 253 tokens + max_new_tokens 2 = 1 + 253 + 2 = 256 <= 256 -> MUST SUCCEED
    let req_253_2 = Request {
        prompt: prompt_253.clone(),
        max_new_tokens: 2,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: false,
    };
    let res_253_2 = bundle.generate(&req_253_2);
    assert!(
        res_253_2.is_ok(),
        "Prompt 253 tokens + 2 max_new_tokens = exact 256 capacity MUST succeed: {:?}",
        res_253_2.err()
    );
}

// -------------------------------------------------------------------------
// 9. Multi-turn Session Interleaved Appends and Generation
// -------------------------------------------------------------------------

#[test]
fn test_interleaved_append_and_generate_multiturn() {
    let bundle = load_quat_bundle();
    let mut session = bundle
        .text_session(ReadMode::Enabled)
        .expect("text_session creation failed");

    // Turn 1
    session.append("Alice went to the store.").unwrap();
    let g1 = session.generate(5, Selection::Greedy, false).unwrap();
    assert!(!g1.decisions.is_empty());
    for d in &g1.decisions {
        assert_eq!(d.probability_sum_q48, PROBABILITY_TOTAL);
    }

    // Turn 2: Append subsequent user query
    session.append("She bought two apples.").unwrap();
    let g2 = session.generate(8, Selection::Greedy, false).unwrap();
    assert!(!g2.decisions.is_empty());
    for d in &g2.decisions {
        assert_eq!(d.probability_sum_q48, PROBABILITY_TOTAL);
    }

    // Turn 3: Interleaved appending and categorical sampling
    session.append("Then what happened?").unwrap();
    let g3 = session.generate(10, Selection::Categorical { top_k: 5, seed: 999 }, false).unwrap();
    assert!(!g3.decisions.is_empty());
    for d in &g3.decisions {
        assert_eq!(d.probability_sum_q48, PROBABILITY_TOTAL);
    }

    assert!(session.len() <= 256);
}

// -------------------------------------------------------------------------
// 10. First Sentence Termination Without Period Byte
// -------------------------------------------------------------------------

#[test]
fn test_first_sentence_without_period_terminates() {
    let bundle = load_quat_bundle();
    // Prompt engineered to produce tokens that may not immediately contain '.'
    let req = Request {
        prompt: "1 2 3 4 5 6 7 8 9 10 11 12 13 14 15".into(),
        max_new_tokens: 20,
        selection: Selection::Greedy,
        read_mode: ReadMode::Enabled,
        first_sentence: true,
    };
    let res = bundle.generate(&req);
    assert!(res.is_ok(), "Must terminate without deadlock: {:?}", res.err());
    let gen = res.unwrap();
    // Verify it terminated cleanly with one of the valid Stop reasons
    match gen.stop {
        Stop::FirstSentenceBoundary => {
            assert!(gen.raw_decoded.contains('.'));
        }
        Stop::MaximumNewTokens | Stop::ShortCycle { .. } | Stop::Eos => {
            // Valid fallback if no period was generated
        }
    }
}

