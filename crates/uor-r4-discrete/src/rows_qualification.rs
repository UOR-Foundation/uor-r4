//! Finite CUDA parity controls for the opt-in row-parallel counterfactual.
//! Each case runs the ordinary `propose` and `propose_rows` on the same device,
//! carried batch, targets and parameter, and requires the same result: equal
//! sixteen-slot loss vectors and chooser observation on success, or the same
//! refusal text on failure, with learned state and epoch unchanged. No host
//! model forward exists; both sides are the device's own kernels. Inputs are
//! deterministic integer data (token identities and targets), not model output.
use crate::{
    device::{Device, Proposal},
    Error, Identity, Parameter, ReadSpec, Result, Spec,
};
use std::path::Path;

pub const NAMES: &[&str] = &[
    "rows_route_parity",
    "rows_flock_gate_parity",
    "rows_relation_gate_parity",
    "rows_refusal_parity",
];

fn require(ok: bool, why: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error::Invalid(why.into()))
    }
}

/// Deterministic integer stream for identities and targets.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self, bound: u32) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) % u64::from(bound)) as u32
    }
}

fn windows(rng: &mut Lcg, rows: usize, context: u32, vocabulary: u32) -> Vec<Vec<Identity>> {
    let mut occurrence = 0u64;
    (0..rows)
        .map(|_| {
            // Variable lengths 2..=context exercise padding and short plans.
            let length = 2 + rng.next(context - 1);
            (0..length)
                .map(|_| {
                    occurrence += 1;
                    Identity {
                        token: rng.next(vocabulary),
                        occurrence,
                        record: 17,
                        version: 9,
                    }
                })
                .collect()
        })
        .collect()
}

#[derive(Default)]
struct Tally {
    cases: u64,
    successes: u64,
    refusals: Vec<String>,
}

/// Run both paths on one case and require identical observable results.
fn compare(
    device: &Device,
    rows: &crate::device::RowsCounterfactual,
    windows: &[Vec<Identity>],
    targets: &[u32],
    parameter: Parameter,
    tally: &mut Tally,
) -> Result<()> {
    let before = device.snapshot()?;
    let input = device.carry_windows(windows)?;
    let ordinary = device.propose(&input, targets, parameter);
    let parallel = device.propose_rows(&input, targets, parameter, rows);
    let after = device.snapshot()?;
    require(
        before.table == after.table
            && before.routes == after.routes
            && before.gates == after.gates
            && before.epoch == after.epoch,
        "a virtual proposal changed learned state or epoch",
    )?;
    tally.cases += 1;
    match (ordinary, parallel) {
        (Ok(a), Ok(b)) => {
            let observe = |p: &Proposal| -> Result<(Vec<u64>, String)> {
                Ok((p.candidate_losses()?, format!("{:?}", p.observation()?)))
            };
            let (la, oa) = observe(&a)?;
            let (lb, ob) = observe(&b)?;
            require(
                la == lb,
                &format!("{parameter:?}: losses differ: ordinary {la:?} rows {lb:?}"),
            )?;
            require(
                oa == ob,
                &format!("{parameter:?}: observation differs: {oa} vs {ob}"),
            )?;
            tally.successes += 1;
            Ok(())
        }
        (Err(Error::Invalid(a)), Err(Error::Invalid(b))) => {
            require(
                a == b,
                &format!("{parameter:?}: refusal differs: {a:?} vs {b:?}"),
            )?;
            tally.refusals.push(a);
            Ok(())
        }
        (Err(error @ Error::Cuda(_)), _) | (_, Err(error @ Error::Cuda(_))) => Err(error),
        (a, b) => Err(Error::Invalid(format!(
            "{parameter:?}: one path refused and the other did not: ordinary ok={} rows ok={}",
            a.is_ok(),
            b.is_ok()
        ))),
    }
}

fn sweep(read: ReadSpec, blocks: u32, seed: u64, tally: &mut Tally) -> Result<()> {
    let (vocabulary, context) = (16, 12);
    let spec = Spec {
        vocabulary,
        blocks,
        context,
        kept_cells: 4,
    };
    for model_seed in [7u32, 11] {
        let device = Device::new_with_read(spec, read, model_seed, 0)?;
        let rows = device.rows_counterfactual()?;
        let gates = device.snapshot()?.gates.len() as u32;
        let mut rng = Lcg(seed ^ u64::from(model_seed));
        // One row, an odd count and the full 64-row admission bound.
        for row_count in [1usize, 7, 64] {
            let batch = windows(&mut rng, row_count, context, vocabulary);
            let targets: Vec<u32> = (0..row_count).map(|_| rng.next(vocabulary)).collect();
            let parameters: Vec<Parameter> = match read {
                ReadSpec::Route => (0..blocks)
                    .flat_map(|b| [Parameter::Route(b), Parameter::VectorBlock(b)])
                    .collect(),
                _ => (0..gates).map(Parameter::Gate).collect(),
            };
            for parameter in parameters {
                compare(&device, &rows, &batch, &targets, parameter, tally)?;
            }
        }
    }
    require(tally.cases > 0, "no parity case ran")
}

fn rows_route_parity(tally: &mut Tally) -> Result<()> {
    sweep(ReadSpec::Route, 2, 0x5157, tally)?;
    sweep(ReadSpec::Route, 4, 0x5158, tally)
}
fn rows_flock_gate_parity(tally: &mut Tally) -> Result<()> {
    sweep(ReadSpec::Flock { kept: 3 }, 2, 0xf10c, tally)?;
    sweep(ReadSpec::Flock { kept: 8 }, 4, 0xf10d, tally)
}
fn rows_relation_gate_parity(tally: &mut Tally) -> Result<()> {
    sweep(ReadSpec::Relation { kept: 3, sweeps: 1 }, 2, 0x2e1a, tally)?;
    sweep(ReadSpec::Relation { kept: 8, sweeps: 2 }, 4, 0x2e1b, tally)
}
/// Kind/mode and index refusals must read identically on both paths.
fn rows_refusal_parity(tally: &mut Tally) -> Result<()> {
    let spec = Spec {
        vocabulary: 16,
        blocks: 2,
        context: 12,
        kept_cells: 4,
    };
    let mut rng = Lcg(0x0e1f);
    for (read, parameters) in [
        (ReadSpec::Route, vec![Parameter::Gate(0)]),
        (
            ReadSpec::Flock { kept: 3 },
            vec![Parameter::Route(0), Parameter::Gate(u32::MAX)],
        ),
    ] {
        let device = Device::new_with_read(spec, read, 7, 0)?;
        let rows = device.rows_counterfactual()?;
        let batch = windows(&mut rng, 3, 12, 16);
        let targets = [1, 2, 3];
        for parameter in parameters {
            let before = tally.refusals.len();
            compare(&device, &rows, &batch, &targets, parameter, tally)?;
            require(
                tally.refusals.len() == before + 1,
                &format!("{parameter:?} under {read:?} was not refused"),
            )?;
        }
    }
    Ok(())
}

pub fn run(report: &Path) -> Result<()> {
    std::fs::create_dir(report).map_err(|e| Error::Invalid(format!("exclusive report: {e}")))?;
    let controls: [fn(&mut Tally) -> Result<()>; 4] = [
        rows_route_parity,
        rows_flock_gate_parity,
        rows_relation_gate_parity,
        rows_refusal_parity,
    ];
    let mut records = Vec::new();
    let mut failed = false;
    for (index, (name, control)) in NAMES.iter().zip(controls).enumerate() {
        let mut tally = Tally::default();
        let record = match control(&mut tally) {
            Ok(()) => serde_json::json!({
                "index": index + 1, "name": name, "status": "PASS",
                "cases": tally.cases, "successes": tally.successes,
                "refusals": tally.refusals,
            }),
            Err(error) => {
                failed = true;
                serde_json::json!({
                    "index": index + 1, "name": name, "status": "FAIL",
                    "cases": tally.cases, "error": error.to_string(),
                })
            }
        };
        println!("{record}");
        records.push(record);
    }
    let payload = serde_json::json!({
        "schema": crate::SCHEMA,
        "suite": "rows-counterfactual-parity",
        "status": if failed { "FAILED" } else { "PASS" },
        "rows_image_sha256": crate::device::compiled_rows_image_sha256(),
        "controls": records,
        "training": "HELD",
        "applied_updates": 0,
    });
    std::fs::write(
        report.join("rows-parity.json"),
        serde_json::to_vec_pretty(&payload).map_err(|e| Error::Invalid(e.to_string()))?,
    )
    .map_err(|e| Error::Invalid(e.to_string()))?;
    require(!failed, "one or more row-parallel parity controls failed")
}
