//! Machine-readable invariant ownership matrix rows (issue #135).

use crate::inference_contract::INFERENCE_OPERATION_CONTRACT_VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantOwner {
    FormatLoader,
    Compiler,
    Certifier,
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvariantOwnershipRow {
    pub name: &'static str,
    pub owner: InvariantOwner,
    pub evidence: &'static str,
    pub contract_version: (u16, u16, u16),
}

pub const OPERATION_SET_CONFORMANCE_ROW: InvariantOwnershipRow = InvariantOwnershipRow {
    name: "Operation-Set Conformance",
    owner: InvariantOwner::Runtime,
    evidence: "P-4 source scan witnesses; disassembly audit target (#160)",
    contract_version: INFERENCE_OPERATION_CONTRACT_VERSION.as_tuple(),
};

pub const INVARIANT_OWNERSHIP_ROWS: [InvariantOwnershipRow; 1] = [OPERATION_SET_CONFORMANCE_ROW];

#[cfg(test)]
mod tests {
    use super::{InvariantOwner, OPERATION_SET_CONFORMANCE_ROW};

    #[test]
    fn operation_set_conformance_is_owned_by_runtime() {
        assert_eq!(OPERATION_SET_CONFORMANCE_ROW.owner, InvariantOwner::Runtime);
        assert_eq!(
            OPERATION_SET_CONFORMANCE_ROW.name,
            "Operation-Set Conformance"
        );
    }
}
