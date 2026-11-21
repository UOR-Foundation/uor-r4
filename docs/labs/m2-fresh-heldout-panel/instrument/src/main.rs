//! CLI entry point for the M2 fresh held-out panel (`m2-fresh256`) and `dev512` noise audit.

use m2_fresh_heldout_panel_instrument::{
    analyze_saved437_dev512_spread, cases_outside_reader_grammar, generate_m2_fresh256_panel,
    generate_m2_fresh256_panel_from, grade_reader_replies, load_pinned_byte_bpe_tokenizer,
    sha256_hex, swap_pair_replies, verify_m2_fresh256_panel, verify_m2_fresh256_panel_from,
    LiteralSet, READER_REPLY_OPENINGS,
};
use std::path::Path;

fn main() -> Result<(), String> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .ok_or_else(|| "cannot resolve repo root".to_string())?;

    let dev_spread = analyze_saved437_dev512_spread(repo_root)?;
    let args: Vec<String> = std::env::args().collect();

    // `--literals FILE`: the panel is built from that literal set (kept
    // outside git), encoded with the pinned tokenizer, checked against the
    // 74 historical and the 34 `FRESH256_*` literals, and required to sit
    // inside the reader grammar's published forms.
    let literals = match args.iter().position(|a| a == "--literals") {
        Some(pos) => {
            let path = args
                .get(pos + 1)
                .ok_or_else(|| "--literals requires a path".to_string())?;
            Some(LiteralSet::load(Path::new(path))?)
        }
        None => None,
    };
    let (bundle, audit, outside_grammar) = match &literals {
        Some((set, literals_sha256)) => {
            let tok = load_pinned_byte_bpe_tokenizer(repo_root)?;
            let encode = |text: &str| Ok(tok.encode(text));
            let bundle = generate_m2_fresh256_panel_from(set, &encode, Some(literals_sha256))?;
            let old: Vec<String> = LiteralSet::fresh256().all().into_iter().collect();
            let old: Vec<&str> = old.iter().map(String::as_str).collect();
            let audit = verify_m2_fresh256_panel_from(repo_root, &bundle, set, &encode, &old)?;
            let grammar_path =
                repo_root.join("docs/labs/m2-fresh-heldout-panel/reader-grammar.json");
            let grammar_bytes = std::fs::read(&grammar_path)
                .map_err(|e| format!("{}: {e}", grammar_path.display()))?;
            let grammar: serde_json::Value = serde_json::from_slice(&grammar_bytes)
                .map_err(|e| format!("{}: {e}", grammar_path.display()))?;
            let outside = cases_outside_reader_grammar(&grammar, &bundle)?;
            if outside != 0 {
                return Err(format!(
                    "{outside} cases are outside the reader grammar's published forms"
                ));
            }
            (bundle, audit, Some(outside))
        }
        None => {
            let bundle = generate_m2_fresh256_panel()?;
            let audit = verify_m2_fresh256_panel(repo_root, &bundle)?;
            (bundle, audit, None)
        }
    };
    let literals_sha256 = literals.as_ref().map(|(_, sha)| sha.clone());

    if let Some(pos) = args.iter().position(|a| a == "--emit-dir") {
        let dir = args
            .get(pos + 1)
            .ok_or_else(|| "--emit-dir requires a path".to_string())?;
        let out_dir = Path::new(dir);
        std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
        std::fs::write(out_dir.join("plan.json"), &bundle.plan_json).map_err(|e| e.to_string())?;
        std::fs::write(out_dir.join("inputs.json"), &bundle.inputs_json)
            .map_err(|e| e.to_string())?;
        std::fs::write(out_dir.join("labels.json"), &bundle.labels_json)
            .map_err(|e| e.to_string())?;
        std::fs::write(out_dir.join("manifest.json"), &bundle.manifest_json)
            .map_err(|e| e.to_string())?;
    }

    // `--grade REPORT_JSON`: grades a `conjunction-probe reader-replies`
    // report (its `replies`, one per case in `inputs.json` order) and prints
    // the grade alone.
    if let Some(pos) = args.iter().position(|a| a == "--grade") {
        let path = args
            .get(pos + 1)
            .ok_or_else(|| "--grade requires a report path".to_string())?;
        let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        let report: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))?;
        let replies: Vec<Vec<u32>> = serde_json::from_value(report["replies"].clone())
            .map_err(|e| format!("{path}: replies: {e}"))?;
        // `--reader SHA256`: the report must come from that reader file.
        let expected = args
            .iter()
            .position(|a| a == "--reader")
            .and_then(|pos| args.get(pos + 1))
            .ok_or_else(|| "--grade requires --reader SHA256".to_string())?;
        if report["reader_sha256"].as_str() != Some(expected.as_str()) {
            return Err(format!(
                "{path}: reader_sha256 {} is not the expected {expected}",
                report["reader_sha256"]
            ));
        }
        // `--swap-pairs`: a control; each pair's two replies are exchanged
        // before grading, so each is graded against the other role's question.
        let pairs_swapped = args.iter().any(|a| a == "--swap-pairs");
        let replies = if pairs_swapped {
            swap_pair_replies(&replies)?
        } else {
            replies
        };
        let tok = load_pinned_byte_bpe_tokenizer(repo_root)?;
        let grade = grade_reader_replies(&tok, &bundle, &replies)?;
        let mut out = serde_json::json!({
                "schema": "uor-r4.m2-fresh256-reader-grade/1",
                "report_sha256": sha256_hex(&bytes),
                "reader_sha256": report["reader_sha256"],
                "panel_sha256": report["panel_sha256"],
                "inputs_sha256": bundle.inputs_sha256,
                "labels_sha256": bundle.labels_sha256,
                "openings": READER_REPLY_OPENINGS,
                "grade": grade,
                "bar_pairs": 52,
                "bar_met": grade.pairs_both_complete_distinct >= 52 && grade.complete >= 104,
                "pairs_swapped": pairs_swapped,
        });
        if let Some(sha) = &literals_sha256 {
            out["literals_sha256"] = serde_json::json!(sha);
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    let mut summary = serde_json::json!({
        "dev512_spread": dev_spread,
        "fresh256_audit": audit,
        "fresh256_seals": {
            "plan_bytes": bundle.plan_json.len(),
            "plan_sha256": bundle.plan_sha256,
            "inputs_bytes": bundle.inputs_json.len(),
            "inputs_sha256": bundle.inputs_sha256,
            "labels_bytes": bundle.labels_json.len(),
            "labels_sha256": bundle.labels_sha256,
            "manifest_bytes": bundle.manifest_json.len(),
            "manifest_sha256": bundle.manifest_sha256,
        }
    });
    if let (Some(sha), Some(outside)) = (&literals_sha256, outside_grammar) {
        summary["literals_sha256"] = serde_json::json!(sha);
        summary["cases_outside_reader_grammar"] = serde_json::json!(outside);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?
    );
    Ok(())
}
