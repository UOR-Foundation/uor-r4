use std::time::Instant;

use uor_r4_cpu_train::{CpuRankMarginTrainer, RankMarginConfig, RankMarginSample, VOCABULARY};
use uor_r4_discrete::normalization_mechanism::orbit_root_coords;
use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth};

const WARMUP_STEPS: usize = 4;
const TIMED_STEPS: usize = 64;

fn main() {
    let mut snapshot = synthetic_fbs_trunk_with(
        20_261_103,
        FbsSynth {
            vocab: VOCABULARY,
            d: 64,
            mlp: 64,
            layers: 2,
            context: 8,
            kept: 8,
            gain_lo: 0.75,
            gain_span: 1.5,
            table: 0.25,
            decay_step: 0.0625,
        },
    );
    snapshot.embed.clear();
    for token in 0..snapshot.vocab {
        for block in 0..snapshot.width / 8 {
            let root = match block {
                0 => token & 15,
                7 => ((token >> 4) ^ (token >> 8)) & 15,
                _ => ((token >> 8) * 7 + ((token >> 4) & 15) * 3 + (token & 15) * 5 + block) & 15,
            };
            snapshot.embed.extend(
                orbit_root_coords(root as u8)
                    .unwrap()
                    .map(|value| value as f32),
            );
        }
    }
    let mut reads = synthetic_hamming(snapshot.width, snapshot.layers.len(), 4, 20_261_104);
    reads
        .reads
        .iter_mut()
        .for_each(|read| read.weight.fill(1.0));
    snapshot.hamming = Some(reads);
    let artifact = export_fbs_trunk_uorlut01(&snapshot).expect("canonical FBS artifact");
    let config = RankMarginConfig::default();

    let mut probe = CpuRankMarginTrainer::from_artifact(
        &artifact,
        vec![RankMarginSample {
            input: vec![0],
            target: 0,
        }],
        config,
        20_261_105,
    )
    .expect("rank-margin probe");
    let sample = (0u32..VOCABULARY as u32)
        .find_map(|input| {
            let reading = probe
                .evaluate_sample(&RankMarginSample {
                    input: vec![input],
                    target: 0,
                })
                .expect("sample reading");
            reading
                .top8
                .iter()
                .copied()
                .filter(|&target| target != reading.top1)
                .find_map(|target| {
                    let sample = RankMarginSample {
                        input: vec![input],
                        target,
                    };
                    let mut trial = CpuRankMarginTrainer::from_artifact(
                        &artifact,
                        vec![sample.clone()],
                        config,
                        20_261_105,
                    )
                    .expect("rank-margin trial");
                    trial
                        .train_steps(1)
                        .expect("rank-margin trial")
                        .first()
                        .is_some_and(|step| step.updated)
                        .then_some(sample)
                })
        })
        .expect("a sample with one accepted update");

    let mut warmup =
        CpuRankMarginTrainer::from_artifact(&artifact, vec![sample.clone()], config, 20_261_105)
            .expect("rank-margin warmup");
    warmup.train_steps(WARMUP_STEPS).expect("warmup");

    let mut trainer =
        CpuRankMarginTrainer::from_artifact(&artifact, vec![sample], config, 20_261_105)
            .expect("rank-margin trainer");
    let started = Instant::now();
    let steps = trainer.train_steps(TIMED_STEPS).expect("timed training");
    let seconds = started.elapsed().as_secs_f64();
    let accepted = steps.iter().filter(|step| step.updated).count();
    let final_loss = steps.last().map_or(0, |step| step.after.margin_loss);
    let checkpoint = trainer.checkpoint().expect("checkpoint");
    println!(
        "schema=uor-r4.cpu-rank-margin-bench/1 vocab={} width={} layers=2 context=8 timed_steps={} accepted_updates={} elapsed_ms={:.3} steps_per_second={:.3} final_margin_loss={final_loss} checkpoint_sha256={} peak_rss_kib={}",
        VOCABULARY,
        64,
        TIMED_STEPS,
        accepted,
        seconds * 1_000.0,
        TIMED_STEPS as f64 / seconds,
        blake3::hash(&checkpoint).to_hex(),
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
