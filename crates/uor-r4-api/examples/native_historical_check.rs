//! Actual selected-artifact API generation; authored expectations are assertions only.
use serde_json::{json, Value};
use std::{error::Error, fs};
use uor_r4_api::native_capability_api::{CompletionRequest, NativeModel, SessionConfig};
fn main() -> Result<(), Box<dyn Error>> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() != 4 {
        return Err("usage: native_historical_check MODEL CASES REPORT".into());
    }
    let bytes = fs::read(&a[1])?;
    let model = NativeModel::load_from_bytes(&bytes)?;
    let cases: Vec<Value> = serde_json::from_slice(&fs::read(&a[2])?)?;
    let mut checks = Vec::new();
    for c in cases.iter().filter(|c| !c["current_record"].is_null()) {
        let prompt = c["prompt"].as_str().ok_or("prompt absent")?;
        let expected = c["expected"].as_str().ok_or("expectation absent")?;
        let mut session = model.create_session(SessionConfig::default())?;
        let response = session.complete(CompletionRequest::new(prompt))?;
        let passed = response.text == expected && response.stopped_by == "eos";
        checks.push(json!({"id":c["id"],"prompt":prompt,"expected":expected,"response":response,"passed":passed}));
        if !passed {
            break;
        }
        let state = session.export_state()?;
        session.import_state(&state)?;
        let sum = session.complete(CompletionRequest::new("User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:"))?;
        let passed = sum.text == "18.\n" && sum.stopped_by == "eos";
        checks.push(json!({"id":format!("{}:checkpoint-independent-sum",c["id"].as_str().unwrap_or("case")),"response":sum,"passed":passed}));
        if !passed {
            break;
        }
    }
    let passed = !checks.is_empty() && checks.iter().all(|c| c["passed"] == true);
    fs::write(
        &a[3],
        serde_json::to_vec(
            &json!({"status":if passed{"PASS"}else{"FAIL"},"checks":checks,"scope":"Actual native API, checkpoint import and independent sum over supplied cases; no HTTP/browser or general-language qualification."}),
        )?,
    )?;
    if !passed {
        return Err("historical API check failed; see report".into());
    }
    Ok(())
}
