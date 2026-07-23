//! Experimental candidate scoring variants (issue #80).
//!
//! The reserve-candidate variants — cloud-size normalization and margin
//! weighting — were evaluated for Gate C and **rejected** (0.0000% top-1
//! gain over the chain-telescoped baseline on the D3 held-out split).
//! They are retained only for the side-by-side Gate C comparison and are
//! **not** part of the deployed scorer.
//!
//! The transforms are deliberately isolated here, away from
//! [`super::score_runtime`], so that the P-4 source scan on
//! `score_runtime.rs` stays a hard guarantee: normalization and margin
//! weighting are inherently multiply/divide operations, and keeping them
//! out of the operator-clean integer scoring core prevents them from
//! silently weakening that guarantee. The arithmetic is integer only (no
//! float); intermediate products are computed in `i128` and saturated
//! back into the `ScoreQ` `i32` range so an out-of-range product can
//! never wrap and corrupt a candidate score.

use serde::{Deserialize, Serialize};
use uor_r4_graph_format::ScoreQ;

/// Candidate scoring variant (issue #80). `ChainTelescoped` is the
/// deployed default; the other two are rejected experimental variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoringVariant {
    /// Chain-telescoped scoring (default HEAD behavior: unweighted sum of
    /// chain region residuals).
    #[default]
    ChainTelescoped,
    /// Cloud-size normalized scoring (residual divided by chain length).
    CloudSizeNormalized,
    /// Margin-weighted residual stacking (residual scaled by the
    /// normalized membership margin relative to region radius).
    MarginWeighted,
}

impl ScoringVariant {
    /// Apply the variant transform to one chain emission residual.
    ///
    /// `chain_len` is the selected covered-chain length, `margin` the
    /// membership margin of the contributing region (clamped to be
    /// non-negative), and `radius` its region radius (clamped to at least
    /// one so the division is well defined). The result is saturated into
    /// the `ScoreQ` range.
    pub fn apply(self, value: ScoreQ, chain_len: usize, margin: i32, radius: u32) -> ScoreQ {
        match self {
            ScoringVariant::ChainTelescoped => value,
            ScoringVariant::CloudSizeNormalized => {
                let n = chain_len.max(1) as i128;
                let scaled = i128::from(value.raw()) / n;
                ScoreQ::from_raw(saturate_i32(scaled))
            }
            ScoringVariant::MarginWeighted => {
                let m = i128::from(margin.max(0));
                let r = i128::from(radius).max(1);
                let scaled = i128::from(value.raw()) * m / r;
                ScoreQ::from_raw(saturate_i32(scaled))
            }
        }
    }
}

/// Clamp a wide intermediate into the `i32` range so an out-of-range
/// product saturates instead of wrapping.
fn saturate_i32(value: i128) -> i32 {
    if value > i128::from(i32::MAX) {
        i32::MAX
    } else if value < i128::from(i32::MIN) {
        i32::MIN
    } else {
        value as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_telescoped_is_identity() {
        let v = ScoreQ::from_raw(1234);
        assert_eq!(ScoringVariant::ChainTelescoped.apply(v, 4, 3, 8), v);
    }

    #[test]
    fn cloud_size_divides_by_chain_length() {
        let v = ScoreQ::from_raw(1000);
        assert_eq!(
            ScoringVariant::CloudSizeNormalized.apply(v, 4, 0, 1).raw(),
            250
        );
        // A zero chain length is clamped to one (no division by zero).
        assert_eq!(
            ScoringVariant::CloudSizeNormalized.apply(v, 0, 0, 1).raw(),
            1000
        );
    }

    #[test]
    fn margin_weighted_scales_and_saturates() {
        let v = ScoreQ::from_raw(1000);
        assert_eq!(ScoringVariant::MarginWeighted.apply(v, 4, 2, 4).raw(), 500);
        // A negative margin clamps to zero; a zero radius clamps to one.
        assert_eq!(ScoringVariant::MarginWeighted.apply(v, 4, -5, 0).raw(), 0);
        // A large product saturates into the i32 range rather than wrapping.
        assert_eq!(
            ScoringVariant::MarginWeighted
                .apply(ScoreQ::from_raw(i32::MAX), 1, i32::MAX, 1)
                .raw(),
            i32::MAX
        );
    }
}
