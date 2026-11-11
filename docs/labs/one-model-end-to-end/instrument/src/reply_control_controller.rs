//! Shuffled-feature control: use a deterministic other row's EOS rank in the
//! fixed-rank rule.

use crate::{apply_flow, Decision, TraceRow};

pub fn control_reply(rows: &[TraceRow]) -> Vec<u32> {
    if rows.is_empty() {
        return Vec::new();
    }
    apply_flow(rows, true, |row, out| {
        let source = (out.len() * 7 + row.case) % rows.len();
        if out.len() >= 2 && rows[source].eos.rank <= 2 {
            Decision::EmitEos
        } else {
            Decision::Keep
        }
    })
}
