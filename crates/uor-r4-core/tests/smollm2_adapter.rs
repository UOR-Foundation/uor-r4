use uor_r4_core::transformerless::compiler::D;
use uor_r4_core::transformerless::scenarios::{export_hf_bytelevel_tokenizer, Tokenizer};
use uor_r4_model_source::{BehaviorSource, SmolLm2Oracle, TeacherOracle};

#[test]
#[ignore = "requires the downloaded 257 MiB SmolLM2 source"]
fn loads_real_safetensors_and_runs_teacher_forward() {
    let source = std::env::var("SMOLLM2_SOURCE")
        .unwrap_or_else(|_| ".uor-models/sources/smollm2-135m-instruct".to_owned());
    let mut oracle = SmolLm2Oracle::load(source).expect("load pinned SmolLM2 source");
    assert_eq!(oracle.vocab(), 49_152);
    assert_eq!(oracle.dim(), D);

    let mut embedding = [0.0f32; D];
    oracle.embedding(1, &mut embedding);
    assert!(embedding.iter().all(|value| value.is_finite()));
    assert!(embedding.iter().any(|value| *value != 0.0));

    let mut logits = vec![0.0f32; oracle.vocab()];
    oracle.reset();
    oracle.step(1, 0, &mut logits);
    assert!(logits.iter().all(|value| value.is_finite()));
    assert!(logits.iter().any(|value| *value != 0.0));
}

#[test]
#[ignore = "requires the downloaded SmolLM2 tokenizer"]
fn exports_runtime_tokenizer_with_text_roundtrip() {
    let source = std::env::var("SMOLLM2_SOURCE")
        .unwrap_or_else(|_| ".uor-models/sources/smollm2-135m-instruct".to_owned());
    let destination = std::env::temp_dir().join("smollm2-tokenizer-test.bin");
    export_hf_bytelevel_tokenizer(
        std::path::Path::new(&source).join("tokenizer.json"),
        &destination,
    )
    .expect("export tokenizer");
    let tokenizer = Tokenizer::try_load(destination).expect("load exported tokenizer");
    let prompt = "why is the sky blue?";
    let tokens = tokenizer.encode(prompt);
    assert_eq!(tokenizer.decode(&tokens).trim_start(), prompt);
}
