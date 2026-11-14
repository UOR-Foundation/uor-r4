//! Bounded GitHub API access through the host-approved `gh` companion.
//!
//! Credentials stay in `gh`'s own host configuration. The page supplies only an
//! API shape and a pinned repository; it never supplies an executable, token,
//! proxy, environment override or arbitrary command line.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};
use serde_json::Value;

const GH_PATHS: [&str; 3] = ["/opt/homebrew/bin/gh", "/usr/local/bin/gh", "/usr/bin/gh"];
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoTarget {
    pub owner: String,
    pub repo: String,
    pub default_branch: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubError {
    pub code: String,
    pub message: String,
}

impl GitHubError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

pub trait GithubExecutor: Send {
    fn execute(
        &mut self,
        method: &str,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Value, GitHubError>;
}

#[derive(Debug, Clone)]
pub struct FixedGhExecutor {
    path: PathBuf,
}

impl FixedGhExecutor {
    pub fn with_path(path: impl AsRef<Path>) -> Result<Self, GitHubError> {
        let path = path.as_ref();
        if !GH_PATHS.iter().any(|allowed| path == Path::new(allowed)) {
            return Err(GitHubError::new(
                "host_unavailable",
                "only the fixed absolute gh executable is allowed",
            ));
        }
        if !fs::metadata(path)
            .map(|metadata| metadata.is_file())
            .unwrap_or(false)
        {
            return Err(GitHubError::new(
                "host_unavailable",
                "the fixed gh executable is unavailable",
            ));
        }
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    pub fn detect() -> Result<Self, GitHubError> {
        for path in GH_PATHS {
            if let Ok(executor) = Self::with_path(path) {
                return Ok(executor);
            }
        }
        Err(GitHubError::new(
            "host_unavailable",
            "no fixed gh executable is installed",
        ))
    }
}

impl GithubExecutor for FixedGhExecutor {
    fn execute(
        &mut self,
        method: &str,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Value, GitHubError> {
        if !matches!(method, "GET" | "POST" | "PUT") || !path.starts_with("/repos/") {
            return Err(GitHubError::new(
                "invalid_arguments",
                "unsupported gh API method or endpoint",
            ));
        }
        let body_bytes = match body {
            Some(value) => {
                let bytes = serde_json::to_vec(value)
                    .map_err(|error| GitHubError::new("invalid_arguments", error.to_string()))?;
                if bytes.len() > MAX_BODY_BYTES {
                    return Err(GitHubError::new(
                        "limit_exceeded",
                        "GitHub request body is too large",
                    ));
                }
                Some(bytes)
            }
            None => None,
        };
        let mut command = Command::new(&self.path);
        command.args(["api", "--method", method, path]);
        if body_bytes.is_some() {
            command.args(["--input", "-"]);
        }
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().map_err(|error| {
            GitHubError::new("host_unavailable", format!("gh could not start: {error}"))
        })?;
        if let Some(bytes) = body_bytes {
            child
                .stdin
                .as_mut()
                .ok_or_else(|| GitHubError::new("host_unavailable", "gh stdin is unavailable"))?
                .write_all(&bytes)
                .map_err(|error| {
                    GitHubError::new("host_unavailable", format!("gh input failed: {error}"))
                })?;
        }
        let output = child.wait_with_output().map_err(|error| {
            GitHubError::new("host_unavailable", format!("gh execution failed: {error}"))
        })?;
        if output.stdout.len() > MAX_RESPONSE_BYTES || output.stderr.len() > MAX_RESPONSE_BYTES {
            return Err(GitHubError::new(
                "limit_exceeded",
                "gh response exceeds its bounded output",
            ));
        }
        if !output.status.success() {
            return Err(GitHubError::new(
                "request_failed",
                sanitize(&String::from_utf8_lossy(&output.stderr)),
            ));
        }
        serde_json::from_slice(&output.stdout).map_err(|error| {
            GitHubError::new(
                "invalid_response",
                format!("gh returned invalid JSON: {error}"),
            )
        })
    }
}

fn sanitize(text: &str) -> String {
    text.split_whitespace()
        .map(|part| {
            if part.starts_with("github_pat_")
                || part.starts_with("ghp_")
                || part.starts_with("Bearer")
            {
                "[redacted]".to_string()
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(2048)
        .collect()
}

pub fn target_from_arguments(arguments: &Value) -> Result<RepoTarget, GitHubError> {
    let target = arguments.get("target").cloned().unwrap_or(Value::Null);
    let target: RepoTarget = serde_json::from_value(target)
        .map_err(|_| GitHubError::new("invalid_arguments", "target must be an exact RepoTarget"))?;
    if target.owner.is_empty()
        || target.repo.is_empty()
        || target.default_branch.is_empty()
        || !target
            .owner
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
        || !target.repo.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        })
    {
        return Err(GitHubError::new(
            "invalid_arguments",
            "repository target is invalid",
        ));
    }
    Ok(target)
}

pub fn validate_request(
    method: &str,
    path: &str,
    target: &RepoTarget,
    body: Option<&Value>,
) -> Result<(), GitHubError> {
    let prefix = format!("/repos/{}/{}", target.owner, target.repo);
    if !path.starts_with(&prefix)
        || path
            .get(prefix.len()..prefix.len().saturating_add(1))
            .is_some_and(|character| character != "/" && !character.is_empty())
    {
        return Err(GitHubError::new(
            "cross_repository",
            "endpoint does not match the pinned repository",
        ));
    }
    match method {
        "GET" => {
            if body.is_some_and(|value| !value.is_null()) {
                return Err(GitHubError::new(
                    "invalid_arguments",
                    "GET may not carry a write body",
                ));
            }
        }
        "POST" | "PUT" => {
            let Some(body) = body.filter(|value| value.is_object()) else {
                return Err(GitHubError::new(
                    "invalid_arguments",
                    "GitHub write body must be an object",
                ));
            };
            if path == format!("{prefix}/git/refs") && method == "POST" {
                let reference = body
                    .get("ref")
                    .and_then(Value::as_str)
                    .ok_or_else(|| GitHubError::new("invalid_arguments", "ref is required"))?;
                let branch = reference.strip_prefix("refs/heads/").ok_or_else(|| {
                    GitHubError::new("invalid_arguments", "ref must be refs/heads/<branch>")
                })?;
                if branch == target.default_branch {
                    return Err(GitHubError::new(
                        "default_branch_refused",
                        "writes to the default branch are refused",
                    ));
                }
                let sha = body.get("sha").and_then(Value::as_str).unwrap_or("");
                if sha.len() != 40 || !sha.chars().all(|character| character.is_ascii_hexdigit()) {
                    return Err(GitHubError::new(
                        "invalid_arguments",
                        "base ref SHA must have 40 hexadecimal characters",
                    ));
                }
            } else if path.starts_with(&format!("{prefix}/contents/")) && method == "PUT" {
                let branch = body.get("branch").and_then(Value::as_str).unwrap_or("");
                if branch == target.default_branch {
                    return Err(GitHubError::new(
                        "default_branch_refused",
                        "writes to the default branch are refused",
                    ));
                }
                for field in ["message", "content", "sha"] {
                    if body.get(field).and_then(Value::as_str).is_none() {
                        return Err(GitHubError::new(
                            "invalid_arguments",
                            format!("{field} is required"),
                        ));
                    }
                }
            } else if path == format!("{prefix}/pulls") && method == "POST" {
                let head = body.get("head").and_then(Value::as_str).unwrap_or("");
                let Some((owner, branch)) = head.split_once(':') else {
                    return Err(GitHubError::new(
                        "cross_repository",
                        "pull head must be owner:branch",
                    ));
                };
                if owner != target.owner {
                    return Err(GitHubError::new(
                        "cross_repository",
                        "cross-repository pull requests are refused",
                    ));
                }
                if branch == target.default_branch {
                    return Err(GitHubError::new(
                        "default_branch_refused",
                        "pull head may not be the default branch",
                    ));
                }
                if body
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .is_empty()
                    || body
                        .get("base")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .is_empty()
                    || body.get("body").and_then(Value::as_str).is_none()
                {
                    return Err(GitHubError::new(
                        "invalid_arguments",
                        "pull title, base and body are required",
                    ));
                }
            } else {
                return Err(GitHubError::new(
                    "invalid_arguments",
                    "unsupported GitHub write endpoint",
                ));
            }
        }
        _ => {
            return Err(GitHubError::new(
                "invalid_arguments",
                "unsupported GitHub method",
            ))
        }
    }
    Ok(())
}
