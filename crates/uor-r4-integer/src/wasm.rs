//! WebAssembly Serving Runtime and C-FFI Surface for UOR-R4 Integer Serving.
//!
//! Provides bare C-ABI exports callable from WebAssembly runtimes,
//! browser JavaScript (Pages Studio), and embedded targets.
//! All operations strictly satisfy D11: zero floating-point, zero hardware multipliers,
//! and zero hardware dividers. Strictly 100% safe Rust (`#![forbid(unsafe_code)]`).

use crate::math::{atan2_q30, UnitS3Q30};
use crate::model::{galois_lfsr_step, score_token_salience, KEY_DIM};
use crate::sampling::exact_min_p_threshold;

/// API version identifier: 0x00010000 = v1.0.0
#[inline(never)]
pub extern "C" fn uor_wasm_version() -> u32 {
    0x00010000
}

/// Native width dimension: 576
#[inline(never)]
pub extern "C" fn uor_wasm_state_dim() -> u32 {
    576
}

/// Vocabulary size: 4096
#[inline(never)]
pub extern "C" fn uor_wasm_vocab_size() -> u32 {
    4096
}

/// Active context capacity: 256
#[inline(never)]
pub extern "C" fn uor_wasm_context_capacity() -> u32 {
    256
}

/// Exact integer shift-add Min-P threshold computation (0 hardware multipliers).
#[inline(never)]
pub extern "C" fn uor_wasm_exact_min_p_threshold(p_max: u64, min_p_q16: u32) -> u64 {
    exact_min_p_threshold(p_max, min_p_q16)
}

/// Discrete Hopf fibration S3 -> S2 projection base coordinate X in Q30 (0 float).
#[inline(never)]
pub extern "C" fn uor_wasm_hopf_project_x(q0: i32, q1: i32, q2: i32, q3: i32) -> i32 {
    let u = UnitS3Q30::from_i32_coords(&[q0, q1, q2, q3]);
    let [x, _, _] = u.hopf_project();
    x
}

/// Discrete Hopf fibration S3 -> S2 projection base coordinate Y in Q30 (0 float).
#[inline(never)]
pub extern "C" fn uor_wasm_hopf_project_y(q0: i32, q1: i32, q2: i32, q3: i32) -> i32 {
    let u = UnitS3Q30::from_i32_coords(&[q0, q1, q2, q3]);
    let [_, y, _] = u.hopf_project();
    y
}

/// Discrete Hopf fibration S3 -> S2 projection base coordinate Z in Q30 (0 float).
#[inline(never)]
pub extern "C" fn uor_wasm_hopf_project_z(q0: i32, q1: i32, q2: i32, q3: i32) -> i32 {
    let u = UnitS3Q30::from_i32_coords(&[q0, q1, q2, q3]);
    let [_, _, z] = u.hopf_project();
    z
}

/// Scalar Riemann zeta-zero phase step modulo 2^31 (0 float vectorization, 0 hardware mul).
#[inline(never)]
pub extern "C" fn uor_wasm_step_zeta_phase_scalar(
    phase: i32,
    freq_val: i32,
    token: u32,
    j: u32,
) -> i32 {
    let freq = freq_val as i64;
    let k = 1i64 + (((token as i64) + (j as i64)) & 0x07);

    // 4-bit unrolled shift-add multiplication: k in [1, 8]
    let mut prod = 0i64;
    if k & 1 != 0 {
        prod += freq;
    }
    if k & 2 != 0 {
        prod += freq << 1;
    }
    if k & 4 != 0 {
        prod += freq << 2;
    }
    if k & 8 != 0 {
        prod += freq << 3;
    }

    let next_phase = (phase as i64) + prod;
    let wrapped = ((next_phase % (1i64 << 31)) + (1i64 << 31)) % (1i64 << 31);
    wrapped as i32
}

/// Galois LFSR state update for deterministic coordinate scrambling (0 mul).
#[inline(never)]
pub extern "C" fn uor_wasm_galois_lfsr_step(state: u64) -> u64 {
    galois_lfsr_step(state)
}

/// Prime signature computation for hierarchical turn memory (0 mul).
#[inline(never)]
pub extern "C" fn uor_wasm_compute_turn_prime_signature(
    turn_id: u32,
    token: u32,
    start_seq: u64,
    end_seq: u64,
) -> u64 {
    let tokens = [token];
    let slots = [0usize];
    crate::model::compute_turn_prime_signature(&tokens, &slots, turn_id, start_seq, end_seq)
}

/// CORDIC-style fixed-point Q30 arctangent (0 float).
#[inline(never)]
pub extern "C" fn uor_wasm_atan2_q30(y: i32, x: i32) -> i32 {
    atan2_q30(y, x)
}

/// Token salience scoring with zero hardware multipliers.
#[inline(never)]
pub extern "C" fn uor_wasm_score_token_salience_default_key(token: u32) -> i32 {
    let key = [0i32; KEY_DIM];
    score_token_salience(token, &key)
}
