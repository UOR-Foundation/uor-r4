//! Real-driver destination checks: an existing destination must fail before any model
//! access, even with an invalid model path, and keep its prior bytes; a fresh
//! destination must be claimed before the model is read. The helper tests cover the
//! helper; these cover its placement in the actual drivers.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "uor-r4-driver-collision-{}-{}-{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(binary: &Path, args: &[&str]) -> (bool, String) {
    let output = Command::new(binary).args(args).output().unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), text)
}

#[test]
#[ignore = "requires UOR_DRIVER_DIR pointing at the built release examples"]
fn drivers_reserve_destinations_before_model_work() {
    let drivers = PathBuf::from(std::env::var("UOR_DRIVER_DIR").unwrap());
    let base = scratch("drivers");
    let missing = base.join("intentionally-missing-model.json");
    let missing = missing.to_str().unwrap();
    let cases = base.join("cases.json");
    fs::write(&cases, b"[]").unwrap();
    let cases = cases.to_str().unwrap();
    // (driver, argument builder given the destination directory)
    let dir_drivers: Vec<(&str, Box<dyn Fn(&str) -> Vec<String>>)> = vec![
        (
            "native_historical_version",
            Box::new(|out| {
                vec![
                    "compare".into(),
                    missing.into(),
                    missing.into(),
                    cases.into(),
                    out.into(),
                ]
            }),
        ),
        (
            "native_historical_version",
            Box::new(|out| vec!["evaluate".into(), missing.into(), cases.into(), out.into()]),
        ),
        (
            "native_historical_version",
            Box::new(|out| vec!["preserve".into(), missing.into(), cases.into(), out.into()]),
        ),
        (
            "native_historical_version",
            Box::new(|out| vec!["probe".into(), missing.into(), out.into()]),
        ),
        (
            "native_historical_version",
            Box::new(|out| vec!["promote".into(), missing.into(), out.into()]),
        ),
        (
            "native_historical_version",
            Box::new(|out| vec!["fit".into(), missing.into(), cases.into(), out.into()]),
        ),
        (
            "native_current_query_handoff",
            Box::new(|out| vec!["evaluate".into(), missing.into(), cases.into(), out.into()]),
        ),
        (
            "native_word_sentence",
            Box::new(|out| vec!["evaluate".into(), missing.into(), cases.into(), out.into()]),
        ),
        (
            "native_operation_transition",
            Box::new(|out| vec!["evaluate".into(), missing.into(), out.into()]),
        ),
        (
            "native_lexical_emission",
            Box::new(|out| vec!["evaluate".into(), missing.into(), out.into()]),
        ),
        (
            "native_instruction_binding",
            Box::new(|out| vec!["evaluate".into(), missing.into(), cases.into(), out.into()]),
        ),
        (
            "native_current_query",
            Box::new(|out| vec!["evaluate".into(), missing.into(), cases.into(), out.into()]),
        ),
        (
            "native_word_emission_history",
            Box::new(|out| vec![missing.into(), cases.into(), out.into()]),
        ),
        (
            "native_copy_add",
            Box::new(|out| vec!["evaluate".into(), missing.into(), out.into()]),
        ),
        (
            "native_action_emission",
            Box::new(|out| vec!["evaluate".into(), missing.into(), out.into()]),
        ),
    ];
    let mut checked = 0;
    for (i, (driver, args)) in dir_drivers.iter().enumerate() {
        let binary = drivers.join(driver);
        assert!(
            binary.is_file(),
            "driver binary missing: {}",
            binary.display()
        );
        // 1. Existing destination with an invalid model: the collision is reported first
        //    and the prior report bytes and file set are untouched.
        let existing = base.join(format!("existing-{i}"));
        fs::create_dir(&existing).unwrap();
        let sentinel = existing.join("result.json");
        fs::write(&sentinel, b"prior rejected-candidate rows").unwrap();
        let argv = args(existing.to_str().unwrap());
        let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
        let (ok, text) = run(&binary, &argv);
        assert!(
            !ok,
            "{driver} {argv:?} must fail on an existing destination"
        );
        assert!(
            text.contains("not created exclusively"),
            "{driver} {argv:?} must report the collision before touching the model: {text}"
        );
        assert!(
            !text.contains("No such file"),
            "{driver} reached the model before the collision: {text}"
        );
        assert_eq!(
            fs::read(&sentinel).unwrap(),
            b"prior rejected-candidate rows"
        );
        let names: Vec<_> = fs::read_dir(&existing)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(
            names,
            vec![std::ffi::OsString::from("result.json")],
            "{driver} wrote into an existing destination"
        );
        // 2. Fresh destination with the same invalid model: the destination is claimed
        //    (sentinel present) before the model read fails.
        let fresh = base.join(format!("fresh-{i}"));
        let argv = args(fresh.to_str().unwrap());
        let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
        let (ok, text) = run(&binary, &argv);
        assert!(
            !ok && text.contains("No such file"),
            "{driver} {argv:?}: {text}"
        );
        assert!(
            fresh.join("attempt.json").is_file(),
            "{driver} must claim the destination before reading the model"
        );
        checked += 1;
    }
    // The API check writes one report file; an existing file fails before the model
    // is loaded and keeps its bytes, a fresh path is created before the model read.
    let api = drivers.join("native_historical_check");
    assert!(api.is_file());
    let report = base.join("api-report.json");
    fs::write(&report, b"prior api report").unwrap();
    let (ok, text) = run(&api, &[missing, cases, report.to_str().unwrap()]);
    assert!(
        !ok && text.contains("not created exclusively") && !text.contains("No such file"),
        "{text}"
    );
    assert_eq!(fs::read(&report).unwrap(), b"prior api report");
    let fresh_report = base.join("api-report-fresh.json");
    let (ok, text) = run(&api, &[missing, cases, fresh_report.to_str().unwrap()]);
    assert!(!ok && text.contains("No such file"), "{text}");
    assert!(
        fresh_report.is_file(),
        "the API report must be reserved before the model read"
    );
    // The word/Rust identity mode also writes one report file; same contract.
    let identity = drivers.join("native_word_sentence");
    let report = base.join("identity-report.json");
    fs::write(&report, b"prior identity report").unwrap();
    let (ok, text) = run(&identity, &["identity", missing, report.to_str().unwrap()]);
    assert!(
        !ok && text.contains("not created exclusively") && !text.contains("No such file"),
        "{text}"
    );
    assert_eq!(fs::read(&report).unwrap(), b"prior identity report");
    let fresh_identity = base.join("identity-report-fresh.json");
    let (ok, text) = run(
        &identity,
        &["identity", missing, fresh_identity.to_str().unwrap()],
    );
    assert!(!ok && text.contains("No such file"), "{text}");
    assert!(
        fresh_identity.is_file(),
        "the identity report must be reserved before the model read"
    );
    println!("driver destination checks: {} directory drivers plus the API report file and the word/Rust identity report file; collisions reported before any model access; prior bytes unchanged", checked);
    fs::remove_dir_all(base).unwrap();
}
