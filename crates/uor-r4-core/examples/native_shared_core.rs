//! One bounded, explicitly experimental shared-core learning decision.
//! The fixed small corpus is a smoke experiment, not general prose acceptance.
use std::fs;
use std::path::Path;
use uor_r4_core::native_geometric::shared_core::{FitConfig, Intervention, SharedCore};
use uor_r4_core::report_output;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        return Err("usage: native_shared_core NEW_REPORT_DIRECTORY".into());
    }
    let out = Path::new(&args[1]);
    report_output::claim(out)?;
    let result = run(out);
    if let Err(error) = &result {
        write(
            out,
            "failure.json",
            &serde_json::json!({"error":error.to_string()}),
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}

fn write(
    out: &Path,
    name: &str,
    value: &impl serde::Serialize,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    fs::File::create_new(out.join(name))?.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn run(out: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let train = [
        "The small boat crossed the lake. The water was calm.\n",
        "A red bird rested on a branch. Then the bird flew away.\n",
        "The garden was dry. Rain made the soil wet.\n",
        "Ada lives in Lima. Ada now lives in Oslo. Ada lives in Oslo.\n",
        "Ben owns a blue cup. Cora owns a red book. Ben owns the cup.\n",
        "User: Hello.\nAssistant: Hello. How are you?\nUser: Well.\n",
        "User: Where is the book?\nAssistant: The book is on the table.\n",
        "fn add(a: i32, b: i32) -> i32 { a + b }\n",
        "fn first() -> i32 { 1 }\nfn second() -> i32 { 2 }\n",
        "let x = 2;\nlet y = x + 3;\nassert_eq!(y, 5);\n",
        "The cup is blue. The book is red. The small book is on the table.\n",
        "User: Tell me about the lake.\nAssistant: The lake is calm and blue.\n",
    ];
    let holdout = [
        "The small bird crossed the garden. Then rain made the branch wet.\n",
        "Cora lives in Lima. Cora now lives in Oslo. The red book is in Oslo.\n",
        "User: Where is the blue cup?\nAssistant: The cup is on the table.\n",
        "fn third(a: i32) -> i32 { a + 1 }\nlet z = 3;\n",
    ];
    let prompts = [
        "The small bird ",
        "Cora now lives in Oslo. Cora lives in ",
        "User: Hello.\nAssistant:",
        "fn third(a: i32) -> i32 { ",
    ];
    // All data, configuration and criteria are written before model construction.
    // Holdout is never passed to fit or used to select another candidate.
    let config = FitConfig {
        seed: 211,
        max_proposals: 100000,
        max_seconds: 120,
    };
    write(
        out,
        "design.json",
        &serde_json::json!({
            "schema":"uor-r4.shared-core-experiment/1","initial_seed":7341,"fit":config,
            "training":train,"holdout":holdout,"prompts":prompts,"output_cap":96,
            "criterion":{"holdout_nll_fraction":0.98,"holdout_accuracy_not_lower":true,
                "context_disabled_nll_fraction":1.01,"state_disabled_nll_fraction":1.01},
            "scope":"One mixed authored byte-corpus feasibility run; held-out combinations use familiar words. No broad language, memory or coding acceptance. No candidate retry."
        }),
    )?;
    let training: Vec<Vec<u8>> = train.iter().map(|s| s.as_bytes().to_vec()).collect();
    let testing: Vec<Vec<u8>> = holdout.iter().map(|s| s.as_bytes().to_vec()).collect();
    let initial = SharedCore::initialized(7341)?;
    fs::write(out.join("initial.json"), initial.to_bytes()?)?;
    let (candidate, fit) = initial.fit(&training, config)?;
    write(out, "fit.json", &fit)?;
    fs::write(out.join("candidate.json"), candidate.to_bytes()?)?;
    let baseline = initial.evaluate(&testing, Intervention::Full)?;
    let full = candidate.evaluate(&testing, Intervention::Full)?;
    let no_context = candidate.evaluate(&testing, Intervention::ContextDisabled)?;
    let no_state = candidate.evaluate(&testing, Intervention::StateDisabled)?;
    let no_zeta = candidate.evaluate(&testing, Intervention::ZetaDisabled)?;
    let no_transport = candidate.evaluate(&testing, Intervention::TransportDisabled)?;
    let gate = full.mean_nll < baseline.mean_nll * 0.98
        && full.correct >= baseline.correct
        && no_context.mean_nll > full.mean_nll * 1.01
        && no_state.mean_nll > full.mean_nll * 1.01;
    let mut generations = Vec::new();
    for (label, model) in [("initial", &initial), ("candidate", &candidate)] {
        for prompt in prompts {
            for control in [
                Intervention::Full,
                Intervention::ContextDisabled,
                Intervention::StateDisabled,
            ] {
                let (bytes, eos, work) = model.generate(prompt.as_bytes(), 96, control)?;
                generations.push(serde_json::json!({"model":label,"prompt":prompt,"control":control,
                    "bytes":bytes,"utf8_lossy":String::from_utf8_lossy(&bytes),"eos":eos,"work":work}));
            }
        }
    }
    write(out, "generation.json", &generations)?;
    let report = serde_json::json!({"decision":if gate {"PASS_PREDICTIVE_SMOKE_ONLY"} else {"FAIL_CONTEXTUAL_TRANSFER_SMOKE"},
        "promoted":false,"artifact":candidate.artifact_cid(),"parent":initial.artifact_cid(),
        "initial_holdout":baseline,"full":full,"context_disabled":no_context,
        "state_disabled":no_state,"zeta_disabled":no_zeta,"transport_disabled":no_transport,
        "language_qualification":"NOT_ESTABLISHED","retained_model_comparison":"NOT_RUN",
        "scope":"Offline likelihood uses the runtime's discrete states/branch scores. Evaluation work counts include both predicted and target branch walks, not serving latency."});
    write(out, "result.json", &report)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
