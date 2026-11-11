//! Seeded random control gated by actual EOS evidence.

use crate::{apply_flow, Decision, TraceRow};

pub fn control_reply(rows: &[TraceRow]) -> Vec<u32> {
    apply_flow(rows, true, |row, out| {
        let mixed = (row.case as u64)
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .wrapping_add((out.len() as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9));
        if out.len() >= 2 && row.eos.rank < 8 && mixed % 32 == 0 {
            Decision::EmitEos
        } else {
            Decision::Keep
        }
    })
}
