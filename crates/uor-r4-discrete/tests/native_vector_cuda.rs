#![cfg(feature = "cuda")]

use uor_r4_discrete::native_vector::q31_hamilton_rotate;
use uor_r4_discrete::native_vector_device::NativeVectorEngine;

fn values(seed: u64, count: usize) -> Vec<[i64; 4]> {
    let mut out = vec![
        [i64::from(i32::MIN); 4],
        [1 << 31, -(1 << 31), 1, -1],
        [1 << 30, 1 << 29, -(1 << 28), 7],
        [0, 1 << 31, 0, -(1 << 31)],
    ];
    let mut seed = seed;
    out.extend((0..count.saturating_sub(out.len())).map(|_| {
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed as i32
        };
        [
            i64::from(next()),
            i64::from(next()),
            i64::from(next()),
            i64::from(next()),
        ]
    }));
    out
}

#[test]
fn cuda_and_rust_return_identical_q31_transport_words() -> uor_r4_discrete::Result<()> {
    let engine = match NativeVectorEngine::new(0) {
        Ok(engine) => engine,
        Err(error) if std::env::var("UOR_REQUIRE_CUDA").as_deref() != Ok("1") => {
            println!("SKIPPED (no CUDA device): {error}");
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    let valid = [[0, 1 << 31, 0, 0]];
    for invalid in [[[(1 << 31) + 1, 0, 0, 0]], [[-((1 << 31) + 1), 0, 0, 0]]] {
        assert!(engine.hamilton_rotate(&invalid, &valid).is_err());
        assert!(engine.hamilton_rotate(&valid, &invalid).is_err());
    }
    let gates = values(0x6a09_e667_f3bc_c909, 4096);
    let held = values(0xbb67_ae85_84ca_a73b, 4096);
    let cuda = engine.hamilton_rotate(&gates, &held)?;
    for i in 0..gates.len() {
        assert_eq!(
            cuda[i],
            q31_hamilton_rotate(gates[i], held[i])?,
            "native Q31 Hamilton parity differs at item {i}"
        );
    }
    assert_eq!(
        NativeVectorEngine::hamilton_rotate_reference(gates[0], held[0])?,
        cuda[0]
    );
    let unit = [[1 << 31, 0, 0, 0]];
    let half = [[1 << 30, 0, 0, 0]];
    let unit_cuda = engine.hamilton_rotate(&unit, &half)?;
    let planted_shift = ((u64::from(1_u32 << 30)) << 31) as i64 >> 30;
    assert_ne!(
        unit_cuda[0][0], planted_shift,
        "a planted Q30/Q31 shift change was not detected"
    );
    let one = 1 << 31;
    let axes = engine.hamilton_rotate(
        &[[0, one, 0, 0], [0, 0, one, 0]],
        &[[0, 0, one, 0], [0, one, 0, 0]],
    )?;
    assert_eq!(axes, [[0, 0, 0, one], [0, 0, 0, -one]]);
    Ok(())
}

#[test]
fn a_one_bit_scale_change_is_detected() {
    let gate = [1 << 31, 0, 0, 0];
    let value = [1 << 30, 0, 0, 0];
    let expected = q31_hamilton_rotate(gate, value).unwrap();
    let wrong_shift = ((value[0].unsigned_abs() << 31) as i64) >> 30;
    assert_ne!(expected[0], wrong_shift);
}
