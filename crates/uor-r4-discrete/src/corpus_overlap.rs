//! Counts how many n-token windows of one pinned stream also occur, exactly,
//! in another. Used to show an evaluation set is not inside a training slice.
//! Identity work only: no scores.
#![forbid(unsafe_code)]
use crate::corpus::UORT_HEADER_BYTES;
use crate::{Error, Result};
use serde_json::{json, Value};
use std::{collections::HashMap, fs, path::Path};

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(format!("corpus overlap: {}", message.into()))
}

fn ids(path: &Path) -> Result<Vec<u16>> {
    let bytes = fs::read(path).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
    if bytes.len() < UORT_HEADER_BYTES || &bytes[..4] != b"UORT" {
        return Err(invalid("needs a UORT stream"));
    }
    Ok(bytes[UORT_HEADER_BYTES..]
        .chunks_exact(2)
        .map(|w| u16::from_le_bytes([w[0], w[1]]))
        .collect())
}

/// Windows of `probe` (length `n`) found in `base`: distinct windows, distinct
/// windows found, and probe positions whose window is found.
pub fn count(base: &[u16], probe: &[u16], n: usize) -> Result<(u64, u64, u64)> {
    if n == 0 || probe.len() < n {
        return Err(invalid("window is longer than the probe stream"));
    }
    let mut windows: HashMap<&[u16], (u64, bool)> = HashMap::new();
    for window in probe.windows(n) {
        let (uses, _) = windows.entry(window).or_insert((0, false));
        *uses += 1;
    }
    for window in base.windows(n) {
        if let Some((_, found)) = windows.get_mut(window) {
            *found = true;
        }
    }
    let distinct = windows.len() as u64;
    let mut found_distinct = 0u64;
    let mut found_positions = 0u64;
    for (uses, found) in windows.values() {
        if *found {
            found_distinct += 1;
            found_positions += *uses;
        }
    }
    Ok((distinct, found_distinct, found_positions))
}

pub fn run(base: &Path, probe: &Path, n: usize) -> Result<Value> {
    let base_ids = ids(base)?;
    let probe_ids = ids(probe)?;
    let (distinct, found_distinct, found_positions) = count(&base_ids, &probe_ids, n)?;
    Ok(json!({
        "schema": "uor-r4.discrete-corpus-overlap/1",
        "window_tokens": n,
        "base_tokens": base_ids.len(),
        "probe_tokens": probe_ids.len(),
        "probe_windows": probe_ids.len() + 1 - n,
        "probe_distinct_windows": distinct,
        "distinct_windows_found_in_base": found_distinct,
        "probe_positions_found_in_base": found_positions,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_exact_shared_windows_only() {
        let base = [1u16, 2, 3, 4, 5, 6, 7];
        let probe = [9u16, 2, 3, 4, 8, 2, 3, 4, 5];
        assert_eq!(count(&base, &probe, 3).unwrap(), (6, 2, 3));
        assert_eq!(count(&base, &probe, 5).unwrap(), (5, 0, 0));
        assert!(count(&base, &probe[..2], 3).is_err());
    }
}
