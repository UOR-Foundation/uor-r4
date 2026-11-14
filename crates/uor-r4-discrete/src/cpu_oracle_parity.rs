//! CUDA leg of the native-space parity contract (claude-372).
//!
//! The CUDA training kernels and the CPU training oracle (`uor-r4-cpu-train`,
//! D84) start from byte-identical `UORLUT01` bytes and are compared word for
//! word: read vectors (`discrete_forward`), the sixteen counterfactual loss
//! words (`discrete_counterfactual`), the chosen proposal (`discrete_choose`)
//! and the exported bytes after one commit (`discrete_apply`). The two sides
//! exchange bytes and integer words only. This module is test-only: it adds
//! no host scorer, chooser or fallback to any product path.

use crate::d11_export::export_d11;
use crate::device::Device;
use crate::{Identity, ObjectivePolicy, Parameter, ReadInitialization, ReadSpec, Spec};
use uor_r4_cpu_train::{
    CpuDiscreteProposalApplier, CpuDiscreteProposalChooser, CpuDiscreteProposalKind,
    CpuDiscreteProposalSample, CpuDiscreteProposalScorer,
};

const SPEC: Spec = Spec {
    vocabulary: 3,
    blocks: 2,
    context: 3,
    kept_cells: 2,
};
const SEED: u32 = 0x5172;

/// All 39 prefixes of lengths 1 to 3 over three tokens; target = sum % 3.
fn samples() -> Vec<CpuDiscreteProposalSample> {
    let mut out = Vec::new();
    for len in 1..=3u32 {
        for code in 0..3u32.pow(len) {
            let prefix: Vec<u32> = (0..len).map(|i| code / 3u32.pow(len - 1 - i) % 3).collect();
            let target = prefix.iter().sum::<u32>() % 3;
            out.push(CpuDiscreteProposalSample { prefix, target });
        }
    }
    out
}

fn windows(samples: &[CpuDiscreteProposalSample]) -> Vec<Vec<Identity>> {
    samples
        .iter()
        .map(|s| {
            s.prefix
                .iter()
                .enumerate()
                .map(|(i, &token)| Identity {
                    token,
                    occurrence: i as u64,
                    record: 17,
                    version: 9,
                })
                .collect()
        })
        .collect()
}

/// Route models start with zero gates. Flock models start `active_leaf_v1`:
/// the CPU Flock oracle refuses CUDA's default Flock initialization (zero gates).
fn device(read: ReadSpec) -> Device {
    let init = match read {
        ReadSpec::Route => ReadInitialization::ZeroGatesV1,
        _ => ReadInitialization::ActiveLeafV1,
    };
    Device::new_with_initialization(
        SPEC,
        read,
        ObjectivePolicy::NearestAddressMarginV1,
        init,
        SEED,
        0,
    )
    .expect("device")
}

fn bytes_of(device: &Device) -> Vec<u8> {
    export_d11(&device.snapshot().expect("snapshot")).expect("export_d11")
}

/// Every parameter the model admits, with the CPU kind and index it maps to.
fn parameters(device: &Device, read: ReadSpec) -> Vec<(Parameter, CpuDiscreteProposalKind, usize)> {
    let snapshot = device.snapshot().expect("snapshot");
    let mut out = Vec::new();
    for i in 0..snapshot.routes.len() {
        out.push((
            Parameter::Route(i as u32),
            CpuDiscreteProposalKind::Route,
            i,
        ));
    }
    for i in 0..snapshot.table.len() / 8 {
        out.push((
            Parameter::VectorBlock(i as u32),
            CpuDiscreteProposalKind::Table,
            i,
        ));
    }
    if read != ReadSpec::Route {
        for i in 0..snapshot.gates.len() {
            out.push((Parameter::Gate(i as u32), CpuDiscreteProposalKind::Gate, i));
        }
    }
    out
}

/// One comparison pass. Returns the mismatches as readable lines and the
/// counts of words compared, so a fault test can show which step caught it.
#[derive(Default, Debug)]
struct Reading {
    forward_words: usize,
    loss_words: usize,
    choices: usize,
    /// Parameters both sides refuse.
    refusals: usize,
    /// Artifact pairs whose raw bytes differ only in header key order.
    key_order_only: usize,
    /// Artifact pairs whose headers differ only in `source.snapshot_sha256`.
    snapshot_sha_only: usize,
    /// Named findings that are not mismatches.
    notes: Vec<String>,
    mismatches: Vec<String>,
}

impl Reading {
    fn steps(&self) -> Vec<&'static str> {
        let mut steps: Vec<&'static str> = self
            .mismatches
            .iter()
            .map(|m| m.split(' ').next().unwrap_or(""))
            .map(|s| match s {
                "forward" => "forward",
                "losses" => "losses",
                "choose" => "choose",
                "apply" => "apply",
                "load" => "load",
                _ => "other",
            })
            .collect();
        steps.sort();
        steps.dedup();
        steps
    }
}

fn compare_forward(
    device: &Device,
    cpu: &CpuDiscreteProposalScorer,
    rows: &[CpuDiscreteProposalSample],
    r: &mut Reading,
) {
    let batch = device.carry_windows(&windows(rows)).expect("carry");
    let targets: Vec<u32> = rows.iter().map(|s| s.target).collect();
    let vectors = device
        .forward(&batch, &targets)
        .expect("forward")
        .vectors()
        .expect("vectors");
    let width = vectors.len() / rows.len();
    for (row, sample) in rows.iter().enumerate() {
        let host = cpu.query(sample).expect("cpu query");
        let gpu = &vectors[row * width..row * width + width];
        r.forward_words += gpu.len();
        if host.as_slice() != gpu {
            r.mismatches.push(format!(
                "forward row {row} prefix {:?}: cuda {gpu:?} cpu {host:?}",
                sample.prefix
            ));
        }
    }
}

/// Losses and choice for every parameter. Returns the first parameter whose
/// CUDA proposal commits, for the apply step. CUDA reads `rows`; the CPU
/// scores `cpu_rows` (the same rows in a clean pass). With `swap_lane`, the
/// CPU chooser gets the CUDA losses with CUDA's chosen lane swapped into lane
/// 0, so the planted losses and choose faults run through this same code.
#[allow(clippy::too_many_arguments)]
fn compare_proposals(
    device: &Device,
    read: ReadSpec,
    scorer: &CpuDiscreteProposalScorer,
    chooser: &CpuDiscreteProposalChooser,
    rows: &[CpuDiscreteProposalSample],
    cpu_rows: &[CpuDiscreteProposalSample],
    swap_lane: bool,
    r: &mut Reading,
) -> Option<(Parameter, CpuDiscreteProposalKind, usize)> {
    let batch = device.carry_windows(&windows(rows)).expect("carry");
    let targets: Vec<u32> = rows.iter().map(|s| s.target).collect();
    let epoch = device.epoch().expect("epoch");
    let mut first_commit = None;
    for (parameter, kind, index) in parameters(device, read) {
        // A parameter CUDA refuses must be one the CPU scorer refuses too.
        let proposal = match device.propose(&batch, &targets, parameter) {
            Ok(proposal) => proposal,
            Err(error) => {
                match scorer.score(kind, index, cpu_rows, epoch) {
                    Err(cpu_error) => {
                        r.refusals += 1;
                        r.notes.push(format!(
                            "refused {parameter:?}: cuda ({error}) cpu ({cpu_error})"
                        ));
                    }
                    Ok(_) => r.mismatches.push(format!(
                        "losses {parameter:?}: cuda refused ({error}), cpu scored"
                    )),
                }
                continue;
            }
        };
        let gpu_losses = proposal.candidate_losses().expect("losses");
        let observed = proposal.observation().expect("observation");
        let host = match scorer.score(kind, index, cpu_rows, epoch) {
            Ok(host) => host,
            Err(error) => {
                r.mismatches
                    .push(format!("losses {parameter:?}: cpu refused ({error})"));
                continue;
            }
        };
        r.loss_words += gpu_losses.len();
        if gpu_losses.as_slice() != host.losses.as_slice() {
            r.mismatches.push(format!(
                "losses {parameter:?}: cuda {gpu_losses:?} cpu {:?}",
                host.losses
            ));
        }
        // The chooser runs on the CUDA loss words, so a choice mismatch is a
        // choose-rule difference, not an inherited loss difference.
        let mut gpu_words: [u64; 16] = match gpu_losses.clone().try_into() {
            Ok(words) => words,
            Err(_) => {
                r.mismatches.push(format!(
                    "choose {parameter:?}: cuda gave {} loss words",
                    gpu_losses.len()
                ));
                continue;
            }
        };
        if swap_lane && observed.candidate != 0 {
            gpu_words.swap(0, observed.candidate as usize);
        }
        let chosen = chooser
            .choose(kind, index, &gpu_words, epoch)
            .expect("cpu choose");
        r.choices += 1;
        let host_tuple = (
            chosen.lane as u64,
            chosen.current_loss,
            chosen.selected_loss,
            chosen.epoch,
            chosen.sensitive,
        );
        let gpu_tuple = (
            observed.candidate,
            observed.before,
            observed.after,
            observed.epoch,
            observed.any_candidate_cost_differs_from_identity,
        );
        if host_tuple != gpu_tuple {
            r.mismatches.push(format!(
                "choose {parameter:?}: cuda (lane, before, after, epoch, sensitive) {gpu_tuple:?} cpu {host_tuple:?}"
            ));
        }
        if first_commit.is_none() && observed.candidate != 0 {
            first_commit = Some((parameter, kind, index));
        }
    }
    first_commit
}

/// Commit one proposal on each side from the same bytes and compare bytes.
fn compare_apply(
    device: &mut Device,
    bytes: &[u8],
    pick: (Parameter, CpuDiscreteProposalKind, usize),
    rows: &[CpuDiscreteProposalSample],
    r: &mut Reading,
) {
    let (parameter, kind, index) = pick;
    let batch = device.carry_windows(&windows(rows)).expect("carry");
    let targets: Vec<u32> = rows.iter().map(|s| s.target).collect();
    let epoch = device.epoch().expect("epoch");
    let proposal = device
        .propose(&batch, &targets, parameter)
        .expect("propose");
    let committed = device.apply(&proposal).expect("cuda apply");
    let after_gpu = bytes_of(device);

    let scorer = CpuDiscreteProposalScorer::from_bytes(bytes).expect("cpu scorer");
    let chooser = CpuDiscreteProposalChooser::from_bytes(bytes).expect("cpu chooser");
    let mut applier = CpuDiscreteProposalApplier::from_bytes(bytes).expect("cpu applier");
    let losses = scorer
        .score(kind, index, rows, epoch)
        .expect("cpu score")
        .losses;
    let chosen = chooser
        .choose(kind, index, &losses, epoch)
        .expect("cpu choose");
    let receipt = applier.apply(kind, index, chosen).expect("cpu apply");
    let after_cpu = applier.artifact_bytes().expect("cpu bytes");

    if committed != receipt.applied {
        r.mismatches.push(format!(
            "apply {parameter:?}: cuda committed {committed}, cpu applied {}",
            receipt.applied
        ));
    }
    compare_bytes(&format!("apply {parameter:?}"), &after_gpu, &after_cpu, r);
}

/// A `UORLUT01` file split into its header JSON and its data section
/// (header padding included in the data, so a non-zero pad byte shows).
fn split(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let len = u64::from_le_bytes(bytes.get(8..16)?.try_into().ok()?) as usize;
    let end = 16usize.checked_add(len)?;
    Some((bytes.get(16..end)?, bytes.get(end..)?))
}

/// Compare two artifacts section by section. Header values and data bytes
/// that differ are mismatches. Raw bytes that differ only in the order of
/// header keys are counted in `key_order_only`, not called a mismatch: both
/// headers carry the same values and both loaders read them.
fn compare_bytes(label: &str, cuda: &[u8], cpu: &[u8], r: &mut Reading) {
    if cuda == cpu {
        return;
    }
    let (Some((cuda_header, cuda_data)), Some((cpu_header, cpu_data))) = (split(cuda), split(cpu))
    else {
        r.mismatches
            .push(format!("{label}: an artifact has no readable header"));
        return;
    };
    let cuda_value: Result<serde_json::Value, _> = serde_json::from_slice(cuda_header);
    let cpu_value: Result<serde_json::Value, _> = serde_json::from_slice(cpu_header);
    let mut differs = false;
    match (cuda_value, cpu_value) {
        (Ok(a), Ok(b)) if a == b => {}
        (Ok(mut a), Ok(mut b)) => {
            // Name the one known header split on its own: the CPU re-save keeps
            // the snapshot hash of the bytes it loaded after it commits a step.
            let sha = |v: &mut serde_json::Value| {
                v.get_mut("source")
                    .and_then(|s| s.as_object_mut())
                    .and_then(|s| s.remove("snapshot_sha256"))
            };
            let (cuda_sha, cpu_sha) = (sha(&mut a), sha(&mut b));
            if a == b && cuda_sha != cpu_sha {
                r.snapshot_sha_only += 1;
                r.notes.push(format!(
                    "{label}: only source.snapshot_sha256 differs: cuda {} cpu {}",
                    cuda_sha.unwrap_or_default(),
                    cpu_sha.unwrap_or_default()
                ));
            } else {
                r.mismatches
                    .push(format!("{label}: header values differ: cuda {a} cpu {b}"));
            }
            differs = true;
        }
        _ => {
            differs = true;
            r.mismatches.push(format!("{label}: a header is not JSON"));
        }
    }
    if cuda_data != cpu_data {
        differs = true;
        let first = cuda_data
            .iter()
            .zip(cpu_data)
            .position(|(a, b)| a != b)
            .unwrap_or(cuda_data.len().min(cpu_data.len()));
        r.mismatches.push(format!(
            "{label}: data differs from data offset {first} (cuda {} bytes, cpu {} bytes)",
            cuda_data.len(),
            cpu_data.len()
        ));
    }
    if !differs {
        r.key_order_only += 1;
    }
}

/// The full contract on one fixture. `cpu_bytes` is what the CPU side loads;
/// in the clean run it is the CUDA model's own export.
fn run(read: ReadSpec, cpu_bytes: Option<Vec<u8>>, rows: &[CpuDiscreteProposalSample]) -> Reading {
    let mut gpu = device(read);
    let start = bytes_of(&gpu);
    let cpu_bytes = cpu_bytes.unwrap_or_else(|| start.clone());
    let mut r = Reading::default();

    match CpuDiscreteProposalApplier::from_bytes(&cpu_bytes).and_then(|a| a.artifact_bytes()) {
        Ok(saved) => compare_bytes("load cpu load-then-save", &cpu_bytes, &saved, &mut r),
        Err(error) => r
            .mismatches
            .push(format!("load cpu refuses the cuda export: {error}")),
    }
    let scorer = CpuDiscreteProposalScorer::from_bytes(&cpu_bytes).expect("cpu scorer");
    let chooser = CpuDiscreteProposalChooser::from_bytes(&cpu_bytes).expect("cpu chooser");
    compare_forward(&gpu, &scorer, rows, &mut r);
    let pick = compare_proposals(&gpu, read, &scorer, &chooser, rows, rows, false, &mut r);
    match pick {
        Some(pick) => compare_apply(&mut gpu, &cpu_bytes, pick, rows, &mut r),
        None => r
            .mismatches
            .push("apply no parameter's cuda proposal commits on this fixture".to_string()),
    }
    r
}

const FIXTURES: [ReadSpec; 2] = [ReadSpec::Route, ReadSpec::Flock { kept: 2 }];

#[test]
fn cuda_and_cpu_oracle_agree_word_for_word_from_identical_bytes() {
    let rows = samples();
    assert_eq!(rows.len(), 39);
    let readings: Vec<(ReadSpec, Reading)> = FIXTURES
        .into_iter()
        .map(|read| (read, run(read, None, &rows)))
        .collect();
    for (read, r) in &readings {
        eprintln!(
            "{read:?}: {} forward words, {} loss words, {} choices, {} refusals both sides, {} key-order-only, {} snapshot-sha-only, {} mismatches",
            r.forward_words,
            r.loss_words,
            r.choices,
            r.refusals,
            r.key_order_only,
            r.snapshot_sha_only,
            r.mismatches.len()
        );
        for line in r.notes.iter().chain(&r.mismatches) {
            eprintln!("  {line}");
        }
    }
    for (read, r) in &readings {
        assert!(r.forward_words > 0 && r.loss_words > 0 && r.choices > 0);
        assert!(r.mismatches.is_empty(), "{read:?}: {:?}", r.mismatches);
        // Pinned findings, not mismatches. At load, the CPU re-save writes the
        // same header values in another key order. After one committed step,
        // the data agree and only the header's snapshot hash differs.
        assert_eq!((r.key_order_only, r.snapshot_sha_only), (1, 1), "{read:?}");
    }
}

#[test]
fn cuda_parity_catches_one_planted_fault_per_step() {
    let rows = samples();
    let read = ReadSpec::Route;
    let gpu = device(read);
    let start = bytes_of(&gpu);
    let scorer = CpuDiscreteProposalScorer::from_bytes(&start).unwrap();
    let chooser = CpuDiscreteProposalChooser::from_bytes(&start).unwrap();

    // Every fault below runs through the same compare_* function the clean
    // test uses, and must be reported at exactly its own step.

    // F-forward: the CPU loads the CUDA export with one byte of the data
    // section changed. Some loadable one-byte fault must change a read word.
    let (header, data) = split(&start).unwrap();
    let data_start = 16 + header.len();
    let caught_forward = (data_start..data_start + data.len()).any(|offset| {
        let mut fault = start.clone();
        fault[offset] ^= 2;
        let Ok(faulted) = CpuDiscreteProposalScorer::from_bytes(&fault) else {
            return false;
        };
        let mut r = Reading::default();
        compare_forward(&gpu, &faulted, &rows, &mut r);
        r.steps() == vec!["forward"]
    });
    assert!(
        caught_forward,
        "no loadable one-byte data fault changed a read word"
    );

    // F-losses: one target moved on the CPU side only. The choice runs on the
    // CUDA loss words, so only the loss step may report it.
    let mut shifted = rows.clone();
    shifted[0].target = (shifted[0].target + 1) % 3;
    let mut r = Reading::default();
    compare_proposals(
        &gpu, read, &scorer, &chooser, &rows, &shifted, false, &mut r,
    );
    assert_eq!(r.steps(), vec!["losses"], "{:?}", r.mismatches);

    // F-choose: the CPU chooser gets the CUDA losses with CUDA's chosen lane
    // swapped into lane 0. Only the choice step may report it.
    let mut r = Reading::default();
    compare_proposals(&gpu, read, &scorer, &chooser, &rows, &rows, true, &mut r);
    assert_eq!(r.steps(), vec!["choose"], "{:?}", r.mismatches);

    // F-apply: CUDA commits a proposal; the CPU commits the noop lane for the
    // same parameter, which it accepts as "not applied". The after-step data
    // section must differ.
    let mut gpu = device(read);
    let mut r_pick = Reading::default();
    let (parameter, kind, index) = compare_proposals(
        &gpu,
        read,
        &scorer,
        &chooser,
        &rows,
        &rows,
        false,
        &mut r_pick,
    )
    .expect("a committing parameter");
    assert!(r_pick.mismatches.is_empty(), "{:?}", r_pick.mismatches);
    let targets: Vec<u32> = rows.iter().map(|s| s.target).collect();
    let batch = gpu.carry_windows(&windows(&rows)).unwrap();
    let proposal = gpu.propose(&batch, &targets, parameter).unwrap();
    assert!(gpu.apply(&proposal).unwrap());
    let after_gpu = bytes_of(&gpu);
    let mut noop = scorer.score(kind, index, &rows, 0).unwrap().proposal;
    noop.lane = 0;
    noop.selected_loss = noop.current_loss;
    let mut applier = CpuDiscreteProposalApplier::from_bytes(&start).unwrap();
    let receipt = applier
        .apply(kind, index, noop)
        .expect("a noop commit is accepted");
    assert!(!receipt.applied);
    let mut r = Reading::default();
    compare_bytes(
        "apply",
        &after_gpu,
        &applier.artifact_bytes().unwrap(),
        &mut r,
    );
    assert!(
        r.mismatches.iter().any(|m| m.contains("data differs")),
        "{:?}",
        r.mismatches
    );

    // F-bytes: one data byte changed in an otherwise identical copy is a data
    // mismatch, and a key-order-only re-save is not.
    let mut r = Reading::default();
    let mut fault = after_gpu.clone();
    let last = fault.len() - 1;
    fault[last] ^= 1;
    compare_bytes("bytes", &after_gpu, &fault, &mut r);
    assert_eq!(r.mismatches.len(), 1, "{:?}", r.mismatches);
    assert!(r.mismatches[0].contains("data differs"));
    let resaved = CpuDiscreteProposalApplier::from_bytes(&after_gpu)
        .unwrap()
        .artifact_bytes()
        .unwrap();
    let mut r = Reading::default();
    compare_bytes("bytes", &after_gpu, &resaved, &mut r);
    assert!(r.mismatches.is_empty(), "{:?}", r.mismatches);
}
