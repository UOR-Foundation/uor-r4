//! Activate a pinned passive S2 carrier without rewriting its retained table or trunk.
use serde::Deserialize;
use serde_json::value::RawValue;
use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
use uor_r4_integer::discrete_d11::{
    IntegerDiscreteModel, DISCRETE_S2_EXPERT_SCHEMA, DISCRETE_S2_SCHEMA,
};

#[derive(Deserialize)]
struct SchemaField<'a> {
    #[serde(borrow)]
    schema: &'a RawValue,
}

fn activate(bytes: &mut [u8], expected: &str) -> Result<(), Box<dyn Error>> {
    let model = IntegerDiscreteModel::from_bytes(bytes)?;
    if model.artifact_schema() != DISCRETE_S2_SCHEMA || model.artifact_sha256() != expected {
        return Err("input must be the pinned passive S2 carrier".into());
    }
    drop(model);
    let len = usize::try_from(u64::from_le_bytes(bytes[8..16].try_into()?))?;
    let header = &bytes[16..16 + len];
    let field: SchemaField = serde_json::from_slice(header)?;
    // RawValue borrows the schema string from this exact header. Both literal
    // '1' and its JSON Unicode escape end in the same ASCII byte.
    let raw = field.schema.get();
    let start = (raw.as_ptr() as usize)
        .checked_sub(header.as_ptr() as usize)
        .ok_or("schema is outside the header")?;
    let final_digit = start + raw.len() - 2;
    if header.get(final_digit..=final_digit + 1) != Some(b"1\"") {
        return Err("unexpected passive S2 schema spelling".into());
    }
    bytes[16 + final_digit] = b'2';
    let updated: SchemaField = serde_json::from_slice(&bytes[16..16 + len])?;
    if serde_json::from_str::<String>(updated.schema.get())? != DISCRETE_S2_EXPERT_SCHEMA {
        return Err("invalid active S2 schema".into());
    }
    Ok(())
}

fn publish(output: &Path, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let tmp = output.with_file_name(format!(".activate-s2-{}-{stamp}.tmp", std::process::id()));
    let mut f = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
    let written = (|| -> std::io::Result<()> {
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        // The destination becomes visible only as a complete file, and a
        // concurrent activation or an existing destination is never overwritten.
        fs::hard_link(&tmp, output)
    })();
    let cleanup = fs::remove_file(tmp);
    written?;
    cleanup?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [input, pin, output] = args.as_slice() else {
        return Err("usage: activate-s2-expert PASSIVE.d11 EXPECTED_SHA256 NEW_ACTIVE.d11".into());
    };
    let mut bytes = fs::read(input)?;
    activate(&mut bytes, pin.to_str().ok_or("invalid SHA-256 text")?)?;
    publish(Path::new(output), &bytes)
}
