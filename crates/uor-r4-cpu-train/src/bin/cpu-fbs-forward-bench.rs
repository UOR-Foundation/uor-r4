use std::time::Instant;

use uor_r4_cpu_train::{CpuFbsForward, CpuFbsHead};
use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth};
use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, FbsMemoryEntry, FbsMemorySnapshot};

const WARMUP_STEPS: usize = 4;
const TIMED_STEPS: usize = 64;

fn main() {
    let mut snapshot = synthetic_fbs_trunk_with(
        20_261_101,
        FbsSynth {
            vocab: 4_096,
            d: 256,
            mlp: 384,
            layers: 6,
            context: 192,
            kept: 8,
            gain_lo: 0.75,
            gain_span: 1.5,
            table: 0.25,
            decay_step: 0.0625,
        },
    );
    let no_read = export_fbs_trunk_uorlut01(&snapshot).expect("synthetic FBS artifact");

    let mut copy_only = snapshot.clone();
    let mut copy_heads = synthetic_hamming(snapshot.width, snapshot.layers.len(), 8, 20_261_102);
    for read in &mut copy_heads.reads {
        let (copy, types) = read.weight.split_at_mut(snapshot.width);
        copy.fill(1.0);
        types.fill(0.0);
    }
    copy_only.hamming = Some(copy_heads);
    let copy_bytes = export_fbs_trunk_uorlut01(&copy_only).expect("synthetic copy-only artifact");

    let mut hamming = synthetic_hamming(snapshot.width, snapshot.layers.len(), 8, 20_261_103);
    for read in &mut hamming.reads {
        read.weight.fill(1.0);
    }
    snapshot.hamming = Some(hamming);
    let with_read = export_fbs_trunk_uorlut01(&snapshot).expect("synthetic Hamming artifact");

    let mut with_memory = snapshot.clone();
    let mut reply_entries: Vec<FbsMemoryEntry> = (1..TIMED_STEPS)
        .map(|step| {
            let token = ((step * 97 + 11) % 4_096) as u32;
            FbsMemoryEntry {
                key: vec![
                    11,
                    uor_r4_integer::stack::MEMORY_PAD,
                    step as u32,
                    uor_r4_integer::stack::MEMORY_PAD,
                    token,
                ],
                value: 42,
            }
        })
        .collect();
    reply_entries.sort_by(|left, right| left.key.cmp(&right.key));
    with_memory.memory = Some(FbsMemorySnapshot {
        k: 1,
        key_window: 192,
        opening: vec![FbsMemoryEntry {
            key: vec![11],
            value: 42,
        }],
        opening_weight: vec![1.0; snapshot.width],
        reply: Some(uor_r4_integer::stack::FbsReplyTable {
            turn: 1,
            entries: reply_entries,
            weight: vec![1.0; snapshot.width],
        }),
    });
    let full_path =
        export_fbs_trunk_uorlut01(&with_memory).expect("synthetic Hamming and memory artifact");

    benchmark("none", &no_read, false);
    benchmark("copy", &copy_bytes, false);
    benchmark("copy+type", &with_read, false);
    benchmark("copy+type+memory", &full_path, true);
}

fn benchmark(read: &str, bytes: &[u8], memory: bool) {
    let mut forward = CpuFbsForward::from_artifact(bytes).expect("CPU FBS forward");
    if memory {
        forward.mark_reply_start();
    }
    for step in 0..WARMUP_STEPS {
        forward
            .step((step * 97 + 11) as u32, CpuFbsHead::Nearest)
            .expect("warmup");
    }
    forward = CpuFbsForward::from_artifact(bytes).expect("fresh CPU FBS forward");
    if memory {
        forward.mark_reply_start();
    }

    let started = Instant::now();
    let mut checksum = 0u64;
    for step in 0..TIMED_STEPS {
        let result = forward
            .step(((step * 97 + 11) % 4_096) as u32, CpuFbsHead::Nearest)
            .expect("timed step");
        checksum = checksum
            .wrapping_mul(131)
            .wrapping_add(u64::from(result.token));
    }
    let seconds = started.elapsed().as_secs_f64();
    println!(
        "schema=uor-r4.cpu-fbs-forward-bench/2 read={read} vocab={} width={} mlp={} layers={} context={} timed_steps={} elapsed_ms={:.3} tokens_per_second={:.3} checksum={checksum} peak_rss_kib={}",
        forward.vocabulary(),
        forward.width(),
        384,
        6,
        192,
        TIMED_STEPS,
        seconds * 1_000.0,
        TIMED_STEPS as f64 / seconds,
        peak_rss_kib()
    );
}

fn peak_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("VmHWM:")
                    .and_then(|value| value.split_whitespace().next())
                    .and_then(|value| value.parse().ok())
            })
        })
        .unwrap_or(0)
}
