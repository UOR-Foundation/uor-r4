//! Tests for the Unified Capability API (`IntegerCapabilityApi`).

use std::path::Path;
use uor_r4_integer::capability_api::{IntegerCapabilityApi, SessionConfig, CAPABILITY_API_SCHEMA};
use uor_r4_integer::config::ReadMode;
use uor_r4_integer::sampling::SamplePolicy;
use uor_r4_integer::Bundle;

const CERTIFIED_BUNDLE_PATH: &str =
    "/Users/casey.allard/uor-r4/.uor-models/investigations/fourth-research-lab-20260926/dialogue-child-bundle-1";

#[test]
fn test_capability_api_full_lifecycle() {
    let bundle_path = Path::new(CERTIFIED_BUNDLE_PATH);
    if !bundle_path.exists() {
        eprintln!("SKIP: bundle path not found at {}", CERTIFIED_BUNDLE_PATH);
        return;
    }

    let bundle = Bundle::load(bundle_path).expect("bundle must load cleanly");
    let api = IntegerCapabilityApi::new(bundle);

    // 1. Verify Metadata
    let meta = api.metadata();
    assert_eq!(meta.api_schema_version, CAPABILITY_API_SCHEMA);
    assert_eq!(meta.width, 576);
    assert_eq!(meta.vocab_size, 4096);
    assert_eq!(meta.context_capacity, 256);
    assert!(meta.zero_transformers);
    assert!(meta.zero_hardware_multipliers);
    assert!(meta.zero_hardware_dividers);
    assert!(meta.zero_floats_in_serving);

    // 2. Create Session
    let config = SessionConfig {
        system_prompt: Some("You are a helpful mathematical assistant.".into()),
        policy: SamplePolicy::Greedy,
        read_mode: ReadMode::Enabled,
        seed: 42,
    };
    let session_id = api.create_session(config).expect("session creation failed");

    // 3. Ingest User Turn
    let ingested_count = api
        .ingest_user_turn(session_id, "What is the square root of 64?")
        .expect("ingest failed");
    assert!(ingested_count > 0);

    // 4. Inspect Telemetry
    let telem = api.get_telemetry(session_id).expect("telemetry failed");
    assert_eq!(telem.session_id, session_id);
    assert!(telem.active_tokens > 0);
    assert_eq!(telem.context_capacity, 256);

    // 5. Generate Text
    let response = api.generate_text(session_id, 16).expect("generate failed");
    println!("API Generated Response: {:?}", response);

    // 6. Save Session
    let saved = api.save_session(session_id).expect("save failed");
    assert_eq!(saved.schema, "uor-r4.integer-session/1");
    assert!(!saved.session_state.state.is_empty());

    // 7. Restore Session
    let restored_id = api.restore_session(saved).expect("restore failed");
    let restored_telem = api
        .get_telemetry(restored_id)
        .expect("restore telemetry failed");
    assert_eq!(restored_telem.context_capacity, telem.context_capacity);
}
