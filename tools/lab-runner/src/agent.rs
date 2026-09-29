//! LaunchAgent installer. Writes the plist only; the human runs the printed
//! `launchctl bootstrap` command. Tests exercise `write_plist` against a
//! temporary path and never touch launchd.

use std::path::{Path, PathBuf};

use crate::{invalid, Result};

pub const AGENT_LABEL: &str = "org.uor.lab-runner";

pub fn default_plist_path() -> Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .ok_or_else(|| invalid("HOME is not set; cannot locate LaunchAgents"))?;
    Ok(PathBuf::from(home)
        .join("Library/LaunchAgents")
        .join(format!("{AGENT_LABEL}.plist")))
}

pub fn plist_content(program: &Path, root: &Path, ledger_dir: &Path) -> String {
    let stdout = root.join("daemon.out.log");
    let stderr = root.join("daemon.err.log");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{AGENT_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{program}</string>
        <string>--root</string>
        <string>{root}</string>
        <string>--ledger-dir</string>
        <string>{ledger}</string>
        <string>daemon</string>
    </array>
    <key>KeepAlive</key>
    <true/>
    <key>RunAtLoad</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{stdout}</string>
    <key>StandardErrorPath</key>
    <string>{stderr}</string>
</dict>
</plist>
"#,
        program = program.display(),
        root = root.display(),
        ledger = ledger_dir.display(),
        stdout = stdout.display(),
        stderr = stderr.display(),
    )
}

/// Write the plist to `plist_path` and return the bootstrap command for a
/// human to run. Does not invoke launchctl.
pub fn write_plist(
    plist_path: &Path,
    program: &Path,
    root: &Path,
    ledger_dir: &Path,
) -> Result<String> {
    if let Some(parent) = plist_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    crate::atomic_write(
        plist_path,
        plist_content(program, root, ledger_dir).as_bytes(),
    )?;
    let uid = Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|uid| !uid.is_empty())
        .unwrap_or_else(|| "$(id -u)".to_string());
    Ok(format!(
        "launchctl bootstrap gui/{uid} {}",
        plist_path.display()
    ))
}

use std::process::Command;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_mentions_label_paths_and_keepalive() {
        let content = plist_content(
            Path::new("/usr/local/bin/lab-runner"),
            Path::new("/Volumes/UOR-Workspace/runner"),
            Path::new("/home/u/.uor-models/native-joint-learning-2026-09-04"),
        );
        assert!(content.contains("<string>org.uor.lab-runner</string>"));
        assert!(content.contains("<key>KeepAlive</key>"));
        assert!(content.contains("<key>RunAtLoad</key>"));
        assert!(content.contains("/usr/local/bin/lab-runner"));
        assert!(content.contains("<string>daemon</string>"));
        assert!(content.contains("daemon.out.log"));
        assert!(content.contains("daemon.err.log"));
    }

    #[test]
    fn write_plist_targets_a_tempdir_and_prints_bootstrap() {
        let dir = std::env::temp_dir().join(format!(
            "lab-runner-agent-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let plist = dir.join("org.uor.lab-runner.plist");
        let command = write_plist(
            &plist,
            Path::new("/usr/local/bin/lab-runner"),
            Path::new("/tmp/runner"),
            Path::new("/tmp/ledger"),
        )
        .unwrap();
        assert!(plist.is_file());
        assert!(command.starts_with("launchctl bootstrap gui/"));
        assert!(command.contains(plist.to_str().unwrap()));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
