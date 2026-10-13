use uor_r4_discrete::{Error, Result};
fn invalid(s: impl Into<String>) -> Error {
    Error::Invalid(s.into())
}
#[cfg(feature = "cuda")]
fn device_command(args: &[String]) -> Result<()> {
    use std::path::Path;
    use uor_r4_discrete::{
        device::{Device, Snapshot},
        Identity, Parameter, Spec,
    };
    let read = |p: &str| std::fs::read(p).map_err(|e| invalid(e.to_string()));
    match args.first().map(String::as_str) {
        Some("qualify") if args.len()==2=>uor_r4_discrete::qualification::run(Path::new(&args[1])),
        Some("init") if args.len()==3=>{
            let spec:Spec=serde_json::from_slice(&read(&args[1])?).map_err(|e|invalid(e.to_string()))?;
            // Exclusive output is reserved before model initialization.
            let output=std::fs::OpenOptions::new().write(true).create_new(true).open(&args[2]).map_err(|e|invalid(e.to_string()))?;
            let model=Device::new(spec,7,0)?;
            serde_json::to_writer(output,&model.snapshot()?).map_err(|e|invalid(e.to_string()))
        }
        Some("infer")|Some("propose") if args.len()>=3=>{
            #[derive(serde::Deserialize)] #[serde(deny_unknown_fields)]
            struct Request{windows:Vec<Vec<Identity>>,targets:Vec<u32>}
            let snapshot:Snapshot=serde_json::from_slice(&read(&args[1])?).map_err(|e|invalid(e.to_string()))?;
            let request:Request=serde_json::from_slice(&read(&args[2])?).map_err(|e|invalid(e.to_string()))?;
            if request.windows.is_empty() || request.windows.iter().any(|w|w.len()!=snapshot.spec.context as usize){return Err(invalid("each window must be one complete, caller-bounded causal prefix"));}
            let model=Device::from_snapshot(snapshot,0)?;
            let input=model.carry(&request.windows.into_iter().flatten().collect::<Vec<_>>())?;
            if args[0]=="infer" && args.len()==3 {let result=model.forward(&input,&request.targets)?;println!("{}",serde_json::json!({"tokens":result.predictions,"loss_numerators":result.losses,"vectors":result.vectors()?,"epoch":result.epoch(),"training":"HELD"}));Ok(())}
            else if args[0]=="propose" && args.len()==5 {let index=args[4].parse().map_err(|_|invalid("parameter index must be unsigned"))?;let parameter=match args[3].as_str(){"route"=>Parameter::Route(index),"vector"=>Parameter::VectorBlock(index),_=>return Err(invalid("parameter must be route or vector"))};let p=model.propose(&input,&request.targets,parameter)?.observation()?;println!("{}",serde_json::json!({"candidate":p.candidate,"before":p.before,"after":p.after,"epoch":p.epoch,"applied":false}));Ok(())}else{Err(invalid("infer requires ARTIFACT REQUEST; propose adds route|vector INDEX"))}
        }
        _=>Err(invalid("CUDA commands: init SPEC.json NEW_ARTIFACT.json | infer ARTIFACT REQUEST | propose ARTIFACT REQUEST route|vector INDEX | qualify NEW_REPORT_DIR")),
    }
}
fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("train") => uor_r4_discrete::training_admission(),
        Some("schema") => {
            println!(
                "{}; integer CUDA only; training HELD",
                uor_r4_discrete::SCHEMA
            );
            Ok(())
        }
        Some("list-controls") => {
            #[cfg(feature = "cuda")]
            for name in uor_r4_discrete::qualification::NAMES {
                println!("{name}");
            }
            Ok(())
        }
        _ => {
            #[cfg(feature = "cuda")]
            return device_command(&args);
            #[cfg(not(feature = "cuda"))]
            Err(invalid(
                "CUDA build required for model commands; no host fallback",
            ))
        }
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(2);
    }
}
