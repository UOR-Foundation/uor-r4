//! Fixed-width domain newtypes (PDF §20/§21 style).
//!
//! Every value crossing the serialization boundary uses one of these
//! fixed-width types — never `usize`, never platform-dependent layout
//! (RFC §1 rule 4). All are `#[repr(transparent)]` over their raw integer
//! or byte-array representation.

/// Region (graph node) identifier.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u32);

/// Offset relative to the start of its containing section (RFC §1 rule 3).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectionOffset(pub u32);

/// Token identifier into the compiled vocabulary.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenId(pub u32);

/// Quantized fixed-point log-domain score.
///
/// Semantic format: one global `i32` Q16.16 declaration in HEAD
/// (RFC §9.3). This is only the raw carrier for now — storage-time dyadic
/// descriptors (`{width, shift, zero_point}`) and shift+add decoding land
/// with the EMIT tables in a later Phase-1 slice.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScoreQ(pub i32);

impl ScoreQ {
    /// Wrap a raw Q16.16 bit pattern.
    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    /// The raw Q16.16 bit pattern.
    pub const fn raw(self) -> i32 {
        self.0
    }
}

/// Multiresolution depth of a region.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Depth(pub u8);

/// Calibrated acceptance radius of a region (masked-Hamming bound).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Radius(pub u16);

/// Content address (blake3 digest) of an artifact or artifact part.
///
/// CIDs preserve identity and provenance of bytes; they are not semantic
/// hashes and are never used as routing codes (GLOSSARY, "κ / content
/// CID").
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtifactCid(pub [u8; 32]);

/// Section identifier (RFC §3 inventory).
///
/// Known IDs `0x01..=0x0B` carry the RFC §3 "Mandatory" column. The RFC
/// does not specify how a reader classifies *unknown* IDs as mandatory or
/// optional; this crate resolves the ambiguity with a PNG-style
/// critical/ancillary bit (see [`SectionId::OPTIONAL_BIT`] and
/// [`SectionId::mandatory`]).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectionId(pub u32);

impl SectionId {
    /// HEAD — identities and limits (mandatory).
    pub const HEAD: SectionId = SectionId(0x01);
    /// CODE — token codes, rolling-state programs, code layout (mandatory).
    pub const CODE: SectionId = SectionId(0x02);
    /// NODE — packed region records (mandatory).
    pub const NODE: SectionId = SectionId(0x03);
    /// EDGE — refinement/overlap/forward edges + reverse index (mandatory).
    pub const EDGE: SectionId = SectionId(0x04);
    /// ROUT — decision programs, prototypes, masks, shortlists (mandatory).
    pub const ROUT: SectionId = SectionId(0x05);
    /// EMIT — root priors, emission/transition residuals (mandatory).
    pub const EMIT: SectionId = SectionId(0x06);
    /// EXCT — exact-context residual evidence (optional).
    pub const EXCT: SectionId = SectionId(0x07);
    /// PROV — provenance roots (mandatory).
    pub const PROV: SectionId = SectionId(0x08);
    /// CERT — certification metadata (optional).
    pub const CERT: SectionId = SectionId(0x09);
    /// PTCH — patch-epoch header (optional, Phase 9).
    pub const PTCH: SectionId = SectionId(0x0A);
    /// SECT — per-section hash table (optional, reserved; RFC §9.2).
    pub const SECT: SectionId = SectionId(0x0B);

    /// Ancillary bit classifying *unknown* section IDs.
    ///
    /// Version-policy resolution (RFC §1 rule 2 / §8, which require
    /// rejecting unknown mandatory sections while skipping unknown
    /// optional ones, without defining how to tell them apart): an
    /// unknown ID with bit 31 set is optional and skipped; an unknown ID
    /// with bit 31 clear is mandatory and rejected. Writers adding a new
    /// optional section in a minor version bump set this bit so older
    /// readers skip it.
    pub const OPTIONAL_BIT: u32 = 0x8000_0000;

    /// The raw wire value.
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// True when the ID is in the RFC §3 inventory (`0x01..=0x0B`).
    pub const fn is_known(self) -> bool {
        matches!(self.0, 0x01..=0x0B)
    }

    /// Mandatory-ness per the RFC §3 column for known IDs.
    ///
    /// Unknown IDs default to mandatory unless [`SectionId::OPTIONAL_BIT`]
    /// is set (see the bit's documentation).
    pub const fn mandatory(self) -> bool {
        match self.0 {
            0x01..=0x06 | 0x08 => true,
            0x07 | 0x09..=0x0B => false,
            _ => self.0 & Self::OPTIONAL_BIT == 0,
        }
    }
}
