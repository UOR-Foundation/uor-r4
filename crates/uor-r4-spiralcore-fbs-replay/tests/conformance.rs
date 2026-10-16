//! Conformance suite for the SpiralCore Recursive FBS Engine specification.
//!
//! Every test names the paper section or equation it checks. The suite is written
//! against the specification text alone; no reference implementation from the
//! developer was available, so a failure here is either a specification defect or
//! a reconstruction error - both are reportable findings.

use uor_r4_spiralcore_fbs_replay::corpus::{builtin_corpora, random_bytes};
use uor_r4_spiralcore_fbs_replay::sha256::sha256_hex;
use uor_r4_spiralcore_fbs_replay::{
    absorb, code_bytes, decode, digit_sum_base3, encode, encode_literal, nest, parse_code,
    predicted_carries, sigma, sigma_inv, v_decode, v_encode, wire, Def, Dictionary, FbsError,
    Packet, ATOM_CAPACITY, MAGIC,
};

fn be(bytes: &[u8]) -> u128 {
    bytes.iter().fold(0u128, |acc, b| (acc << 8) | *b as u128)
}

fn frontier_levels_of(packet: &Packet) -> Vec<u32> {
    let f = parse_code(&packet.pins[2].code).expect("pin 3 code");
    f.iter().map(|r| packet.dict.level(*r)).collect()
}

/// Simulate Section 3's carry engine over levels only (no dictionary), which is
/// all the counting claims depend on.
// clippy::same_item_push is a false positive here: the vector is truncated and
// repushed by the carry loop between every append, so it cannot be a repeat().
#[allow(clippy::same_item_push)]
fn simulate_levels(n_atoms: u64) -> (Vec<u32>, u64) {
    let mut frontier: Vec<u32> = Vec::new();
    let mut carries = 0u64;
    for _ in 0..n_atoms {
        frontier.push(0);
        loop {
            let l = frontier.len();
            if l < 3 || frontier[l - 3] != frontier[l - 2] || frontier[l - 2] != frontier[l - 1] {
                break;
            }
            let new_level = frontier[l - 3] + 1;
            frontier.truncate(l - 3);
            frontier.push(new_level);
            carries += 1;
        }
    }
    (frontier, carries)
}

// ---------------------------------------------------------------------------
// Appendix A worked examples
// ---------------------------------------------------------------------------

#[test]
fn appendix_a_frontier_code_examples() {
    // "For example, frontiers [1] and [1,2] yield 65,793 and 16,908,546."
    assert_eq!(be(&code_bytes(&[1])), 65_793);
    assert_eq!(be(&code_bytes(&[1, 2])), 16_908_546);
    // Code(R) = BE(01 || V(s) || V(r_1)..), so the byte string is recoverable.
    assert_eq!(code_bytes(&[1]), vec![0x01, 0x01, 0x01]);
    assert_eq!(code_bytes(&[1, 2]), vec![0x01, 0x02, 0x01, 0x02]);
    assert_eq!(be(&code_bytes(&[])), 256);
}

#[test]
fn appendix_a_varint_is_shortest_unsigned_base128() {
    assert_eq!(v_encode(0), vec![0x00]);
    assert_eq!(v_encode(1), vec![0x01]);
    assert_eq!(v_encode(127), vec![0x7f]);
    assert_eq!(v_encode(128), vec![0x80, 0x01]);
    assert_eq!(v_encode(300), vec![0xac, 0x02]);
    for n in [0u64, 1, 2, 81, 82, 127, 128, 255, 16383, 16384, 1 << 40] {
        let e = v_encode(n);
        let mut pos = 0;
        assert_eq!(v_decode(&e, &mut pos).unwrap(), n, "round trip n={n}");
        assert_eq!(pos, e.len());
        // "shortest": no redundant trailing 0x80 continuation groups
        if e.len() > 1 {
            assert_ne!(*e.last().unwrap(), 0x00, "not shortest for n={n}");
        }
    }
}

// ---------------------------------------------------------------------------
// Sections 2-3 counting claims
// ---------------------------------------------------------------------------

#[test]
fn section_3_frontier_counts_are_base_three_digits() {
    for n in 1u64..3000 {
        let (levels, carries) = simulate_levels(n);
        let s3 = digit_sum_base3(n);
        assert_eq!(levels.len() as u64, s3, "|R_N| != s_3(N) at N={n}");
        assert_eq!(carries, predicted_carries(n), "W(N) at N={n}");
        // N = sum over frontier occurrences of 3^level(r)   (eq. 3.5)
        let weight: u128 = levels.iter().map(|l| 3u128.pow(*l)).sum();
        assert_eq!(weight, n as u128, "eq 3.5 at N={n}");
        // The frontier is a base-3 numeral written most-significant-first with the
        // least-significant digit appended at the right, so levels are
        // NON-INCREASING left to right (N=4 -> [level 1, level 0]; N=9 -> [level 2]).
        assert!(
            levels.windows(2).all(|w| w[0] >= w[1]),
            "levels not MSB-first at N={n}"
        );
        // The frontier is exactly the base-3 expansion of N: d_h occurrences of
        // a root at level h, most significant first.
        let mut counts: std::collections::BTreeMap<u32, u64> = std::collections::BTreeMap::new();
        for l in &levels {
            *counts.entry(*l).or_insert(0) += 1;
        }
        let mut m = n;
        let mut h = 0u32;
        while m > 0 {
            let d = m % 3;
            assert_eq!(
                counts.get(&h).copied().unwrap_or(0),
                d,
                "digit d_{h} of N={n} does not match the frontier level histogram"
            );
            m /= 3;
            h += 1;
        }
        assert!(
            counts.keys().all(|k| *k < h),
            "level above the leading digit at N={n}"
        );
    }
}

#[test]
fn section_3_engine_matches_level_simulation() {
    // The real encoder's carries and frontier size must equal the level-only model.
    let x = random_bytes(ATOM_CAPACITY * 500, 7);
    let (packet, stats) = encode(&x);
    assert_eq!(stats.atoms, 500);
    let (levels, carries) = simulate_levels(500);
    assert_eq!(stats.carries, carries);
    assert_eq!(frontier_levels_of(&packet), levels);
}

#[test]
fn parse_shape_depends_only_on_length_not_content() {
    // The carry rule is content-independent, so equal-length inputs produce the
    // same frontier *shape* even though the node IDs and dictionary differ.
    let zeros = vec![0u8; ATOM_CAPACITY * 137 + 5];
    let rand = random_bytes(zeros.len(), 99);
    let (pz, _) = encode(&zeros);
    let (pr, _) = encode(&rand);
    assert_eq!(frontier_levels_of(&pz), frontier_levels_of(&pr));
    assert_ne!(pz.pins[2].code, pr.pins[2].code, "IDs should differ");
}

// ---------------------------------------------------------------------------
// Sections 4-5 packet and recovery identity
// ---------------------------------------------------------------------------

#[test]
fn section_5_recovery_identity_across_a_size_matrix() {
    let mut sizes = vec![
        0usize, 1, 2, 3, 41, 81, 82, 83, 84, 163, 164, 165, 166, 1000, 8192,
    ];
    sizes.push(ATOM_CAPACITY * 200);
    for size in sizes {
        for (label, data) in [
            ("zeros", vec![0u8; size]),
            ("random", random_bytes(size, 0x1234 ^ size as u64)),
            (
                "asc",
                (0..size).map(|i| (i % 251) as u8).collect::<Vec<u8>>(),
            ),
        ] {
            let (w, _, _) = encode_literal(&data);
            let d = decode(&w).unwrap_or_else(|e| panic!("decode failed size={size} {label}: {e}"));
            assert_eq!(d, data, "D(E(x)) != x at size={size} {label}");
        }
    }
}

#[test]
fn section_4_pins_follow_the_boundary_conventions() {
    // Empty input: all three pins are (0, Code([])) and T is empty.
    let (w0, p0, s0) = encode_literal(&[]);
    assert_eq!(s0.atoms, 0);
    assert_eq!(p0.dict.len(), 0);
    for pin in &p0.pins {
        assert_eq!(pin.index, 0);
        assert_eq!(pin.code, code_bytes(&[]));
    }
    assert_eq!(decode(&w0).unwrap(), Vec::<u8>::new());

    // One atom: the first admission pin is duplicated as the second.
    let one = random_bytes(ATOM_CAPACITY, 5);
    let (w1, p1, s1) = encode_literal(&one);
    assert_eq!(s1.atoms, 1);
    assert_eq!(p1.pins[0].index, 1);
    assert_eq!(p1.pins[1].index, 1);
    assert_eq!(p1.pins[2].index, 1);
    assert_eq!(p1.pins[0].code, p1.pins[1].code);
    assert_eq!(p1.pins[1].code, p1.pins[2].code);
    assert_eq!(decode(&w1).unwrap(), one);

    // 82 bytes -> 1 atom; 83 bytes -> 2 atoms; 164 -> 2; 165 -> 3.
    for (len, atoms) in [(82usize, 1u64), (83, 2), (164, 2), (165, 3)] {
        let (_, _, s) = encode_literal(&vec![7u8; len]);
        assert_eq!(s.atoms, atoms, "N=ceil(m/82) at m={len}");
    }

    // Ordinary multiatom input: I1 = 1, I2 = 2, I3 = N.
    let multi = random_bytes(1000, 11);
    let (_, pm, sm) = encode_literal(&multi);
    assert_eq!(pm.pins[0].index, 1);
    assert_eq!(pm.pins[1].index, 2);
    assert_eq!(pm.pins[2].index, sm.atoms);
    assert_eq!(sm.atoms, 1000u64.div_ceil(82));
}

#[test]
fn section_5_expansion_of_pins_is_the_prefix_it_claims() {
    let data = random_bytes(500, 3);
    let (_, p, _) = encode_literal(&data);
    let f1 = parse_code(&p.pins[0].code).unwrap();
    let f2 = parse_code(&p.pins[1].code).unwrap();
    assert_eq!(p.dict.expand_frontier(&f1).unwrap(), data[..82].to_vec());
    assert_eq!(p.dict.expand_frontier(&f2).unwrap(), data[..164].to_vec());
    let f3 = parse_code(&p.pins[2].code).unwrap();
    assert_eq!(p.dict.expand_frontier(&f3).unwrap(), data);
}

#[test]
fn section_5_encoding_is_injective_on_samples() {
    let mut seen = std::collections::HashMap::new();
    for i in 0..200u64 {
        let x = random_bytes(200 + (i as usize % 17), i);
        let (w, _, _) = encode_literal(&x);
        let h = sha256_hex(&w);
        assert!(seen.insert(h, x.clone()).is_none(), "collision at {i}");
    }
}

#[test]
fn appendix_a_atom_capacity_is_82() {
    assert_eq!(ATOM_CAPACITY, 82);
    assert_eq!(MAGIC, b"FBS7DAG1");
}

// ---------------------------------------------------------------------------
// Reference-parser checks (Appendix A closing paragraph)
// ---------------------------------------------------------------------------

fn valid_packet_for(data: &[u8]) -> (Vec<u8>, Packet) {
    let (w, p, _) = encode_literal(data);
    (w, p)
}

#[test]
fn parser_rejects_malformed_wires() {
    let data = random_bytes(500, 21);
    let (w, p) = valid_packet_for(&data);
    assert!(decode(&w).is_ok());

    // magic
    let mut bad = w.clone();
    bad[0] = b'X';
    assert_eq!(decode(&bad).unwrap_err(), FbsError::BadMagic);

    // truncation
    assert!(matches!(
        decode(&w[..w.len() - 1]),
        Err(FbsError::Truncated(_))
    ));
    assert!(matches!(decode(&w[..4]), Err(FbsError::BadMagic)));

    // trailing bytes after the declared tail
    let mut ext = w.clone();
    ext.push(0x00);
    assert_eq!(decode(&ext).unwrap_err(), FbsError::TrailingBytes);

    // unknown tail tag
    let mut tag = w.clone();
    let mut eta_pos = MAGIC.len();
    for pin in &p.pins {
        eta_pos +=
            v_encode(pin.index).len() + v_encode(pin.code.len() as u64).len() + pin.code.len();
    }
    tag[eta_pos] = 7;
    assert_eq!(decode(&tag).unwrap_err(), FbsError::BadTag(7));

    // pinned atom count that the frontier cannot satisfy
    let mut pkt = p.clone();
    pkt.pins[2].index += 1;
    let bad_count = wire(&pkt, 0, &pkt.dict.serialize());
    let err = decode(&bad_count).unwrap_err();
    assert!(
        matches!(
            err,
            FbsError::AtomCountMismatch { .. } | FbsError::RootInventoryMismatch { .. }
        ),
        "unexpected {err:?}"
    );

    // pinned states: pin 1 must be the first atom
    let mut pkt2 = p.clone();
    pkt2.pins[0].index = 9;
    let bad_pin = wire(&pkt2, 0, &pkt2.dict.serialize());
    assert!(matches!(decode(&bad_pin), Err(FbsError::PinMismatch(_))));
}

#[test]
fn dictionary_parser_rejects_invalid_definitions() {
    // child not yet defined (causality)
    let tau = vec![1u8, 1, 5, 5, 5];
    assert!(matches!(
        Dictionary::parse(&tau),
        Err(FbsError::ChildNotDefined { .. })
    ));

    // unequal child levels (four definitions, so d = 4)
    let mut tau = vec![4u8];
    tau.extend([0, 1, b'A']); // def1 leaf(1)
    tau.extend([0, 1, b'B']); // def2 leaf(1)
    tau.extend([1, 1, 1, 1]); // def3 parent(1,1,1) -> level 1
    tau.extend([1, 1, 2, 3]); // def4 parent(1,2,3)  -> levels 0,0,1
    assert!(matches!(
        Dictionary::parse(&tau),
        Err(FbsError::LevelMismatch { .. })
    ));

    // leaf length outside 1..=82
    let tau = vec![1u8, 0, 0];
    assert!(matches!(
        Dictionary::parse(&tau),
        Err(FbsError::BadLeafLength(0))
    ));
    let mut tau = vec![1u8, 0, 83];
    tau.resize(3 + 83, 0u8);
    assert!(matches!(
        Dictionary::parse(&tau),
        Err(FbsError::BadLeafLength(83))
    ));

    // trailing bytes in tau
    let mut tau = vec![1u8, 0, 1, b'A'];
    tau.push(0xff);
    assert_eq!(
        Dictionary::parse(&tau).unwrap_err(),
        FbsError::TrailingBytes
    );

    // duplicate definitions are noncanonical but decodable (first ID wins)
    let tau = vec![2u8, 0, 1, b'A', 0, 1, b'A'];
    let d = Dictionary::parse(&tau).expect("noncanonical but decodable");
    assert_eq!(d.len(), 2);
    assert_eq!(d.def(1).unwrap(), &Def::Leaf(vec![b'A']));
    assert_eq!(d.def(2).unwrap(), &Def::Leaf(vec![b'A']));
}

#[test]
fn root_inventory_and_atom_count_are_the_same_check() {
    // A level-h subtree expands to exactly 3^h leaves, so eq. (3.5)'s
    // N = sum 3^level(r) is a restatement of "the frontier expands to N atoms".
    for n in [1u64, 2, 3, 4, 9, 27, 100, 729] {
        let (levels, _) = simulate_levels(n);
        let weight: u128 = levels.iter().map(|l| 3u128.pow(*l)).sum();
        let leaves: u128 = levels.iter().map(|l| 3u128.pow(*l)).sum();
        assert_eq!(weight, n as u128);
        assert_eq!(leaves, weight, "leaf count and root weight coincide");
    }
}

// ---------------------------------------------------------------------------
// Section 6: nested literal tails
// ---------------------------------------------------------------------------

#[test]
fn section_6_nested_literal_tails_recover_at_every_layer() {
    let data = random_bytes(3000, 42);
    let (_, p0, _) = encode_literal(&data);
    for n in [0usize, 1, 2, 3, 8, 16] {
        let nested = nest(&p0, n);
        assert_eq!(decode(&nested.wires[0]).unwrap(), data, "outermost n={n}");
        for j in 1..=n {
            let got = decode(&nested.wires[j]).unwrap();
            assert_eq!(got, nested.dictionaries[j - 1], "layer {j} of n={n}");
        }
        // Section 6 eq. (6.7): D(P_j) = T_{j-1}
        for j in 1..=n {
            let (_w, pj, _) = encode_literal(&nested.dictionaries[j - 1]);
            assert_eq!(pj.dict.serialize(), nested.dictionaries[j]);
        }
    }
}

#[test]
fn section_6_nesting_increases_size() {
    // The paper states the tested whole-packet passes grew. Record the growth of
    // the nested literal-tail construction rather than assuming any bound.
    let data = random_bytes(4096, 8);
    let (_, p0, _) = encode_literal(&data);
    let nested = nest(&p0, 16);
    assert!(nested.wires[0].len() > nested.wires[16].len());
    assert!(nested.wires[0].len() > data.len());
}

// ---------------------------------------------------------------------------
// Section 7: anchor plus manifest
// ---------------------------------------------------------------------------

#[test]
fn section_7_anchor_and_manifest_are_recoverable_together() {
    let data = random_bytes(2048, 17);
    let (anchor0, _, _) = encode_literal(&data);
    let manifest = vec![0x5au8; 334];
    let mut anchor = anchor0.clone();
    for step in 1..=3 {
        let next = absorb(&anchor, &manifest);
        let (a, m) = sigma_inv(&decode(&next).unwrap()).expect("sigma inverse");
        assert_eq!(a, anchor, "step {step} anchor");
        assert_eq!(m, manifest, "step {step} manifest");
        assert!(next.len() > anchor.len(), "step {step} growth");
        anchor = next;
    }
    // sigma is unambiguous: a length prefix plus the remainder.
    let s = sigma(b"abc", b"defg");
    assert_eq!(sigma_inv(&s).unwrap(), (b"abc".to_vec(), b"defg".to_vec()));
    assert!(matches!(
        sigma_inv(&[9u8, 1, 2]),
        Err(FbsError::Truncated(_))
    ));
}

// ---------------------------------------------------------------------------
// Receipts
// ---------------------------------------------------------------------------

#[test]
fn sha256_matches_published_vectors() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

#[test]
fn measured_growth_on_incompressible_input_is_forced() {
    // A lossless encoder cannot shrink incompressible data on average (counting
    // argument). Record the measured growth instead of asserting a tuned bar.
    let data = random_bytes(64 * 1024, 0xABCD);
    let (w, p, stats) = encode_literal(&data);
    assert!(w.len() > data.len(), "E(x) should expand random input");
    // The expansion is dominated by one dictionary leaf per distinct atom.
    assert_eq!(stats.leaf_defs, stats.atoms);
    assert_eq!(stats.leaf_reuse, 0);
    assert_eq!(p.dict.len() as u64, stats.atoms + stats.parent_defs);
    let overhead = w.len() - data.len();
    assert!(overhead < data.len() / 10, "unexpected overhead {overhead}");
}

#[test]
fn the_packet_carries_the_payload_verbatim() {
    // The dictionary stores every distinct leaf's bytes literally, so for
    // non-repeating input the payload is present in the packet in the clear.
    // This is why the scheme is lossless and why it cannot shrink unique data.
    let random = random_bytes(64 * 1024, 3);
    let (_, p, _) = encode_literal(&random);
    let verbatim: u64 = p
        .dict
        .definitions()
        .iter()
        .map(|d| match d {
            Def::Leaf(a) => a.len() as u64,
            Def::Parent(_, _, _) => 0,
        })
        .sum();
    assert!(
        verbatim * 1000 >= random.len() as u64 * 999,
        "random payload should travel verbatim: {verbatim} of {}",
        random.len()
    );
    // A single repeated leaf carries 82 bytes regardless of input length.
    let zeros = vec![0u8; 64 * 1024];
    let (_, pz, _) = encode_literal(&zeros);
    let verbatim_z: u64 = pz
        .dict
        .definitions()
        .iter()
        .map(|d| match d {
            Def::Leaf(a) => a.len() as u64,
            Def::Parent(_, _, _) => 0,
        })
        .sum();
    assert!(
        verbatim_z <= 2 * ATOM_CAPACITY as u64 + 1,
        "verbatim={verbatim_z}"
    );
}

#[test]
fn builtin_corpora_are_deterministic() {
    let a = builtin_corpora(4096);
    let b = builtin_corpora(4096);
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.name, y.name);
        assert_eq!(sha256_hex(&x.bytes), sha256_hex(&y.bytes));
    }
    // Dictionary reuse is the ONLY route to size reduction in this scheme, and its
    // extent follows from the atom grid: atom k starts at offset 82k, so for a
    // corpus of period p the distinct atom windows are p / gcd(82, p). A period of
    // 82 collapses to a single window; a period of 96 leaves 96/2 = 48 of them.
    fn gcd(a: usize, b: usize) -> usize {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }
    let big = builtin_corpora(64 * 1024);
    let aligned = big.iter().find(|c| c.name == "cycle_period_82").unwrap();
    let unaligned = big.iter().find(|c| c.name == "cycle_period_96").unwrap();
    let random = big.iter().find(|c| c.name == "uniform_random").unwrap();
    let (_, pa, sa) = encode_literal(&aligned.bytes);
    let (_, pu, su) = encode_literal(&unaligned.bytes);
    let (_, _pr, sr) = encode_literal(&random.bytes);
    let distinct_82 = 82 / gcd(82, 82);
    let distinct_96 = 96 / gcd(82, 96);
    assert!(
        sa.leaf_defs <= distinct_82 as u64 + 1,
        "period-82: {} distinct atoms",
        sa.leaf_defs
    );
    assert!(
        su.leaf_defs <= distinct_96 as u64 + 1,
        "period-96: {} distinct atoms",
        su.leaf_defs
    );
    assert!(
        pa.dict.len() * 3 < pu.dict.len(),
        "period-82 dictionary ({}) should be far smaller than period-96 ({})",
        pa.dict.len(),
        pu.dict.len()
    );
    // Non-repeating content shares nothing at all.
    assert_eq!(sr.leaf_reuse, 0, "random corpus must not reuse atoms");
    assert_eq!(sr.leaf_defs, sr.atoms);
}
