//! Integration tests for `uor-harness loop guard`, `loop install`, `loop onboard` and
//! `loop copyright` (loop PR 9, issue #2914).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};
use uor_r4_harness::loopguard::{
    canary_payload_for_client, check_all_receipts, check_copyright, client_skills_dir,
    evaluate_guard, normalize_skill_frontmatter, run_canary, run_install, run_onboard,
    GuardOutcome, OnboardVerdict, ALLOWED_SKILLS, CLIENTS, EXCLUDED_SKILL,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_uor-harness"))
}

/// Build a minimal UOR-R4 worktree fixture with `CHARTER.md`, `AGENTS.md`, `docs/loop/README.md`,
/// `docs/harness/{labs.json,owner-paths.json,rule-sources.json}`, an item with `scope:`, and an
/// active ORDER for `lab`.
fn make_uor_fixture(lab: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let real = repo_root();

    fs::create_dir_all(root.join("docs/harness")).expect("mkdir");
    fs::create_dir_all(root.join("docs/loop")).expect("mkdir");
    fs::create_dir_all(root.join("docs/roadmap/items")).expect("mkdir");
    fs::create_dir_all(root.join("archive")).expect("mkdir");

    for rel in [
        "docs/harness/labs.json",
        "docs/harness/owner-paths.json",
        "docs/harness/rule-sources.json",
    ] {
        fs::copy(real.join(rel), root.join(rel)).expect("copy registry");
    }
    fs::write(
        root.join("CHARTER.md"),
        "# CHARTER\n\n## §5.10 The loop\n\nCharter rule book.\n",
    )
    .expect("write CHARTER.md");
    fs::write(root.join("AGENTS.md"), "# AGENTS\n\nOnboarding summary.\n")
        .expect("write AGENTS.md");
    fs::write(
        root.join("docs/loop/README.md"),
        "# The loop\n\nP-1 and P-2 results; `uor-harness loop guard`.\n",
    )
    .expect("write loop README");
    fs::write(root.join("archive/FROZEN.md"), "frozen archive record\n")
        .expect("write archive file");

    let item_text = "\
---
id: infra-loop-pr09-client-adapters
title: Loop PR 9
milestone: infra
area: tooling
status: in_progress
open_to: lab
needs_gpu: false
scope: [crates/uor-r4-harness/src/loopguard.rs, crates/uor-r4-harness/tests/**, docs/harness/onboarding/**]
---
";
    fs::write(
        root.join("docs/roadmap/items/infra-loop-pr09-client-adapters.md"),
        item_text,
    )
    .expect("write item");

    let order_dir = root
        .join("docs/loop/tasks/infra-loop-harness")
        .join(format!("{lab}-1"));
    fs::create_dir_all(&order_dir).expect("mkdir order");
    let order_text = format!(
        "\
# ORDER: infra-loop-pr09-client-adapters, {lab}-1

Lab: {lab}. Item: [infra-loop-pr09-client-adapters](../../../../roadmap/items/infra-loop-pr09-client-adapters.md) (#2914). Setup row: 9.

## Files in scope

- `crates/uor-r4-harness/src/loopguard.rs`
- `crates/uor-r4-harness/tests/**`
- `docs/harness/onboarding/**`
"
    );
    fs::write(order_dir.join("ORDER.md"), order_text).expect("write ORDER");
    dir
}

/// Build a synthetic Loop Kit source directory with the 6 allowed skills plus `start-here`.
fn make_synthetic_kit() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    for skill in ALLOWED_SKILLS {
        let sdir = dir.path().join("LoopKit/skills").join(skill);
        fs::create_dir_all(&sdir).expect("mkdir skill");
        let desc = if skill == "gates" {
            // Unquoted colon in YAML description (the exact DeepSeek dsh issue from #2028).
            "description: Builds a check suite: one command that runs every check"
        } else {
            "description: Synthetic skill description for testing"
        };
        let content = format!(
            "---\nname: {skill}\n{desc}\n---\n\n# {skill}\n\nSynthetic test body for {skill} alpha bravo charlie delta echo foxtrot golf hotel.\n"
        );
        fs::write(sdir.join("SKILL.md"), content).expect("write SKILL.md");
    }
    let start_dir = dir.path().join("LoopKit/skills").join(EXCLUDED_SKILL);
    fs::create_dir_all(&start_dir).expect("mkdir start-here");
    fs::write(
        start_dir.join("SKILL.md"),
        "---\nname: start-here\ndescription: First-run interview\n---\n\nDo not install.\n",
    )
    .expect("write start-here");
    dir
}

/// The planted fault for loop PR 9: each client's canary write to a denied path is refused.
#[test]
fn each_clients_canary_write_is_refused() {
    let root = repo_root();
    for client in CLIENTS {
        let reason = run_canary(&root, client)
            .unwrap_or_else(|e| panic!("canary for {client} was not refused: {e}"));
        assert!(
            reason.contains("CHARTER.md")
                || reason.contains("owner-only")
                || reason.contains("R10"),
            "{client}: unexpected refusal reason: {reason}"
        );

        // Also verify via the CLI binary with `--input`.
        let tmp = tempfile::tempdir().expect("tempdir");
        let input_path = tmp.path().join("canary.json");
        let payload = canary_payload_for_client(&root, client);
        fs::write(&input_path, payload.to_string()).expect("write payload");
        let out = Command::new(bin())
            .current_dir(&root)
            .args([
                "loop",
                "guard",
                "--client",
                client,
                "--lab",
                client,
                "--event",
                "PreToolUse",
                "--input",
            ])
            .arg(&input_path)
            .output()
            .expect("run uor-harness loop guard");
        assert_eq!(
            out.status.code(),
            Some(2),
            "CLI canary write for {client} must exit 2; stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("REFUSED:"), "{client}: {stderr}");
        if client == "antigravity" {
            let stdout: Value =
                serde_json::from_slice(&out.stdout).expect("antigravity JSON decision");
            assert_eq!(stdout["decision"], "deny");
        }
    }
}

/// `loop guard` exits 0 at once when invoked outside the UOR-R4 repository and its worktrees.
#[test]
fn guard_exits_zero_immediately_outside_repo() {
    let outside = tempfile::tempdir().expect("tempdir outside repo");
    let payload_file = outside.path().join("payload.json");
    let payload = json!({
        "cwd": outside.path().display().to_string(),
        "workspacePaths": [outside.path().display().to_string()],
        "tool_name": "Write",
        "tool_input": {
            "file_path": outside.path().join("CHARTER.md").display().to_string(),
            "content": "outside project write"
        },
        "toolCall": {
            "name": "write_file",
            "args": {
                "Path": outside.path().join("CHARTER.md").display().to_string()
            }
        }
    });
    fs::write(&payload_file, payload.to_string()).expect("write payload");

    for client in CLIENTS {
        let out = Command::new(bin())
            .current_dir(outside.path())
            .env("UOR_LAB", client)
            .args([
                "loop",
                "guard",
                "--client",
                client,
                "--lab",
                client,
                "--event",
                "PreToolUse",
                "--input",
            ])
            .arg(&payload_file)
            .output()
            .expect("run loop guard outside repo");
        assert_eq!(
            out.status.code(),
            Some(0),
            "{client} outside repo failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// Per-client subagent scoping matches the P-1 probe findings on `docs/loop/README.md`:
/// - Claude Code (`agent_id`)
/// - Antigravity (`SessionStart` root `conversationId` vs subagent `conversationId`)
/// - Kimi Code (`SubagentStart` / `SubagentStop` depth per `session_id`)
#[test]
fn subagent_scoping_enforced_per_client_p1_design() {
    // 1. Claude Code: `agent_id` in PreToolUse.
    let claude_repo = make_uor_fixture("claude");
    let c_root = claude_repo.path();
    let in_parent_only = c_root
        .join("crates/uor-r4-harness/src/loopguard.rs")
        .display()
        .to_string();
    let in_suborder = c_root
        .join("crates/uor-r4-harness/tests/loop_guard.rs")
        .display()
        .to_string();

    // Parent session can write to `loopguard.rs`.
    let parent_call = json!({
        "cwd": c_root.display().to_string(),
        "tool_name": "Write",
        "tool_input": { "file_path": in_parent_only }
    });
    assert_eq!(
        evaluate_guard(c_root, "claude", "claude", "PreToolUse", &parent_call),
        GuardOutcome::Allow
    );

    // Subagent without a suborder is read-only (refused on any write).
    let sub_call_parent_file = json!({
        "cwd": c_root.display().to_string(),
        "agent_id": "subagent-alpha",
        "tool_name": "Write",
        "tool_input": { "file_path": in_suborder }
    });
    assert!(matches!(
        evaluate_guard(
            c_root,
            "claude",
            "claude",
            "PreToolUse",
            &sub_call_parent_file
        ),
        GuardOutcome::Refused(_)
    ));

    // Write a narrowed suborder `local/claude/claude-1/orders/1.md` allowing only `tests/**`.
    let orders_dir = c_root.join("local/claude/claude-1/orders");
    fs::create_dir_all(&orders_dir).expect("mkdir orders");
    fs::write(
        orders_dir.join("1.md"),
        "# Suborder\n\n## Scope\n\n- `crates/uor-r4-harness/tests/**`\n",
    )
    .expect("write suborder");

    // Bind `subagent-alpha` via SubagentStart (`1.md`).
    let sub_start_alpha = json!({
        "cwd": c_root.display().to_string(),
        "agent_id": "subagent-alpha"
    });
    assert_eq!(
        evaluate_guard(
            c_root,
            "claude",
            "claude",
            "SubagentStart",
            &sub_start_alpha
        ),
        GuardOutcome::Allow
    );

    // Subagent write inside narrowed suborder (`tests/loop_guard.rs`) is ALLOWED.
    assert_eq!(
        evaluate_guard(
            c_root,
            "claude",
            "claude",
            "PreToolUse",
            &sub_call_parent_file
        ),
        GuardOutcome::Allow
    );
    // A second concurrent subagent `subagent-beta` with no second suborder is REFUSED (never
    // inherits `1.md`).
    let sub_call_beta = json!({
        "cwd": c_root.display().to_string(),
        "agent_id": "subagent-beta",
        "tool_name": "Write",
        "tool_input": { "file_path": in_suborder }
    });
    assert!(matches!(
        evaluate_guard(c_root, "claude", "claude", "PreToolUse", &sub_call_beta),
        GuardOutcome::Refused(_)
    ));

    // Subagent write to `src/loopguard.rs` (in parent ORDER, outside suborder) is REFUSED.
    let sub_call_outside = json!({
        "cwd": c_root.display().to_string(),
        "agent_id": "subagent-alpha",
        "tool_name": "Write",
        "tool_input": { "file_path": in_parent_only }
    });
    match evaluate_guard(c_root, "claude", "claude", "PreToolUse", &sub_call_outside) {
        GuardOutcome::Refused(r) => assert!(r.contains("outside suborder"), "{r}"),
        other => panic!("expected Refused, got {other:?}"),
    }

    // 2. Antigravity: root `conversationId` recorded at first `SessionStart`; differing
    // `conversationId` in `PreToolUse` is scoped as a subagent.
    let agy_repo = make_uor_fixture("antigravity");
    let a_root = agy_repo.path();
    let a_orders = a_root.join("local/antigravity/antigravity-1/orders");
    fs::create_dir_all(&a_orders).expect("mkdir agy orders");
    fs::write(
        a_orders.join("1.md"),
        "# Suborder\n\n## Scope\n\n- `crates/uor-r4-harness/tests/**`\n",
    )
    .expect("write agy suborder");

    // Parent SessionStart records `root-conv-100`.
    let start_root = json!({
        "conversationId": "root-conv-100",
        "workspacePaths": [a_root.display().to_string()]
    });
    assert_eq!(
        evaluate_guard(
            a_root,
            "antigravity",
            "antigravity",
            "SessionStart",
            &start_root
        ),
        GuardOutcome::Allow
    );
    // Subagent SessionStart with `sub-conv-200` does not overwrite `root-conv-100`.
    let start_sub = json!({
        "conversationId": "sub-conv-200",
        "workspacePaths": [a_root.display().to_string()]
    });
    assert_eq!(
        evaluate_guard(
            a_root,
            "antigravity",
            "antigravity",
            "SessionStart",
            &start_sub
        ),
        GuardOutcome::Allow
    );

    let a_parent_file = a_root
        .join("crates/uor-r4-harness/src/loopguard.rs")
        .display()
        .to_string();
    let a_sub_file = a_root
        .join("crates/uor-r4-harness/tests/loop_guard.rs")
        .display()
        .to_string();

    // Root conversation can edit `src/loopguard.rs`.
    let root_write = json!({
        "conversationId": "root-conv-100",
        "workspacePaths": [a_root.display().to_string()],
        "toolCall": { "name": "edit_file", "args": { "Path": a_parent_file } }
    });
    assert_eq!(
        evaluate_guard(
            a_root,
            "antigravity",
            "antigravity",
            "PreToolUse",
            &root_write
        ),
        GuardOutcome::Allow
    );

    // Subagent conversation `sub-conv-200` is refused on `src/loopguard.rs`, allowed on `tests/`.
    let sub_write_denied = json!({
        "conversationId": "sub-conv-200",
        "workspacePaths": [a_root.display().to_string()],
        "toolCall": { "name": "edit_file", "args": { "Path": a_parent_file } }
    });
    assert!(matches!(
        evaluate_guard(
            a_root,
            "antigravity",
            "antigravity",
            "PreToolUse",
            &sub_write_denied
        ),
        GuardOutcome::Refused(_)
    ));
    let sub_write_allowed = json!({
        "conversationId": "sub-conv-200",
        "workspacePaths": [a_root.display().to_string()],
        "toolCall": { "name": "edit_file", "args": { "Path": a_sub_file } }
    });
    assert_eq!(
        evaluate_guard(
            a_root,
            "antigravity",
            "antigravity",
            "PreToolUse",
            &sub_write_allowed
        ),
        GuardOutcome::Allow
    );

    // 3. Kimi Code: depth tracked between `SubagentStart` and `SubagentStop` per `session_id`.
    let kimi_repo = make_uor_fixture("kimi");
    let k_root = kimi_repo.path();
    let k_orders = k_root.join("local/kimi/kimi-1/orders");
    fs::create_dir_all(&k_orders).expect("mkdir kimi orders");
    fs::write(
        k_orders.join("1.md"),
        "# Suborder\n\n## Scope\n\n- `crates/uor-r4-harness/tests/**`\n",
    )
    .expect("write kimi suborder");

    let k_parent_file = k_root
        .join("crates/uor-r4-harness/src/loopguard.rs")
        .display()
        .to_string();
    let k_sess = json!({ "session_id": "kimi-s1", "cwd": k_root.display().to_string() });
    assert_eq!(
        evaluate_guard(k_root, "kimi", "kimi", "SessionStart", &k_sess),
        GuardOutcome::Allow
    );

    let k_write = json!({
        "session_id": "kimi-s1",
        "cwd": k_root.display().to_string(),
        "tool_name": "WriteFile",
        "tool_input": { "path": k_parent_file }
    });
    // Depth 0: allowed.
    assert_eq!(
        evaluate_guard(k_root, "kimi", "kimi", "PreToolUse", &k_write),
        GuardOutcome::Allow
    );
    // Enter SubagentStart (depth 1): write outside suborder is refused.
    assert_eq!(
        evaluate_guard(k_root, "kimi", "kimi", "SubagentStart", &k_sess),
        GuardOutcome::Allow
    );
    assert!(matches!(
        evaluate_guard(k_root, "kimi", "kimi", "PreToolUse", &k_write),
        GuardOutcome::Refused(_)
    ));
    // Exit SubagentStop (depth 0): allowed again.
    assert_eq!(
        evaluate_guard(k_root, "kimi", "kimi", "SubagentStop", &k_sess),
        GuardOutcome::Allow
    );
    assert_eq!(
        evaluate_guard(k_root, "kimi", "kimi", "PreToolUse", &k_write),
        GuardOutcome::Allow
    );
}

/// D56 & CHARTER §5.1, §5.2, §5.10: release commands, cap overrides, forbidden git/process/secret
/// commands, and cross-lab writes are refused.
#[test]
fn command_and_cross_lab_guards_refuse_violations() {
    let fixture = make_uor_fixture("codex");
    let root = fixture.path();

    let forbidden_cmds = [
        "unset UOR_LAB && uor-harness status",
        "env -u UOR_LAB uor-harness check",
        "UOR_LAB=\"\" uor-harness release",
        "uor-harness release --sheet s1",
        "scripts/pod/uor-pod up --max-pods 8",
        "UOR_POD_MAX_HOURLY_USD=20 scripts/pod/uor-pod up",
        "git commit -m '[codex] wip' --no-verify",
        "git commit -nm '[codex] wip'",
        "git -c core.hooksPath=/dev/null commit -m '[codex] wip'",
        "git stash",
        "git push --force origin codex/test",
        "git -c a=b push --force origin codex/test",
        "git push origin main",
        "git push -u origin main",
        "git push origin feat:main",
        "git push origin HEAD:refs/heads/main",
        "gh pr merge 2914 --squash",
        "gh  pr  merge 2914 --squash",
        "gh api repos/UOR-Foundation/uor-r4/pulls/2914/merge -X PUT",
        "pkill -f uor-harness",
        "/usr/bin/pkill foo",
        "killall cargo",
        "gh auth token",
        "cat ~/.env",
        "head -n 20 ~/.netrc",
        "printenv",
        "env",
        "export -p",
        "echo $GITHUB_TOKEN",
        "echo hacked > CHARTER.md",
        "rm CHARTER.md",
        "cp /tmp/x CHARTER.md",
        "sed -i '' 's/a/b/' CHARTER.md",
        "dd if=/dev/zero of=CHARTER.md",
        "git checkout -- CHARTER.md",
        "uor-harness loop install --all --apply",
        "uor-harness git install-hooks",
    ];

    for cmd in forbidden_cmds {
        let payload = json!({
            "cwd": root.display().to_string(),
            "tool_name": "Bash",
            "tool_input": { "command": cmd }
        });
        assert!(
            matches!(
                evaluate_guard(root, "codex", "codex", "PreToolUse", &payload),
                GuardOutcome::Refused(_)
            ),
            "command should have been refused: {cmd}"
        );
    }

    // `git push --force-with-lease` on a feature branch and `uor-harness loop install --all`
    // (plan-only default without `--apply`) are allowed.
    for allowed_cmd in [
        "git push --force-with-lease origin codex/test",
        "uor-harness loop install --all",
    ] {
        let payload = json!({
            "cwd": root.display().to_string(),
            "tool_name": "Bash",
            "tool_input": { "command": allowed_cmd }
        });
        assert_eq!(
            evaluate_guard(root, "codex", "codex", "PreToolUse", &payload),
            GuardOutcome::Allow,
            "command should have been allowed: {allowed_cmd}"
        );
    }

    // Subagent attempting `gh pr create` or `git push` is refused.
    let sub_pr = json!({
        "cwd": root.display().to_string(),
        "agent_id": "sub-1",
        "tool_name": "Bash",
        "tool_input": { "command": "gh pr create --draft --title '[codex] test'" }
    });
    assert!(matches!(
        evaluate_guard(root, "codex", "codex", "PreToolUse", &sub_pr),
        GuardOutcome::Refused(_)
    ));

    // Cross-lab `local/claude/` write and existing `archive/` edit are refused.
    for denied_rel in [
        "local/claude/PICKUP.md",
        "local/loop/STOP",
        "archive/FROZEN.md",
    ] {
        let payload = json!({
            "cwd": root.display().to_string(),
            "tool_name": "Write",
            "tool_input": { "file_path": root.join(denied_rel).display().to_string() }
        });
        assert!(
            matches!(
                evaluate_guard(root, "codex", "codex", "PreToolUse", &payload),
                GuardOutcome::Refused(_)
            ),
            "path should have been refused: {denied_rel}"
        );
    }

    // Own `local/codex/PICKUP.md` and `gh pr merge --match-head-commit <sha>` are allowed.
    let own_local = json!({
        "cwd": root.display().to_string(),
        "tool_name": "Write",
        "tool_input": { "file_path": root.join("local/codex/PICKUP.md").display().to_string() }
    });
    assert_eq!(
        evaluate_guard(root, "codex", "codex", "PreToolUse", &own_local),
        GuardOutcome::Allow
    );
    let merge_ok = json!({
        "cwd": root.display().to_string(),
        "tool_name": "Bash",
        "tool_input": {
            "command": "gh pr merge 2914 --squash --match-head-commit 7864b05167864b05167864b05167864b05167864"
        }
    });
    assert_eq!(
        evaluate_guard(root, "codex", "codex", "PreToolUse", &merge_ok),
        GuardOutcome::Allow
    );
}

/// `loop install` installs the 6 allowed skills (quoting unquoted YAML descriptions and skipping
/// `start-here`), retires Kimi's second loop with `<= 1` KB pointers, refuses to overwrite
/// differing files without `--replace`, and backs up replaced files to `local/loop/install-backups/`.
#[test]
fn install_skills_backups_and_replace_refusal() {
    let fixture = make_uor_fixture("antigravity");
    let root = fixture.path();
    let home = tempfile::tempdir().expect("temp home");
    let kit = make_synthetic_kit();

    // Seed Kimi's retired second loop hook and an existing `start-here` skill to verify `--replace`
    // refusal and backup.
    let kimi_dir = home.path().join(".kimi-code");
    fs::create_dir_all(&kimi_dir).expect("mkdir kimi");
    fs::write(
        kimi_dir.join("config.toml"),
        "default_model = \"kimi-for-coding\"\n\n[[hooks]]\nevent = \"Stop\"\nmatcher = \"^(Proceed|Continue) UOR-R4$\"\ncommand = \"~/.kimi-code/hooks/kimi-uor-r4-loop-hook.sh\"\n",
    )
    .expect("write kimi config");

    let all_clients: Vec<String> = CLIENTS.iter().map(|s| (*s).to_string()).collect();

    // Without `--replace`, `run_install` refuses because `~/.kimi-code/config.toml` already exists
    // and differs.
    let err = run_install(root, home.path(), &all_clients, kit.path(), false, false)
        .expect_err("must refuse without --replace");
    assert!(err.contains("REFUSED"), "{err}");

    // With `--replace`, `run_install` backs up `config.toml` under `local/loop/install-backups/` and
    // writes all client configs and the 6 allowed skills.
    let summary = run_install(root, home.path(), &all_clients, kit.path(), true, false)
        .expect("install with --replace succeeds");
    assert_eq!(summary["start_here_excluded"], true);
    let backups = summary["backups"].as_array().expect("backups array");
    assert!(!backups.is_empty(), "expected backup of config.toml");

    for client in CLIENTS {
        let sdir = client_skills_dir(home.path(), client);
        for skill in ALLOWED_SKILLS {
            let p = sdir.join(skill).join("SKILL.md");
            assert!(p.is_file(), "{client}: missing skill {skill}");
        }
        assert!(
            !sdir.join(EXCLUDED_SKILL).exists(),
            "{client}: start-here must not be installed"
        );
        // Verify `gates/SKILL.md` had its unquoted colon description normalized.
        let gates_md = fs::read_to_string(sdir.join("gates/SKILL.md")).expect("read gates");
        assert!(
            gates_md.contains(
                "description: \"Builds a check suite: one command that runs every check\""
            ),
            "{client}: gates description was not quoted: {gates_md}"
        );

        // Run `run_onboard` against the freshly installed `home`.
        let report = run_onboard(root, home.path(), client, client).expect("run_onboard succeeds");
        for row in &report.checks {
            if row.check != "binary" {
                assert_eq!(
                    row.verdict,
                    OnboardVerdict::Pass,
                    "{client} check {} failed: {}",
                    row.check,
                    row.detail
                );
            }
        }
    }

    // Verify Kimi's pointers are <= 1024 B and the retired hook is gone.
    let start_ptr = fs::read_to_string(kimi_dir.join("UOR-R4-START.md")).expect("read start");
    let handoff_ptr = fs::read_to_string(kimi_dir.join("UOR-R4-HANDOFF.md")).expect("read handoff");
    assert!(start_ptr.len() <= 1024);
    assert!(handoff_ptr.len() <= 1024);
    let new_kimi_cfg = fs::read_to_string(kimi_dir.join("config.toml")).expect("read kimi config");
    assert!(!new_kimi_cfg.contains("Proceed|Continue) UOR-R4"));
    assert!(new_kimi_cfg.contains("default_model = \"kimi-for-coding\""));
    assert!(new_kimi_cfg.contains("uor-harness loop guard --client kimi"));

    // Verify unit helper for frontmatter normalization directly.
    let raw = "---\nname: gates\ndescription: A: B\n---\nBody\n";
    assert!(normalize_skill_frontmatter(raw).contains("description: \"A: B\""));
}

/// All five committed onboarding receipts in `docs/harness/onboarding/<client>.json` exist, match
/// `docs/harness/labs.json`, and pass live canary refusal verification.
#[test]
fn committed_onboarding_receipts_are_valid_and_canaries_refuse() {
    let root = repo_root();
    let verified = check_all_receipts(&root).expect("all 5 onboarding receipts must pass");
    assert_eq!(verified, CLIENTS);
}

/// `loop copyright` detects an 8-word overlap against a Loop Kit source and passes when prose is in
/// our own words.
#[test]
fn copyright_check_detects_eight_word_window_and_passes_clean_text() {
    let fixture = make_uor_fixture("antigravity");
    let root = fixture.path();
    let kit = make_synthetic_kit();

    // Clean fixture has 0 matches against the synthetic kit.
    let clean_hits = check_copyright(root, kit.path()).expect("check_copyright runs");
    assert!(clean_hits.is_empty(), "expected 0 hits, got {clean_hits:?}");

    // Plant an 8-word sequence from the synthetic kit into a repo markdown file.
    fs::write(
        root.join("docs/loop/LEAK.md"),
        "# Leak test\n\nHere is alpha bravo charlie delta echo foxtrot golf hotel in prose.\n",
    )
    .expect("write LEAK.md");
    let leak_hits = check_copyright(root, kit.path()).expect("check_copyright detects leak");
    assert!(
        !leak_hits.is_empty(),
        "expected planted 8-word leak to fail"
    );
    assert!(leak_hits.iter().any(|h| h.repo_file == "docs/loop/LEAK.md"));
}

/// Regression tests for repair order `antigravity-2`:
/// - closed historical ORDER folders (with `REPORT.md`) do not scope top-level writes;
/// - Antigravity `PreToolUse` without prior `SessionStart` fails closed as a subagent, and a new
///   top-level `SessionStart` after the old root ran tool calls replaces a stale `root_cid`;
/// - outside-repo `loop guard` exits 0 even with an unknown `--client` or malformed JSON;
/// - `loop install` CLI is plan-only by default (writes nothing without `--apply`).
#[test]
fn repair_order_antigravity_2_regressions() {
    // 1. Closed historical task folder (`claude-1` with `REPORT.md`) is ignored by `find_active_order`.
    let claude_fixture = make_uor_fixture("claude");
    let c_root = claude_fixture.path();
    fs::write(
        c_root.join("docs/loop/tasks/infra-loop-harness/claude-1/REPORT.md"),
        "# REPORT: claude-1\n",
    )
    .expect("write REPORT.md to close claude-1");
    let unscoped_write = json!({
        "cwd": c_root.display().to_string(),
        "tool_name": "Write",
        "tool_input": {
            "file_path": c_root.join("crates/uor-r4-harness/src/doccheck.rs").display().to_string()
        }
    });
    assert_eq!(
        evaluate_guard(c_root, "claude", "claude", "PreToolUse", &unscoped_write),
        GuardOutcome::Allow
    );

    // 2. Antigravity fail-closed PreToolUse without SessionStart, and stale root replacement.
    let agy_fixture = make_uor_fixture("antigravity");
    let a_root = agy_fixture.path();
    let a_file = a_root
        .join("crates/uor-r4-harness/src/loopguard.rs")
        .display()
        .to_string();
    let pre_without_start = json!({
        "conversationId": "unregistered-conv",
        "workspacePaths": [a_root.display().to_string()],
        "toolCall": { "name": "edit_file", "args": { "Path": a_file } }
    });
    assert!(matches!(
        evaluate_guard(
            a_root,
            "antigravity",
            "antigravity",
            "PreToolUse",
            &pre_without_start
        ),
        GuardOutcome::Refused(_)
    ));

    // Establish root `old-root`, run a normal tool call, then start a new top-level session
    // `new-root` (simulating `old-root` killed without `Stop`): `new-root` replaces `old-root`.
    let start_old = json!({
        "conversationId": "old-root",
        "workspacePaths": [a_root.display().to_string()]
    });
    assert_eq!(
        evaluate_guard(
            a_root,
            "antigravity",
            "antigravity",
            "SessionStart",
            &start_old
        ),
        GuardOutcome::Allow
    );
    let old_write = json!({
        "conversationId": "old-root",
        "workspacePaths": [a_root.display().to_string()],
        "toolCall": { "name": "edit_file", "args": { "Path": a_file } }
    });
    assert_eq!(
        evaluate_guard(
            a_root,
            "antigravity",
            "antigravity",
            "PreToolUse",
            &old_write
        ),
        GuardOutcome::Allow
    );
    let start_new = json!({
        "conversationId": "new-root",
        "workspacePaths": [a_root.display().to_string()]
    });
    assert_eq!(
        evaluate_guard(
            a_root,
            "antigravity",
            "antigravity",
            "SessionStart",
            &start_new
        ),
        GuardOutcome::Allow
    );
    let new_write = json!({
        "conversationId": "new-root",
        "workspacePaths": [a_root.display().to_string()],
        "toolCall": { "name": "edit_file", "args": { "Path": a_file } }
    });
    assert_eq!(
        evaluate_guard(
            a_root,
            "antigravity",
            "antigravity",
            "PreToolUse",
            &new_write
        ),
        GuardOutcome::Allow
    );

    // 3. Outside-repo `evaluate_guard` returns `OutsideRepo` even for an unknown client.
    let outside = tempfile::tempdir().expect("outside dir");
    assert_eq!(
        evaluate_guard(
            outside.path(),
            "unknown-client",
            "",
            "PreToolUse",
            &json!({})
        ),
        GuardOutcome::OutsideRepo
    );

    // 4. `uor-harness loop install --client claude` is plan-only by default (writes nothing).
    let home = tempfile::tempdir().expect("temp home");
    let kit = make_synthetic_kit();
    let out = Command::new(bin())
        .current_dir(a_root)
        .args(["loop", "install", "--client", "claude", "--home"])
        .arg(home.path())
        .arg("--zip")
        .arg(kit.path())
        .output()
        .expect("run loop install default plan-only");
    assert_eq!(out.status.code(), Some(0));
    assert!(
        !home.path().join(".claude/settings.json").exists(),
        "default loop install without --apply must not write files"
    );
}
