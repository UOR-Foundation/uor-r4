//! Offline ancestor labels and actual prompt-only checks for historical version intent.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufWriter, Read, Write},
    path::Path,
};
use uor_r4_core::answer_oracle::{self, Intent};
use uor_r4_core::native_geometric::{
    Control, HistoricalVersionExample, Model, RoutingMode, SourceRoutingConfig, BOS, EOS,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn save(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec(value)?)?;
    Ok(())
}
fn generate(
    model: &Model,
    history: &[String],
    prompt: &str,
    control: Control,
    verify: bool,
    inherited: bool,
) -> Result<Value> {
    let tokens = model.encode(prompt)?;
    if prompt.len() > if inherited { 65536 } else { 4096 }
        || tokens.len() > if inherited { 8192 } else { 512 }
    {
        return Err(format!(
            "configured input bound: {} bytes, {} tokens",
            prompt.len(),
            tokens.len()
        )
        .into());
    }
    let mut session = model.session(control)?;
    session.observe(model, BOS)?;
    // Prior turns are answered by this model under the same control; their
    // responses stay in the session exactly as a user would have seen them.
    for prior in history {
        let prior_tokens = model.encode(prior)?;
        if prior.len() > 4096 || prior_tokens.len() > 512 {
            return Err("configured history input bound".into());
        }
        if session.needs_input_boundary() {
            session.end_response(model)?;
        }
        for t in prior_tokens {
            session.observe(model, t)?;
        }
        session.begin_response(model)?;
        for _ in 0..96 {
            let p = session.predict(model)?;
            session.observe(model, p.token)?;
            if p.token == EOS {
                break;
            }
        }
    }
    if session.needs_input_boundary() {
        session.end_response(model)?;
    }
    for t in tokens {
        session.observe(model, t)?;
    }
    session.begin_response(model)?;
    let initial: Value = serde_json::from_slice(&session.checkpoint()?)?;
    let mut generated = Vec::new();
    let mut first = Value::Null;
    let mut eos = false;
    let mut positions = 0;
    for i in 0..96 {
        let mut restored = if verify {
            Some(model.restore_session(&session.checkpoint()?)?)
        } else {
            None
        };
        let p = session.predict(model)?;
        if verify && session.predict(model)? != p {
            return Err("repeated prediction differs".into());
        }
        if let Some(s) = &mut restored {
            if s.predict(model)? != p {
                return Err("restored prediction differs".into());
            }
        }
        if i == 0 {
            first = json!({"word_copy":session.word_copy_decision(),"field":session.field_composition_decision()});
        }
        session.observe(model, p.token)?;
        if let Some(s) = &mut restored {
            s.observe(model, p.token)?;
            let mut a: Value = serde_json::from_slice(&session.checkpoint()?)?;
            let mut b: Value = serde_json::from_slice(&s.checkpoint()?)?;
            a.as_object_mut().ok_or("checkpoint object")?.remove("work");
            b.as_object_mut().ok_or("checkpoint object")?.remove("work");
            if a != b {
                return Err("complete restored causal state differs".into());
            }
            positions += 1;
        }
        if p.token == EOS {
            eos = true;
            break;
        }
        generated.push(p.token);
    }
    let final_state: Value = serde_json::from_slice(&session.checkpoint()?)?;
    Ok(
        json!({"text":String::from_utf8(model.decode(&generated)?)?,"eos":eos,"first_decision":first,"initial_relations":initial["values"]["relations"],"final_relations":final_state["values"]["relations"],"checkpoint_positions":positions}),
    )
}
fn comparable(v: &Value) -> Value {
    let mut first = v["first_decision"].clone();
    for key in ["field", "word_copy"] {
        if let Some(o) = first[key].as_object_mut() {
            o.remove("score");
        }
    }
    json!({"text":v["text"],"eos":v["eos"],"first_decision":first,"initial_relations":v["initial_relations"],"final_relations":v["final_relations"]})
}

#[derive(Clone, Serialize, Deserialize)]
struct Case {
    id: String,
    prompt: String,
    #[serde(default)]
    expected: Option<String>,
    /// Live head of the queried chain; non-null marks an exact target row.
    #[serde(default)]
    current_record: Option<u64>,
    /// Exact ancestor (assertion root) record the answer must come from.
    #[serde(default)]
    target_record: Option<u64>,
    /// Validated link count from the head to the target.
    #[serde(default)]
    depth: Option<u8>,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    value: String,
    /// The named chain is truncated: the exact answer is an abstention.
    #[serde(default)]
    abstain: bool,
    /// Prior turns replayed in the same session before `prompt`, each answered by
    /// the evaluated model itself; empty for a single turn.
    #[serde(default)]
    history: Vec<String>,
    /// Follow-up target: the exact record a previous/current request must keep
    /// after earlier turns (the head's immediate previous record or the head).
    #[serde(default)]
    follow_up: Option<u64>,
    /// Frozen accepted complete answers, authored from the typed intent at preparation
    /// and shared with the native API check; membership only, never derived from output.
    #[serde(default)]
    accepted: Vec<String>,
    /// The target record must be the chain's assertion root (initial intent); false for
    /// an exact previous-record target below a same-value reassertion head.
    #[serde(default = "default_true")]
    target_root: bool,
}
fn default_true() -> bool {
    true
}
/// Disjoint typed partition of the targets: initial + previous + current + abstain.
fn partition(cases: &[Case]) -> Value {
    let mut counts = [0usize; 4];
    for c in cases
        .iter()
        .filter(|c| c.target_record.is_some() || c.abstain || c.follow_up.is_some())
    {
        counts[match intent(c) {
            Intent::Initial => 0,
            Intent::Previous => 1,
            Intent::Current => 2,
            Intent::Abstain => 3,
        }] += 1;
    }
    json!({"initial":counts[0],"previous":counts[1],"current":counts[2],"abstain":counts[3],"total":counts.iter().sum::<usize>()})
}
/// The request's typed intent, from the case's exact labels.
fn intent(c: &Case) -> Intent {
    if c.abstain {
        Intent::Abstain
    } else if let Some(record) = c.follow_up {
        if Some(record) == c.current_record {
            Intent::Current
        } else {
            Intent::Previous
        }
    } else if !c.target_root {
        Intent::Previous
    } else {
        Intent::Initial
    }
}
/// The frozen accepted list, or the same typed list for case files authored before
/// `accepted` existed (a plain request is one whose expected text is the bare value).
fn accepted_answers(c: &Case) -> Vec<String> {
    if !c.accepted.is_empty() {
        return c.accepted.clone();
    }
    let plain = c
        .expected
        .as_ref()
        .is_some_and(|e| *e == format!(" {}.\n", c.value));
    answer_oracle::accepted(intent(c), plain, &c.owner, &c.value)
}
struct Owner {
    name: String,
    values: Vec<String>,
}
const INITIAL_REQUESTS: [&str; 4] = [
    "What was the initial location of {o}?",
    "Where was {o} at first?",
    "What was the first location of {o}?",
    "Where was {o} originally?",
];
const PRESERVE_REQUESTS: [&str; 4] = [
    "What was the previous location of {o}?",
    "Where was {o} before?",
    "Where is {o}?",
    "What is the current location of {o}?",
];
const STYLES: [&str; 3] = ["", "Name the owner first.", "State the owner first."];
fn chain(owner: &Owner, versions: usize, reverse: bool) -> String {
    let mut facts = if reverse {
        format!("{} holds {}.", owner.values[0], owner.name)
    } else {
        format!("{} in {}.", owner.name, owner.values[0])
    };
    for value in &owner.values[1..versions] {
        facts.push_str(&format!(" {} now in {value}.", owner.name));
    }
    facts
}
fn request(template: &str, owner: &str, style: &str) -> String {
    let question = template.replace("{o}", owner);
    if style.is_empty() {
        format!("{question} Answer:")
    } else {
        format!("{question} {style} Answer:")
    }
}
fn answer(style: &str, owner: &str, value: &str) -> String {
    if style.is_empty() {
        format!(" {value}.\n")
    } else {
        format!(" {owner} was in {value}.\n")
    }
}
#[allow(clippy::too_many_arguments)]
fn push_targets(
    cases: &mut Vec<Case>,
    id: &str,
    facts: &str,
    owner: &str,
    initial: &str,
    root: u64,
    head: u64,
    templates: &[&str],
) {
    // A contiguous chain has one validated link per record between head and root.
    push_targets_depth(
        cases,
        id,
        facts,
        owner,
        initial,
        root,
        head,
        (head - root) as u8,
        templates,
    );
}
/// Exact root target with an explicit validated link count: chains interleaved with
/// other owners' records or containing same-value reassertions are not contiguous.
#[allow(clippy::too_many_arguments)]
fn push_targets_depth(
    cases: &mut Vec<Case>,
    id: &str,
    facts: &str,
    owner: &str,
    initial: &str,
    root: u64,
    head: u64,
    depth: u8,
    templates: &[&str],
) {
    for (t, template) in templates.iter().enumerate() {
        for (s, style) in STYLES.iter().enumerate() {
            cases.push(Case {
                id: format!("target/{id}/{t}/{s}"),
                prompt: format!("{facts} {}", request(template, owner, style)),
                expected: Some(answer(style, owner, initial)),
                current_record: Some(head),
                target_record: Some(root),
                depth: Some(depth),
                owner: owner.into(),
                value: initial.into(),
                abstain: false,
                history: Vec::new(),
                follow_up: None,
                accepted: answer_oracle::accepted(
                    Intent::Initial,
                    style.is_empty(),
                    owner,
                    initial,
                ),
                target_root: true,
            });
        }
    }
}
/// Follow-up turns after an initial-version request in the same session: the
/// previous and current requests must keep the frozen parent's immediate-previous
/// and current records whatever the first turn answered, in owner-first styles and
/// the plain style, including a current request after both earlier turns.
#[allow(clippy::too_many_arguments)]
fn push_follow_ups(
    cases: &mut Vec<Case>,
    id: &str,
    facts: &str,
    owner: &str,
    previous_value: &str,
    current_value: &str,
    previous_id: u64,
    head: u64,
    previous_is_target: bool,
) {
    let initial = |s: usize| format!("{facts} {}", request(INITIAL_REQUESTS[0], owner, STYLES[s]));
    let previous_turn = |s: usize| request(PRESERVE_REQUESTS[0], owner, STYLES[s]);
    let case = |name: String,
                prompt: String,
                expected: String,
                history: Vec<String>,
                record: u64,
                value: &str| {
        // The typed intent authors the frozen accepted list: a previous request
        // never accepts a present-tense statement of its old value.
        let plain = !expected.contains(" was in ") && !expected.contains(" is in ");
        let kind = if record == head {
            Intent::Current
        } else {
            Intent::Previous
        };
        // Below a same-value reassertion head the frozen parent cannot read the
        // previous record, so that follow-up is an exact previous-record target for
        // the selector rather than a deferral.
        let target = previous_is_target && record != head;
        Case {
            id: format!("follow/{id}/{name}"),
            prompt,
            expected: Some(expected),
            current_record: Some(head),
            target_record: target.then_some(record),
            depth: target.then_some(1),
            owner: owner.into(),
            value: value.into(),
            abstain: false,
            history,
            follow_up: Some(record),
            accepted: answer_oracle::accepted(kind, plain, owner, value),
            target_root: !target,
        }
    };
    for s in [1usize, 2] {
        cases.push(case(
            format!("previous/{s}"),
            previous_turn(s),
            format!(" {owner} was in {previous_value}.\n"),
            vec![initial(s)],
            previous_id,
            previous_value,
        ));
        cases.push(case(
            format!("current/{s}"),
            request(PRESERVE_REQUESTS[2], owner, STYLES[s]),
            format!(" {owner} is in {current_value}.\n"),
            vec![initial(s)],
            head,
            current_value,
        ));
        cases.push(case(
            format!("current-after-previous/{s}"),
            request(PRESERVE_REQUESTS[3], owner, STYLES[s]),
            format!(" {owner} is in {current_value}.\n"),
            vec![initial(s), previous_turn(s)],
            head,
            current_value,
        ));
    }
    cases.push(case(
        "previous/0".into(),
        previous_turn(0),
        format!(" {previous_value}.\n"),
        vec![initial(0)],
        previous_id,
        previous_value,
    ));
}
/// Exact previous-record targets below a same-value reassertion head: the frozen
/// parent cannot read them, so the learned selector must pick the head's immediate
/// previous record (record-hop semantics) through the head hop.
fn push_previous_targets(
    cases: &mut Vec<Case>,
    id: &str,
    facts: &str,
    owner: &str,
    previous_value: &str,
    record: u64,
    head: u64,
) {
    for (t, template) in PRESERVE_REQUESTS.iter().take(2).enumerate() {
        for (s, style) in STYLES.iter().enumerate() {
            cases.push(Case {
                id: format!("previous-target/{id}/{t}/{s}"),
                prompt: format!("{facts} {}", request(template, owner, style)),
                expected: Some(answer(style, owner, previous_value)),
                current_record: Some(head),
                target_record: Some(record),
                depth: Some(1),
                owner: owner.into(),
                value: previous_value.into(),
                abstain: false,
                history: Vec::new(),
                follow_up: None,
                accepted: answer_oracle::accepted(
                    Intent::Previous,
                    style.is_empty(),
                    owner,
                    previous_value,
                ),
                target_root: false,
            });
        }
    }
}
/// Abstention targets: the named chain's root is proven evicted, so the exact
/// answer is the parent's no-read `Unknown.`; `head` is the chain's live record.
fn push_abstain(
    cases: &mut Vec<Case>,
    id: &str,
    facts: &str,
    owner: &str,
    head: u64,
    templates: &[&str],
) {
    for (t, template) in templates.iter().enumerate() {
        for (s, style) in STYLES.iter().enumerate() {
            cases.push(Case {
                id: format!("{id}/abstain/{t}/{s}"),
                prompt: format!("{facts} {}", request(template, owner, style)),
                expected: Some(" Unknown.\n".into()),
                current_record: Some(head),
                target_record: None,
                depth: None,
                owner: owner.into(),
                value: String::new(),
                abstain: true,
                history: Vec::new(),
                follow_up: None,
                accepted: answer_oracle::accepted(Intent::Abstain, style.is_empty(), owner, ""),
                target_root: true,
            });
        }
    }
}
fn push_preserve(
    cases: &mut Vec<Case>,
    id: &str,
    facts: &str,
    owner: &str,
    requests: &[(usize, usize)],
) {
    for (t, s) in requests {
        cases.push(Case {
            id: format!("preserve/{id}/{t}/{s}"),
            prompt: format!(
                "{facts} {}",
                request(PRESERVE_REQUESTS[*t], owner, STYLES[*s])
            ),
            expected: None,
            current_record: None,
            target_record: None,
            depth: None,
            owner: owner.into(),
            value: String::new(),
            abstain: false,
            history: Vec::new(),
            follow_up: None,
            accepted: Vec::new(),
            target_root: true,
        });
    }
}
/// Deterministic singleton spellings: each occurs in exactly one construction
/// prompt, so it never earns a lexical identity and behaves like a fresh word.
fn singleton(id: &str, t: usize, s: usize) -> String {
    let hash = blake3::hash(format!("absent-owner:{id}:{t}:{s}").as_bytes());
    hash.as_bytes()[..6]
        .iter()
        .map(|b| (b'a' + b % 26) as char)
        .collect()
}
fn push_absent(cases: &mut Vec<Case>, id: &str, facts: &str, absent: &str) {
    for (t, s, known) in [
        (0, 0, true),
        (1, 1, true),
        (0, 0, false),
        (2, 2, false),
        (3, 1, false),
    ] {
        let name = if known {
            absent.to_owned()
        } else {
            singleton(id, t, s)
        };
        cases.push(Case {
            id: format!(
                "absent/{id}/{t}/{s}/{}",
                if known { "known" } else { "singleton" }
            ),
            prompt: format!("{facts} {}", request(INITIAL_REQUESTS[t], &name, STYLES[s])),
            expected: None,
            current_record: None,
            target_record: None,
            depth: None,
            owner: name,
            value: String::new(),
            abstain: false,
            history: Vec::new(),
            follow_up: None,
            accepted: Vec::new(),
            target_root: true,
        });
    }
}
/// Lexeme-ring padding between the facts and the request. Known filler, the
/// retained histories' filler, or a per-prompt singleton filler the dictionary
/// never learns, so unfamiliar words in the request view are ordinary.
/// Unrelated single facts that push a chain's root out of the sixteen-slot ring.
/// Construction uses fixed names; a fresh draw derives its own spellings.
fn unrelated(count: usize, seed: &str) -> Vec<String> {
    const FIXED: [&str; 16] = [
        "arbor", "brook", "cairn", "delta", "ember", "fjord", "glade", "haven", "islet", "jetty",
        "knoll", "lagoon", "marsh", "nadir", "oriel", "pylon",
    ];
    (0..count)
        .map(|k| {
            if seed.is_empty() {
                FIXED[k].to_owned()
            } else {
                blake3::hash(format!("unrelated:{seed}:{k}").as_bytes()).as_bytes()[..5]
                    .iter()
                    .map(|b| (b'a' + b % 26) as char)
                    .collect()
            }
        })
        .collect()
}
fn evicted_facts(chain_facts: &str, count: usize, seed: &str) -> String {
    let mut facts = format!("Record: {chain_facts}");
    for (k, name) in unrelated(count, seed).iter().enumerate() {
        facts.push_str(&format!(" {name} in Zone{k}."));
    }
    facts
}
fn padding(evicted: bool, id: &str) -> String {
    if !evicted {
        return String::new();
    }
    match blake3::hash(id.as_bytes()).as_bytes()[0] % 3 {
        0 => "oak ash elm ".repeat(12),
        1 => "quiet sky. ".repeat(12),
        _ => {
            let word = |k: usize| singleton(id, 20 + k, 0);
            format!("{} {} {} ", word(0), word(1), word(2)).repeat(4)
        }
    }
}
fn authored_cases(owners: &[Owner], filler_seed: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    let absent = &owners[2].name;
    for (i, owner) in owners.iter().take(2).enumerate() {
        for versions in [2, 3, 4] {
            for reverse in [false, true] {
                for evicted in [false, true] {
                    let id = format!("single/{i}/{versions}/{reverse}/{evicted}");
                    let facts = format!(
                        "Record: {} {}",
                        chain(owner, versions, reverse),
                        padding(evicted, &id)
                    )
                    .trim_end()
                    .to_owned();
                    push_targets(
                        &mut cases,
                        &id,
                        &facts,
                        &owner.name,
                        &owner.values[0],
                        1,
                        versions as u64,
                        &INITIAL_REQUESTS,
                    );
                    if !evicted && versions >= 3 {
                        push_follow_ups(
                            &mut cases,
                            &id,
                            &facts,
                            &owner.name,
                            &owner.values[versions - 2],
                            &owner.values[versions - 1],
                            versions as u64 - 1,
                            versions as u64,
                            false,
                        );
                    }
                    push_preserve(
                        &mut cases,
                        &id,
                        &facts,
                        &owner.name,
                        &[
                            (0, 0),
                            (0, 1),
                            (1, 0),
                            (1, 2),
                            (2, 0),
                            (2, 1),
                            (3, 0),
                            (3, 1),
                        ],
                    );
                    push_absent(&mut cases, &id, &facts, absent);
                }
            }
        }
    }
    // Two competing chains in both insertion orders; each root must be reached
    // through its own head only.
    for swap in [false, true] {
        let order = if swap { [1, 0] } else { [0, 1] };
        for versions in [2, 3] {
            for evicted in [false, true] {
                let id = format!("owners/{swap}/{versions}/{evicted}");
                let facts = format!(
                    "Record: {} {} {}",
                    chain(&owners[order[0]], versions, false),
                    chain(&owners[order[1]], versions, true),
                    padding(evicted, &id)
                )
                .trim_end()
                .to_owned();
                for i in 0..2 {
                    let (root, head) = if i == order[0] {
                        (1, versions as u64)
                    } else {
                        (versions as u64 + 1, 2 * versions as u64)
                    };
                    push_targets(
                        &mut cases,
                        &format!("{id}/{i}"),
                        &facts,
                        &owners[i].name,
                        &owners[i].values[0],
                        root,
                        head,
                        &[INITIAL_REQUESTS[0], INITIAL_REQUESTS[2]],
                    );
                    if versions == 3 && !evicted {
                        push_follow_ups(
                            &mut cases,
                            &format!("{id}/{i}"),
                            &facts,
                            &owners[i].name,
                            &owners[i].values[1],
                            &owners[i].values[2],
                            head - 1,
                            head,
                            false,
                        );
                    }
                    push_preserve(
                        &mut cases,
                        &format!("{id}/{i}"),
                        &facts,
                        &owners[i].name,
                        &[(0, 1), (2, 0), (3, 1)],
                    );
                }
                push_absent(&mut cases, &id, &facts, absent);
            }
        }
    }
    // Competing chains of different lengths, each root reached only through its
    // own named head; absent owners under current, previous and initial phrasing.
    for (versions_a, versions_b) in [(2, 3), (4, 2), (3, 4)] {
        for swap in [false, true] {
            let order = if swap { [1, 0] } else { [0, 1] };
            let counts = [versions_a, versions_b];
            let id = format!("mixed/{versions_a}/{versions_b}/{swap}");
            let facts = format!(
                "Record: {} {} {}",
                chain(&owners[order[0]], counts[0], false),
                chain(&owners[order[1]], counts[1], true),
                padding(swap, &id)
            )
            .trim_end()
            .to_owned();
            let first = counts[0] as u64;
            for i in 0..2 {
                let (root, head) = if i == order[0] {
                    (1, first)
                } else {
                    (first + 1, first + counts[1] as u64)
                };
                push_targets(
                    &mut cases,
                    &format!("{id}/{i}"),
                    &facts,
                    &owners[i].name,
                    &owners[i].values[0],
                    root,
                    head,
                    &[INITIAL_REQUESTS[0], INITIAL_REQUESTS[1]],
                );
                push_preserve(
                    &mut cases,
                    &format!("{id}/{i}"),
                    &facts,
                    &owners[i].name,
                    &[(0, 0), (0, 1), (1, 0), (2, 0), (2, 1), (3, 0), (3, 1)],
                );
            }
            for (k, template) in PRESERVE_REQUESTS.iter().enumerate() {
                for (s, style) in STYLES.iter().enumerate().take(2) {
                    let name = singleton(&id, 10 + k, s);
                    cases.push(Case {
                        id: format!("absent/{id}/preserve/{k}/{s}/singleton"),
                        prompt: format!("{facts} {}", request(template, &name, style)),
                        expected: None,
                        current_record: None,
                        target_record: None,
                        depth: None,
                        owner: name,
                        value: String::new(),
                        abstain: false,
                        history: Vec::new(),
                        follow_up: None,
                        accepted: Vec::new(),
                        target_root: true,
                    });
                }
            }
            push_absent(&mut cases, &id, &facts, absent);
        }
    }
    // Re-asserted then revised chains across interleaved facts, as in the retained
    // multi-turn histories: ordinary current requests must keep deferring, with
    // and without the histories' filler before the request.
    for (i, owner) in owners.iter().take(2).enumerate() {
        let other = &owners[1 - i];
        let (v0, v1) = (&owner.values[0], &owner.values[1]);
        let o = &owner.name;
        let layouts = [
            format!(
                "{o} in {v0}. {} in {}. {o} in {v0}. {o} now in {v1}.",
                other.name, other.values[0]
            ),
            format!(
                "{o} in {v0}. {} in {}. {o} now in {v1}.",
                other.name, other.values[0]
            ),
            format!("Record: {o} in {v0}. {o} in {v0}. {o} now in {v1}."),
            format!(
                "{o} in {v0}. {o} now in {v1}. {} in {}.",
                other.name, other.values[0]
            ),
        ];
        for (k, base) in layouts.iter().enumerate() {
            for filler in [false, true] {
                let id = format!("reassert/{i}/{k}/{filler}");
                let facts = format!("{base} {}", padding(filler, &id))
                    .trim_end()
                    .to_owned();
                push_preserve(
                    &mut cases,
                    &id,
                    &facts,
                    o,
                    &[(2, 0), (2, 1), (3, 0), (0, 0), (1, 0)],
                );
                for (s, style) in STYLES.iter().enumerate() {
                    let question = if style.is_empty() {
                        format!("Where is {o}? Explain in a sentence. Answer:")
                    } else {
                        format!("Explain in a sentence. Where is {o}? {style} Answer:")
                    };
                    cases.push(Case {
                        id: format!("preserve/{id}/explain/{s}"),
                        prompt: format!("{facts} {question}"),
                        expected: None,
                        current_record: None,
                        target_record: None,
                        depth: None,
                        owner: o.clone(),
                        value: String::new(),
                        abstain: false,
                        history: Vec::new(),
                        follow_up: None,
                        accepted: Vec::new(),
                        target_root: true,
                    });
                }
            }
        }
    }
    // Root evicted from the sixteen-slot ring by later unrelated facts: no genuine
    // root survives, so an initial request must abstain instead of answering with
    // the oldest survivor, while previous/current requests keep the parent's answer.
    // Fourteen unrelated facts evict the root; fifteen also evict the previous version.
    for (i, owner) in owners.iter().take(2).enumerate() {
        for reverse in [false, true] {
            for trailing in [14, 15] {
                let facts = evicted_facts(&chain(owner, 3, reverse), trailing, filler_seed);
                let id = format!("evicted-root/{i}/{reverse}/{trailing}");
                for (t, template) in INITIAL_REQUESTS.iter().enumerate() {
                    for (s, style) in STYLES.iter().enumerate() {
                        cases.push(Case {
                            id: format!("{id}/initial/{t}/{s}"),
                            prompt: format!("{facts} {}", request(template, &owner.name, style)),
                            expected: Some(" Unknown.\n".into()),
                            current_record: Some(3),
                            target_record: None,
                            depth: None,
                            owner: owner.name.clone(),
                            value: String::new(),
                            abstain: true,
                            history: Vec::new(),
                            follow_up: None,
                            accepted: answer_oracle::accepted(
                                Intent::Abstain,
                                style.is_empty(),
                                &owner.name,
                                "",
                            ),
                            target_root: true,
                        });
                    }
                }
                push_preserve(
                    &mut cases,
                    &id,
                    &facts,
                    &owner.name,
                    &[(0, 0), (0, 1), (1, 0), (2, 0), (2, 1), (3, 0)],
                );
                if trailing == 14 {
                    // After an abstained initial turn the previous and current
                    // versions stay readable in the same session.
                    push_follow_ups(
                        &mut cases,
                        &id,
                        &facts,
                        &owner.name,
                        &owner.values[1],
                        &owner.values[2],
                        2,
                        3,
                        false,
                    );
                }
            }
        }
    }
    // One truncated and one complete chain in the same record ring: the named
    // chain decides between abstention and the exact root.
    for swap in [false, true] {
        let order = if swap { [1, 0] } else { [0, 1] };
        let (a, b) = (&owners[order[0]], &owners[order[1]]);
        // Six chain records plus eleven unrelated facts: the seventeenth record
        // overwrites the first chain's root while the second chain stays complete.
        let facts = evicted_facts(
            &format!("{} {}", chain(a, 3, false), chain(b, 3, true)),
            11,
            filler_seed,
        );
        let id = format!("evicted-pair/{swap}");
        for (t, template) in [INITIAL_REQUESTS[0], INITIAL_REQUESTS[2]]
            .iter()
            .enumerate()
        {
            for (s, style) in STYLES.iter().enumerate() {
                cases.push(Case {
                    id: format!("{id}/truncated/{t}/{s}"),
                    prompt: format!("{facts} {}", request(template, &a.name, style)),
                    expected: Some(" Unknown.\n".into()),
                    current_record: Some(3),
                    target_record: None,
                    depth: None,
                    owner: a.name.clone(),
                    value: String::new(),
                    abstain: true,
                    history: Vec::new(),
                    follow_up: None,
                    accepted: answer_oracle::accepted(
                        Intent::Abstain,
                        style.is_empty(),
                        &a.name,
                        "",
                    ),
                    target_root: true,
                });
            }
        }
        push_targets(
            &mut cases,
            &format!("{id}/complete"),
            &facts,
            &b.name,
            &b.values[0],
            4,
            6,
            &[INITIAL_REQUESTS[0], INITIAL_REQUESTS[2]],
        );
        push_preserve(
            &mut cases,
            &format!("{id}/a"),
            &facts,
            &a.name,
            &[(0, 0), (2, 0), (3, 1)],
        );
        push_preserve(
            &mut cases,
            &format!("{id}/b"),
            &facts,
            &b.name,
            &[(0, 0), (2, 0), (3, 1)],
        );
    }
    // Equal spelling across distinct versions: the root occurrence, not the
    // current one, must be selected.
    for (i, owner) in owners.iter().take(2).enumerate() {
        for reverse in [false, true] {
            let mut same = Owner {
                name: owner.name.clone(),
                values: vec![
                    owner.values[0].clone(),
                    owner.values[1].clone(),
                    owner.values[0].clone(),
                ],
            };
            same.values.truncate(3);
            let facts = format!("Record: {}", chain(&same, 3, reverse));
            let id = format!("same-spelling/{i}/{reverse}");
            push_targets(
                &mut cases,
                &id,
                &facts,
                &owner.name,
                &owner.values[0],
                1,
                3,
                &[INITIAL_REQUESTS[0]],
            );
            push_preserve(&mut cases, &id, &facts, &owner.name, &[(0, 1), (2, 0)]);
        }
    }
    // Structural negatives that preserve the complete parent: one record,
    // conflicting assertions, and the intent words used in data roles.
    for (i, owner) in owners.iter().take(2).enumerate() {
        let single = format!("Record: {} in {}.", owner.name, owner.values[0]);
        for (s, style) in STYLES.iter().enumerate() {
            cases.push(Case {
                id: format!("one-record/{i}/{s}"),
                prompt: format!(
                    "{single} {}",
                    request(INITIAL_REQUESTS[0], &owner.name, style)
                ),
                expected: None,
                current_record: None,
                target_record: None,
                depth: None,
                owner: owner.name.clone(),
                value: String::new(),
                abstain: false,
                history: Vec::new(),
                follow_up: None,
                accepted: Vec::new(),
                target_root: true,
            });
        }
        let conflict = format!(
            "Record: {} in {}. {} in {}.",
            owner.name, owner.values[0], owner.name, owner.values[1]
        );
        for s in [0, 1] {
            cases.push(Case {
                id: format!("conflict/{i}/{s}"),
                prompt: format!(
                    "{conflict} {}",
                    request(INITIAL_REQUESTS[0], &owner.name, STYLES[s])
                ),
                expected: None,
                current_record: None,
                target_record: None,
                depth: None,
                owner: owner.name.clone(),
                value: String::new(),
                abstain: false,
                history: Vec::new(),
                follow_up: None,
                accepted: Vec::new(),
                target_root: true,
            });
        }
        let (v0, v1, v2) = (&owner.values[0], &owner.values[1], &owner.values[2]);
        let o = &owner.name;
        for (k, prompt) in [
            format!("{o} in initial. initial in {v1}. Where is the location of {o}? Answer:"),
            format!("Record: first in {v0}. first now in {v1}. What was the previous location of first? Name the owner first. Answer:"),
            format!("Record: {o} in {v0}. {o} now in {v1}. {o} now in {v2}. Where is initial? Answer:"),
            format!("Record: {o} in {v0}. {o} now in {v1}. originally in {v2}. Where is {o}? Answer:"),
        ]
        .iter()
        .enumerate()
        {
            cases.push(Case { id: format!("literal-role/{i}/{k}"), prompt: prompt.clone(), expected: None, current_record: None, target_record: None, depth: None, owner: o.clone(), value: String::new(), abstain: false, history: Vec::new(), follow_up: None, accepted: Vec::new(), target_root: true });
        }
        // The intent word as an owner name: its own root is still the exact answer.
        let facts = format!("Record: initial in {v0}. initial now in {v1}. initial now in {v2}.");
        push_targets(
            &mut cases,
            &format!("literal-owner/{i}"),
            &facts,
            "initial",
            v0,
            1,
            3,
            &[INITIAL_REQUESTS[0]],
        );
        push_preserve(
            &mut cases,
            &format!("literal-owner/{i}"),
            &facts,
            "initial",
            &[(0, 1)],
        );
    }
    // Resident same-value reassertions are validated links, never truncations: an
    // initial request must reach the exact assertion root through them, at the root,
    // in the middle, repeated, interleaved with another owner, and beside an equal
    // value owned by someone else. Depth counts validated record links.
    for (i, owner) in owners.iter().take(2).enumerate() {
        let other = &owners[1 - i];
        let (o, v0, v1, v2) = (
            &owner.name,
            &owner.values[0],
            &owner.values[1],
            &owner.values[2],
        );
        let (p, w0) = (&other.name, &other.values[0]);
        let layouts = [
            (
                "root",
                format!("Record: {o} in {v0}. {o} in {v0}. {o} now in {v1}."),
                1,
                3,
                2,
            ),
            (
                "root-long",
                format!("Record: {o} in {v0}. {o} in {v0}. {o} now in {v1}. {o} now in {v2}."),
                1,
                4,
                3,
            ),
            (
                "middle",
                format!("Record: {o} in {v0}. {o} now in {v1}. {o} in {v1}. {o} now in {v2}."),
                1,
                4,
                3,
            ),
            (
                "multiple",
                format!("Record: {o} in {v0}. {o} in {v0}. {o} in {v0}. {o} now in {v1}."),
                1,
                4,
                3,
            ),
            (
                "interleaved",
                format!("Record: {o} in {v0}. {p} in {w0}. {o} in {v0}. {o} now in {v1}."),
                1,
                4,
                2,
            ),
            (
                "equal-valued-owner",
                format!("Record: {o} in {v0}. {p} in {v0}. {o} in {v0}. {o} now in {v1}."),
                1,
                4,
                2,
            ),
        ];
        for (name, facts, root, head, depth) in layouts {
            let id = format!("reassert-link/{i}/{name}");
            push_targets_depth(
                &mut cases,
                &id,
                &facts,
                o,
                v0,
                root,
                head,
                depth,
                &INITIAL_REQUESTS,
            );
            if name == "root" {
                push_follow_ups(&mut cases, &id, &facts, o, v0, v1, 2, 3, false);
            } else if name == "middle" {
                push_follow_ups(&mut cases, &id, &facts, o, v1, v2, 3, 4, false);
            }
            push_preserve(
                &mut cases,
                &id,
                &facts,
                o,
                &[(0, 0), (0, 1), (1, 0), (2, 0), (2, 1), (3, 1)],
            );
        }
        // Equal values under distinct owners are distinct records with their own roots.
        let facts = format!("Record: {o} in {v0}. {p} in {v0}. {o} now in {v1}. {p} now in {v1}.");
        let two = [INITIAL_REQUESTS[0], INITIAL_REQUESTS[2]];
        push_targets_depth(
            &mut cases,
            &format!("equal-valued/{i}/a"),
            &facts,
            o,
            v0,
            1,
            3,
            1,
            &two,
        );
        push_targets_depth(
            &mut cases,
            &format!("equal-valued/{i}/b"),
            &facts,
            p,
            v0,
            2,
            4,
            1,
            &two,
        );
        push_preserve(
            &mut cases,
            &format!("equal-valued/{i}"),
            &facts,
            o,
            &[(0, 0), (2, 1)],
        );
        // A head that is itself a same-value reassertion is answered under the head
        // contract (the head-reassert families below); only its current request is
        // kept here as a parent-preserved row.
        let facts = format!("Record: {o} in {v0}. {o} now in {v1}. {o} in {v1}.");
        push_preserve(
            &mut cases,
            &format!("reassert-head/{i}"),
            &facts,
            o,
            &[(2, 0)],
        );
    }
    // Reassertion chains at the ring boundary: twelve trailing facts keep the
    // four-record chain's root resident; thirteen evict the root while its
    // reassertion survives; fourteen evict the reassertion as well. Only proven
    // eviction abstains; the surviving reassertion is not a root.
    for (i, owner) in owners.iter().take(2).enumerate() {
        let (o, v0, v1, v2) = (
            &owner.name,
            &owner.values[0],
            &owner.values[1],
            &owner.values[2],
        );
        let chain_facts = format!("{o} in {v0}. {o} in {v0}. {o} now in {v1}. {o} now in {v2}.");
        let two = [INITIAL_REQUESTS[0], INITIAL_REQUESTS[2]];
        for trailing in [12, 13, 14] {
            let facts = evicted_facts(&chain_facts, trailing, filler_seed);
            let id = format!("reassert-boundary/{i}/{trailing}");
            if trailing == 12 {
                push_targets_depth(&mut cases, &id, &facts, o, v0, 1, 4, 3, &two);
            } else {
                push_abstain(&mut cases, &id, &facts, o, 4, &two);
                if trailing == 13 {
                    push_follow_ups(&mut cases, &id, &facts, o, v1, v2, 3, 4, false);
                }
            }
            push_preserve(&mut cases, &id, &facts, o, &[(0, 0), (2, 0), (3, 1)]);
        }
    }
    // Live heads that are themselves same-value reassertions: repeating the current
    // fact must keep the retained history readable. Record-hop semantics: initial is
    // the root through the head hop; previous is the head's immediate previous record
    // (the record the head repeats); current stays the parent's answer.
    for (i, owner) in owners.iter().take(2).enumerate() {
        let other = &owners[1 - i];
        let (o, v0, v1, v2) = (
            &owner.name,
            &owner.values[0],
            &owner.values[1],
            &owner.values[2],
        );
        let (p, w0) = (&other.name, &other.values[0]);
        // (layout, facts, root, head, root depth, previous record, previous value)
        let layouts = [
            (
                "head",
                format!("Record: {o} in {v0}. {o} now in {v1}. {o} in {v1}."),
                1,
                3,
                2,
                2,
                v1,
            ),
            (
                "head-long",
                format!("Record: {o} in {v0}. {o} now in {v1}. {o} now in {v2}. {o} in {v2}."),
                1,
                4,
                3,
                3,
                v2,
            ),
            (
                "head-and-root",
                format!("Record: {o} in {v0}. {o} in {v0}. {o} now in {v1}. {o} in {v1}."),
                1,
                4,
                3,
                3,
                v1,
            ),
            (
                "head-twice",
                format!("Record: {o} in {v0}. {o} now in {v1}. {o} in {v1}. {o} in {v1}."),
                1,
                4,
                3,
                3,
                v1,
            ),
            (
                "head-interleaved",
                format!("Record: {o} in {v0}. {p} in {w0}. {o} now in {v1}. {o} in {v1}."),
                1,
                4,
                2,
                3,
                v1,
            ),
        ];
        for (name, facts, root, head, depth, previous_id, previous_value) in layouts {
            let id = format!("head-reassert/{i}/{name}");
            push_targets_depth(
                &mut cases,
                &id,
                &facts,
                o,
                v0,
                root,
                head,
                depth,
                &INITIAL_REQUESTS,
            );
            push_previous_targets(
                &mut cases,
                &id,
                &facts,
                o,
                previous_value,
                previous_id,
                head,
            );
            push_preserve(
                &mut cases,
                &id,
                &facts,
                o,
                &[(2, 0), (2, 1), (3, 0), (3, 1)],
            );
            if name == "head" || name == "head-long" {
                push_follow_ups(
                    &mut cases,
                    &id,
                    &facts,
                    o,
                    previous_value,
                    previous_value,
                    previous_id,
                    head,
                    true,
                );
            }
        }
        // Boundary: thirteen trailing facts keep the head chain resident (initial and
        // previous answer); fourteen evict the root (initial abstains, previous still
        // reads record 2); fifteen evict the previous record as well (both abstain).
        let chain_facts = format!("{o} in {v0}. {o} now in {v1}. {o} in {v1}.");
        let two = [INITIAL_REQUESTS[0], INITIAL_REQUESTS[2]];
        for trailing in [13, 14, 15] {
            let facts = evicted_facts(&chain_facts, trailing, filler_seed);
            let id = format!("head-boundary/{i}/{trailing}");
            if trailing == 13 {
                push_targets_depth(&mut cases, &id, &facts, o, v0, 1, 3, 2, &two);
            } else {
                push_abstain(&mut cases, &id, &facts, o, 3, &two);
            }
            if trailing < 15 {
                push_previous_targets(&mut cases, &id, &facts, o, v1, 2, 3);
            } else {
                for (t, template) in PRESERVE_REQUESTS.iter().take(1).enumerate() {
                    for (s, style) in STYLES.iter().enumerate() {
                        cases.push(Case {
                            id: format!("{id}/previous-abstain/{t}/{s}"),
                            prompt: format!("{facts} {}", request(template, o, style)),
                            expected: Some(" Unknown.\n".into()),
                            current_record: Some(3),
                            target_record: None,
                            depth: None,
                            owner: o.clone(),
                            value: String::new(),
                            abstain: true,
                            history: Vec::new(),
                            follow_up: None,
                            accepted: answer_oracle::accepted(
                                Intent::Abstain,
                                style.is_empty(),
                                o,
                                "",
                            ),
                            target_root: true,
                        });
                    }
                }
            }
            push_preserve(&mut cases, &id, &facts, o, &[(2, 0), (3, 1)]);
        }
    }
    // Identical assertions only: the smallest repeated-head chains. Under record-hop
    // semantics initial and previous both name exact records even when every value
    // is equal: for two records, previous is record 1 (the root reached through the
    // head hop, chain class seven); for three, previous is record 2 and initial is
    // record 1 two links down. Crossed with the current intent (parent-preserved),
    // a competing owner between the repeats, all styles, the ring boundary and
    // actual follow-up turns.
    for (i, owner) in owners.iter().take(2).enumerate() {
        let other = &owners[1 - i];
        let (o, v0) = (&owner.name, &owner.values[0]);
        let (p, w0) = (&other.name, &other.values[0]);
        // (layout, facts, root, head, root depth, previous record)
        let layouts = [
            (
                "two",
                format!("Record: {o} in {v0}. {o} in {v0}."),
                1,
                2,
                1,
                1,
            ),
            (
                "three",
                format!("Record: {o} in {v0}. {o} in {v0}. {o} in {v0}."),
                1,
                3,
                2,
                2,
            ),
            (
                "two-competing",
                format!("Record: {o} in {v0}. {p} in {w0}. {o} in {v0}."),
                1,
                3,
                1,
                1,
            ),
            (
                "three-competing",
                format!("Record: {o} in {v0}. {o} in {v0}. {p} in {w0}. {o} in {v0}."),
                1,
                4,
                2,
                2,
            ),
        ];
        for (name, facts, root, head, depth, previous_id) in layouts {
            let id = format!("same-only/{i}/{name}");
            push_targets_depth(
                &mut cases,
                &id,
                &facts,
                o,
                v0,
                root,
                head,
                depth,
                &INITIAL_REQUESTS,
            );
            push_previous_targets(&mut cases, &id, &facts, o, v0, previous_id, head);
            push_preserve(
                &mut cases,
                &id,
                &facts,
                o,
                &[(2, 0), (2, 1), (3, 0), (3, 1)],
            );
            if name == "two" || name == "three" {
                push_follow_ups(&mut cases, &id, &facts, o, v0, v0, previous_id, head, true);
            }
        }
        // The retained matched histories repeat a fact in a later turn and then ask a
        // current question with the explanatory phrasing; the live head, not the
        // root, must remain the source. Single-turn and actual two-turn forms, with
        // and without a competing owner, in both fact orders.
        let explain = [
            format!("Where is {o}? Explain in a sentence. Answer:"),
            format!("Explain in a sentence. Where is {o}? Answer:"),
            format!("Where is {o}? Explain in a sentence. Name the owner first. Answer:"),
        ];
        let openers = [
            (
                format!("{o} in {v0}. {p} in {w0}."),
                format!("{o} in {v0}."),
            ),
            (
                format!("Record: {v0} holds {o}. Record: {w0} holds {p}."),
                format!("Record: {v0} holds {o}."),
            ),
            (format!("{o} in {v0}."), format!("{o} in {v0}.")),
        ];
        for (k, (first, repeat)) in openers.iter().enumerate() {
            for (q, question) in explain.iter().enumerate() {
                let inherit = |id: String, prompt: String, history: Vec<String>| Case {
                    id,
                    prompt,
                    expected: None,
                    current_record: None,
                    target_record: None,
                    depth: None,
                    owner: o.clone(),
                    value: String::new(),
                    abstain: false,
                    history,
                    follow_up: None,
                    accepted: Vec::new(),
                    target_root: true,
                };
                cases.push(inherit(
                    format!("same-only-explain/{i}/{k}/{q}/single"),
                    format!("{first} {repeat} {question}"),
                    Vec::new(),
                ));
                cases.push(inherit(
                    format!("same-only-explain/{i}/{k}/{q}/turn"),
                    format!("{repeat} {question}"),
                    vec![format!("{first} Where is {o}? Answer:")],
                ));
            }
        }
        // Boundary for the two-record chain: fourteen trailing facts keep both records
        // resident; fifteen evict the root, so the head's predecessor is proven absent
        // and initial and previous both abstain.
        let chain_facts = format!("{o} in {v0}. {o} in {v0}.");
        let two = [INITIAL_REQUESTS[0], INITIAL_REQUESTS[2]];
        for trailing in [14, 15] {
            let facts = evicted_facts(&chain_facts, trailing, filler_seed);
            let id = format!("same-only-boundary/{i}/{trailing}");
            if trailing == 14 {
                push_targets_depth(&mut cases, &id, &facts, o, v0, 1, 2, 1, &two);
                push_previous_targets(&mut cases, &id, &facts, o, v0, 1, 2);
            } else {
                push_abstain(&mut cases, &id, &facts, o, 2, &two);
                for (t, template) in PRESERVE_REQUESTS.iter().take(1).enumerate() {
                    for (s, style) in STYLES.iter().enumerate() {
                        cases.push(Case {
                            id: format!("{id}/previous-abstain/{t}/{s}"),
                            prompt: format!("{facts} {}", request(template, o, style)),
                            expected: Some(" Unknown.\n".into()),
                            current_record: Some(2),
                            target_record: None,
                            depth: None,
                            owner: o.clone(),
                            value: String::new(),
                            abstain: true,
                            history: Vec::new(),
                            follow_up: None,
                            accepted: answer_oracle::accepted(
                                Intent::Abstain,
                                style.is_empty(),
                                o,
                                "",
                            ),
                            target_root: true,
                        });
                    }
                }
            }
            push_preserve(&mut cases, &id, &facts, o, &[(2, 0), (3, 1)]);
        }
    }
    // Thirteen trailing facts after a three-version chain leave its root resident:
    // the exact root answer, not an abstention, at the boundary.
    for (i, owner) in owners.iter().take(2).enumerate() {
        for reverse in [false, true] {
            let facts = evicted_facts(&chain(owner, 3, reverse), 13, filler_seed);
            let id = format!("evicted-root/{i}/{reverse}/13");
            push_targets_depth(
                &mut cases,
                &id,
                &facts,
                &owner.name,
                &owner.values[0],
                1,
                3,
                2,
                &[INITIAL_REQUESTS[0], INITIAL_REQUESTS[2]],
            );
            push_preserve(&mut cases, &id, &facts, &owner.name, &[(0, 0), (2, 0)]);
        }
    }
    cases
}
fn prepare(out: &Path, fresh: bool) -> Result<()> {
    // Fail on an existing destination: a draw is never silently overwritten.
    uor_r4_core::report_output::claim(out)?;
    let mut entropy_tail: Vec<u8> = Vec::new();
    let owners: Vec<Owner> = if fresh {
        let mut entropy = [0_u8; 160];
        fs::File::open("/dev/urandom")?.read_exact(&mut entropy)?;
        let hex: String = entropy.iter().map(|b| format!("{b:02x}")).collect();
        save(
            &out.join("draw.json"),
            &json!({"source":"/dev/urandom","bytes":160,"entropy_hex":hex,"blake3":blake3::hash(&entropy).to_hex().to_string(),"at":format!("{:?}",std::time::SystemTime::now()),"redraw":false,"scope":"Fresh spelling draw; caller must invoke only after artifact selection. No fit is performed."}),
        )?;
        let mut result = Vec::new();
        for bytes in entropy[..120].chunks_exact(40) {
            let word = |start: usize, n: usize| {
                bytes[start..start + n]
                    .iter()
                    .map(|b| (b'a' + b % 26) as char)
                    .collect::<String>()
            };
            result.push(Owner {
                name: word(0, 5),
                values: (0..4)
                    .map(|k| format!("{} {}", word(5 + 8 * k, 4), word(9 + 8 * k, 4)))
                    .collect(),
            });
        }
        let names: Vec<_> = result.iter().map(|o| o.name.clone()).collect();
        let mut values: Vec<_> = result.iter().flat_map(|o| o.values.clone()).collect();
        values.sort();
        values.dedup();
        entropy_tail = entropy[120..].to_vec();
        if names[0] == names[1]
            || names[0] == names[2]
            || names[1] == names[2]
            || values.len() != 12
        {
            return Err(
                "recorded fresh spelling collision; retain draw and do not silently redraw".into(),
            );
        }
        result
    } else {
        vec![
            Owner {
                name: "selvi".into(),
                values: ["Dusk Ridge", "Copper Vale", "Amber Field", "Silver Cove"]
                    .map(String::from)
                    .to_vec(),
            },
            Owner {
                name: "tilva".into(),
                values: ["moss dale", "Birch Grove", "Pine Hollow", "Cedar Point"]
                    .map(String::from)
                    .to_vec(),
            },
            Owner {
                name: "merli".into(),
                values: Vec::new(),
            },
        ]
    };
    let filler_seed = if fresh {
        blake3::hash(&entropy_tail).to_hex().to_string()
    } else {
        String::new()
    };
    let authored = authored_cases(&owners, &filler_seed);
    let mut cases = authored.clone();
    let mut sources = Vec::new();
    let mut duplicate_receipts = Vec::new();
    if !fresh {
        let mut seen: BTreeMap<String, usize> = cases
            .iter()
            .enumerate()
            .map(|(i, c)| (c.prompt.clone(), i))
            .collect();
        for (group, path) in [
            ("handoff-balanced", "/Users/casey.allard/Documents/Codex/2026-09-09/uor-r4-current-query-handoff/balanced-roles/cases.json"),
            ("query-owner", "/Users/casey.allard/Documents/Codex/2026-09-09/uor-r4-query-owner-selection/fresh-data/cases.json"),
            ("historical-transfer", "/Users/casey.allard/Documents/Codex/2026-09-09/uor-r4-reverse-start-transfer/fresh-data/cases.json"),
        ] {
            let bytes = fs::read(path)?;
            let prior: Vec<Value> = serde_json::from_slice(&bytes)?;
            sources.push(json!({"group":group,"path":path,"bytes":bytes.len(),"blake3":blake3::hash(&bytes).to_hex().to_string(),"rows":prior.len(),"treatment":"defer to the complete frozen parent; prior task labels are not imported"}));
            for c in prior {
                let prompt = c["prompt"].as_str().ok_or("prior prompt absent")?.to_owned();
                let id = c["id"].as_str().ok_or("prior id absent")?.to_owned();
                if let Some(&index) = seen.get(&prompt) {
                    duplicate_receipts.push(json!({"source":path,"id":id,"kept":cases[index].id,"explicit_target":cases[index].target_record.is_some()}));
                    continue;
                }
                seen.insert(prompt.clone(), cases.len());
                cases.push(Case { id: format!("{group}/{id}"), prompt, expected: None, current_record: None, target_record: None, depth: None, owner: String::new(), value: String::new(), abstain: false, history: Vec::new(), follow_up: None, accepted: Vec::new(), target_root: true });
            }
        }
    }
    let docs: Vec<_> = cases
        .iter()
        .map(|c| HistoricalVersionExample {
            id: c.id.clone(),
            prompt: c.prompt.clone(),
            target_record: c.target_record,
            inherit: c.target_record.is_none() && !c.abstain,
            abstain: c.abstain,
            history: c.history.clone(),
        })
        .collect();
    save(&out.join("cases.json"), &cases)?;
    save(&out.join("authored-cases.json"), &authored)?;
    if !fresh {
        save(&out.join("training.json"), &docs)?;
    }
    let depths: BTreeMap<u8, usize> =
        cases
            .iter()
            .filter_map(|c| c.depth)
            .fold(BTreeMap::new(), |mut m, d| {
                *m.entry(d).or_default() += 1;
                m
            });
    save(
        &out.join("receipt.json"),
        &json!({"schema":"uor-r4.historical-version-preparation/1","fresh":fresh,"owners":owners.iter().map(|o| json!({"name":o.name,"values":o.values})).collect::<Vec<_>>(),"cases":cases.len(),"authored":authored.len(),"targets":cases.iter().filter(|c|c.target_record.is_some()).count(),"abstain_targets":cases.iter().filter(|c|c.abstain).count(),"follow_up_targets":cases.iter().filter(|c|c.follow_up.is_some()).count(),"previous_targets":cases.iter().filter(|c|!c.target_root&&c.target_record.is_some()).count(),"targets_by_intent":partition(&cases),"count_scope":"targets_by_intent is the disjoint typed partition of all targets (initial + previous + current + abstain); follow_up_targets and previous_targets are cross-cutting categories that overlap it","targets_by_depth":depths,"inherited":cases.iter().filter(|c|c.target_record.is_none()).count(),"sources":sources,"duplicate_receipts":duplicate_receipts,"labels_offline_only":true,"training_written":!fresh,"initial_request_forms":INITIAL_REQUESTS,"scope":"Authored exact ancestor IDs; fresh spellings in unchanged authored forms do not establish general prose or temporal language."}),
    )?;
    uor_r4_core::report_output::seal(out)?;
    Ok(())
}
fn atom(v: &Value) -> Option<String> {
    let n = usize::try_from(v.get("len")?.as_u64()?).ok()?;
    let bytes = v
        .get("bytes")?
        .as_array()?
        .get(..n)?
        .iter()
        .map(|b| u8::try_from(b.as_u64()?).ok())
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}
fn record_value(record: &Value) -> Option<String> {
    atom(if record["span"].is_object() {
        &record["span"]
    } else {
        &record["value"]
    })
}
/// The exact root record must be an assertion root with the authored owner/value,
/// reachable from the live head through `depth` validated revision links.
fn target_record<'a>(c: &Case, actual: &'a Value) -> Option<&'a Value> {
    let (root, head, depth) = (c.target_record?, c.current_record?, c.depth?);
    let relations = &actual["initial_relations"];
    let records = relations["records"].as_array()?;
    let directory = relations["directory"].as_array()?;
    if !directory.contains(&json!(head)) || directory.contains(&json!(root)) {
        return None;
    }
    let mut cursor = records.iter().find(|r| r["id"].as_u64() == Some(head))?;
    for hop in 0..depth {
        if cursor["conflict"] != false || atom(&cursor["owner"])? != c.owner {
            return None;
        }
        let previous = cursor["previous"].as_u64()?;
        if previous == 0 || previous >= cursor["id"].as_u64()? {
            return None;
        }
        let older = records
            .iter()
            .find(|r| r["id"].as_u64() == Some(previous))?;
        // A link is an explicit revision or a resident same-owner, same-value,
        // nonconflicting reassertion (the versioned chain contract, re-proven from the
        // actual retained records); at hop zero a same-value reassertion head is the
        // head contract's first hop.
        let same_value = cursor["action"] == 1
            && older["conflict"] == false
            && atom(&older["owner"])? == c.owner
            && record_value(cursor)? == record_value(older)?;
        if cursor["action"] != 2 && !same_value {
            return None;
        }
        let _ = hop;
        cursor = older;
    }
    (cursor["id"].as_u64() == Some(root)
        && (!c.target_root || (cursor["previous"] == 0 && cursor["action"] == 1))
        && cursor["conflict"] == false
        && atom(&cursor["owner"])? == c.owner
        && record_value(cursor)? == c.value)
        .then_some(cursor)
}
fn abstained(actual: &Value) -> bool {
    let copy = &actual["first_decision"]["word_copy"];
    actual["first_decision"]["field"].is_null()
        && copy["action"] == "no_read"
        && copy["word_index"].as_u64() == Some(16)
}
/// A follow-up keeps the frozen parent's record: the head's immediate previous
/// record through a one-link proof, or the live head itself, with the exact source
/// occurrence, whatever the earlier turns answered.
fn follow_up_selected(c: &Case, actual: &Value) -> bool {
    let (Some(record_id), Some(head)) = (c.follow_up, c.current_record) else {
        return false;
    };
    let relations = &actual["initial_relations"];
    let (Some(records), Some(directory)) = (
        relations["records"].as_array(),
        relations["directory"].as_array(),
    ) else {
        return false;
    };
    let Some(record) = records.iter().find(|r| r["id"].as_u64() == Some(record_id)) else {
        return false;
    };
    if atom(&record["owner"]).as_deref() != Some(c.owner.as_str()) || record["conflict"] != false {
        return false;
    }
    let is_head = record_id == head;
    if is_head {
        if !directory.contains(&json!(head)) {
            return false;
        }
    } else {
        let Some(h) = records.iter().find(|r| r["id"].as_u64() == Some(head)) else {
            return false;
        };
        let head_hop = h["action"] == 1
            && atom(&h["owner"]).as_deref() == Some(c.owner.as_str())
            && record_value(h) == record_value(record);
        if h["previous"].as_u64() != Some(record_id)
            || !(h["action"] == 2 || head_hop)
            || h["conflict"] != false
        {
            return false;
        }
    }
    let source = 32 + ((record_id - 1) & 15);
    let field = &actual["first_decision"]["field"];
    let copy = &actual["first_decision"]["word_copy"];
    let endpoint = |d: &Value| {
        d["source_end"] == record["value"]["end"]
            && d["source_byte_end"] == record["value"]["byte_end"]
    };
    if field.is_object() {
        let a = &field["anchor"];
        a["relation_id"].as_u64() == Some(record_id)
            && a["source"].as_u64() == Some(source)
            && (if is_head {
                a["current_revision"].is_null()
            } else {
                a["current_revision"].as_u64() == Some(head) && a["ancestor_depth"].is_null()
            })
            && endpoint(a)
    } else {
        copy["word_index"].as_u64() == Some(source)
            && copy["dependency"].is_null()
            && endpoint(copy)
    }
}
fn selected(c: &Case, actual: &Value) -> bool {
    if c.follow_up.is_some() {
        return follow_up_selected(c, actual);
    }
    if c.abstain {
        return abstained(actual);
    }
    let Some(record) = target_record(c, actual) else {
        return false;
    };
    let (Some(root), Some(head), Some(depth)) = (c.target_record, c.current_record, c.depth) else {
        return false;
    };
    let source = 32 + ((root - 1) & 15);
    let field = &actual["first_decision"]["field"];
    let copy = &actual["first_decision"]["word_copy"];
    let endpoint = |d: &Value| {
        d["source_end"] == record["value"]["end"]
            && d["source_byte_end"] == record["value"]["byte_end"]
    };
    if field.is_object() {
        let a = &field["anchor"];
        a["relation_id"].as_u64() == Some(root)
            && a["source"].as_u64() == Some(source)
            && a["current_revision"].as_u64() == Some(head)
            && (if depth > 1 {
                a["ancestor_depth"].as_u64() == Some(u64::from(depth))
            } else {
                a["ancestor_depth"].is_null()
            })
            && endpoint(a)
    } else {
        copy["word_index"].as_u64() == Some(source)
            && copy["dependency"].is_null()
            && endpoint(copy)
    }
}
/// Exact authored text. A plain-style request also accepts the frozen emitter's
/// inherited owner sentence for the same exact record; both counts are reported.
fn exact_primary(c: &Case, actual: &Value) -> bool {
    c.expected.as_ref().is_some_and(|e| actual["text"] == *e)
        && actual["eos"] == true
        && actual["initial_relations"] == actual["final_relations"]
}
fn exact(c: &Case, actual: &Value) -> bool {
    // Membership in the frozen, typed accepted list shared with the API check;
    // no alternative is derived from the response.
    actual["text"]
        .as_str()
        .is_some_and(|t| answer_oracle::accepts(&accepted_answers(c), t))
        && actual["eos"] == true
        && actual["initial_relations"] == actual["final_relations"]
}
fn diagnose(model: &Model, cases: &[Case], out: &Path) -> Result<()> {
    // `out` is claimed by main before the model is loaded.
    let mut rows = Vec::new();
    let (mut targets, mut offered, mut exact_count) = (0, 0, 0);
    for c in cases.iter().filter(|c| c.target_record.is_some()) {
        let actual = generate(model, &c.history, &c.prompt, Control::Full, false, false)?;
        let trace = model.historical_version_trace(&c.prompt)?;
        let candidate = trace["candidates"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|v| {
                v["record"].as_u64() == c.target_record
                    && v["current"].as_u64() == c.current_record
                    && v["representable"] == true
            });
        targets += 1;
        offered += usize::from(candidate.is_some());
        exact_count += usize::from(exact(c, &actual) && selected(c, &actual));
        rows.push(json!({"id":c.id,"prompt":c.prompt,"expected":c.expected,"actual":actual,"exact_and_selected":exact(c,&actual)&&selected(c,&actual),"target_offered":candidate,"trace":trace}));
    }
    let summary = json!({"artifact":model.artifact_cid(),"targets":targets,"target_candidate_offered":offered,"exact_and_selected":exact_count,"scope":"No labels selected from predictions; unoffered targets are explicit admission negatives before fitting."});
    save(
        &out.join("diagnostic.json"),
        &json!({"summary":summary,"rows":rows}),
    )?;
    println!("{summary}");
    uor_r4_core::report_output::seal(out)?;
    Ok(())
}
fn probe(model: &Model, out: &Path) -> Result<()> {
    // `out` is claimed by main before the model is loaded.
    let has_witness =
        model.without_historical_version_intent()?.artifact_cid() != model.artifact_cid();
    let mut rows = Vec::new();
    let mut probes = Vec::new();
    for (layout, fact) in [
        "Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi now in Amber Field.",
        "Record: moss dale holds selvi. selvi now in Copper Vale. selvi now in Amber Field.",
    ]
    .iter()
    .enumerate()
    {
        for (query, q) in [
            "What was the previous location of selvi?",
            "What was the initial location of selvi?",
            "Where is selvi?",
        ]
        .iter()
        .enumerate()
        {
            for (style, s) in STYLES.iter().enumerate() {
                probes.push((
                    format!("intent/{layout}/{query}/{style}"),
                    format!("{fact} {q} {s} Answer:"),
                ));
            }
        }
    }
    // Root evicted from the sixteen-slot ring by later unrelated facts: no genuine
    // root survives, so no initial answer may be manufactured from the oldest survivor.
    let mut evicted = String::from(
        "Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi now in Amber Field.",
    );
    for (i, name) in [
        "arbor", "brook", "cairn", "delta", "ember", "fjord", "glade", "haven", "islet", "jetty",
        "knoll", "lagoon", "marsh", "nadir",
    ]
    .iter()
    .enumerate()
    {
        evicted.push_str(&format!(" {name} in Zone{i}."));
    }
    for (style, s) in STYLES.iter().enumerate() {
        probes.push((
            format!("evicted-root/{style}"),
            format!("{evicted} What was the initial location of selvi? {s} Answer:"),
        ));
        probes.push((
            format!("evicted-previous/{style}"),
            format!("{evicted} What was the previous location of selvi? {s} Answer:"),
        ));
    }
    for (style, s) in STYLES.iter().enumerate() {
        probes.push((format!("same-spelling/{style}"), format!("Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi now in Dusk Ridge. What was the initial location of selvi? {s} Answer:")));
        probes.push((format!("four-versions/{style}"), format!("Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi now in Amber Field. selvi now in Silver Cove. What was the initial location of selvi? {s} Answer:")));
    }
    for (id, prompt) in probes {
        rows.push(json!({"id":id,"prompt":prompt,"actual":generate(model,&[],&prompt,Control::Full,false,true)?,"trace":model.historical_version_trace(&prompt)?,"frozen_trace":model.historical_query_trace(&prompt)?}));
    }
    save(
        &out.join("probe.json"),
        &json!({"artifact":model.artifact_cid(),"version_witness":has_witness,"rows":rows}),
    )?;
    let summary: Vec<_> = rows
        .iter()
        .map(|r| json!({"id":r["id"],"text":r["actual"]["text"]}))
        .collect();
    save(&out.join("summary.json"), &summary)?;
    println!(
        "{}",
        json!({"artifact":model.artifact_cid(),"rows":rows.len(),"version_witness":has_witness})
    );
    uor_r4_core::report_output::seal(out)?;
    Ok(())
}
fn evaluate(
    model: &Model,
    parent: &Model,
    cases: &[Case],
    out: &Path,
    controls: bool,
) -> Result<()> {
    // `out` is claimed by main before the model is loaded.
    let mut rows = BufWriter::new(fs::File::create_new(out.join("rows.jsonl"))?);
    let mut written = 0usize;
    let (
        mut targets,
        mut correct,
        mut selected_count,
        mut inherited,
        mut preserved,
        mut disabled_equal,
        mut transform_exact,
        mut ancestor_exact,
        mut ancestor_exact_deep,
        mut deep_targets,
        mut positions,
        mut scope_exact,
        mut scope_preserved,
        mut primary_exact,
        mut abstain_targets,
        mut abstain_exact,
        mut abstain_disabled_parent_equal,
        mut reassertion_disabled_exact,
        mut reassertion_disabled_parent_equal,
        mut follow_up_targets,
        mut follow_up_exact,
        mut head_disabled_exact,
        mut head_disabled_parent_equal,
        mut previous_targets,
        mut previous_exact,
    ) = (
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    );
    let mut by_depth: BTreeMap<u8, [usize; 2]> = BTreeMap::new();
    let mut by_intent: BTreeMap<&str, [usize; 2]> = BTreeMap::new();
    for c in cases {
        let target = c.target_record.is_some() || c.abstain || c.follow_up.is_some();
        let actual = generate(model, &c.history, &c.prompt, Control::Full, target, !target)?;
        let reference = generate(parent, &c.history, &c.prompt, Control::Full, false, !target)?;
        // A follow-up compares text and EOS: its prior turns were answered by each
        // model itself, so positions differ while the record answer must not.
        let equivalent = |a: &Value, b: &Value| {
            if c.follow_up.is_some() {
                a["text"] == b["text"] && a["eos"] == b["eos"]
            } else {
                comparable(a) == comparable(b)
            }
        };
        let same = equivalent(&actual, &reference);
        let selection = target && selected(c, &actual);
        let target_correct = target && exact(c, &actual) && selection;
        let primary = target && exact_primary(c, &actual) && selection;
        if target {
            targets += 1;
            correct += usize::from(target_correct);
            primary_exact += usize::from(primary);
            selected_count += usize::from(selection);
            if !c.target_root && c.target_record.is_some() {
                previous_targets += 1;
                previous_exact += usize::from(target_correct);
            }
            let kind = by_intent
                .entry(match intent(c) {
                    Intent::Initial => "initial",
                    Intent::Previous => "previous",
                    Intent::Current => "current",
                    Intent::Abstain => "abstain",
                })
                .or_default();
            kind[0] += 1;
            kind[1] += usize::from(target_correct);
            if c.follow_up.is_some() {
                follow_up_targets += 1;
                follow_up_exact += usize::from(target_correct);
            } else {
                let entry = by_depth.entry(c.depth.unwrap_or(0)).or_default();
                entry[0] += 1;
                entry[1] += usize::from(target_correct);
            }
            if c.depth.is_some_and(|d| d > 1) {
                deep_targets += 1;
            }
            if c.abstain {
                abstain_targets += 1;
                abstain_exact += usize::from(target_correct);
            }
        } else {
            inherited += 1;
            preserved += usize::from(same);
        }
        positions += actual["checkpoint_positions"].as_u64().unwrap_or(0);
        let mut interventions = Vec::new();
        if target && controls {
            for control in [
                Control::HistoricalVersionIntentDisabled,
                Control::HistoricalVersionIntentTransformDisabled,
                Control::HistoricalVersionIntentAncestorDisabled,
                Control::HistoricalVersionIntentAbstainDisabled,
                Control::HistoricalVersionIntentReassertionDisabled,
                Control::HistoricalVersionIntentReassertionHeadDisabled,
            ] {
                let result = generate(model, &c.history, &c.prompt, control, false, false)?;
                let result_exact = exact(c, &result) && selected(c, &result);
                match control {
                    Control::HistoricalVersionIntentDisabled => {
                        disabled_equal += usize::from(equivalent(&result, &reference))
                    }
                    Control::HistoricalVersionIntentTransformDisabled => {
                        transform_exact += usize::from(result_exact)
                    }
                    Control::HistoricalVersionIntentAbstainDisabled => {
                        if c.abstain {
                            abstain_disabled_parent_equal +=
                                usize::from(equivalent(&result, &reference));
                        }
                    }
                    Control::HistoricalVersionIntentReassertionDisabled => {
                        reassertion_disabled_exact += usize::from(result_exact);
                        reassertion_disabled_parent_equal +=
                            usize::from(equivalent(&result, &reference));
                    }
                    Control::HistoricalVersionIntentReassertionHeadDisabled => {
                        head_disabled_exact += usize::from(result_exact);
                        head_disabled_parent_equal += usize::from(equivalent(&result, &reference));
                    }
                    _ => {
                        ancestor_exact += usize::from(result_exact);
                        if c.depth.is_some_and(|d| d > 1) {
                            ancestor_exact_deep += usize::from(result_exact);
                        }
                    }
                }
                // Control rows keep text/EOS/first decision; relation state stays on actual/parent rows.
                interventions.push(json!({"control":control,"result":comparable(&result),"records_unchanged":result["initial_relations"]==result["final_relations"],"exact_and_selected":result_exact}));
            }
        }
        let scope = generate(
            model,
            &c.history,
            &c.prompt,
            Control::HistoricalVersionIntentScopeDisabled,
            false,
            !target,
        )?;
        let scope_same = equivalent(&scope, &reference);
        let scope_correct = target && exact(c, &scope) && selected(c, &scope);
        scope_exact += usize::from(scope_correct);
        scope_preserved += usize::from(!target && scope_same);
        let row = json!({"id":c.id,"prompt":c.prompt,"history":c.history,"follow_up":c.follow_up,"expected":c.expected,"current_record":c.current_record,"target_record":c.target_record,"depth":c.depth,"owner":c.owner,"value":c.value,"actual":actual,"parent":reference,"correct":if target{Some(target_correct)}else{None},"primary_text":if target{Some(primary)}else{None},"selected":if target{Some(selection)}else{None},"parent_equal":same,"records_unchanged":actual["initial_relations"]==actual["final_relations"],"controls":interventions,"scope_disabled":{"text":scope["text"],"eos":scope["eos"],"parent_equal":scope_same,"exact_and_selected":scope_correct}});
        serde_json::to_writer(&mut rows, &row)?;
        rows.write_all(b"\n")?;
        written += 1;
    }
    rows.flush()?;
    drop(rows);
    let summary = json!({"artifact":model.artifact_cid(),"parent":parent.artifact_cid(),"total":cases.len(),"targets":targets,"exact_and_selected":correct,"primary_text_exact_and_selected":primary_exact,"abstain_targets":abstain_targets,"abstain_exact":abstain_exact,"abstain_disabled_parent_equal":abstain_disabled_parent_equal,"reassertion_disabled_exact_and_selected":reassertion_disabled_exact,"reassertion_disabled_parent_equal":reassertion_disabled_parent_equal,"follow_up_targets":follow_up_targets,"follow_up_exact_and_selected":follow_up_exact,"previous_targets":previous_targets,"previous_exact_and_selected":previous_exact,"head_disabled_exact_and_selected":head_disabled_exact,"head_disabled_parent_equal":head_disabled_parent_equal,"targets_by_intent":by_intent.iter().map(|(k,[t,c])| json!({"intent":k,"targets":t,"exact_and_selected":c})).collect::<Vec<_>>(),"count_scope":"targets_by_intent is the disjoint typed partition; follow_up_targets and previous_targets are cross-cutting categories that overlap it","selected":selected_count,"targets_by_depth":by_depth.iter().map(|(d,[t,c])| json!({"depth":d,"targets":t,"exact_and_selected":c})).collect::<Vec<_>>(),"deep_targets":deep_targets,"inherited":inherited,"preserved":preserved,"controls":controls,"disabled_parent_equal":disabled_equal,"transform_disabled_exact":transform_exact,"ancestor_disabled_exact":ancestor_exact,"ancestor_disabled_exact_deep":ancestor_exact_deep,"checkpoint_positions":positions,"scope_disabled_exact_and_selected":scope_exact,"scope_disabled_preserved":scope_preserved});
    let mut result = summary.clone();
    result["rows_file"] = json!(out.join("rows.jsonl"));
    result["rows_written"] = json!(written);
    save(&out.join("result.json"), &result)?;
    save(&out.join("summary.json"), &summary)?;
    println!("{summary}");
    Ok(())
}
fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() == 3 && ["prepare", "fresh"].contains(&a[1].as_str()) {
        return prepare(Path::new(&a[2]), a[1] == "fresh");
    }
    if a.len() == 4 && a[1] == "probe" {
        // Every destination is reserved before the model is loaded or anything is generated.
        let out = Path::new(&a[3]);
        uor_r4_core::report_output::claim(out)?;
        let model = Model::from_bytes(&fs::read(&a[2])?)?;
        return probe(&model, out);
    }
    if a.len() == 3 && a[1] == "verify" {
        // Complete sealed file set: listed hashes and no unlisted files.
        uor_r4_core::report_output::verify(Path::new(&a[2]))?;
        println!("{}", json!({"verified":a[2]}));
        return Ok(());
    }
    if a.len() == 4 && a[1] == "api-cases" {
        // Derived API inputs live in their own claimed attempt, bound to the source
        // case file (and its sealed manifest when present); the source root is not touched.
        let out = Path::new(&a[3]);
        uor_r4_core::report_output::claim(out)?;
        let source = Path::new(&a[2]);
        let bytes = fs::read(source)?;
        let cases: Vec<Case> = serde_json::from_slice(&bytes)?;
        let api: Vec<Value> = cases
            .iter()
            .filter(|c| c.current_record.is_some())
            .map(|c| {
                let mut v = serde_json::to_value(c).unwrap_or(Value::Null);
                v["accepted"] = json!(accepted_answers(c));
                v["intent"] = json!(intent(c));
                v
            })
            .collect();
        save(&out.join("api-cases.json"), &api)?;
        let manifest = source
            .parent()
            .map(|d| d.join("manifest.json"))
            .filter(|m| m.is_file());
        save(
            &out.join("source.json"),
            &json!({"schema":"uor-r4.api-cases-source/1","source":source,"source_blake3":blake3::hash(&bytes).to_hex().to_string(),"source_bytes":bytes.len(),
                "source_manifest":manifest.as_ref().map(|m| json!({"path":m,"blake3":fs::read(m).map(|b| blake3::hash(&b).to_hex().to_string()).unwrap_or_default()})),
                "cases":api.len(),"rule":"accepted lists come from the typed intent frozen at preparation; the API check judges membership only"}),
        )?;
        println!("{}", json!({"api_cases":api.len(),"out":out}));
        uor_r4_core::report_output::seal(out)?;
        return Ok(());
    }
    if a.len() == 3 && a[1] == "seal" {
        // Seal a completed report directory produced by any driver.
        let manifest = uor_r4_core::report_output::seal(Path::new(&a[2]))?;
        println!("{}", json!({"sealed":manifest}));
        return Ok(());
    }
    if a.len() == 4 && a[1] == "promote-heads" {
        // The same learned witness under the head contract; no refit (diagnostic).
        let out = Path::new(&a[3]);
        uor_r4_core::report_output::claim(out)?;
        let bytes = fs::read(&a[2])?;
        let model = Model::from_bytes(&bytes)?;
        let candidate = model.with_reassertion_heads()?;
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(
            &out.join("promote.json"),
            &json!({"schema":"uor-r4.historical-version-contract-promotion/1","source":model.artifact_cid(),"artifact":candidate.artifact_cid(),"source_bytes_blake3":blake3::hash(&bytes).to_hex().to_string(),"change":"reassertion_heads=true only; router codes, dictionary, receipts and every inner parameter unchanged"}),
        )?;
        println!(
            "{}",
            json!({"source":model.artifact_cid(),"artifact":candidate.artifact_cid()})
        );
        uor_r4_core::report_output::seal(out)?;
        return Ok(());
    }
    if a.len() == 4 && a[1] == "promote" {
        // The same learned witness under the versioned chain contract; no refit.
        let out = Path::new(&a[3]);
        uor_r4_core::report_output::claim(out)?;
        let bytes = fs::read(&a[2])?;
        let model = Model::from_bytes(&bytes)?;
        let candidate = model.with_reassertion_links()?;
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(
            &out.join("promote.json"),
            &json!({"schema":"uor-r4.historical-version-contract-promotion/1","source":model.artifact_cid(),"artifact":candidate.artifact_cid(),"source_bytes_blake3":blake3::hash(&bytes).to_hex().to_string(),"change":"reassertion_links=true only; router codes, dictionary, receipts and every inner parameter unchanged"}),
        )?;
        println!(
            "{}",
            json!({"source":model.artifact_cid(),"artifact":candidate.artifact_cid()})
        );
        uor_r4_core::report_output::seal(out)?;
        return Ok(());
    }
    if a.len() == 6 && a[1] == "compare" {
        // Actual outputs of this artifact against another artifact on the same prompts.
        let out = Path::new(&a[5]);
        uor_r4_core::report_output::claim(out)?;
        let model = Model::from_bytes(&fs::read(&a[2])?)?;
        let other = Model::from_bytes(&fs::read(&a[3])?)?;
        let cases: Vec<Case> = serde_json::from_slice(&fs::read(&a[4])?)?;
        let mut rows = BufWriter::new(fs::File::create_new(out.join("rows.jsonl"))?);
        let (mut equal, mut differing) = (0, Vec::new());
        for c in &cases {
            let mine = generate(&model, &c.history, &c.prompt, Control::Full, false, true)?;
            let theirs = generate(&other, &c.history, &c.prompt, Control::Full, false, true)?;
            let same = comparable(&mine) == comparable(&theirs);
            equal += usize::from(same);
            if !same {
                differing.push(json!({"id":c.id,"abstain_target":c.abstain,"follow_up":c.follow_up,"this":mine["text"],"other":theirs["text"]}));
            }
            serde_json::to_writer(
                &mut rows,
                &json!({"id":c.id,"equal":same,"this":comparable(&mine),"other":comparable(&theirs)}),
            )?;
            rows.write_all(b"\n")?;
        }
        rows.flush()?;
        let summary = json!({"artifact":model.artifact_cid(),"other":other.artifact_cid(),"total":cases.len(),"equal":equal,"differing":differing});
        save(&out.join("summary.json"), &summary)?;
        println!(
            "{}",
            json!({"artifact":model.artifact_cid(),"other":other.artifact_cid(),"total":cases.len(),"equal":equal,"differing":differing.len()})
        );
        uor_r4_core::report_output::seal(out)?;
        return Ok(());
    }
    if a.len() != 5
        || !["diagnose", "fit", "refit", "evaluate", "preserve"].contains(&a[1].as_str())
    {
        return Err("usage: prepare/fresh OUT | probe MODEL OUT | promote MODEL OUT | seal DIR | verify DIR | api-cases CASES OUT | diagnose MODEL CASES OUT | fit/refit MODEL TRAIN OUT | evaluate/preserve MODEL CASES OUT | compare MODEL OTHER CASES OUT".into());
    }
    let out = Path::new(&a[4]);
    uor_r4_core::report_output::claim(out)?;
    let bytes = fs::read(&a[2])?;
    let model = Model::from_bytes(&bytes)?;
    if model.to_bytes()? != bytes {
        return Err("supplied artifact byte roundtrip differs".into());
    }
    if a[1] == "fit" || a[1] == "refit" {
        let docs: Vec<HistoricalVersionExample> = serde_json::from_slice(&fs::read(&a[3])?)?;
        let config = SourceRoutingConfig {
            learned_features: 768,
            passes: 8,
            proposals: 120,
            max_seconds: 120,
            mode: RoutingMode::Angular,
            seed: 973,
            role_context_only: false,
        };
        let (candidate, report) = if a[1] == "refit" {
            model.refit_historical_version_intent(&docs, config)?
        } else {
            model.fit_historical_version_intent(&docs, config)?
        };
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(&out.join("fit.json"), &report)?;
        let mut brief = report.clone();
        if let Some(o) = brief.as_object_mut() {
            o.remove("labels");
            o.remove("dictionary");
        }
        println!("{brief}");
        uor_r4_core::report_output::seal(out)?;
        return Ok(());
    }
    let mut cases: Vec<Case> = serde_json::from_slice(&fs::read(&a[3])?)?;
    if a[1] == "preserve" {
        for c in &mut cases {
            c.current_record = None;
            c.target_record = None;
            c.depth = None;
            c.expected = None;
            c.abstain = false;
            c.follow_up = None;
            c.accepted.clear();
            c.target_root = true;
        }
    }
    if a[1] == "diagnose" {
        return diagnose(&model, &cases, out);
    }
    let parent = model.without_historical_version_intent()?;
    if parent.artifact_cid() == model.artifact_cid() {
        return Err("evaluate requires the actual historical-version outer witness".into());
    }
    evaluate(&model, &parent, &cases, out, a[1] == "evaluate")?;
    save(
        &out.join("lineage.json"),
        &json!({"parent":parent.artifact_cid(),"artifact":model.artifact_cid(),"candidate_roundtrip":true,"parent_bytes_blake3":blake3::hash(&parent.to_bytes()?).to_hex().to_string()}),
    )?;
    uor_r4_core::report_output::seal(out)?;
    Ok(())
}
