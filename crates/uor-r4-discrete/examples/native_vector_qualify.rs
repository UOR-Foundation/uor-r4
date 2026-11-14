//! Finite RTX 4090 qualification for the native Q31 Hamilton vector operator.
//! Run with:
//! `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 cargo run -p
//! uor-r4-discrete --features cuda --example native_vector_qualify`

use std::time::Instant;
use uor_r4_discrete::native_vector::q31_hamilton_rotate;
use uor_r4_discrete::native_vector_device::{
    compiled_native_vector_image, native_vector_entry, NativeVectorEngine,
};

const ITEMS: usize = 4096;

fn values(seed: u64) -> Vec<[i64; 4]> {
    let mut out = vec![
        [i64::from(i32::MIN); 4],
        [1 << 31, -(1 << 31), 1, -1],
        [1 << 30, 1 << 29, -(1 << 28), 7],
        [0, 1 << 31, 0, -(1 << 31)],
    ];
    let mut seed = seed;
    while out.len() < ITEMS {
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed as i32
        };
        out.push([
            i64::from(next()),
            i64::from(next()),
            i64::from(next()),
            i64::from(next()),
        ]);
    }
    out
}

fn main() -> uor_r4_discrete::Result<()> {
    let engine = NativeVectorEngine::new(0)?;
    let gates = values(0x6a09_e667_f3bc_c909);
    let held = values(0xbb67_ae85_84ca_a73b);
    let started = Instant::now();
    let cuda = engine.hamilton_rotate(&gates, &held)?;
    let elapsed_ns = started.elapsed().as_nanos();
    for (index, ((gate, value), observed)) in gates.iter().zip(&held).zip(&cuda).enumerate() {
        let expected = q31_hamilton_rotate(*gate, *value)?;
        if *observed != expected {
            return Err(uor_r4_discrete::Error::Invalid(format!(
                "Q31 parity differs at item {index}: CUDA {observed:?}, Rust {expected:?}"
            )));
        }
    }
    println!(
        "{}",
        serde_json::json!({
            "schema": "uor-r4.native-vector-qualification/1",
            "operator": native_vector_entry(),
            "state": "q31:i64x4",
            "items": ITEMS,
            "raw_words_compared": ITEMS * 4,
            "bit_identical": true,
            "wall_ns_including_transfer": elapsed_ns,
            "image_source_ptx_cubin_report_sha256": compiled_native_vector_image(),
            "first": cuda.first(),
            "last": cuda.last(),
        })
    );
    Ok(())
}
