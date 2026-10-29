//! claude-146's induction-keyed Hamming reads in the integer artifact (claude-188): their
//! parameters as a `uor-r4.lut-stack/6` container stores them, one read per depth.
//!
//! Serving the reads is a later unit of `integer-hamming-read`. Until then
//! [`super::IntegerStackModel::parse`] refuses such a container, so a read is never dropped.

use super::format::{Container, StackHamming};
use super::StackError;

/// One Hamming read's integers, at the exponents its [`StackHamming`] record names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HammingReadImage {
    /// After this many layers, or on the readout at the layer count.
    pub depth: usize,
    /// `width / 4` raw quaternions `[w, x, y, z]` at `2^rotation_exp`: the type heads' query.
    pub query_rotation: Vec<i32>,
    /// `width / 4` raw quaternions: the key, which the copy heads' queries share.
    pub key_rotation: Vec<i32>,
    /// `width / 4` raw quaternions: the type heads' value.
    pub value_rotation: Vec<i32>,
    /// `heads` margins in bits at `2^margin_exp`, each positive.
    pub margin: Vec<i32>,
    /// `2 * width` at `2^weight_exp`: row 0 the copy heads' weights, row 1 the type heads'.
    pub weight: Vec<i32>,
}

/// A stack's Hamming reads: the header record and one image per depth, in depth order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HammingImage {
    pub record: StackHamming,
    pub reads: Vec<HammingReadImage>,
}

/// The Hamming reads of a `UORLUT01` stack artifact, with every section at the length its shape
/// and record imply, or `None` for a stack without them.
pub fn read_hamming_uorlut01(bytes: &[u8]) -> Result<Option<HammingImage>, StackError> {
    hamming_image(&Container::parse(bytes)?)
}

/// The Hamming reads of a parsed container, as [`read_hamming_uorlut01`].
pub(crate) fn hamming_image(artifact: &Container<'_>) -> Result<Option<HammingImage>, StackError> {
    let Some(record) = artifact.hamming.clone() else {
        return Ok(None);
    };
    let width = artifact.shape.width;
    let mut reads = Vec::with_capacity(record.depths.len());
    for &depth in &record.depths {
        let section = |part: &str, len: usize| -> Result<Vec<i32>, StackError> {
            let name = format!("hamming.d{depth}.{part}");
            let values = artifact.table_i32(&name)?;
            if values.len() != len {
                return Err(StackError::TableLength {
                    name,
                    len: values.len(),
                    expected: len,
                });
            }
            Ok(values)
        };
        let margin = section("margin", record.heads)?;
        if margin.iter().any(|&m| m <= 0) {
            return Err(StackError::Numerics(format!(
                "hamming.d{depth}.margin: a margin is not positive"
            )));
        }
        reads.push(HammingReadImage {
            depth,
            query_rotation: section("query_rotation", width)?,
            key_rotation: section("key_rotation", width)?,
            value_rotation: section("value_rotation", width)?,
            margin,
            weight: section("weight", 2 * width)?,
        });
    }
    Ok(Some(HammingImage { record, reads }))
}
