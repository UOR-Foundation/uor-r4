//! `uor-review v1` records, setup-PR status, the D59 switch and the review gate (loop PR 6),
//! through the pure functions in `uor_r4_harness::loopreview`.

use std::collections::BTreeMap;
use uor_r4_harness::loopreview::*;

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER_HEAD: &str = "fedcba9876543210fedcba9876543210fedcba98";
const PR: u64 = 2926;
const LOOP_PAGE: &str = include_str!("../../../docs/loop/README.md");

fn vendors() -> BTreeMap<String, String> {
    [
        ("claude", "anthropic"),
        ("codex", "openai"),
        ("kimi", "moonshot"),
    ]
    .into_iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect()
}

fn rows() -> Vec<String> {
    build_rows(LOOP_PAGE)
}

fn record(lab: &str, verdict: Verdict) -> Record {
    Record {
        lab: lab.to_string(),
        pr: PR,
        head: HEAD.to_string(),
        model: "unknown".to_string(),
        launch: "manual".to_string(),
        verdict,
        order: "docs/loop/tasks/infra-loop-harness/claude-18/ORDER.md".to_string(),
        fresh_context: false,
        done_when: DoneWhen::Host("cargo test -p uor-r4-harness: ok".to_string()),
    }
}

fn body(lab: &str, verdict: Verdict) -> String {
    render(&record(lab, verdict))
}

/// A gate run over `comments` for a PR built by `order_lab` with item `item`.
struct Case<'a> {
    order_lab: &'a str,
    item: &'a str,
    setup_row: Option<&'a str>,
    d59: bool,
    doc_only: bool,
}

const CODE_PR: Case = Case {
    order_lab: "claude",
    item: "infra-other",
    setup_row: None,
    d59: true,
    doc_only: false,
};

const SETUP_PR: Case = Case {
    order_lab: "claude",
    item: "infra-loop-pr06-review-records",
    setup_row: Some("6"),
    d59: true,
    doc_only: false,
};

fn run(c: &Case, comments: &[String]) -> GateOutcome {
    let (rows, vendors) = (rows(), vendors());
    gate(&GateInput {
        pr: PR,
        head: HEAD,
        order_lab: c.order_lab,
        order_item: c.item,
        setup_row: c.setup_row,
        rows: &rows,
        vendors: &vendors,
        d59_in_force: c.d59,
        doc_only: c.doc_only,
        comments,
    })
}

fn has(out: &GateOutcome, text: &str) -> bool {
    out.details.iter().any(|d| d.contains(text))
}

#[test]
fn planted_fault_an_untagged_record_is_refused() {
    let tagged = body("codex", Verdict::Approve);
    let untagged = tagged.replacen("[codex] uor-review v1", "uor-review v1", 1);
    match parse(&untagged) {
        Parsed::Malformed(why) => assert!(why.contains("untagged"), "{why}"),
        other => panic!("expected Malformed, got {other:?}"),
    }
    let out = run(&CODE_PR, &[untagged]);
    assert!(!out.pass, "{:?}", out.details);
    assert!(
        has(&out, "refused, malformed record: untagged"),
        "{:?}",
        out.details
    );

    let mismatched = tagged.replacen("[codex] uor-review v1", "[kimi] uor-review v1", 1);
    match parse(&mismatched) {
        Parsed::Malformed(why) => assert!(why.contains("differs from the lab"), "{why}"),
        other => panic!("expected Malformed, got {other:?}"),
    }
    let out = run(&CODE_PR, &[mismatched]);
    assert!(!out.pass);
    assert!(has(&out, "the tag [kimi] differs"), "{:?}", out.details);

    // The same record, tagged, counts: the refusal is the tag alone.
    assert!(run(&CODE_PR, &[tagged]).pass);
}

#[test]
fn planted_fault_a_same_vendor_record_outside_d59_is_refused() {
    let out = run(&CODE_PR, &[body("claude", Verdict::Approve)]);
    assert!(!out.pass);
    assert!(!out.setup_pr);
    assert!(has(&out, "not counted, same vendor"), "{:?}", out.details);
    assert!(run(&CODE_PR, &[body("codex", Verdict::Approve)]).pass);
}

#[test]
fn planted_fault_a_non_claude_record_on_a_setup_pr_is_refused() {
    let out = run(&SETUP_PR, &[body("codex", Verdict::Approve)]);
    assert!(out.setup_pr);
    assert!(!out.pass);
    assert!(
        has(
            &out,
            "setup PR under D59 counts only claude records, not codex"
        ),
        "{:?}",
        out.details
    );
    let out = run(
        &SETUP_PR,
        &[
            body("codex", Verdict::Approve),
            body("claude", Verdict::Approve),
        ],
    );
    assert!(out.pass, "{:?}", out.details);
    assert!(has(&out, "[claude] APPROVE"), "{:?}", out.details);
    // A later CHANGES from another lab does not count, so it does not block.
    let out = run(
        &SETUP_PR,
        &[
            body("claude", Verdict::Approve),
            body("codex", Verdict::Changes),
        ],
    );
    assert!(out.pass, "{:?}", out.details);
    assert!(has(&out, "[codex] CHANGES"), "{:?}", out.details);
}

#[test]
fn a_claude_code_pr_without_a_setup_row_refuses_a_claude_review() {
    let c = Case {
        item: "infra-loop-pr06-review-records",
        setup_row: None,
        ..CODE_PR
    };
    let out = run(&c, &[body("claude", Verdict::Approve)]);
    assert!(!out.setup_pr);
    assert!(!out.pass);
    assert!(has(&out, "same vendor"), "{:?}", out.details);
}

#[test]
fn a_setup_row_not_in_the_table_is_not_a_setup_pr() {
    let c = Case {
        setup_row: Some("99"),
        ..SETUP_PR
    };
    let out = run(&c, &[body("claude", Verdict::Approve)]);
    assert!(!out.setup_pr);
    assert!(!out.pass);
    assert!(has(&out, "same vendor"), "{:?}", out.details);
}

#[test]
fn the_item_must_be_the_harness_item_or_the_row_s_own() {
    let rows = rows();
    assert!(!is_setup_pr(Some("6"), &rows, "infra-loop-pr07b-session"));
    assert!(is_setup_pr(Some("6"), &rows, "infra-loop-harness"));
    assert!(is_setup_pr(
        Some("6"),
        &rows,
        "infra-loop-pr06-review-records"
    ));
    assert!(!is_setup_pr(None, &rows, "infra-loop-harness"));
    let c = Case {
        item: "infra-loop-pr07b-session",
        ..SETUP_PR
    };
    let out = run(&c, &[body("claude", Verdict::Approve)]);
    assert!(!out.setup_pr);
    assert!(!out.pass);
}

#[test]
fn a_doc_only_pr_takes_a_fresh_same_lab_record() {
    let doc = Case {
        doc_only: true,
        ..CODE_PR
    };
    let fresh = render(&Record {
        fresh_context: true,
        ..record("claude", Verdict::Approve)
    });
    assert!(fresh.contains("\ncontext: fresh\n"));
    let out = run(&doc, std::slice::from_ref(&fresh));
    assert!(out.pass, "{:?}", out.details);

    let out = run(&doc, &[body("claude", Verdict::Approve)]);
    assert!(!out.pass);
    assert!(has(&out, "needs `context: fresh`"), "{:?}", out.details);

    let out = run(&CODE_PR, &[fresh]);
    assert!(!out.pass);
    assert!(has(&out, "same vendor"), "{:?}", out.details);
}

#[test]
fn the_last_counted_record_decides() {
    let approve = body("codex", Verdict::Approve);
    let changes = body("codex", Verdict::Changes);
    let out = run(&CODE_PR, &[approve.clone(), changes.clone()]);
    assert!(!out.pass);
    assert!(
        has(&out, "fail: the last counted record is CHANGES"),
        "{:?}",
        out.details
    );
    assert!(run(&CODE_PR, &[changes, approve]).pass);

    let stale = render(&Record {
        head: OTHER_HEAD.to_string(),
        ..record("codex", Verdict::Approve)
    });
    let out = run(&CODE_PR, &[stale]);
    assert!(!out.pass);
    assert!(has(&out, "not the current head"), "{:?}", out.details);

    let elsewhere = render(&Record {
        pr: PR + 1,
        ..record("codex", Verdict::Approve)
    });
    let out = run(&CODE_PR, &[elsewhere]);
    assert!(!out.pass);
    assert!(has(&out, "not PR 2926"), "{:?}", out.details);

    let out = run(&CODE_PR, &["LGTM".to_string()]);
    assert!(!out.pass);
    assert!(has(&out, "fail: no counted record"), "{:?}", out.details);
}

#[test]
fn the_switch_ends_d59_only_when_listed_and_ended() {
    let unlisted = r#"{"schema":1,"paths":[{"path":"CHARTER.md"}]}"#;
    let listed = r#"{"schema":1,"paths":[{"path":"CHARTER.md"},{"path":"docs/harness/d59.json"}]}"#;
    let ended = r#"{"schema":1,"d59":"ended","owner_words":"x","time":"2026-10-21T00:00:00Z"}"#;
    assert!(d59_in_force(unlisted, Some(ended)));
    assert!(!d59_in_force(listed, Some(ended)));
    assert!(d59_in_force(listed, Some("{not json")));
    assert!(d59_in_force(listed, None));
    assert!(d59_in_force(listed, Some(r#"{"d59":"in force"}"#)));
    let real = include_str!("../../../docs/harness/owner-paths.json");
    assert!(d59_in_force(real, Some(ended)));

    let off = Case {
        d59: d59_in_force(listed, Some(ended)),
        ..SETUP_PR
    };
    let out = run(&off, &[body("claude", Verdict::Approve)]);
    assert!(out.setup_pr);
    assert!(!out.pass);
    assert!(has(&out, "same vendor"), "{:?}", out.details);
    assert!(run(&off, &[body("codex", Verdict::Approve)]).pass);
}

#[test]
fn rows_prefixes_round_trip_and_setup_row() {
    let rows = rows();
    for r in ["A", "1", "6", "7a", "7b", "7c", "12"] {
        assert!(rows.iter().any(|x| x == r), "{r} missing from {rows:?}");
    }
    assert_eq!(row_item_prefix("6").as_deref(), Some("infra-loop-pr06-"));
    assert_eq!(row_item_prefix("7a").as_deref(), Some("infra-loop-pr07a-"));
    assert_eq!(row_item_prefix("12").as_deref(), Some("infra-loop-pr12-"));
    assert_eq!(row_item_prefix("A"), None);
    assert_eq!(row_item_prefix("1"), None);

    let mut r = record("kimi", Verdict::Changes);
    r.fresh_context = true;
    r.done_when = DoneWhen::Gpu("ab".repeat(32));
    assert_eq!(parse(&render(&r)), Parsed::Record(r.clone()));
    let plain = record("codex", Verdict::Approve);
    let with_findings = format!("{}\nFindings: none.\nverdict: CHANGES\n", render(&plain));
    assert_eq!(parse(&with_findings), Parsed::Record(plain));

    assert_eq!(
        setup_row("Lab: claude. Setup row: 7b. Base: x").as_deref(),
        Some("7b")
    );
    assert_eq!(setup_row("Lab: claude."), None);
}

#[test]
fn malformed_fields_are_named() {
    let good = body("codex", Verdict::Approve);
    let cases = [
        (good.replace("pr: 2926\n", ""), "missing field \"pr\""),
        (
            good.replace("pr: 2926\n", "pr: 2926\npr: 2926\n"),
            "duplicated field",
        ),
        (good.replace(HEAD, &HEAD.to_uppercase()), "bad head"),
        (good.replace("pr: 2926", "pr: #2926"), "bad pr"),
        (
            good.replace("verdict: APPROVE", "verdict: LGTM"),
            "bad verdict",
        ),
        (
            good.replace("done-when: host: ", "done-when: maybe: "),
            "bad done-when",
        ),
        (
            good.replace("model: unknown", "model: unknown\nmood: good"),
            "unknown field",
        ),
    ];
    for (text, want) in cases {
        match parse(&text) {
            Parsed::Malformed(why) => assert!(why.contains(want), "{want}: {why}"),
            other => panic!("{want}: got {other:?}"),
        }
    }
    assert_eq!(parse("Looks good to me"), Parsed::NotARecord);
}
