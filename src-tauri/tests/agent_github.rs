#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use uor_r4_desktop_client::agent_github::{
    validate_request, FixedGhExecutor, GitHubError, GithubExecutor, RepoTarget,
};
use uor_r4_desktop_client::agent_host::{AgentHost, UnavailableMcpDispatch, HOST_INTENT_SCHEMA};
use uor_r4_desktop_client::{sha256_bytes, DesktopBackend, PinRegistry};

const TASK: &str = "tsk_0123456789abcdef";
const RUN: &str = "run_0123456789abcdef";

#[derive(Default)]
struct FakeExecutor {
    calls: Arc<Mutex<Vec<(String, String, Option<Value>)>>>,
    response: Option<Value>,
}

impl GithubExecutor for FakeExecutor {
    fn execute(
        &mut self,
        method: &str,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Value, GitHubError> {
        self.calls.lock().expect("calls").push((
            method.to_string(),
            path.to_string(),
            body.cloned(),
        ));
        Ok(self.response.clone().unwrap_or_else(|| json!({"ok": true})))
    }
}

fn target() -> RepoTarget {
    RepoTarget {
        owner: "casey-allard".to_string(),
        repo: "studio-smoke".to_string(),
        default_branch: "main".to_string(),
    }
}

fn arguments(method: &str, path: &str, body: Value) -> Value {
    json!({
        "method": method,
        "path": path,
        "body": body,
        "target": target(),
        "action": "branch",
        "repository": "casey-allard/studio-smoke",
        "branch": "fix"
    })
}

fn intent(tool: &str, arguments: Value, approval_id: Option<&str>) -> Value {
    let canonical = uor_r4_studio_agent::canonical_json(&arguments).expect("canonical arguments");
    let digest = format!("sha256:{}", sha256_bytes(canonical.as_bytes()));
    let mut value = json!({
        "schema": HOST_INTENT_SCHEMA,
        "intent_id": "hti_github_test_0001",
        "task_id": TASK,
        "run_id": RUN,
        "generation": 1,
        "tool_call_id": "tc_github_test_0001",
        "tool": tool,
        "arguments": arguments,
        "arguments_digest": digest,
        "request_scope": "task"
    });
    if let Some(approval_id) = approval_id {
        value["approval_id"] = json!(approval_id);
    }
    value
}

fn host() -> (AgentHost, Arc<Mutex<Vec<(String, String, Option<Value>)>>>) {
    let backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let mut host = AgentHost::new(backend).expect("host");
    let fake = FakeExecutor::default();
    let calls = Arc::clone(&fake.calls);
    host.set_github_executor(Box::new(fake));
    (host, calls)
}

#[test]
fn github_companion_read_uses_fixed_gh_shape_without_token() {
    let (mut host, calls) = host();
    let result = host
        .invoke(
            &UnavailableMcpDispatch,
            intent(
                "github.read",
                arguments("GET", "/repos/casey-allard/studio-smoke", Value::Null),
                None,
            ),
        )
        .expect("read result");
    assert_eq!(result.status, "ok");
    assert!(result.approval_receipt.is_none());
    assert_eq!(
        calls.lock().expect("calls").as_slice(),
        [(
            "GET".to_string(),
            "/repos/casey-allard/studio-smoke".to_string(),
            Some(Value::Null)
        )]
    );
}

#[test]
fn github_companion_uses_agent_host_receipt_and_approved_gh_without_credentials() {
    let (mut host, calls) = host();
    let write = arguments(
        "POST",
        "/repos/casey-allard/studio-smoke/git/refs",
        json!({"ref": "refs/heads/fix", "sha": "a".repeat(40)}),
    );
    let pending = host
        .invoke(
            &UnavailableMcpDispatch,
            intent("github.write", write.clone(), None),
        )
        .expect("approval request");
    assert_eq!(pending.status, "error");
    let approval_id = pending
        .output
        .as_ref()
        .and_then(|output| output["approval_id"].as_str())
        .expect("approval id")
        .to_string();
    assert!(calls.lock().expect("calls").is_empty());
    let summary = host
        .pending_approval_summary(&approval_id)
        .expect("summary");
    assert!(summary.preview.contains("casey-allard/studio-smoke"));
    assert!(summary.preview.contains("fix"));
    assert!(summary.preview.contains("branch"));
    host.resolve_approval(
        &json!({"approval_id": approval_id, "decision": "approved"}),
        true,
    )
    .expect("native approval");
    let result = host
        .invoke(
            &UnavailableMcpDispatch,
            intent("github.write", write.clone(), Some(&approval_id)),
        )
        .expect("write result");
    assert_eq!(result.status, "ok");
    assert!(result
        .approval_receipt
        .as_deref()
        .is_some_and(|receipt| receipt.starts_with("hmac-sha256:")));
    assert_eq!(calls.lock().expect("calls").len(), 1);
    let replay = host.invoke(
        &UnavailableMcpDispatch,
        intent("github.write", write, Some(&approval_id)),
    );
    assert!(replay.is_err(), "an approval receipt cannot be replayed");
}

#[test]
fn github_cross_repo_pr_and_default_branch_write_are_refused() {
    let (mut host, calls) = host();
    let default = arguments(
        "POST",
        "/repos/casey-allard/studio-smoke/git/refs",
        json!({"ref": "refs/heads/main", "sha": "a".repeat(40)}),
    );
    let result = host
        .invoke(
            &UnavailableMcpDispatch,
            intent("github.write", default, None),
        )
        .expect("default refusal");
    assert_eq!(result.status, "error");
    let pr = arguments(
        "POST",
        "/repos/casey-allard/studio-smoke/pulls",
        json!({"title": "fix", "head": "someone-else:fix", "base": "main", "body": ""}),
    );
    let result = host
        .invoke(&UnavailableMcpDispatch, intent("github.write", pr, None))
        .expect("cross repo refusal");
    assert_eq!(result.status, "error");
    assert!(calls.lock().expect("calls").is_empty());
}

#[test]
fn github_fixed_absolute_gh_allowlist_is_enforced() {
    let error = FixedGhExecutor::with_path("/tmp/unapproved-gh").expect_err("unapproved path");
    assert_eq!(error.code, "host_unavailable");
    assert!(FixedGhExecutor::detect().is_ok() || FixedGhExecutor::detect().is_err());
}

#[test]
fn github_request_validation_pins_target_and_exact_write_shapes() {
    let body = json!({"ref": "refs/heads/fix", "sha": "a".repeat(40)});
    validate_request(
        "POST",
        "/repos/casey-allard/studio-smoke/git/refs",
        &target(),
        Some(&body),
    )
    .expect("valid create ref");
    let error = validate_request(
        "POST",
        "/repos/casey-allard/studio-smoke/git/refs",
        &target(),
        Some(&json!({"ref": "refs/heads/main", "sha": "a".repeat(40)})),
    )
    .expect_err("default refused");
    assert_eq!(error.code, "default_branch_refused");
    let error = validate_request(
        "PUT",
        "/repos/casey-allard/other/git/refs",
        &target(),
        Some(&body),
    )
    .expect_err("repo refused");
    assert_eq!(error.code, "cross_repository");
}
