//! Host control tests for the UOR-R4 Tauri desktop client (`infra-desktop-client-tauri`, #2826).
#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

use uor_r4_desktop_client::{
    build_synthetic_stack_artifact, sha256_bytes, DesktopBackend, DesktopClientError, PinRegistry,
    TerminalCommandRequest, SYNTHETIC_STANDIN_FILE_NAME, SYNTHETIC_STANDIN_SEED,
};

const EXPECTED_SYNTHETIC_32: [u32; 32] = [
    38, 9, 33, 8, 25, 38, 30, 25, 17, 30, 30, 30, 17, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    30, 30, 30, 30, 30, 30, 30, 30,
];

fn unique_temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uor-tauri-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn native_list_local_models_returns_synthetic_standin_and_refuses_tampered_hash() {
    let tmp = unique_temp_dir("models");

    let standin_bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
    assert_eq!(
        standin_bytes.len(),
        469_252,
        "seed-5 synthetic stand-in must match mk-synthetic-stack byte length"
    );
    let standin_sha = sha256_bytes(&standin_bytes);
    let standin_path = tmp.join(SYNTHETIC_STANDIN_FILE_NAME);
    fs::write(&standin_path, &standin_bytes).expect("write synthetic stand-in");

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend.add_model_search_dir(&tmp);

    let listed = backend
        .native_list_local_models()
        .expect("list local models with valid pinned stand-in");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, SYNTHETIC_STANDIN_FILE_NAME);
    assert_eq!(listed[0].format, "UORLUT01");
    assert_eq!(listed[0].size_bytes, 469_252);
    assert_eq!(listed[0].sha256, standin_sha);
    assert!(listed[0].synthetic);
    assert_eq!(listed[0].status_label, "synthetic stand-in");

    // Flip one byte in the payload so the SHA-256 differs from its pin while the UORLUT01 magic stays intact.
    let mut tampered = standin_bytes.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0x01;
    fs::write(&standin_path, &tampered).expect("write tampered stand-in");

    let err = backend
        .native_list_local_models()
        .expect_err("tampered model file whose hash differs from its pin must be refused");
    assert!(
        matches!(err, DesktopClientError::UnpinnedOrTamperedModel { .. }),
        "expected UnpinnedOrTamperedModel, got {err:?}"
    );

    // Also verify non-UORLUT01 model formats are refused before format.rs.
    fs::write(&standin_path, b"GGUF_NOT_UORLUT01_PAYLOAD").expect("write non-UORLUT01 file");
    let fmt_err = backend
        .native_list_local_models()
        .expect_err("non-UORLUT01 file must be refused");
    assert!(
        matches!(fmt_err, DesktopClientError::UnsupportedModelFormat(_)),
        "expected UnsupportedModelFormat, got {fmt_err:?}"
    );

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn native_engine_step_tokens_matches_reference_32_token_trajectory() {
    let standin_bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
    let standin_sha = sha256_bytes(&standin_bytes);

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let caps = backend
        .native_load_model_bytes(
            SYNTHETIC_STANDIN_FILE_NAME,
            &standin_bytes,
            Some(&standin_sha),
        )
        .expect("load pinned synthetic stand-in");
    assert_eq!(caps.weights_per_token, 15_968);
    assert_eq!(caps.shape.vocab, 40);
    assert_eq!(caps.shape.width, 32);
    assert!(caps.synthetic);

    let mut streamed_tokens = Vec::new();
    let receipt = backend
        .native_step_tokens("conv-32", &[0], 32, |ev| {
            streamed_tokens.push(ev.token_id);
        })
        .expect("step 32 tokens on synthetic stand-in");

    assert_eq!(receipt.token_ids, EXPECTED_SYNTHETIC_32);
    assert_eq!(streamed_tokens, EXPECTED_SYNTHETIC_32);
    assert_eq!(receipt.checkpoint_bytes, 9_782);
}

#[test]
fn durable_memory_object_store_close_and_resume_continues_token_for_token() {
    let store_dir = unique_temp_dir("durable-store");
    let standin_bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
    let standin_sha = sha256_bytes(&standin_bytes);

    // 1. Run the first 16 tokens in backend_a and persist to uor-r4-object-store on disk.
    let first_16_tokens: Vec<u32>;
    let save_receipt = {
        let mut backend_a = DesktopBackend::new(PinRegistry::with_synthetic_standin());
        backend_a
            .set_durable_store_dir(&store_dir)
            .expect("configure durable store dir on backend_a");
        backend_a
            .native_load_model_bytes(
                SYNTHETIC_STANDIN_FILE_NAME,
                &standin_bytes,
                Some(&standin_sha),
            )
            .expect("load synthetic stand-in in backend_a");

        let gen16 = backend_a
            .native_step_tokens("conv-resume", &[0], 16, |_| {})
            .expect("step first 16 tokens");
        assert_eq!(gen16.token_ids.len(), 16);
        assert_eq!(gen16.checkpoint_bytes, 5_366);
        first_16_tokens = gen16.token_ids;

        let saved = backend_a
            .native_save_session_durable("local-user", "studio", "conv-resume")
            .expect("save session snapshot to object store");
        assert_eq!(saved.ur4sess1_bytes_len, 5_366);
        assert!(saved.store_record_count > 0);
        saved
    };

    // 2. Start a fresh DesktopBackend process (backend_b), reload the object store from disk,
    //    restore the UR4SESS1 session, and continue 16 more tokens.
    let mut backend_b = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend_b
        .set_durable_store_dir(&store_dir)
        .expect("reload durable store dir on backend_b");
    backend_b
        .native_load_model_bytes(
            SYNTHETIC_STANDIN_FILE_NAME,
            &standin_bytes,
            Some(&standin_sha),
        )
        .expect("load synthetic stand-in in backend_b");

    let restored = backend_b
        .native_restore_session_durable("local-user", "studio", "conv-resume", None)
        .expect("restore session from object store on disk");
    assert_eq!(restored.position, 16);
    assert_eq!(restored.ur4sess1_bytes_len, 5_366);
    assert_eq!(restored.artifact_sha256, standin_sha);

    let last_tok = first_16_tokens[15];
    let gen_second_16 = backend_b
        .native_step_tokens("conv-resume", &[last_tok], 16, |_| {})
        .expect("continue 16 tokens after object-store restore");

    let mut combined = first_16_tokens;
    combined.extend_from_slice(&gen_second_16.token_ids);
    assert_eq!(
        combined, EXPECTED_SYNTHETIC_32,
        "16 + 16 close-and-resume through uor-r4-object-store must equal uninterrupted 32-token stream"
    );

    // 3. Corrupting the manifest bytes must fail verification on restore.
    let mut bad_manifest = save_receipt.manifest_bytes.clone();
    let tail = bad_manifest.len() - 1;
    bad_manifest[tail] ^= 0xff;
    let restore_err = backend_b
        .native_restore_session_durable("local-user", "studio", "conv-resume", Some(&bad_manifest))
        .expect_err("corrupted manifest must be refused by object-store digest verification");
    assert!(matches!(restore_err, DesktopClientError::ObjectStore(_)));

    let _ = fs::remove_dir_all(&store_dir);
}

#[test]
fn scoped_file_and_git_commands_enforce_granted_folder_allowlist() {
    let root = unique_temp_dir("scoped-fs");
    let granted_dir = root.join("granted-project");
    let outside_dir = root.join("outside-secret");
    fs::create_dir_all(&granted_dir).expect("create granted dir");
    fs::create_dir_all(&outside_dir).expect("create outside dir");

    let inside_file = granted_dir.join("hello.rs");
    let outside_file = outside_dir.join("secret.txt");
    fs::write(&outside_file, "classified").expect("write outside file");

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());

    // Before granting, writing inside granted_dir is refused.
    let ungranted_err = backend
        .native_write_file(&inside_file, "fn main() {}\n")
        .expect_err("writing to ungranted folder must fail");
    assert!(matches!(
        ungranted_err,
        DesktopClientError::PathNotGranted(_)
    ));

    // Grant granted_dir.
    backend
        .native_grant_folder(&granted_dir)
        .expect("grant folder");

    backend
        .native_write_file(&inside_file, "fn main() {}\n")
        .expect("write inside granted folder");
    let read_back = backend
        .native_read_file(&inside_file)
        .expect("read inside granted folder");
    assert_eq!(read_back, "fn main() {}\n");

    let listed = backend
        .native_list_directory(&granted_dir, 4)
        .expect("list granted folder");
    assert_eq!(listed, vec!["hello.rs".to_string()]);

    // Reading outside_file (not in granted_folders) is refused.
    let outside_err = backend
        .native_read_file(&outside_file)
        .expect_err("reading outside granted folders must fail");
    assert!(matches!(outside_err, DesktopClientError::PathNotGranted(_)));

    // Parent-dir traversal (`..`) from inside granted_dir is refused.
    let traversal = granted_dir.join("../outside-secret/secret.txt");
    let trav_err = backend
        .native_read_file(&traversal)
        .expect_err("parent directory traversal must fail");
    assert!(matches!(
        trav_err,
        DesktopClientError::PathTraversalRefused(_)
    ));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn terminal_command_is_off_by_default_and_requires_consent_and_granted_folder() {
    let root = unique_temp_dir("terminal-guard");
    let granted_dir = root.join("workspace");
    let ungranted_dir = root.join("ungranted");
    fs::create_dir_all(&granted_dir).expect("create granted dir");
    fs::create_dir_all(&ungranted_dir).expect("create ungranted dir");

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&granted_dir)
        .expect("grant workspace");

    assert!(
        !backend.terminal_enabled(),
        "terminal execution must be off by default"
    );

    let valid_req = TerminalCommandRequest {
        cwd: granted_dir.clone(),
        command: "pwd".to_string(),
        user_confirmed: true,
        confirmed_command: "pwd".to_string(),
    };

    // 1. Off by default -> refused even with user confirmation and granted folder.
    let disabled_err = backend
        .native_run_terminal_command(&valid_req)
        .expect_err("terminal command must be refused while disabled by default");
    assert_eq!(disabled_err, DesktopClientError::TerminalDisabledByDefault);

    // 2. Enable terminal, but omit user confirmation -> refused.
    backend.native_set_terminal_enabled(true);
    let unconfirmed_req = TerminalCommandRequest {
        cwd: granted_dir.clone(),
        command: "pwd".to_string(),
        user_confirmed: false,
        confirmed_command: "pwd".to_string(),
    };
    assert_eq!(
        backend
            .native_run_terminal_command(&unconfirmed_req)
            .expect_err("unconfirmed command must be refused"),
        DesktopClientError::TerminalConsentRequired
    );

    // 3. Mismatched confirmed_command -> refused.
    let mismatched_req = TerminalCommandRequest {
        cwd: granted_dir.clone(),
        command: "pwd".to_string(),
        user_confirmed: true,
        confirmed_command: "ls".to_string(),
    };
    assert_eq!(
        backend
            .native_run_terminal_command(&mismatched_req)
            .expect_err("mismatched confirmed command must be refused"),
        DesktopClientError::TerminalConsentRequired
    );

    // 4. Ungranted cwd -> refused.
    let ungranted_req = TerminalCommandRequest {
        cwd: ungranted_dir.clone(),
        command: "pwd".to_string(),
        user_confirmed: true,
        confirmed_command: "pwd".to_string(),
    };
    assert!(matches!(
        backend
            .native_run_terminal_command(&ungranted_req)
            .expect_err("ungranted cwd must be refused"),
        DesktopClientError::PathNotGranted(_)
    ));

    // 5. Shell chaining / metacharacters -> refused.
    let chained_req = TerminalCommandRequest {
        cwd: granted_dir.clone(),
        command: "pwd; whoami".to_string(),
        user_confirmed: true,
        confirmed_command: "pwd; whoami".to_string(),
    };
    assert!(matches!(
        backend
            .native_run_terminal_command(&chained_req)
            .expect_err("shell chaining must be refused"),
        DesktopClientError::TerminalMetacharacterRefused(_)
    ));

    // 6. Confirmed single command inside granted folder -> succeeds.
    let out = backend
        .native_run_terminal_command(&valid_req)
        .expect("confirmed command in granted folder succeeds");
    assert!(!out.trim().is_empty());

    let _ = fs::remove_dir_all(&root);
}
