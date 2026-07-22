//! Executable proof module: Packed section range boundary verification.

/// Verify that packed section range `(start, len)` is strictly bounded within section length `total_len`.
pub fn verify_range_bounds(
    start: usize,
    len: usize,
    total_len: usize,
    range_name: &str,
) -> Result<(), String> {
    let end = start
        .checked_add(len)
        .ok_or_else(|| format!("Range overflow in {}", range_name))?;
    if end > total_len {
        Err(format!(
            "Range bounds violation in {}: range [{}..{}] exceeds section length {}",
            range_name, start, end, total_len
        ))
    } else {
        Ok(())
    }
}
