//! Source-bound strict event/age admission on retained parents; zero updates.
//! A rejected fixed grid is a construction decision, not a geometry verdict.
use candle_core::Device;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs, path::PathBuf, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::{
    geometric_event::{self, CompiledEvents, EventWeights},
    geometric_potential_native::{CompiledGeometricPotentials, PotentialSourceBinding},
    geometric_read_native::{CompiledGeometricRead, ReadSourceBinding},
    geometric_stack::StackModel,
    sha256_bytes, Result, TrainingError,
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    parent_arguments: PathBuf,
    context_attempt: PathBuf,
    out: PathBuf,
    maximum_seconds: u64,
}
fn invalid(s: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(s.into())
}
fn path(a: &Value, key: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(a[key].as_str().ok_or_else(|| {
        invalid(format!("missing parent path:{key}"))
    })?))
}
fn write(p: PathBuf, v: &impl serde::Serialize) -> Result<()> {
    fs::write(p, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn trace_record(t: &geometric_event::NativeEventTrace) -> Value {
    json!({"batch":t.batch,"time":t.time,"lanes":t.lanes,
        "actions":t.actions.iter().map(|a|*a as u8).collect::<Vec<_>>(),
        "transition_actions":t.transition_actions,"states":t.states,
        "event_scores":t.event_scores,"coefficient_reads":t.coefficient_reads})
}
fn main() -> Result<()> {
    let mut cli = std::env::args().skip(1);
    let input = cli
        .next()
        .ok_or_else(|| invalid("usage: attention-geometric-event-age-q4 ARGS.json"))?;
    if cli.next().is_some() {
        return Err(invalid("one arguments file required"));
    }
    let bytes = fs::read(input)?;
    let a: Args = serde_json::from_slice(&bytes)?;
    if !(60..=840).contains(&a.maximum_seconds) {
        return Err(invalid("explicit60..840 second bound required"));
    }
    report_output::claim(&a.out)?;
    let at = Instant::now();
    fs::write(a.out.join("arguments.json"), bytes)?;
    let result = run(&a, at);
    if let Err(e) = &result {
        write(
            a.out.join("failure.json"),
            &json!({"error":e.to_string(),"elapsed_seconds":at.elapsed().as_secs_f64(),"decision":"INCOMPLETE_NO_QUALITY_VERDICT"}),
        )?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
fn run(a: &Args, at: Instant) -> Result<()> {
    report_output::verify(&a.context_attempt)?;
    let parent_bytes = fs::read(&a.parent_arguments)?;
    if parent_bytes != fs::read(a.context_attempt.join("arguments.json"))? {
        return Err(invalid(
            "parent argument bytes differ from sealed context attempt",
        ));
    }
    let parent: Value = serde_json::from_slice(&parent_bytes)?;
    let parent_report_bytes = fs::read(a.context_attempt.join("report.json"))?;
    let parent_report: Value = serde_json::from_slice(&parent_report_bytes)?;
    if parent_report["complete"] != true || parent_report["optimizer_updates"] != 0 {
        return Err(invalid("complete zero-update parent required"));
    }
    let model_path = path(&parent, "model")?;
    let event_source = path(&parent, "event_source")?;
    let event_native = path(&parent, "event_native")?;
    let span_native = path(&parent, "span_native")?;
    let potential_parent = path(&parent, "potential_parent")?;
    report_output::verify(&potential_parent)?;
    let model = StackModel::load(&model_path, &Device::Cpu)?;
    if model.config.vocab_size != 40
        || model.config.heads != 2
        || model.config.width != 32
        || model.config.context != 128
    {
        return Err(invalid(
            "retained V40 H2 width32 context128 source required",
        ));
    }
    let tokenizer = fs::read(span_native.join("tokenizer-identity.bin"))?;
    let old_events = CompiledEvents::load(&event_native, &event_source, &model_path, &tokenizer)?;
    let raw_events = EventWeights::load_source(&event_source, &model_path, &tokenizer)?;
    let event_result = raw_events.into_q4();
    let (event_weights, events, event_admission) = match event_result {
        Ok(w) => {
            let source = a.out.join("event-q4-source");
            let native = a.out.join("event-q4-native");
            w.save_source(&source, &model_path, &tokenizer)?;
            let w = EventWeights::load_source(&source, &model_path, &tokenizer)?;
            CompiledEvents::compile(&w, &source, &model_path, &tokenizer)?.save(&native)?;
            let e = CompiledEvents::load(&native, &source, &model_path, &tokenizer)?;
            e.validate_q4_training(&w)?;
            let record = json!({"admitted":true,"metadata":e.metadata(),"packed_sha256":sha256_bytes(&w.packed_coefficients()?)});
            (Some(w), Some(e), record)
        }
        Err(e) => (
            None,
            None,
            json!({"admitted":false,"refusal":e.to_string(),"decision":"FIXED_SOURCE_GRID_NOT_ADMITTED_NO_CLIPPING"}),
        ),
    };
    let potential = CompiledGeometricPotentials::load(
        &potential_parent.join("potential-q4-native"),
        &PotentialSourceBinding::from_directory(&model_path, &tokenizer)?,
    )?;
    let binding = ReadSourceBinding::from_directory(&model_path, &tokenizer, &potential, 2)?;
    let old_read =
        CompiledGeometricRead::load(&a.context_attempt.join("rebound-native-read"), &binding)?;
    let age = model
        .variables()
        .get("layers.02.read.age")
        .ok_or_else(|| invalid("saved age missing"))?
        .as_tensor();
    let age_admission = match CompiledGeometricRead::compile_q4_age(age, &potential, &binding) {
        Ok(read) => {
            let native = a.out.join("age-q4-native");
            read.save(&native)?;
            let read = CompiledGeometricRead::load(&native, &binding)?;
            if old_read.exp_q31() != read.exp_q31() {
                return Err(invalid("strict age changed exp table"));
            }
            json!({"admitted":true,"metadata":read.metadata(),"exp_table_unchanged":true})
        }
        Err(e) => {
            json!({"admitted":false,"refusal":e.to_string(),"decision":"FIXED_SOURCE_GRID_NOT_ADMITTED_NO_CLIPPING"})
        }
    };
    write(
        a.out.join("boundary-admission.json"),
        &json!({"event":event_admission,"age":age_admission}),
    )?;
    let mut panels = Vec::new();
    if let (Some(weights), Some(events)) = (&event_weights, &events) {
        for panel in ["original", "stress"] {
            let input = fs::read(a.context_attempt.join(format!("{panel}-episodes.json")))?;
            let episodes: Vec<Value> = serde_json::from_slice(&input)?;
            if episodes.len() != 128 {
                return Err(invalid("128 retained rows required"));
            }
            let mut rows = Vec::new();
            let mut changed = 0usize;
            for (index, episode) in episodes.iter().enumerate() {
                if at.elapsed().as_secs() >= a.maximum_seconds {
                    return Err(invalid("configured admission comparison limit reached"));
                }
                let ids: Vec<u32> = serde_json::from_value(episode["ids"].clone())?;
                let source = weights.forward_q4(&ids, 1, ids.len(), false)?;
                let native = geometric_event::trace_native(&ids, 1, ids.len(), events, false)?;
                let old = geometric_event::trace_native(&ids, 1, ids.len(), &old_events, false)?;
                if source.trace != native {
                    return Err(invalid("strict event source/native trace differs"));
                }
                let changes = old
                    .actions
                    .iter()
                    .zip(&native.actions)
                    .filter(|(a, b)| a != b)
                    .count();
                changed += changes;
                rows.push(json!({"index":index,"episode":episode,"old_trace":trace_record(&old),"q4_trace":trace_record(&native),"changed_actions":changes,"source_native_exact":true}));
            }
            write(a.out.join(format!("{panel}-event-rows.json")), &rows)?;
            panels.push(json!({"panel":panel,"rows":rows.len(),"input_sha256":sha256_bytes(&input),"changed_actions":changed}));
        }
    }
    write(
        a.out.join("report.json"),
        &json!({
            "schema":"uor-r4.geometric-event-age-q4-admission/1","complete":true,"optimizer_updates":0,
            "parent_arguments_sha256":sha256_bytes(&parent_bytes),"parent_report_sha256":sha256_bytes(&parent_report_bytes),
            "event":event_admission,"age":age_admission,"event_panels":panels,
            "elapsed_seconds":at.elapsed().as_secs_f64(),
            "decision":if event_admission["admitted"]==true && age_admission["admitted"]==true {"BOUNDARIES_ADMITTED_INTEGRATED_REBIND_REQUIRED"} else {"PRESERVE_FIXED_GRID_REFUSAL_REQUIRES_CAUSAL_LEARNING_DESIGN"},
            "scope":"Actual source/native event fidelity and explicit fixed-grid age admission; no integrated new reader, answer improvement, optimizer fit, full serving or geometric advantage claim"
        }),
    )?;
    Ok(())
}
