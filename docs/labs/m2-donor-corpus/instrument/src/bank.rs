//! The prompt bank (claude-400): user prompts drawn from licensed sources, guarded
//! against the project's panels, deduplicated and seed-shuffled. Not yet built.

use std::collections::BTreeMap;

pub fn cmd_bank(_m: &BTreeMap<String, String>) -> Result<(), String> {
    Err("bank: not built yet".into())
}
