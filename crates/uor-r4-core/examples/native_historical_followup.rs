//! Read-only actual-generation diagnostic of historical -> current turns.
//! Full and disabled branches share one observed history and input checkpoint.
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{Control, Model, Session, BOS, EOS};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn state(session: &Session) -> Result<Value> {
    Ok(serde_json::from_slice(&session.checkpoint()?)?)
}
fn metadata(s: &Value) -> Value {
    json!({"control":s["control"],"query_boundary":s["values"]["query_boundary"],"seen":s["values"]["seen"],"started_at":s["values"]["started_at"],"relations":s["values"]["relations"],
        "lexemes":s["values"]["lexemes"],"response_entry":s["response_entry"],
        "word_copy":s["word_copy"],"field_composition":s["field_composition"]})
}
fn ingest(model: &Model, session: &mut Session, prompt: &str) -> Result<()> {
    let tokens = model.encode(prompt)?;
    if tokens.len() > 512 || prompt.len() > 4096 {
        return Err("diagnostic input bound".into());
    }
    for token in tokens {
        session.observe(model, token)?;
    }
    Ok(())
}
fn generate(model: &Model, session: &mut Session) -> Result<Value> {
    session.begin_response(model)?;
    let before = state(session)?;
    let mut tokens = Vec::new();
    let mut eos = false;
    let mut first = Value::Null;
    for _ in 0..96 {
        let p = session.predict(model)?;
        if tokens.is_empty() {
            first = json!({"prediction":p,"word_copy":session.word_copy_decision(),"field":session.field_composition_decision(),"entry":session.response_entry_decision()});
        }
        session.observe(model, p.token)?;
        if p.token == EOS {
            eos = true;
            break;
        }
        tokens.push(p.token);
    }
    let after = state(session)?;
    Ok(
        json!({"text":String::from_utf8(model.decode(&tokens)?)?,"tokens":tokens,"eos":eos,
        "first_decision":first,"before":metadata(&before),"after":metadata(&after),
        "records_unchanged":before["values"]["relations"]==after["values"]["relations"]}),
    )
}
fn branch(model: &Model, checkpoint: &[u8], control: Control) -> Value {
    let result = (|| -> Result<Value> {
        let mut wire: Value = serde_json::from_slice(checkpoint)?;
        wire["control"] = serde_json::to_value(control)?;
        let mut session = model.restore_session(&serde_json::to_vec(&wire)?)?;
        generate(model, &mut session)
    })();
    match result {
        Ok(v) => json!({"status":"OBSERVED","control":control,"result":v}),
        Err(e) => {
            json!({"status":"UNAVAILABLE","control":control,"error":e.to_string(),"scope":"Control-edited checkpoint restore/generation failed; no substitute state was run."})
        }
    }
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: native_historical_followup MODEL OUTPUT_JSON".into());
    }
    let path = Path::new(&args[1]);
    if path.exists() {
        return Err("output already exists; preserve prior diagnostic".into());
    }
    let model = Model::from_bytes(&fs::read(&args[0])?)?;
    let mut rows = Vec::new();
    for (label,facts,padding,owner) in [
        ("reverse","Record: Dusk Ridge holds selvi. selvi now in Copper Vale.",String::new(),"selvi"),
        ("evicted","Record: Dusk Ridge holds selvi. selvi now in Copper Vale.","oak ash elm ".repeat(12),"selvi"),
        ("two_owners","Record: Dusk Ridge holds selvi. selvi now in Copper Vale. Silver Cove holds tilva. tilva now in Birch Grove.",String::new(),"selvi"),
        ("two_revisions","Record: Dusk Ridge holds selvi. selvi now in Copper Vale. selvi now in Amber Field.",String::new(),"selvi"),
    ] {
        for query in [format!("Where was {owner} before? Answer:"),format!("What was the previous location of {owner}? Answer:")] {
            let prompt=format!("{facts} {padding}{query}");
            let mut session=model.session(Control::Full)?;session.observe(&model,BOS)?;
            ingest(&model,&mut session,&prompt)?;
            let history=generate(&model,&mut session)?;
            let after_history=state(&session)?;
            if session.needs_input_boundary(){session.end_response(&model)?;}
            let after_boundary=state(&session)?;
            let current=format!("Where is {owner}? Answer:");
            ingest(&model,&mut session,&current)?;
            let checkpoint=session.checkpoint()?;
            let before_current:Value=serde_json::from_slice(&checkpoint)?;
            let full=branch(&model,&checkpoint,Control::Full);
            let disabled=branch(&model,&checkpoint,Control::HistoricalReadDisabled);
            rows.push(json!({"case":label,"historical_prompt":prompt,"current_prompt":current,"history":history,
                "after_history":metadata(&after_history),"after_input_boundary":metadata(&after_boundary),
                "before_current":metadata(&before_current),"checkpoint_bytes":checkpoint.len(),"full":full,"disabled":disabled}));
        }
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({"artifact":model.artifact_cid(),"rows":rows,
        "scope":"Open actual-generation diagnostic. Historical tokens are generated once under Full, never expected-answer injected. Both current branches restore the same observed-history/input checkpoint, changing only serialized control before validated restoration. No fitting or correctness qualification."}))?,
    )?;
    Ok(())
}
