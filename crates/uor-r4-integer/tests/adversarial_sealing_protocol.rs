use std::fs;
use std::path::{Path, PathBuf};
use uor_r4_integer::{format, report_output};

fn temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "uor-r4-seal-test-{}-{}-{label}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn test_clean_sealing_and_verification_both_pass() {
    let base = temp_dir("clean");
    let report_dir = base.join("report_1");

    // 1. Claim exclusively
    report_output::claim(&report_dir).expect("claim should succeed");
    assert!(report_dir.join(report_output::ATTEMPT_FILE).is_file());

    // 2. Add files
    fs::write(report_dir.join("data.txt"), b"canonical data payload").unwrap();
    fs::create_dir(report_dir.join("nested")).unwrap();
    fs::write(report_dir.join("nested").join("params.bin"), vec![1, 2, 3, 4, 5]).unwrap();

    // 3. Seal
    let manifest_path = report_output::seal(&report_dir).expect("seal should succeed");
    assert!(manifest_path.is_file());

    // 4. Verify with report_output::verify
    let unlisted = report_output::verify(&report_dir).expect("report_output::verify should pass");
    assert!(unlisted.is_empty(), "unlisted list must be empty");

    // 5. Verify with format::verify_sealed
    format::verify_sealed(&report_dir).expect("format::verify_sealed should pass");

    let _ = fs::remove_dir_all(base);
}

#[test]
fn test_adversarial_mutation_any_byte_causes_immediate_failure() {
    let base = temp_dir("mutate");
    let report_dir = base.join("report_mutate");

    report_output::claim(&report_dir).unwrap();
    fs::write(report_dir.join("weights.bin"), b"0123456789abcdef").unwrap();
    report_output::seal(&report_dir).unwrap();

    // Verify baseline passes
    assert!(report_output::verify(&report_dir).is_ok());
    assert!(format::verify_sealed(&report_dir).is_ok());

    // Adversarially flip 1 byte in weights.bin: '0' -> '1'
    let mut corrupted = fs::read(report_dir.join("weights.bin")).unwrap();
    corrupted[0] ^= 1;
    fs::write(report_dir.join("weights.bin"), corrupted).unwrap();

    // report_output::verify MUST fail
    let err_report = report_output::verify(&report_dir).expect_err("mutated byte must fail report_output::verify");
    assert!(
        err_report.to_string().contains("sealed file changed: weights.bin"),
        "Expected hash mismatch for weights.bin, got: {err_report}"
    );

    // format::verify_sealed MUST fail
    let err_format = format::verify_sealed(&report_dir).expect_err("mutated byte must fail format::verify_sealed");
    assert!(
        err_format.to_string().contains("sealed file changed: weights.bin"),
        "Expected hash mismatch for weights.bin, got: {err_format}"
    );

    let _ = fs::remove_dir_all(base);
}

#[test]
fn test_adversarial_adding_unlisted_file_causes_immediate_failure() {
    let base = temp_dir("add_unlisted");
    let report_dir = base.join("report_add");

    report_output::claim(&report_dir).unwrap();
    fs::write(report_dir.join("valid.txt"), b"initial valid content").unwrap();
    report_output::seal(&report_dir).unwrap();

    assert!(report_output::verify(&report_dir).is_ok());
    assert!(format::verify_sealed(&report_dir).is_ok());

    // Inject unlisted file at root
    fs::write(report_dir.join("injected.json"), b"{\"backdoor\": true}").unwrap();

    let err_report = report_output::verify(&report_dir).expect_err("injected file must fail report_output::verify");
    assert!(
        err_report.to_string().contains("sealed directory holds unlisted files")
            && err_report.to_string().contains("injected.json"),
        "Expected unlisted files error, got: {err_report}"
    );

    let err_format = format::verify_sealed(&report_dir).expect_err("injected file must fail format::verify_sealed");
    assert!(
        err_format.to_string().contains("sealed directory complete file set differs"),
        "Expected set mismatch error, got: {err_format}"
    );

    // Remove injected file from root, inject into nested subdir
    fs::remove_file(report_dir.join("injected.json")).unwrap();
    assert!(report_output::verify(&report_dir).is_ok());

    fs::create_dir(report_dir.join("sub")).unwrap();
    fs::write(report_dir.join("sub").join("secret.bin"), b"untracked").unwrap();

    assert!(report_output::verify(&report_dir).is_err());
    assert!(format::verify_sealed(&report_dir).is_err());

    let _ = fs::remove_dir_all(base);
}

#[test]
fn test_adversarial_removing_listed_file_causes_immediate_failure() {
    let base = temp_dir("remove_listed");
    let report_dir = base.join("report_rm");

    report_output::claim(&report_dir).unwrap();
    fs::write(report_dir.join("file_a.txt"), b"first file").unwrap();
    fs::write(report_dir.join("file_b.txt"), b"second file").unwrap();
    report_output::seal(&report_dir).unwrap();

    assert!(report_output::verify(&report_dir).is_ok());
    assert!(format::verify_sealed(&report_dir).is_ok());

    // Remove file_b.txt from filesystem
    fs::remove_file(report_dir.join("file_b.txt")).unwrap();

    // report_output::verify must fail when reading missing file
    let err_report = report_output::verify(&report_dir).expect_err("missing listed file must fail report_output::verify");
    assert_eq!(err_report.kind(), std::io::ErrorKind::NotFound);

    // format::verify_sealed must fail with complete file set differs or missing file
    let err_format = format::verify_sealed(&report_dir).expect_err("missing listed file must fail format::verify_sealed");
    assert!(
        err_format.to_string().contains("sealed directory complete file set differs"),
        "Expected file set difference, got: {err_format}"
    );

    let _ = fs::remove_dir_all(base);
}

#[test]
fn test_adversarial_double_sealing_refusal() {
    let base = temp_dir("double_seal");
    let report_dir = base.join("report_double");

    report_output::claim(&report_dir).unwrap();
    fs::write(report_dir.join("data.bin"), b"test").unwrap();
    report_output::seal(&report_dir).unwrap();

    // Sealing again must return AlreadyExists
    let err = report_output::seal(&report_dir).expect_err("double seal must fail");
    assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);

    let _ = fs::remove_dir_all(base);
}

#[test]
fn test_adversarial_claim_collision_preserves_bytes() {
    let base = temp_dir("claim_collision");
    let report_dir = base.join("report_collision");

    report_output::claim(&report_dir).unwrap();
    let sent_file = report_dir.join(report_output::ATTEMPT_FILE);
    let original_bytes = fs::read(&sent_file).unwrap();

    // Attempting to claim an already existing directory must fail
    let err = report_output::claim(&report_dir).expect_err("re-claim must fail");
    assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);

    // Verify existing bytes were not overwritten or touched
    let after_bytes = fs::read(&sent_file).unwrap();
    assert_eq!(original_bytes, after_bytes);

    let _ = fs::remove_dir_all(base);
}

#[test]
fn test_adversarial_manifest_tampering_is_detected() {
    let base = temp_dir("manifest_tamper");
    let report_dir = base.join("report_tamper");

    report_output::claim(&report_dir).unwrap();
    fs::write(report_dir.join("artifact.bin"), b"sensitive model bytes").unwrap();
    report_output::seal(&report_dir).unwrap();

    let manifest_path = report_dir.join(report_output::MANIFEST_FILE);
    let orig_manifest = fs::read_to_string(&manifest_path).unwrap();

    // Tamper 1: modify blake3 hash in manifest
    let mut manifest_val: serde_json::Value = serde_json::from_str(&orig_manifest).unwrap();
    manifest_val["files"][1]["blake3"] = serde_json::Value::String("0000000000000000000000000000000000000000000000000000000000000000".into());
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest_val).unwrap()).unwrap();

    assert!(report_output::verify(&report_dir).is_err());
    assert!(format::verify_sealed(&report_dir).is_err());

    // Tamper 2: modify schema
    manifest_val["schema"] = serde_json::Value::String("uor-r4.tampered-manifest/9".into());
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest_val).unwrap()).unwrap();
    let err_format = format::verify_sealed(&report_dir).expect_err("schema mismatch must fail");
    assert!(err_format.to_string().contains("sealed manifest schema differs"));

    // Tamper 3: size mismatch (bytes)
    manifest_val["schema"] = serde_json::Value::String("uor-r4.report-manifest/1".into());
    manifest_val["files"][1]["bytes"] = serde_json::json!(999999);
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest_val).unwrap()).unwrap();
    assert!(report_output::verify(&report_dir).is_err());
    assert!(format::verify_sealed(&report_dir).is_err());

    let _ = fs::remove_dir_all(base);
}

#[test]
fn test_adversarial_path_traversal_in_manifest() {
    let base = temp_dir("path_traversal");
    let report_dir = base.join("report_traversal");

    report_output::claim(&report_dir).unwrap();
    fs::write(report_dir.join("ok.txt"), b"ok").unwrap();

    let manifest = serde_json::json!({
        "schema": "uor-r4.report-manifest/1",
        "root": report_dir,
        "files": [
            {"path": "ok.txt", "bytes": 2, "blake3": blake3::hash(b"ok").to_hex().to_string()},
            {"path": "../secret_file", "bytes": 10, "blake3": "0000000000000000000000000000000000000000000000000000000000000000"}
        ]
    });
    fs::write(report_dir.join("manifest.json"), serde_json::to_string(&manifest).unwrap()).unwrap();

    let err = format::verify_sealed(&report_dir).expect_err("path traversal must be rejected");
    assert!(
        err.to_string().contains("sealed manifest path escapes or repeats"),
        "Expected path escape error, got: {err}"
    );

    let _ = fs::remove_dir_all(base);
}

#[test]
fn test_real_bundles_pass_verify_sealed() {
    let quat_path = Path::new("/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1");
    let ord_path = Path::new("/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-householder_pair-1");

    assert!(quat_path.exists(), "quaternion bundle must exist");
    assert!(ord_path.exists(), "householder_pair bundle must exist");

    format::verify_sealed(quat_path).expect("bundle-quaternion-1 must pass verify_sealed");
    format::verify_sealed(ord_path).expect("bundle-householder_pair-1 must pass verify_sealed");

    // Also check tables inside them
    format::verify_sealed(&quat_path.join("tables")).expect("bundle-quaternion-1/tables must pass verify_sealed");
    format::verify_sealed(&ord_path.join("tables")).expect("bundle-householder_pair-1/tables must pass verify_sealed");
}

#[test]
fn test_verify_core_artifacts_hashes_and_blake3() {
    let artifacts = [
        (
            "reference-evaluator-v2.json",
            Path::new("/Users/casey.allard/uor-r4-worktrees/geometric-lm-goal/docs/integration/reference-evaluator-v2.json"),
            "d2432fbba0e24ba51d7568700d6718c4e85d01ccc08e4fc3cc3fa2a77e928a62",
        ),
        (
            "bundle-quaternion-1/bundle.json",
            Path::new("/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1/bundle.json"),
            "b52c9cdba9800a601435f3c798f88d798bb742ff1a0adce97572c46f36d79412",
        ),
        (
            "bundle-householder_pair-1/bundle.json",
            Path::new("/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-householder_pair-1/bundle.json"),
            "521f2a4c3dc68c835db61d01e512412772b24f61722d08edc7cc2bc1cdd9d960",
        ),
        (
            "dev.u16",
            Path::new("/Users/casey.allard/uor-r4/.uor-models/research/issue-1017/tokens/dev.u16"),
            "74f7d85fa7670355f65805a9ffe06c2fb9db150b9f4cae8029e0d2f35e8993b6",
        ),
        (
            "train.u16",
            Path::new("/Users/casey.allard/uor-r4/.uor-models/research/issue-1017/tokens/train.u16"),
            "7b12a8b0cad32689584d5b14e10aa60d449859dcb735c57cdec26bcddd2d4873",
        ),
        (
            "tokenizer.json",
            Path::new("/Users/casey.allard/uor-r4/.uor-models/research/issue-1017/export/tokenizer.json"),
            "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89",
        ),
    ];

    for (name, path, expected_sha) in artifacts {
        assert!(path.exists(), "artifact must exist on disk: {name} at {}", path.display());
        let bytes = fs::read(path).expect("failed to read artifact bytes");
        let computed_sha = uor_r4_integer::sha256_file(path).expect("sha256 computation failed");
        let computed_blake3 = blake3::hash(&bytes).to_hex().to_string();

        println!("ARTIFACT_VERIFIED: {name}\n  Size: {} bytes\n  SHA-256: {computed_sha}\n  BLAKE3:  {computed_blake3}", bytes.len());
        assert_eq!(
            computed_sha, expected_sha,
            "SHA-256 mismatch for {name}! Expected {expected_sha}, got {computed_sha}"
        );
    }
}

