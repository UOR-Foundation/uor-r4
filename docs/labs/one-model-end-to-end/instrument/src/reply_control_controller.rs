//! Stability/tension rule: after two kept tokens, treat a low nearest-row
//! margin and an EOS rank in the top eight as terminal tension.

use crate::{apply_flow, Decision, TraceRow};

pub fn control_reply(rows: &[TraceRow]) -> Vec<u32> {
    apply_flow(rows, true, |row, out| {
        if out.len() >= 2 && row.nearest.margin == 0 && row.eos.rank <= 8 {
            Decision::EmitEos
        } else {
            Decision::Keep
        }
    })
}
