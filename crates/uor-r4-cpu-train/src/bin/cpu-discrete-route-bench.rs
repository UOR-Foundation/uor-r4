use std::time::Instant;

use uor_r4_cpu_train::{CpuDiscreteRoute, VOCABULARY};
use uor_r4_discrete::normalization_mechanism::orbit_root_coords;

const BLOCKS: usize = 4;
const CONTEXT: usize = 192;
const PREFIX: usize = 64;
const TIMED_ROWS: usize = 1_024;

fn main() {
    let token_blocks = (0..VOCABULARY)
        .map(|token| {
            (0..BLOCKS)
                .map(|block| orbit_root_coords(((token + block * 7) & 15) as u8).unwrap())
                .collect()
        })
        .collect();
    let stages = expected_stages(CONTEXT);
    let routes = (0..BLOCKS)
        .map(|block| {
            (0..stages)
                .map(|stage| ((block * 7 + stage * 3) & 63) as u32)
                .collect()
        })
        .collect();
    let route = CpuDiscreteRoute::from_parts(CONTEXT, token_blocks, routes).unwrap();
    let prefix: Vec<u32> = (0..PREFIX as u32)
        .map(|index| (index * 997 + 17) % VOCABULARY as u32)
        .collect();
    for _ in 0..4 {
        route.read(&prefix).unwrap();
    }
    let started = Instant::now();
    let mut checksum = 0u64;
    for _ in 0..TIMED_ROWS {
        let query = route.read(&prefix).unwrap();
        for value in query {
            checksum = checksum.rotate_left(7).wrapping_add(value as i64 as u64);
        }
    }
    let seconds = started.elapsed().as_secs_f64();
    println!(
        "schema=uor-r4.cpu-discrete-route-bench/1 vocab={VOCABULARY} blocks={BLOCKS} stages={stages} prefix={PREFIX} timed_rows={TIMED_ROWS} elapsed_ms={:.3} rows_per_second={:.3} checksum={checksum:016x} peak_rss_kib={}",
        seconds * 1_000.0,
        TIMED_ROWS as f64 / seconds,
        peak_rss_kib()
    );
}

fn expected_stages(context: usize) -> usize {
    let mut stages = 1usize;
    let mut span = 3usize;
    while span <= context {
        stages += 1;
        span *= 3;
    }
    stages
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
