//! Deterministic corpora for the measurement harness. Everything here is
//! reproducible from the source alone; file-backed corpora are bound by SHA-256
//! in the receipt. No randomness is drawn from the operating system.

#[derive(Debug, Clone)]
pub struct Corpus {
    pub name: String,
    pub source: String,
    pub bytes: Vec<u8>,
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub fn random_bytes(n: usize, seed: u64) -> Vec<u8> {
    let mut st = seed;
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        out.extend_from_slice(&splitmix64(&mut st).to_be_bytes());
    }
    out.truncate(n);
    out
}

fn cycle(pattern: &[u8], n: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let take = pattern.len().min(n - out.len());
        out.extend_from_slice(&pattern[..take]);
    }
    out
}

fn alphabet_bytes(n: usize, alphabet: &[u8], seed: u64) -> Vec<u8> {
    let mut st = seed;
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let r = splitmix64(&mut st);
        for k in 0..8 {
            if out.len() == n {
                break;
            }
            out.push(alphabet[((r >> (8 * k)) & 0xff) as usize % alphabet.len()]);
        }
    }
    out
}

fn structured_records(n: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(n + 256);
    let mut i: u64 = 0;
    while out.len() < n {
        let rec = format!(
            "{{\"id\":{i},\"kind\":\"fbs\",\"state\":\"pinned\",\"level\":{},\"note\":\"ordered ternary reduction\"}}\n",
            i % 7
        );
        out.extend_from_slice(rec.as_bytes());
        i += 1;
    }
    out.truncate(n);
    out
}

/// Built-in corpora: synthetic extremes plus order-0-compressible controls.
pub fn builtin_corpora(size: usize) -> Vec<Corpus> {
    vec![
        Corpus {
            name: "uniform_random".into(),
            source: "splitmix64 seed=0x5EED".into(),
            bytes: random_bytes(size, 0x5EED),
        },
        Corpus {
            name: "zeros".into(),
            source: "all zero bytes".into(),
            bytes: vec![0u8; size],
        },
        Corpus {
            name: "cycle_period_82".into(),
            source: "82-byte pseudo-random pattern repeated (atom-aligned)".into(),
            bytes: cycle(&random_bytes(82, 0xA11A), size),
        },
        Corpus {
            name: "cycle_period_96".into(),
            source: "96-byte pseudo-random pattern repeated (not atom-aligned)".into(),
            bytes: cycle(&random_bytes(96, 0xB0B0), size),
        },
        Corpus {
            name: "cycle_period_1".into(),
            source: "single repeated byte 0x41".into(),
            bytes: vec![0x41u8; size],
        },
        Corpus {
            name: "structured_records".into(),
            source: "repeated JSON-ish records with a varying counter".into(),
            bytes: structured_records(size),
        },
        Corpus {
            name: "low_entropy_4symbols".into(),
            source: "pseudo-random draws over a 4-symbol alphabet".into(),
            bytes: alphabet_bytes(size, b"ACGT", 0xC0FFEE),
        },
        Corpus {
            name: "base64_alphabet".into(),
            source: "pseudo-random draws over a 64-symbol alphabet".into(),
            bytes: alphabet_bytes(
                size,
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
                0xFACE,
            ),
        },
        Corpus {
            name: "all_byte_values".into(),
            source: "0..=255 repeated (period 256)".into(),
            bytes: cycle(&(0u8..=255).collect::<Vec<u8>>(), size),
        },
    ]
}

/// Order-0 empirical entropy in bits per byte, and the implied order-0 code
/// length in bits. Reporting statistics only: the codec itself is exact integer
/// and byte arithmetic.
pub fn order0_entropy_bits(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut hist = [0u64; 256];
    for b in data {
        hist[*b as usize] += 1;
    }
    let n = data.len() as f64;
    let mut bits = 0.0f64;
    for c in hist {
        if c > 0 {
            let p = c as f64 / n;
            bits -= c as f64 * p.log2();
        }
    }
    bits
}
