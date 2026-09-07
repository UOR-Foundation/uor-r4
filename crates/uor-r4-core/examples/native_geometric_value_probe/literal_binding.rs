//! Continue the existing literal router with paired entity/fact-order examples.
use super::*;
use uor_r4_core::native_geometric::{RoutingMode, SourceRoutingConfig};
fn contrasts(prefix: &str, worlds: &[(&str, &str, &str, i64, i64)]) -> Vec<ValueExample> {
    let mut out = Vec::new();
    for (i, &(a, b, city, x, y)) in worlds.iter().enumerate() {
        for reverse in [false, true] {
            for place_first in [false, true] {
                let one = format!("{a} has {x} coins.");
                let two = format!("{b} has {y} coins.");
                let nums = if reverse {
                    format!("{two} {one}")
                } else {
                    format!("{one} {two}")
                };
                let place = format!("{a} lives in {city}.");
                let facts = if place_first {
                    format!("{place} {nums}")
                } else {
                    format!("{nums} {place}")
                };
                for (q, name, value) in [(0, a, x), (1, b, y)] {
                    out.push(ValueExample {
                        id: format!("{prefix}/{i}/{reverse}/{place_first}/{q}"),
                        prompt: format!(
                            "User: {facts}\nUser: How many coins does {name} have?\nAssistant:"
                        ),
                        response: format!("{value}.\n"),
                    });
                }
            }
        }
    }
    out
}
pub(super) fn run(args: &[String]) -> ProbeResult<()> {
    match args.first().map(String::as_str) {
        Some("prepare") if args.len()==3 => {
            let old:Value=serde_json::from_slice(&fs::read(&args[1])?)?;
            let mut fit:Vec<ValueExample>=serde_json::from_value(old["fit"].clone())?;
            fit.extend(contrasts("binding-fit",&[("adara","cyris","Rome",13,7),("hesta","vurin","Paris",-6,23)]));
            let development=contrasts("binding-open",&[("nelva","sorvi","Dover",17,-5)]);
            let first_use=contrasts("binding-fresh",&[("morva","seldri","Kestral",31,-9),("telvi","dorna","Ostara",-27,46)]);
            let seen=serde_json::to_string(&fit)?;
            for name in ["morva","seldri","Kestral","telvi","dorna","Ostara"] {if seen.contains(name){return Err("fresh identity overlaps fit".into());}}
            write_json(Path::new(&args[2]),&json!({"schema":"uor-r4.literal-binding-data/1","fit":fit,"development":development,"first_use":first_use,"scope":"615 prior construction plus16 distinct-value named-operand/order contrasts; fixed question family, new identities only after selection. Prior joint-admission first-use excluded from fitting."}))?;
        }
        Some("fit") if args.len()==6 => {
            let model=Model::from_bytes(&fs::read(&args[1])?)?;
            let data:Value=serde_json::from_slice(&fs::read(&args[2])?)?;
            let fit:Vec<ValueExample>=serde_json::from_value(data["fit"].clone())?;
            let mode=match args[5].as_str(){"angular"=>RoutingMode::Angular,"equality"=>RoutingMode::Equality,_=>return Err("mode".into())};
            let config=SourceRoutingConfig{learned_features:args[4].parse()?,passes:4,proposals:12,max_seconds:45,mode,..Default::default()};
            let(candidate,report)=model.refine_literal_routing(&fit,config)?;
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
            let control=match args[5].as_str(){"full"=>Control::Full,"disabled"=>Control::LiteralRefinementDisabled,_=>return Err("control".into())};
            let report=source_noread::responses(&model,&docs,false,control)?;
            write_json(Path::new(&args[4]),&report)?;
            println!("{}",json!({"exact":report["exact"],"total":report["total"]}));
        }
        _=>return Err("literal-binding prepare PRIOR OUTPUT | fit MODEL SOURCE NEW_DIR FEATURES angular|equality | evaluate MODEL SOURCE SPLIT REPORT full|disabled".into())
    }
    Ok(())
}
