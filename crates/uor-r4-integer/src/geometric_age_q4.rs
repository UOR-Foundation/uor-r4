//! Four-bit independent temporal biases in fixed quarter-nat units.
//!
//! Admission regenerates head-major [head, lag] Q24 entries. Lag zero is
//! retained; no slope, bucketing, centering or semantic metric is introduced.
//! This codec changes coefficient storage, not causal support or normalization.
use crate::geometric_no_read::{unpack_coefficients, NoReadError};
use serde::{Deserialize, Serialize};
use std::fmt;

pub const POLICY: &str = "signed-q4[-7,7];reserved-minus8;independent-head-lag;fixed-quarter-nat;exact-shift22;lag0-retained;no-centering/1";
pub const SCHEMA: &str = "uor-r4.geometric-age-q4/1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgeQ4Config {
    pub heads: usize,
    pub context: usize,
}
impl AgeQ4Config {
    pub fn validate(self) -> Result<(), AgeQ4Error> {
        if !(1..=2).contains(&self.heads) || !(1..=128).contains(&self.context) {
            return Err(AgeQ4Error::Configuration);
        }
        Ok(())
    }
    pub fn coefficient_count(self) -> Result<usize, AgeQ4Error> {
        self.validate()?;
        Ok(self.heads * self.context)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgeQ4Error {
    Configuration,
    Packed(NoReadError),
}
impl fmt::Display for AgeQ4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geometric age q4: {self:?}")
    }
}
impl std::error::Error for AgeQ4Error {}

pub struct NativeAgeQ4 {
    config: AgeQ4Config,
    packed: Vec<u8>,
    age_q24: Vec<i64>,
}
impl NativeAgeQ4 {
    pub fn new(config: AgeQ4Config, packed: &[u8]) -> Result<Self, AgeQ4Error> {
        let coefficients =
            unpack_coefficients(packed, config.coefficient_count()?).map_err(AgeQ4Error::Packed)?;
        Ok(Self {
            config,
            packed: packed.to_vec(),
            age_q24: coefficients
                .into_iter()
                .map(|q| i64::from(q) << 22)
                .collect(),
        })
    }
    pub fn config(&self) -> AgeQ4Config {
        self.config
    }
    pub fn packed(&self) -> &[u8] {
        &self.packed
    }
    pub fn age_q24(&self) -> &[i64] {
        &self.age_q24
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_no_read::pack_coefficients;
    type TestResult = Result<(), Box<dyn std::error::Error>>;
    #[test]
    fn age_q4_exact_units_head_lag_zero_and_bounds() -> TestResult {
        let coefficients = [-7, -1, 0, 1, 7, 2];
        let codec = NativeAgeQ4::new(
            AgeQ4Config {
                heads: 2,
                context: 3,
            },
            &pack_coefficients(&coefficients)?,
        )?;
        assert_eq!(
            codec.age_q24(),
            &[-29360128, -4194304, 0, 4194304, 29360128, 8388608]
        );
        assert_eq!(codec.config().coefficient_count()?, coefficients.len());
        assert_eq!(codec.age_q24()[0], -7i64 << 22);
        assert_eq!(codec.age_q24()[3], 1i64 << 22);
        Ok(())
    }
    #[test]
    fn age_q4_rejects_reserved_padding_shape_and_length() {
        let config = AgeQ4Config {
            heads: 1,
            context: 1,
        };
        assert!(NativeAgeQ4::new(config, &[8]).is_err());
        assert!(NativeAgeQ4::new(config, &[0x10]).is_err());
        assert!(NativeAgeQ4::new(config, &[]).is_err());
        for config in [
            AgeQ4Config {
                heads: 0,
                context: 1,
            },
            AgeQ4Config {
                heads: 3,
                context: 1,
            },
            AgeQ4Config {
                heads: 1,
                context: 0,
            },
            AgeQ4Config {
                heads: 1,
                context: 129,
            },
        ] {
            assert!(NativeAgeQ4::new(config, &[]).is_err());
        }
    }
}
