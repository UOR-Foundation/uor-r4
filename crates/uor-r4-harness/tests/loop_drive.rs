//! The loop PR 10 supervisor tests: the planner is pure and time-injected, and the process layer is
//! exercised on a real short-lived shell. The planted-fault gate runs this file; the fake stalled
//! client past two hours must be closed and carded, while a fake progressing one continues with a
//! note.

use std::path::PathBuf;
use std::process::Command;

use uor_r4_harness::loopdrive::{
    check_in, classify, close_stalled_session, compose_prompt, launch_argv, pm_card_text,
    read_record, session_alive, start_session, terminate_session, watch_lease_until_renew_fails,
    watch_until_exit, CheckIn, Observation, Progress, SessionRecord, CHECK_IN_SECONDS,
};

fn observation(head: &str, commits: u64, gate_runs: u64, changed_paths: usize) -> Observation {
    Observation {
        head: head.to_string(),
        commits,
        gate_runs,
        changed_paths,
    }
}

fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("uor-loop-drive-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("temp root");
    root
}

#[test]
fn the_soft_check_in_continues_real_progress_and_closes_a_stall() {
    // Under two hours: always continue, even with no movement.
    assert!(matches!(
        check_in(60, &Progress::Stalled, 0),
        CheckIn::Continue { .. }
    ));
    // Past two hours: a fake progressing client continues with a note (D53).
    match check_in(CHECK_IN_SECONDS, &Progress::Progressing("new commits"), 0) {
        CheckIn::Continue { note } => assert!(note.contains("real progress"), "{note}"),
        other => panic!("a progressing session must continue with a note, got {other:?}"),
    }
    // Past two hours: the fake stalled client is closed and carded.
    match check_in(CHECK_IN_SECONDS, &Progress::Stalled, 0) {
        CheckIn::Close { reason } => assert!(reason.starts_with("stalled:"), "{reason}"),
        other => panic!("a fake stalled client past 2 hours is closed and carded: {other:?}"),
    }
    // A repeat of the same stall is spiraled, not stalled.
    match check_in(CHECK_IN_SECONDS, &Progress::Stalled, 2) {
        CheckIn::Close { reason } => assert!(reason.starts_with("spiraled:"), "{reason}"),
        other => panic!("a repeated stall is spiraled: {other:?}"),
    }
}

#[test]
fn progress_is_commits_gate_runs_or_edits_and_nothing_else() {
    let before = observation("aaaa", 10, 3, 0);
    assert_eq!(
        classify(&before, &observation("bbbb", 10, 3, 0)),
        Progress::Progressing("new HEAD")
    );
    assert_eq!(
        classify(&before, &observation("aaaa", 11, 3, 0)),
        Progress::Progressing("new commits")
    );
    assert_eq!(
        classify(&before, &observation("aaaa", 10, 4, 0)),
        Progress::Progressing("new gate runs")
    );
    assert_eq!(
        classify(&before, &observation("aaaa", 10, 3, 2)),
        Progress::Progressing("working tree changed")
    );
    assert_eq!(classify(&before, &before), Progress::Stalled);
}

#[test]
fn the_launch_command_gets_exactly_one_prompt() {
    let argv = vec!["dsh".to_string(), "{prompt}".to_string()];
    assert_eq!(
        launch_argv(&argv, "prompt text").unwrap(),
        vec!["dsh".to_string(), "prompt text".to_string()]
    );
    assert!(launch_argv(&["dsh".to_string()], "x").is_err());
    assert!(launch_argv(&["{prompt}".to_string(), "{prompt}".to_string()], "x").is_err());
    let default = compose_prompt(None, "GO item infra-loop-pr10 - ready");
    assert!(
        default.contains("GO item infra-loop-pr10 - ready"),
        "{default}"
    );
    assert_eq!(compose_prompt(Some("mine"), "GO x"), "mine");
}

#[test]
fn a_real_short_session_is_tracked_and_its_record_read_back() {
    let root = temp_root("session");
    let log = root.join("log/session.log");
    let (mut child, record) = start_session(
        &root,
        "deepseek",
        "fake-client",
        "pinned-model-1",
        &["sh".to_string(), "-c".to_string(), "exit 0".to_string()],
        "deadbeef",
        &log,
    )
    .expect("start a short session");
    assert_eq!(record.client, "fake-client");
    assert!(log.exists(), "the session log is created");
    assert!(
        session_alive(&record),
        "the session is recorded before the caller waits"
    );
    let status = child.wait().expect("wait");
    assert!(status.success());
    let read = read_record(&root, "deepseek").expect("the record is on disk");
    assert_eq!(read.pid, record.pid);
    assert_eq!(read.model, "pinned-model-1");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_supervisor_signals_only_its_recorded_session_and_waits_it_out() {
    let root = temp_root("terminate");
    let log = root.join("session.log");
    let (mut child, record) = start_session(
        &root,
        "deepseek",
        "fake-client",
        "pinned-model-1",
        &["sh".to_string(), "-c".to_string(), "sleep 30".to_string()],
        "deadbeef",
        &log,
    )
    .expect("start a long session");
    // Give the shell a moment to exist, then close it through the recorded group.
    std::thread::sleep(std::time::Duration::from_millis(200));
    terminate_session(&record).expect("terminate the recorded session");
    let waited = child.wait().expect("wait");
    assert!(
        !waited.success(),
        "the long session is signalled, not left running"
    );
    assert!(
        !session_alive(&record),
        "the recorded pid/group is gone after termination"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_watcher_outlives_the_run_and_never_cancels_it() {
    let root = temp_root("watch");
    let log = root.join("session.log");
    let (mut child, record) = start_session(
        &root,
        "deepseek",
        "fake-client",
        "pinned-model-1",
        &["sh".to_string(), "-c".to_string(), "sleep 30".to_string()],
        "deadbeef",
        &log,
    )
    .expect("start the watched session");
    std::thread::sleep(std::time::Duration::from_millis(200));
    let notes = watch_until_exit(&record, &["true".to_string()], 0, Some(2)).expect("watch");
    assert!(
        notes.iter().any(|note| note.contains("renewed")),
        "the watcher renews the lease while the run is alive: {notes:?}"
    );
    assert!(
        session_alive(&record),
        "the watcher never cancels a healthy run"
    );
    terminate_session(&record).expect("terminate");
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_lease_watcher_outlives_the_session_and_stops_when_the_lease_ends() {
    // A detached watcher renews while the renew command succeeds...
    let notes = watch_lease_until_renew_fails(&["true".to_string()], 0, Some(2)).expect("watch");
    assert!(
        notes.iter().any(|note| note.contains("renewed")),
        "the watcher renews a healthy lease: {notes:?}"
    );
    assert!(
        notes.iter().any(|note| note.contains("test bound")),
        "the bounded watch stops: {notes:?}"
    );
    // ...and stops when the lease ends, without cancelling the run.
    let notes = watch_lease_until_renew_fails(&["false".to_string()], 0, Some(2)).expect("watch");
    assert!(
        notes.iter().any(|note| note.contains("lease ended")),
        "a failed renew ends the watch without cancelling: {notes:?}"
    );
}

#[test]
fn the_pm_card_names_the_lab_reason_head_and_needed_prompt() {
    let card = pm_card_text("deepseek", "stalled: no commits", "abc1234", 7);
    assert!(card.starts_with("[deepseek]"), "{card}");
    assert!(card.contains("stalled: no commits"), "{card}");
    assert!(card.contains("abc1234"), "{card}");
    assert!(card.contains("needs a prompt"), "{card}");
}

#[test]
fn a_closed_session_records_a_report_stub_and_a_pickup_from_git_state() {
    let root = temp_root("close");
    let record = SessionRecord {
        client: "fake-client".to_string(),
        model: "pinned-model-1".to_string(),
        pid: 1,
        pgid: 1,
        started_unix: 0,
        prompt_sha256: "deadbeef".to_string(),
    };
    let directory = close_stalled_session(
        &root,
        "deepseek",
        &record,
        "stalled: no commits, gate runs or edits for 7200s of the 2-hour task",
    )
    .expect("close the stalled session");
    let stub = std::fs::read_to_string(directory.join("REPORT-stub.md")).expect("stub");
    assert!(stub.contains("stalled:"), "{stub}");
    assert!(stub.contains("no result is claimed"), "{stub}");
    let pickup = std::fs::read_to_string(directory.join("PICKUP.md")).expect("pickup");
    assert!(pickup.contains("loop next"), "{pickup}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_real_repository_reads_a_head_and_commit_count() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the checkout root");
    let observation = Observation::take(&root, "deepseek");
    assert!(observation.head.len() >= 7, "HEAD: {observation:?}");
    assert!(observation.commits > 0, "commits: {observation:?}");
    // git is available in this workspace; a failed read would leave the fields empty.
    assert!(!Command::new("git").arg("--version").output().is_err());
}
