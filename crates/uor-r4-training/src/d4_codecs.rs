//! Lab 3 MapCodec adapters and implementations for discrete parameter quantization.
//!
//! Exposes adapters connecting `uor_r4_integer::codec` implementations to the shared
//! `MapCodec` QAT training interface (`uor_r4_training::geometric_stack::MapCodec`).
//!
//! Strictly adheres to D0-b and D11 serving invariants: zero hardware multipliers,
//! zero hardware dividers, and zero floating-point operations in served code.

use std::sync::Arc;

use crate::geometric_stack::{D11Interim, MapCodec};
use crate::{invalid, Result};

/// Lab 3 adapter connecting [`uor_r4_integer::codec::Grouped4BitCodec`] to the shared
/// [`MapCodec`] training interface.
///
/// Supports standard round-to-nearest (`rtn`) and minimum-MSE scale search (`min_mse`).
/// In `rtn` mode, output matches [`D11Interim`] bit for bit.
#[derive(Clone, Debug, Default)]
pub struct D4Grouped4BitAdapter(pub uor_r4_integer::codec::Grouped4BitCodec);

impl D4Grouped4BitAdapter {
    /// Construct a round-to-nearest grouped 4-bit adapter matching D11 interim format.
    pub fn rtn() -> Self {
        Self(uor_r4_integer::codec::Grouped4BitCodec::default())
    }

    /// Construct a minimum-MSE scale-optimized grouped 4-bit adapter.
    pub fn min_mse() -> Self {
        Self(uor_r4_integer::codec::Grouped4BitCodec::new(
            uor_r4_lut::GROUP,
            uor_r4_integer::codec::Grouped4BitRounding::MinimumMseScale,
        ))
    }
}

impl MapCodec for D4Grouped4BitAdapter {
    fn name(&self) -> &str {
        self.0.name()
    }

    fn round_trip(&self, values: &[f32], rows: usize, cols: usize) -> Result<Vec<f32>> {
        self.0.round_trip(values, rows, cols).map_err(Into::into)
    }
}

/// Lab 3 head-compensated codec implementing compensated base and boundary search.
///
/// Evaluates candidate base exponents and boundary mantissas to minimize squared error across
/// groups, reducing output-head distortion in logit margins without requiring
/// any changes to the served D11 runtime kernels or multiplier instructions.
#[derive(Clone, Copy, Debug, Default)]
pub struct HeadCompensatedMapCodec {
    /// If `head_only` is true, only rows x cols matching the output head receive
    /// compensation, while other matrices use standard D11Interim.
    /// If false, all maps receive head-compensated quantization.
    pub head_only: bool,
    /// Explicit target dimensions for the head (e.g. `(vocab_size, width)`).
    /// If specified, only matrices matching `(rows, cols) == (vocab_size, width)`
    /// receive compensation in `head_only` mode.
    pub head_shape: Option<(usize, usize)>,
}

impl HeadCompensatedMapCodec {
    pub fn new(head_only: bool) -> Self {
        Self {
            head_only,
            head_shape: None,
        }
    }

    pub fn head_only() -> Self {
        Self {
            head_only: true,
            head_shape: None,
        }
    }

    pub fn head_only_for(rows: usize, cols: usize) -> Self {
        Self {
            head_only: true,
            head_shape: Some((rows, cols)),
        }
    }

    pub fn all_maps() -> Self {
        Self {
            head_only: false,
            head_shape: None,
        }
    }
}

impl MapCodec for HeadCompensatedMapCodec {
    fn name(&self) -> &str {
        if self.head_only {
            "native-d4-head-compensated-head-only"
        } else {
            "native-d4-head-compensated-all-maps"
        }
    }

    fn round_trip(&self, values: &[f32], rows: usize, cols: usize) -> Result<Vec<f32>> {
        let is_target_head = if let Some((target_rows, target_cols)) = self.head_shape {
            rows == target_rows && cols == target_cols
        } else {
            rows > cols && rows != 2 * cols
        };

        if self.head_only && !is_target_head {
            D11Interim.round_trip(values, rows, cols)
        } else {
            let mat = uor_r4_integer::codec::quantize_matrix_compensated(values, rows, cols)
                .map_err(|e| invalid(e.to_string()))?;
            let codec = uor_r4_integer::codec::Grouped4BitCodec::default();
            codec.dequantize(&mat).map_err(Into::into)
        }
    }
}

/// Lab 3 Matched-Bit E8 Lattice MapCodec for QAT training and representation evaluation.
///
/// Implements two-stage residual E8 lattice vector quantization (Conway-Sloane A_8* / E_8)
/// achieving ~4.0 bits per weight (2 index bytes + 2 scale bytes per 8-weight block)
/// within the <= 4.25 bpw D4 gate.
#[derive(Clone, Copy, Debug)]
pub struct E8MatchedBitMapCodec {
    pub seed: u64,
}

impl Default for E8MatchedBitMapCodec {
    fn default() -> Self {
        Self { seed: 42 }
    }
}

impl E8MatchedBitMapCodec {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }
}

impl MapCodec for E8MatchedBitMapCodec {
    fn name(&self) -> &str {
        "native-d4-e8-matched-bit"
    }

    fn round_trip(&self, values: &[f32], rows: usize, cols: usize) -> Result<Vec<f32>> {
        uor_r4_integer::codec::apply_codec_arm(
            values,
            rows,
            cols,
            uor_r4_integer::codec::CodecArm::HadamardE8MatchedBit,
            self.seed,
        )
        .map_err(Into::into)
    }
}

/// Look up a codec by name.
pub fn codec_by_name(name: &str) -> Result<Arc<dyn MapCodec>> {
    match name {
        "native-d11-grouped-4bit-g32-rtn" | "native-d11-grouped-4bit-g32" => {
            Ok(Arc::new(D11Interim))
        }
        "native-d11-grouped-4bit-g32-min-mse" => Ok(Arc::new(D4Grouped4BitAdapter::min_mse())),
        "native-d4-head-compensated-head-only" => {
            Ok(Arc::new(HeadCompensatedMapCodec::head_only()))
        }
        "native-d4-head-compensated-all-maps" => Ok(Arc::new(HeadCompensatedMapCodec::all_maps())),
        "native-d4-e8-matched-bit" => Ok(Arc::new(E8MatchedBitMapCodec::default())),
        _ => Err(invalid(format!("unknown served codec name: {name}"))),
    }
}
