use super::score_q::ScoreQ;

pub const TOKEN_STATE_CAPACITY: usize = 32;
pub const LOCAL_STATE_CAPACITY: usize = 8;
pub const SEGMENT_STATE_CAPACITY: usize = 8;
pub const SESSION_STATE_CAPACITY: usize = 8;

/// One fixed-capacity semantic-state slot reserved for compiler-generated
/// update programs in Phase 8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SemanticStateSlot {
    pub region_id: u32,
    pub token: u32,
    pub score_q: ScoreQ,
    pub age: u16,
}

/// Skeleton hook payload for compiler-generated local/segment/session updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReservedStateUpdate {
    pub program_id: u16,
    pub slot: SemanticStateSlot,
}

/// Fixed-capacity token history; when saturated it keeps the most recent
/// tokens without allocating.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenState<const CAP: usize> {
    len: usize,
    recent: [u32; CAP],
}

impl<const CAP: usize> Default for TokenState<CAP> {
    fn default() -> Self {
        Self {
            len: 0,
            recent: [0; CAP],
        }
    }
}

impl<const CAP: usize> TokenState<CAP> {
    pub const fn capacity(&self) -> usize {
        CAP
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[u32] {
        &self.recent[..self.len]
    }

    pub fn clear(&mut self) {
        for token in &mut self.recent[..self.len] {
            *token = 0;
        }
        self.len = 0;
    }

    pub fn push(&mut self, token: u32) {
        if CAP == 0 {
            return;
        }
        if self.len < CAP {
            self.recent[self.len] = token;
            self.len += 1;
            return;
        }
        self.recent.copy_within(1..CAP, 0);
        self.recent[CAP - 1] = token;
    }

    pub fn occurrences(&self, token: u32) -> usize {
        self.as_slice()
            .iter()
            .filter(|&&seen| seen == token)
            .count()
    }
}

/// Fixed-capacity reserved state for the local/segment/session levels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservedState<const CAP: usize> {
    len: usize,
    slots: [SemanticStateSlot; CAP],
    last_program_id: Option<u16>,
    updates_applied: u64,
}

impl<const CAP: usize> Default for ReservedState<CAP> {
    fn default() -> Self {
        Self {
            len: 0,
            slots: [SemanticStateSlot::default(); CAP],
            last_program_id: None,
            updates_applied: 0,
        }
    }
}

impl<const CAP: usize> ReservedState<CAP> {
    pub const fn capacity(&self) -> usize {
        CAP
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[SemanticStateSlot] {
        &self.slots[..self.len]
    }

    pub fn last_program_id(&self) -> Option<u16> {
        self.last_program_id
    }

    pub fn updates_applied(&self) -> u64 {
        self.updates_applied
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots[..self.len] {
            *slot = SemanticStateSlot::default();
        }
        self.len = 0;
        self.last_program_id = None;
        self.updates_applied = 0;
    }

    pub fn apply(&mut self, update: ReservedStateUpdate) {
        if CAP == 0 {
            self.last_program_id = Some(update.program_id);
            self.updates_applied += 1;
            return;
        }
        if self.len < CAP {
            self.slots[self.len] = update.slot;
            self.len += 1;
        } else {
            self.slots.copy_within(1..CAP, 0);
            self.slots[CAP - 1] = update.slot;
        }
        self.last_program_id = Some(update.program_id);
        self.updates_applied += 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStateLevel {
    Local,
    Segment,
    Session,
}

/// Multi-timescale fixed-capacity runtime state: token state is live today;
/// local/segment/session are reserved with bounded update hooks for Phase 8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeState<
    const TOKEN_CAP: usize = TOKEN_STATE_CAPACITY,
    const LOCAL_CAP: usize = LOCAL_STATE_CAPACITY,
    const SEGMENT_CAP: usize = SEGMENT_STATE_CAPACITY,
    const SESSION_CAP: usize = SESSION_STATE_CAPACITY,
> {
    token: TokenState<TOKEN_CAP>,
    local: ReservedState<LOCAL_CAP>,
    segment: ReservedState<SEGMENT_CAP>,
    session: ReservedState<SESSION_CAP>,
}

impl<
        const TOKEN_CAP: usize,
        const LOCAL_CAP: usize,
        const SEGMENT_CAP: usize,
        const SESSION_CAP: usize,
    > Default for RuntimeState<TOKEN_CAP, LOCAL_CAP, SEGMENT_CAP, SESSION_CAP>
{
    fn default() -> Self {
        Self {
            token: TokenState::default(),
            local: ReservedState::default(),
            segment: ReservedState::default(),
            session: ReservedState::default(),
        }
    }
}

impl<
        const TOKEN_CAP: usize,
        const LOCAL_CAP: usize,
        const SEGMENT_CAP: usize,
        const SESSION_CAP: usize,
    > RuntimeState<TOKEN_CAP, LOCAL_CAP, SEGMENT_CAP, SESSION_CAP>
{
    pub fn token(&self) -> &TokenState<TOKEN_CAP> {
        &self.token
    }

    pub fn local(&self) -> &ReservedState<LOCAL_CAP> {
        &self.local
    }

    pub fn segment(&self) -> &ReservedState<SEGMENT_CAP> {
        &self.segment
    }

    pub fn session(&self) -> &ReservedState<SESSION_CAP> {
        &self.session
    }

    pub fn clear_token_state(&mut self) {
        self.token.clear();
    }

    pub fn record_token(&mut self, token: u32) {
        self.token.push(token);
    }

    pub fn token_occurrences(&self, token: u32) -> usize {
        self.token.occurrences(token)
    }

    pub fn apply_update(&mut self, level: RuntimeStateLevel, update: ReservedStateUpdate) {
        match level {
            RuntimeStateLevel::Local => self.local.apply(update),
            RuntimeStateLevel::Segment => self.segment.apply(update),
            RuntimeStateLevel::Session => self.session.apply(update),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ReservedStateUpdate, RuntimeState, RuntimeStateLevel, SemanticStateSlot, TokenState,
    };
    use crate::transformerless::score_q::ScoreQ;

    #[test]
    fn token_state_is_fixed_capacity_fifo() {
        let mut state = TokenState::<3>::default();
        state.push(10);
        state.push(20);
        state.push(10);
        state.push(30);
        state.push(40);

        assert_eq!(state.capacity(), 3);
        assert_eq!(state.len(), 3);
        assert_eq!(state.as_slice(), &[10, 30, 40]);
        assert_eq!(state.occurrences(10), 1);
        assert_eq!(state.occurrences(20), 0);
    }

    #[test]
    fn runtime_state_reserves_each_timescale_with_update_hooks() {
        let mut state = RuntimeState::<4, 2, 2, 2>::default();
        let local = ReservedStateUpdate {
            program_id: 7,
            slot: SemanticStateSlot {
                region_id: 1,
                token: 11,
                score_q: ScoreQ::from_raw(5),
                age: 0,
            },
        };
        let segment = ReservedStateUpdate {
            program_id: 9,
            slot: SemanticStateSlot {
                region_id: 2,
                token: 12,
                score_q: ScoreQ::from_raw(6),
                age: 1,
            },
        };
        let session = ReservedStateUpdate {
            program_id: 11,
            slot: SemanticStateSlot {
                region_id: 3,
                token: 13,
                score_q: ScoreQ::from_raw(7),
                age: 2,
            },
        };

        state.apply_update(RuntimeStateLevel::Local, local);
        state.apply_update(RuntimeStateLevel::Segment, segment);
        state.apply_update(RuntimeStateLevel::Session, session);

        assert_eq!(state.local().capacity(), 2);
        assert_eq!(state.segment().capacity(), 2);
        assert_eq!(state.session().capacity(), 2);
        assert_eq!(state.local().last_program_id(), Some(7));
        assert_eq!(state.segment().last_program_id(), Some(9));
        assert_eq!(state.session().last_program_id(), Some(11));
        assert_eq!(state.local().as_slice(), &[local.slot]);
        assert_eq!(state.segment().as_slice(), &[segment.slot]);
        assert_eq!(state.session().as_slice(), &[session.slot]);
    }
}
