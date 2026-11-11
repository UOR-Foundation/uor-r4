//! Fixed EOS rank threshold: after two kept tokens, emit EOS when EOS is
//! ranked first or second in the writer table.

use crate::{apply_flow, Decision, TraceRow};

pub fn control_reply(rows: &[TraceRow]) -> Vec<u32> {
    apply_flow(rows, true, |row, out| {
        if out.len() >= 2 && row.eos.rank <= 2 {
            Decision::EmitEos
        } else {
            Decision::Keep
        }
    })
}
