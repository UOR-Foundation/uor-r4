mod common;

use serde_json::{json, Value};
use uor_r4_studio_agent::{
    canonical_json, digest_event, parse_canonical_json, schema_document, AGENT_ERROR_SCHEMA,
    AGENT_EVENT_SCHEMA, CANONICAL_JSON_SCHEMA,
};

#[test]
fn keys_are_sorted_ascii() {
    let value = json!({"b": 2, "a": 1, "a_b": 5, "a.b": 3, "a-b": 4});
    assert_eq!(
        canonical_json(&value).unwrap(),
        r#"{"a":1,"a-b":4,"a.b":3,"a_b":5,"b":2}"#
    );

    let nested = json!({"z": {"b": true, "a": null}, "y": [3, 2, 1]});
    assert_eq!(
        canonical_json(&nested).unwrap(),
        r#"{"y":[3,2,1],"z":{"a":null,"b":true}}"#
    );
}

#[test]
fn insertion_order_does_not_change_digest() {
    let first = digest_event(&common::event_with_order(true)).unwrap();
    let second = digest_event(&common::event_with_order(false)).unwrap();
    assert_eq!(first, second);

    let hex = first.strip_prefix("sha256:").unwrap();
    assert_eq!(hex.len(), 64);
    assert!(hex
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));

    let ordered_first = parse_canonical_json(r#"{"b":2,"a":{"d":4,"c":3}}"#).unwrap();
    let ordered_second = parse_canonical_json(r#"{"a":{"c":3,"d":4},"b":2}"#).unwrap();
    assert_eq!(ordered_first, ordered_second);
}

#[test]
fn floats_are_rejected() {
    assert_eq!(
        canonical_json(&json!({"x": 1.5})).unwrap_err().code(),
        "invalid_event"
    );
    assert_eq!(
        canonical_json(&json!({"x": 1.0})).unwrap_err().code(),
        "invalid_event"
    );
    assert_eq!(
        parse_canonical_json(r#"{"x":1.5}"#).unwrap_err().code(),
        "invalid_event"
    );
    assert_eq!(
        parse_canonical_json(r#"{"x":1e3}"#).unwrap_err().code(),
        "invalid_event"
    );
}

#[test]
fn non_ascii_keys_are_rejected() {
    assert_eq!(
        canonical_json(&json!({"é": 1})).unwrap_err().code(),
        "invalid_event"
    );
    assert_eq!(
        canonical_json(&json!({"A": 1})).unwrap_err().code(),
        "invalid_event"
    );
    assert_eq!(
        canonical_json(&json!({"$schema": 1})).unwrap_err().code(),
        "invalid_event"
    );
    assert_eq!(
        parse_canonical_json(r#"{"é":1}"#).unwrap_err().code(),
        "invalid_event"
    );
    assert!(canonical_json(&json!({"ok": "é"})).is_ok());
}

#[test]
fn schema_document_is_stable_sorted() {
    let first = schema_document();
    let second = schema_document();
    assert_eq!(first, second);
    assert!(first.starts_with(r#"{"$id":"#));
    assert!(first.contains("\"run.queued\""));
    assert!(first.contains("\"run.started\""));
    assert!(first.contains("\"run.resumed\""));
    assert!(first.contains("\"retry_of_run_id\""));
    assert!(first.contains("\"runtime_unavailable\""));
    assert!(first.contains("\"host_unavailable\""));
    assert!(first.contains(AGENT_EVENT_SCHEMA));
    assert!(first.contains(AGENT_ERROR_SCHEMA));
    assert!(first.contains(CANONICAL_JSON_SCHEMA));
    assert!(first.contains("\"uorse1\""));
    assert!(!first.contains(": "));

    let parsed: Value = serde_json::from_str(&first).unwrap();
    assert!(parsed.get("event_types").is_some());
    assert!(parsed.get("error_codes").is_some());
}

#[test]
fn raw_parser_rejects_duplicate_keys_at_every_depth() {
    for text in [
        r#"{"a":1,"a":2}"#,
        r#"{"outer":{"a":1,"a":2}}"#,
        r#"[{"x":1,"x":1}]"#,
        r#"{"a":{"b":{"c":1,"c":1}}}"#,
    ] {
        assert_eq!(
            parse_canonical_json(text).unwrap_err().code(),
            "invalid_event",
            "{text}"
        );
    }
    assert_eq!(
        parse_canonical_json(r#"{"a":1,"b":2}"#).unwrap(),
        r#"{"a":1,"b":2}"#
    );
}

#[test]
fn raw_parser_rejects_floats_negative_and_unsafe_integers() {
    for text in [
        r#"{"x":1.5}"#,
        r#"{"x":-1}"#,
        r#"{"x":-0}"#,
        r#"{"x":9007199254740992}"#,
        r#"{"x":1e3}"#,
        r#"{"x":-9007199254740991}"#,
    ] {
        assert_eq!(
            parse_canonical_json(text).unwrap_err().code(),
            "invalid_event",
            "{text}"
        );
    }
    assert_eq!(
        parse_canonical_json(r#"{"x":0,"y":9007199254740991}"#).unwrap(),
        r#"{"x":0,"y":9007199254740991}"#
    );
}

#[test]
fn raw_parser_rejects_malformed_text_and_unpaired_surrogates() {
    for text in [
        "{\"x\":1",
        "\u{feff}{\"x\":1}",
        "{\"x\":1} trailing",
        "{\"x\":\"\\ud800\"}",
        "",
        "[] []",
    ] {
        assert_eq!(
            parse_canonical_json(text).unwrap_err().code(),
            "malformed_event",
            "{text:?}"
        );
    }
}

#[test]
fn raw_parser_escapes_minimally_and_preserves_utf8() {
    let text = r#"{"a":"caf\u00e9 \"q\" \\ line\nend","z":"\u0000\u001f"}"#;
    assert_eq!(
        parse_canonical_json(text).unwrap(),
        "{\"a\":\"café \\\"q\\\" \\\\ line\\u000aend\",\"z\":\"\\u0000\\u001f\"}"
    );

    assert_eq!(
        parse_canonical_json(r#"{"a":"é ☃"}"#).unwrap(),
        "{\"a\":\"é ☃\"}"
    );
    assert_eq!(
        parse_canonical_json(r#"{"é":1}"#).unwrap_err().code(),
        "invalid_event"
    );
}
