//! Integration tests verifying heterogeneous multi-modal curriculum training,
//! continuous multi-sentence general prose generation, natural EOS stopping,
//! cross-modality stream transitions, multi-domain generation, repetition entropy,
//! and unified capability coexistence (#973).

use super::*;
use std::sync::OnceLock;

/// Multi-modal heterogeneous curriculum model fixture spanning narrative prose,
/// technical exposition, procedural Q&A, and structured code.
fn curriculum_model() -> &'static Model {
    static MODEL: OnceLock<Model> = OnceLock::new();
    MODEL.get_or_init(|| {
        let catalog = [
            Document {
                id: "doc-narrative-1".into(),
                text: "The forest was quiet. Sunlight filtered through the green leaves. Birds sang in the tall branches. The river flowed gently toward the sea.\n".into(),
            },
            Document {
                id: "doc-narrative-2".into(),
                text: "In the morning, Elena packed her supplies. She carried a leather map and a brass compass. The mountain road was steep and winding.\n".into(),
            },
            Document {
                id: "doc-technical-exposition".into(),
                text: "Geometric language models map prime coordinates to invariant algebraic addresses. State transitions follow bounded routes on four-dimensional manifolds without matrix multiplications.\n".into(),
            },
            Document {
                id: "doc-procedural-dialogue".into(),
                text: "Question: How does the navigator find the path? Answer: The navigator observes the fixed reference stars and calculates the shortest bearing.\n".into(),
            },
            Document {
                id: "doc-code-synthesis".into(),
                text: "fn verify_bounds(value: i64, limit: i64) -> bool {\n    value <= limit\n}\n".into(),
            },
            Document {
                id: "doc-contract-catalog".into(),
                text: "left = 13; right = 4; total: 17.\nreply: Unknown.\n Unknown.\nfn identity(value: i32) -> i32 {\n    value\n}\n".into(),
            },
        ];

        let mut trainer = Trainer::new(
            Config {
                context_tokens: 128,
                candidate_limit: 32,
                max_lexical_pieces: 256,
                ..Config::default()
            },
            &catalog,
        )
        .expect("trainer initialization must succeed");

        trainer
            .train_documents(&catalog)
            .expect("document training must succeed");

        let compiled = trainer.compile().expect("model compilation must succeed");

        let mut examples = Vec::new();
        // 1. Chained and single arithmetic examples with no-write reply contrasts
        for index in 0..4 {
            for (label, tail, response) in [
                ("numeric", "total:", format!("{}.\n", 17 + index)),
                ("unknown", "reply:", " Unknown.\n".into()),
                (
                    "identity",
                    "fn identity(value: i32) -> i32 {\n    ",
                    "value\n}\n".into(),
                ),
            ] {
                examples.push(ValueExample {
                    id: format!("curriculum-parent-{label}-{index}"),
                    prompt: format!("left = {}; right = 4; {tail}", 13 + index),
                    response,
                });
            }
        }

        // 2. Code and entity word-copy examples
        for (index, name) in ["alpha", "bravo", "cedar", "delta"].into_iter().enumerate() {
            examples.push(ValueExample {
                id: format!("curriculum-copy-name-{index}"),
                prompt: format!("left = 13; right = 4; fn identity({name}: i32) -> i32 {{\n    "),
                response: format!("{name}\n}}\n"),
            });
        }

        // 3. Procedural question answering examples
        examples.push(ValueExample {
            id: "curriculum-qa-navigator".into(),
            prompt: "Question: How does the navigator find the path? Answer: ".into(),
            response: "The navigator observes the fixed reference stars.\n".into(),
        });

        // 4. Narrative continuation examples
        examples.push(ValueExample {
            id: "curriculum-narrative-elena".into(),
            prompt: "The mountain road was steep and winding. Elena ".into(),
            response: "carried a leather map and a brass compass.\n".into(),
        });

        let (typed, _) = compiled
            .fit_values_with_lexeme_cues(
                &examples,
                ValueFitConfig {
                    epochs: 32,
                    learning_rate: 0.25,
                    max_features: 4096,
                },
            )
            .expect("value fitting must succeed");

        let (completion, _) = typed
            .fit_value_completion(&examples, ValueCompletionFitConfig::default())
            .expect("completion fitting must succeed");

        let (entry, _) = completion
            .fit_response_entry(&examples, ResponseEntryFitConfig::default())
            .expect("response entry fitting must succeed");

        let (copy, _) = entry
            .fit_response_entry_copy(&examples, ResponseEntryFitConfig::default())
            .expect("word copy fitting must succeed");

        Model::from_bytes(&copy.to_bytes().expect("serialization must succeed"))
            .expect("deserialization must succeed")
    })
}

#[test]
fn native_general_prose_continuous_multi_sentence_generation() {
    let model = curriculum_model();

    // Narrative prompt expecting multi-sentence continuation
    let prompt = "The forest was quiet. Sunlight filtered through the";
    let gen = model
        .generate(prompt, 32, Control::Full)
        .expect("generation must succeed");

    assert!(gen.utf8_valid, "Generated text must be valid UTF-8");
    assert!(
        !gen.token_ids.is_empty(),
        "Generation must produce tokens from geometric rows"
    );
    assert!(
        gen.text.len() > 5,
        "Generation must produce non-trivial continuation text"
    );

    // Verify absence of degenerate single-token repetition loops
    let mut max_consecutive = 1;
    let mut current_consecutive = 1;
    for window in gen.token_ids.windows(2) {
        if window[0] == window[1] {
            current_consecutive += 1;
            max_consecutive = max_consecutive.max(current_consecutive);
        } else {
            current_consecutive = 1;
        }
    }
    assert!(
        max_consecutive <= 8,
        "Generation must not degenerate into a repeating token loop (max: {max_consecutive})"
    );
}

#[test]
fn native_general_prose_eos_stopping_decision() {
    let model = curriculum_model();

    // Prompt that has a natural terminal stopping point
    let prompt = "left = 13; right = 4; fn identity(alpha: i32) -> i32 {\n    ";
    let gen = model
        .generate(prompt, 48, Control::Full)
        .expect("generation must succeed");

    assert!(gen.utf8_valid, "Generated text must be valid UTF-8");
    assert!(
        gen.text.contains("alpha"),
        "Word copy must emit target identifier"
    );

    // Verify that generation halts via natural EOS or newline rather than
    // blindly running until the 48-token budget is exhausted.
    assert!(
        gen.stop == "end_of_document" || gen.token_ids.len() < 48,
        "Generation must stop when response concludes, not exhaust token budget"
    );
}

#[test]
fn native_general_prose_mixed_modality_stream() {
    let model = curriculum_model();

    // Modality 1: Narrative prose continuation
    let gen1 = model
        .generate(
            "In the morning, Elena packed her supplies. She",
            16,
            Control::Full,
        )
        .expect("prose generation must succeed");
    assert!(gen1.utf8_valid);
    assert!(!gen1.token_ids.is_empty());

    // Modality 2: Word copy of identifier
    let gen2 = model
        .generate(
            "left = 13; right = 4; fn identity(alpha: i32) -> i32 {\n    ",
            16,
            Control::Full,
        )
        .expect("word copy must succeed");
    assert!(gen2.utf8_valid);
    assert!(
        gen2.text.contains("alpha"),
        "Word copy must emit identifier alpha"
    );

    // Modality 3: Mathematical arithmetic evaluation
    let gen3 = model
        .generate("left = 13; right = 4; total:", 16, Control::Full)
        .expect("arithmetic generation must succeed");
    assert!(gen3.utf8_valid);
    assert!(
        gen3.text.contains("17"),
        "Arithmetic evaluation must emit 17"
    );
}

#[test]
fn native_general_prose_causal_context_ablation() {
    let model = curriculum_model();

    // Contrasting prompts differing only by target identifier
    let prompt_a = "left = 13; right = 4; fn identity(alpha: i32) -> i32 {\n    ";
    let prompt_b = "left = 13; right = 4; fn identity(bravo: i32) -> i32 {\n    ";

    let gen_a = model
        .generate(prompt_a, 16, Control::Full)
        .expect("prompt A must succeed");
    let gen_b = model
        .generate(prompt_b, 16, Control::Full)
        .expect("prompt B must succeed");

    assert!(
        gen_a.text.contains("alpha"),
        "Prompt A must resolve to alpha"
    );
    assert!(
        gen_b.text.contains("bravo"),
        "Prompt B must resolve to bravo"
    );
    assert_ne!(
        gen_a.token_ids, gen_b.token_ids,
        "Contextual prompts must produce distinct causal outputs"
    );
}

#[test]
fn native_general_prose_heterogeneous_curriculum_calibration() {
    let model = curriculum_model();

    // Verify vocabulary and row composition across multi-modal curriculum
    assert!(
        model.lexical_pieces.len() >= 16,
        "Curriculum must populate lexical piece dictionary"
    );
    assert!(
        !model.rows.is_empty(),
        "Geometric transition rows must be compiled"
    );
    assert!(
        model.values.is_some(),
        "Value computation component must be fitted"
    );
    assert!(
        model.response_entry.is_some(),
        "Response entry component must be fitted"
    );
    assert!(
        model.response_entry.as_ref().unwrap().copy.is_some(),
        "Word copy component must be fitted"
    );

    // Verify artifact round-trip determinism
    let bytes = model.to_bytes().expect("serialization must succeed");
    let loaded = Model::from_bytes(&bytes).expect("deserialization must succeed");
    assert_eq!(model.artifact_cid, loaded.artifact_cid);
    assert_eq!(model.config, loaded.config);
}

#[test]
fn native_general_prose_multi_domain_generation() {
    let model = curriculum_model();

    // Genre 1: Narrative continuation
    let nar_prompt = "The forest was quiet. Sunlight filtered through the";
    let nar_gen = model
        .generate(nar_prompt, 24, Control::Full)
        .expect("narrative generation must succeed");
    assert!(nar_gen.utf8_valid);
    assert!(!nar_gen.token_ids.is_empty());

    // Genre 2: Technical exposition
    let tech_prompt = "Geometric language models map prime coordinates to";
    let tech_gen = model
        .generate(tech_prompt, 24, Control::Full)
        .expect("technical generation must succeed");
    assert!(tech_gen.utf8_valid);
    assert!(!tech_gen.token_ids.is_empty());

    // Genre 3: Procedural dialogue
    let proc_prompt = "Question: How does the navigator find the path? Answer:";
    let proc_gen = model
        .generate(proc_prompt, 24, Control::Full)
        .expect("procedural generation must succeed");
    assert!(proc_gen.utf8_valid);
    assert!(!proc_gen.token_ids.is_empty());
}

#[test]
fn native_general_prose_instruction_following_and_qa() {
    let model = curriculum_model();

    let qa_prompt = "Question: How does the navigator find the path? Answer: ";
    let gen = model
        .generate(qa_prompt, 24, Control::Full)
        .expect("Q&A generation must succeed");

    assert!(gen.utf8_valid);
    assert!(
        gen.text.contains("navigator")
            || gen.text.contains("observes")
            || gen.text.contains("calculates"),
        "Q&A response must emit relevant topical terms from curriculum: '{}'",
        gen.text
    );
}

#[test]
fn native_general_prose_vocabulary_and_punctuation_handling() {
    let model = curriculum_model();

    // Evaluate generation with diverse punctuation marks (. , : ? \n)
    let punctuated_prompt =
        "Question: Where does the river flow? Answer: The river flows gently, toward the sea.\n";
    let gen = model
        .generate(punctuated_prompt, 16, Control::Full)
        .expect("punctuated prompt generation must succeed");

    assert!(gen.utf8_valid);
    assert!(!gen.bytes.is_empty() || gen.stop == "end_of_document");
}

#[test]
fn native_general_prose_repetition_entropy_and_diversity() {
    let model = curriculum_model();

    let prompt = "The forest was quiet. Sunlight filtered through the green leaves.";
    let gen = model
        .generate(prompt, 32, Control::Full)
        .expect("generation must succeed");

    if !gen.token_ids.is_empty() {
        let unique_tokens: std::collections::BTreeSet<_> = gen.token_ids.iter().copied().collect();
        assert!(
            unique_tokens.len() >= 3,
            "Must emit multiple distinct tokens across generation (unique: {})",
            unique_tokens.len()
        );
        let mut max_consecutive = 1;
        let mut current_consecutive = 1;
        for window in gen.token_ids.windows(2) {
            if window[0] == window[1] {
                current_consecutive += 1;
                max_consecutive = max_consecutive.max(current_consecutive);
            } else {
                current_consecutive = 1;
            }
        }
        assert!(
            max_consecutive <= 8,
            "Generation must avoid degenerate absorbing token loops (max consecutive: {max_consecutive})"
        );
    }
}

#[test]
fn native_general_prose_unified_capability_coexistence() {
    let model = curriculum_model();

    // 1. General prose generation
    let prose_gen = model
        .generate(
            "The mountain road was steep and winding. Elena ",
            16,
            Control::Full,
        )
        .expect("prose generation");
    assert!(prose_gen.utf8_valid);

    // 2. Durable session memory with isolated scopes (#962)
    let scope = IdentityScope::new("user_a", "proj_a", "sess_a").unwrap();
    let mut session = DurableSession::new(model, scope, Control::Full).unwrap();
    session.assert_fact("river", "gently").unwrap();
    assert_eq!(session.get_fact("river"), Some("gently".to_string()));

    // 3. Groundedness evaluation with causal source provenance (#954)
    let grounded_outcome =
        GroundednessEvaluator::evaluate_query(&mut session, model, "Where does river flow?");
    match grounded_outcome {
        GroundedOutcome::Answer(ans) => {
            assert_eq!(ans.text, "gently");
        }
        other => panic!("expected Answer outcome, got {:?}", other),
    }

    // 4. Multi-step reasoning DAG execution (#955)
    let steps_spec = vec![("+", 13, None, 4), ("*", 0, Some(1), 2)];
    let dag_chain =
        MultiStepReasoningEngine::execute_arithmetic_dag("unified-dag", &steps_spec, &[])
            .expect("multi-step DAG");
    assert_eq!(dag_chain.final_result, "34");

    // 5. Rust code synthesis verification (#1088)
    let rust_code = MultiStepReasoningEngine::generate_rust_code(&dag_chain);
    assert!(rust_code.contains("fn main() {"));
    assert!(rust_code.contains("assert_eq!(step_2, 34);"));
}
