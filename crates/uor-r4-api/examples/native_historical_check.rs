//! Actual selected-artifact API generation; authored expectations are assertions only.
use serde_json::{json, Value};
use std::{error::Error, fs};
use uor_r4_api::native_capability_api::{CompletionRequest, NativeModel, SessionConfig};
fn main() -> Result<(), Box<dyn Error>> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() != 4 {
        return Err("usage: native_historical_check MODEL CASES REPORT".into());
    }
    // The report file is reserved exclusively before the model is loaded or anything
    // is generated: an existing report fails here and keeps its bytes.
    let mut report = fs::File::create_new(&a[3]).map_err(|e| {
        format!(
            "report file {} was not created exclusively ({e}); choose a new attempt path instead of reusing or overwriting an existing report",
            a[3]
        )
    })?;
    let bytes = fs::read(&a[1])?;
    let model = NativeModel::load_from_bytes(&bytes)?;
    let cases: Vec<Value> = serde_json::from_slice(&fs::read(&a[2])?)?;
    let mut checks = Vec::new();
    for c in cases.iter().filter(|c| !c["current_record"].is_null()) {
        let prompt = c["prompt"].as_str().ok_or("prompt absent")?;
        let expected = c["expected"].as_str().ok_or("expectation absent")?;
        // Optional authored alternates (declared in the case file, never from predictions).
        let accepted: Vec<&str> = c["accepted"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_else(|| vec![expected]);
        let mut session = model.create_session(SessionConfig::default())?;
        // Prior turns of a follow-up case are completed first through the same API
        // session; their responses are recorded but only the labeled prompt is judged.
        let mut prior_responses = Vec::new();
        for prior in c["history"].as_array().into_iter().flatten() {
            let prior = prior.as_str().ok_or("history prompt absent")?;
            let r = session.complete(CompletionRequest::new(prior))?;
            prior_responses.push(json!({"prompt":prior,"text":r.text,"stopped_by":r.stopped_by}));
        }
        let response = session.complete(CompletionRequest::new(prompt))?;
        let passed = accepted.contains(&response.text.as_str()) && response.stopped_by == "eos";
        checks.push(json!({"id":c["id"],"prompt":prompt,"history":prior_responses,"expected":expected,"accepted":accepted,"response":response,"passed":passed}));
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
    std::io::Write::write_all(
        &mut report,
        &serde_json::to_vec(
            &json!({"status":if passed{"PASS"}else{"FAIL"},"checks":checks,"scope":"Actual native API, checkpoint import and independent sum over supplied cases; no HTTP/browser or general-language qualification."}),
        )?,
    )?;
    if !passed {
        return Err("historical API check failed; see report".into());
    }
    Ok(())
}
