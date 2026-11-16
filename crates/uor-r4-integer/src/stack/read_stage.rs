//! The `Read` stage interface (ADR-0007, I1): one selection call and one result.
//!
//! The crate's reads take different inputs — the native Q31 first-pick read steps a history and a
//! residual row, the relation read scans the context's tokens — so until now each was called
//! through its own signature and parsed from its own artifact record. This module is the stage
//! boundary the ADR asks for: [`Read::select`] over a [`Query`] and a [`ContextView`], returning
//! the common [`ReadOut`].
//!
//! The reference implementation is the native Q31 first-pick read (`/14`, or `/17` on a bound
//! artifact): it is the read D86 step 6 trains and the stack serves, and its adapter calls exactly
//! the method the session called before, so the interface lands with bit-identical tests. The
//! relation read is the second consumer, which the ADR required before an adapter could be added.
//!
//! This module adds no arithmetic. A read that cannot express its result as a [`ReadOut`] is out of
//! scope until it can (ADR §1).

use uor_r4_native_vector::Q31ReadHistory;

use super::read_out::ReadOut;
use super::StackError;

/// The mechanism an artifact names for the `Read` stage.
///
/// The names are the record's own: an artifact carries one of these and an engine refuses a
/// mechanism it does not implement (ADR §1 rule 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadMechanism {
    /// The native Q31 first-pick read, `/14` unbound and `/17` bound.
    Q31FirstPick,
    /// The config-only two-class relation read, `/16`.
    Relation,
}

impl ReadMechanism {
    /// The mechanism's name in an artifact record and in the cost ledger.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Q31FirstPick => "q31-first-pick",
            Self::Relation => "relation",
        }
    }
}

/// What a read is asked about at one served row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Query {
    /// The token being served.
    pub token: u32,
    /// The row's 0-based position in the context.
    pub row: u16,
}

/// The view a read selects over.
///
/// It carries what every read may look at and nothing a read may write: the context's tokens, the
/// residual row at the read's depth, the writer table, and the read's own incremental scan. A
/// read that needs another input is a new stage field, not a wider view (ADR §1 rule 4).
pub struct ContextView<'a> {
    /// The context's tokens up to and including the row being served.
    pub tokens: &'a [u32],
    /// The residual row at the read's depth, native Q31 words.
    pub x: &'a [i32],
    /// The writer table at Q12, `vocab * width` words.
    pub table: &'a [i16],
    /// The row width.
    pub width: usize,
    /// The slots whose asked fact was already served; the relation read excludes them.
    pub earlier_asked: &'a [usize],
    /// The reference read's incremental scan. A read with no scan ignores it.
    pub scan: &'a mut Q31ReadHistory,
}

/// One read mechanism, selected at one row.
///
/// `&self` is the contract: a read may hold its table, its gates or its config, but the selection
/// it returns is a pure function of the view. The reference read's scan lives in the view for that
/// reason — the session owns it, and `reset` clears it.
pub trait Read {
    /// The mechanism this value implements.
    fn mechanism(&self) -> ReadMechanism;

    /// One selection at `query` over `view`.
    fn select(&self, query: &Query, view: &mut ContextView<'_>) -> Result<ReadOut, StackError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mechanism_names_are_the_records_own() {
        assert_eq!(ReadMechanism::Q31FirstPick.name(), "q31-first-pick");
        assert_eq!(ReadMechanism::Relation.name(), "relation");
    }

    #[test]
    fn an_empty_result_satisfies_the_out_invariant() {
        let none = ReadOut::none();
        assert_eq!(none.support, Vec::<u16>::new());
        assert_eq!(none.weights_q31, Vec::<i128>::new());
        assert_eq!(none.payload, None);
        assert_eq!(none.pick(), 0);
        assert_eq!(none.check(), Ok(()));
    }
}
