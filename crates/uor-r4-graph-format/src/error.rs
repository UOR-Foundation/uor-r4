//! The single focused error type for parsing, validation, serialization,
//! and CID verification.

use core::fmt;

use crate::types::SectionId;

/// Which packed-node range field failed to resolve within its target
/// section (stage 2, RFC §6 item 4). Targets per the v0 draft line:
/// `Child` → the canonical edge array, `Forward` → the EDGE reverse
/// index, `Emission` → the EMIT remainder (bytes after the storage
/// descriptor), `Prototype`/`Mask` → the ROUT section size in u64 words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeField {
    /// `child_start`/`child_len` over the canonical edge array.
    Child,
    /// `forward_start`/`forward_len` over the reverse index.
    Forward,
    /// `emission_start`/`emission_len` over the EMIT remainder.
    Emission,
    /// `prototype_word_start` into the ROUT section (u64 words).
    Prototype,
    /// `mask_word_start` into the ROUT section (u64 words).
    Mask,
}

impl fmt::Display for RangeField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            RangeField::Child => "child",
            RangeField::Forward => "forward",
            RangeField::Emission => "emission",
            RangeField::Prototype => "prototype",
            RangeField::Mask => "mask",
        };
        write!(f, "{name}")
    }
}

/// Which HEAD-declared bound was smaller than the maximum observed in
/// the sections (stage 2, RFC §6 item 7: bounds must be honest).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundKind {
    /// `A` (max frontier width) vs. the observed max `child_len`.
    FrontierWidth,
    /// `E` (max emission entries per region) vs. the observed max
    /// `emission_len`.
    EmissionEntries,
    /// `depth_count` vs. the observed max node `depth`.
    DepthCount,
    /// `signature_bytes` vs. `W * 8` (cross-check).
    SignatureBytes,
}

impl fmt::Display for BoundKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            BoundKind::FrontierWidth => "max frontier width A",
            BoundKind::EmissionEntries => "max emission entries E",
            BoundKind::DepthCount => "depth_count",
            BoundKind::SignatureBytes => "signature_bytes",
        };
        write!(f, "{name}")
    }
}

/// Every fallible operation in this crate returns this error.
///
/// Variants map one-to-one onto the stage-1 structural invariants of
/// RFC §6 plus the serializer/CID failure modes. All data carried is
/// `Copy`; no allocation, no source chains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatError {
    /// Bytes do not start with the `R4G1` magic.
    BadMagic,
    /// `format_version.major` is not one this reader supports.
    UnsupportedMajorVersion(u8),
    /// Endianness marker is not `0x01` (little-endian).
    UnsupportedEndianness(u8),
    /// `alignment_log2` is outside the supported range `3..=31`
    /// (RFC §2 requires ≥ 3; > 31 is meaningless under u32 offsets).
    UnsupportedAlignment(u8),
    /// Fewer bytes than the fixed 88-byte header.
    TruncatedHeader,
    /// Declared `total_len` does not equal the actual buffer length.
    TotalLenMismatch {
        /// `total_len` as declared in the header.
        declared: u64,
        /// Actual buffer length.
        actual: u64,
    },
    /// A set header flag bit lies in the mandatory feature space but is
    /// not defined by this format version (RFC §6 stage-1 rule 2).
    /// Carries the offending bit mask.
    UnknownMandatoryFeature(u32),
    /// The section table (`section_count` × 16 bytes) extends past
    /// `total_len`.
    SectionTableOutOfBounds,
    /// Section table entries are not in strictly increasing
    /// `section_id` order (canonical ordering, RFC §2). Also covers
    /// duplicate IDs.
    SectionsNotSorted,
    /// An unknown section ID without [`SectionId::OPTIONAL_BIT`] —
    /// i.e. an unknown *mandatory* section (RFC §6 stage-1 rule 2).
    UnknownMandatorySection(u32),
    /// A section offset is not a multiple of `1 << alignment_log2`.
    SectionMisaligned,
    /// `offset + length` overflowed `u32` under checked arithmetic.
    OffsetOverflow,
    /// A section's `[offset, offset + length)` range extends past
    /// `total_len`.
    SectionOutOfBounds,
    /// Two section bodies overlap, or a section body overlaps the
    /// header / section-table region.
    SectionsOverlap,
    /// Serializer: the same section ID was added twice.
    DuplicateSection(SectionId),
    /// Serializer or CID verifier: the mandatory HEAD section is absent,
    /// so `head_cid` cannot be computed or checked.
    MissingHead,
    /// Serializer: a section payload exceeds the u32 length ceiling
    /// (RFC §9.1, ≤ 4 GiB per section). Carries the payload length.
    SectionTooLarge(u64),
    /// `head_cid` does not recompute to the HEAD section body.
    HeadCidMismatch,
    /// `artifact_cid` does not recompute to `artifact_bytes[56..]`.
    ArtifactCidMismatch,
    /// HEAD section body is shorter than the fixed 224-byte v0 prefix
    /// (RFC §4 draft-line layout).
    HeadTooShort {
        /// Actual HEAD payload length.
        actual: u64,
    },
    /// HEAD section body carries trailing bytes past the fixed 224-byte
    /// v0 prefix. Rejected (not ignored) so a future HEAD extension must
    /// arrive with a format minor-version bump (RFC §8).
    HeadTooLong {
        /// Actual HEAD payload length.
        actual: u64,
    },
    /// HEAD declares `node_count > 0` but the NODE section is absent
    /// (stage 2 requires NODE iff `node_count > 0`).
    MissingNodeSection,
    /// NODE section byte length ≠ `node_count × 30` (RFC §6 item 4:
    /// record count must equal the declared count).
    NodeCountMismatch {
        /// `node_count` declared in HEAD.
        declared: u32,
        /// Actual NODE section length in bytes.
        section_len: u64,
    },
    /// HEAD declares `edge_count > 0` but the EDGE section is absent.
    MissingEdgeSection,
    /// EDGE section byte length ≠ `edge_count × (16 + 4)` — canonical
    /// edges plus the reverse index (RFC §5 EDGE).
    EdgeCountMismatch {
        /// `edge_count` declared in HEAD.
        declared: u32,
        /// Actual EDGE section length in bytes.
        section_len: u64,
    },
    /// A packed-node range field does not resolve within its target
    /// section under checked arithmetic (RFC §6 item 4).
    RangeOutOfBounds {
        /// Node (record) index carrying the bad range.
        node: u32,
        /// Which range field failed to resolve.
        field: RangeField,
    },
    /// An edge endpoint is ≥ `node_count` (RFC §6 item 5).
    EdgeEndpointOutOfBounds {
        /// Edge (canonical array) index.
        edge: u32,
        /// Decoded `src` field.
        src: u32,
        /// Decoded `dst` field.
        dst: u32,
    },
    /// A reverse-index entry is ≥ `edge_count` (RFC §6 item 5).
    ReverseIndexOutOfBounds {
        /// Reverse-index position.
        index: u32,
        /// Offending edge ID stored there.
        edge_id: u32,
    },
    /// A canonical edge has no entry anywhere in the reverse index —
    /// the v0 existence approximation of Theorem 7 (RFC §6 item 5;
    /// full per-node range wiring comes later).
    ReverseIndexMissing {
        /// Canonical edge index with no reverse entry.
        edge: u32,
    },
    /// A HEAD-declared bound is smaller than the maximum observed in
    /// the sections (RFC §6 item 7: bounds must be honest).
    DishonestBounds {
        /// Which bound was understated.
        bound: BoundKind,
        /// Value declared in HEAD.
        declared: u32,
        /// Maximum actually observed (for `SignatureBytes`: the honest
        /// value `W * 8`).
        observed: u32,
    },
    /// ROUT bytecode opcode outside the v0 set (RFC §6 item 6).
    UnknownRoutingOp {
        /// Byte offset of the opcode within the ROUT section.
        offset: u32,
        /// The unknown opcode byte.
        opcode: u8,
    },
    /// A ROUT op's fixed operands run past the section end.
    TruncatedRoutingOp {
        /// Byte offset of the op within the ROUT section.
        offset: u32,
        /// The opcode whose operands are truncated.
        opcode: u8,
    },
    /// ROUT static op count exceeds HEAD `D` (with forward-only jumps,
    /// the static count bounds every execution path — RFC §6 item 6).
    RoutingProgramTooDeep {
        /// Ops parsed before the terminator.
        ops: u32,
        /// HEAD `D` (max decision-program steps).
        max: u32,
    },
    /// ROUT program ends neither at `HALT` nor (at section end) at a
    /// `LEAF` — the v0 form of "at least one LEAF or HALT reachable".
    RoutingProgramUnterminated,
    /// `JMP_FWD` target op index lies outside the program (jumps are
    /// forward-only by construction; this is the in-bounds half of
    /// RFC §6 item 6).
    RoutingJumpOutOfBounds {
        /// Index of the jumping op.
        op_index: u32,
        /// Computed target op index.
        target: u64,
    },
    /// `TEST_POPCOUNT_LE` operand out of range: `word` ≥ HEAD `W` or
    /// `threshold` > 64 (popcount ceiling of a u64).
    RoutingOperandOutOfBounds {
        /// Index of the offending op.
        op_index: u32,
    },
    /// `LEAF` shortlist range does not resolve within the trailing
    /// shortlist table — or no table is present and `shortlist_len ≠ 0`.
    RoutingShortlistOutOfBounds {
        /// Index of the offending LEAF op.
        op_index: u32,
    },
    /// EMIT/EXCT storage descriptor invalid (RFC §6 item 8): fewer than
    /// 4 bytes, `width ∉ {0,1,2}`, or `|shift| > 31`.
    InvalidStorageDescriptor {
        /// Section carrying the bad descriptor (EMIT or EXCT).
        section: SectionId,
    },
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::BadMagic => write!(f, "bad magic: not an R4G1 artifact"),
            FormatError::UnsupportedMajorVersion(v) => {
                write!(f, "unsupported format major version {v}")
            }
            FormatError::UnsupportedEndianness(m) => {
                write!(
                    f,
                    "unsupported endianness marker 0x{m:02x} (expected 0x01 = little)"
                )
            }
            FormatError::UnsupportedAlignment(a) => {
                write!(f, "unsupported alignment_log2 {a} (supported: 3..=31)")
            }
            FormatError::TruncatedHeader => write!(f, "buffer shorter than the 88-byte header"),
            FormatError::TotalLenMismatch { declared, actual } => write!(
                f,
                "total_len mismatch: header declares {declared} bytes, buffer has {actual}"
            ),
            FormatError::UnknownMandatoryFeature(mask) => {
                write!(f, "unknown mandatory feature bit(s) set: 0x{mask:08x}")
            }
            FormatError::SectionTableOutOfBounds => {
                write!(f, "section table extends past total_len")
            }
            FormatError::SectionsNotSorted => {
                write!(f, "section table entries not sorted by section_id")
            }
            FormatError::UnknownMandatorySection(id) => {
                write!(f, "unknown mandatory section id 0x{id:08x}")
            }
            FormatError::SectionMisaligned => {
                write!(f, "section offset not aligned to 1 << alignment_log2")
            }
            FormatError::OffsetOverflow => write!(f, "section offset + length overflowed u32"),
            FormatError::SectionOutOfBounds => {
                write!(f, "section body range extends past total_len")
            }
            FormatError::SectionsOverlap => {
                write!(
                    f,
                    "section bodies overlap each other or the header/table region"
                )
            }
            FormatError::DuplicateSection(id) => {
                write!(f, "duplicate section id 0x{:08x}", id.raw())
            }
            FormatError::MissingHead => write!(f, "mandatory HEAD section is absent"),
            FormatError::SectionTooLarge(len) => {
                write!(
                    f,
                    "section payload of {len} bytes exceeds the u32 length ceiling"
                )
            }
            FormatError::HeadCidMismatch => {
                write!(f, "head_cid does not match the HEAD section body")
            }
            FormatError::ArtifactCidMismatch => {
                write!(
                    f,
                    "artifact_cid does not match artifact_bytes[56..total_len]"
                )
            }
            FormatError::HeadTooShort { actual } => write!(
                f,
                "HEAD payload of {actual} bytes is shorter than the fixed 224-byte prefix"
            ),
            FormatError::HeadTooLong { actual } => write!(
                f,
                "HEAD payload of {actual} bytes has trailing bytes past the 224-byte prefix"
            ),
            FormatError::MissingNodeSection => {
                write!(f, "HEAD declares node_count > 0 but the NODE section is absent")
            }
            FormatError::NodeCountMismatch {
                declared,
                section_len,
            } => write!(
                f,
                "NODE section holds {section_len} bytes, not node_count {declared} x 30"
            ),
            FormatError::MissingEdgeSection => {
                write!(f, "HEAD declares edge_count > 0 but the EDGE section is absent")
            }
            FormatError::EdgeCountMismatch {
                declared,
                section_len,
            } => write!(
                f,
                "EDGE section holds {section_len} bytes, not edge_count {declared} x 20"
            ),
            FormatError::RangeOutOfBounds { node, field } => write!(
                f,
                "node {node}: {field} range does not resolve within its target section"
            ),
            FormatError::EdgeEndpointOutOfBounds { edge, src, dst } => write!(
                f,
                "edge {edge}: endpoint (src {src}, dst {dst}) is not below node_count"
            ),
            FormatError::ReverseIndexOutOfBounds { index, edge_id } => write!(
                f,
                "reverse index entry {index} references edge {edge_id}, not below edge_count"
            ),
            FormatError::ReverseIndexMissing { edge } => {
                write!(f, "canonical edge {edge} has no reverse-index entry")
            }
            FormatError::DishonestBounds {
                bound,
                declared,
                observed,
            } => write!(
                f,
                "dishonest HEAD bound: {bound} declared {declared} but observed {observed}"
            ),
            FormatError::UnknownRoutingOp { offset, opcode } => write!(
                f,
                "ROUT offset {offset}: unknown routing opcode 0x{opcode:02x}"
            ),
            FormatError::TruncatedRoutingOp { offset, opcode } => write!(
                f,
                "ROUT offset {offset}: operands of opcode 0x{opcode:02x} run past the section end"
            ),
            FormatError::RoutingProgramTooDeep { ops, max } => write!(
                f,
                "ROUT program of {ops} ops exceeds HEAD D (max {max} steps)"
            ),
            FormatError::RoutingProgramUnterminated => {
                write!(f, "ROUT program ends at neither a HALT nor a final LEAF op")
            }
            FormatError::RoutingJumpOutOfBounds { op_index, target } => write!(
                f,
                "ROUT op {op_index}: jump target op {target} is outside the program"
            ),
            FormatError::RoutingOperandOutOfBounds { op_index } => write!(
                f,
                "ROUT op {op_index}: operand out of range (word < HEAD W, threshold <= 64)"
            ),
            FormatError::RoutingShortlistOutOfBounds { op_index } => write!(
                f,
                "ROUT op {op_index}: LEAF shortlist range does not resolve within the trailing table"
            ),
            FormatError::InvalidStorageDescriptor { section } => write!(
                f,
                "section 0x{:08x}: invalid storage descriptor (4 bytes, width in {{0,1,2}}, |shift| <= 31)",
                section.raw()
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FormatError {}
