use super::compiler::SIG_BYTES;

#[inline(always)]
pub fn hamming_distance_36_scalar(a: &[u8; SIG_BYTES], b: &[u8; SIG_BYTES]) -> u32 {
    let mut dist = 0u32;
    for (chunk_a, chunk_b) in a.chunks(8).zip(b.chunks(8)) {
        let mut ba = [0u8; 8];
        ba[..chunk_a.len()].copy_from_slice(chunk_a);
        let mut bb = [0u8; 8];
        bb[..chunk_b.len()].copy_from_slice(chunk_b);
        dist += (u64::from_le_bytes(ba) ^ u64::from_le_bytes(bb)).count_ones();
    }
    dist
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn hamming_distance_36_avx2(a: &[u8; SIG_BYTES], b: &[u8; SIG_BYTES]) -> u32 {
    use std::arch::x86_64::*;

    // Load first 32 bytes
    let va = _mm256_loadu_si256(a.as_ptr() as *const __m256i);
    let vb = _mm256_loadu_si256(b.as_ptr() as *const __m256i);
    let vxor = _mm256_xor_si256(va, vb);

    // popcount lookup table for lower nibble
    let lookup = _mm256_setr_epi8(
        0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3,
        3, 4,
    );
    let low_mask = _mm256_set1_epi8(0x0f);

    let lo = _mm256_and_si256(vxor, low_mask);
    let hi = _mm256_and_si256(_mm256_srli_epi16(vxor, 4), low_mask);

    let pop_lo = _mm256_shuffle_epi8(lookup, lo);
    let pop_hi = _mm256_shuffle_epi8(lookup, hi);

    let pop = _mm256_add_epi8(pop_lo, pop_hi);

    // Sum absolute differences against 0 to sum bytes in 64-bit blocks
    let zero = _mm256_setzero_si256();
    let sum_64 = _mm256_sad_epu8(pop, zero);

    // sum_64 has 4 64-bit values containing sums. Extract them.
    let mut sum = [0u64; 4];
    _mm256_storeu_si256(sum.as_mut_ptr() as *mut __m256i, sum_64);
    let simd_pop = (sum[0] + sum[1] + sum[2] + sum[3]) as u32;

    // Add the remaining 4 bytes
    let rem_a = u32::from_le_bytes(a[32..36].try_into().unwrap());
    let rem_b = u32::from_le_bytes(b[32..36].try_into().unwrap());
    let rem_pop = (rem_a ^ rem_b).count_ones();

    simd_pop + rem_pop
}

#[cfg(target_arch = "aarch64")]
unsafe fn hamming_distance_36_neon(a: &[u8; SIG_BYTES], b: &[u8; SIG_BYTES]) -> u32 {
    use std::arch::aarch64::*;

    // 32 bytes = 2x 16-byte vectors
    let va1 = vld1q_u8(a.as_ptr());
    let vb1 = vld1q_u8(b.as_ptr());
    let va2 = vld1q_u8(a.as_ptr().add(16));
    let vb2 = vld1q_u8(b.as_ptr().add(16));

    let vxor1 = veorq_u8(va1, vb1);
    let vxor2 = veorq_u8(va2, vb2);

    let vcnt1 = vcntq_u8(vxor1);
    let vcnt2 = vcntq_u8(vxor2);

    let simd_pop = vaddvq_u8(vcnt1) as u32 + vaddvq_u8(vcnt2) as u32;

    // Add remaining 4 bytes
    let rem_a = u32::from_le_bytes(a[32..36].try_into().unwrap());
    let rem_b = u32::from_le_bytes(b[32..36].try_into().unwrap());
    let rem_pop = (rem_a ^ rem_b).count_ones();

    simd_pop + rem_pop
}

/// Dispatches to the fastest available Hamming distance implementation for 36-byte arrays.
#[inline(always)]
pub fn hamming_distance_36(a: &[u8; SIG_BYTES], b: &[u8; SIG_BYTES]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            return unsafe { hamming_distance_36_avx2(a, b) };
        }
        hamming_distance_36_scalar(a, b)
    }

    #[cfg(target_arch = "aarch64")]
    {
        // NEON is baseline for aarch64 in Rust
        unsafe { hamming_distance_36_neon(a, b) }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        hamming_distance_36_scalar(a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_simd_hamming_equivalence(
            a in proptest::collection::vec(any::<u8>(), SIG_BYTES).prop_map(|v| {
                let mut arr = [0u8; SIG_BYTES];
                arr.copy_from_slice(&v);
                arr
            }),
            b in proptest::collection::vec(any::<u8>(), SIG_BYTES).prop_map(|v| {
                let mut arr = [0u8; SIG_BYTES];
                arr.copy_from_slice(&v);
                arr
            })
        ) {
            let scalar = hamming_distance_36_scalar(&a, &b);
            let dispatched = hamming_distance_36(&a, &b);

            #[cfg(target_arch = "x86_64")]
            if std::arch::is_x86_feature_detected!("avx2") {
                let avx2 = unsafe { hamming_distance_36_avx2(&a, &b) };
                assert_eq!(scalar, avx2, "AVX2 implementation differs from scalar");
            }

            #[cfg(target_arch = "aarch64")]
            {
                let neon = unsafe { hamming_distance_36_neon(&a, &b) };
                assert_eq!(scalar, neon, "NEON implementation differs from scalar");
            }

            assert_eq!(scalar, dispatched, "Dispatched implementation differs from scalar");
        }
    }
}
