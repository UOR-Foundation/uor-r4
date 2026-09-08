//! Direct tests for conversation and identity-scoped durable memory (#962).

use super::durable_memory::*;
use super::*;

fn fixture_model() -> Model {
    let docs = vec![
        Document {
            id: "durable-memory-fixture".into(),
            text: "ada Rome cyra Perth liam cabinet leon basket".into(),
        },
        Document {
            id: "durable-memory-filler".into(),
            text: "the quick brown fox jumps over the lazy dog and runs through the forest".into(),
        },
    ];
    let mut trainer = Trainer::new(Config::default(), &docs).expect("trainer construction");
    trainer.train_documents(&docs).expect("train documents");
    trainer.compile().expect("model compilation")
}

#[test]
fn native_durable_memory_scope_validation_and_isolation() {
    // Validation
    assert!(IdentityScope::new("", "proj", "sess").is_err());
    assert!(IdentityScope::new("user", "  ", "sess").is_err());
    assert!(IdentityScope::new("user", "proj", "").is_err());

    let scope_a = IdentityScope::new("alice", "proj_x", "session_1").unwrap();
    let scope_b = IdentityScope::new("alice", "proj_x", "session_2").unwrap();
    let scope_c = IdentityScope::new("bob", "proj_y", "session_1").unwrap();

    assert_eq!(scope_a.key(), "alice/proj_x/session_1");
    assert!(scope_a.is_same_user_project(&scope_b));
    assert!(!scope_a.is_same_user_project(&scope_c));

    let store = DurableMemoryStore::new();
    assert!(!store.is_isolated(&scope_a, &scope_a));
    assert!(store.is_isolated(&scope_a, &scope_b));
    assert!(store.is_isolated(&scope_a, &scope_c));
}

#[test]
fn native_durable_memory_multi_turn_fact_retention_and_query() {
    let model = fixture_model();
    let scope = IdentityScope::new("user1", "project1", "session1").unwrap();
    let mut session = DurableSession::new(&model, scope.clone(), Control::Full).unwrap();

    assert_eq!(session.fact_count(), 0);
    assert_eq!(session.get_fact("ada"), None);

    // Turn 1: Assert fact Ada -> Rome
    let id1 = session.assert_fact("ada", "Rome").expect("assert fact");
    assert_eq!(id1, 1);
    assert_eq!(session.fact_count(), 1);

    // Turn 2: Query fact
    assert_eq!(session.get_fact("ada"), Some("Rome".into()));
    assert_eq!(session.last_entity.as_deref(), Some("ada"));

    let facts = session.active_facts();
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0], ("ada".into(), "Rome".into()));

    let record = session.get_fact_record("ada").expect("record exists");
    assert_eq!(record.id, 1);
    assert_eq!(record.action, 1);
    assert_eq!(record.previous, 0);
    assert!(!record.conflict);
}

#[test]
fn native_durable_memory_versioned_corrections_and_conflict_handling() {
    let model = fixture_model();
    let scope = IdentityScope::new("user1", "project1", "session1").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    // Turn 1: Initial assertion Ada -> Rome
    session.assert_fact("ada", "Rome").unwrap();
    assert_eq!(session.get_fact("ada"), Some("Rome".into()));
    assert!(!session.is_fact_in_conflict("ada"));

    // Turn 2: Explicit correction/revision (Action 2): Ada -> Dover
    let id2 = session.revise_fact("ada", "Dover").unwrap();
    assert_eq!(id2, 2);
    assert_eq!(session.get_fact("ada"), Some("Dover".into()));
    assert!(!session.is_fact_in_conflict("ada"));

    // Verify version history: Dover links back to Rome
    let history = session.get_fact_history("ada");
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].id, 2);
    assert_eq!(history[0].action, 2);
    assert_eq!(history[0].previous, 1);
    assert!(!history[0].conflict);
    assert_eq!(history[1].id, 1);
    assert_eq!(history[1].action, 1);
    assert_eq!(history[1].previous, 0);

    // Turn 3: Unannounced conflicting assertion (Action 1): Ada -> Cairo
    let id3 = session.assert_fact("ada", "Cairo").unwrap();
    assert_eq!(id3, 3);
    assert!(session.is_fact_in_conflict("ada"));

    // Turn 4: Resolution via explicit revision (Action 2): Ada -> Dover
    let id4 = session.revise_fact("ada", "Dover").unwrap();
    assert_eq!(id4, 4);
    assert!(!session.is_fact_in_conflict("ada"));
    assert_eq!(session.get_fact("ada"), Some("Dover".into()));
}

#[test]
fn native_durable_memory_pronoun_antecedent_resolution() {
    let model = fixture_model();
    let scope = IdentityScope::new("user1", "project1", "session1").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    session.assert_fact("ada", "Rome").unwrap();
    assert_eq!(session.last_entity.as_deref(), Some("ada"));

    // Pronoun queries resolve to Ada
    assert_eq!(session.resolve_pronoun("she"), "ada");
    assert_eq!(session.resolve_pronoun("She"), "ada");
    assert_eq!(session.resolve_pronoun("they"), "ada");
    assert_eq!(session.resolve_pronoun("it"), "ada");
    assert_eq!(session.resolve_pronoun("other"), "other");

    // get_fact resolves pronoun automatically
    assert_eq!(session.get_fact("she"), Some("Rome".into()));
    assert_eq!(session.get_fact("She"), Some("Rome".into()));

    // When a new entity is asserted, pronoun tracks the new antecedent
    session.assert_fact("liam", "cabinet").unwrap();
    assert_eq!(session.last_entity.as_deref(), Some("liam"));
    assert_eq!(session.resolve_pronoun("he"), "liam");
    assert_eq!(session.get_fact("he"), Some("cabinet".into()));
}

#[test]
fn native_durable_memory_eviction_survivability() {
    let docs = vec![Document {
        id: "eviction-test-doc".into(),
        text: "cyra Perth ada Rome the quick brown fox jumps over the lazy dog repeatedly".into(),
    }];
    // Small context window of 32 tokens to easily induce circular buffer eviction
    let config = Config {
        context_tokens: 32,
        candidate_limit: 16,
        max_lexical_pieces: 1024,
        max_rows: 4096,
        max_associations: 50_000,
        postings_per_row: 8,
    };
    let mut trainer = Trainer::new(config, &docs).unwrap();
    trainer.train_documents(&docs).unwrap();
    let model = trainer.compile().unwrap();

    let scope = IdentityScope::new("eviction_user", "proj", "s1").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    // Assert fact into durable relation memory
    session.assert_fact("cyra", "Perth").unwrap();
    assert_eq!(session.get_fact("cyra"), Some("Perth".into()));

    // Feed filler text repeated 10 times to far exceed the 32-token window
    let filler =
        "the quick brown fox jumps over the lazy dog and runs through the forest repeatedly ";
    let repeated = filler.repeat(10);
    session.session.observe(&model, BOS).unwrap();
    for token in model.encode(&repeated).unwrap() {
        session.session.observe(&model, token).unwrap();
    }

    // Verify that evictions definitely occurred in the circular token ring
    assert!(
        session.session.work.evictions > 50,
        "evictions={}",
        session.session.work.evictions
    );

    // Verify durable memory survived eviction intact!
    assert_eq!(
        session.get_fact("cyra"),
        Some("Perth".into()),
        "durable memory must survive circular ring buffer token eviction"
    );
}

#[test]
fn native_durable_memory_restart_and_export_import_roundtrip() {
    let model = fixture_model();
    let scope = IdentityScope::new("user_restart", "project_persist", "sess_42").unwrap();
    let mut session = DurableSession::new(&model, scope.clone(), Control::Full).unwrap();

    session.assert_fact("ada", "Rome").unwrap();
    session.assert_fact("liam", "cabinet").unwrap();
    assert_eq!(session.fact_count(), 2);

    // Export to envelope bytes
    let bytes = session.checkpoint().expect("export checkpoint");
    assert!(!bytes.is_empty());

    // Restore into a fresh session from checkpoint
    let restored = DurableSession::from_checkpoint(&model, &bytes).expect("restore checkpoint");
    assert_eq!(restored.scope, scope);
    assert_eq!(restored.get_fact("ada"), Some("Rome".into()));
    assert_eq!(restored.get_fact("liam"), Some("cabinet".into()));
    assert_eq!(restored.fact_count(), 2);

    // Restart: clears transient ring buffers but preserves durable relations
    session.restart(&model).expect("restart session");
    assert_eq!(
        session.session.work.observed_tokens, 0,
        "token stream reset"
    );
    assert_eq!(
        session.get_fact("ada"),
        Some("Rome".into()),
        "durable fact survives session restart"
    );
    assert_eq!(
        session.get_fact("liam"),
        Some("cabinet".into()),
        "durable fact survives session restart"
    );
    assert_eq!(session.fact_count(), 2);
}

#[test]
fn native_durable_memory_cross_identity_isolation_in_store() {
    let model = fixture_model();
    let mut store = DurableMemoryStore::new();

    let scope_alice = IdentityScope::new("alice", "confidential_proj", "s1").unwrap();
    let scope_bob = IdentityScope::new("bob", "public_proj", "s1").unwrap();

    // Alice stores a private confidential fact
    {
        let alice_session = store
            .get_or_create(&model, scope_alice.clone(), Control::Full)
            .unwrap();
        alice_session.assert_fact("ada", "Rome").unwrap();
        assert_eq!(alice_session.get_fact("ada"), Some("Rome".into()));
    }

    // Bob enters store under his own scope
    {
        let bob_session = store
            .get_or_create(&model, scope_bob.clone(), Control::Full)
            .unwrap();
        // Bob has ZERO access to Alice's facts
        assert_eq!(
            bob_session.get_fact("ada"),
            None,
            "Bob must not have access to Alice's memory"
        );
        assert_eq!(bob_session.fact_count(), 0);

        // Bob asserts his own distinct fact for the same entity
        bob_session.assert_fact("ada", "Dover").unwrap();
        assert_eq!(bob_session.get_fact("ada"), Some("Dover".into()));
    }

    // Verify Alice's memory is unchanged and completely uninfluenced by Bob
    {
        let alice_session = store.get(&scope_alice).unwrap();
        assert_eq!(
            alice_session.get_fact("ada"),
            Some("Rome".into()),
            "Alice's memory must remain completely isolated from Bob"
        );
    }
}

#[test]
fn native_durable_memory_selective_forget_and_full_reset() {
    let model = fixture_model();
    let scope = IdentityScope::new("forget_user", "proj", "s1").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    session.assert_fact("ada", "Rome").unwrap();
    session.assert_fact("cyra", "Perth").unwrap();
    session.assert_fact("liam", "cabinet").unwrap();
    assert_eq!(session.fact_count(), 3);

    // Selective forget: forget Ada
    let forgotten = session.forget(Some("ada")).expect("forget entity");
    assert_eq!(forgotten, 1);
    assert_eq!(session.get_fact("ada"), None);
    assert_eq!(session.get_fact("cyra"), Some("Perth".into()));
    assert_eq!(session.get_fact("liam"), Some("cabinet".into()));
    assert_eq!(session.fact_count(), 2);

    // Full reset
    session.reset(&model).expect("reset session");
    assert_eq!(session.fact_count(), 0);
    assert_eq!(session.get_fact("cyra"), None);
    assert_eq!(session.get_fact("liam"), None);
}

#[test]
fn native_durable_memory_consolidation() {
    let model = fixture_model();
    let scope = IdentityScope::new("consol_user", "proj", "s1").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    session.assert_fact("ada", "Rome").unwrap();
    session.revise_fact("ada", "Dover").unwrap();
    session.assert_fact("cyra", "Perth").unwrap();

    let report = session.consolidate().expect("consolidate");
    assert_eq!(report.active_records, 2);
    assert_eq!(report.capacity, 16);
    assert_eq!(session.get_fact("ada"), Some("Dover".into()));
    assert_eq!(session.get_fact("cyra"), Some("Perth".into()));
}

#[test]
fn native_durable_memory_turn_execution_flow() {
    let model = fixture_model();
    let scope = IdentityScope::new("dialogue_user", "proj", "turn_sess").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    // Turn 1: user input
    let gen1 = session.execute_turn(&model, "ada Rome", 16).unwrap();
    assert!(!gen1.token_ids.is_empty() || gen1.stop == "end_of_document");

    // Turn 2: continuous follow-up turn in same session
    let gen2 = session.execute_turn(&model, "cyra Perth", 16).unwrap();
    assert!(!gen2.token_ids.is_empty() || gen2.stop == "end_of_document");
    assert!(session.session.work.observed_tokens > gen1.token_ids.len() as u64);
}

#[test]
fn native_durable_memory_rejects_truncating_fact_identity() {
    let model = fixture_model();
    let mut session = DurableSession::new(
        &model,
        IdentityScope::new("u", "p", "s").unwrap(),
        Control::Full,
    )
    .unwrap();
    let prefix = "a".repeat(super::value_lexemes::WORD_BYTES);
    session.assert_fact(&prefix, "value").unwrap();
    let overlong = format!("{prefix}x");
    assert!(session.assert_fact(&overlong, "other").is_err());
    assert_eq!(session.get_fact(&overlong), None);
    assert!(session.revise_fact(&prefix, &overlong).is_err());
    assert_eq!(session.get_fact(&prefix).as_deref(), Some("value"));
    assert!(session.forget(Some(&overlong)).is_err());
}

#[test]
#[ignore = "requires explicit retained artifact to verify learned relation persistence"]
fn native_durable_memory_learned_relations_are_authoritative() {
    let path = std::env::var("UOR_R4_MODEL").expect("UOR_R4_MODEL required");
    let model = Model::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let scope = IdentityScope::new("user", "project", "session").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();
    session
        .execute_turn(
            &model,
            "the report says quiet river holds selra. Where is selra? Answer:",
            24,
        )
        .unwrap();
    let learned = session
        .session
        .values
        .as_ref()
        .and_then(|v| v.relations.as_ref())
        .expect("artifact has learned relations")
        .clone();
    assert!(
        learned.directory.iter().any(|&id| id != 0),
        "prompt must produce actual learned writes"
    );
    let count = learned.directory.iter().filter(|&&id| id != 0).count();
    assert_eq!(session.fact_count(), count);
    // Checkpoint must include the current core state, not the stale initial copy.
    let bytes = session.checkpoint().unwrap();
    let restored = DurableSession::from_checkpoint(&model, &bytes).unwrap();
    assert_eq!(restored.fact_count(), count);
    assert_eq!(
        restored
            .session
            .values
            .as_ref()
            .unwrap()
            .relations
            .as_ref()
            .unwrap(),
        &learned
    );
    session.restart(&model).unwrap();
    assert_eq!(
        session
            .session
            .values
            .as_ref()
            .unwrap()
            .relations
            .as_ref()
            .unwrap(),
        &learned
    );
    // An envelope cannot replace the learned core state with another store.
    let mut corrupted: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    corrupted["relations"]["next_id"] = serde_json::json!(999999);
    assert!(
        DurableSession::from_checkpoint(&model, &serde_json::to_vec(&corrupted).unwrap()).is_err()
    );
}
