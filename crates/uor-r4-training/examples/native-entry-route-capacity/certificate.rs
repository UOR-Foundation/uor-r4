//! Conservative fixed-Generate entry dominance, not a model-quality verdict.
//! The caller independently proves that target has no Copy alias and binds the
//! supplied full legal Generate scores/exp table to its authentic native step.
use serde::Serialize;

pub const CLIP_Q24: i64 = 8 << 24;
pub const INTERVAL_Q24: i64 = 1 << 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificateError {
    Dimensions,
    LegalIds,
    TargetNotLegal,
    Table,
    Alignment,
}
impl std::fmt::Display for CertificateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "entry dominance certificate: {self:?}")
    }
}
impl std::error::Error for CertificateError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DominanceProof {
    LowerIdMonotonic,
    HigherIdStrictInterpolation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DominanceCertificate {
    pub target_token_id: u32,
    pub competitor_token_id: u32,
    pub target_clipped_score_q24: i64,
    pub competitor_clipped_score_q24: i64,
    pub proof: DominanceProof,
    pub reference_min_q24: i64,
    pub reference_max_q24: i64,
    pub interpolation_interval_q24: i64,
    pub tested_intervals: usize,
    pub tested_reference_endpoints: usize,
    pub minimum_endpoint_weight_gap_q31: u64,
}

/// Scores are vocabulary-indexed; holes outside the strictly ascending legal
/// ID list are ignored. The table must be authenticated by the caller; this
/// helper checks numerical monotonicity/coverage but does not establish SHA
/// identity. Legal clipped scores must align to the native 2^16 Q24 grid.
///
/// A certificate proves target cannot be the native token winner for any
/// possible Copy scores, provided target has NO Copy aliases and Generate
/// scores stay fixed. Competitor Copy aliases only increase its token mass.
/// None means UNRESOLVED, never viability or impossibility.
///
/// The possible common reference is [max legal clipped Generate, +8 Q24].
/// All aligned scores give the same interpolation fraction on each reference
/// interval. The unrounded interpolants are affine there. If the integer
/// endpoint gap is >=1 at both endpoints, their unrounded gap is >=1 throughout;
/// native a-floor((a-b)*frac/2^16) is the ceiling of that interpolant, so its
/// integer gap remains >=1. This permits plateaus but never assumes strict LUT
/// monotonicity. Equal masses suffice only for a lower-ID competitor.
pub fn certify_generate_dominance(
    scores: &[i64],
    legal_ids: &[u32],
    target: u32,
    exp: &[u32],
) -> Result<Option<DominanceCertificate>, CertificateError> {
    if scores.is_empty() || legal_ids.is_empty() {
        return Err(CertificateError::Dimensions);
    }
    if legal_ids.windows(2).any(|w| w[0] >= w[1])
        || legal_ids.iter().any(|&id| id as usize >= scores.len())
    {
        return Err(CertificateError::LegalIds);
    }
    if legal_ids.binary_search(&target).is_err() {
        return Err(CertificateError::TargetNotLegal);
    }
    // Native interpolation reads BOTH endpoints, even at an aligned positive
    // gap with fraction zero. Maximum possible gap is 16 Q24 units.
    let last_required = ((2 * CLIP_Q24) / INTERVAL_Q24) as usize + 1;
    if exp.len() <= last_required
        || exp.windows(2).any(|w| w[0] < w[1])
        || exp[..=last_required].contains(&0)
    {
        return Err(CertificateError::Table);
    }
    let clipped = |id: u32| scores[id as usize].clamp(-CLIP_Q24, CLIP_Q24);
    if legal_ids.iter().any(|&id| clipped(id) % INTERVAL_Q24 != 0) {
        return Err(CertificateError::Alignment);
    }
    let reference_min = legal_ids
        .iter()
        .map(|&id| clipped(id))
        .max()
        .ok_or(CertificateError::Dimensions)?;
    let target_score = clipped(target);
    let intervals = ((CLIP_Q24 - reference_min) / INTERVAL_Q24) as usize;
    for &competitor in legal_ids {
        if competitor == target {
            continue;
        }
        let competitor_score = clipped(competitor);
        let lower = competitor < target;
        if competitor_score < target_score || (!lower && competitor_score == target_score) {
            continue;
        }
        let mut minimum_gap = u64::MAX;
        let mut strict = true;
        for step in 0..=intervals {
            let reference = reference_min + step as i64 * INTERVAL_Q24;
            let cw = u64::from(exp[((reference - competitor_score) / INTERVAL_Q24) as usize]);
            let tw = u64::from(exp[((reference - target_score) / INTERVAL_Q24) as usize]);
            let gap = cw.checked_sub(tw).ok_or(CertificateError::Table)?;
            minimum_gap = minimum_gap.min(gap);
            strict &= gap > 0;
        }
        if lower || strict {
            return Ok(Some(DominanceCertificate {
                target_token_id: target,
                competitor_token_id: competitor,
                target_clipped_score_q24: target_score,
                competitor_clipped_score_q24: competitor_score,
                proof: if lower {
                    DominanceProof::LowerIdMonotonic
                } else {
                    DominanceProof::HigherIdStrictInterpolation
                },
                reference_min_q24: reference_min,
                reference_max_q24: CLIP_Q24,
                interpolation_interval_q24: INTERVAL_Q24,
                tested_intervals: intervals,
                tested_reference_endpoints: intervals + 1,
                minimum_endpoint_weight_gap_q31: minimum_gap,
            }));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn descending() -> Vec<u32> {
        (0..8194).map(|i| 1_000_000 - i * 17).collect()
    }
    #[test]
    fn lower_id_tie_is_certificate_higher_id_tie_is_unresolved() -> Result<(), CertificateError> {
        let exp = descending();
        let c = certify_generate_dominance(&[0, 0], &[0, 1], 1, &exp)?
            .ok_or(CertificateError::Dimensions)?;
        assert_eq!(c.proof, DominanceProof::LowerIdMonotonic);
        assert_eq!(c.minimum_endpoint_weight_gap_q31, 0);
        assert!(certify_generate_dominance(&[0, 0], &[0, 1], 0, &exp)?.is_none());
        Ok(())
    }
    #[test]
    fn clipping_equalizes_and_sparse_holes_cannot_compete() -> Result<(), CertificateError> {
        let exp = descending();
        let scores = [CLIP_Q24, i64::MAX, CLIP_Q24 + INTERVAL_Q24];
        assert!(certify_generate_dominance(&scores, &[0, 2], 0, &exp)?.is_none());
        let c = certify_generate_dominance(&scores, &[0, 2], 2, &exp)?
            .ok_or(CertificateError::Dimensions)?;
        assert_eq!(c.competitor_token_id, 0);
        assert_eq!(c.reference_min_q24, CLIP_Q24);
        assert_eq!(c.tested_intervals, 0);
        assert_eq!(c.tested_reference_endpoints, 1);
        // Hole 1's huge score neither changes reference nor supplies a witness.
        assert!(
            certify_generate_dominance(&[0, i64::MAX, -INTERVAL_Q24], &[0, 2], 0, &exp)?.is_none()
        );
        Ok(())
    }
    #[test]
    fn plateaus_do_not_supply_higher_id_strict_proof() -> Result<(), CertificateError> {
        let exp = vec![10; 8194];
        assert!(certify_generate_dominance(&[0, INTERVAL_Q24], &[0, 1], 0, &exp)?.is_none());
        let mut mixed = descending();
        let plateau = mixed[2048];
        mixed[2048..].fill(plateau);
        // At maximum reference +8, target reads 2048 but competitor reads
        // 2047. A plateau starting at 2048 leaves their strict gap intact.
        assert!(certify_generate_dominance(&[0, INTERVAL_Q24], &[0, 1], 0, &mixed)?.is_some());
        // Include BOTH endpoints at that admissible reference. Their tie
        // prevents a higher-ID certificate even though earlier gaps are strict.
        let plateau = mixed[2047];
        mixed[2047..].fill(plateau);
        assert!(certify_generate_dominance(&[0, INTERVAL_Q24], &[0, 1], 0, &mixed)?.is_none());
        Ok(())
    }
    // The independent ceiling-of-rational form tests the native floor
    // interpolation equivalence, then exercises every fraction of an interval.
    fn native_weight(gap: i64, exp: &[u32]) -> u64 {
        if gap <= 0 {
            return u64::from(exp[0]);
        }
        let index = (gap >> 16) as usize;
        let fraction = (gap & 65535) as u64;
        let a = u64::from(exp[index]);
        let b = u64::from(exp[index + 1]);
        a - (((a - b) * fraction) >> 16)
    }
    #[test]
    fn aligned_strict_certificate_covers_native_floor_interiors() -> Result<(), CertificateError> {
        let exp = descending();
        let target = -3 * INTERVAL_Q24;
        let competitor = 2 * INTERVAL_Q24;
        let c = certify_generate_dominance(&[target, competitor], &[0, 1], 0, &exp)?
            .ok_or(CertificateError::Dimensions)?;
        assert_eq!(c.proof, DominanceProof::HigherIdStrictInterpolation);
        assert_eq!(c.minimum_endpoint_weight_gap_q31, 5 * 17);
        for base in [
            c.reference_min_q24,
            4 * INTERVAL_Q24,
            CLIP_Q24 - INTERVAL_Q24,
        ] {
            for fraction in 0..65536i64 {
                let reference = base + fraction;
                let cw = native_weight(reference - competitor, &exp);
                let tw = native_weight(reference - target, &exp);
                assert!(cw > tw);
                for gap in [reference - competitor, reference - target] {
                    let index = (gap >> 16) as usize;
                    let f = (gap & 65535) as u64;
                    let rational =
                        u64::from(exp[index]) * (65536 - f) + u64::from(exp[index + 1]) * f;
                    assert_eq!(native_weight(gap, &exp), rational.div_ceil(65536));
                }
            }
        }
        Ok(())
    }
    #[test]
    fn rejects_invalid_admission_without_claiming_viability() {
        let exp = descending();
        assert_eq!(
            certify_generate_dominance(&[0, 0], &[1, 0], 0, &exp),
            Err(CertificateError::LegalIds)
        );
        assert_eq!(
            certify_generate_dominance(&[0, 0], &[0, 0], 0, &exp),
            Err(CertificateError::LegalIds)
        );
        assert_eq!(
            certify_generate_dominance(&[0], &[0], 1, &exp),
            Err(CertificateError::TargetNotLegal)
        );
        assert_eq!(
            certify_generate_dominance(&[1, 0], &[0, 1], 0, &exp),
            Err(CertificateError::Alignment)
        );
        assert_eq!(
            certify_generate_dominance(&[0], &[0], 0, &[1]),
            Err(CertificateError::Table)
        );
        let mut invalid = exp;
        invalid[1] = invalid[0] + 1;
        assert_eq!(
            certify_generate_dominance(&[0], &[0], 0, &invalid),
            Err(CertificateError::Table)
        );
    }
}
