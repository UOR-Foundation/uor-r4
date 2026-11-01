use std::time::Instant;

use uor_r4_cpu_train::{CpuFbsForward, CpuFbsHead};
use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};
use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, IntegerStackModel};

const WARMUP_STEPS: usize = 4;
const TIMED_STEPS: usize = 64;

fn main() {
    let snapshot = synthetic_fbs_trunk_with(
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
    let bytes = export_fbs_trunk_uorlut01(&snapshot).expect("synthetic FBS artifact");
    let model = IntegerStackModel::parse(&bytes).expect("parse FBS artifact");
    let image = model.fbs_image().expect("no-read FBS image");
    let mut forward =
        CpuFbsForward::from_image(image, model.rms_epsilon()).expect("CPU FBS forward");

    for step in 0..WARMUP_STEPS {
        forward
            .step((step * 97 + 11) as u32, CpuFbsHead::Nearest)
            .expect("warmup");
    }
    forward = CpuFbsForward::from_image(
        model.fbs_image().expect("no-read FBS image"),
        model.rms_epsilon(),
    )
    .expect("fresh CPU FBS forward");

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
        "schema=uor-r4.cpu-fbs-forward-bench/1 vocab={} width={} mlp={} layers={} context={} timed_steps={} elapsed_ms={:.3} tokens_per_second={:.3} checksum={checksum}",
        forward.vocabulary(),
        forward.width(),
        384,
        6,
        192,
        TIMED_STEPS,
        seconds * 1_000.0,
        TIMED_STEPS as f64 / seconds
    );
}
