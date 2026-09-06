use super::dependent_read::{follow, valid};
use super::relation::RelationState;
use super::value_lexemes::WordAtom;
use super::value_types::{ValueState, ValueWork};
use super::*;

fn atom(text: &str, end: u64) -> WordAtom {
    let mut out = WordAtom {
        len: text.len() as u8,
        end,
        byte_end: end,
        ..Default::default()
    };
    out.bytes[..text.len()].copy_from_slice(text.as_bytes());
    out
}

#[test]
fn dependent_read_requires_current_exact_nonconflicting_intermediate_and_final_versions() {
    let docs = [Document {
        id: "dependent-layout".into(),
        text: "crate ada Rome cyra Perth".into(),
    }];
    let mut trainer = Trainer::new(Config::default(), &docs).unwrap();
    trainer.train_documents(&docs).unwrap();
    let model = trainer.compile().unwrap();
    let mut values = ValueState::new(&model);
    let mut state = RelationState::default();
    let mut work = ValueWork::default();
    state.commit(atom("crate", 8), atom("ada", 16), 1, &mut work);
    assert!(follow(&state, state.record(1).unwrap(), &mut work).is_none());
    state.commit(atom("ada", 24), atom("Rome", 32), 1, &mut work);
    state.commit(atom("cyra", 40), atom("Perth", 48), 1, &mut work);
    assert_eq!(
        follow(&state, state.record(1).unwrap(), &mut work)
            .unwrap()
            .id,
        2
    );
    values.relations = Some(state.clone());
    assert!(valid(&values, [1, 2], &mut work));
    assert!(!valid(&values, [1, 3], &mut work));
    state.commit(atom("crate", 56), atom("cyra", 64), 2, &mut work);
    values.relations = Some(state.clone());
    assert!(!valid(&values, [1, 2], &mut work));
    assert!(valid(&values, [4, 3], &mut work));
    state.commit(atom("cyra", 72), atom("Cairo", 80), 1, &mut work);
    values.relations = Some(state.clone());
    assert!(follow(&state, state.record(4).unwrap(), &mut work).is_none());
    assert!(!valid(&values, [4, 3], &mut work));
    assert!(!valid(&values, [4, 5], &mut work));
    assert!(work.relations.directory_reads > 0);
    assert!(work.lexical_byte_comparisons > 0);
}
