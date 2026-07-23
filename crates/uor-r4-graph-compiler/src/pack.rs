//! Profile-guided cache-line aligned packing & emission deduplication (Phase 7).

/// 64-byte cache line alignment for high-performance memory bandwidth.
pub const CACHE_LINE_BYTES: usize = 64;

/// Pad a byte vector to the nearest 64-byte cache line boundary.
pub fn pad_to_cache_line(buffer: &mut Vec<u8>) {
    let remainder = buffer.len() % CACHE_LINE_BYTES;
    if remainder != 0 {
        let padding = CACHE_LINE_BYTES - remainder;
        buffer.resize(buffer.len() + padding, 0u8);
    }
}

/// Emission table deduplication result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedEmissionTable {
    /// Contiguous packed EMIT byte section.
    pub bytes: Vec<u8>,
    /// Per-region emission start offsets (relative to EMIT remainder) and byte lengths.
    pub ranges: Vec<(u32, u32)>,
}

/// Deduplicate identical emission tables across co-activated regions
/// and align emission blocks to 64-byte cache lines.
pub fn pack_emission_tables(region_emissions: &[Vec<u8>]) -> PackedEmissionTable {
    let mut packed_bytes = Vec::new();
    let mut ranges = Vec::with_capacity(region_emissions.len());

    // Storage descriptor prefix [2, 0, 0, 0] (4 bytes)
    packed_bytes.extend_from_slice(&[2, 0, 0, 0]);

    for table in region_emissions {
        if table.is_empty() {
            ranges.push((0, 0));
            continue;
        }

        // Search for existing identical emission slice in remainder
        let remainder = &packed_bytes[4..];
        if let Some(pos) = remainder.windows(table.len()).position(|w| w == table) {
            ranges.push((pos as u32, table.len() as u32));
        } else {
            let start_offset = (packed_bytes.len() - 4) as u32;
            packed_bytes.extend_from_slice(table);
            ranges.push((start_offset, table.len() as u32));
        }
    }

    pad_to_cache_line(&mut packed_bytes);

    PackedEmissionTable {
        bytes: packed_bytes,
        ranges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_line_padding() {
        let mut buf = vec![1, 2, 3];
        pad_to_cache_line(&mut buf);
        assert_eq!(buf.len() % CACHE_LINE_BYTES, 0);
        assert_eq!(buf.len(), 64);
    }

    #[test]
    fn test_pack_emission_tables_deduplication() {
        let t1 = vec![10, 0, 0, 0, 100, 0, 0, 0];
        let t2 = vec![20, 0, 0, 0, 200, 0, 0, 0];
        let t3 = t1.clone(); // identical to t1

        let result = pack_emission_tables(&[t1, t2, t3]);
        assert_eq!(result.ranges.len(), 3);
        assert_eq!(result.ranges[0], result.ranges[2]); // t3 reuses t1 offset
        assert_eq!(result.bytes.len() % CACHE_LINE_BYTES, 0);
    }
}
