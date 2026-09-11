//! Causal storage checks; learned complete-answer behavior is evaluated separately.
use super::relation::*;
use super::value_lexemes::{LexemeState, WordAtom};
use super::value_types::{ValueState, ValueWork};
use super::*;

fn atom(name: &str, ordinal: u64) -> WordAtom {
    let mut w = WordAtom {
        len: name.len() as u8,
        end: ordinal,
        byte_end: ordinal,
        ..WordAtom::default()
    };
    w.bytes[..name.len()].copy_from_slice(name.as_bytes());
    w
}
#[test]
fn native_relation_versions_preserve_unrelated_members_and_conflicts() {
    println!(
        "relation_layout record={} state={}",
        std::mem::size_of::<RelationRecord>(),
        std::mem::size_of::<RelationState>()
    );
    let mut state = RelationState::default();
    let mut work = ValueWork::default();
    state.commit(atom("ada", 8), atom("Rome", 16), 1, &mut work);
    state.commit(atom("cyra", 24), atom("Perth", 32), 1, &mut work);
    state.commit(atom("ada", 40), atom("Dover", 48), 2, &mut work);
    assert_eq!(state.record(1).unwrap().value, atom("Rome", 16));
    assert!(state.directory.contains(&2));
    assert!(state.directory.contains(&3));
    assert!(!state.directory.contains(&1));
    assert_eq!(state.record(3).unwrap().previous, 1);
    assert!(!state.record(3).unwrap().conflict);
    state.commit(atom("ada", 56), atom("Cairo", 64), 1, &mut work);
    assert!(state.record(4).unwrap().conflict);
    state.commit(atom("ada", 72), atom("Dover", 80), 2, &mut work);
    assert!(!state.record(5).unwrap().conflict);
    assert_eq!(state.record(2).unwrap().value, atom("Perth", 32));
    let mut other = state.clone();
    other.commit(atom("cyra", 88), atom("Cairo", 96), 2, &mut work);
    assert_eq!(state.record(2).unwrap().value, atom("Perth", 32));
    assert_eq!(state.next_id, 6);
    for i in 0..24 {
        other.commit(
            atom("extra", 104 + i * 16),
            atom("place", 112 + i * 16),
            2,
            &mut work,
        );
    }
    assert_eq!(other.records.iter().filter(|r| r.id != 0).count(), 16);
    assert!(other.record(1).is_none());
    assert!(work.relations.record_evictions > 0);
}

#[test]
fn native_relation_snapshot_rejects_dangling_or_rewritten_current_versions() {
    let docs = vec![Document {
        id: "relation-shape".into(),
        text: "ada Rome cyra Perth".into(),
    }];
    let mut trainer = Trainer::new(Config::default(), &docs).unwrap();
    trainer.train_documents(&docs).unwrap();
    let model = trainer.compile().unwrap();
    let mut values = ValueState::new(&model);
    values.seen = 1000;
    values.lexemes = Some(LexemeState {
        source_bytes_seen: 1000,
        ..LexemeState::default()
    });
    let mut state = RelationState::default();
    let mut work = ValueWork::default();
    state.commit(atom("ada", 8), atom("Rome", 16), 1, &mut work);
    state.commit(atom("ada", 24), atom("Perth", 32), 2, &mut work);
    assert!(state.validate(&values, &model).is_ok());
    let mut bad = state.clone();
    bad.directory[0] = 1;
    assert!(bad.validate(&values, &model).is_err());
    let mut bad = state.clone();
    bad.directory[0] = 999;
    assert!(bad.validate(&values, &model).is_err());
    let mut bad = state.clone();
    bad.records[1].previous = 2;
    assert!(bad.validate(&values, &model).is_err());
    let mut bad = state.clone();
    bad.records[1].conflict = true;
    assert!(bad.validate(&values, &model).is_err());
}

#[test]
fn native_relation_role_features_ignore_payload_renaming_and_global_pose() {
    let docs = vec![Document {
        id: "role-transport".into(),
        text: "in now not holds".into(),
    }];
    let mut trainer = Trainer::new(Config::default(), &docs).unwrap();
    trainer.train_documents(&docs).unwrap();
    let model = trainer.compile().unwrap();
    let mut context = vec![
        model.geometry.tokens[7].clone(),
        model.geometry.tokens[11].clone(),
    ];
    context[0].prime = 13;
    context[1].prime = 17;
    let words = [
        atom("oldvalue", 32),
        atom("in", 24),
        atom("now", 16),
        atom("owner", 8),
    ];
    let addr = [3, 13, 17, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    // Every offered pair is invariant to its own participant payloads, including
    // pairs where the neighbor feature addresses the other participant.
    for o in 0..4 {
        for v in 0..4 {
            if o == v || (o != 0 && v != 0) {
                continue;
            }
            let mut renamed = words;
            let mut changed_addr = addr;
            for (i, name) in [(o, "differentowner"), (v, "differentvalue")] {
                renamed[i] = atom(name, 80);
                renamed[i].pose = 37;
                renamed[i].phases = [62000; PHASE_CHANNELS];
                changed_addr[i] = 0;
            }
            let a = write_features_with_context(
                &model,
                &words,
                &addr,
                o,
                v,
                Some(&context),
                &mut ValueWork::default(),
            );
            let b = write_features_with_context(
                &model,
                &renamed,
                &changed_addr,
                o,
                v,
                Some(&context),
                &mut ValueWork::default(),
            );
            assert_eq!(a, b);
        }
    }
}

#[test]
fn native_relation_role_path_keeps_context_order_and_direction() {
    let docs = vec![Document {
        id: "role-path-order".into(),
        text: "context".into(),
    }];
    let mut trainer = Trainer::new(Config::default(), &docs).unwrap();
    trainer.train_documents(&docs).unwrap();
    let model = trainer.compile().unwrap();
    let g = &model.geometry;
    let (a, b) = (0..g.inverses.len())
        .flat_map(|a| (0..g.inverses.len()).map(move |b| (a, b)))
        .find(|&(a, b)| g.products[g.row_bases[a] + b] != g.products[g.row_bases[b] + a])
        .unwrap();
    let context = [
        TokenGeometry {
            prime: 13,
            leaf: a as u16,
            phases: [8192; PHASE_CHANNELS],
        },
        TokenGeometry {
            prime: 17,
            leaf: b as u16,
            phases: [4096; PHASE_CHANNELS],
        },
    ];
    let words = [
        atom("value", 32),
        atom("contexta", 24),
        atom("contextb", 16),
        atom("owner", 8),
    ];
    let mut addr = [0; 16];
    addr[1] = 13;
    addr[2] = 17;
    let (f, n) = write_features_with_context(
        &model,
        &words,
        &addr,
        3,
        0,
        Some(&context),
        &mut ValueWork::default(),
    );
    let mut reversed = addr;
    reversed.swap(1, 2);
    let (r, m) = write_features_with_context(
        &model,
        &words,
        &reversed,
        3,
        0,
        Some(&context),
        &mut ValueWork::default(),
    );
    let root = |features: &[super::value_types::ValueFeature]| {
        features.iter().find(|f| f.kind == 10).unwrap().a as usize
    };
    assert_ne!(root(&f[..n]), root(&r[..m]));
    let (back, k) = write_features_with_context(
        &model,
        &words,
        &addr,
        0,
        3,
        Some(&context),
        &mut ValueWork::default(),
    );
    assert_eq!(root(&back[..k]), usize::from(g.inverses[root(&f[..n])]));
}

#[test]
fn native_writer_cue_identity_is_exact_and_does_not_change_reader_addresses() {
    use super::relation_training::WriterRevision;
    use super::word_copy_types::WordCopyAddress;
    let docs = [Document {
        id: "writer-cue".into(),
        text: "Now Question now".into(),
    }];
    let mut trainer = Trainer::new(Config::default(), &docs).unwrap();
    trainer.train_documents(&docs).unwrap();
    let mut model = trainer.compile().unwrap();
    let word = |text: &str, prime| {
        let mut bytes = [0; 32];
        bytes[..text.len()].copy_from_slice(text.as_bytes());
        WordCopyAddress {
            bytes,
            len: text.len() as u8,
            prime,
        }
    };
    model.relation_writer = Some(WriterRevision {
        schema: "uor-r4.relation-writer/1".into(),
        parent: String::new(),
        dictionary: vec![word("Now", 2), word("Question", 3), word("now", 5)],
        role_context: vec![],
        rows: vec![],
        training: vec![],
        epochs: 1,
        reuse_admission: false,
        admission: None,
    });
    let words = [
        atom("Now", 0),
        atom("Question", 8),
        atom("newname", 16),
        atom("now", 24),
    ];
    let mut work = ValueWork::default();
    assert_eq!(
        &writer_addresses(&model, &words, &mut work)[..4],
        &[2, 3, 0, 5]
    );
    assert_eq!(&addresses(&model, &words, &mut work)[..4], &[0, 0, 0, 0]);
    assert!(work.relations.dictionary_byte_comparisons > 0);
    // Exact word payloads, including unknown names and case, remain untouched.
    assert_eq!(words[2], atom("newname", 16));
}

/// Feed a prompt through the word scanner into a query view; token ends equal byte
/// offsets in this fixture, so boundary tests read them from the words themselves.
fn scanned(text: &str) -> LexemeState {
    use super::value_types::ValueEntry;
    let mut words = LexemeState::default();
    let mut work = ValueWork::default();
    for (sequence, &byte) in text.as_bytes().iter().enumerate() {
        words.feed(
            byte,
            ValueEntry {
                sequence: sequence as u64,
                ..Default::default()
            },
            &mut work,
        );
    }
    words.finish(&mut work);
    words.begin();
    words
}

fn scan_counts(
    owner: &str,
    text: &str,
    window: usize,
    cutoff: Option<u64>,
    boundary: Option<u64>,
) -> (usize, Vec<(u64, u64)>) {
    let mut state = RelationState::default();
    let mut work = ValueWork::default();
    state.commit(atom(owner, 8), atom("Ridge", 16), 1, &mut work);
    let record = state.record(1).unwrap().clone();
    let words = scanned(text);
    let mut addr = [0u32; 16];
    for (q, slot) in addr.iter_mut().enumerate() {
        *slot = q as u32 + 1;
    }
    let (features, n) =
        read_features_scan(&record, &words, &addr, window, cutoff, boundary, &mut work);
    assert!(n <= RELATION_FEATURES);
    // (before, after) pairs of every owner match, in scan order
    let pairs = features[..n]
        .iter()
        .filter(|f| f.kind == 3)
        .map(|f| (f.a, f.b))
        .collect();
    (n, pairs)
}

#[test]
fn native_relation_owner_scan_capacity_holds_every_match_count_up_to_the_full_view() {
    let flood = |k: usize| format!("Record: selvi in Dusk Ridge. {}", "selvi ".repeat(k));
    assert_eq!(RELATION_FEATURES, 2 + 4 * super::value_lexemes::WORD_QUERY);
    for matches in [0usize, 1, 8, 15, 16] {
        let (n, pairs) = scan_counts("selvi", &flood(matches), 16, None, None);
        // The view holds sixteen words; the fact's own owner word is still inside it
        // for small floods, so count what the view actually exposes.
        let words = scanned(&flood(matches));
        let visible = words.queries[..words.query_len]
            .iter()
            .filter(|w| &w.bytes[..usize::from(w.len)] == b"selvi")
            .count();
        assert_eq!(n, 2 + 4 * visible, "matches={matches}");
        assert_eq!(pairs.len(), visible);
    }
    // The legacy eight-word base never exceeds its own bound.
    let (n, _) = scan_counts("selvi", &flood(16), 8, None, None);
    assert_eq!(n, 2 + 4 * 8);
    // A mixed flood counts only the requested owner's spelling: four in the flood plus
    // the fact's own owner word, which the thirteen-word view still holds.
    let (n, _) = scan_counts(
        "selvi",
        "Record: selvi in Dusk Ridge. selvi tilva selvi tilva selvi tilva selvi tilva ",
        16,
        None,
        None,
    );
    assert_eq!(n, 2 + 4 * 5);
    // No owner match at all leaves only the two base features.
    let (n, pairs) = scan_counts("other", &flood(16), 16, None, None);
    assert_eq!((n, pairs.len()), (2, 0));
}

#[test]
fn native_relation_owner_scan_stops_at_the_fact_cutoff_and_the_turn_boundary() {
    // A previous turn's question and answer, then a new turn asking about another owner.
    // Fifteen words, so the fact owner, the question owner and the answer owner all sit
    // inside the sixteen-word view.
    let text = "nemvi in quay. Where is nemvi? Answer: nemvi is in quay. Where is zalfe? Answer:";
    let words = scanned(text);
    let by_text = |needle: &[u8]| {
        words.queries[..words.query_len]
            .iter()
            .filter(|w| &w.bytes[..usize::from(w.len)] == needle)
            .map(|w| (w.end, w.byte_end))
            .collect::<Vec<_>>()
    };
    let nemvi = by_text(b"nemvi");
    assert_eq!(
        nemvi.len(),
        3,
        "fact owner, question owner and answer owner are all in the sixteen-word view"
    );
    // Legacy base: eight most recent words, no cutoff or boundary: the answer's owner
    // word is inside the eight and matches.
    let (legacy, _) = scan_counts("nemvi", text, 8, None, None);
    assert!(legacy > 2);
    // Request-window contract: the fact cutoff removes the fact's owner word only; the
    // previous question and generated answer still match (the measured cross-turn leak).
    let fact_cutoff = Some(nemvi[nemvi.len() - 1].1);
    let (post_fact, _) = scan_counts("nemvi", text, 16, fact_cutoff, None);
    assert_eq!(post_fact, 2 + 4 * 2);
    // Turn-window contract: the current turn begins at "Where is zalfe?": nothing older
    // may name the owner, so no match remains.
    let zalfe = by_text(b"zalfe");
    let where_end = words.queries[..words.query_len]
        .iter()
        .filter(|w| &w.bytes[..usize::from(w.len)] == b"Where")
        .map(|w| w.end)
        .max()
        .unwrap();
    let turn_start = Some(where_end.min(zalfe[0].0));
    let (turn, pairs) = scan_counts("nemvi", text, 16, fact_cutoff, turn_start);
    assert_eq!((turn, pairs.len()), (2, 0));
    // The requested owner inside the current turn is still found, and its older
    // neighbour outside the turn contributes no context (before address masked to 0).
    let (found, pairs) = scan_counts("zalfe", text, 16, fact_cutoff, turn_start);
    assert_eq!(found, 2 + 4);
    assert_eq!(
        pairs[0].0 != 0,
        true,
        "the previous word 'is' lies inside the turn"
    );
    let mid_turn = Some(zalfe[0].0);
    let (found, pairs) = scan_counts("zalfe", text, 16, fact_cutoff, mid_turn);
    assert_eq!(found, 2 + 4);
    assert_eq!(
        pairs[0].0, 0,
        "a neighbour older than the boundary is masked"
    );
}
