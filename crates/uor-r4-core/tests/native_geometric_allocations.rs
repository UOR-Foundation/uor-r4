//! Direct allocation census for successful native session observe/predict.
//! Fitting, model construction, tokenization, session allocation, and reports
//! are deliberately outside the measured section. TLS excludes libtest noise.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use uor_r4_core::native_geometric::{
    Config, Control, Document, MemoryReadFitConfig, MemoryReadSchedule, MemoryReadSupervision,
    MemoryReadTokenSpan, MemoryReadTrainer, ReadoutFitConfig, ResponseEntryFitConfig, Trainer,
    ValueCompletionFitConfig, ValueExample, ValueFitConfig, WordCopyAction, BOS, EOS,
};

mod copy_fixture {
    use uor_r4_core::native_geometric as native;
    include!("support/native_word_copy_fixture.rs");
}

struct CountingAllocator;
thread_local! {
    static MEASURING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            let _ = MEASURING.try_with(|enabled| {
                if enabled.get() {
                    let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
                    let _ = BYTES.try_with(|count| count.set(count.get() + layout.size()));
                }
            });
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// Source operator/type-token guard for the actual native kernel and its
/// project-defined feature helpers. This does not inspect transitive standard
/// library implementations or generated machine code. Runtime allocation is
/// measured independently by the census below. Session setup, state rendering,
/// model construction/fitting, encoding and checkpoints are explicit host work.
#[test]
fn native_kernel_source_has_no_forbidden_arithmetic_or_float_types() {
    use uor_r4_core::transformerless::source_scan::{
        scan_for_forbidden_arith_and_floats, ALLOW_MARKER,
    };

    fn region<'a>(source: &'a str, begin: &str, end: &str) -> (&'a str, &'a str, &'a str) {
        assert_eq!(
            source.matches(begin).count(),
            1,
            "unique begin marker {begin}"
        );
        assert_eq!(source.matches(end).count(), 1, "unique end marker {end}");
        let (before, rest) = source.split_once(begin).unwrap();
        let (body, after) = rest.split_once(end).unwrap();
        (before, body, after)
    }

    let runtime = include_str!("../src/native_geometric/runtime.rs");
    let (host, kernel, after) = region(
        runtime,
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
    );
    assert!(
        after.trim().is_empty(),
        "runtime helpers must remain inside the scanned kernel region"
    );
    // The three existing constructor/report accessors are the only function
    // bodies before the runtime boundary; adding another requires reviewing
    // whether it belongs to host setup or the scanned prediction path.
    let host_functions: Vec<_> = host
        .lines()
        .map(str::trim)
        .filter(|line| line.contains("fn "))
        .collect();
    assert_eq!(
        host_functions,
        [
            "pub(super) fn new(model: &Model, control: Control) -> Self {",
            "pub fn state(&self) -> StateView {",
            "pub fn candidates(&self) -> &[Candidate] {",
        ]
    );
    for function in [
        "fn check_model(",
        "fn product(",
        "fn observe(",
        "fn begin_response(",
        "fn end_response(",
        "fn needs_input_boundary(",
        "fn response_decision(",
        "fn value_decision(",
        "fn completion_decision(",
        "fn response_entry_decision(",
        "fn word_copy_decision(",
        "fn recent(",
        "fn features(",
        "fn score_candidate(",
        "fn offer(",
        "fn offer_memory(",
        "fn predict(",
        "fn gate_eighths(",
    ] {
        assert_eq!(
            kernel.matches(function).count(),
            1,
            "kernel coverage includes {function}"
        );
    }
    let (_, features, _) = region(
        include_str!("../src/native_geometric/mod.rs"),
        "// NATIVE_GEOMETRIC_INTEGER_FEATURE_METHODS_BEGIN",
        "// NATIVE_GEOMETRIC_INTEGER_FEATURE_METHODS_END",
    );
    for function in ["fn group(", "fn shift(", "fn admitted("] {
        assert_eq!(
            features.matches(function).count(),
            1,
            "feature coverage includes {function}"
        );
    }
    // MemoryState allocation lives in memory_types.rs. The entire separate
    // memory runtime (including its private feature-admission helper) is scanned.
    let memory = include_str!("../src/native_geometric/memory_runtime.rs");
    for function in [
        "fn cue_identity(",
        "fn pack_query_occurrence(",
        "fn admitted(",
        "fn state(",
        "fn recent(",
        "fn observe(",
        "fn collect(",
    ] {
        assert_eq!(
            memory.matches(function).count(),
            1,
            "memory kernel coverage includes {function}"
        );
    }
    let response = include_str!("../src/native_geometric/response_runtime.rs");
    for function in [
        "fn response_view(",
        "fn begin_response(",
        "fn end_response(",
        "fn reference_for_sequence(",
        "fn select_response(",
        "fn commit_response(",
        "fn collect_continuation(",
    ] {
        assert_eq!(
            response.matches(function).count(),
            1,
            "response kernel coverage includes {function}"
        );
    }
    let value_runtime = include_str!("../src/native_geometric/value_runtime.rs");
    for function in [
        "fn observe(",
        "fn begin(",
        "fn end(",
        "fn proposal(",
        "fn features(",
        "fn offer(",
        "fn selected(",
        "fn commit(",
        "fn execute(",
    ] {
        assert_eq!(
            value_runtime.matches(function).count(),
            1,
            "typed-value kernel coverage includes {function}"
        );
    }
    let completion = include_str!("../src/native_geometric/completion_runtime.rs");
    for function in [
        "fn reset(",
        "fn observe(",
        "fn features(",
        "fn offer(",
        "fn selected(",
        "fn candidates(",
        "fn offer_token(",
        "fn score_candidate(",
    ] {
        assert_eq!(
            completion.matches(function).count(),
            1,
            "completion kernel coverage includes {function}"
        );
    }
    let completion_types = include_str!("../src/native_geometric/completion_types.rs");
    let response_entry = include_str!("../src/native_geometric/response_entry_runtime.rs");
    let word_copy = include_str!("../src/native_geometric/word_copy_runtime.rs");
    for function in [
        "fn eligible(",
        "fn reset(",
        "fn begin(",
        "fn observe(",
        "fn features(",
        "fn offer(",
        "fn selected(",
    ] {
        assert_eq!(
            response_entry.matches(function).count(),
            1,
            "response-entry kernel coverage includes {function}"
        );
    }
    let seed = completion_types
        .split_once("impl From<&ValueDecision> for CompletionSeed {")
        .unwrap()
        .1
        .split_once("#[derive")
        .unwrap()
        .0;
    let (_, numeral, _) = region(
        include_str!("../src/native_geometric/numeral.rs"),
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
    );
    let (_, lexemes, _) = region(
        include_str!("../src/native_geometric/value_lexemes.rs"),
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
    );
    for function in ["fn from_zphi(", "fn digit(", "fn feed(", "fn finish("] {
        assert_eq!(
            numeral.matches(function).count(),
            1,
            "numeral kernel coverage includes {function}"
        );
    }
    // The only external project arithmetic called by typed execution is the
    // existing exact checked coefficient addition; scan that actual method.
    let zphi_source = include_str!("../src/prime_route_attention.rs");
    let zphi = zphi_source.split_once("impl ZPhi {").unwrap().1;
    let zphi_add = zphi
        .split_once("pub fn checked_add(")
        .unwrap()
        .1
        .split_once("/// Exact checked multiplication")
        .unwrap()
        .0;
    let (_, operation_transition, _) = region(
        include_str!("../src/native_geometric/operation_transition.rs"),
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
    );
    for (name, source) in [
        ("native operation transition", operation_transition),
        ("native runtime", kernel),
        ("native Feature helpers", features),
        ("native memory runtime", memory),
        ("native response runtime", response),
        ("native typed-value runtime", value_runtime),
        ("native completion runtime", completion),
        ("native response-entry runtime", response_entry),
        ("native retained-word copy runtime", word_copy),
        (
            "native learned routing runtime",
            include_str!("../src/native_geometric/learned_routing.rs"),
        ),
        (
            "native recurrent routing output",
            include_str!("../src/native_geometric/recurrent_routing.rs"),
        ),
        (
            "native retained-source routing",
            include_str!("../src/native_geometric/source_routing.rs"),
        ),
        (
            "native dependent read",
            include_str!("../src/native_geometric/dependent_read.rs"),
        ),
        (
            "native typed routing",
            include_str!("../src/native_geometric/typed_routing.rs"),
        ),
        (
            "native joint admission",
            region(
                include_str!("../src/native_geometric/joint_admission.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native source span",
            include_str!("../src/native_geometric/source_span.rs"),
        ),
        (
            "native relation memory",
            region(
                include_str!("../src/native_geometric/relation.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native retained relation extent",
            region(
                include_str!("../src/native_geometric/relation_span.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native learned relation start",
            region(
                include_str!("../src/native_geometric/relation_start.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        ("native completion seed", seed),
        ("native numeral codec", numeral),
        ("native whole-word codec", lexemes),
        ("exact ZPhi checked addition", zphi_add),
    ] {
        assert!(
            !source.contains(ALLOW_MARKER),
            "{name} permits no source-arithmetic exceptions"
        );
        let outcome = scan_for_forbidden_arith_and_floats(source);
        assert!(
            outcome.offenders.is_empty(),
            "{name}: {:?}",
            outcome.offenders
        );
        assert!(
            outcome.allowed.is_empty(),
            "{name}: no kernel allowances are permitted"
        );
    }
}

#[test]
fn native_learned_routing_selection_and_transformation_are_allocation_free() {
    routing_allocation(false);
}

#[test]
fn native_recurrent_routing_joint_output_is_allocation_free() {
    routing_allocation(true);
}

fn routing_allocation(recurrent: bool) {
    use uor_r4_core::native_geometric::{RoutingFitConfig, RoutingMode};
    let docs = [Document {
        id: "routing-allocation".into(),
        text: "Alice saved red. Bob saved blue. Alice gave Bob red.".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 16,
            ..Config::default()
        },
        &docs,
    )
    .unwrap();
    trainer.train_documents(&docs).unwrap();
    let parent = trainer.compile().unwrap();
    let config = RoutingFitConfig {
        max_positions: 32,
        learned_tokens: 2,
        passes: 1,
        mode: RoutingMode::Angular,
        ..RoutingFitConfig::default()
    };
    let model = if recurrent {
        parent
            .fit_recurrent_routing(
                &docs,
                &[uor_r4_core::native_geometric::ValueExample {
                    id: "routing-allocation-response".into(),
                    prompt: "Alice saved".into(),
                    response: " red.".into(),
                }],
                config,
            )
            .unwrap()
            .0
    } else {
        parent.fit_routing_block(&docs, config).unwrap().0
    };
    let tokens = model.encode(&docs[0].text).unwrap();
    let mut session = model.session(Control::Full).unwrap();
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));
    let result = (|| {
        for _ in 0..3 {
            for &token in &tokens {
                session.observe(&model, token)?;
                session.predict(&model)?;
            }
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|v| v.set(false));
    result.unwrap();
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
    assert_eq!(BYTES.with(Cell::get), 0);
    assert!(session.work.learned_routing.payload_gathers > 0);
    assert!(session.work.learned_routing.operator_executions > 0);
}

#[test]
fn native_typed_value_ingest_write_emission_and_eviction_are_allocation_free() {
    typed_value_allocation_census(false);
    typed_value_allocation_census(true);
}

fn typed_value_allocation_census(lexeme_cues: bool) {
    let documents = [Document {
        id: "typed-allocation-catalog".into(),
        text: "left = 13; right = 4; total: 17 unknown".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 32,
            candidate_limit: 8,
            ..Config::default()
        },
        &documents,
    )
    .unwrap();
    trainer.train_documents(&documents).unwrap();
    let baseline = trainer.compile().unwrap();
    let source = [ValueExample {
        id: "typed-allocation-fit".into(),
        prompt: "left = 13; right = 4; total:".into(),
        response: "17".into(),
    }];
    let config = ValueFitConfig {
        epochs: 64,
        learning_rate: 0.25,
        max_features: 4096,
    };
    let (model, _) = if lexeme_cues {
        baseline.fit_values_with_lexeme_cues(&source, config)
    } else {
        baseline.fit_values(&source, config)
    }
    .unwrap();
    let tokens = model.encode(&source[0].prompt).unwrap();
    let mut session = model.session(Control::Full).unwrap();
    let before = session.state().values.unwrap().storage_bytes;

    ALLOCATIONS.with(|count| count.set(0));
    BYTES.with(|count| count.set(0));
    MEASURING.with(|enabled| enabled.set(true));
    let result = (|| {
        session.observe(&model, BOS)?;
        for &token in &tokens {
            session.observe(&model, token)?;
        }
        session.begin_response(&model)?;
        let first = session.predict(&model)?;
        let selected = session.value_decision().is_some();
        session.observe(&model, first.token)?;
        // Interrupt a committed multi-byte value using a different teacher
        // byte. It must clear the cursor without an allocating recovery path.
        session.predict(&model)?;
        session.observe(&model, u32::from(b'x') + 2)?;
        session.end_response(&model)?;
        for _ in 0..64 {
            for &token in &tokens {
                session.observe(&model, token)?;
            }
            session.begin_response(&model)?;
            for _ in 0..2 {
                let prediction = session.predict(&model)?;
                session.observe(&model, prediction.token)?;
            }
            session.refresh_value_sources(&model)?;
            session.end_response(&model)?;
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(selected)
    })();
    MEASURING.with(|enabled| enabled.set(false));
    let allocations = ALLOCATIONS.with(Cell::get);
    let bytes = BYTES.with(Cell::get);
    assert!(result.unwrap(), "fixture must select a fitted typed result");
    assert_eq!((allocations, bytes), (0, 0));
    assert_eq!(session.state().values.unwrap().storage_bytes, before);
    assert!(session.work.values.input_bytes > 0);
    assert!(session.work.values.literal_writes > 16);
    assert!(session.work.values.record_evictions > 0);
    assert!(session.work.values.derived_writes > 0);
    assert!(session.work.values.source_refreshes > 0);
    assert!(session.work.values.emission_commits > 0);
    assert!(session.work.values.emission_mismatches > 0);
    assert!(session.work.evictions > 0);
    println!(
        "typed-value census allocations={allocations} bytes={bytes} work={:?}",
        session.work.values
    );
}

#[test]
fn native_value_completion_commit_mismatch_and_limit_are_allocation_free() {
    let documents = [Document {
        id: "completion-allocation-catalog".into(),
        text: "left = 13; right = 4; total: 17.\n".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 32,
            candidate_limit: 8,
            ..Config::default()
        },
        &documents,
    )
    .unwrap();
    trainer.train_documents(&documents).unwrap();
    let baseline = trainer.compile().unwrap();
    let source = [ValueExample {
        id: "completion-allocation-fit".into(),
        prompt: "left = 13; right = 4; total:".into(),
        response: "17.\n".into(),
    }];
    let (typed, _) = baseline
        .fit_values_with_lexeme_cues(
            &source,
            ValueFitConfig {
                epochs: 64,
                learning_rate: 0.25,
                max_features: 4096,
            },
        )
        .unwrap();
    let (model, report) = typed
        .fit_value_completion(&source, ValueCompletionFitConfig::default())
        .unwrap();
    assert_eq!(report.matched_numeric, 1);
    let prompt = model.encode(&source[0].prompt).unwrap();
    let mut session = model.session(Control::Full).unwrap();
    let storage = session.state().completion.unwrap().storage_bytes;
    ALLOCATIONS.with(|count| count.set(0));
    BYTES.with(|count| count.set(0));
    MEASURING.with(|enabled| enabled.set(true));
    let result = (|| {
        session.observe(&model, BOS)?;
        for _ in 0..4 {
            session.end_response(&model)?;
            for &token in &prompt {
                session.observe(&model, token)?;
            }
            session.begin_response(&model)?;
            for _ in 0..2 {
                let next = session.predict(&model)?.token;
                session.observe(&model, next)?;
            }
            // Commit one learned suffix token, then deliberately observe a
            // different byte and exercise the finite progress limit.
            let next = session.predict(&model)?.token;
            session.observe(&model, next)?;
            session.predict(&model)?;
            session.observe(&model, u32::from(b'x') + 2)?;
            for _ in 0..31 {
                session.predict(&model)?;
                session.observe(&model, u32::from(b'x') + 2)?;
            }
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|enabled| enabled.set(false));
    result.unwrap();
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    assert_eq!(session.state().completion.unwrap().storage_bytes, storage);
    assert_eq!(session.work.completion.anchors, 4);
    assert!(session.work.completion.commits > 0);
    assert!(session.work.completion.mismatches > 0);
    assert_eq!(session.work.completion.step_limits, 4);
    assert!(session.work.evictions > 0);
}

#[test]
fn native_response_entry_commit_mismatch_and_limit_are_allocation_free() {
    let catalog = [Document {
        id: "entry-allocation-catalog".into(),
        text: "left = 13; right = 4; total: 17.\nreply: Unknown.\n Unknown.\n".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 32,
            candidate_limit: 8,
            ..Config::default()
        },
        &catalog,
    )
    .unwrap();
    trainer.train_documents(&catalog).unwrap();
    let examples = (0..4)
        .flat_map(|index| {
            [
                ValueExample {
                    id: format!("entry-allocation-numeric-{index}"),
                    prompt: format!("left = {}; right = 4; total:", 13 + index),
                    response: format!("{}.\n", 17 + index),
                },
                ValueExample {
                    id: format!("entry-allocation-no-write-{index}"),
                    prompt: format!("left = {}; right = 4; reply:", 13 + index),
                    response: " Unknown.\n".into(),
                },
            ]
        })
        .collect::<Vec<_>>();
    let (typed, _) = trainer
        .compile()
        .unwrap()
        .fit_values_with_lexeme_cues(
            &examples,
            ValueFitConfig {
                epochs: 64,
                learning_rate: 0.25,
                max_features: 4096,
            },
        )
        .unwrap();
    let (completion, _) = typed
        .fit_value_completion(&examples, ValueCompletionFitConfig::default())
        .unwrap();
    let (model, _) = completion
        .fit_response_entry(&examples, ResponseEntryFitConfig::default())
        .unwrap();
    let prompt = model.encode(&examples[1].prompt).unwrap();
    let mut session = model.session(Control::Full).unwrap();
    let storage = session.state().response_entry.unwrap().storage_bytes;
    ALLOCATIONS.with(|count| count.set(0));
    BYTES.with(|count| count.set(0));
    MEASURING.with(|enabled| enabled.set(true));
    let result = (|| {
        session.observe(&model, BOS)?;
        let mut selected_entry = true;
        for _ in 0..4 {
            session.end_response(&model)?;
            for &token in &prompt {
                session.observe(&model, token)?;
            }
            session.begin_response(&model)?;
            let first = session.predict(&model)?.token;
            selected_entry &= session.response_entry_decision().is_some();
            // A repeated prediction must preserve the pending first choice.
            selected_entry &= session.predict(&model)?.token == first;
            session.observe(&model, first)?;
            // A mismatching observed byte follows actual history. Repeated
            // observations then exercise the bounded entry-progress limit.
            for _ in 0..40 {
                session.predict(&model)?;
                session.observe(&model, u32::from(b'x') + 2)?;
            }
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(selected_entry)
    })();
    MEASURING.with(|enabled| enabled.set(false));
    assert!(
        result.unwrap(),
        "fixture must select its learned lexical entry"
    );
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    assert_eq!(
        session.state().response_entry.unwrap().storage_bytes,
        storage
    );
    assert!(session.work.response_entry.commits > 0);
    assert!(session.work.response_entry.mismatches > 0);
    assert_eq!(session.work.response_entry.step_limits, 4);
    assert_eq!(session.work.values.derived_writes, 0);
    assert!(session.work.evictions > 0);
}

#[test]
fn native_word_copy_selection_bytes_mismatch_and_cap_are_allocation_free() {
    let (_, original) = copy_fixture::fitted();
    let completed_word = copy_fixture::fitted_completed_word();
    let composed = copy_fixture::fitted_composed();
    let binding = copy_fixture::fitted_shared_binding();
    for (variant, model) in [
        ("actual-byte suffix", original),
        ("completed-word suffix", completed_word),
        ("composed dispatch", composed),
        ("shared binding entry", binding),
    ] {
        let prompt = model.encode(copy_fixture::COPY_PROMPT).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        let storage = session.state().word_copy.unwrap().storage_bytes;
        let mut selected = false;
        let mut repeat_stable = true;
        let mut copied_bytes = 0;
        let mut suffix_decisions = 0;
        let mut observed_eos = false;
        ALLOCATIONS.with(|count| count.set(0));
        BYTES.with(|count| count.set(0));
        MEASURING.with(|enabled| enabled.set(true));
        let result = (|| {
            session.observe(model, BOS)?;
            for &token in &prompt {
                session.observe(model, token)?;
            }
            session.begin_response(model)?;
            for _ in 0..32 {
                let next = session.predict(model)?;
                let decision = session.word_copy_decision();
                selected |= decision.is_some();
                copied_bytes += usize::from(decision.is_some_and(|choice| {
                    matches!(choice.action, WordCopyAction::Start | WordCopyAction::Byte)
                }));
                suffix_decisions += usize::from(decision.is_some_and(|choice| {
                    matches!(choice.action, WordCopyAction::Emit | WordCopyAction::Stop)
                }));
                let repeated = session.predict(model)?;
                repeat_stable &= repeated == next;
                session.observe(model, next.token)?;
                if next.token == EOS {
                    observed_eos = true;
                    break;
                }
            }
            session.end_response(model)?;
            for &token in &prompt {
                session.observe(model, token)?;
            }
            session.begin_response(model)?;
            let first = session.predict(model)?.token;
            session.observe(model, first)?;
            for _ in 0..40 {
                session.predict(model)?;
                session.observe(model, u32::from(b'?') + 2)?;
            }
            session.end_response(model)?;
            Ok::<_, uor_r4_core::native_geometric::Error>(())
        })();
        MEASURING.with(|enabled| enabled.set(false));
        result.expect(variant);
        assert!(
            selected,
            "{variant}: fixture must execute learned retained-word selection"
        );
        assert!(
            repeat_stable,
            "{variant}: repeated prediction must retain the selected byte"
        );
        assert!(
            copied_bytes >= 5,
            "{variant}: fixture must execute its multi-byte cursor"
        );
        assert!(
            suffix_decisions > 0 && observed_eos,
            "{variant}: fixture must execute its suffix through EOS"
        );
        assert_eq!(
            (ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)),
            (0, 0),
            "{variant}"
        );
        assert_eq!(session.state().word_copy.unwrap().storage_bytes, storage);
        assert!(session.work.word_copy.word_candidates > 0);
        assert!(session.work.word_copy.dictionary_comparisons > 0);
        assert!(session.work.word_copy.byte_reads > 0);
        if variant == "composed dispatch" {
            assert!(session.work.word_copy.forced_dispatches > 0);
        }
        assert!(session.work.word_copy.selector.mismatches > 0);
        assert!(session.work.response_entry.step_limits > 0);
        assert_eq!(session.work.values.derived_writes, 0);
    }
}

#[test]
fn native_observe_predict_stays_allocation_free_through_evictions() {
    let documents = [
        Document {
            id: "a".into(),
            text: "red fox finds green stone. red fox keeps green stone.".into(),
        },
        Document {
            id: "b".into(),
            text: "blue bird finds amber seed. blue bird keeps amber seed.".into(),
        },
        Document {
            id: "c".into(),
            text: "green turtle gives red fox a blue shell.".into(),
        },
    ];
    let config = Config {
        context_tokens: 32,
        candidate_limit: 3,
        postings_per_row: 4,
        ..Config::default()
    };
    let mut trainer = Trainer::new(config, &documents).unwrap();
    trainer.train_documents(&documents).unwrap();
    let model = trainer.compile().unwrap();
    let readout_documents = [Document {
        id: "readout-fit".into(),
        text: "red fox gives blue bird green stone. blue bird finds red shell.".into(),
    }];
    let (learned, _) = model
        .fit_readout(
            &readout_documents,
            ReadoutFitConfig {
                max_positions: 128,
                epochs: 2,
                max_queries: 16,
            },
        )
        .unwrap();
    let memory_config = MemoryReadFitConfig {
        advance_response_path: false,
        query_tokens: 4,
        source_offsets: 2,
        postings_per_address: 2,
        candidate_limit: 16,
        max_positions: 128,
        epochs: 2,
        max_features: 1024,
    };
    let (with_memory, memory_fit) = learned
        .fit_memory_read_with_word_cues(&readout_documents, memory_config)
        .unwrap();
    let (exact_memory, _) = learned
        .fit_memory_read(&readout_documents, memory_config)
        .unwrap();
    let (query_context_memory, _) = learned
        .fit_memory_read_with_query_context(&readout_documents, memory_config, true)
        .unwrap();
    let mut composition_trainer = MemoryReadTrainer::new_with_occurrence_composition(
        &learned,
        &readout_documents,
        memory_config,
        MemoryReadSchedule {
            total_positions: 128,
            batch_positions: 128,
        },
        true,
        None,
    )
    .unwrap();
    while !composition_trainer.is_complete() {
        composition_trainer
            .advance(16, std::time::Duration::from_secs(10))
            .unwrap();
    }
    let (occurrence_memory, _) = composition_trainer.finish().unwrap();
    let response_prompts = [
        "red fox finds green stone. blue bird finds amber seed. red fox keeps",
        "blue bird finds amber seed. red fox finds green stone. blue bird keeps",
    ];
    let response_documents: Vec<_> = response_prompts
        .iter()
        .enumerate()
        .map(|(index, prompt)| Document {
            id: format!("response-allocation-fit-{index}"),
            text: format!(
                "{prompt} {}.",
                if index == 0 {
                    "green stone"
                } else {
                    "amber seed"
                }
            ),
        })
        .collect();
    let response_spans = response_documents
        .iter()
        .zip(response_prompts)
        .map(|(document, prompt)| {
            let prefix = learned.encode(prompt).unwrap();
            let full = learned.encode(&document.text).unwrap();
            assert!(full.starts_with(&prefix));
            vec![MemoryReadTokenSpan {
                start: prefix.len(),
                end: full.len() + 1,
            }]
        })
        .collect();
    let supervision =
        MemoryReadSupervision::new(&learned, &response_documents, response_spans).unwrap();
    let fit_response = |advance_response_path| {
        let mut response_trainer = MemoryReadTrainer::new_with_response_state(
            &learned,
            &response_documents,
            MemoryReadFitConfig {
                advance_response_path,
                ..memory_config
            },
            MemoryReadSchedule {
                total_positions: 128,
                batch_positions: 128,
            },
            true,
            supervision.clone(),
        )
        .unwrap();
        while !response_trainer.is_complete() {
            response_trainer
                .advance(16, std::time::Duration::from_secs(10))
                .unwrap();
        }
        response_trainer.finish().unwrap().0
    };
    let response_memory = fit_response(false);
    let advancing_response_memory = fit_response(true);
    assert!(
        !response_memory
            .memory_read_config()
            .unwrap()
            .advance_response_path
    );
    assert!(
        advancing_response_memory
            .memory_read_config()
            .unwrap()
            .advance_response_path
    );
    assert_ne!(
        response_memory.memory_read_feature_layout(),
        advancing_response_memory.memory_read_feature_layout()
    );
    assert!(
        memory_fit.target_in_memory > 0,
        "fixture must exercise fitted memory alternatives"
    );
    let tokens = model.encode(&documents[0].text).unwrap();
    assert!(!tokens.is_empty());
    let mut measured_decisions = 0;
    for (readout, model) in [
        ("fixed_v1", model),
        ("learned_v1", learned),
        ("learned_memory_exact", exact_memory),
        ("learned_memory_with_aliases", with_memory),
        ("query_context_memory_with_aliases", query_context_memory),
        ("occurrence_composition", occurrence_memory),
        ("persistent_response", response_memory),
        ("persistent_advancing_response", advancing_response_memory),
    ] {
        let persistent = model
            .memory_read_version()
            .is_some_and(|version| version.ends_with("/5"));
        let word_cues =
            model.memory_cue_identity() == Some("leading-unicode-whitespace-word-equivalence/1");
        for control in [
            Control::Full,
            Control::GeometryDisabled,
            Control::ZetaDisabled,
            Control::H4Disabled,
            Control::OrientationDisabled,
            Control::PairedDisabled,
            Control::RadialDisabled,
            Control::HeatmapDisabled,
            Control::MemoryDisabled,
            Control::ResponseStateDisabled,
        ] {
            let active_response = persistent
                && control != Control::MemoryDisabled
                && control != Control::ResponseStateDisabled;
            let mut session = model.session(control).unwrap();
            session.observe(&model, BOS).unwrap();
            let before = session.state();
            ALLOCATIONS.with(|count| count.set(0));
            BYTES.with(|count| count.set(0));
            MEASURING.with(|enabled| enabled.set(true));
            let result = (|| {
                let mut stopped = false;
                for index in 0..1024 {
                    let response_step = active_response && index % 32 >= 16;
                    if persistent && index % 32 == 0 {
                        session.end_response(&model)?;
                    }
                    if response_step && (index % 32 == 16 || stopped) {
                        session.begin_response(&model)?;
                    }
                    let prediction = session.predict(&model)?;
                    let token = if response_step {
                        prediction.token
                    } else {
                        tokens[index % tokens.len()]
                    };
                    stopped = response_step && token == EOS;
                    session.observe(&model, token)?;
                }
                Ok::<_, uor_r4_core::native_geometric::Error>(())
            })();
            MEASURING.with(|enabled| enabled.set(false));
            let allocations = ALLOCATIONS.with(Cell::get);
            let bytes = BYTES.with(Cell::get);
            result.unwrap();
            measured_decisions += 1024;
            assert_eq!((allocations, bytes), (0, 0), "control {control:?}");
            let after = session.state();
            assert_eq!(after.context_capacity, 32);
            assert_eq!(after.retained_tokens, 32);
            assert_eq!(after.tokens_seen, 1025);
            assert_eq!(session.work.evictions, 993);
            assert_eq!(after.ring_storage_bytes, before.ring_storage_bytes);
            assert_eq!(
                after.candidate_storage_bytes,
                before.candidate_storage_bytes
            );
            assert!(session.candidates().len() <= 3);
            assert!(session.work.candidate_offers <= 1024 * (3 + 26 * 4));
            assert!(session.work.candidate_evaluations <= session.work.candidate_offers);
            if active_response {
                assert!(session.work.response_query_captures >= 32);
                assert!(session.work.response_query_captures <= 512);
                assert_eq!(
                    session.work.response_commits + session.work.response_stops,
                    512
                );
                assert!(session.work.response_reference_reads > 0);
                assert_eq!(session.work.response_mismatches, 0);
            } else {
                assert_eq!(session.work.response_query_captures, 0);
                assert_eq!(session.work.response_commits, 0);
                assert_eq!(session.work.response_reference_reads, 0);
            }
            let captures = session.work.response_query_captures;
            let memory_bytes = if let (Some(before), Some(after)) =
                (before.memory_read, after.memory_read)
            {
                assert_eq!(after.retained_tokens, 32);
                assert_eq!(before.ring_storage_bytes, after.ring_storage_bytes);
                assert_eq!(before.index_storage_bytes, after.index_storage_bytes);
                assert_eq!(
                    before.candidate_storage_bytes,
                    after.candidate_storage_bytes
                );
                assert!(session.work.memory_index_writes > 0);
                assert!(
                    session.work.memory_candidates
                        <= 1024 * (memory_config.candidate_limit + usize::from(persistent)) as u64
                );
                assert!(
                    session.work.memory_index_reads
                        <= 1025
                            * (memory_config.source_offsets
                                * (memory_config.postings_per_address - 1))
                                as u64
                            + 1024 * memory_config.candidate_limit as u64
                            + captures * memory_config.candidate_limit as u64
                );
                assert!(
                    session.work.memory_index_writes
                        <= 1025
                            * (memory_config.source_offsets * memory_config.postings_per_address)
                                as u64
                );
                assert!(session.work.memory_score_lookups <= session.work.memory_candidates * 18);
                assert_eq!(
                    before.composed_candidate_storage_bytes,
                    after.composed_candidate_storage_bytes
                );
                assert_eq!(
                    before.composition_feature_storage_bytes,
                    after.composition_feature_storage_bytes
                );
                assert_eq!(before.response_storage_bytes, after.response_storage_bytes);
                assert_eq!(after.response_storage_bytes > 0, persistent);
                if word_cues {
                    assert!(session.work.memory_cue_reads > 0);
                } else {
                    assert_eq!(session.work.memory_cue_reads, 0);
                }
                assert!(
                    session.work.memory_cue_reads
                        <= 1025 * memory_config.source_offsets as u64
                            + 1024 * memory_config.candidate_limit as u64
                            + captures * memory_config.candidate_limit as u64
                );
                if control == Control::MemoryDisabled {
                    assert_eq!(session.work.memory_candidates, 0);
                    assert_eq!(session.work.memory_score_lookups, 0);
                    // Observation still updates the same memory state; only
                    // prediction-side cue reads disappear under this control.
                    assert!(
                        session.work.memory_cue_reads <= 1025 * memory_config.source_offsets as u64
                    );
                } else {
                    assert!(session.work.memory_candidates > 0);
                    assert!(session.work.memory_score_lookups > 0);
                }
                after.ring_storage_bytes
                    + after.index_storage_bytes
                    + after.candidate_storage_bytes
                    + after.composed_candidate_storage_bytes
                    + after.composition_feature_storage_bytes
                    + after.response_storage_bytes
            } else {
                assert!(model.memory_read_version().is_none());
                assert_eq!(session.work.memory_candidates, 0);
                assert_eq!(session.work.memory_cue_reads, 0);
                0
            };
            println!("{readout} {control:?}: decisions=1024 evictions=993 allocations={allocations} bytes={bytes} ring_bytes={} candidate_bytes={} offers={} evaluations={} memory_bytes={memory_bytes} memory_candidates={} memory_score_lookups={} memory_cue_reads={} response_captures={} response_commits={} response_requeries={} response_continuations={} response_stops={}",
            after.ring_storage_bytes, after.candidate_storage_bytes, session.work.candidate_offers, session.work.candidate_evaluations,
            session.work.memory_candidates,session.work.memory_score_lookups,session.work.memory_cue_reads,
            captures, session.work.response_commits, session.work.response_requeries, session.work.response_continuations, session.work.response_stops);
        }
    }
    assert_eq!(measured_decisions, 8 * 10 * 1024);
}

#[test]
fn source_routing_observe_select_and_copy_are_allocation_free() {
    use uor_r4_core::native_geometric::SourceRoutingConfig;
    let docs = [
        ValueExample {
            id: "source-allocation-copy".into(),
            prompt: "holder in alpha. Where is holder? Answer:".into(),
            response: " alpha.\n".into(),
        },
        ValueExample {
            id: "source-allocation-none".into(),
            prompt: "alpha in city. Where is missing? Answer:".into(),
            response: " Unknown.\n".into(),
        },
    ];
    let (parent, _) = copy_fixture::fitted_composed()
        .fit_role_read(
            &docs,
            ResponseEntryFitConfig {
                epochs: 8,
                ..ResponseEntryFitConfig::default()
            },
        )
        .unwrap();
    let (model, _) = parent
        .fit_source_routing(
            &docs,
            SourceRoutingConfig {
                learned_features: 8,
                passes: 1,
                proposals: 2,
                role_context_only: true,
                ..SourceRoutingConfig::default()
            },
        )
        .unwrap();
    let tokens = model.encode(&docs[0].prompt).unwrap();
    let mut s = model.session(Control::Full).unwrap();
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|n| n.set(true));
    let result = (|| {
        s.observe(&model, BOS)?;
        for &t in &tokens {
            s.observe(&model, t)?;
        }
        s.begin_response(&model)?;
        for _ in 0..24 {
            let p = s.predict(&model)?;
            s.observe(&model, p.token)?;
            if p.token == EOS {
                break;
            }
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|n| n.set(false));
    result.unwrap();
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
    assert_eq!(BYTES.with(Cell::get), 0);
    assert!(s.work.word_copy.routing.predictions > 0);
}

/// Actual retained artifact exercise. The large local learned artifact is not a
/// repository fixture; CI's small-fixture census does not claim this execution.
#[test]
#[ignore = "requires R4_DEPENDENT_READ_MODEL retained development artifact"]
fn native_dependent_artifact_copy_is_allocation_free() {
    let path = std::env::var("R4_DEPENDENT_READ_MODEL").expect("named model path");
    let model =
        uor_r4_core::native_geometric::Model::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let tokens = model
        .encode(
            "casket in elvin. elvin in Bremen. Question: Where is the location of casket? Answer:",
        )
        .unwrap();
    let mut session = model.session(Control::Full).unwrap();
    let mut selected = false;
    ALLOCATIONS.with(|v| v.set(0));
    BYTES.with(|v| v.set(0));
    MEASURING.with(|v| v.set(true));
    let result = (|| {
        session.observe(&model, BOS)?;
        for &token in &tokens {
            session.observe(&model, token)?;
        }
        session.begin_response(&model)?;
        for _ in 0..32 {
            let p = session.predict(&model)?;
            selected |= session
                .word_copy_decision()
                .is_some_and(|d| d.dependency.is_some_and(|ids| ids[1] != 0));
            session.observe(&model, p.token)?;
            if p.token == EOS {
                break;
            }
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|v| v.set(false));
    result.unwrap();
    assert!(selected, "dependent path was not exercised");
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    println!("dependent exact two-link ingest/select/observe/copy: allocations=0 bytes=0");
}

#[test]
#[ignore = "requires R4_TYPED_ROUTING_MODEL learned development artifact"]
fn native_typed_artifact_routing_is_allocation_free() {
    let path = std::env::var("R4_TYPED_ROUTING_MODEL").expect("named model path");
    let model =
        uor_r4_core::native_geometric::Model::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let first=model.encode("User: suri has 13 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:").unwrap();
    let second = model
        .encode("User: There are 5 new coins. Add the new coins to the previous total.\nAssistant:")
        .unwrap();
    let mut session = model.session(Control::Full).unwrap();
    ALLOCATIONS.with(|v| v.set(0));
    BYTES.with(|v| v.set(0));
    MEASURING.with(|v| v.set(true));
    let result = (|| {
        session.observe(&model, BOS)?;
        for prompt in [&first, &second] {
            for &token in prompt {
                session.observe(&model, token)?;
            }
            session.begin_response(&model)?;
            for _ in 0..32 {
                let token = session.predict(&model)?.token;
                session.observe(&model, token)?;
                if token == EOS {
                    break;
                }
            }
            session.end_response(&model)?;
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|v| v.set(false));
    result.unwrap();
    assert!(session.work.values.routing.predictions > 0);
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    println!("actual typed geometric routing/commit: allocations=0 bytes=0");
}

#[test]
#[ignore = "requires R4_TYPED_ROLES_MODEL retained role selection artifact"]
fn native_typed_role_artifact_is_allocation_free() {
    let path = std::env::var("R4_TYPED_ROLES_MODEL").expect("named model path");
    let model =
        uor_r4_core::native_geometric::Model::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let prompts=[
        "User: suri has 13 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:",
        "User: There are 5 new coins. Add the new coins to the previous total.\nAssistant:",
        "User: Repeat the original total.\nAssistant:",
        "User: There are 2 extra coins. Add the extra coins to the updated total.\nAssistant:",
    ].map(|p|model.encode(p).unwrap());
    let mut s = model.session(Control::Full).unwrap();
    let mut answers = [None; 4];
    ALLOCATIONS.with(|v| v.set(0));
    BYTES.with(|v| v.set(0));
    MEASURING.with(|v| v.set(true));
    let result = (|| {
        s.observe(&model, BOS)?;
        for (i, prompt) in prompts.iter().enumerate() {
            for &token in prompt {
                s.observe(&model, token)?;
            }
            s.begin_response(&model)?;
            for _ in 0..32 {
                let token = s.predict(&model)?.token;
                if let Some(d) = s.value_decision().filter(|d| d.cursor == 0) {
                    answers[i] = Some(d.value);
                }
                s.observe(&model, token)?;
                if token == EOS {
                    break;
                }
            }
            s.end_response(&model)?;
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|v| v.set(false));
    result.unwrap();
    assert_eq!(answers, [Some(17), Some(22), Some(17), Some(24)]);
    assert!(s.work.values.alias_self_add_rejections > 0);
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    let checkpoint = s.checkpoint().unwrap();
    let restored = model.restore_session(&checkpoint).unwrap();
    assert_eq!(restored.checkpoint().unwrap(), checkpoint);
    let mut forged: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    assert!(forged["values"]["query_boundary"].is_number());
    forged["values"]
        .as_object_mut()
        .unwrap()
        .remove("query_boundary");
    assert!(model
        .restore_session(&serde_json::to_vec(&forged).unwrap())
        .is_err());
    forged["values"]["query_boundary"] = serde_json::json!(u64::MAX);
    assert!(model
        .restore_session(&serde_json::to_vec(&forged).unwrap())
        .is_err());
    println!("actual competing typed roles: four committed values correct; allocations=0 bytes=0; boundary restore and rejection checks PASS");
}

#[test]
#[ignore = "requires R4_INDEPENDENT_MODEL retained operand-provenance artifact"]
fn native_independent_artifact_is_allocation_free() {
    use uor_r4_core::native_geometric::Model;
    let model =
        Model::from_bytes(&std::fs::read(std::env::var("R4_INDEPENDENT_MODEL").unwrap()).unwrap())
            .unwrap();
    let prompts = [
        "User: suri has 13 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:",
        "User: mira has 5 coins. neri has 9 coins.\nUser: What is the sum of mira's and neri's coins?\nAssistant:",
        "User: Repeat the total for mira and neri.\nAssistant:",
    ].map(|p|model.encode(p).unwrap());
    let expected = ["17.\n", "14.\n", "14.\n"];
    let mut outputs = [[0; 32]; 3];
    let mut lengths = [0; 3];
    let mut ended = [false; 3];
    let mut s = model.session(Control::Full).unwrap();
    ALLOCATIONS.with(|v| v.set(0));
    BYTES.with(|v| v.set(0));
    MEASURING.with(|v| v.set(true));
    let result = (|| {
        s.observe(&model, BOS)?;
        for (i, prompt) in prompts.iter().enumerate() {
            for &t in prompt {
                s.observe(&model, t)?;
            }
            s.begin_response(&model)?;
            for _ in 0..32 {
                let token = s.predict(&model)?.token;
                s.observe(&model, token)?;
                if token == EOS {
                    ended[i] = true;
                    break;
                }
                outputs[i][lengths[i]] = token;
                lengths[i] += 1;
            }
            s.end_response(&model)?;
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|v| v.set(false));
    result.unwrap();
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    for i in 0..3 {
        assert!(ended[i]);
        assert_eq!(
            model.decode(&outputs[i][..lengths[i]]).unwrap(),
            expected[i].as_bytes()
        );
    }
    let checkpoint = s.checkpoint().unwrap();
    assert_eq!(
        model
            .restore_session(&checkpoint)
            .unwrap()
            .checkpoint()
            .unwrap(),
        checkpoint
    );
    let mut wire: serde_json::Value = serde_json::from_slice(&model.to_bytes().unwrap()).unwrap();
    if wire.get("typed_literals").is_some() {
        let parent_bytes = std::fs::read(std::env::var("R4_LITERAL_PARENT").unwrap()).unwrap();
        let mut parent: serde_json::Value = serde_json::from_slice(&parent_bytes).unwrap();
        assert_eq!(
            wire["typed_literals"]["router"]["parent_artifact"],
            parent["artifact_cid"]
        );
        let mut protected = wire.clone();
        // A source-only refinement retains the exact earlier router as its
        // provenance witness; restore it before checking the literal parent.
        if let Some(witness) = protected
            .as_object_mut()
            .unwrap()
            .remove("source_routing_refinement")
        {
            protected["source_routing"] = witness["previous"].clone();
        }
        protected.as_object_mut().unwrap().remove("typed_literals");
        protected
            .as_object_mut()
            .unwrap()
            .remove("no_read_completion");
        for key in ["artifact_cid", "uor_model_address"] {
            protected.as_object_mut().unwrap().remove(key);
            parent.as_object_mut().unwrap().remove(key);
        }
        assert_eq!(
            protected, parent,
            "every inherited model field must remain unchanged"
        );
        let mut invalid = wire.clone();
        invalid["typed_literals"]["literal_answers"] = serde_json::json!(false);
        assert!(Model::from_bytes(&serde_json::to_vec(&invalid).unwrap()).is_err());
        println!("literal admission: entire parent unchanged; literal flag mutation rejected");
    }
    wire["typed_roles"]
        .as_object_mut()
        .unwrap()
        .remove("operand_provenance");
    assert!(Model::from_bytes(&serde_json::to_vec(&wire).unwrap()).is_err());
    println!("actual independent sums and named Copy: all bytes and EOS correct; allocations=0 bytes=0; checkpoint roundtrip and unbound flag rejection PASS");
}

#[test]
#[ignore = "requires the explicitly supplied accepted NoRead completion artifact"]
fn native_no_read_artifact_is_allocation_free() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_NO_READ_MODEL").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let mut outer: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(outer
        .as_object_mut()
        .unwrap()
        .remove("no_read_completion")
        .is_some());
    let mut parent: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("R4_NO_READ_PARENT").unwrap()).unwrap(),
    )
    .unwrap();
    for doc in [&mut outer, &mut parent] {
        for k in ["artifact_cid", "uor_model_address"] {
            doc.as_object_mut().unwrap().remove(k);
        }
    }
    assert_eq!(outer, parent, "every parent parameter must be unchanged");
    let prompt=model.encode("User: leni has 17 coins. varo has -5 coins. tavi has 301 coins.\nUser: Where is the location of leni?\nAssistant:").unwrap();
    let mut s = model.session(Control::Full).unwrap();
    let mut out = [EOS; 32];
    let mut n = 0;
    ALLOCATIONS.with(|v| v.set(0));
    BYTES.with(|v| v.set(0));
    MEASURING.with(|v| v.set(true));
    s.observe(&model, BOS).unwrap();
    for t in prompt {
        s.observe(&model, t).unwrap();
    }
    s.begin_response(&model).unwrap();
    for slot in &mut out {
        let p = s.predict(&model).unwrap();
        *slot = p.token;
        n += 1;
        s.observe(&model, p.token).unwrap();
        if p.token == EOS {
            break;
        }
    }
    MEASURING.with(|v| v.set(false));
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    assert_eq!(out[n - 1], EOS);
    assert_eq!(model.decode(&out[..n]).unwrap(), b" Unknown.\n");
    let checkpoint = s.checkpoint().unwrap();
    assert_eq!(
        model
            .restore_session(&checkpoint)
            .unwrap()
            .checkpoint()
            .unwrap(),
        checkpoint
    );
    println!("actual NoRead answer: exact bytes and EOS; allocations=0 bytes=0; complete parent equality and checkpoint roundtrip PASS");
}

/// Exercises the fitted replacement router itself, including the observation
/// boundary that turns a transient source choice into a committed copy/NoRead.
#[test]
#[ignore = "requires R4_SOURCE_NO_READ_MODEL and R4_SOURCE_NO_READ_PARENT artifacts"]
fn native_source_no_read_artifact_is_allocation_free_and_causal() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_SOURCE_NO_READ_MODEL").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut parent: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("R4_SOURCE_NO_READ_PARENT").unwrap()).unwrap(),
    )
    .unwrap();
    let witness = wire
        .as_object_mut()
        .unwrap()
        .remove("source_routing_refinement")
        .expect("actual replacement-router provenance");
    assert_eq!(witness["parent_artifact"], parent["artifact_cid"]);
    assert_eq!(witness["previous"], parent["source_routing"]);
    wire["source_routing"] = witness["previous"].clone();
    for document in [&mut wire, &mut parent] {
        for key in ["artifact_cid", "uor_model_address"] {
            document.as_object_mut().unwrap().remove(key);
        }
    }
    assert_eq!(
        wire, parent,
        "all fields outside the replaced router are unchanged"
    );

    let cases = [
        (
            "User: varo has -5 coins. leni has 17 coins. tavi has 301 coins.\nUser: Where is the location of leni?\nAssistant:",
            b" Unknown.\n".as_slice(),
            false,
        ),
        (
            "User: varo has stones. leni has coins.\nUser: What does leni have?\nAssistant:",
            b" coins.\n".as_slice(),
            true,
        ),
    ];
    for (prompt, expected, supported) in cases {
        let tokens = model.encode(prompt).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        let ingest = (|| {
            session.observe(&model, BOS)?;
            for &token in &tokens {
                session.observe(&model, token)?;
            }
            session.begin_response(&model)
        })();
        MEASURING.with(|v| v.set(false));
        ingest.unwrap();
        let boundary = session.checkpoint().unwrap();
        let before: serde_json::Value = serde_json::from_slice(&boundary).unwrap();
        assert!(before["word_copy"]["read_commit"].is_null());
        let commits = session.work.word_copy.selector.commits;

        MEASURING.with(|v| v.set(true));
        let first = session.predict(&model);
        let selected = session.word_copy_decision();
        let repeated = session.predict(&model);
        let repeated_choice = session.word_copy_decision();
        MEASURING.with(|v| v.set(false));
        let first = first.unwrap();
        assert_eq!(first, repeated.unwrap());
        let selected = selected.expect("joint source or NoRead selection exercised");
        assert_eq!(Some(selected), repeated_choice);
        assert_eq!(session.work.word_copy.selector.commits, commits);
        assert_eq!(selected.action == WordCopyAction::NoRead, !supported);
        if supported {
            assert!(matches!(
                selected.action,
                WordCopyAction::Read | WordCopyAction::Prepare
            ));
        }
        let transient: serde_json::Value =
            serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
        assert_eq!(
            transient["word_copy"], before["word_copy"],
            "prediction cannot commit a source"
        );

        MEASURING.with(|v| v.set(true));
        let observed = session.observe(&model, first.token);
        MEASURING.with(|v| v.set(false));
        observed.unwrap();
        assert_eq!(session.work.word_copy.selector.commits, commits + 1);
        let committed = session.checkpoint().unwrap();
        let state: serde_json::Value = serde_json::from_slice(&committed).unwrap();
        let read = &state["word_copy"]["read_commit"];
        assert!(read.is_object());
        assert_eq!(read["token"], serde_json::json!(first.token));
        assert_eq!(read["source_end"], serde_json::json!(selected.source_end));
        assert_eq!(
            read["source_byte_end"],
            serde_json::json!(selected.source_byte_end)
        );
        if supported {
            assert_eq!(read["source"], serde_json::json!(selected.word_index));
        } else {
            assert!(read["source"].is_null());
        }
        assert_eq!(
            model
                .restore_session(&committed)
                .unwrap()
                .checkpoint()
                .unwrap(),
            committed
        );

        let mut output = [EOS; 32];
        output[0] = first.token;
        let mut length = 1;
        MEASURING.with(|v| v.set(true));
        let generated = (|| {
            while output[length - 1] != EOS && length < output.len() {
                let token = session.predict(&model)?.token;
                session.observe(&model, token)?;
                output[length] = token;
                length += 1;
            }
            session.end_response(&model)
        })();
        MEASURING.with(|v| v.set(false));
        generated.unwrap();
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
        assert_eq!(output[length - 1], EOS, "complete answer must terminate");
        assert_eq!(model.decode(&output[..length]).unwrap(), expected);

        // Restore an uncommitted boundary and observe a different legal byte.
        // Checkpoint parsing/serialization are deliberately outside the census.
        let mut mismatch = model.restore_session(&boundary).unwrap();
        let predicted = mismatch.predict(&model).unwrap();
        let mismatch_commits = mismatch.work.word_copy.selector.commits;
        let other = if predicted.token == u32::from(b'x') + 2 {
            u32::from(b'y') + 2
        } else {
            u32::from(b'x') + 2
        };
        mismatch.observe(&model, other).unwrap();
        assert_eq!(mismatch.work.word_copy.selector.commits, mismatch_commits);
        let mismatch_state: serde_json::Value =
            serde_json::from_slice(&mismatch.checkpoint().unwrap()).unwrap();
        assert!(mismatch_state["word_copy"]["read_commit"].is_null());
    }
    println!("actual source/NoRead: unsupported and supported complete answers; allocations=0 bytes=0; transient/matching/mismatched commit, checkpoint restore and full parent lineage PASS");
}

#[test]
#[ignore = "requires R4_JOINT_ADMISSION_MODEL and R4_JOINT_ADMISSION_PARENT"]
fn native_joint_admission_preserves_parent_and_executes_only_admitted_payloads() {
    use std::time::Instant;
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_JOINT_ADMISSION_MODEL").unwrap()).unwrap();
    let parent_bytes = std::fs::read(std::env::var("R4_JOINT_ADMISSION_PARENT").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let parent_model = Model::from_bytes(&parent_bytes).unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut parent: serde_json::Value = serde_json::from_slice(&parent_bytes).unwrap();
    let gate = wire
        .as_object_mut()
        .unwrap()
        .remove("joint_admission")
        .unwrap();
    assert_eq!(gate["router"]["parent_artifact"], parent["artifact_cid"]);
    for w in [&mut wire, &mut parent] {
        w.as_object_mut().unwrap().remove("artifact_cid");
        w.as_object_mut().unwrap().remove("uor_model_address");
    }
    assert_eq!(wire, parent, "complete inherited model equality");
    for field in 0..4 {
        let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        match field {
            0 => bad["joint_admission"]["router"]["parent_artifact"] = "wrong-parent".into(),
            1 => bad["joint_admission"]["router"]["codes"][0]["roots"][0] = 120.into(),
            2 => bad["joint_admission"]["router"]["config"]["role_context_only"] = true.into(),
            _ => bad["typed_literals"]["router"]["biases"][0] = 32.into(),
        }
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let cases = [
        ("User: cyra has 13 coins. cyra lives in Paris.\nUser: Where is cyra?\nAssistant:"," Paris.\n",false),
        ("User: ada has 13 coins. other has 7 coins. ada lives in Rome.\nUser: How many coins does ada have?\nAssistant:","13.\n",true),
        ("User: varo has -5 coins. leni has 17 coins. tavi has 301 coins.\nUser: Where is the location of leni?\nAssistant:"," Unknown.\n",false),
    ];
    let mut times = Vec::with_capacity(96);
    let mut total_rejections = 0;
    for (prompt, expected, numeric) in cases {
        assert_eq!(
            model
                .generate(prompt, 32, Control::JointAdmissionDisabled)
                .unwrap()
                .text,
            parent_model
                .generate(prompt, 32, Control::Full)
                .unwrap()
                .text
        );
        let tokens = model.encode(prompt).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        let ingest = (|| {
            session.observe(&model, BOS)?;
            for &t in &tokens {
                session.observe(&model, t)?;
            }
            session.begin_response(&model)
        })();
        MEASURING.with(|v| v.set(false));
        ingest.unwrap();
        let boundary = session.checkpoint().unwrap();
        MEASURING.with(|v| v.set(true));
        let first = session.predict(&model);
        let repeated = session.predict(&model);
        MEASURING.with(|v| v.set(false));
        assert_eq!(first.unwrap(), repeated.unwrap());
        assert_eq!(session.work.values.derived_writes, 0);
        assert_eq!(session.work.values.emission_commits, 0);
        if !numeric {
            assert!(session.value_decision().is_none());
            assert_eq!(
                session.work.values.operator_executions, 0,
                "declined arithmetic was never executed"
            );
        } else {
            assert!(session.value_decision().is_some());
        }
        total_rejections += session.work.values.admission_rejections;
        // A different observation may not commit the predicted numeric value or source.
        let mut mismatch = model.restore_session(&boundary).unwrap();
        let p = mismatch.predict(&model).unwrap();
        let wrong = if p.token == u32::from(b'x') + 2 {
            u32::from(b'y') + 2
        } else {
            u32::from(b'x') + 2
        };
        mismatch.observe(&model, wrong).unwrap();
        assert_eq!(mismatch.work.values.derived_writes, 0);
        assert_eq!(mismatch.work.word_copy.selector.commits, 0);
        let mut output = [EOS; 32];
        let mut length = 0;
        loop {
            MEASURING.with(|v| v.set(true));
            let started = Instant::now();
            let step = (|| {
                let t = session.predict(&model)?.token;
                session.observe(&model, t)?;
                Ok::<_, uor_r4_core::native_geometric::Error>(t)
            })();
            let ns = started.elapsed().as_nanos();
            MEASURING.with(|v| v.set(false));
            let t = step.unwrap();
            times.push(ns);
            output[length] = t;
            length += 1;
            if length == 1 {
                let checkpoint = session.checkpoint().unwrap();
                assert_eq!(
                    model
                        .restore_session(&checkpoint)
                        .unwrap()
                        .checkpoint()
                        .unwrap(),
                    checkpoint
                );
                if numeric {
                    assert_eq!(session.work.values.derived_writes, 1);
                }
            }
            if t == EOS || length == output.len() {
                break;
            }
        }
        assert_eq!(output[length - 1], EOS);
        assert_eq!(
            model.decode(&output[..length]).unwrap(),
            expected.as_bytes()
        );
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    }
    assert!(
        total_rejections > 0,
        "actual learned declining path executed"
    );
    times.sort_unstable();
    println!("joint admission: exact parent, valid/invalid load, numeric and word/NoRead outputs, observation-only commitment, checkpoint and zero allocations PASS; warm predict+observe samples={}, median_ns={}, max_ns={} (encoding/loading/ingestion/checkpoints excluded)",times.len(),times[times.len()/2],times.last().unwrap());
}

#[test]
#[ignore = "requires R4_LITERAL_REFINEMENT_MODEL and R4_LITERAL_REFINEMENT_PARENT"]
fn native_literal_refinement_preserves_parent_and_binds_operands() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_LITERAL_REFINEMENT_MODEL").unwrap()).unwrap();
    let parent_bytes =
        std::fs::read(std::env::var("R4_LITERAL_REFINEMENT_PARENT").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let old = Model::from_bytes(&parent_bytes).unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut parent: serde_json::Value = serde_json::from_slice(&parent_bytes).unwrap();
    let witness = wire
        .as_object_mut()
        .unwrap()
        .remove("literal_routing_refinement")
        .unwrap();
    assert_eq!(witness["parent_artifact"], parent["artifact_cid"]);
    wire["typed_literals"]["router"] = witness["previous"].clone();
    for j in [&mut wire, &mut parent] {
        j.as_object_mut().unwrap().remove("artifact_cid");
        j.as_object_mut().unwrap().remove("uor_model_address");
    }
    assert_eq!(
        wire, parent,
        "all other parameters, dictionary and parent lineage are fixed"
    );
    for k in 0..3 {
        let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        match k {
            0 => bad["literal_routing_refinement"]["parent_artifact"] = "wrong-parent".into(),
            1 => bad["typed_literals"]["router"]["codes"][0]["roots"][0] = 120.into(),
            _ => bad["joint_admission"]["router"]["biases"][0] = 1000.into(),
        }
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let cases = [
        ("User: adara lives in Rome. adara has 13 coins. cyris has 7 coins.\nUser: How many coins does adara have?\nAssistant:","13.\n",true),
        ("User: cyris has 7 coins. adara has 13 coins. adara lives in Rome.\nUser: How many coins does cyris have?\nAssistant:","7.\n",true),
        ("User: cyra has 13 coins. cyra lives in Paris.\nUser: Where is cyra?\nAssistant:"," Paris.\n",false),
    ];
    let mut times = [0_u128; 96];
    let mut time_len = 0;
    for (prompt, expected, numeric) in cases {
        assert_eq!(
            model
                .generate(prompt, 32, Control::LiteralRefinementDisabled)
                .unwrap()
                .text,
            old.generate(prompt, 32, Control::Full).unwrap().text
        );
        let tokens = model.encode(prompt).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        session.observe(&model, BOS).unwrap();
        for t in tokens {
            session.observe(&model, t).unwrap();
        }
        session.begin_response(&model).unwrap();
        let first = session.predict(&model).unwrap();
        assert_eq!(first, session.predict(&model).unwrap());
        assert_eq!(session.work.values.derived_writes, 0);
        MEASURING.with(|v| v.set(false));
        let checkpoint = session.checkpoint().unwrap();
        let mut wrong = model.restore_session(&checkpoint).unwrap();
        let p = wrong.predict(&model).unwrap();
        wrong
            .observe(&model, if p.token == 2 { 3 } else { 2 })
            .unwrap();
        assert_eq!(wrong.work.values.derived_writes, 0);
        let mut out = [EOS; 32];
        let mut len = 0;
        MEASURING.with(|v| v.set(true));
        loop {
            let started = std::time::Instant::now();
            let t = session.predict(&model).unwrap().token;
            session.observe(&model, t).unwrap();
            times[time_len] = started.elapsed().as_nanos();
            time_len += 1;
            out[len] = t;
            len += 1;
            if t == EOS || len == 32 {
                break;
            }
        }
        MEASURING.with(|v| v.set(false));
        assert_eq!(out[len - 1], EOS);
        assert_eq!(model.decode(&out[..len]).unwrap(), expected.as_bytes());
        assert_eq!(session.work.values.derived_writes, u64::from(numeric));
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
        let checkpoint = session.checkpoint().unwrap();
        assert_eq!(
            model
                .restore_session(&checkpoint)
                .unwrap()
                .checkpoint()
                .unwrap(),
            checkpoint
        );
    }
    times[..time_len].sort_unstable();
    println!("warm predict+observe samples={}, median_ns={}, max_ns={} (loading/encoding/ingestion/checkpoints excluded)",time_len,times[time_len/2],times[time_len-1]);
    println!("literal refinement: exact parent restoration, malformed-artifact rejection, disabled-parent behavior, changed-order operands, source preservation, observation-only writes, checkpoints and zero allocations PASS");
}

#[test]
#[ignore = "requires R4_SOURCE_CONTEXT_MODEL and R4_SOURCE_CONTEXT_PARENT"]
fn native_source_context_retains_owner_and_preserves_causal_copy() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_SOURCE_CONTEXT_MODEL").unwrap()).unwrap();
    let parent_bytes = std::fs::read(std::env::var("R4_SOURCE_CONTEXT_PARENT").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut parent: serde_json::Value = serde_json::from_slice(&parent_bytes).unwrap();
    let witness = wire
        .as_object_mut()
        .unwrap()
        .remove("source_context")
        .unwrap();
    assert_eq!(witness["parent_artifact"], parent["artifact_cid"]);
    wire["source_routing"] = witness["previous"].clone();
    for j in [&mut wire, &mut parent] {
        j.as_object_mut().unwrap().remove("artifact_cid");
        j.as_object_mut().unwrap().remove("uor_model_address");
    }
    assert_eq!(
        wire, parent,
        "all other parameters, dictionary and parent lineage are fixed"
    );
    for k in 0..3 {
        let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        match k {
            0 => bad["source_context"]["parent_artifact"] = "wrong-parent".into(),
            1 => bad["source_routing"]["codes"][0]["roots"][0] = 120.into(),
            _ => bad["joint_admission"]["router"]["biases"][0] = 1000.into(),
        }
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let cases = [
        ("User: ada lives in Rome. ada has 13 coins. other has 7 coins.\nUser: Where is the location of ada?\nAssistant:"," Rome.\n",false),
        ("User: cyra lives in Rome. ada has 13 coins. other has 7 coins.\nUser: Where is the location of ada?\nAssistant:"," Unknown.\n",false),
        ("User: adara lives in Rome. adara has 13 coins. cyris has 7 coins.\nUser: How many coins does adara have?\nAssistant:","13.\n",true),
    ];
    let mut times = [0_u128; 96];
    let mut time_len = 0;
    for (prompt, expected, numeric) in cases {
        let tokens = model.encode(prompt).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        session.observe(&model, BOS).unwrap();
        for t in tokens {
            session.observe(&model, t).unwrap();
        }
        session.begin_response(&model).unwrap();
        let first = session.predict(&model).unwrap();
        assert_eq!(first, session.predict(&model).unwrap());
        assert_eq!(session.work.values.derived_writes, 0);
        MEASURING.with(|v| v.set(false));
        let checkpoint = session.checkpoint().unwrap();
        let mut wrong = model.restore_session(&checkpoint).unwrap();
        let p = wrong.predict(&model).unwrap();
        wrong
            .observe(&model, if p.token == 2 { 3 } else { 2 })
            .unwrap();
        assert_eq!(wrong.work.values.derived_writes, 0);
        let mut out = [EOS; 32];
        let mut len = 0;
        MEASURING.with(|v| v.set(true));
        loop {
            let started = std::time::Instant::now();
            let t = session.predict(&model).unwrap().token;
            session.observe(&model, t).unwrap();
            times[time_len] = started.elapsed().as_nanos();
            time_len += 1;
            out[len] = t;
            len += 1;
            if t == EOS || len == 32 {
                break;
            }
        }
        MEASURING.with(|v| v.set(false));
        assert_eq!(out[len - 1], EOS);
        assert_eq!(model.decode(&out[..len]).unwrap(), expected.as_bytes());
        assert_eq!(session.work.values.derived_writes, u64::from(numeric));
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
        let checkpoint = session.checkpoint().unwrap();
        assert_eq!(
            model
                .restore_session(&checkpoint)
                .unwrap()
                .checkpoint()
                .unwrap(),
            checkpoint
        );
    }
    times[..time_len].sort_unstable();
    println!("warm predict+observe samples={}, median_ns={}, max_ns={} (loading/encoding/ingestion/checkpoints excluded)",time_len,times[time_len/2],times[time_len-1]);
    println!("source context: exact parent restoration, malformed-artifact rejection, evicted-owner contrast, numeric binding, source preservation, observation-only writes, checkpoints and zero allocations PASS");
}

#[test]
#[ignore = "requires R4_SOURCE_SPAN_MODEL and R4_SOURCE_SPAN_PARENT"]
fn native_source_span_preserves_commit_checkpoints_and_zero_allocation() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_SOURCE_SPAN_MODEL").unwrap()).unwrap();
    let parent_bytes = std::fs::read(std::env::var("R4_SOURCE_SPAN_PARENT").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut parent: serde_json::Value = serde_json::from_slice(&parent_bytes).unwrap();
    let contextual = wire["source_span_context"].is_array();
    wire.as_object_mut().unwrap().remove("source_span_context");
    let block = wire.as_object_mut().unwrap().remove("source_span").unwrap();
    assert_eq!(block["parent_artifact"], parent["artifact_cid"]);
    for j in [&mut wire, &mut parent] {
        j.as_object_mut().unwrap().remove("artifact_cid");
        j.as_object_mut().unwrap().remove("uor_model_address");
    }
    assert_eq!(
        wire, parent,
        "the complete selector and parent parameters are unchanged"
    );
    let mut invalid: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    invalid["source_span"]["codes"][0]["roots"][0] = 120.into();
    assert!(Model::from_bytes(&serde_json::to_vec(&invalid).unwrap()).is_err());
    let mut times = [0_u128; 96];
    let mut samples = 0;
    let mut cases = vec![
        (
            "User: ada lives in New York.\nUser: Where is ada?\nAssistant:",
            " New York.\n",
            1,
        ),
        (
            "User: ada lives in Rio de Janeiro.\nUser: Where is ada?\nAssistant:",
            " Rio de Janeiro.\n",
            2,
        ),
    ];
    if contextual {
        cases.push((
            "Record: Dover holds cyra. Where is cyra? Answer:",
            " Dover.\n",
            0,
        ));
        let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        bad["source_span_context"] = false.into();
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        bad["source_span_context"][0]["prime"] = 0.into();
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        bad["source_span_context"]
            .as_array_mut()
            .unwrap()
            .swap(0, 1);
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    for (prompt, expected, extra) in cases {
        let tokens = model.encode(prompt).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        session.observe(&model, BOS).unwrap();
        for t in tokens {
            session.observe(&model, t).unwrap();
        }
        session.begin_response(&model).unwrap();
        let first = session.predict(&model).unwrap();
        assert_eq!(first, session.predict(&model).unwrap());
        assert_eq!(session.word_copy_decision().unwrap().span_words, extra);
        assert_eq!(session.work.word_copy.selector.commits, 0);
        MEASURING.with(|v| v.set(false));
        let mut out = [EOS; 32];
        let mut len = 0;
        loop {
            // Pending predictions are omitted and reconstructed by checkpoints.
            let checkpoint = session.checkpoint().unwrap();
            let mut restored = model.restore_session(&checkpoint).unwrap();
            let replay_prediction = restored.predict(&model).unwrap();
            if len == 5 {
                let mut bad: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
                bad["word_copy"]["span_words"] = 15.into();
                assert!(model
                    .restore_session(&serde_json::to_vec(&bad).unwrap())
                    .is_err());
                let commits = restored.work.word_copy.selector.commits;
                let p = restored.predict(&model).unwrap();
                restored
                    .observe(&model, if p.token == 2 { 3 } else { 2 })
                    .unwrap();
                assert_eq!(restored.work.word_copy.selector.commits, commits);
                assert!(restored.work.word_copy.selector.mismatches > 0);
            }
            MEASURING.with(|v| v.set(true));
            let start = std::time::Instant::now();
            let prediction = session.predict(&model).unwrap();
            let t = prediction.token;
            session.observe(&model, t).unwrap();
            let elapsed = start.elapsed().as_nanos();
            // The first step was predicted above to inspect its pending source.
            // Subsequent steps start without a pending prediction.
            if len != 0 {
                times[samples] = elapsed;
                samples += 1;
            }
            MEASURING.with(|v| v.set(false));
            assert_eq!(prediction, replay_prediction);
            out[len] = t;
            len += 1;
            if t == EOS || len == 32 {
                break;
            }
        }
        assert_eq!(out[len - 1], EOS);
        assert_eq!(model.decode(&out[..len]).unwrap(), expected.as_bytes());
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
        let checkpoint = session.checkpoint().unwrap();
        assert_eq!(
            model
                .restore_session(&checkpoint)
                .unwrap()
                .checkpoint()
                .unwrap(),
            checkpoint
        );
        if extra > 0 {
            assert!(
                model
                    .generate(prompt, 32, Control::SourceSpanDisabled)
                    .unwrap()
                    .text
                    .len()
                    < expected.len()
            );
        } else if contextual {
            assert_ne!(
                model
                    .generate(prompt, 32, Control::SourceSpanContextDisabled)
                    .unwrap()
                    .text,
                expected
            );
        }
    }
    times[..samples].sort_unstable();
    println!("span uncached warm predict+observe samples={samples} median_ns={} max_ns={} (loading, ingestion and checkpoints excluded)", times[samples/2], times[samples-1]);
    println!("parent restoration, malformed code rejection, source commitment, interrupted copy, every-byte checkpoint and zero allocations PASS");
}

#[test]
#[ignore = "requires the retained relation span artifact; charged model work"]
fn native_retained_relation_span_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::{Control, Model};
    let path = std::env::var("R4_RETAINED_SPAN_MODEL").unwrap();
    let model = Model::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let mut session = model.session(Control::Full).unwrap();
    session.observe(&model, 0).unwrap();
    for token in model.encode("ada in New ").unwrap() {
        session.observe(&model, token).unwrap();
    }
    let checkpoint = session.checkpoint().unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    assert!(wire["values"]["relations"]["pending"].is_object());
    assert_eq!(wire["values"]["relations"]["next_id"], 1);
    session = model.restore_session(&checkpoint).unwrap();
    let rest = format!("York. {}Where is ada? Answer:", "quiet sky. ".repeat(96));
    let tokens = model.encode(&rest).unwrap();
    ALLOCATIONS.with(|v| v.set(0));
    BYTES.with(|v| v.set(0));
    MEASURING.with(|v| v.set(true));
    for token in tokens {
        session.observe(&model, token).unwrap();
    }
    session.begin_response(&model).unwrap();
    MEASURING.with(|v| v.set(false));
    let saved = session.checkpoint().unwrap();
    let mut bad: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    bad["values"]["relations"]["records"][0]["span"]["len"] = 29.into();
    assert!(model
        .restore_session(&serde_json::to_vec(&bad).unwrap())
        .is_err());
    let mut out = [1; 32];
    let mut used = 0;
    let mut times = [0u128; 32];
    loop {
        let mut restored = model
            .restore_session(&session.checkpoint().unwrap())
            .unwrap();
        let predicted = restored.predict(&model).unwrap();
        MEASURING.with(|v| v.set(true));
        let start = std::time::Instant::now();
        let actual = session.predict(&model).unwrap();
        session.observe(&model, actual.token).unwrap();
        times[used] = start.elapsed().as_nanos();
        MEASURING.with(|v| v.set(false));
        assert_eq!(actual, predicted);
        out[used] = actual.token;
        used += 1;
        if actual.token == 1 || used == 32 {
            break;
        }
    }
    assert_eq!(out[used - 1], 1);
    assert_eq!(model.decode(&out[..used]).unwrap(), b" New York.\n");
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    times[..used].sort_unstable();
    println!("retained span pending/atomic write, malformed extent, eviction, each-step checkpoint and zero allocation PASS; uncached predict+observe samples={used} median_ns={} max_ns={} (includes initial selection; excludes load/encoding/ingestion/checkpoints)",times[used/2],times[used-1]);
}

#[test]
#[ignore = "requires the reverse relation span artifact; charged model work"]
fn native_reverse_relation_span_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::{Control, Model};
    let bytes = std::fs::read(std::env::var("R4_REVERSE_SPAN_MODEL").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let mut wrong: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    wrong["relation_reverse_spans"] = "wrong-parent".into();
    assert!(Model::from_bytes(&serde_json::to_vec(&wrong).unwrap()).is_err());
    let mut samples = Vec::new();
    for padding in [String::new(), "quiet sky. ".repeat(96)] {
        let mut s = model.session(Control::Full).unwrap();
        s.observe(&model, 0).unwrap();
        let prompt = format!("Orin Grove holds nelra. {padding}Where is nelra? Answer:");
        let tokens = model.encode(&prompt).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        for token in tokens {
            s.observe(&model, token).unwrap();
        }
        s.begin_response(&model).unwrap();
        MEASURING.with(|v| v.set(false));
        let checkpoint = s.checkpoint().unwrap();
        let wire: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
        let record = &wire["values"]["relations"]["records"][0];
        assert!(record["span"]["start"].is_object());
        assert_eq!(record["span"]["terminal"], record["value"]);
        let mut malformed = wire.clone();
        malformed["values"]["relations"]["records"][0]["span"]["start"]["byte_end"] = 999999.into();
        assert!(model
            .restore_session(&serde_json::to_vec(&malformed).unwrap())
            .is_err());
        let mut output = [1; 32];
        let mut used = 0;
        loop {
            let mut restored = model.restore_session(&s.checkpoint().unwrap()).unwrap();
            let expected = restored.predict(&model).unwrap();
            MEASURING.with(|v| v.set(true));
            let start = std::time::Instant::now();
            let actual = s.predict(&model).unwrap();
            s.observe(&model, actual.token).unwrap();
            let elapsed = start.elapsed().as_nanos();
            MEASURING.with(|v| v.set(false));
            samples.push(elapsed);
            assert_eq!(actual, expected);
            output[used] = actual.token;
            used += 1;
            if actual.token == 1 || used == 32 {
                break;
            }
        }
        assert_eq!(output[used - 1], 1);
        assert_eq!(model.decode(&output[..used]).unwrap(), b" Orin Grove.\n");
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    }
    samples.sort_unstable();
    println!("reverse span parent rejection, preserved terminal anchor, malformed start rejection, short/evicted reads, per-step checkpoints and zero allocations PASS; samples={} median_ns={} max_ns={} (uncached predict+observe includes initial selection; excludes load/encoding/ingestion/checkpoints)",samples.len(),samples[samples.len()/2],samples[samples.len()-1]);
}

#[test]
#[ignore = "requires learned relation-start artifact; charged model execution"]
fn native_relation_start_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::{Control, Model};
    let bytes = std::fs::read(std::env::var("R4_RELATION_START_MODEL").unwrap()).unwrap();
    let loading = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = loading.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut bad = wire.clone();
    bad["relation_start"]["codes"][0]["roots"][0] = 120.into();
    assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut bad = wire.clone();
    bad["relation_start"]["parent_artifact"] = "wrong-parent".into();
    assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let contextual = wire.get("relation_start_context").is_some();
    if contextual {
        let mut bad = wire.clone();
        bad["relation_start_context"][0]["prime"] = 0.into();
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        let control = model.without_relation_start_context().unwrap();
        let restored = Model::from_bytes(&control.to_bytes().unwrap()).unwrap();
        assert_eq!(control.artifact_cid(), restored.artifact_cid());
    }
    let mut times = Vec::new();
    let mut input_times = Vec::new();
    let mut cases = vec![
        (
            "Notes say Amber Meadow holds telra.",
            "telra",
            " Amber Meadow.\n",
            false,
        ),
        (
            "Notes say Amber Meadow holds telra.",
            "telra",
            " Amber Meadow.\n",
            true,
        ),
        ("Notes say Amber holds telra.", "telra", " Amber.\n", false),
    ];
    if contextual {
        cases.extend([
            (
                "notes say fine sand holds calvi.",
                "calvi",
                " fine sand.\n",
                false,
            ),
            (
                "notes say fine sand holds calvi.",
                "calvi",
                " fine sand.\n",
                true,
            ),
        ]);
    }
    for (fact, owner, expected, long) in cases {
        let mut s = model.session(Control::Full).unwrap();
        s.observe(&model, 0).unwrap();
        let padding = if long {
            "quiet sky. ".repeat(96)
        } else {
            String::new()
        };
        let encoding = std::time::Instant::now();
        let tokens = model
            .encode(&format!("{fact} {padding}Where is {owner}? Answer:"))
            .unwrap();
        let encode_ns = encoding.elapsed().as_nanos();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        let ingest = std::time::Instant::now();
        for token in tokens {
            s.observe(&model, token).unwrap();
        }
        s.begin_response(&model).unwrap();
        let ingest_ns = ingest.elapsed().as_nanos();
        MEASURING.with(|v| v.set(false));
        input_times.push((long, encode_ns, ingest_ns));
        let mut output = [1; 32];
        let mut used = 0;
        loop {
            let mut restored = model.restore_session(&s.checkpoint().unwrap()).unwrap();
            let predicted = restored.predict(&model).unwrap();
            MEASURING.with(|v| v.set(true));
            let start = std::time::Instant::now();
            let actual = s.predict(&model).unwrap();
            s.observe(&model, actual.token).unwrap();
            let elapsed = start.elapsed().as_nanos();
            MEASURING.with(|v| v.set(false));
            assert_eq!(actual, predicted);
            times.push(elapsed);
            output[used] = actual.token;
            used += 1;
            if actual.token == 1 || used == 32 {
                break;
            }
        }
        assert_eq!(output[used - 1], 1);
        assert_eq!(model.decode(&output[..used]).unwrap(), expected.as_bytes());
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    }
    times.sort_unstable();
    println!("relation-start load_ns={load_ns}; (evicted,encode_ns,ingest_begin_response_ns)={input_times:?}; ingestion includes start scoring, excludes emission/checkpoints; sampled wall times, no energy measurement");
    println!("relation-start model/root rejection, short/evicted/singleton outputs, per-step checkpoints and zero allocations PASS; samples={} median_ns={} max_ns={} (uncached predict+observe including initial selection; excludes loading/encoding/ingestion/checkpoints)",times.len(),times[times.len()/2],times[times.len()-1]);
}

#[test]
#[ignore = "requires refined writer artifact; charged model execution"]
fn native_writer_refinement_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_WRITER_REFINEMENT_MODEL").unwrap()).unwrap();
    let load = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = load.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_ne!(
        wire["relation_writer"]["rows"],
        wire["relation_writer_refinement"]["previous"]["rows"]
    );
    let mut bad = wire.clone();
    bad["relation_writer_refinement"]["parent_artifact"] = "wrong-parent".into();
    assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut bad = wire.clone();
    bad["relation_writer"]["dictionary"][0]["prime"] = 0.into();
    assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut bad = wire.clone();
    bad["relation_writer"]["admission"] =
        wire["relation_writer_refinement"]["previous"]["admission"].clone();
    assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut emission_ns = Vec::new();
    let mut inputs = Vec::new();
    for (fact, owner, expected, writes, evicted) in [
        (
            "notes say quiet harbor holds ranvi.",
            "ranvi",
            " quiet harbor.\n",
            1,
            false,
        ),
        (
            "notes say quiet harbor holds ranvi.",
            "ranvi",
            " quiet harbor.\n",
            1,
            true,
        ),
        ("notes say quiet harbor.", "ranvi", " Unknown.\n", 0, false),
        (
            "Report notes Cobalt Field holds vorin.",
            "vorin",
            " Cobalt Field.\n",
            1,
            false,
        ),
    ] {
        let padding = if evicted {
            "quiet sky. ".repeat(96)
        } else {
            String::new()
        };
        let prompt = format!("{fact} {padding}Where is {owner}? Answer:");
        let encoding = std::time::Instant::now();
        let tokens = model.encode(&prompt).unwrap();
        let encode_ns = encoding.elapsed().as_nanos();
        let mut session = model.session(Control::Full).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        let ingest = std::time::Instant::now();
        session.observe(&model, BOS).unwrap();
        for token in tokens {
            session.observe(&model, token).unwrap();
        }
        session.begin_response(&model).unwrap();
        let ingest_ns = ingest.elapsed().as_nanos();
        MEASURING.with(|v| v.set(false));
        assert_eq!(session.work.values.relations.record_writes, writes);
        inputs.push((evicted, encode_ns, ingest_ns));
        let mut output = [EOS; 32];
        let mut used = 0;
        loop {
            let mut restored = model
                .restore_session(&session.checkpoint().unwrap())
                .unwrap();
            let expected_prediction = restored.predict(&model).unwrap();
            assert_eq!(expected_prediction, restored.predict(&model).unwrap());
            MEASURING.with(|v| v.set(true));
            let start = std::time::Instant::now();
            let actual = session.predict(&model).unwrap();
            session.observe(&model, actual.token).unwrap();
            let elapsed = start.elapsed().as_nanos();
            MEASURING.with(|v| v.set(false));
            assert_eq!(actual, expected_prediction);
            emission_ns.push(elapsed);
            output[used] = actual.token;
            used += 1;
            if actual.token == EOS || used == 32 {
                break;
            }
        }
        assert_eq!(output[used - 1], EOS);
        assert_eq!(model.decode(&output[..used]).unwrap(), expected.as_bytes());
        assert_eq!(session.work.values.relations.record_writes, writes);
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    }
    emission_ns.sort_unstable();
    println!("writer-refinement load_ns={load_ns}; (evicted,encode_ns,ingest_begin_response_ns)={inputs:?}; ingestion includes changed writer/cache and phrase selection; measured wall samples, no energy result");
    println!("writer-refinement parent/dictionary/stale-cache rejection, exact assertion/nonassertion writes and output, evicted reads, every-step checkpoints and zero allocations PASS; predict_observe samples={} median_ns={} max_ns={} (loading, encoding, ingestion and checkpoints excluded)",emission_ns.len(),emission_ns[emission_ns.len()/2],emission_ns.last().unwrap());
}

#[test]
#[ignore = "requires learned operation artifact; charged actual-model execution"]
fn native_operation_transition_actual_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_OPERATION_TRANSITION_MODEL").unwrap()).unwrap();
    let load = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = load.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["operation_transition"].is_object());
    let mut times = Vec::new();
    let mut positions = 0;
    for (a, b, extra) in [(13, 4, 3), (-19, 4, 7)] {
        let mut session = model.session(Control::Full).unwrap();
        session.observe(&model, BOS).unwrap();
        for (prompt,expected) in [
            (format!("User: suri has {a} coins. orin has {b} coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:"),format!("{}.\n",a+b)),
            (format!("User: There are {extra} extra coins. Add the extra coins to the original total. Again.\nAssistant:"),format!("{}.\n{}.\n",a+b+extra,a+b+extra+extra)),
        ] {
            let tokens=model.encode(&prompt).unwrap();
            ALLOCATIONS.with(|v|v.set(0));BYTES.with(|v|v.set(0));
            MEASURING.with(|v|v.set(true));
            for token in tokens {session.observe(&model,token).unwrap();}
            session.begin_response(&model).unwrap();
            MEASURING.with(|v|v.set(false));
            let mut output=[EOS;64];let mut used=0;
            loop {
                let mut restored=model.restore_session(&session.checkpoint().unwrap()).unwrap();
                let prediction=restored.predict(&model).unwrap();
                assert_eq!(restored.predict(&model).unwrap(),prediction);
                MEASURING.with(|v|v.set(true));
                let start=std::time::Instant::now();
                let actual=session.predict(&model).unwrap();
                session.observe(&model,actual.token).unwrap();
                let elapsed=start.elapsed().as_nanos();
                MEASURING.with(|v|v.set(false));
                assert_eq!(actual,prediction);times.push(elapsed);positions+=1;
                output[used]=actual.token;used+=1;
                if actual.token==EOS||used==64{break;}
            }
            assert_eq!(output[used-1],EOS);
            assert_eq!(model.decode(&output[..used]).unwrap(),expected.as_bytes());
            assert_eq!((ALLOCATIONS.with(Cell::get),BYTES.with(Cell::get)),(0,0));
            session.end_response(&model).unwrap();
        }
    }
    times.sort_unstable();
    println!("actual learned operation artifact={}; load_ns={load_ns}; checkpoint_positions={positions}; allocations=0 bytes=0 for ingestion/begin/predict/observe; predict_observe median_ns={} max_ns={} (load/encode/session/checkpoint/report excluded; no energy claim)",model.artifact_cid(),times[times.len()/2],times[times.len()-1]);
}
