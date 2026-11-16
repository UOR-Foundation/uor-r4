//! The one-model-end-to-end verdict rule, executable (item Done-when 3).
//!
//! The item fixes the rule in advance: a result "memorized" when the gains appear **only on keyed
//! rows** and generated replies are mostly TRAIN replies, and "generalizes" when it **beats (r0)
//! and the longest-suffix table on unkeyed and new-template rows**, with novelty near DEV's own
//! replies. This module encodes that rule and nothing else: it measures nothing, and it reports
//! `Undetermined` rather than guessing when a reading the rule needs is absent.
//!
//! Two knobs are the protocol sheet's to fix, not this module's: what "mostly" means (passed as
//! `mostly_training_ppm`, defaulting to one half) and how near DEV's novelty must be (the band, in
//! parts per million). A caller that has not fixed them is not ready to read a verdict.

use serde::{Deserialize, Serialize};

/// One stratum's teacher-forced reading: the rows and how many the arm got right.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stratum {
    /// Rows in the stratum.
    pub targets: u64,
    /// Rows whose top-1 is right.
    pub correct: u64,
}

impl Stratum {
    /// `self` is strictly above `other` on this stratum, by cross-multiplication.
    fn above(&self, other: &Stratum) -> bool {
        u128::from(self.correct) * u128::from(other.targets)
            > u128::from(other.correct) * u128::from(self.targets)
    }
}

/// The three strata the protocol names.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmStrata {
    pub keyed: Stratum,
    pub unkeyed: Stratum,
    pub new_template: Stratum,
}

/// One arm's readings across the strata: the candidate, the memory-off control (r0), and the
/// longest-suffix table over TRAIN.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerdictInput {
    /// The arm under test, e.g. (r).
    pub arm: ArmStrata,
    /// The same trunk with its memory off (r0).
    pub memory_off: ArmStrata,
    /// The longest-suffix table over TRAIN, the lawful control.
    pub table: ArmStrata,
    /// Generated replies that are verbatim TRAIN replies.
    pub verbatim_train_replies: u64,
    /// Generated replies scored.
    pub replies: u64,
    /// The arm's new n-gram share (n = 4..8), in parts per million.
    pub novelty_arm_ppm: Option<u64>,
    /// DEV's own replies' new n-gram share, in parts per million.
    pub novelty_dev_ppm: Option<u64>,
    /// The band the sheet fixes: how far below DEV's novelty may fall and still count as near.
    #[serde(default)]
    pub novelty_band_ppm: u64,
    /// The sheet's "mostly": the share of verbatim TRAIN replies at or above which replies are
    /// mostly TRAIN replies. Defaults to one half.
    #[serde(default = "default_mostly")]
    pub mostly_training_ppm: u64,
}

fn default_mostly() -> u64 {
    500_000
}

/// The rule's answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    /// Gains only on keyed rows, and mostly TRAIN replies.
    Memorized,
    /// Beats both controls on unkeyed and new-template rows, with novelty near DEV's.
    Generalizes,
    /// Neither reading holds.
    Mixed,
    /// A reading the rule needs is absent; the reason names it.
    Undetermined { reason: String },
}

/// The verdict with the readings it was drawn from, so a record can be audited.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct VerdictReport {
    pub verdict: Verdict,
    pub arm: ArmStrata,
    pub memory_off: ArmStrata,
    pub table: ArmStrata,
    pub verbatim_training_ppm: u64,
    pub novelty_arm_ppm: Option<u64>,
    pub novelty_dev_ppm: Option<u64>,
    pub mostly_training_ppm: u64,
    pub novelty_band_ppm: u64,
    pub gains: Gains,
}

/// Which comparisons moved, so the reasons are visible beside the verdict.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Gains {
    pub keyed: bool,
    pub unkeyed: bool,
    pub new_template: bool,
}

/// Parts per million, floored; `0` when the whole is zero.
pub fn share_ppm(part: u64, whole: u64) -> u64 {
    if whole == 0 {
        return 0;
    }
    (u128::from(part) * 1_000_000 / u128::from(whole)) as u64
}

/// Apply the pre-registered rule to one input.
pub fn verdict(input: &VerdictInput) -> VerdictReport {
    let gains = Gains {
        keyed: input.arm.keyed.above(&input.memory_off.keyed)
            && input.arm.keyed.above(&input.table.keyed),
        unkeyed: input.arm.unkeyed.above(&input.memory_off.unkeyed)
            && input.arm.unkeyed.above(&input.table.unkeyed),
        new_template: input.arm.new_template.above(&input.memory_off.new_template)
            && input.arm.new_template.above(&input.table.new_template),
    };
    let verbatim_training_ppm = share_ppm(input.verbatim_train_replies, input.replies);
    let empty = |stratum: Stratum| stratum.targets == 0;
    let missing = empty(input.arm.keyed)
        || empty(input.arm.unkeyed)
        || empty(input.arm.new_template)
        || empty(input.memory_off.keyed)
        || empty(input.memory_off.unkeyed)
        || empty(input.memory_off.new_template)
        || empty(input.table.keyed)
        || empty(input.table.unkeyed)
        || empty(input.table.new_template);
    let verdict = if missing {
        Verdict::Undetermined {
            reason: "a stratum has no rows in one of the arms".to_owned(),
        }
    } else if gains.unkeyed && gains.new_template {
        // Generalizes needs novelty near DEV's; without DEV's reading the rule cannot decide
        // between Generalizes and Mixed, and must not choose.
        match (input.novelty_arm_ppm, input.novelty_dev_ppm) {
            (Some(arm), Some(dev)) if arm.saturating_add(input.novelty_band_ppm) >= dev => {
                Verdict::Generalizes
            }
            (Some(_), Some(_)) => Verdict::Mixed,
            _ => Verdict::Undetermined {
                reason: "novelty is missing; Generalizes cannot be read without DEV's share"
                    .to_owned(),
            },
        }
    } else if gains.keyed
        && !gains.unkeyed
        && !gains.new_template
        && verbatim_training_ppm >= input.mostly_training_ppm
    {
        Verdict::Memorized
    } else {
        Verdict::Mixed
    };
    VerdictReport {
        verdict,
        arm: input.arm,
        memory_off: input.memory_off,
        table: input.table,
        verbatim_training_ppm,
        novelty_arm_ppm: input.novelty_arm_ppm,
        novelty_dev_ppm: input.novelty_dev_ppm,
        mostly_training_ppm: input.mostly_training_ppm,
        novelty_band_ppm: input.novelty_band_ppm,
        gains,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stratum(correct: u64) -> Stratum {
        Stratum {
            targets: 100,
            correct,
        }
    }

    fn strata(keyed: u64, unkeyed: u64, new_template: u64) -> ArmStrata {
        ArmStrata {
            keyed: stratum(keyed),
            unkeyed: stratum(unkeyed),
            new_template: stratum(new_template),
        }
    }

    fn input(arm: ArmStrata, memory_off: ArmStrata, table: ArmStrata) -> VerdictInput {
        VerdictInput {
            arm,
            memory_off,
            table,
            verbatim_train_replies: 80,
            replies: 100,
            novelty_arm_ppm: Some(400_000),
            novelty_dev_ppm: Some(420_000),
            novelty_band_ppm: 50_000,
            mostly_training_ppm: default_mostly(),
        }
    }

    #[test]
    fn keyed_only_gains_with_training_replies_read_memorized() {
        let report = verdict(&input(
            strata(90, 40, 40),
            strata(50, 40, 40),
            strata(50, 40, 40),
        ));
        assert_eq!(report.verdict, Verdict::Memorized);
        assert_eq!(
            report.gains,
            Gains {
                keyed: true,
                unkeyed: false,
                new_template: false
            }
        );
    }

    #[test]
    fn gains_over_both_controls_on_the_fresh_strata_with_near_novelty_read_generalizes() {
        let report = verdict(&input(
            strata(90, 80, 80),
            strata(50, 40, 40),
            strata(55, 45, 45),
        ));
        assert_eq!(report.verdict, Verdict::Generalizes);
        assert!(report.gains.unkeyed && report.gains.new_template);
    }

    #[test]
    fn fresh_gains_with_far_below_dev_novelty_read_mixed() {
        let mut input = input(
            strata(90, 80, 80),
            strata(50, 40, 40),
            strata(55, 45, 45),
        );
        input.novelty_arm_ppm = Some(100_000);
        assert_eq!(verdict(&input).verdict, Verdict::Mixed);
    }

    #[test]
    fn a_missing_novelty_reading_refuses_to_choose_between_generalizes_and_mixed() {
        let mut input = input(
            strata(90, 80, 80),
            strata(50, 40, 40),
            strata(55, 45, 45),
        );
        input.novelty_dev_ppm = None;
        match verdict(&input).verdict {
            Verdict::Undetermined { reason } => assert!(reason.contains("novelty"), "{reason}"),
            other => panic!("expected Undetermined, got {other:?}"),
        }
    }

    #[test]
    fn no_gains_is_mixed_however_training_the_replies() {
        let report = verdict(&input(
            strata(50, 40, 40),
            strata(50, 40, 40),
            strata(50, 40, 40),
        ));
        assert_eq!(report.verdict, Verdict::Mixed);
    }

    #[test]
    fn keyed_gains_with_mostly_fresh_replies_is_mixed_not_memorized() {
        let mut input = input(
            strata(90, 40, 40),
            strata(50, 40, 40),
            strata(50, 40, 40),
        );
        input.verbatim_train_replies = 20;
        assert_eq!(verdict(&input).verdict, Verdict::Mixed);
    }

    #[test]
    fn an_empty_stratum_is_undetermined_rather_than_a_verdict() {
        let mut input = input(
            strata(90, 80, 80),
            strata(50, 40, 40),
            strata(55, 45, 45),
        );
        input.arm.new_template = Stratum {
            targets: 0,
            correct: 0,
        };
        match verdict(&input).verdict {
            Verdict::Undetermined { reason } => assert!(reason.contains("no rows"), "{reason}"),
            other => panic!("expected Undetermined, got {other:?}"),
        }
    }

    #[test]
    fn the_shares_are_integer_and_a_zero_whole_is_zero() {
        assert_eq!(share_ppm(1, 3), 333_333);
        assert_eq!(share_ppm(3, 3), 1_000_000);
        assert_eq!(share_ppm(1, 0), 0);
    }
}
