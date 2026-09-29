//! Job spec schema `uor-r4.lab-runner-job/1` and its validation.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::{invalid, Result};

pub const JOB_SCHEMA: &str = "uor-r4.lab-runner-job/1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KillKind {
    /// Kill once `wall_s` has elapsed.
    Wall,
    /// Additionally kill when `value` (a literal substring) appears in the
    /// job's stdout or stderr log.
    LogMatch,
    /// Additionally run `value` as a shell check command (`sh -c value`);
    /// a nonzero exit status kills the job.
    Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillCriterion {
    pub kind: KillKind,
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSpec {
    #[serde(default)]
    pub schema: Option<String>,
    pub id: String,
    pub lab: String,
    pub cwd: PathBuf,
    pub argv: Vec<String>,
    #[serde(default)]
    pub env: Option<BTreeMap<String, String>>,
    pub threads: u32,
    pub rss_gib: f64,
    #[serde(default)]
    pub gpu: bool,
    pub wall_s: u64,
    pub kill_criterion: KillCriterion,
    #[serde(default)]
    pub exclusive: bool,
}

impl JobSpec {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let spec: JobSpec = serde_json::from_slice(bytes)?;
        spec.validate()?;
        Ok(spec)
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(schema) = &self.schema {
            if schema != JOB_SCHEMA {
                return Err(invalid(format!(
                    "spec schema must be {JOB_SCHEMA}, found {schema}"
                )));
            }
        }
        if self.id.is_empty() || self.id.contains('/') || self.id.contains("..") {
            return Err(invalid("spec id must be a non-empty single path component"));
        }
        if self.argv.is_empty() {
            return Err(invalid("spec argv must be non-empty"));
        }
        if !self.cwd.is_dir() {
            return Err(invalid(format!(
                "spec cwd {} does not exist or is not a directory",
                self.cwd.display()
            )));
        }
        if self.threads == 0 {
            return Err(invalid("spec threads must be at least 1"));
        }
        if !(self.rss_gib > 0.0) || !self.rss_gib.is_finite() {
            return Err(invalid("spec rss_gib must be a positive finite number"));
        }
        if self.wall_s == 0 {
            return Err(invalid("spec wall_s must be at least 1"));
        }
        match self.kill_criterion.kind {
            KillKind::Wall => {}
            KillKind::LogMatch | KillKind::Command => {
                if self.kill_criterion.value.is_empty() {
                    return Err(invalid(
                        "kill_criterion value must be non-empty for log_match and command",
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_spec_json() -> serde_json::Value {
        serde_json::json!({
            "schema": JOB_SCHEMA,
            "id": "job-1",
            "lab": "lab-test",
            "cwd": std::env::temp_dir(),
            "argv": ["sh", "-c", "true"],
            "threads": 1,
            "rss_gib": 0.5,
            "gpu": false,
            "wall_s": 10,
            "kill_criterion": {"kind": "wall", "value": ""},
            "exclusive": false
        })
    }

    #[test]
    fn valid_spec_parses() {
        let spec = JobSpec::parse(&serde_json::to_vec(&base_spec_json()).unwrap()).unwrap();
        assert_eq!(spec.id, "job-1");
        assert_eq!(spec.kill_criterion.kind, KillKind::Wall);
    }

    #[test]
    fn missing_wall_s_names_the_field() {
        let mut value = base_spec_json();
        value.as_object_mut().unwrap().remove("wall_s");
        let error = JobSpec::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(
            error.to_string().contains("wall_s"),
            "error must name wall_s: {error}"
        );
    }

    #[test]
    fn missing_kill_criterion_names_the_field() {
        let mut value = base_spec_json();
        value.as_object_mut().unwrap().remove("kill_criterion");
        let error = JobSpec::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(
            error.to_string().contains("kill_criterion"),
            "error must name kill_criterion: {error}"
        );
    }

    #[test]
    fn empty_argv_and_missing_cwd_are_rejected() {
        let mut value = base_spec_json();
        value["argv"] = serde_json::json!([]);
        let error = JobSpec::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(error.to_string().contains("argv"), "{error}");

        let mut value = base_spec_json();
        value["cwd"] = serde_json::json!("/definitely/not/a/real/dir-lab-runner");
        let error = JobSpec::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(error.to_string().contains("cwd"), "{error}");
    }

    #[test]
    fn log_match_requires_a_value() {
        let mut value = base_spec_json();
        value["kill_criterion"] = serde_json::json!({"kind": "log_match", "value": ""});
        let error = JobSpec::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(error.to_string().contains("kill_criterion"), "{error}");
    }
}
