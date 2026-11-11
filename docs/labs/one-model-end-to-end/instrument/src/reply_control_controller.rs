//! Permeability-aware rule: after two kept tokens, a memory-named row may stop
//! at EOS rank 16; a reflective row may stop only at EOS rank 4.

use crate::{apply_flow, Decision, TraceRow};

pub fn control_reply(rows: &[TraceRow]) -> Vec<u32> {
    apply_flow(rows, true, |row, out| {
        let ceiling = match row.class.as_str() {
            "permeable" => 16,
            _ => 4,
        };
        if out.len() >= 2 && row.eos.rank <= ceiling {
            Decision::EmitEos
        } else {
            Decision::Keep
        }
    })
}
