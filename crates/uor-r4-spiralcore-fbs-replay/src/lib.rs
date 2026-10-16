//! Independent reference implementation of the SpiralCore "Recursive FBS Engine"
//! mathematical specification.
//!
//! Source of truth: *SpiralCore Recursive FBS Engine - Exact Transformation Rules,
//! Recovery Invariants, and Embedded Replay Manifests*, SpiralCore research notes,
//! 15 October 2026, Version 1.0 (Sections 1-7 and Appendices A-B). The paper ships
//! no reference code, so this crate is a from-specification reconstruction used to
//! test the paper's claims independently. Every place where the paper is silent or
//! ambiguous is marked with a NOTE comment and collected in the lab record.
//!
//! This is a data-transformation review instrument. It performs no model compute:
//! integer and byte operations only, no floating point, no matrix multiplication.

#![forbid(unsafe_code)]

pub mod corpus;
pub mod sha256;

use std::collections::HashMap;

/// Payload capacity per atom, paper eq. (1.1): q = L0 - 1 with L0 = 83.
///
/// NOTE: the paper does not derive L0 = 83; it is taken as given.
pub const ATOM_CAPACITY: usize = 82;

/// Wire-format magic, Appendix A eq. (A.5): eight ASCII bytes.
pub const MAGIC: &[u8; 8] = b"FBS7DAG1";

/// Appendix A: "All leaf payloads have lengths 1 through 82."
pub const MAX_LEAF_LEN: usize = 82;

/// Nesting guard. The paper's composition theorem assumes adequate resources;
/// an implementation still needs a bound (Section 7, "Implementation limits").
pub const MAX_NEST_DEPTH: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FbsError {
    BadMagic,
    Truncated(&'static str),
    TrailingBytes,
    BadTag(u8),
    BadVarint,
    BadLeafLength(usize),
    ChildNotDefined { parent: u32, child: u32 },
    LevelMismatch { parent: u32, levels: [u32; 3] },
    AtomCountMismatch { pinned: u64, expanded: u64 },
    RootInventoryMismatch { pinned: u64, weight: u128 },
    BadCode,
    PinMismatch(&'static str),
    NestingLimit,
}

impl std::fmt::Display for FbsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FbsError::BadMagic => write!(f, "wire does not begin with FBS7DAG1"),
            FbsError::Truncated(what) => write!(f, "truncated field: {what}"),
            FbsError::TrailingBytes => write!(f, "bytes remain after the declared tail"),
            FbsError::BadTag(t) => write!(f, "unknown tag byte {t}"),
            FbsError::BadVarint => write!(f, "varint overflow"),
            FbsError::BadLeafLength(l) => write!(f, "leaf payload length {l} outside 1..=82"),
            FbsError::ChildNotDefined { parent, child } => {
                write!(
                    f,
                    "definition {parent} references undefined/non-causal child {child}"
                )
            }
            FbsError::LevelMismatch { parent, levels } => {
                write!(
                    f,
                    "definition {parent} binds unequal child levels {levels:?}"
                )
            }
            FbsError::AtomCountMismatch { pinned, expanded } => {
                write!(
                    f,
                    "pinned atom count {pinned} != expanded leaf count {expanded}"
                )
            }
            FbsError::RootInventoryMismatch { pinned, weight } => {
                write!(
                    f,
                    "pinned N={pinned} != sum of 3^level over frontier roots ({weight})"
                )
            }
            FbsError::BadCode => write!(f, "frontier code is not 0x01 || V(s) || V(r_1).."),
            FbsError::PinMismatch(what) => write!(f, "pinned-state check failed: {what}"),
            FbsError::NestingLimit => write!(f, "nesting limit exceeded"),
        }
    }
}

impl std::error::Error for FbsError {}

// ---------------------------------------------------------------------------
// Appendix A: V(n), shortest unsigned base-128 representation
// ---------------------------------------------------------------------------

/// V(n): shortest unsigned base-128 varint. For n = 0, one zero byte.
pub fn v_encode(mut n: u64) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let b = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(b);
            break;
        }
        out.push(b | 0x80);
    }
    out
}

pub fn v_decode(buf: &[u8], pos: &mut usize) -> Result<u64, FbsError> {
    let mut n: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        if *pos >= buf.len() {
            return Err(FbsError::Truncated("varint"));
        }
        let b = buf[*pos];
        *pos += 1;
        if shift >= 64 || (shift == 63 && (b & 0x7f) > 1) {
            return Err(FbsError::BadVarint);
        }
        n |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    Ok(n)
}

// ---------------------------------------------------------------------------
// Frontier code, Appendix A eq. (A.1)
// ---------------------------------------------------------------------------

/// B(Code(R)) for R = [r_1..r_s]: the byte string 0x01 || V(s) || V(r_1).. .
///
/// Code(R) is defined as the big-endian integer of that byte string. Because the
/// string always begins with 0x01, its shortest non-empty big-endian encoding
/// B(f) is the string itself, so no integer round-trip is needed. The paper's
/// worked examples are asserted in the conformance suite.
pub fn code_bytes(frontier: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(1 + 8 + frontier.len());
    out.push(1u8);
    out.extend(v_encode(frontier.len() as u64));
    for r in frontier {
        out.extend(v_encode(*r as u64));
    }
    out
}

/// Parse B(Code(R)) back into an ordered frontier.
pub fn parse_code(code: &[u8]) -> Result<Vec<u32>, FbsError> {
    let mut pos = 0usize;
    if code.is_empty() || code[0] != 1 {
        return Err(FbsError::BadCode);
    }
    pos += 1;
    let s = v_decode(code, &mut pos)? as usize;
    let mut frontier = Vec::with_capacity(s.min(1 << 20));
    for _ in 0..s {
        let r = v_decode(code, &mut pos)?;
        if r == 0 || r > u32::MAX as u64 {
            return Err(FbsError::BadCode);
        }
        frontier.push(r as u32);
    }
    if pos != code.len() {
        return Err(FbsError::BadCode);
    }
    Ok(frontier)
}

/// G(f) = V(|B(f)|) || B(f), Appendix A eq. (A.5).
pub fn g_encode(frontier: &[u32]) -> Vec<u8> {
    let b = code_bytes(frontier);
    let mut out = v_encode(b.len() as u64);
    out.extend_from_slice(&b);
    out
}

// ---------------------------------------------------------------------------
// Dictionary, Sections 2-3 and Appendix A eqs. (A.2)-(A.4)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Def {
    /// Leaf(a) = (0, a)
    Leaf(Vec<u8>),
    /// Parent(u, v, w) = (1, u, v, w)
    Parent(u32, u32, u32),
}

#[derive(Debug, Default, Clone)]
pub struct Dictionary {
    defs: Vec<Def>,
    levels: Vec<u32>,
    index: HashMap<Def, u32>,
}

impl Dictionary {
    pub fn new() -> Self {
        Dictionary::default()
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    pub fn def(&self, id: u32) -> Option<&Def> {
        if id == 0 {
            None
        } else {
            self.defs.get((id - 1) as usize)
        }
    }

    pub fn level(&self, id: u32) -> u32 {
        if id == 0 {
            0
        } else {
            self.levels[(id - 1) as usize]
        }
    }

    pub fn definitions(&self) -> &[Def] {
        &self.defs
    }

    pub fn leaf_count_of(&self, id: u32) -> u64 {
        let mut count = 0u64;
        let mut stack = vec![id];
        while let Some(cur) = stack.pop() {
            match self.def(cur) {
                Some(Def::Leaf(_)) => count += 1,
                Some(Def::Parent(u, v, w)) => {
                    stack.push(*w);
                    stack.push(*v);
                    stack.push(*u);
                }
                None => return 0,
            }
        }
        count
    }

    /// Intern_T for a leaf definition (Section 2 eq. 2.1). Returns (id, reused).
    pub fn intern_leaf(&mut self, a: &[u8]) -> (u32, bool) {
        let key = Def::Leaf(a.to_vec());
        if let Some(id) = self.index.get(&key) {
            return (*id, true);
        }
        let id = self.defs.len() as u32 + 1;
        self.defs.push(key.clone());
        self.levels.push(0);
        self.index.insert(key, id);
        (id, false)
    }

    /// Intern_T for a parent definition. Callers must have checked equal child levels.
    pub fn intern_parent(&mut self, u: u32, v: u32, w: u32) -> (u32, bool) {
        debug_assert_eq!(self.level(u), self.level(v));
        debug_assert_eq!(self.level(v), self.level(w));
        let key = Def::Parent(u, v, w);
        if let Some(id) = self.index.get(&key) {
            return (*id, true);
        }
        let id = self.defs.len() as u32 + 1;
        let lvl = self.level(u) + 1;
        self.defs.push(key.clone());
        self.levels.push(lvl);
        self.index.insert(key, id);
        (id, false)
    }

    /// Positional append used by the parser. The wire's definition list is
    /// positional by node ID (Appendix A), so a repeated definition is NOT
    /// merged: it keeps its own ID. Duplicates are noncanonical (the encoder
    /// never emits them) but the paper does not promise to reject every
    /// noncanonical encoding, so the parser accepts them and the first ID wins
    /// in the lookup index.
    fn push_positional(&mut self, d: Def) -> Result<u32, FbsError> {
        let id = self.defs.len() as u32 + 1;
        let level = match &d {
            Def::Leaf(a) => {
                if a.is_empty() || a.len() > MAX_LEAF_LEN {
                    return Err(FbsError::BadLeafLength(a.len()));
                }
                0
            }
            Def::Parent(u, v, w) => {
                for c in [u, v, w] {
                    if *c == 0 || *c >= id {
                        return Err(FbsError::ChildNotDefined {
                            parent: id,
                            child: *c,
                        });
                    }
                }
                let (lu, lv, lw) = (self.level(*u), self.level(*v), self.level(*w));
                if lu != lv || lv != lw {
                    return Err(FbsError::LevelMismatch {
                        parent: id,
                        levels: [lu, lv, lw],
                    });
                }
                lu + 1
            }
        };
        let key = d.clone();
        self.defs.push(d);
        self.levels.push(level);
        self.index.entry(key).or_insert(id);
        Ok(id)
    }

    /// Expand_T(j) (Section 5 eq. 5.1), iterative so that deep dictionaries
    /// cannot overflow the stack.
    pub fn expand(&self, id: u32) -> Result<Vec<u8>, FbsError> {
        let mut out = Vec::new();
        let mut stack = vec![id];
        while let Some(cur) = stack.pop() {
            match self.def(cur) {
                Some(Def::Leaf(a)) => out.extend_from_slice(a),
                Some(Def::Parent(u, v, w)) => {
                    stack.push(*w);
                    stack.push(*v);
                    stack.push(*u);
                }
                None => {
                    return Err(FbsError::ChildNotDefined {
                        parent: cur,
                        child: 0,
                    })
                }
            }
        }
        Ok(out)
    }

    /// D(P) for an ordered frontier (Section 5 eq. 5.2).
    pub fn expand_frontier(&self, frontier: &[u32]) -> Result<Vec<u8>, FbsError> {
        let mut out = Vec::new();
        for r in frontier {
            out.extend_from_slice(&self.expand(*r)?);
        }
        Ok(out)
    }

    /// Number of leaf occurrences reached from an ordered frontier.
    pub fn leaf_count_frontier(&self, frontier: &[u32]) -> u64 {
        frontier.iter().map(|r| self.leaf_count_of(*r)).sum()
    }

    /// tau(T), Appendix A eq. (A.2)-(A.4).
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = v_encode(self.defs.len() as u64);
        for d in &self.defs {
            match d {
                Def::Leaf(a) => {
                    out.push(0);
                    out.extend(v_encode(a.len() as u64));
                    out.extend_from_slice(a);
                }
                Def::Parent(u, v, w) => {
                    out.push(1);
                    out.extend(v_encode(*u as u64));
                    out.extend(v_encode(*v as u64));
                    out.extend(v_encode(*w as u64));
                }
            }
        }
        out
    }

    /// Parse tau(T) with the reference parser's checks: extents, node references,
    /// level compatibility, leaf lengths.
    pub fn parse(bytes: &[u8]) -> Result<Self, FbsError> {
        let mut pos = 0usize;
        let d = v_decode(bytes, &mut pos)? as usize;
        let mut dict = Dictionary::new();
        for _ in 0..d {
            if pos >= bytes.len() {
                return Err(FbsError::Truncated("definition tag"));
            }
            let tag = bytes[pos];
            pos += 1;
            match tag {
                0 => {
                    let len = v_decode(bytes, &mut pos)? as usize;
                    if len == 0 || len > MAX_LEAF_LEN {
                        return Err(FbsError::BadLeafLength(len));
                    }
                    if pos + len > bytes.len() {
                        return Err(FbsError::Truncated("leaf payload"));
                    }
                    let a = bytes[pos..pos + len].to_vec();
                    pos += len;
                    dict.push_positional(Def::Leaf(a))?;
                }
                1 => {
                    let u = v_decode(bytes, &mut pos)?;
                    let v = v_decode(bytes, &mut pos)?;
                    let w = v_decode(bytes, &mut pos)?;
                    if u > u32::MAX as u64 || v > u32::MAX as u64 || w > u32::MAX as u64 {
                        return Err(FbsError::BadVarint);
                    }
                    dict.push_positional(Def::Parent(u as u32, v as u32, w as u32))?;
                }
                t => return Err(FbsError::BadTag(t)),
            }
        }
        if pos != bytes.len() {
            return Err(FbsError::TrailingBytes);
        }
        Ok(dict)
    }
}

// ---------------------------------------------------------------------------
// The packet, Sections 3-4 and Appendix A eq. (A.5)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    pub index: u64,
    pub code: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub pins: [Pin; 3],
    pub dict: Dictionary,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EncodeStats {
    pub atoms: u64,
    pub carries: u64,
    pub leaf_defs: u64,
    pub parent_defs: u64,
    pub leaf_reuse: u64,
    pub parent_reuse: u64,
}

/// Ordered ternary reduction with exact dictionary reuse (Sections 1-3).
pub fn encode(x: &[u8]) -> (Packet, EncodeStats) {
    let mut dict = Dictionary::new();
    let mut stats = EncodeStats::default();
    let n_atoms = x.len().div_ceil(ATOM_CAPACITY) as u64;
    let mut frontier: Vec<u32> = Vec::new();
    let mut snap1: Option<Vec<u32>> = None;
    let mut snap2: Option<Vec<u32>> = None;

    for i in 0..n_atoms {
        let start = (i as usize) * ATOM_CAPACITY;
        let end = (start + ATOM_CAPACITY).min(x.len());
        let (id, reused) = dict.intern_leaf(&x[start..end]);
        if reused {
            stats.leaf_reuse += 1;
        } else {
            stats.leaf_defs += 1;
        }
        frontier.push(id);
        loop {
            let l = frontier.len();
            if l < 3 {
                break;
            }
            let (a, b, c) = (frontier[l - 3], frontier[l - 2], frontier[l - 1]);
            if dict.level(a) == dict.level(b) && dict.level(b) == dict.level(c) {
                let (p, reused) = dict.intern_parent(a, b, c);
                if reused {
                    stats.parent_reuse += 1;
                } else {
                    stats.parent_defs += 1;
                }
                frontier.truncate(l - 3);
                frontier.push(p);
                stats.carries += 1;
            } else {
                break;
            }
        }
        if i == 0 {
            snap1 = Some(frontier.clone());
        }
        if i == 1 {
            snap2 = Some(frontier.clone());
        }
    }
    stats.atoms = n_atoms;

    let pins = if n_atoms == 0 {
        // Boundary convention: all three pins are (0, Code([])) and T is empty.
        let code = code_bytes(&[]);
        [
            Pin {
                index: 0,
                code: code.clone(),
            },
            Pin {
                index: 0,
                code: code.clone(),
            },
            Pin { index: 0, code },
        ]
    } else if n_atoms == 1 {
        // Boundary convention: the first admission pin is duplicated as the second.
        let code = code_bytes(snap1.as_deref().unwrap_or(&[]));
        [
            Pin {
                index: 1,
                code: code.clone(),
            },
            Pin {
                index: 1,
                code: code.clone(),
            },
            Pin { index: 1, code },
        ]
    } else {
        [
            Pin {
                index: 1,
                code: code_bytes(snap1.as_deref().unwrap_or(&[])),
            },
            Pin {
                index: 2,
                code: code_bytes(snap2.as_deref().unwrap_or(&[])),
            },
            Pin {
                index: n_atoms,
                code: code_bytes(&frontier),
            },
        ]
    };

    (Packet { pins, dict }, stats)
}

/// Wire(P, eta, b), Appendix A eq. (A.5).
pub fn wire(packet: &Packet, eta: u8, tail: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + 16 + tail.len());
    out.extend_from_slice(MAGIC);
    for pin in &packet.pins {
        out.extend(v_encode(pin.index));
        out.extend(v_encode(pin.code.len() as u64));
        out.extend_from_slice(&pin.code);
    }
    out.push(eta);
    out.extend(v_encode(tail.len() as u64));
    out.extend_from_slice(tail);
    out
}

/// The canonical literal packet E(x) = ((1,F1),(2,F2),(N,F3),T).
pub fn encode_literal(x: &[u8]) -> (Vec<u8>, Packet, EncodeStats) {
    let (packet, stats) = encode(x);
    let w = wire(&packet, 0, &packet.dict.serialize());
    (w, packet, stats)
}

/// D(P): parse, check, expand.
pub fn decode(w: &[u8]) -> Result<Vec<u8>, FbsError> {
    decode_at_depth(w, 0)
}

fn decode_at_depth(w: &[u8], depth: usize) -> Result<Vec<u8>, FbsError> {
    if depth > MAX_NEST_DEPTH {
        return Err(FbsError::NestingLimit);
    }
    if w.len() < MAGIC.len() || &w[..MAGIC.len()] != MAGIC {
        return Err(FbsError::BadMagic);
    }
    let mut pos = MAGIC.len();
    let mut pins: Vec<Pin> = Vec::with_capacity(3);
    for _ in 0..3 {
        let index = v_decode(w, &mut pos)?;
        let blen = v_decode(w, &mut pos)? as usize;
        if pos + blen > w.len() {
            return Err(FbsError::Truncated("pin code"));
        }
        let code = w[pos..pos + blen].to_vec();
        pos += blen;
        pins.push(Pin { index, code });
    }
    if pos >= w.len() {
        return Err(FbsError::Truncated("eta"));
    }
    let eta = w[pos];
    pos += 1;
    if eta > 1 {
        return Err(FbsError::BadTag(eta));
    }
    let tlen = v_decode(w, &mut pos)? as usize;
    if pos + tlen > w.len() {
        return Err(FbsError::Truncated("tail"));
    }
    if pos + tlen != w.len() {
        return Err(FbsError::TrailingBytes);
    }
    let tail = &w[pos..];
    let dict_bytes = if eta == 0 {
        tail.to_vec()
    } else {
        decode_at_depth(tail, depth + 1)?
    };
    let dict = Dictionary::parse(&dict_bytes)?;

    // Third pin: the final frontier R_N.
    let frontier = parse_code(&pins[2].code)?;
    let payload = dict.expand_frontier(&frontier)?;

    // Reference-parser checks (Appendix A closing paragraph).
    let leaves = dict.leaf_count_frontier(&frontier);
    if leaves != pins[2].index {
        return Err(FbsError::AtomCountMismatch {
            pinned: pins[2].index,
            expanded: leaves,
        });
    }
    let mut weight: u128 = 0;
    for r in &frontier {
        weight += 3u128.pow(dict.level(*r));
    }
    if weight != pins[2].index as u128 {
        return Err(FbsError::RootInventoryMismatch {
            pinned: pins[2].index,
            weight,
        });
    }
    if pins[2].index == 0 {
        if !frontier.is_empty() || !payload.is_empty() {
            return Err(FbsError::PinMismatch("zero-atom packet must be empty"));
        }
        if pins[0].index != 0 || pins[1].index != 0 {
            return Err(FbsError::PinMismatch(
                "zero-atom packet pins must be (0,Code([]))",
            ));
        }
        return Ok(payload);
    }

    // Pinned states: pin 1 is the frontier after one admission, pin 2 after two.
    let f1 = parse_code(&pins[0].code)?;
    let e1 = dict.expand_frontier(&f1)?;
    let atom1_len = ATOM_CAPACITY.min(payload.len());
    if pins[0].index != 1 || e1 != payload[..atom1_len] {
        return Err(FbsError::PinMismatch("pin 1 is not the first atom"));
    }
    if pins[2].index == 1 {
        if pins[1].index != 1 || pins[1].code != pins[0].code {
            return Err(FbsError::PinMismatch(
                "one-atom packet must duplicate pin 1",
            ));
        }
    } else {
        let f2 = parse_code(&pins[1].code)?;
        let e2 = dict.expand_frontier(&f2)?;
        let two_len = (2 * ATOM_CAPACITY).min(payload.len());
        if pins[1].index != 2 || e2 != payload[..two_len] {
            return Err(FbsError::PinMismatch("pin 2 is not the first two atoms"));
        }
    }
    Ok(payload)
}

// ---------------------------------------------------------------------------
// Section 6: nested literal-tail composition, and Section 7: anchor+manifest
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Nested {
    /// dictionaries[j] = tau(T_j)
    pub dictionaries: Vec<Vec<u8>>,
    /// packets[j] = P_j = E(T_{j-1}) for j >= 1, P_0 given
    pub packets: Vec<Packet>,
    /// wires[j] = Q_j ; wires[0] is the outermost nested packet
    pub wires: Vec<Vec<u8>>,
}

/// C_n(P_0) from Section 6.
pub fn nest(p0: &Packet, n: usize) -> Nested {
    let mut dictionaries = vec![p0.dict.serialize()];
    let mut packets = vec![p0.clone()];
    for j in 1..=n {
        let payload = dictionaries[j - 1].clone();
        let (pkt, _) = encode(&payload);
        dictionaries.push(pkt.dict.serialize());
        packets.push(pkt);
    }
    let mut wires = vec![Vec::new(); n + 1];
    wires[n] = wire(&packets[n], 0, &dictionaries[n]);
    for j in (0..n).rev() {
        let inner = wires[j + 1].clone();
        wires[j] = wire(&packets[j], 1, &inner);
    }
    Nested {
        dictionaries,
        packets,
        wires,
    }
}

/// Section 7: sigma(a, b) = V(|a|) || a || b. A fixed, unambiguous binary
/// serialization with inverse sigma^{-1} on its valid image; the paper leaves
/// sigma unspecified, so this is the review's instantiation (NOTE).
pub fn sigma(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut out = v_encode(a.len() as u64);
    out.extend_from_slice(a);
    out.extend_from_slice(b);
    out
}

pub fn sigma_inv(s: &[u8]) -> Result<(Vec<u8>, Vec<u8>), FbsError> {
    let mut pos = 0usize;
    let alen = v_decode(s, &mut pos)? as usize;
    if pos + alen > s.len() {
        return Err(FbsError::Truncated("sigma payload"));
    }
    let a = s[pos..pos + alen].to_vec();
    let b = s[pos + alen..].to_vec();
    Ok((a, b))
}

/// One round of P_{k+1} = E(sigma(P_k, M_k)).
pub fn absorb(anchor: &[u8], manifest: &[u8]) -> Vec<u8> {
    let (w, _, _) = encode_literal(&sigma(anchor, manifest));
    w
}

// ---------------------------------------------------------------------------
// Small integer helpers used by the conformance suite
// ---------------------------------------------------------------------------

/// s_3(N): sum of the base-3 digits of N.
pub fn digit_sum_base3(mut n: u64) -> u64 {
    let mut s = 0;
    while n > 0 {
        s += n % 3;
        n /= 3;
    }
    s
}

/// W(N) = (N - s_3(N)) / 2, paper eq. (3.8).
pub fn predicted_carries(n: u64) -> u64 {
    (n - digit_sum_base3(n)) / 2
}
