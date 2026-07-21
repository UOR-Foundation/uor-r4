use core::ops::Range;

/// Fixed-point score used by reference frontier entries.
pub type ScoreQ = i32;

/// One active region tracked during bounded refinement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActiveFrontierEntry {
    pub region_id: u32,
    pub score_q: ScoreQ,
    pub margin: i16,
    pub depth: u8,
}

/// Fixed-capacity active frontier (caller-owned, no heap allocation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveFrontier<const CAP: usize> {
    len: usize,
    entries: [ActiveFrontierEntry; CAP],
}

impl<const CAP: usize> Default for ActiveFrontier<CAP> {
    fn default() -> Self {
        Self {
            len: 0,
            entries: [ActiveFrontierEntry::default(); CAP],
        }
    }
}

impl<const CAP: usize> ActiveFrontier<CAP> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[ActiveFrontierEntry] {
        &self.entries[..self.len]
    }

    pub fn clear(&mut self) {
        for entry in &mut self.entries[..self.len] {
            *entry = ActiveFrontierEntry::default();
        }
        self.len = 0;
    }

    pub fn push(&mut self, entry: ActiveFrontierEntry) -> bool {
        if self.len == CAP {
            return false;
        }
        self.entries[self.len] = entry;
        self.len += 1;
        true
    }
}

/// Packed section-relative edge ranges for refinement and overlap edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PackedEdgeRanges {
    pub refinement_start: u32,
    pub refinement_len: u32,
    pub overlap_start: u32,
    pub overlap_len: u32,
}

impl PackedEdgeRanges {
    pub fn refinement_range(self, edge_count: usize) -> Option<Range<usize>> {
        resolve_range(self.refinement_start, self.refinement_len, edge_count)
    }

    pub fn overlap_range(self, edge_count: usize) -> Option<Range<usize>> {
        resolve_range(self.overlap_start, self.overlap_len, edge_count)
    }
}

fn resolve_range(start: u32, len: u32, edge_count: usize) -> Option<Range<usize>> {
    let start = usize::try_from(start).ok()?;
    let len = usize::try_from(len).ok()?;
    let end = start.checked_add(len)?;
    if end > edge_count {
        return None;
    }
    Some(start..end)
}

#[cfg(test)]
mod tests {
    use super::{ActiveFrontier, ActiveFrontierEntry, PackedEdgeRanges};

    #[test]
    fn active_frontier_enforces_capacity() {
        let mut frontier = ActiveFrontier::<2>::default();
        assert!(frontier.push(ActiveFrontierEntry {
            region_id: 3,
            score_q: 42,
            margin: 7,
            depth: 1,
        }));
        assert!(frontier.push(ActiveFrontierEntry {
            region_id: 5,
            score_q: 21,
            margin: -3,
            depth: 2,
        }));
        assert!(!frontier.push(ActiveFrontierEntry::default()));
        assert_eq!(frontier.len(), 2);
        assert_eq!(frontier.as_slice()[0].region_id, 3);
        assert_eq!(frontier.as_slice()[1].region_id, 5);
    }

    #[test]
    fn packed_edge_ranges_resolve_checked() {
        let packed = PackedEdgeRanges {
            refinement_start: 2,
            refinement_len: 3,
            overlap_start: 7,
            overlap_len: 2,
        };
        assert_eq!(packed.refinement_range(10), Some(2..5));
        assert_eq!(packed.overlap_range(10), Some(7..9));

        let out_of_bounds = PackedEdgeRanges {
            overlap_start: 9,
            overlap_len: 2,
            ..packed
        };
        assert_eq!(out_of_bounds.overlap_range(10), None);
    }

    #[test]
    fn clear_resets_used_entries() {
        let mut frontier = ActiveFrontier::<2>::default();
        assert!(frontier.push(ActiveFrontierEntry {
            region_id: 7,
            score_q: 100,
            margin: 2,
            depth: 3,
        }));
        frontier.clear();
        assert_eq!(frontier, ActiveFrontier::<2>::default());
    }
}
