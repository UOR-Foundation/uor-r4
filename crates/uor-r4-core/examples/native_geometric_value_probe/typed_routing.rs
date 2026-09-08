use super::*;
use uor_r4_core::native_geometric::{
    RoutingMode, SourceRoutingConfig, TypedRoutingExample, TypedRoutingTurn, ValueAction,
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
    TypedRoutingExample {id, current_query_literals: false, current_query_operand: false, literal_only: false, continuation: None, refresh: Vec::new(), target_intermediate: 0,
        initial_prompt:format!("User: suri has {a} coins. orin has {b} coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:"),
        initial_response:format!("{}.\n",a+b),query:format!("User: There are {delta} new coins. {question}\nAssistant:"),
        response:if action.is_some(){format!("{value}.\n")}else{" Unknown.\n".into()},
        action,operands:action.map(|op|if op==ValueAction::Copy{[a+b,a+b]}else{[a+b,delta]}),
    }
}
fn role_example(
    id: String,
    a: i64,
    b: i64,
    delta: i64,
    extra: i64,
    kind: usize,
    refresh: bool,
) -> TypedRoutingExample {
    let questions = [
        "Repeat the original total.",
        "Repeat the updated total.",
        "Copy the total before the new coins.",
        "Copy the total after the new coins.",
        "Add the extra coins to the original total.",
        "Add the extra coins to the updated total.",
        "Where is the location of suri?",
    ];
    let target = usize::from(kind == 1 || kind == 3 || kind == 5);
    let value = a + b + if target == 1 { delta } else { 0 };
    let action = if kind == 6 {
        None
    } else if kind >= 4 {
        Some(ValueAction::Add)
    } else {
        Some(ValueAction::Copy)
    };
    let mut d = example(id, a, b, delta, questions[kind], action);
    d.continuation=Some(TypedRoutingTurn {prompt:format!("User: There are {delta} new coins. Add the new coins to the previous total.\nAssistant:"),response:format!("{}.\n",a+b+delta)});
    d.target_intermediate = target;
    d.query = if kind == 4 || kind == 5 {
        format!(
            "User: There are {extra} extra coins. {}\nAssistant:",
            questions[kind]
        )
    } else {
        format!("User: {}\nAssistant:", questions[kind])
    };
    d.operands = action.map(|op| {
        if op == ValueAction::Copy {
            [value, value]
        } else {
            [value, extra]
        }
    });
    d.response = if action.is_none() {
        " Unknown.\n".into()
    } else {
        format!(
            "{}.\n",
            value
                + if action == Some(ValueAction::Add) {
                    extra
                } else {
                    0
                }
        )
    };
    if refresh {
        d.refresh.push(TypedRoutingTurn {
            prompt: "User: Repeat the original total.\nAssistant:".into(),
            response: format!("{}.\n", a + b),
        });
    }
    d
}

fn independent_example(
    id: String,
    names: [&str; 4],
    numbers: [i64; 5],
    reverse: bool,
    kind: usize,
) -> TypedRoutingExample {
    let [a, b, c, d, e] = numbers;
    let [first, second, third, fourth] = names;
    let prompt = |x: &str, y: &str, a: i64, b: i64| {
        format!("User: {x} has {a} coins. {y} has {b} coins.\nUser: What is the sum of {x}'s and {y}'s coins?\nAssistant:")
    };
    let (initial_prompt, initial_value, next_prompt, next_value) = if reverse {
        (
            prompt(third, fourth, c, d),
            c + d,
            prompt(first, second, a, b),
            a + b,
        )
    } else {
        (
            prompt(first, second, a, b),
            a + b,
            prompt(third, fourth, c, d),
            c + d,
        )
    };
    let target = kind & 1;
    let (x, y, value) = if target == 0 {
        (first, second, a + b)
    } else {
        (third, fourth, c + d)
    };
    let action = if kind < 2 {
        ValueAction::Copy
    } else {
        ValueAction::Add
    };
    TypedRoutingExample {
        id,
        current_query_literals: false,
        current_query_operand: false,
        literal_only: false,
        initial_prompt,
        initial_response: format!("{initial_value}.\n"),
        continuation: Some(TypedRoutingTurn {
            prompt: next_prompt,
            response: format!("{next_value}.\n"),
        }),
        refresh: Vec::new(),
        target_intermediate: if reverse { 1 - target } else { target },
        query: if action == ValueAction::Copy {
            format!("User: Repeat the total for {x} and {y}.\nAssistant:")
        } else {
            format!("User: There are {e} extra coins. Add the extra coins to the total for {x} and {y}.\nAssistant:")
        },
        response: format!(
            "{}.\n",
            value + if action == ValueAction::Add { e } else { 0 }
        ),
        action: Some(action),
        operands: Some(if action == ValueAction::Copy {
            [value, value]
        } else {
            [value, e]
        }),
    }
}

fn literal_example(
    id: String,
    names: [&str; 2],
    numbers: [i64; 2],
    reverse: bool,
    kind: usize,
) -> TypedRoutingExample {
    let [x, y] = names;
    let [a, b] = numbers;
    let facts = if reverse {
        format!("{y} has {b} coins. {x} has {a} coins.")
    } else {
        format!("{x} has {a} coins. {y} has {b} coins.")
    };
    let question = match kind {
        0 => format!("How many coins does {x} have now?"),
        1 => format!("How many coins does {y} have now?"),
        2 => format!("What is the sum of {x}'s and {y}'s coins?"),
        _ => format!("Where is the location of {x}?"),
    };
    let (action, operands, response) = match kind {
        0 => (Some(ValueAction::Copy), Some([a, a]), format!("{a}.\n")),
        1 => (Some(ValueAction::Copy), Some([b, b]), format!("{b}.\n")),
        2 => (
            Some(ValueAction::Add),
            Some([a, b]),
            format!("{}.\n", a + b),
        ),
        _ => (None, None, " Unknown.\n".into()),
    };
    TypedRoutingExample {
        id,
        current_query_literals: false,
        current_query_operand: false,
        literal_only: true,
        initial_prompt: String::new(),
        initial_response: String::new(),
        continuation: None,
        refresh: Vec::new(),
        target_intermediate: 0,
        query: format!("User: {facts} tavi has 301 coins.\nUser: {question}\nAssistant:"),
        response,
        action,
        operands,
    }
}

pub(super) fn run(args: &[String]) -> ProbeResult<()> {
    match args.first().map(String::as_str) {
        Some("answer-fit") if args.len() == 4 => {
            let model = Model::from_bytes(&fs::read(&args[1])?)?;
            let source: Value = serde_json::from_slice(&fs::read(&args[2])?)?;
            let docs: Vec<TypedRoutingExample> = serde_json::from_value(source["fit"].clone())?;
            let docs: Vec<ValueExample> = docs.into_iter().filter(|d| d.literal_only)
                .map(|d| ValueExample { id:d.id, prompt:d.query, response:d.response }).collect();
            let (fitted, report) = model.fit_no_read_completion(&docs, ResponseEntryFitConfig::default())?;
            let root = Path::new(&args[3]); fs::create_dir(root)?;
            write_new(&root.join("model.json"), &fitted.to_bytes()?)?;
            write_json(&root.join("fit.json"), &report)?;
            println!("{}", json!({"artifact":fitted.artifact_cid(),"report":report}));
        }
        Some("answer-source") if args.len() == 3 => {
            let mut source: Value = serde_json::from_slice(&fs::read(&args[1])?)?;
            let mut first = Vec::new();
            // New names/numbers and one new short query form, saved before fit.
            for (i,(names,numbers)) in [(["feni","mavo"],[23,-8]),(["daro","nesa"],[-13,41])].into_iter().enumerate() {
                for reverse in [false,true] { for kind in 0..4 {
                    let mut d = literal_example(format!("answer-new/{i}/{reverse}/{kind}"), names,numbers,reverse,kind);
                    if kind == 3 && i == 1 { d.query=d.query.replace("Where is the location of", "Where is"); }
                    first.push(d);
                }}
            }
            source["exposed_answers"] = source["first_use"].clone();
            source["first_use"] = serde_json::to_value(first)?;
            source["scope"] = json!("Construction unchanged:103 earlier literal/identifier examples. Only actual selected NoRead continuations fitted. Four prior failed location outputs exposed; new16 cases prepared before fit, including one shorter query form. No full held-out capability claim.");
            write_json(Path::new(&args[2]), &source)?;
        }

        Some("answer-trace") if args.len() == 4 => {
            let model = Model::from_bytes(&fs::read(&args[1])?)?;
            let source: Value = serde_json::from_slice(&fs::read(&args[2])?)?;
            let docs: Vec<TypedRoutingExample> = serde_json::from_value(source["first_use"].clone())?;
            let mut rows = Vec::new();
            for d in docs.into_iter().filter(|d| d.action.is_none()) {
                let generated = model.generate(&d.query, 48, Control::Full)?;
                rows.push(json!({"id":d.id,"prompt":d.query,"expected":d.response,"generated":generated}));
            }
            write_json(Path::new(&args[3]), &rows)?;
        }

        Some("admission-source") if args.len()==4 => {
            let prior:Value=serde_json::from_slice(&fs::read(&args[1])?)?;
            let old:Source=serde_json::from_slice(&fs::read(&args[2])?)?;
            let all:Vec<TypedRoutingExample>=serde_json::from_value(prior["fit"].clone())?;
            let mut fit:Vec<_>=all.into_iter().filter(|d|d.literal_only).collect();
            for c in old.fit.iter().filter(|c|c.id.starts_with("word-copy/fit/")) {
                fit.push(TypedRoutingExample{id:format!("literal-admission/{}",c.id),current_query_literals: false, current_query_operand: false, literal_only:true,initial_prompt:String::new(),initial_response:String::new(),continuation:None,refresh:Vec::new(),target_intermediate:0,query:c.prompt.clone(),response:c.response.clone(),action:None,operands:None});
            }
            let mut first_use=Vec::new();let mut transfer=Vec::new();let mut identifiers=Vec::new();
            for (i,(names,numbers)) in [(["leni","varo"],[17,-5]),(["rona","peli"],[-6,28])].into_iter().enumerate(){for reverse in [false,true]{for kind in 0..4{first_use.push(literal_example(format!("admission-new-literal/{i}/{reverse}/{kind}"),names,numbers,reverse,kind));}}}
            for (i,(names,numbers)) in [(["leni","varo","kesa","tanu"],[17,-5,8,3,6]),(["rona","peli","sena","jora"],[-6,28,5,-9,7])].into_iter().enumerate(){for reverse in [false,true]{for kind in 0..4{transfer.push(independent_example(format!("admission-new-chain/{i}/{reverse}/{kind}"),names,numbers,reverse,kind));}}}
            for context in 0..4 {let id=format!("word-copy/fit/context-{context}/name-0");let template=old.fit.iter().find(|c|c.id==id).ok_or("missing construction identifier template")?;
                for name in ["zorin","navi"] {identifiers.push(TypedRoutingExample{id:format!("admission-new-identifier/{context}/{name}"),current_query_literals: false, current_query_operand: false, literal_only:true,initial_prompt:String::new(),initial_response:String::new(),continuation:None,refresh:Vec::new(),target_intermediate:0,query:template.prompt.replace("item",name).replace("11","-17").replace("301","907"),response:template.response.replace("item",name),action:None,operands:None});}}
            write_json(Path::new(&args[3]),&json!({"schema":"uor-r4.literal-admission-source/1","fit":fit,"development":prior["development"],"exposed_first_use":prior["first_use"],"exposed_chain":prior["transfer"],"first_use":first_use,"transfer":transfer,"identifiers":identifiers,"scope":"71 prior literal construction frames plus32 original word-copy construction frames labeled NoOperation; protected computed roles unchanged. Prior transfers exposed. New16 literal,16 complete chain and8 identifier tasks opened after design selection. No serving target injection."}))?;
        }
        Some("admission-fit") if args.len()==6 => {
            let model=Model::from_bytes(&fs::read(&args[1])?)?;
            let donor=Model::from_bytes(&fs::read(&args[2])?)?;
            let source:Value=serde_json::from_slice(&fs::read(&args[3])?)?;
            let docs:Vec<TypedRoutingExample>=serde_json::from_value(source["fit"].clone())?;
            let mode=match args[5].as_str(){"angular"=>RoutingMode::Angular,"equality"=>RoutingMode::Equality,_=>return Err("literal admission mode".into())};
            let config=SourceRoutingConfig{learned_features:768,passes:4,proposals:12,max_seconds:30,mode,..SourceRoutingConfig::default()};
            let (candidate,report)=model.fit_literal_admission(&docs,config,&donor)?;
            let out=Path::new(&args[4]);fs::create_dir(out)?;write_new(&out.join("model.json"),&candidate.to_bytes()?)?;write_json(&out.join("fit.json"),&report)?;println!("{report}");
        }
        Some("literal-source") if args.len()==5 => {
            let model=Model::from_bytes(&fs::read(&args[1])?)?;
            let prior:Value=serde_json::from_slice(&fs::read(&args[2])?)?;
            let old:Source=serde_json::from_slice(&fs::read(&args[3])?)?;
            let mut fit:Vec<TypedRoutingExample>=serde_json::from_value(prior["fit"].clone())?;
            let mut preservation_labels=Vec::new();
            let mut prefixes:Vec<_>=old.fit.iter().filter(|c|c.world<4).map(|c|(c.id.clone(),c.prompt.clone(),c.response.clone())).collect();
            let mut seen=std::collections::BTreeSet::new();
            for c in &fit {if seen.insert(c.initial_prompt.clone()){prefixes.push((format!("prefix/{}",c.id),c.initial_prompt.clone(),c.initial_response.clone()));}}
            for (id,prompt,expected) in prefixes {
                let mut session=model.session(Control::Full)?;session.observe(&model,uor_r4_core::native_geometric::BOS)?;
                for t in model.encode(&prompt)? {session.observe(&model,t)?;}session.begin_response(&model)?;
                let mut out=Vec::new();let mut decision=None;let mut eos=false;
                for _ in 0..64 {let token=session.predict(&model)?.token;if let Some(d)=session.value_decision().filter(|d|d.cursor==0){decision=Some(d);}session.observe(&model,token)?;if token==uor_r4_core::native_geometric::EOS {eos=true;break;}out.push(token);}
                let response=String::from_utf8(model.decode(&out)?)?;
                let correct=eos&&response==expected;
                preservation_labels.push(json!({"id":id,"accepted":correct,"text":response,"expected":expected,"decision":decision}));
                if correct {fit.push(TypedRoutingExample {id:format!("literal-preserve/{id}"),current_query_literals: false, current_query_operand: false, literal_only:true,initial_prompt:String::new(),initial_response:String::new(),continuation:None,refresh:Vec::new(),target_intermediate:0,query:prompt,response:expected,action:decision.map(|d|d.action),operands:decision.map(|d|d.operands.map(|r|r.value))});}
            }
            for (i,(names,numbers)) in [(["suri","orin"],[13,4]),(["mira","neri"],[5,9]),(["kira","fenn"],[2,5]),(["ada","ben"],[8,7])].into_iter().enumerate(){for reverse in [false,true] {for kind in 0..4 {fit.push(literal_example(format!("literal-fit/{i}/{reverse}/{kind}"),names,numbers,reverse,kind));}}}
            let mut development=Vec::new();let mut first_use=Vec::new();let mut transfer=Vec::new();
            for (i,(names,numbers)) in [(["suri","orin"],[13,4]),(["kira","fenn"],[2,5])].into_iter().enumerate(){for reverse in [false,true] {for kind in 0..4 {development.push(literal_example(format!("literal-open/{i}/{reverse}/{kind}"),names,numbers,reverse,kind));}}}
            for (i,(names,numbers)) in [(["nova","sela"],[-7,19]),(["iven","rusk"],[25,6])].into_iter().enumerate(){for reverse in [false,true] {for kind in 0..4 {first_use.push(literal_example(format!("literal-transfer/{i}/{reverse}/{kind}"),names,numbers,reverse,kind));}}}
            for (i,(names,numbers)) in [(["nova","sela","tora","vela"],[-7,19,3,5,4]),(["iven","rusk","nela","kori"],[25,6,-4,13,2])].into_iter().enumerate(){for reverse in [false,true] {for kind in 0..4 {transfer.push(independent_example(format!("literal-chain-transfer/{i}/{reverse}/{kind}"),names,numbers,reverse,kind));}}}
            write_json(Path::new(&args[4]),&json!({"schema":"uor-r4.literal-selection-source/1","fit":fit,"development":development,"first_use":first_use,"transfer":transfer,"native_preservation_labels":preservation_labels,"initialization":model.artifact_cid(),"scope":"58 existing role cases, correct native-parent labels on original construction worlds0..3 and unique actual first prompts from existing role construction,32 explicit literal frames;16 exposed development;16 new literal and16 new complete three-turn transfers opened after design selection. No expected intermediate supplied. Prior labels come from native parent execution, never an LLM."}))?;
        }
        Some("independent-reachable-source") if args.len()==3 => {
            let mut source:Value=serde_json::from_slice(&fs::read(&args[1])?)?;
            for split in ["fit","development"] {
                let rows:Vec<TypedRoutingExample>=serde_json::from_value(source[split].clone())?;
                let (blocked,admitted):(Vec<_>,Vec<_>)=rows.into_iter().partition(|d|d.id.starts_with("independent-")&&d.id.contains("/true/"));
                source[format!("exposed_prefix_failures_{split}")]=serde_json::to_value(blocked)?;
                source[split]=serde_json::to_value(admitted)?;
            }
            source["scope"]=json!("58 reachable construction (42 prior roles +16 named independent);8 reachable OPEN development. Reversed computation-prefix failures are exposed and preserved, not repaired or counted as selector errors. Original16 first-use name/number/order trajectories remain unchanged and may fail before selection. No broad language or first-answer qualification.");
            write_json(Path::new(&args[2]),&source)?;
        }
        Some("independent-chain-source") if args.len()==3 => {
            let old:Value=serde_json::from_slice(&fs::read(&args[1])?)?;
            let mut fit:Vec<TypedRoutingExample>=serde_json::from_value(old["fit"].clone())?;
            for (i,nums) in [[13,4,5,9,2],[14,4,6,9,3],[-3,8,4,9,6],[21,3,7,11,4]].into_iter().enumerate() {
                for reverse in [false,true] {for kind in 0..4 {fit.push(independent_example(format!("independent-fit/{i}/{reverse}/{kind}"),["suri","orin","mira","neri"],nums,reverse,kind));}}
            }
            let mut development=Vec::new();
            for (i,nums) in [[13,4,5,9,2],[8,7,2,9,4]].into_iter().enumerate() {
                for reverse in [false,true] {for kind in 0..4 {development.push(independent_example(format!("independent-open/{i}/{reverse}/{kind}"),["suri","orin","mira","neri"],nums,reverse,kind));}}
            }
            let mut first_use=Vec::new();
            for (i,(names,nums)) in [(["luma","tavi","bela","zori"],[20,3,6,9,4]),(["vexa","dori","kira","fenn"],[-6,16,2,5,3])].into_iter().enumerate() {
                for reverse in [false,true] {for kind in 0..4 {first_use.push(independent_example(format!("independent-transfer/{i}/{reverse}/{kind}"),names,nums,reverse,kind));}}
            }
            write_json(Path::new(&args[2]),&json!({"schema":"uor-r4.independent-source/2","fit":fit,"development":development,"first_use":first_use,"scope":"74 construction including42 prior dependent roles;16 open development;16 predeclared changed name/number/computation-order transfers opened after selection. Both intermediate responses generated. No broad language qualification."}))?;
        }

        Some("canonical-roles") if args.len()==3 => {
            let model=Model::from_bytes(&fs::read(&args[1])?)?;
            let candidate=model.canonicalize_typed_role_aliases()?;
            write_new(Path::new(&args[2]),&candidate.to_bytes()?)?;
            println!("{}",json!({"before":model.artifact_cid(),"artifact":candidate.artifact_cid(),"change":"Exact Copy aliases cannot introduce a reflexive Add candidate; learned parameters unchanged."}));
        }
        Some("alias-source") if args.len()==3 => {
            let mut source:Value=serde_json::from_slice(&fs::read(&args[1])?)?;
            source["exposed_first_use"]=source["first_use"].clone();
            let mut reserved=Vec::new();
            for (i,(a,b,d,e)) in [(11,7,4,3),(16,5,8,4),(-5,13,6,2)].into_iter().enumerate() {
                for kind in [0,1,4,5] {
                    let mut case=role_example(format!("roles-alias-transfer/{i}/{kind}"),a,b,d,e,kind,true);
                    if i != 1 {case.refresh[0]=TypedRoutingTurn {prompt:"User: Repeat the updated total.\nAssistant:".into(),response:format!("{}.\n",a+b+d)};}
                    reserved.push(case);
                }
            }
            source["first_use"]=serde_json::to_value(reserved)?;
            source["scope"]=json!("Alias-admission revision: construction and development unchanged; old9/12 transfer remains an exposed negative. Twelve new operands/refresh cases after selection; refreshes original or updated. No refit or broad language qualification.");
            write_json(Path::new(&args[2]),&source)?;
        }
        Some("query-source") if args.len()==3 => {
            let mut source:Value=serde_json::from_slice(&fs::read(&args[1])?)?;
            source["exposed_alias_first_use"]=source["first_use"].clone();
            let mut reserved=Vec::new();
            for (i,(a,b,d,e)) in [(20,3,7,2),(9,8,4,6),(-6,16,5,3)].into_iter().enumerate() {
                for kind in [0,1,4,5] {
                    let mut case=role_example(format!("roles-query-transfer/{i}/{kind}"),a,b,d,e,kind,true);
                    if i != 1 {case.refresh[0]=TypedRoutingTurn {prompt:"User: Repeat the updated total.\nAssistant:".into(),response:format!("{}.\n",a+b+d)};}
                    reserved.push(case);
                }
            }
            source["first_use"]=serde_json::to_value(reserved)?;
            source["scope"]=json!("Explicit query-boundary revision: same construction; prior9/12 and8/12 transfers remain exposed negatives. Twelve new operand/refresh cases after selection. No supplied intermediates or sealed-language claim.");
            write_json(Path::new(&args[2]),&source)?;
        }
        Some("roles-source") if args.len()==2 => {
            let mut fit=Vec::new();
            for (i,(a,b,d,e)) in [(13,4,5,2),(8,7,6,3),(-3,8,4,5),(21,3,7,4),(10,6,2,7),(4,9,3,6)].into_iter().enumerate() {
                for kind in 0..7 {fit.push(role_example(format!("roles-fit/{i}/{kind}"),a,b,d,e,kind,false));}
            }
            let mut development=Vec::new();
            for (i,(a,b,d,e)) in [(13,4,5,2),(14,4,6,3),(-3,8,4,5)].into_iter().enumerate() {
                for kind in [0,1,4,5] {development.push(role_example(format!("roles-open/{i}/{kind}"),a,b,d,e,kind,false));}
            }
            let mut first_use=Vec::new();
            for (i,(a,b,d,e)) in [(19,12,5,4),(23,8,6,2),(-4,15,3,7)].into_iter().enumerate() {
                for kind in [0,1,4,5] {first_use.push(role_example(format!("roles-transfer/{i}/{kind}"),a,b,d,e,kind,true));}
            }
            write_json(Path::new(&args[1]),&json!({"schema":"uor-r4.typed-roles-source/1","fit":fit,"development":development,"first_use":first_use,"scope":"42 construction cases, 12 OPEN development, 12 after-selection transfers with changed operands and an actually generated original-total refresh that reverses derived recency. No intermediate insertion. Small authored scope, not general language."}))?;
        }
        Some("roles-fit"|"roles-fit-local"|"roles-fit-provenance"|"literal-fit") if args.len()==5 => {
            let mode=match args[4].as_str(){"angular"=>RoutingMode::Angular,"equality"=>RoutingMode::Equality,_=>return Err("typed role fit mode".into())};
            let model=Model::from_bytes(&fs::read(&args[1])?)?;
            let source:Value=serde_json::from_slice(&fs::read(&args[2])?)?;
            let docs:Vec<TypedRoutingExample>=serde_json::from_value(source["fit"].clone())?;
            let config=SourceRoutingConfig{learned_features:if args[0]=="literal-fit" {768} else if args[0]=="roles-fit-provenance" {256} else {192},passes:4,proposals:12,max_seconds:30,mode,..SourceRoutingConfig::default()};
            let (candidate,report)=if args[0]=="literal-fit" {model.fit_typed_literal_answers(&docs,config)?} else if args[0]=="roles-fit-provenance" {model.fit_typed_roles_provenance(&docs,config)?} else if args[0]=="roles-fit-local" {model.fit_typed_roles_local(&docs,config)?} else {model.fit_typed_roles(&docs,config)?};
            let out=Path::new(&args[3]);fs::create_dir(out)?;
            write_new(&out.join("model.json"),&candidate.to_bytes()?)?;
            write_json(&out.join("fit.json"),&report)?;println!("{report}");
        }
        Some("competition" | "independent") if args.len()==3 => {
            let model=Model::from_bytes(&fs::read(&args[1])?)?;
            let mut cases=Vec::new();
            for (a,b,delta) in [(13,4,5),(14,4,6),(-3,8,4)] {
                for (question,wanted) in if args[0]=="independent" {[("Repeat the total for suri and orin.",a+b),("Repeat the total for mira and neri.",delta+9),("Copy the total for suri and orin.",a+b),("Copy the total for mira and neri.",delta+9)]} else {[("Repeat the original total.",a+b),("Repeat the updated total.",a+b+delta),("Repeat the previous total.",a+b+delta),("Copy the total before the new coins.",a+b)]} {
                    let mut session=model.session(Control::Full)?;
                    session.observe(&model,uor_r4_core::native_geometric::BOS)?;
                    let mut turns=Vec::new();
                    let first=format!("User: suri has {a} coins. orin has {b} coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:");
                    let second=if args[0]=="independent" {format!("User: mira has {delta} coins. neri has 9 coins.\nUser: What is the sum of mira's and neri's coins?\nAssistant:")} else {format!("User: There are {delta} new coins. Add the new coins to the previous total.\nAssistant:")};
                    let third=format!("User: {question}\nAssistant:");
                    for (prompt,expected) in [(first,a+b),(second,if args[0]=="independent" {delta+9} else {a+b+delta}),(third,wanted)] {
                        for token in model.encode(&prompt)? {session.observe(&model,token)?;}
                        session.begin_response(&model)?;
                        let mut tokens=Vec::new();let mut decision=None;let mut eos=false;
                        for _ in 0..32 {
                            let token=session.predict(&model)?.token;
                            if let Some(d)=session.value_decision().filter(|d|d.cursor==0){decision=Some(d);}
                            session.observe(&model,token)?;
                            if token==uor_r4_core::native_geometric::EOS {eos=true;break;}
                            tokens.push(token);
                        }
                        let text=String::from_utf8(model.decode(&tokens)?)?;
                        turns.push(json!({"prompt":prompt,"expected":format!("{expected}.\n"),"exact":text==format!("{expected}.\n")&&eos,"text":text,"eos":eos,"decision":decision}));
                        session.end_response(&model)?;
                    }
                    cases.push(json!({"turns":turns,"work":session.work}));
                }
            }
            let report=json!({"artifact":model.artifact_cid(),"scope":format!("OPEN actual three-turn {} generation. No expected intermediate is inserted.",args[0]),"cases":cases});
            write_json(Path::new(&args[2]),&report)?;
        }
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
            if !["fit","development","first_use","exposed_first_use","exposed_alias_first_use","transfer","identifiers","exposed_chain"].contains(&args[3].as_str()){return Err("typed evaluation split".into());}
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
