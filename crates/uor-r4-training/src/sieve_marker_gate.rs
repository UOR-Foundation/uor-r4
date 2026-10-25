//! A discourse-marker gate over the prime-sieve frame compiler
//! (`m3-memory-turn-discriminator-gate`).
//!
//! A marker is the cofactor one training frame leaves when it is sieved by
//! another training frame of the same relation: `f = m ++ g` on the words
//! before the slot, with the same words after it, so `sig(f) = sig(m) ·
//! sig(g)`. "Actually, my hometown is {v}." sieved by "My hometown is {v}."
//! leaves the marker "actually". Markers are found only in the training
//! phrasings, and they are relation-independent: a marker found on one
//! relation's frames may precede any relation's slot frame.
//!
//! A turn compiles through the gate when it is at most [`MAX_MARKERS`]
//! markers followed by an exact training slot frame and its value, or an
//! exact frame alone (the sieve compiler's own reading). The margin gate is
//! exact explanation: every word outside the value is a marker word or a
//! frame word, and every surviving reading names one expert and one span.
//! The decision runs on the GPU (`sieve_frame.cu`, `sieve_frame_compile_marked`);
//! this module prepares data and decodes stored decisions.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::sieve_frame_compiler::{Act, FrameTable, Word};
use crate::turn_action::{CompiledAction, SourceSpan};
use crate::{invalid, sha256_bytes, Result};

/// The marker table's schema.
pub const MARKER_TABLE_SCHEMA: &str = "uor-r4.sieve-marker-table/1";
/// The most markers a turn may carry before its frame.
pub const MAX_MARKERS: u32 = 2;
/// The SHA-256 of the world=v2 marker table's bytes, registered before the
/// gated run; its test fails if the frames, the rule or the layout change.
pub const WORLD_V2_MARKER_TABLE_SHA256: &str =
    "cd79399490c0abcfc14253b5680a282fc840672c5932e925d62a3b95375a125d";

/// One discourse marker: lexicon word IDs, and whether it turns an assert
/// frame into a correction.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Marker {
    pub words: Vec<u32>,
    pub update: bool,
}

/// The markers found in a frame table, bound to that table by digest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkerTable {
    pub schema: String,
    pub frame_table_sha256: String,
    pub max_markers: u32,
    pub markers: Vec<Marker>,
}

impl MarkerTable {
    /// Every marker the training frames imply. A sequence is an update
    /// marker when it only ever turns an assert frame into an update frame
    /// (or sits between two update frames), and neutral when it never
    /// changes the act. A sequence that would erase a correction, or that
    /// plays both roles, is left out.
    pub fn derive(table: &FrameTable) -> Result<Self> {
        let mut pairs: BTreeMap<Vec<u32>, BTreeSet<(Act, Act)>> = BTreeMap::new();
        let slots = || table.frames.iter().filter(|f| f.slot);
        for f in slots() {
            for g in slots().filter(|g| g.relation == f.relation && g.suffix == f.suffix) {
                if f.prefix.len() <= g.prefix.len() {
                    continue;
                }
                let cut = f.prefix.len() - g.prefix.len();
                if f.prefix[cut..] == g.prefix[..] {
                    pairs
                        .entry(f.prefix[..cut].to_vec())
                        .or_default()
                        .insert((f.act, g.act));
                }
            }
        }
        let mut markers = Vec::new();
        for (words, kinds) in pairs {
            let erases = kinds.contains(&(Act::Assert, Act::Update));
            let converts = kinds.contains(&(Act::Update, Act::Assert));
            let keeps_assert = kinds.contains(&(Act::Assert, Act::Assert));
            if erases || (converts && keeps_assert) {
                continue;
            }
            markers.push(Marker {
                words,
                update: converts,
            });
        }
        Ok(Self {
            schema: MARKER_TABLE_SCHEMA.into(),
            frame_table_sha256: table.identity()?.artifact_sha256,
            max_markers: MAX_MARKERS,
            markers,
        })
    }

    /// The marker table's bytes: its JSON, in field and table order.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| invalid(format!("marker table JSON: {e}")))
    }

    pub fn sha256(&self) -> Result<String> {
        Ok(sha256_bytes(&self.canonical_bytes()?))
    }

    /// The markers in the flat layout the kernel reads.
    pub fn device_markers(&self) -> Result<DeviceMarkers> {
        let mut out = DeviceMarkers::default();
        for marker in &self.markers {
            out.offset.push(len_u32(out.tokens.len())?);
            out.length.push(len_u32(marker.words.len())?);
            out.update.push(u32::from(marker.update));
            out.tokens.extend(&marker.words);
        }
        Ok(out)
    }
}

fn len_u32(len: usize) -> Result<u32> {
    u32::try_from(len).map_err(|_| invalid("device array too large"))
}

/// Markers in the kernel's flat layout.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceMarkers {
    pub tokens: Vec<u32>,
    pub offset: Vec<u32>,
    pub length: Vec<u32>,
    pub update: Vec<u32>,
}

/// Act codes the gated kernel reports.
pub const ACT_ASSERT: u32 = 0;
pub const ACT_UPDATE: u32 = 1;
pub const ACT_QUERY: u32 = 2;

/// One turn's stored decision from the gated kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatedDecision {
    pub status: u32,
    pub frame: u32,
    pub act: u32,
    pub hole_start: u32,
    pub hole_len: u32,
    pub survivors: u32,
}

/// The action a stored gated decision stands for. The act comes from the
/// kernel (a marker can turn an assert frame into a correction); the
/// relation comes from the base frame.
pub fn decode_gated(
    table: &FrameTable,
    turn: &[Word],
    decision: GatedDecision,
) -> Result<CompiledAction> {
    use crate::sieve_frame_compiler::{STATUS_COMPILED, STATUS_DISAGREE, STATUS_NONE};
    match decision.status {
        STATUS_NONE => {
            return Ok(CompiledAction::Unresolved {
                reason: "no frame survives the markers, sieve and alignment".into(),
            })
        }
        STATUS_DISAGREE => {
            return Ok(CompiledAction::Unresolved {
                reason: "the surviving readings disagree".into(),
            })
        }
        STATUS_COMPILED => {}
        other => return Err(invalid(format!("unknown decision status {other}"))),
    }
    let frame = table
        .frames
        .get(decision.frame as usize)
        .ok_or_else(|| invalid("decision names a frame outside the table"))?;
    let relation = frame.relation;
    match (decision.act, frame.act) {
        (ACT_QUERY, Act::Query) => return Ok(CompiledAction::QueryCurrent { relation }),
        (ACT_ASSERT, Act::Assert) | (ACT_UPDATE, Act::Assert | Act::Update) => {}
        _ => return Err(invalid("decision act does not fit its frame")),
    }
    let first = decision.hole_start as usize;
    let last = (first + decision.hole_len as usize)
        .checked_sub(1)
        .filter(|_| decision.hole_len > 0)
        .ok_or_else(|| invalid("a compiled slot decision has an empty value"))?;
    let (Some(head), Some(tail)) = (turn.get(first), turn.get(last)) else {
        return Err(invalid("decision value lies outside the turn"));
    };
    let span = SourceSpan {
        start: head.start,
        end: tail.end,
    };
    Ok(if decision.act == ACT_UPDATE {
        CompiledAction::Correct { relation, span }
    } else {
        CompiledAction::Assert { relation, span }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sieve_frame_compiler::words;

    fn texts(table: &FrameTable, marker: &Marker) -> String {
        marker
            .words
            .iter()
            .map(|&id| table.lexicon[id as usize - 1].as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn markers_are_cofactors_of_training_frames() -> Result<()> {
        let table = FrameTable::world_v2()?;
        let markers = MarkerTable::derive(&table)?;
        assert!(
            !markers.markers.is_empty(),
            "the training frames imply no marker"
        );
        for marker in &markers.markers {
            assert!(!marker.words.is_empty());
            // Some training frame is exactly this marker followed by another.
            let witnessed = table.frames.iter().filter(|f| f.slot).any(|f| {
                table.frames.iter().filter(|g| g.slot).any(|g| {
                    g.relation == f.relation
                        && g.suffix == f.suffix
                        && f.prefix.len() == marker.words.len() + g.prefix.len()
                        && f.prefix[..marker.words.len()] == marker.words[..]
                        && f.prefix[marker.words.len()..] == g.prefix[..]
                })
            });
            assert!(witnessed, "{}", texts(&table, marker));
        }
        let listed: Vec<String> = markers
            .markers
            .iter()
            .map(|m| {
                format!(
                    "{}{}",
                    texts(&table, m),
                    if m.update { " [update]" } else { "" }
                )
            })
            .collect();
        // The registered marker table, found by the rule above from the training
        // phrasings alone (no development phrasing or panel row was read).
        assert_eq!(
            listed,
            [
                "actually [update]",
                "people",
                "please note that",
                "remember",
                "sorry [update]",
                "sorry i meant [update]"
            ]
        );
        assert_eq!(markers.sha256()?, WORLD_V2_MARKER_TABLE_SHA256);
        Ok(())
    }

    #[test]
    fn decode_gated_takes_the_act_from_the_kernel() -> Result<()> {
        let table = FrameTable::world_v2()?;
        let assert_frame = table
            .frames
            .iter()
            .position(|f| f.act == Act::Assert && f.relation == 4)
            .expect("frame") as u32;
        let turn = words("Actually, my hometown is Rivertown.");
        let d = |act, hole_start, hole_len| GatedDecision {
            status: 1,
            frame: assert_frame,
            act,
            hole_start,
            hole_len,
            survivors: 1,
        };
        let span = SourceSpan {
            start: turn[4].start,
            end: turn[4].end,
        };
        assert_eq!(
            decode_gated(&table, &turn, d(ACT_UPDATE, 4, 1))?,
            CompiledAction::Correct { relation: 4, span }
        );
        assert_eq!(
            decode_gated(&table, &turn, d(ACT_ASSERT, 4, 1))?,
            CompiledAction::Assert { relation: 4, span }
        );
        assert!(decode_gated(&table, &turn, d(ACT_QUERY, 0, 0)).is_err());
        assert!(decode_gated(&table, &turn, d(ACT_UPDATE, 4, 0)).is_err());
        Ok(())
    }
}
