#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStatus {
    /// Token resolved within calibrated mask-Hamming margin of the region.
    Supported,
    /// Token resolved at the exact boundary of the region's acceptance radius.
    Boundary,
    /// Context signature dropped out of active graph; fell back to broad priors.
    BackedOff,
    /// Novel context not strongly supported by any semantic region.
    Novel,
    /// Context activates mutually exclusive/contradictory regions.
    Contradictory,
}
