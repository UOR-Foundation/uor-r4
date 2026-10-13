//! Finite, exact CUDA construction controls. No optimizer application or corpus run.
use crate::{
    device::{Device, Snapshot},
    Error, Identity, Parameter, Result, Spec,
};
use std::path::Path;
fn require(ok: bool, why: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error::Invalid(why.into()))
    }
}
fn identity(token: u32, occurrence: u64) -> Identity {
    Identity {
        token,
        occurrence,
        record: 17,
        version: 9,
    }
}
fn fixture(vocabulary: u32, context: u32, kept: u32) -> Result<Snapshot> {
    let device = Device::new(
        Spec {
            vocabulary,
            blocks: 2,
            context,
            kept_cells: kept,
        },
        7,
        0,
    )?;
    let mut snapshot = device.snapshot()?;
    // Literal independent E8 witnesses. These are fixture bytes, not a CPU model.
    let a = [2, 2, 0, 0, 0, 0, 0, 0];
    let negative = [-2, -2, 0, 0, 0, 0, 0, 0];
    let b = [0, 0, 2, 0, 2, 0, 0, 0];
    for token in 0..vocabulary as usize {
        for block in 0..2 {
            snapshot.table[(token * 2 + block) * 8..(token * 2 + block + 1) * 8].copy_from_slice(
                if token == 1 {
                    &negative
                } else if token == 2 {
                    &b
                } else {
                    &a
                },
            );
        }
    }
    Ok(snapshot)
}
fn exact_writer() -> Result<()> {
    let device = Device::from_snapshot(fixture(2, 1, 2)?, 0)?;
    let carried = device.carry(&[identity(0, 0), identity(1, 1)])?;
    let result = device.forward(&carried, &[0, 0])?;
    require(
        result.predictions == [0, 1],
        "literal two-token ranking differs",
    )?;
    require(
        result.losses == [0, 512],
        "exact token+address margin numerators differ",
    )?;
    require(
        result.vectors()? == carried.values()?,
        "one-leaf transport differs from identity",
    )
}
fn proposals() -> Result<()> {
    let device = Device::from_snapshot(fixture(2, 1, 2)?, 0)?;
    let input = device.carry(&[identity(1, 0)])?;
    let before = device.snapshot()?;
    let proposal = device
        .propose(&input, &[0], Parameter::VectorBlock(0))?
        .observation()?;
    require(
        proposal.candidate > 0
            && proposal.before == 512
            && proposal.after == 480
            && proposal.epoch == 0,
        "nonvacuous exact geometric codebook proposal differs",
    )?;
    let noop = device
        .propose(&input, &[0], Parameter::Route(0))?
        .observation()?;
    require(
        noop.candidate == 0 && noop.before == 512 && noop.after == 512,
        "identity must win a complete no-choice route tie",
    )?;
    let after = device.snapshot()?;
    require(
        before.table == after.table && before.routes == after.routes && before.epoch == after.epoch,
        "virtual proposal changed learned state",
    )
}
fn context_transport() -> Result<()> {
    let device = Device::from_snapshot(fixture(3, 3, 3)?, 0)?;
    // The last token's B_12*a query chooses the middle FBS child. Its carrier
    // is -a, and actual BFS transport must produce -B_12*a, not the query.
    let input = device.carry(&[identity(0, 0), identity(1, 1), identity(2, 2)])?;
    let result = device.forward(&input, &[0])?;
    let expected = [0, 0, -2, 0, -2, 0, 0, 0, 0, 0, -2, 0, -2, 0, 0, 0];
    require(
        result.vectors()? == expected,
        "actual FBS earlier-child transport differs",
    )?;
    let second = device.carry(&[identity(0, 0), identity(0, 1), identity(2, 2)])?;
    let changed = device.forward(&second, &[0])?;
    let expected_changed = [0, 0, 2, 0, 2, 0, 0, 0, 0, 0, 2, 0, 2, 0, 0, 0];
    require(
        changed.vectors()? == expected_changed,
        "selected earlier occurrence does not reach writer",
    )
}
fn isolated_windows() -> Result<()> {
    let device = Device::from_snapshot(fixture(3, 3, 3)?, 0)?;
    let input = device.carry(&[
        identity(0, 0),
        identity(1, 1),
        identity(2, 2),
        identity(1, 3),
        identity(0, 4),
        identity(2, 5),
    ])?;
    let first = device.forward(&input, &[0, 0])?;
    let changed = device.carry(&[
        identity(0, 0),
        identity(1, 1),
        identity(2, 2),
        identity(2, 3),
        identity(2, 4),
        identity(0, 5),
    ])?;
    let second = device.forward(&changed, &[0, 1])?;
    require(
        first.vectors()?[..16] == second.vectors()?[..16]
            && first.predictions[0] == second.predictions[0]
            && first.losses[0] == second.losses[0],
        "a different window/future row changed earlier output",
    )
}
fn collision_and_omitted_target() -> Result<()> {
    let device = Device::from_snapshot(fixture(257, 1, 1)?, 0)?;
    let ids = [identity(0, 5), identity(256, 8)];
    let input = device.carry(&ids)?;
    require(
        input.identities() == ids,
        "colliding carrier identities were conflated",
    )?;
    let result = device.forward(&input, &[0, 256])?;
    require(
        result.predictions == [0, 0] && result.losses == [128, 128],
        "collision rank tie or legal absent-address rival differs",
    )?;
    let omitted = device.forward(&input, &[1, 1])?;
    require(
        omitted.predictions == [0, 0] && omitted.losses == [512, 512],
        "omitted target changed admission or escaped objective",
    )
}
fn nonvacuity() -> Result<()> {
    let device = Device::from_snapshot(fixture(2, 1, 1)?, 0)?;
    let input = device.carry(&[identity(0, 0)])?;
    require(
        device.forward(&input, &[0]).is_err(),
        "missing token rival incorrectly received perfect cost",
    )?;
    require(
        device
            .propose(&input, &[0], Parameter::VectorBlock(0))
            .is_err(),
        "invalid baseline admission was accepted for learning",
    )
}
fn artifact_and_identity() -> Result<()> {
    let original = Device::from_snapshot(fixture(2, 1, 2)?, 0)?;
    let input = original.carry(&[identity(1, 0)])?;
    let bytes =
        serde_json::to_vec(&original.snapshot()?).map_err(|e| Error::Invalid(e.to_string()))?;
    let restored = Device::from_snapshot(
        serde_json::from_slice(&bytes).map_err(|e| Error::Invalid(e.to_string()))?,
        0,
    )?;
    require(
        restored.forward(&input, &[0]).is_err(),
        "foreign live carrier accepted after artifact reload",
    )?;
    let recarried = restored.carry(input.identities())?;
    let a = original.forward(&input, &[0])?;
    let b = restored.forward(&recarried, &[0])?;
    require(
        a.predictions == b.predictions && a.losses == b.losses && a.vectors()? == b.vectors()?,
        "artifact round-trip changed exact output",
    )?;
    let mut invalid = original.snapshot()?;
    invalid.table[0] = 3;
    require(
        Device::from_snapshot(invalid, 0).is_err(),
        "non-geometric codebook admitted",
    )?;
    let mut wrong = original.snapshot()?;
    wrong.schema = "old-float-writer".into();
    require(
        Device::from_snapshot(wrong, 0).is_err(),
        "historical writer artifact admitted",
    )
}
fn inventory_bounds() -> Result<()> {
    let device = Device::from_snapshot(fixture(3, 3, 3)?, 0)?;
    let partial = device.carry(&[identity(0, 0)])?;
    require(
        device.forward(&partial, &[0]).is_err(),
        "incomplete context admitted",
    )?;
    require(
        device.carry(&[identity(3, 0)]).is_err(),
        "out-of-range token admitted",
    )?;
    let full = device.carry(&[identity(0, 0), identity(1, 1), identity(2, 2)])?;
    require(
        device
            .propose(&full, &[0], Parameter::Route(u32::MAX))
            .is_err(),
        "out-of-range proposal parameter admitted",
    )?;
    require(
        crate::training_admission().is_err(),
        "training hold released",
    )
}
pub const NAMES: &[&str] = &[
    "exact_writer",
    "virtual_proposals",
    "context_transport",
    "isolated_windows",
    "collision_and_omitted_target",
    "nonvacuity",
    "artifact_and_identity",
    "inventory_bounds",
];
/// Every independent control runs. Creating an exclusive report precedes CUDA.
pub fn run(report: &Path) -> Result<()> {
    std::fs::create_dir(report).map_err(|e| Error::Invalid(format!("exclusive report: {e}")))?;
    let controls: [fn() -> Result<()>; 8] = [
        exact_writer,
        proposals,
        context_transport,
        isolated_windows,
        collision_and_omitted_target,
        nonvacuity,
        artifact_and_identity,
        inventory_bounds,
    ];
    let mut records = Vec::new();
    let mut failed = false;
    for (name, test) in NAMES.iter().zip(controls) {
        let outcome = test();
        let record = match outcome {
            Ok(()) => serde_json::json!({"name":name,"status":"PASS"}),
            Err(e) => {
                failed = true;
                serde_json::json!({"name":name,"status":"FAIL","error":e.to_string()})
            }
        };
        println!("{record}");
        records.push(record);
    }
    let payload = serde_json::json!({"schema":crate::SCHEMA,"controls":records,"training":"HELD","applied_updates":0});
    std::fs::write(
        report.join("controls.json"),
        serde_json::to_vec_pretty(&payload).map_err(|e| Error::Invalid(e.to_string()))?,
    )
    .map_err(|e| Error::Invalid(e.to_string()))?;
    require(!failed, "one or more finite construction controls failed")
}
