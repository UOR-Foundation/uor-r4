//! Existing driver extension for literal-versus-language admission.
use super::*;
use uor_r4_core::native_geometric::{RoutingMode, SourceRoutingConfig};

fn contrasts(prefix: &str, worlds: &[(&str, &str, i64, i64)]) -> Vec<ValueExample> {
    let mut out = Vec::new();
    for (i, &(name, city, a, b)) in worlds.iter().enumerate() {
        for reverse in [false, true] {
            let place = format!("{name} lives in {city}.");
            let number = format!("{name} has {a} coins. other has {b} coins.");
            let facts = if reverse {
                format!("{place} {number}")
            } else {
                format!("{number} {place}")
            };
            for (q, question, response) in [
                (0, format!("Where is {name}?"), format!(" {city}.\n")),
                (
                    1,
                    format!("Where is the location of {name}?"),
                    format!(" {city}.\n"),
                ),
                (
                    2,
                    format!("How many coins does {name} have?"),
                    format!("{a}.\n"),
                ),
            ] {
                out.push(ValueExample {
                    id: format!("{prefix}/{i}/{reverse}/{q}"),
                    prompt: format!("User: {facts}\nUser: {question}\nAssistant:"),
                    response,
                });
            }
        }
    }
    out
}
pub(super) fn run(args: &[String]) -> ProbeResult<()> {
    match args.first().map(String::as_str) {
        Some("prepare") if args.len()==3 => {
            let prior:Value=serde_json::from_slice(&fs::read(&args[1])?)?;
            let mut fit:Vec<ValueExample>=serde_json::from_value(prior["fit"].clone())?;
            fit.extend(contrasts("admission-fit",&[("ada","Rome",13,7),("cyra","Paris",8,4)]));
            let development=contrasts("admission-open",&[("leni","Dover",17,5)]);
            let first_use=contrasts("admission-fresh",&[("velri","Torsen",-19,26),("navel","Mistra",37,-8)]);
            let encoded=serde_json::to_string(&fit)?;
            for word in ["velri","Torsen","navel","Mistra"] {if encoded.contains(word) {return Err("admission first-use identity overlaps construction".into());}}
            write_json(Path::new(&args[2]),&json!({"schema":"uor-r4.joint-admission-data/1","fit":fit,"development":development,"first_use":first_use,"scope":"Existing 603 construction cases plus 12 paired numeric/location prompts. Separate changed-name/value/place/order first-use after design selection; familiar wording, not general-language held-out qualification."}))?;
        }
        Some("fit") if args.len()==6 => {
            let model=Model::from_bytes(&fs::read(&args[1])?)?;
            let data:Value=serde_json::from_slice(&fs::read(&args[2])?)?;
            let fit:Vec<ValueExample>=serde_json::from_value(data["fit"].clone())?;
            let mode=match args[5].as_str(){"angular"=>RoutingMode::Angular,"equality"=>RoutingMode::Equality,_=>return Err("admission mode".into())};
            let config=SourceRoutingConfig{learned_features:args[4].parse()?,passes:4,proposals:12,max_seconds:45,mode,..SourceRoutingConfig::default()};
            let(candidate,report)=model.fit_joint_admission(&fit,config)?;
            let out=Path::new(&args[3]);fs::create_dir(out)?;
            write_new(&out.join("model.json"),&candidate.to_bytes()?)?;
            write_json(&out.join("fit.json"),&report)?;
            write_json(&out.join("construction.json"),&source_noread::responses(&candidate,&fit,false,Control::Full)?)?;
            let open:Vec<ValueExample>=serde_json::from_value(data["development"].clone())?;
            write_json(&out.join("development.json"),&source_noread::responses(&candidate,&open,false,Control::Full)?)?;
            println!("{}",report);
        }
        Some("evaluate") if args.len()==6 => {
            let model=Model::from_bytes(&fs::read(&args[1])?)?;
            let data:Value=serde_json::from_slice(&fs::read(&args[2])?)?;
            let docs:Vec<ValueExample>=serde_json::from_value(data[&args[3]].clone())?;
            let control=match args[5].as_str(){"full"=>Control::Full,"admission-disabled"=>Control::JointAdmissionDisabled,_=>return Err("admission control".into())};
            let report=source_noread::responses(&model,&docs,false,control)?;
            write_json(Path::new(&args[4]),&report)?;
            println!("{}",json!({"exact":report["exact"],"total":report["total"]}));
        }
        _=>return Err("joint-admission prepare PRIOR OUTPUT | fit MODEL SOURCE NEW_DIR FEATURES angular|equality | evaluate MODEL SOURCE SPLIT REPORT full|admission-disabled".into()),
    }
    Ok(())
}
