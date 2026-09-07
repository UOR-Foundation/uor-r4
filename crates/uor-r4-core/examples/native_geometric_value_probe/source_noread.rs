//! Bounded continuation of the existing source selector; authored data stays offline.
use super::*;
use uor_r4_core::native_geometric::{
    RelationExample, RoutingMode, SourceRoutingConfig, TypedRoutingExample,
};

fn contrasts(prefix: &str, worlds: &[(&str, &str, &str, i64, i64)]) -> Vec<ValueExample> {
    let mut docs = Vec::new();
    for (world, &(name, other, city, a, b)) in worlds.iter().enumerate() {
        for reverse in [false, true] {
            for supported in [false, true] {
                let first = format!("{name} has {a} coins.");
                let second = if supported {
                    format!("{name} lives in {city}.")
                } else {
                    format!("{other} has {b} coins.")
                };
                let facts = if reverse {
                    format!("{second} {first}")
                } else {
                    format!("{first} {second}")
                };
                for wording in 0..2 {
                    let question = if wording == 0 {
                        format!("Where is {name}?")
                    } else {
                        format!("Where is the location of {name}?")
                    };
                    docs.push(ValueExample {
                        id: format!("{prefix}/{world}/{reverse}/{supported}/{wording}"),
                        prompt: format!("User: {facts}\nUser: {question}\nAssistant:"),
                        response: if supported {
                            format!(" {city}.\n")
                        } else {
                            " Unknown.\n".into()
                        },
                    });
                }
            }
            // A positive answer equal to the observed false copy prevents treating
            // the word coins itself as universally unsupported.
            let facts = if reverse {
                format!("{other} has stones. {name} has coins.")
            } else {
                format!("{name} has coins. {other} has stones.")
            };
            docs.push(ValueExample {
                id: format!("{prefix}/{world}/{reverse}/supported-object"),
                prompt: format!("User: {facts}\nUser: What does {name} have?\nAssistant:"),
                response: " coins.\n".into(),
            });
        }
    }
    docs
}
pub(super) fn responses(
    model: &Model,
    docs: &[ValueExample],
    trace: bool,
    control: Control,
) -> ProbeResult<Value> {
    let mut rows = Vec::new();
    for d in docs {
        let generation = model.generate(&d.prompt, 32, control)?;
        let decision = if trace && control == Control::Full {
            Some(model.source_routing_trace(&d.prompt)?)
        } else {
            None
        };
        rows.push(json!({"id":d.id,"prompt":d.prompt,"expected":d.response,"exact":generation.text==d.response && generation.stop=="end_of_document","generation":generation,"source_trace":decision}));
    }
    Ok(
        json!({"artifact":model.artifact_cid(),"exact":rows.iter().filter(|r|r["exact"]==true).count(),"total":rows.len(),"cases":rows}),
    )
}
fn relation_responses(model: &Model, docs: &[RelationExample]) -> ProbeResult<Value> {
    let mut report = writer_binding::evaluate(model, docs)?;
    let rows = report["cases"]
        .as_array_mut()
        .ok_or("relation report cases absent")?;
    for row in rows.iter_mut() {
        row["exact"] =
            json!(row["exact"] == true && row["generation"]["stop"] == "end_of_document");
    }
    let exact = rows.iter().filter(|row| row["exact"] == true).count();
    report["exact"] = json!(exact);
    report["exact_scope"] = json!("Complete bytes and EOS; writes counted separately.");
    Ok(report)
}
fn save(out: &Path, name: &str, report: &Value) -> ProbeResult<()> {
    write_json(&out.join(format!("{name}.json")), report)?;
    println!(
        "{}",
        json!({"split":name,"exact":report["exact"],"total":report["total"]})
    );
    Ok(())
}
pub(super) fn run(args: &[String]) -> ProbeResult<()> {
    match args.first().map(String::as_str) {
        Some("prepare") if args.len()==4 => {
            let prior:Value=serde_json::from_slice(&fs::read(&args[1])?)?;
            let literal:Value=serde_json::from_slice(&fs::read(&args[2])?)?;
            let mut fit:Vec<ValueExample>=serde_json::from_value(prior["fit"].clone())?;
            let inherited=fit.len();
            let literal_fit:Vec<TypedRoutingExample>=serde_json::from_value(literal["fit"].clone())?;
            fit.extend(literal_fit.into_iter().filter(|d|d.literal_only).map(|d|ValueExample{id:format!("source-joint/{}",d.id),prompt:d.query,response:d.response}));
            let added=contrasts("source-joint-fit",&[("ada","ben","Rome",8,7),("cyra","dara","Paris",13,4)]);
            fit.extend(added);
            let mut development=contrasts("source-joint-open",&[("leni","varo","Dover",17,-5)]);
            let exposed:Vec<TypedRoutingExample>=serde_json::from_value(literal["exposed_answers"].clone())?;
            development.extend(exposed.into_iter().filter(|d|d.action.is_none()).map(|d|ValueExample{id:format!("observed-failure/{}",d.id),prompt:d.query,response:d.response}));
            let first_use=contrasts("source-joint-reserved",&[("selvi","dovra","Norvik",-9,26),("pavren","milso","Kestra",32,-11)]);
            let encoded=serde_json::to_string(&fit)?;
            for name in ["selvi","dovra","Norvik","pavren","milso","Kestra"] {if encoded.contains(name){return Err("new source names overlap fit".into());}}
            write_json(Path::new(&args[3]),&json!({"schema":"uor-r4.source-noread-contrasts/1","scope":"Existing source and literal construction plus paired supported/missing location and positive coins answers. Exact names/order/literal distractors vary. Fresh names/places/numbers opened after design selection; familiar wording is not general-language qualification.","inherited_source_examples":inherited,"fit":fit,"development":development,"first_use":first_use}))?;
        }
        Some("fit") if args.len()==6 => {
            let model=Model::from_bytes(&fs::read(&args[1])?)?;
            let source:Value=serde_json::from_slice(&fs::read(&args[2])?)?;
            let docs:Vec<ValueExample>=serde_json::from_value(source["fit"].clone())?;
            let mode=match args[5].as_str(){"angular"=>RoutingMode::Angular,"equality"=>RoutingMode::Equality,_=>return Err("source mode".into())};
            let config=SourceRoutingConfig{learned_features:args[4].parse()?,passes:4,proposals:12,max_seconds:45,mode,role_context_only:true,..SourceRoutingConfig::default()};
            let (candidate,report)=model.refine_source_routing(&docs,config)?;
            let out=Path::new(&args[3]);fs::create_dir(out)?;
            write_new(&out.join("model.json"),&candidate.to_bytes()?)?;
            save(out,"fit",&report)?;
            save(out,"construction",&responses(&candidate,&docs,false,Control::Full)?)?;
            let open:Vec<ValueExample>=serde_json::from_value(source["development"].clone())?;
            save(out,"development",&responses(&candidate,&open,true,Control::Full)?)?;
            println!("{}",report);
        }
        Some("evaluate") if args.len()==6 => {
            let model=Model::from_bytes(&fs::read(&args[1])?)?;
            let source:Value=serde_json::from_slice(&fs::read(&args[2])?)?;
            let docs:Vec<ValueExample>=serde_json::from_value(source[&args[3]].clone())?;
            let control=match args[5].as_str(){"full"=>Control::Full,"codes-disabled"=>Control::LearnedRoutingTransformDisabled,_=>return Err("source control".into())};
            let report=responses(&model,&docs,true,control)?;
            write_json(Path::new(&args[4]),&report)?;
            println!("{}",json!({"exact":report["exact"],"total":report["total"]}));
        }
        Some("preserve") if args.len()==4 => {
            let model=Model::from_bytes(&fs::read(&args[1])?)?;
            let root=Path::new(&args[2]);let out=Path::new(&args[3]);fs::create_dir(out)?;
            for (file,split,label) in [
                ("answer-entry-source.json","fit","literal-construction"),
                ("answer-entry-source.json","first_use","prior-fresh"),
                ("literal-admission-source.json","first_use","exposed-literals"),
                ("literal-admission-source.json","transfer","chains"),
                ("literal-admission-source.json","identifiers","identifiers"),
                ("independent-reachable-source.json","first_use","prior-chains"),
                ("independent-reachable-source.json","fit","computed-construction"),
                ("typed-routing-case-source.json","first_use","numeric"),
                ("typed-roles-query-source.json","first_use","roles"),
                ("typed-roles-query-source.json","exposed_first_use","exposed-roles"),
                ("typed-roles-query-source.json","exposed_alias_first_use","aliases")
            ] {
                let source:Value=serde_json::from_slice(&fs::read(root.join(file))?)?;
                let docs:Vec<TypedRoutingExample>=serde_json::from_value(source[split].clone())?;
                save(out,label,&model.evaluate_typed_routing(&docs,false)?)?;
            }
            let source:Value=serde_json::from_slice(&fs::read(root.join("writer-binding-continuation-source.json"))?)?;
            for (split,label) in [("development","dependent"),("fresh","names")] {
                let docs:Vec<RelationExample>=serde_json::from_value(source[split].clone())?;
                save(out,label,&relation_responses(&model,&docs)?)?;
            }
            for split in ["preservation","prior"] {
                let docs:Vec<ValueExample>=serde_json::from_value(source[split].clone())?;
                save(out,split,&responses(&model,&docs,false,Control::Full)?)?;
            }
            let source:Value=serde_json::from_slice(&fs::read(root.join("relation-admission-source.json"))?)?;
            let docs:Vec<RelationExample>=serde_json::from_value(source["first_use"].clone())?;
            save(out,"long",&relation_responses(&model,&docs)?)?;
            relation_memory::verify_model(model,&out.join("sessions.json"))?;
        }
        _=>return Err("source-noread prepare PRIOR LITERAL OUTPUT | fit MODEL SOURCE NEW_DIR FEATURES angular|equality | evaluate MODEL SOURCE SPLIT REPORT full|codes-disabled | preserve MODEL ROOT NEW_DIR".into())
    }
    Ok(())
}
