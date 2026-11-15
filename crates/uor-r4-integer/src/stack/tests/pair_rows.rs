//! claude-384: the pair rows in a served stack's artifact.

use serde_json::json;

use super::super as stack;
use super::{data_bytes, drawn_address, served_read_trunk, with_header, IntegerStackModel};

/// Pair rows of three pairs over the stack's vocabulary at `width`, the rows drawn from a small
/// integer walk.
fn drawn_pair_rows(vocab: usize, width: usize) -> stack::PairRowsImage {
    let primes = crate::fbs_pair_rows::token_primes(vocab).expect("the stack's vocabulary");
    let mut addresses = vec![
        primes[1] * primes[2],
        primes[2] * primes[5],
        primes[3] * primes[7],
    ];
    addresses.sort_unstable();
    let rows = (0..addresses.len() * width)
        .map(|i| ((i * 37) % 401) as i16 - 200)
        .collect();
    let marks = stack::PairMarks {
        user: vec![1, 2],
        assistant: vec![3, 4],
    };
    stack::PairRowsImage::checked(vocab, width, marks, addresses, rows)
        .expect("pair rows of the stack's size")
}

/// Pair rows round-trip. A stack without them inspects with no `pair_rows` key and reads no
/// image. Added, the image read back equals the one written bit for bit, under `lut-stack/12`,
/// every earlier section keeps its bytes, and inspect shows the record.
#[test]
fn pair_rows_round_trip_and_inspect() {
    let (_, plain, with_read) = served_read_trunk(&[1, 3], 1.0, 1.0);
    for (name, bytes) in [("no reads", &plain), ("reads", &with_read)] {
        assert_eq!(
            stack::read_pair_rows_uorlut01(bytes).expect("parse"),
            None,
            "{name}"
        );
        let before = stack::inspect_uorlut01(bytes).expect("inspect");
        let written = serde_json::to_value(&before).expect("the metadata is written");
        assert!(
            written.get("pair_rows").is_none(),
            "{name}: a stack without pair rows inspects as before"
        );
        let image = drawn_pair_rows(before.shape.vocab, before.shape.width);
        let out = stack::add_pair_rows_uorlut01(bytes, &image).expect("the rows are added");
        let back = stack::read_pair_rows_uorlut01(&out)
            .expect("parse")
            .expect("the rows are read");
        assert_eq!(back, image, "{name}");
        assert_eq!(back.to_bytes(), image.to_bytes(), "{name}: bit for bit");
        assert!(IntegerStackModel::parse(&out).is_ok(), "{name}");
        let after = stack::inspect_uorlut01(&out).expect("inspect");
        assert_eq!(after.schema, stack::STACK_FBS_PAIR_ROWS_SCHEMA, "{name}");
        assert_eq!(
            after.pair_rows,
            Some(stack::StackPairRows::of(&image)),
            "{name}"
        );
        let shown = serde_json::to_value(&after).expect("the metadata is written");
        assert_eq!(
            shown["pair_rows"]["sha256"],
            json!(image.sha256()),
            "{name}"
        );
        assert_eq!(
            shown["pair_rows"]["row_exp"],
            json!(stack::PAIR_ROW_EXP),
            "{name}"
        );
        let (old, new) = (data_bytes(bytes), data_bytes(&out));
        assert_eq!(
            new[..old.len()],
            old[..],
            "{name}: every earlier section keeps its bytes"
        );
    }
}

/// The values table of the wrong length is refused.
#[test]
fn pair_rows_refuse_a_short_values_table() {
    let (_, plain, _) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let shape = stack::inspect_uorlut01(&plain).expect("inspect").shape;
    let image = drawn_pair_rows(shape.vocab, shape.width);
    let out = stack::add_pair_rows_uorlut01(&plain, &image).expect("the rows are added");
    // A header rewritten with no edit is still read: a refusal below is the edit's own.
    assert!(stack::read_pair_rows_uorlut01(&with_header(&out, |_| {}))
        .expect("read")
        .is_some());
    let short = with_header(&out, |h| {
        for t in h["tables"].as_array_mut().unwrap() {
            if t["name"] == json!("pair_rows.values") {
                let len = t["len"].as_u64().unwrap() - shape.width as u64;
                t["len"] = json!(len);
                t["span"]["bytes"] = json!(len * 2);
            }
        }
    });
    assert!(stack::read_pair_rows_uorlut01(&short).is_err());
}

/// A record whose hash is not the rows file's is refused.
#[test]
fn pair_rows_refuse_a_changed_sha256() {
    let (_, plain, _) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let shape = stack::inspect_uorlut01(&plain).expect("inspect").shape;
    let image = drawn_pair_rows(shape.vocab, shape.width);
    let out = stack::add_pair_rows_uorlut01(&plain, &image).expect("the rows are added");
    let hashed = with_header(&out, |h| h["pair_rows"]["sha256"] = json!("00".repeat(32)));
    assert!(
        stack::inspect_uorlut01(&hashed).is_ok(),
        "the header itself still parses"
    );
    assert!(stack::read_pair_rows_uorlut01(&hashed).is_err());
}

/// The record is read strictly and with its own schema: it is refused under `/11` and under the
/// `/6` and under the stack's own earlier schema, with an unknown key, or at another exponent.
#[test]
fn pair_rows_record_and_schema_come_together() {
    let (_, plain, _) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let before = stack::inspect_uorlut01(&plain).expect("inspect");
    let image = drawn_pair_rows(before.shape.vocab, before.shape.width);
    let out = stack::add_pair_rows_uorlut01(&plain, &image).expect("the rows are added");
    let earlier = before.schema.as_str();
    for schema in [
        stack::STACK_FBS_ADDRESS_SCHEMA,
        stack::STACK_FBS_HAMMING_SCHEMA,
        earlier,
    ] {
        let bytes = with_header(&out, |h| h["schema"] = json!(schema));
        assert!(
            stack::inspect_uorlut01(&bytes).is_err(),
            "the record under {schema}"
        );
        assert!(
            IntegerStackModel::parse(&bytes).is_err(),
            "the record under {schema}"
        );
    }
    let unknown = with_header(&out, |h| h["pair_rows"]["extra"] = json!(1));
    assert!(
        stack::inspect_uorlut01(&unknown).is_err(),
        "a key the reader does not know"
    );
    let exp = with_header(&out, |h| h["pair_rows"]["row_exp"] = json!(-16));
    assert!(stack::inspect_uorlut01(&exp).is_err(), "another exponent");
}

/// Pair rows are added once, never beside an address table, and only at the stack's own width
/// and vocabulary.
#[test]
fn pair_rows_are_added_once_and_alone() {
    let (_, plain, _) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let shape = stack::inspect_uorlut01(&plain).expect("inspect").shape;
    let image = drawn_pair_rows(shape.vocab, shape.width);
    let out = stack::add_pair_rows_uorlut01(&plain, &image).expect("the rows are added");
    assert!(
        stack::add_pair_rows_uorlut01(&out, &image).is_err(),
        "a second add"
    );
    let table = drawn_address(stack::AddressArm::E8, shape.width, 3, 9);
    let address = stack::add_address_uorlut01(&plain, &table).expect("the address table is added");
    assert!(
        stack::add_pair_rows_uorlut01(&address, &image).is_err(),
        "an address-table stack"
    );
    let record = serde_json::to_value(stack::StackPairRows::of(&image)).unwrap();
    let both = with_header(&address, |h| h["pair_rows"] = record);
    assert!(
        stack::inspect_uorlut01(&both).is_err(),
        "both records together"
    );
    let narrow = drawn_pair_rows(shape.vocab, shape.width - 8);
    assert!(
        stack::add_pair_rows_uorlut01(&plain, &narrow).is_err(),
        "another width"
    );
    let wider = drawn_pair_rows(shape.vocab + 1, shape.width);
    assert!(
        stack::add_pair_rows_uorlut01(&plain, &wider).is_err(),
        "another vocabulary"
    );
}
