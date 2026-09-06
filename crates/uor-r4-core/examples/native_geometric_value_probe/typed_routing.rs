use super::*;
use uor_r4_core::native_geometric::{
    RoutingMode, SourceRoutingConfig, TypedRoutingExample, ValueAction,
};

fn example(
    id: String,
    a: i64,
    b: i64,
    delta: i64,
    question: &str,
    action: Option<ValueAction>,
) -> TypedRoutingExample {
    let value = if action == Some(ValueAction::Add) {
        a + b + delta
    } else {
        a + b
    };
    TypedRoutingExample {id,
        initial_prompt:format!("User: suri has {a} coins. orin has {b} coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:"),
        initial_response:format!("{}.\n",a+b),query:format!("User: There are {delta} new coins. {question}\nAssistant:"),
        response:if action.is_some(){format!("{value}.\n")}else{" Unknown.\n".into()},
        action,operands:action.map(|op|if op==ValueAction::Copy{[a+b,a+b]}else{[a+b,delta]}),
    }
}
pub(super) fn run(args: &[String]) -> ProbeResult<()> {
    match args.first().map(String::as_str) {
        Some("prepare") if args.len()==2 => {
            let phrases=[
                ("Repeat the previous total without adding the new coins.",Some(ValueAction::Copy)),
                ("Return the previous total.",Some(ValueAction::Copy)),
                ("Copy the previous total.",Some(ValueAction::Copy)),
                ("Add the new coins to the previous total.",Some(ValueAction::Add)),
                ("What is the sum of the previous total and the new coins?",Some(ValueAction::Add)),
                ("Return the previous total plus the new coins.",Some(ValueAction::Add)),
                ("Where is the location of suri?",None),
            ];
            let mut fit=Vec::new();
            for (i,(a,b,d)) in [(13,4,5),(8,7,6),(-3,8,4),(21,3,7),(10,6,2),(4,9,3)].into_iter().enumerate(){
                for (j,(q,op)) in phrases.iter().enumerate(){fit.push(example(format!("fit/{i}/{j}"),a,b,d,q,*op));}
            }
            let mut development=Vec::new();
            for (i,(a,b,d)) in [(13,4,5),(14,4,5),(-3,8,4)].into_iter().enumerate(){
                for j in [0,3]{development.push(example(format!("open/{i}/{j}"),a,b,d,phrases[j].0,phrases[j].1));}
            }
            let mut first_use=Vec::new();
            for (i,(a,b,d)) in [(31,6,4),(17,9,3),(-6,14,7)].into_iter().enumerate(){
                for (j,(q,op)) in [
                    ("Please copy the previous total.",ValueAction::Copy),
                    ("Please return the sum of the new coins and the previous total.",ValueAction::Add),
                ].into_iter().enumerate(){first_use.push(example(format!("reserved/{i}/{j}"),a,b,d,q,Some(op)));}
            }
            let source=json!({"schema":"uor-r4.typed-routing-source/1","fit":fit,"development":development,"first_use":first_use,"scope":"42 authored construction follow-ups. Six previously exposed development failures may inform design. Six predeclared operand/wording transfers are opened only after design selection; they are not broad language qualification. First responses are generated, not supplied to serving."});
            write_json(Path::new(&args[1]),&source)?;
        }
        Some("case-source") if args.len()==3 => {
            let mut source:Value=serde_json::from_slice(&fs::read(&args[1])?)?;
            source["exposed_first_use"]=source["first_use"].clone();
            let mut reserved=Vec::new();
            for (i,(a,b,d)) in [(19,12,5),(23,8,6),(-4,15,3)].into_iter().enumerate(){
                for (j,(q,op)) in [
                    ("Please repeat the previous total without adding the new coins.",ValueAction::Copy),
                    ("Please add the new coins to the previous total.",ValueAction::Add),
                ].into_iter().enumerate(){reserved.push(example(format!("case-reserved/{i}/{j}"),a,b,d,q,Some(op)));}
            }
            source["first_use"]=serde_json::to_value(reserved)?;
            source["scope"]=json!("Case-fold revision: original construction and development unchanged; prior3/6 first-use failures are exposed repair checks, not held out. New six predeclared operands/wording cases remain unopened until revised design selection. No general language qualification.");
            write_json(Path::new(&args[2]),&source)?;
        }
        Some("fit") if args.len()==5=>{
            let mode=match args[4].as_str(){"angular"|"angular-folded"=>RoutingMode::Angular,"equality"|"equality-folded"=>RoutingMode::Equality,_=>return Err("typed fit mode".into())};
            let model=Model::from_bytes(&fs::read(&args[1])?)?;
            let source:Value=serde_json::from_slice(&fs::read(&args[2])?)?;
            let docs:Vec<TypedRoutingExample>=serde_json::from_value(source["fit"].clone())?;
            let (candidate,report)=model.fit_typed_routing(&docs,SourceRoutingConfig{learned_features:128,passes:4,proposals:12,max_seconds:30,mode,..SourceRoutingConfig::default()},args[4].ends_with("-folded"))?;
            let out=Path::new(&args[3]);fs::create_dir(out)?;
            write_new(&out.join("model.json"),&candidate.to_bytes()?)?;
            write_json(&out.join("fit.json"),&report)?;println!("{report}");
        }
        Some("evaluate") if args.len()==6=>{
            let model=Model::from_bytes(&fs::read(&args[1])?)?;
            let source:Value=serde_json::from_slice(&fs::read(&args[2])?)?;
            if !["fit","development","first_use","exposed_first_use"].contains(&args[3].as_str()){return Err("typed evaluation split".into());}
            let docs:Vec<TypedRoutingExample>=serde_json::from_value(source[&args[3]].clone())?;
            let remove=match args[5].as_str(){"full"=>false,"remove-intermediate"=>true,_=>return Err("typed control".into())};
            let report=model.evaluate_typed_routing(&docs,remove)?;
            write_json(Path::new(&args[4]),&report)?;
            println!("{}",json!({"artifact":report["artifact"],"split":args[3],"control":args[5],"exact":report["exact"],"total":report["total"]}));
        }
        _=>return Err("typed-routing prepare SOURCE | fit MODEL SOURCE NEW_DIR angular|equality | evaluate MODEL SOURCE SPLIT NEW_REPORT full|remove-intermediate".into()),
    }
    Ok(())
}
