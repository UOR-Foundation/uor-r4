use std::hint::black_box;
use std::time::Instant;

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use uor_r4_cpu_train::{CpuWriterCore, BLOCK_WIDTH, READ_WIDTH, TRUNK_BLOCKS, VOCABULARY};
use uor_r4_discrete::normalization_mechanism::orbit_root_coords;

const WARMUP_ROWS: usize = 128;
const TIMED_ROWS: usize = 32_768;

fn roots() -> Vec<[u8; TRUNK_BLOCKS]> {
    (0..VOCABULARY)
        .map(|token| {
            [
                (token & 15) as u8,
                ((token >> 4) & 15) as u8,
                ((token >> 8) & 15) as u8,
                (((token >> 4) + (token >> 8)) & 15) as u8,
            ]
        })
        .collect()
}

fn query(row: usize) -> [i32; READ_WIDTH] {
    let rows = roots();
    let tokens = [
        row % VOCABULARY,
        (row * 17 + 3) % VOCABULARY,
        (row * 31 + 7) % VOCABULARY,
        (row * 47 + 11) % VOCABULARY,
    ];
    let mut query = [0i32; READ_WIDTH];
    for (block, token) in tokens.into_iter().enumerate() {
        let root = rows[token][block];
        query[block * BLOCK_WIDTH..(block + 1) * BLOCK_WIDTH]
            .copy_from_slice(&orbit_root_coords(root).expect("canonical root"));
    }
    query
}

fn main() {
    let core = CpuWriterCore::from_roots(&roots(), 8).expect("writer core");
    let queries: Vec<[i32; READ_WIDTH]> = (0..WARMUP_ROWS + TIMED_ROWS).map(query).collect();

    for query in queries.iter().take(WARMUP_ROWS) {
        black_box(core.decide(black_box(query)));
    }

    for threads in [1usize, 8, 16, 32] {
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("thread pool");
        let started = Instant::now();
        let checksum = pool.install(|| {
            queries
                .par_iter()
                .skip(WARMUP_ROWS)
                .map(|query| {
                    let decision = core.decide(black_box(query));
                    decision.top8.iter().fold(0u64, |sum, &token| {
                        sum.wrapping_mul(131).wrapping_add(u64::from(token))
                    })
                })
                .reduce(|| 0, u64::wrapping_add)
        });
        let seconds = started.elapsed().as_secs_f64();
        println!(
            "schema=uor-r4.cpu-writer-bench/1 threads={threads} timed_rows={TIMED_ROWS} elapsed_ms={:.3} rows_per_second={:.3} checksum={checksum}",
            seconds * 1_000.0,
            TIMED_ROWS as f64 / seconds
        );
    }
}
