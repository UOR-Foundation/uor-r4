//! The single focused error type for parsing, validation, serialization,
//! and CID verification.

use core::fmt;

use crate::types::SectionId;

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
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FormatError {}
