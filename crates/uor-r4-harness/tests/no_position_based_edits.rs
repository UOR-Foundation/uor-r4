//! The hazard cannot be reintroduced into our own tooling.
//!
//! On 14 October `gh issue comment --edit-last` overwrote the whole body of a comment another lab wrote,
//! and GitHub keeps no edit history, so the text was unrecoverable.
//!
//! What this test CAN do is stop the hazard being baked into anything that executes: a script or a
//! workflow that edits by position fails here, before it can overwrite a comment on a schedule. It cannot
//! intercept a hand-typed `gh` call - which is exactly why the safe path had to be BUILT rather than
//! merely documented (`uor-harness comment edit --id <id> --body-file <f>`).

use std::path::{Path, PathBuf};

fn files_with_ext(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            files_with_ext(&p, exts, out);
        } else if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            if exts.iter().any(|x| name.ends_with(x)) {
                out.push(p);
            }
        }
    }
}

#[test]
fn no_executable_workflow_edits_a_comment_by_position() {
    // `cargo test` runs with the package directory as the working directory.
    let root = Path::new("../..");
    let mut files = Vec::new();
    files_with_ext(&root.join("scripts"), &[".py", ".sh", ".bash"], &mut files);
    files_with_ext(&root.join(".github"), &[".yml", ".yaml"], &mut files);
    assert!(
        !files.is_empty(),
        "found no scripts or workflows to scan - the test would pass vacuously"
    );

    let flag = "--edit-last";
    let mut offenders = Vec::new();
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            if line.contains(flag) {
                offenders.push(format!("{}:{}: {}", f.display(), i + 1, line.trim()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "a script or workflow edits a comment by POSITION - that flag edits the last comment on the \
         issue, not the last one this lab wrote, and every lab here posts as one account. Edit by id \
         instead: uor-harness comment edit --id <id> --body-file <f>.\n{}",
        offenders.join("\n")
    );
}
