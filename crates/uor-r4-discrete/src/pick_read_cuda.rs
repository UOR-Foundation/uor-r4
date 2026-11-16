//! CUDA leg of the Pick read (claude-408).
//!
//! The CUDA Pick read (`MODE_PICK` in `discrete.cu`) is compared word for word
//! with the CPU oracle `CpuDiscretePick` (`uor-r4-cpu-train`): the read vector
//! of every row and the first five words of each (row, block) trace record,
//! `[pick, d(pick), ties, gate+1, out root]`. Both sides start from one
//! snapshot: its token table, its level-0 gate trits and, for prefix
//! holonomy, its step table. Test-only; nothing here is a product path.

use crate::device::{Device, FLOCK_TRACE_HEADER_WORDS, FLOCK_TRACE_LEVEL_WORDS};
use crate::normalization_mechanism::{orbit_root_coords, NormalizationMode};
use crate::position_mechanism::{PositionConfig, PositionMode, PositionStepTable};
use crate::snapshot::Snapshot;
use crate::{Identity, ObjectivePolicy, ReadInitialization, ReadSpec, Spec};
use uor_r4_core::spiralcore_operator::cl06_finite_composition_table;
use uor_r4_cpu_train::{CpuDiscretePick, CpuDiscretePickTrace, CpuDiscretePickTransport};

const ABSENT: u32 = u32::MAX;
const SPEC: Spec = Spec {
    vocabulary: 16,
    blocks: 2,
    context: 9,
    kept_cells: 2,
};
const SEED: u32 = 0x408;
const SEQUENCES: [&[u32]; 4] = [
    &[0, 1, 2, 3, 4, 5, 1, 2, 9],
    &[7, 7, 7, 7, 7, 7, 7],
    &[3, 8, 3, 8, 3, 8, 15, 3, 8],
    &[12, 0, 5, 12, 0, 5, 12, 0],
];

fn patterned_steps(blocks: u32) -> PositionStepTable {
    let candidates = (0..blocks as usize)
        .flat_map(|b| (0..16usize).map(move |r| ((b * 5 + r * 3) % 15 + 1) as u8))
        .collect();
    PositionStepTable::new(blocks, candidates).expect("step table")
}

fn identities(prefix: &[u32]) -> Vec<Identity> {
    prefix
        .iter()
        .enumerate()
        .map(|(i, &token)| Identity {
            token,
            occurrence: i as u64,
            record: 17,
            version: 9,
        })
        .collect()
}

/// A Pick device over `spec`, loaded from a snapshot whose level-0 gates are
/// `gates0` (other levels 0) and, when `roots` is given, whose token `v`
/// block `b` is orbit root `roots(v, b)`.
fn pick_device(
    spec: Spec,
    position: PositionConfig,
    gates0: &[i32],
    roots: Option<&dyn Fn(u32, u32) -> u8>,
) -> (Device, Snapshot) {
    let fresh = Device::new_with_position(
        spec,
        ReadSpec::Pick,
        ObjectivePolicy::NearestAddressMarginV1,
        ReadInitialization::ZeroGatesV1,
        NormalizationMode::ScaleMatchedSnapV1,
        Some(position),
        SEED,
        0,
    )
    .expect("pick device");
    let mut snapshot = fresh.snapshot().expect("snapshot");
    drop(fresh);
    let blocks = spec.blocks as usize;
    let stages = snapshot.gates.len() / blocks;
    snapshot.gates.iter_mut().for_each(|g| *g = 0);
    for (b, &gate) in gates0.iter().enumerate() {
        snapshot.gates[b * stages] = gate;
    }
    if let Some(roots) = roots {
        for v in 0..spec.vocabulary {
            for b in 0..spec.blocks {
                let at = ((v * spec.blocks + b) * 8) as usize;
                let coords = orbit_root_coords(roots(v, b)).expect("root");
                snapshot.table[at..at + 8].copy_from_slice(&coords);
            }
        }
    }
    let device = Device::from_snapshot(snapshot.clone(), 0).expect("load pick snapshot");
    (device, snapshot)
}

/// The CPU oracle from the same snapshot, with the transport named.
fn oracle(snapshot: &Snapshot, transport_word: u32) -> CpuDiscretePick {
    let blocks = snapshot.spec.blocks as usize;
    let stages = snapshot.gates.len() / blocks;
    let token_blocks: Vec<Vec<[i32; 8]>> = snapshot
        .table
        .chunks_exact(8 * blocks)
        .map(|row| {
            row.chunks_exact(8)
                .map(|c| c.try_into().expect("8 words"))
                .collect()
        })
        .collect();
    let gates0 = (0..blocks).map(|b| snapshot.gates[b * stages]).collect();
    let transport = match transport_word {
        3 => CpuDiscretePickTransport::Identity,
        1 => {
            let steps = &snapshot.position.as_ref().expect("position").steps;
            let frames = (0..blocks)
                .flat_map(|b| (0..16u8).map(move |r| (b, r)))
                .map(|(b, r)| steps.step_action(b, r).expect("step"))
                .collect();
            CpuDiscretePickTransport::PrefixHolonomy { frames }
        }
        other => panic!("no Pick transport word {other}"),
    };
    CpuDiscretePick::from_parts(
        token_blocks,
        gates0,
        transport,
        cl06_finite_composition_table().expect("composition"),
    )
    .expect("cpu pick oracle")
}

/// Every row's CUDA read and trace record against the oracle. Returns the
/// mismatches and the count of rows compared.
fn compare(
    device: &Device,
    cpu: &CpuDiscretePick,
    prefixes: &[Vec<u32>],
    windows: &[Vec<Identity>],
) -> (Vec<String>, usize, bool) {
    let blocks = device.snapshot().expect("snapshot").spec.blocks as usize;
    let batch = device.carry_windows(windows).expect("carry");
    let targets: Vec<u32> = prefixes.iter().map(|p| p[p.len() - 1]).collect();
    let forward = device.forward_traced(&batch, &targets).expect("forward");
    let vectors = forward.vectors().expect("vectors");
    let trace = forward
        .flock_trace()
        .expect("trace")
        .expect("pick trace present");
    let width = vectors.len() / prefixes.len();
    assert_eq!(width, blocks * 8);
    let stride = trace.len() / (prefixes.len() * blocks);
    assert!(stride >= FLOCK_TRACE_HEADER_WORDS + FLOCK_TRACE_LEVEL_WORDS);
    let mut mismatches = Vec::new();
    let mut moved = false;
    for (row, prefix) in prefixes.iter().enumerate() {
        let host: CpuDiscretePickTrace = cpu.read_traced(prefix).expect("cpu read");
        let host_words: Vec<i32> = host.blocks.iter().flatten().copied().collect();
        let gpu = &vectors[row * width..(row + 1) * width];
        if gpu != host_words.as_slice() {
            mismatches.push(format!(
                "forward row {row} prefix {prefix:?}: cuda {gpu:?} cpu {host_words:?}"
            ));
        }
        let query = &batch_query(device, prefix);
        moved |= gpu != query.as_slice();
        for b in 0..blocks {
            let rec = &trace[(row * blocks + b) * stride..][..5];
            let gate = cpu_gate(device, b);
            let want = [
                host.pick.map_or(ABSENT, |p| p as u32),
                host.distance.unwrap_or(ABSENT),
                host.ties,
                (gate + 1) as u32,
                host.output_roots[b] as u32,
            ];
            if rec != want {
                mismatches.push(format!(
                    "trace row {row} block {b} prefix {prefix:?}: cuda {rec:?} cpu {want:?}"
                ));
            }
            if trace[(row * blocks + b) * stride + 5] != ABSENT {
                mismatches.push(format!("trace row {row} block {b}: word 5 is not ABSENT"));
            }
        }
    }
    (mismatches, prefixes.len(), moved)
}

fn batch_query(device: &Device, prefix: &[u32]) -> Vec<i32> {
    let snapshot = device.snapshot().expect("snapshot");
    let width = snapshot.spec.blocks as usize * 8;
    let token = prefix[prefix.len() - 1] as usize;
    snapshot.table[token * width..(token + 1) * width].to_vec()
}

fn cpu_gate(device: &Device, block: usize) -> i32 {
    let snapshot = device.snapshot().expect("snapshot");
    let stages = snapshot.gates.len() / snapshot.spec.blocks as usize;
    snapshot.gates[block * stages]
}

fn every_prefix() -> Vec<Vec<u32>> {
    SEQUENCES
        .iter()
        .flat_map(|s| (1..=s.len()).map(move |end| s[..end].to_vec()))
        .collect()
}

fn distinct_roots(v: u32, b: u32) -> u8 {
    if b == 0 {
        v as u8
    } else {
        ((v * 3 + 1) % 16) as u8
    }
}

/// G1/G2: CUDA forward and trace agree with the oracle on every prefix of
/// four sequences, at four gate settings and both transports; the oracle with
/// the other transport disagrees somewhere (the comparison can fail).
#[test]
#[ignore = "requires a CUDA GPU"]
fn cuda_pick_read_and_trace_match_cpu_oracle_on_every_prefix() {
    let prefixes = every_prefix();
    let windows: Vec<Vec<Identity>> = prefixes.iter().map(|p| identities(p)).collect();
    let mut compared = 0;
    let mut moved_any = false;
    let mut planted_caught = false;
    for (mode, word) in [
        (PositionMode::IdentityControlV1, 3u32),
        (PositionMode::PrefixHolonomyV1, 1u32),
    ] {
        for gates0 in [[1, 1], [1, -1], [0, 0], [-1, 1]] {
            let config = PositionConfig::new(mode, patterned_steps(SPEC.blocks));
            let (device, snapshot) =
                pick_device(SPEC, config, &gates0, Some(&distinct_roots));
            let (mismatches, rows, moved) =
                compare(&device, &oracle(&snapshot, word), &prefixes, &windows);
            assert!(
                mismatches.is_empty(),
                "word {word} gates {gates0:?}: {mismatches:#?}"
            );
            compared += rows;
            moved_any |= moved;
            if gates0 == [1, 1] {
                let other = if word == 1 { 3 } else { 1 };
                let (planted, _, _) =
                    compare(&device, &oracle(&snapshot, other), &prefixes, &windows);
                planted_caught |= !planted.is_empty();
            }
        }
    }
    assert_eq!(compared, 2 * 4 * (9 + 7 + 9 + 8));
    assert!(moved_any, "no CUDA row's read moved off its query");
    assert!(planted_caught, "a swapped transport went unseen");
}

/// G3: a planted copy. With distinct token rows, the prefix
/// `5 9 2 3 4 6 5 9` matches its last pair only at p = 1, so every block picks
/// 1 at distance 0 with no tie, and the read moves off the query.
#[test]
#[ignore = "requires a CUDA GPU"]
fn cuda_pick_read_copies_the_planted_pair() {
    let prefix = vec![5u32, 9, 2, 3, 4, 6, 5, 9];
    let config = PositionConfig::new(PositionMode::IdentityControlV1, patterned_steps(2));
    let (device, snapshot) = pick_device(SPEC, config, &[1, 1], Some(&distinct_roots));
    let batch = device
        .carry_windows(&[identities(&prefix)])
        .expect("carry");
    let forward = device.forward_traced(&batch, &[9]).expect("forward");
    let trace = forward.flock_trace().expect("trace").expect("present");
    let stride = trace.len() / 2;
    for b in 0..2 {
        assert_eq!(&trace[b * stride..b * stride + 3], &[1, 0, 1], "block {b}");
    }
    let host = oracle(&snapshot, 3).read(&prefix).expect("cpu read");
    let host: Vec<i32> = host.into_iter().flatten().collect();
    let gpu = forward.vectors().expect("vectors");
    assert_eq!(gpu, host);
    assert_ne!(gpu, batch_query(&device, &prefix));
}

/// G4: prefix holonomy over the identity step table is the identity
/// transport, row for row on CUDA.
#[test]
#[ignore = "requires a CUDA GPU"]
fn cuda_pick_holonomy_with_identity_steps_is_identity_transport() {
    let prefixes = every_prefix();
    let windows: Vec<Vec<Identity>> = prefixes.iter().map(|p| identities(p)).collect();
    let targets: Vec<u32> = prefixes.iter().map(|p| p[p.len() - 1]).collect();
    let steps = PositionStepTable::identity(SPEC.blocks).expect("identity steps");
    let read = |mode| {
        let config = PositionConfig::new(mode, steps.clone());
        let (device, _) = pick_device(SPEC, config, &[1, -1], Some(&distinct_roots));
        let batch = device.carry_windows(&windows).expect("carry");
        device
            .forward(&batch, &targets)
            .expect("forward")
            .vectors()
            .expect("vectors")
    };
    assert_eq!(
        read(PositionMode::PrefixHolonomyV1),
        read(PositionMode::IdentityControlV1)
    );
}

/// G6: the bucket writer carries no Pick read, so it refuses a Pick model.
#[test]
#[ignore = "requires a CUDA GPU"]
fn bucket_writer_refuses_a_pick_model() {
    use crate::device::writer_bucket_device::BucketWriterEngine;
    let config = PositionConfig::new(PositionMode::IdentityControlV1, patterned_steps(2));
    let (device, _) = pick_device(SPEC, config, &[1, 1], Some(&distinct_roots));
    let engine = BucketWriterEngine::from_device(&device).expect("bucket engine");
    let error = engine.build_index(&device).err().expect("Pick refused");
    assert!(error.to_string().contains("Pick"), "{error}");
}

/// G5: one 64-row student batch at corpus scale (4096 tokens, 4 blocks,
/// context 192) over the seeded table: CUDA forward and trace agree with the
/// oracle at both transports.
#[test]
#[ignore = "needs a GPU and UOR_STUDENT_STREAM (claude-408)"]
fn cuda_pick_read_matches_cpu_oracle_on_the_student_batch() {
    use crate::cpu_oracle_parity::{student_batch, STUDENT_SPEC};
    let student = student_batch();
    let prefixes: Vec<Vec<u32>> = student.rows.iter().map(|s| s.prefix.clone()).collect();
    eprintln!("student batch: {}; {} rows", student.pins, prefixes.len());
    for (mode, word) in [
        (PositionMode::IdentityControlV1, 3u32),
        (PositionMode::PrefixHolonomyV1, 1u32),
    ] {
        let config = PositionConfig::new(mode, patterned_steps(STUDENT_SPEC.blocks));
        let (device, snapshot) = pick_device(STUDENT_SPEC, config, &[1, -1, 1, 1], None);
        let (mismatches, rows, moved) = compare(
            &device,
            &oracle(&snapshot, word),
            &prefixes,
            &student.corpus_windows,
        );
        eprintln!(
            "student word {word}: {rows} rows, {} mismatches, moved {moved}",
            mismatches.len()
        );
        assert!(mismatches.is_empty(), "word {word}: {:#?}", &mismatches[..mismatches.len().min(8)]);
        assert_eq!(rows, 64);
    }
}
