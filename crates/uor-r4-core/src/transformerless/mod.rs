//! transformerless — cross-compilation of a transformer LM into a
//! multiplication-free, table-native, certifiable inference artifact.
//!
//! The library holds the pieces both sides share:
//!
//! - [`OpKernel`]: the COMPLETE arithmetic interface of the runtime. Its
//!   method set — add, sub, shift, xor, compare, table read — is enumerated
//!   here and contains no multiply. Every arithmetic operation the runtime
//!   performs goes through this kernel and increments its census, so the
//!   multiplication-free claim is by construction (no multiply exists in the
//!   interface) and by measurement (the census is printed per run).
//! - the derived popcount table: Hamming distance between bit signatures is
//!   XOR then table reads then adds — the stratum observable of the byte
//!   plane, repurposed as the runtime's only metric arithmetic.
//! - bit signatures: "a vector at each bit" — bit b of a signature records
//!   which side of threshold b the content falls on; a prefix of bits is an
//!   intersection of regions; prefix depth is resolution.
//!
//! Multiplication is confined to the COMPILER (offline, once, κ-pinned
//! outputs) and to the CERTIFIER (instrumentation). See docs/PROOF.md.

#[cfg(not(target_arch = "wasm32"))]
pub mod teacher;

pub use reference_state::{ActiveFrontier, ActiveFrontierEntry, PackedEdgeRanges};
pub use runtime::{derive_popcount_table, hamming, sign_signature, OpKernel};
pub use score_q::ScoreQ;
#[cfg(not(target_arch = "wasm32"))]
pub use certificate::Certificate;

#[cfg(test)]
mod witnesses {
    use super::runtime::{derive_popcount_table, hamming, sign_signature, OpKernel};

    /// P-1: the popcount table matches its definition on all 256 bytes and
    /// carries the stratum partition sizes C(8,k).
    #[test]
    fn p1_popcount_table() {
        let t = derive_popcount_table();
        let mut sizes = [0usize; 9];
        for x in 0..=255u8 {
            assert_eq!(t[x as usize], x.count_ones() as u8);
            sizes[t[x as usize] as usize] += 1;
        }
        assert_eq!(sizes, [1, 8, 28, 56, 70, 56, 28, 8, 1]);
    }

    /// P-2: kernel Hamming equals the direct definition on random pairs,
    /// and the census records only kernel ops.
    #[test]
    fn p2_hamming_exact() {
        let pop = derive_popcount_table();
        let mut s = 0x1234u64;
        let mut rng = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s & 0xff) as u8
        };
        for _ in 0..64 {
            let a: Vec<u8> = (0..36).map(|_| rng()).collect();
            let b: Vec<u8> = (0..36).map(|_| rng()).collect();
            let direct: u32 = a.iter().zip(&b).map(|(x, y)| (x ^ y).count_ones()).sum();
            let mut k = OpKernel::default();
            assert_eq!(hamming(&mut k, &pop, &a, &b), direct as i64);
            assert_eq!(k.xors, 36);
            assert_eq!(k.table_reads, 36);
            assert_eq!(k.adds, 36);
        }
    }

    /// P-4: the runtime module's source contains no multiplication,
    /// division, or modulo operator on values. Doc lines and comments are
    /// stripped; dereference `*x` (star not preceded by an operand) is not
    /// an arithmetic operator and does not match. This makes the
    /// "no source-level mul/div/mod in the runtime" claim machine-checked
    /// on every test run rather than a review assertion.
    #[test]
    fn p4_runtime_source_scan() {
        let src = include_str!("runtime.rs");
        let mut offenders = Vec::new();
        for (ln, line) in src.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            let b = code.as_bytes();
            for (i, &ch) in b.iter().enumerate() {
                if ch != b'*' && ch != b'/' && ch != b'%' {
                    continue;
                }
                if ch == b'/'
                    && ((i + 1 < b.len() && b[i + 1] == b'/') || (i >= 1 && b[i - 1] == b'/'))
                {
                    continue; // comment slashes
                }
                let prev = if i >= 2 && b[i - 1] == b' ' {
                    b[i - 2]
                } else if i >= 1 {
                    b[i - 1]
                } else {
                    b' '
                };
                let next = if i + 2 < b.len() && b[i + 1] == b' ' {
                    b[i + 2]
                } else if i + 1 < b.len() {
                    b[i + 1]
                } else {
                    b' '
                };
                let operand_l =
                    |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b')' || c == b']';
                let operand_r = |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b'(';
                if operand_l(prev) && operand_r(next) {
                    offenders.push(format!("line {}: {}", ln + 1, code));
                    break;
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "value arithmetic in runtime.rs:\n{}",
            offenders.join("\n")
        );
    }

    /// P-3: sign signatures agree with the direct definition, bit for bit.
    #[test]
    fn p3_sign_signature() {
        let vals: Vec<i64> = (0..288).map(|i| (i as i64 * 7919) % 1000 - 500).collect();
        let thr: Vec<i64> = (0..288).map(|i| (i as i64 * 104729) % 1000 - 500).collect();
        let mut k = OpKernel::default();
        let sig = sign_signature(&mut k, &vals, &thr);
        for i in 0..288 {
            let want = vals[i] > thr[i];
            let got = sig[i / 8] >> (i % 8) & 1 == 1;
            assert_eq!(want, got, "bit {}", i);
        }
        assert_eq!(k.compares, 288);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod anti_degeneracy;
#[cfg(not(target_arch = "wasm32"))]
pub mod certify;
#[cfg(not(target_arch = "wasm32"))]
pub mod certificate;
#[cfg(not(target_arch = "wasm32"))]
pub mod command;
#[cfg(not(target_arch = "wasm32"))]
pub mod compare;
#[cfg(not(target_arch = "wasm32"))]
pub mod compiler;
#[cfg(not(target_arch = "wasm32"))]
pub mod fairness_provenance;
#[cfg(not(target_arch = "wasm32"))]
pub mod graph_patch;
#[cfg(not(target_arch = "wasm32"))]
pub mod progress;
#[cfg(not(target_arch = "wasm32"))]
pub mod performance_certificate;
#[cfg(not(target_arch = "wasm32"))]
pub mod predictive_sufficiency;
pub mod reference_state;
pub mod resolution_status;
pub mod runtime;
pub mod score_q;
#[cfg(not(target_arch = "wasm32"))]
pub mod scenarios;
#[cfg(not(target_arch = "wasm32"))]
pub mod shortlist_evaluator;
#[cfg(not(target_arch = "wasm32"))]
pub mod transitions;
