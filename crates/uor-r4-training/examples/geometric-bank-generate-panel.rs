//! Offline input-derived answerability reference and prose-label preparation.
//! The bounded text rule is an authoring control, never an inference mechanism.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    answer_oracle::{FrozenAnswers, RecordedValueIntent},
    report_output,
};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding,
    geometric_source_emission_view::SourceEmissionCompiler,
};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{sha256_bytes, sha256_file};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const CAP: usize = 64 << 20;
fn invalid(s: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s.into())
}
struct Args {
    input: PathBuf,
    tokenizer: PathBuf,
    out: PathBuf,
    reference_only: bool,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Segment {
    Source {
        event: u64,
        record: u64,
        commit: u64,
        scope: String,
        entity: Vec<u32>,
        relation: u32,
        view: u32,
        original_source_ids: Vec<u32>,
    },
    Context {
        event: u64,
        role: u32,
        token_ids: Vec<u32>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Role {
    Job,
    Where,
    Other,
}
impl Role {
    fn name(self) -> &'static str {
        match self {
            Self::Job => "job",
            Self::Where => "where",
            Self::Other => "other",
        }
    }
    fn prose(self, literal: &str) -> Result<String> {
        match self {
            Self::Job => Ok(format!("Your job is {literal}.")),
            Self::Where => Ok(format!("You live in {literal}.")),
            Self::Other => Err(invalid("other role is not a supported question").into()),
        }
    }
}
struct Reference {
    role: Role,
    literal: String,
    segment: usize,
    event: u64,
    record: u64,
    commit: u64,
}
fn query_role(text: &str) -> Result<Role> {
    match text {
        "What is my current job?"|"What job do I currently have?"|"What is my job now?"|"Which job do I have now?"|"Remind me of my current job."|"Tell me my current job."|"What is my job currently?"|"Remind me what my job is." => Ok(Role::Job),
        "Where do I currently live?"|"Where do I live now?"|"What is my current residence?"|"Remind me where I currently live."|"Tell me where I live now."|"Where is my current home?"|"Where do I live currently?"|"Where am I living currently?"|"Remind me where I live." => Ok(Role::Where),
        _ => Err(invalid(format!("unsupported input query text {text:?}; extend the declared authoring reference, not a model fallback")).into()),
    }
}
fn cue_role(cue: &str, literal: &str) -> Result<Role> {
    for (role, prefix) in [
        (Role::Job, "My current job is "),
        (Role::Job, "My current job has changed to "),
        (Role::Job, "I work as "),
        (Role::Job, "I work as a "),
        (Role::Where, "I currently live in "),
        (Role::Where, "I now live in "),
        (Role::Where, "I live in "),
        (Role::Other, "I mentioned "),
    ] {
        if cue == format!("{prefix}{literal}.") || cue == prefix.trim_end() {
            return Ok(role);
        }
    }
    Err(invalid(format!(
        "raw input cue {cue:?} does not bind its exact Source literal {literal:?}"
    ))
    .into())
}
fn decode(tok: &ByteBpeTokenizer, ids: &[u32], binding: &SourceActionBinding) -> Result<String> {
    if ids.iter().any(|&id| !binding.admits_token(id)) {
        return Err(invalid("unknown or sparse-hole model token").into());
    }
    if ids.is_empty() {
        return Err(invalid("empty model token sequence").into());
    }
    Ok(String::from_utf8(tok.decode_bytes(ids))?)
}
fn encoded(tok: &ByteBpeTokenizer, text: &str, binding: &SourceActionBinding) -> Result<Vec<u32>> {
    let ids = tok.encode(text);
    if decode(tok, &ids, binding)? != text {
        return Err(invalid("exact byte-BPE roundtrip failed").into());
    }
    Ok(ids)
}
/// Chooses solely from model input bytes and chronology. No label/context-data
/// receipt or typed_facts field is accepted by this function.
#[derive(Clone, Copy)]
enum ReferenceChronology {
    HighestEvent,
    LatestSegment,
}
fn reference(
    packet: &Packet,
    tok: &ByteBpeTokenizer,
    binding: &SourceActionBinding,
) -> Result<Reference> {
    reference_with_chronology(packet, tok, binding, ReferenceChronology::HighestEvent)
}
fn reference_with_chronology(
    packet: &Packet,
    tok: &ByteBpeTokenizer,
    binding: &SourceActionBinding,
    chronology: ReferenceChronology,
) -> Result<Reference> {
    if packet.id.is_empty() || !packet.actual_prefix_ids.is_empty() {
        return Err(invalid("blank ID or supplied answer prefix").into());
    }
    let wanted = query_role(&decode(tok, &packet.query_ids, binding)?)?;
    let mut best: Option<Reference> = None;
    let mut address: Option<(&str, &[u32])> = None;
    for (index, segment) in packet.segments.iter().enumerate() {
        if let Segment::Source {
            event,
            record,
            commit,
            scope,
            entity,
            relation,
            view,
            original_source_ids,
        } = segment
        {
            if scope.is_empty() || entity.is_empty() || *view != 0 {
                return Err(invalid("unsupported Source address/view").into());
            }
            if let Some((old_scope, old_entity)) = address {
                if old_scope != scope || old_entity != entity.as_slice() {
                    return Err(invalid("multiple Source namespaces/entities require an explicit query-address reference").into());
                }
            } else {
                address = Some((scope.as_str(), entity.as_slice()));
            }
            let literal = decode(tok, original_source_ids, binding)?;
            let previous = index
                .checked_sub(1)
                .ok_or_else(|| invalid("Source lacks preceding raw Context cue"))?;
            let Some(Segment::Context {
                event: cue_event,
                role,
                token_ids,
            }) = packet.segments.get(previous)
            else {
                return Err(invalid("Source predecessor is not Context").into());
            };
            if cue_event != event || *role != 1 {
                return Err(invalid("Source/raw user cue event binding differs").into());
            }
            let role = cue_role(&decode(tok, token_ids, binding)?, &literal)?;
            let addressed_role = match (scope.as_str(), *relation) {
                ("compiler-probe", 1) | ("m-world-v2", 7) => Role::Job,
                ("compiler-probe", 2) | ("m-world-v2", 8) => Role::Where,
                ("m-world-v2", 9) => Role::Other,
                _ => {
                    return Err(invalid(
                        "unknown input Source relation/scope; semantic mapping required",
                    )
                    .into())
                }
            };
            if role != addressed_role {
                return Err(invalid("input cue and Source relation disagree").into());
            }
            if role != wanted {
                continue;
            }
            if let ReferenceChronology::HighestEvent = chronology {
                if let Some(old) = &best {
                    if old.event == *event {
                        return Err(invalid("ambiguous equal-event matching Sources").into());
                    }
                    if old.event > *event {
                        continue;
                    }
                }
            }
            best = Some(Reference {
                role,
                literal,
                segment: index,
                event: *event,
                record: *record,
                commit: *commit,
            });
        }
    }
    best.ok_or_else(|| invalid("input has no matching current Source").into())
}
fn read(path: &Path) -> Result<Vec<u8>> {
    let bytes = fs::read(path)?;
    if bytes.len() > CAP {
        return Err(invalid("input file exceeds64MiB").into());
    }
    Ok(bytes)
}
fn field<'a>(v: &'a Value, key: &str) -> Result<&'a Value> {
    v.get(key)
        .ok_or_else(|| invalid(format!("required field {key} absent")).into())
}
fn string<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    field(v, key)?
        .as_str()
        .ok_or_else(|| invalid(format!("{key} is not text")).into())
}
fn array<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    field(v, key)?
        .as_array()
        .ok_or_else(|| invalid(format!("{key} is not array")).into())
}
fn write(root: &Path, name: &str, v: &Value) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(v)?;
    let used = fs::read_dir(root)?.try_fold(0usize, |sum, e| -> io::Result<usize> {
        Ok(sum + e?.metadata()?.len() as usize)
    })?;
    if used.saturating_add(bytes.len()) > CAP - 128 * 1024 {
        return Err(invalid("output64MiB cap").into());
    }
    fs::write(root.join(name), bytes)?;
    Ok(())
}
fn args() -> Result<Args> {
    let mut fields = BTreeMap::new();
    let mut reference_only = false;
    let mut it = std::env::args().skip(1);
    while let Some(k) = it.next() {
        if k == "--reference-only" {
            if reference_only {
                return Err(invalid("duplicate --reference-only").into());
            }
            reference_only = true;
            continue;
        }
        if !["--input", "--tokenizer", "--out"].contains(&k.as_str()) {
            return Err(invalid("unknown flag").into());
        }
        let v = it.next().ok_or_else(|| invalid("missing flag value"))?;
        if fields.insert(k, v).is_some() {
            return Err(invalid("duplicate flag").into());
        }
    }
    let input = fs::canonicalize(
        fields
            .remove("--input")
            .ok_or_else(|| invalid("--input required"))?,
    )?;
    let tokenizer = fs::canonicalize(
        fields
            .remove("--tokenizer")
            .ok_or_else(|| invalid("--tokenizer required"))?,
    )?;
    let out = output_support::prospective_output(Path::new(
        &fields
            .remove("--out")
            .ok_or_else(|| invalid("--out required"))?,
    ))?;
    if out.starts_with(&input) || input.starts_with(&out) || tokenizer.starts_with(&out) {
        return Err(invalid("input/output overlap").into());
    }
    Ok(Args {
        input,
        tokenizer,
        out,
        reference_only,
    })
}
/// Selection precedes and is independent of all label inspection. This is an
/// offline authoring control: the returned identity is never a model input.
fn reference_receipt(
    packet: &Packet,
    label: &Value,
    tok: &ByteBpeTokenizer,
    binding: &SourceActionBinding,
    compiler: &SourceEmissionCompiler,
) -> Result<Value> {
    // Native chronology is physical segment order; event IDs are opaque and
    // may repeat or decrease. The retained default author's policy is unchanged.
    let r = reference_with_chronology(packet, tok, binding, ReferenceChronology::LatestSegment)?;
    if string(label, "id")? != packet.id {
        return Err(invalid("reference label row identity mismatch").into());
    }
    let answers: FrozenAnswers = serde_json::from_value(field(label, "answers")?.clone())?;
    answers.validate()?;
    let prose = format!(" {}", r.role.prose(&r.literal)?);
    if answers.intent != RecordedValueIntent::Current || !answers.accepts(&prose) {
        return Err(invalid(format!("{}: input-derived prose fails existing membership; labels cannot select a different Source", packet.id)).into());
    }
    let prefix = match r.role {
        Role::Job => " Your job is ",
        Role::Where => " You live in ",
        Role::Other => return Err(invalid("unsupported reference prose role").into()),
    };
    let literal_start = prefix.len();
    let literal_end = literal_start
        .checked_add(r.literal.len())
        .ok_or_else(|| invalid("literal byte span overflow"))?;
    let target = encoded(tok, &prose, binding)?;
    let mut byte_offset = 0usize;
    let mut spans = Vec::new();
    for (position, &id) in target.iter().enumerate() {
        let bytes = tok.decode_bytes(&[id]);
        let end = byte_offset
            .checked_add(bytes.len())
            .ok_or_else(|| invalid("canonical token byte span overflow"))?;
        if prose.as_bytes().get(byte_offset..end) != Some(bytes.as_slice()) {
            return Err(invalid("canonical token byte provenance differs from exact prose").into());
        }
        let lo = byte_offset.max(literal_start);
        let hi = end.min(literal_end);
        spans.push(json!({"canonical_position":position,"token_id":id,"reply_byte_start":byte_offset,"reply_byte_end":end,"overlaps_literal":lo<hi,"wholly_inside_literal":byte_offset>=literal_start&&end<=literal_end,"literal_byte_start":if lo<hi {Some(lo-literal_start)} else {None},"literal_byte_end":if lo<hi {Some(hi-literal_start)} else {None}}));
        byte_offset = end;
    }
    if byte_offset != prose.len() {
        return Err(invalid("canonical token byte provenance incomplete").into());
    }
    let mut views = Vec::new();
    for (segment_index, segment) in packet.segments.iter().enumerate() {
        if let Segment::Source {
            original_source_ids,
            ..
        } = segment
        {
            views.push(json!({"segment_index":segment_index,"source_view":compiler.compile(original_source_ids)?}));
        }
    }
    let selected = views
        .iter()
        .find(|v| v["segment_index"] == json!(r.segment))
        .ok_or_else(|| invalid("selected Source view absent"))?;
    if selected["source_view"]["original_bytes"] != json!(r.literal.as_bytes()) {
        return Err(invalid("selected Source literal byte binding differs").into());
    }
    let mut target_eos = target;
    target_eos.push(binding.eos_token_id());
    Ok(
        json!({"id":packet.id,"serialized_input_packet_sha256":sha256_bytes(&serde_json::to_vec(packet)?),"reference_role":r.role.name(),"reference_source_segment_labels_only":r.segment,"reference_event":r.event,"reference_record":r.record,"reference_commit":r.commit,"reference_literal_labels_only":r.literal,"reference_prose_labels_only":prose,"reply_literal_byte_start":literal_start,"reply_literal_byte_end":literal_end,"canonical_token_byte_provenance_labels_only":spans,"target_ids_labels_only":target_eos,"source_views":views,"existing_prose_membership_pass":true}),
    )
}
fn run_reference_only(a: &Args, start: Instant) -> Result<Value> {
    report_output::verify(&a.input)?;
    let tokenizer = read(&a.tokenizer)?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer)
        .ok_or_else(|| invalid("actual tokenizer invalid"))?;
    let binding = SourceActionBinding::new(&tokenizer)?;
    let compiler = SourceEmissionCompiler::new(&tokenizer)?;
    let raw = read(&a.input.join("inputs.json"))?;
    let raw_labels = read(&a.input.join("labels.json"))?;
    let inputs: Value = serde_json::from_slice(&raw)?;
    let labels: Value = serde_json::from_slice(&raw_labels)?;
    if string(&inputs, "schema")? != "uor-r4.native-source-bank-probe-input/1"
        || string(&labels, "schema")? != "uor-r4.native-source-bank-labels/1"
        || string(&labels, "protocol")? != "uor-r4.literal-role-dialogue/2"
        || field(&labels, "membership_only")? != &json!(true)
    {
        return Err(invalid("reference input schema/protocol differs").into());
    }
    let packets = array(&inputs, "cases")?;
    let labels = array(&labels, "cases")?;
    if packets.is_empty() || packets.len() > 512 || packets.len() != labels.len() {
        return Err(invalid("reference rows require matching nonempty counts <=512").into());
    }
    let mut seen = BTreeSet::new();
    let mut receipts = Vec::new();
    for (packet, label) in packets.iter().zip(labels) {
        if start.elapsed().as_secs() >= 300 {
            return Err(invalid("reference300second bound").into());
        }
        let packet: Packet = serde_json::from_value(packet.clone())?;
        if !seen.insert(packet.id.clone()) {
            return Err(invalid("duplicate reference row ID").into());
        }
        receipts.push(reference_receipt(
            &packet, label, &tok, &binding, &compiler,
        )?);
    }
    let identities = json!({"input_manifest_sha256":sha256_file(&a.input.join("manifest.json"))?,"inputs_sha256":sha256_bytes(&raw),"labels_sha256":sha256_bytes(&raw_labels),"tokenizer_sha256":sha256_bytes(&tokenizer)});
    write(
        &a.out,
        "answerability-reference.json",
        &json!({"schema":"uor-r4.bank-generate-answerability/2","provenance":"fresh derived reference; not restoration of historical reference","selector":"input-only raw query/cue bytes plus exact Source relation and latest matching physical segment; selection before existing prose label validation","runtime_reference":false,"chronology_scope":"native physical segment order; latest matching Source; event/record/commit are preserved opaque provenance, not ordering keys","input_bindings":identities,"cases":receipts}),
    )?;
    Ok(
        json!({"schema":"uor-r4.bank-generate-panel-reference/1","status":"COMPLETED","cases":packets.len(),"reference_only":true,"input_bindings":identities,"derived_not_restored":true,"runtime_reference":false,"inputs_written":false,"labels_reauthored":false,"new_heldout_claim":false,"exposure_scope":"retains original input exposure; reference derivation creates no heldout data","native_predictions":"NOT_RUN","model_training":"NOT_RUN","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_file(&std::env::current_exe()?)?,"elapsed_seconds":start.elapsed().as_secs_f64()}),
    )
}
fn run(a: &Args, start: Instant) -> Result<Value> {
    report_output::verify(&a.input)?;
    let tokenizer = read(&a.tokenizer)?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer)
        .ok_or_else(|| invalid("actual tokenizer invalid"))?;
    let binding = SourceActionBinding::new(&tokenizer)?;
    let compiler = SourceEmissionCompiler::new(&tokenizer)?;
    let raw = read(&a.input.join("inputs.json"))?;
    let inputs: Value = serde_json::from_slice(&raw)?;
    let oldlabels: Value = serde_json::from_slice(&read(&a.input.join("labels.json"))?)?;
    let mut context: Value = serde_json::from_slice(&read(&a.input.join("context-data.json"))?)?;
    if string(&inputs, "schema")? != "uor-r4.native-source-bank-probe-input/1"
        || string(&oldlabels, "schema")? != "uor-r4.native-source-bank-labels/1"
        || string(&oldlabels, "protocol")? != "uor-r4.literal-role-dialogue/2"
        || field(&oldlabels, "membership_only")? != &json!(true)
        || string(&context, "schema")? != "uor-r4.geometric-bank-context-data/1"
        || string(&context, "layout_policy")? != "raw-natural-allbank-pairs/1"
    {
        return Err(invalid("sealed all-source schema/protocol differs").into());
    }
    let packets = array(&inputs, "cases")?;
    let labels = array(&oldlabels, "cases")?;
    let contexts = context
        .get_mut("cases")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| invalid("context cases absent"))?;
    if packets.is_empty()
        || packets.len() > 512
        || packets.len() % 2 != 0
        || labels.len() != packets.len()
        || contexts.len() != packets.len()
    {
        return Err(invalid("paired panel counts differ (2..512 even required)").into());
    }
    let mut authored = Vec::new();
    let mut receipts = Vec::new();
    let mut seen = BTreeSet::new();
    let mut roles = Vec::new();
    let mut strata = BTreeMap::<String, usize>::new();
    for (i, packet_value) in packets.iter().enumerate() {
        if start.elapsed().as_secs() >= 300 {
            return Err(invalid("authoring300second bound").into());
        }
        let packet: Packet = serde_json::from_value(packet_value.clone())?;
        if !seen.insert(packet.id.clone())
            || string(&labels[i], "id")? != packet.id
            || string(&contexts[i], "id")? != packet.id
        {
            return Err(invalid("case identity mismatch").into());
        }
        let r = reference(&packet, &tok, &binding)?;
        let old: FrozenAnswers = serde_json::from_value(field(&labels[i], "answers")?.clone())?;
        old.validate()?;
        if old.intent != RecordedValueIntent::Current || !old.accepts(&format!("{}.", r.literal)) {
            return Err(invalid(format!(
                "{}: input-derived answer fails old membership positive control",
                packet.id
            ))
            .into());
        }
        if string(&contexts[i], "query_role")? != r.role.name()
            || string(&contexts[i], "kind")? != "bank"
        {
            return Err(invalid("context role/kind audit mismatch").into());
        }
        let stratum = string(&contexts[i], "stratum")?.to_owned();
        *strata.entry(stratum.clone()).or_default() += 1;
        let mut views = Vec::new();
        let mut candidates = BTreeSet::new();
        let mut projected = packet.query_ids.len();
        let mut source_count = 0;
        for (j, s) in packet.segments.iter().enumerate() {
            match s {
                Segment::Context { token_ids, .. } => {
                    decode(&tok, token_ids, &binding)?;
                    projected += token_ids.len();
                }
                Segment::Source {
                    original_source_ids,
                    ..
                } => {
                    let view = compiler.compile(original_source_ids)?;
                    projected += view.emitted_token_ids().len();
                    candidates.extend(view.emitted_token_ids().iter().copied());
                    views.push(json!({"segment_index":j,"source_view":view}));
                    source_count += 1;
                }
            }
        }
        if field(&contexts[i], "source_views")? != &json!(views) {
            return Err(invalid("source views fail independent tokenizer recompilation").into());
        }
        // Frame the accepted bytes once at authoring. Runtime membership is
        // exact; neither an evaluator trim nor a serving formatter supplies it.
        let answer = format!(" {}", r.role.prose(&r.literal)?);
        let mut target = encoded(&tok, &answer, &binding)?;
        target.push(binding.eos_token_id());
        let prefix = match r.role {
            Role::Job => " Your job is ",
            Role::Where => " You live in ",
            Role::Other => return Err(invalid("unsupported query").into()),
        };
        let noncopy = encoded(&tok, prefix, &binding)?
            .into_iter()
            .filter(|id| !candidates.contains(id))
            .collect::<BTreeSet<_>>();
        let actual_noncopy = target[..target.len() - 1]
            .iter()
            .copied()
            .filter(|id| noncopy.contains(id))
            .collect::<BTreeSet<_>>();
        if actual_noncopy.is_empty() {
            return Err(invalid(format!("{}: response framing has no actual noncandidate token; cannot require ordinary Generate",packet.id)).into());
        }
        if target.len() > 32
            || projected + 32 > 128
            || projected + target.len() > 128
            || source_count < 2
        {
            return Err(invalid("complete answer32/context128/multisource budget mismatch").into());
        }
        let answers = FrozenAnswers {
            intent: RecordedValueIntent::Current,
            accepted: vec![answer],
        };
        answers.validate()?;
        authored.push(json!({"id":packet.id,"answers":answers}));
        contexts[i]["target_ids_labels_only"] = json!(target);
        receipts.push(json!({"id":packet.id,"reference_role":r.role.name(),"reference_source_segment_labels_only":r.segment,"reference_event":r.event,"reference_record":r.record,"reference_commit":r.commit,"reference_literal_labels_only":r.literal,"old_membership_pass":true,"stratum":stratum,"source_count":source_count,"actual_reply_noncandidate_token_ids":actual_noncopy,"projected_context_tokens":projected,"target_ids_labels_only":target}));
        roles.push(r.role);
    }
    let mut pairs = BTreeSet::new();
    for i in (0..packets.len()).step_by(2) {
        let pair = string(&contexts[i], "pair_id")?;
        if pair.is_empty()
            || !pairs.insert(pair.to_owned())
            || string(&contexts[i + 1], "pair_id")? != pair
            || field(&packets[i], "segments")? != field(&packets[i + 1], "segments")?
            || field(&packets[i], "query_ids")? == field(&packets[i + 1], "query_ids")?
            || roles[i] == roles[i + 1]
        {
            return Err(invalid("adjacent same-bank opposite-query pair mismatch").into());
        }
    }
    // Preserve model inputs byte for byte; only labels and label receipts change.
    fs::write(a.out.join("inputs.json"), &raw)?;
    fs::write(a.out.join("tokenizer.json"), &tokenizer)?;
    write(
        &a.out,
        "labels.json",
        &json!({"schema":"uor-r4.native-source-bank-labels/1","protocol":"uor-r4.literal-role-dialogue/2","membership_only":true,"cases":authored}),
    )?;
    write(&a.out, "context-data.json", &context)?;
    write(
        &a.out,
        "answerability-reference.json",
        &json!({"schema":"uor-r4.bank-generate-answerability/1","selector":"input-only raw query/cue bytes plus exact Source relation and event; never label or typed_facts selection","runtime_reference":false,"cases":receipts}),
    )?;
    Ok(
        json!({"schema":"uor-r4.bank-generate-panel/1","status":"COMPLETED","cases":packets.len(),"samebank_opposite_query_pairs":pairs.len(),"input_root":a.input,"input_manifest_sha256":sha256_file(&a.input.join("manifest.json"))?,"input_bytes_sha256":sha256_bytes(&raw),"output_inputs_sha256":sha256_file(&a.out.join("inputs.json"))?,"tokenizer_sha256":sha256_bytes(&tokenizer),"strata":strata,"old_membership_positive_control_passed":packets.len(),"all_sources_admitted":true,"learned_prose_labels_only":true,"new_heldout_claim":false,"exposure_scope":"relabelled existing sealed inputs retain original exposure; changing answer framing does not create held-out data","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"native_predictions":"NOT_RUN","model_training":"NOT_RUN","elapsed_seconds":start.elapsed().as_secs_f64()}),
    )
}
fn main() -> Result<()> {
    let a = args()?;
    report_output::claim(&a.out)?;
    let start = Instant::now();
    let result = if a.reference_only {
        run_reference_only(&a, start)
    } else {
        run(&a, start)
    };
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.bank-generate-panel/1","status":"FAILED","error":e.to_string(),"elapsed_seconds":start.elapsed().as_secs_f64(),"model_quality_verdict":"NOT_APPLICABLE"})
        }
    };
    write(&a.out, "report.json", &report)?;
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tokenizer() -> Result<(ByteBpeTokenizer, SourceActionBinding)> {
        let mut vocab = serde_json::Map::new();
        for (id, token) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
            vocab.insert((*token).into(), json!(id));
        }
        for (offset, byte) in (b'!'..=b'~').enumerate() {
            vocab.insert(char::from(byte).to_string(), json!(offset + 3));
        }
        vocab.insert("Ġ".into(), json!(97));
        let bytes = serde_json::to_vec(
            &json!({"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":vocab,"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}),
        )?;
        Ok((
            ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
                .ok_or_else(|| invalid("test tokenizer"))?,
            SourceActionBinding::new(&bytes)?,
        ))
    }
    fn packet(tok: &ByteBpeTokenizer) -> Packet {
        let mut segments = Vec::new();
        for (event, relation, cue, value) in [
            (10, 1, "My current job is amber.", "amber"),
            (
                50,
                1,
                "My current job has changed to amber amber.",
                "amber amber",
            ),
            (90, 2, "I currently live in cedar.", "cedar"),
        ] {
            segments.push(Segment::Context {
                event,
                role: 1,
                token_ids: tok.encode(cue),
            });
            segments.push(Segment::Source {
                event,
                record: relation as u64,
                commit: event,
                scope: "compiler-probe".into(),
                entity: vec![1],
                relation,
                view: 0,
                original_source_ids: tok.encode(value),
            });
        }
        Packet {
            id: "fixture".into(),
            segments,
            query_ids: tok.encode("What is my current job?"),
            actual_prefix_ids: vec![],
        }
    }
    #[test]
    fn input_reference_current_versions_repetition_distractor_and_query_swap() -> Result<()> {
        let (tok, binding) = tokenizer()?;
        let mut p = packet(&tok);
        let job = reference(&p, &tok, &binding)?;
        assert_eq!(job.literal, "amber amber");
        assert_eq!(job.event, 50);
        assert_eq!(job.segment, 3);
        assert_eq!(job.role.prose(&job.literal)?, "Your job is amber amber.");
        p.query_ids = tok.encode("Where do I currently live?");
        let home = reference(&p, &tok, &binding)?;
        assert_eq!(home.literal, "cedar");
        assert_eq!(home.role.prose(&home.literal)?, "You live in cedar.");
        let mut reversed = packet(&tok);
        reversed.segments.swap(0, 2);
        reversed.segments.swap(1, 3);
        assert_eq!(reference(&reversed, &tok, &binding)?.literal, "amber amber");
        Ok(())
    }
    #[test]
    fn reference_rejects_inconsistent_cue_and_supplied_answer_prefix() -> Result<()> {
        let (tok, binding) = tokenizer()?;
        let mut p = packet(&tok);
        if let Segment::Context { token_ids, .. } = &mut p.segments[2] {
            *token_ids = tok.encode("I currently live in amber amber.");
        }
        assert!(reference(&p, &tok, &binding).is_err());
        let mut p = packet(&tok);
        p.actual_prefix_ids = tok.encode(" Your job is amber.");
        assert!(reference(&p, &tok, &binding).is_err());
        assert!(query_role("What was my previous job?").is_err());
        Ok(())
    }
    fn compiler() -> Result<SourceEmissionCompiler> {
        // Use the exact same tokenizer fixture bytes, not a separately chosen map.
        let mut vocab = serde_json::Map::new();
        for (id, token) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
            vocab.insert((*token).into(), json!(id));
        }
        for (offset, byte) in (b'!'..=b'~').enumerate() {
            vocab.insert(char::from(byte).to_string(), json!(offset + 3));
        }
        vocab.insert("Ġ".into(), json!(97));
        Ok(SourceEmissionCompiler::new(&serde_json::to_vec(
            &json!({"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":vocab,"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}),
        )?)?)
    }
    fn prose_label(answer: &str) -> Value {
        json!({"id":"fixture","answers":{"intent":"current","accepted":[answer]}})
    }
    #[test]
    fn reference_only_labels_cannot_steer_source_and_byte_spans_are_exact() -> Result<()> {
        let (tok, binding) = tokenizer()?;
        let p = packet(&tok);
        let c = compiler()?;
        assert!(
            reference_receipt(&p, &prose_label(" Your job is amber."), &tok, &binding, &c).is_err()
        );
        let receipt = reference_receipt(
            &p,
            &prose_label(" Your job is amber amber."),
            &tok,
            &binding,
            &c,
        )?;
        assert_eq!(receipt["reference_source_segment_labels_only"], json!(3));
        assert_eq!(receipt["reply_literal_byte_start"], json!(13));
        assert_eq!(receipt["reply_literal_byte_end"], json!(24));
        assert_eq!(
            receipt["source_views"]
                .as_array()
                .ok_or_else(|| invalid("views"))?
                .len(),
            3
        );
        let spans = array(&receipt, "canonical_token_byte_provenance_labels_only")?;
        assert_eq!(
            spans
                .iter()
                .filter(|s| s["overlaps_literal"] == json!(true))
                .count(),
            11
        );
        Ok(())
    }
    #[test]
    fn reference_only_same_value_reassertion_preserves_physical_identity_and_scope() -> Result<()> {
        let (tok, binding) = tokenizer()?;
        let mut p = packet(&tok);
        if let Segment::Context { token_ids, .. } = &mut p.segments[2] {
            *token_ids = tok.encode("My current job has changed to amber.");
        }
        if let Segment::Source {
            original_source_ids,
            commit,
            ..
        } = &mut p.segments[3]
        {
            *original_source_ids = tok.encode("amber");
            *commit = 10; // Same record AND commit: event/ordinal still distinguish.
        }
        let c = compiler()?;
        let receipt =
            reference_receipt(&p, &prose_label(" Your job is amber."), &tok, &binding, &c)?;
        assert_eq!(receipt["reference_source_segment_labels_only"], json!(3));
        assert_eq!(receipt["reference_event"], json!(50));
        assert_eq!(receipt["reference_record"], json!(1));
        assert_eq!(receipt["reference_commit"], json!(10));
        // Equal/decreasing event IDs are opaque; the last matching physical
        // occurrence remains selected even when record/commit/value also match.
        for later_event in [10, 7] {
            if let Segment::Context { event, .. } = &mut p.segments[2] {
                *event = later_event;
            }
            if let Segment::Source { event, .. } = &mut p.segments[3] {
                *event = later_event;
            }
            let receipt =
                reference_receipt(&p, &prose_label(" Your job is amber."), &tok, &binding, &c)?;
            assert_eq!(receipt["reference_source_segment_labels_only"], json!(3));
            assert_eq!(receipt["reference_event"], json!(later_event));
            if later_event == 10 {
                assert!(reference(&p, &tok, &binding).is_err());
            } else {
                assert_eq!(reference(&p, &tok, &binding)?.segment, 1);
            }
        }
        Ok(())
    }
}
