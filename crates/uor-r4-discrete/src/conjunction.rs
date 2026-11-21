//! Conjunction memory (PLAN row 39, step 2): one row per token pair, addressed
//! by a prime product, holding a few pointer tokens with counts. A row changes
//! which tokens it points at only when the output was wrong.
//!
//! 1. **Address.** Unordered ([`Order::Unordered`]) is the rule of
//!    `uor-r4-integer`'s `fbs_pair_rows`: token `t` stands for the `t`-th prime
//!    and the pair of different tokens `a`, `b` has the address
//!    `primes[a] * primes[b]`, under 2^31. Ordered ([`Order::Ordered`]) gives
//!    the nearer token its own primes: `primes[a] * primes[vocabulary + b]`,
//!    under 2^32, so `a b` and `b a` have different rows and equal tokens have
//!    one. Unique factorisation keeps both free of collisions.
//! 2. **Pair of a target.** The two tokens before it, when the nearer one is
//!    not BOS (a window never crosses a record).
//! 3. **Read.** The output is the row's pointer with the largest count (the
//!    earliest slot on a tie) when the pair has one, else the model's output.
//! 4. **Write.** With no row: nothing when the model's output was right, else a
//!    new row pointing at the target. With a row: a slot that points at the
//!    target gains one count; otherwise the target takes an empty slot, and
//!    with none empty every slot loses one count and the target takes a slot
//!    that reached zero.
//!
//! 5. **Readout from the pick** (PLAN row 39, step 3). The copy read
//!    ([`copy_at`]) is the Pick read at distance zero: the token after the
//!    latest earlier place of the same pair in the window. Where it and the
//!    row output name different tokens, the row's two trust counts decide;
//!    each count is one side's wins on targets the row had not yet seen
//!    ([`Conjunction::judge`], [`Row::answer`]).
//!
//! Every step is an integer compare, add or table lookup.
#![forbid(unsafe_code)]
use crate::{Error, Result};
use std::collections::HashMap;

pub mod bind;
pub mod flat;
pub mod free;
pub mod saved;
pub mod trained;
pub mod trunk_key;

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(format!("conjunction: {}", message.into()))
}

/// Most pointer slots a row holds.
pub const MAX_SLOTS: usize = 8;
pub const SCHEMA: &str = "uor-r4.discrete-conjunction-probe/3";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    Unordered,
    Ordered,
}

/// A row: `count[s] == 0` marks slot `s` empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Row {
    pub pointer: [u32; MAX_SLOTS],
    pub count: [u32; MAX_SLOTS],
    /// Times the copy read was right against the row output, and the row
    /// output against the copy read, counted only where the two differed.
    pub trust: [u32; 2],
}
impl Row {
    /// The readout that answers from the pick: where the copy read names a
    /// token other than the row output, the side with more wins on this row
    /// answers (the copy read on a tie); else the row output; else the copy.
    pub fn answer(&self, copy: Option<u32>) -> Option<u32> {
        match (self.output(), copy) {
            (Some(output), Some(copy)) if copy != output => {
                Some(if self.trust[0] >= self.trust[1] {
                    copy
                } else {
                    output
                })
            }
            (Some(output), _) => Some(output),
            (None, copy) => copy,
        }
    }
    /// Counts which side was right where the copy read and the row output
    /// differ and one of them is the target. True when a count moved.
    pub fn judge(&mut self, copy: Option<u32>, target: u32) -> bool {
        let (Some(copy), Some(output)) = (copy, self.output()) else {
            return false;
        };
        if copy == output {
            return false;
        }
        let side = if copy == target {
            0
        } else if output == target {
            1
        } else {
            return false;
        };
        self.trust[side] = self.trust[side].saturating_add(1);
        true
    }
    /// Takes `target` into one of the first `slots` slots: a slot that points
    /// at it gains a count; else it takes an empty slot; with none empty every
    /// slot loses a count and the target takes one that reached zero.
    pub fn take(&mut self, slots: usize, target: u32) -> Written {
        for slot in 0..slots {
            if self.count[slot] > 0 && self.pointer[slot] == target {
                self.count[slot] = self.count[slot].saturating_add(1);
                return Written::Strengthened;
            }
        }
        let mut written = Written::Inserted;
        if (0..slots).all(|slot| self.count[slot] > 0) {
            for slot in 0..slots {
                self.count[slot] -= 1;
            }
            written = Written::Replaced;
        }
        match (0..slots).find(|&slot| self.count[slot] == 0) {
            Some(slot) => {
                self.pointer[slot] = target;
                self.count[slot] = 1;
                written
            }
            None => Written::Weakened,
        }
    }
    /// The pointer with the largest count, the earliest slot on a tie.
    pub fn output(&self) -> Option<u32> {
        let mut best: Option<usize> = None;
        for slot in 0..MAX_SLOTS {
            if self.count[slot] > best.map_or(0, |held| self.count[held]) {
                best = Some(slot);
            }
        }
        best.map(|slot| self.pointer[slot])
    }
}

/// What one write did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Written {
    /// The pair has no address.
    NoAddress,
    /// No row, and the model's output was right: nothing stored.
    NotNeeded,
    /// A new row, or an empty slot of a row, took the target.
    Inserted,
    Strengthened,
    /// Every slot lost one count and none reached zero.
    Weakened,
    /// Every slot lost one count and the target took one that reached zero.
    Replaced,
}

/// Counts of [`Written`] outcomes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct WriteTally {
    pub no_address: u64,
    pub not_needed: u64,
    pub inserted: u64,
    pub strengthened: u64,
    pub weakened: u64,
    pub replaced: u64,
}
impl WriteTally {
    pub fn add(&mut self, written: Written) {
        match written {
            Written::NoAddress => self.no_address += 1,
            Written::NotNeeded => self.not_needed += 1,
            Written::Inserted => self.inserted += 1,
            Written::Strengthened => self.strengthened += 1,
            Written::Weakened => self.weakened += 1,
            Written::Replaced => self.replaced += 1,
        }
    }
}

fn first_primes(count: usize) -> Vec<u32> {
    let mut primes: Vec<u32> = Vec::with_capacity(count);
    let mut candidate = 2u32;
    while primes.len() < count {
        let prime = primes
            .iter()
            .take_while(|&&p| u64::from(p) * u64::from(p) <= u64::from(candidate))
            .all(|&p| candidate % p != 0);
        if prime {
            primes.push(candidate);
        }
        candidate += 1;
    }
    primes
}

#[derive(Debug)]
pub struct Conjunction {
    vocabulary: usize,
    slots: usize,
    order: Order,
    primes: Vec<u32>,
    rows: HashMap<u32, Row>,
}

impl Conjunction {
    pub fn new(vocabulary: u32, slots: usize, order: Order) -> Result<Self> {
        if vocabulary < 2 || !(1..=MAX_SLOTS).contains(&slots) {
            return Err(invalid("needs a vocabulary of at least 2 and 1..8 slots"));
        }
        let vocabulary = vocabulary as usize;
        let (count, limit) = match order {
            Order::Unordered => (vocabulary, 1u64 << 31),
            Order::Ordered => (2 * vocabulary, 1u64 << 32),
        };
        let primes = first_primes(count);
        let top = match order {
            Order::Unordered => {
                u64::from(primes[vocabulary - 1]) * u64::from(primes[vocabulary - 2])
            }
            Order::Ordered => u64::from(primes[vocabulary - 1]) * u64::from(primes[count - 1]),
        };
        if top >= limit {
            return Err(invalid(format!(
                "a vocabulary of {vocabulary} has the pair address {top}, past the address width"
            )));
        }
        Ok(Self {
            vocabulary,
            slots,
            order,
            primes,
            rows: HashMap::new(),
        })
    }
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    /// The address of the pair `a`, `b` (`b` the nearer token).
    pub fn address(&self, a: u32, b: u32) -> Option<u32> {
        let (a, b) = (a as usize, b as usize);
        if a >= self.vocabulary || b >= self.vocabulary {
            return None;
        }
        match self.order {
            Order::Unordered if a == b => None,
            Order::Unordered => Some(self.primes[a] * self.primes[b]),
            Order::Ordered => Some(self.primes[a] * self.primes[self.vocabulary + b]),
        }
    }
    pub fn read(&self, a: u32, b: u32) -> Option<Row> {
        self.rows.get(&self.address(a, b)?).copied()
    }
    /// The row's output when the pair has one, else the model's output.
    pub fn output(&self, a: u32, b: u32, model: Option<u32>) -> Option<u32> {
        self.read(a, b).and_then(|row| row.output()).or(model)
    }
    /// Counts which side was right where the copy read and the row output
    /// differ and one of them is the target. Call before `write`, so the row
    /// is judged on a target it has not yet seen. True when a count moved.
    pub fn judge(&mut self, a: u32, b: u32, copy: Option<u32>, target: u32) -> bool {
        let (Some(address), Some(copy)) = (self.address(a, b), copy) else {
            return false;
        };
        self.rows
            .get_mut(&address)
            .is_some_and(|row| row.judge(Some(copy), target))
    }
    pub fn write(&mut self, a: u32, b: u32, model: Option<u32>, target: u32) -> Written {
        let Some(address) = self.address(a, b) else {
            return Written::NoAddress;
        };
        let slots = self.slots;
        let Some(row) = self.rows.get_mut(&address) else {
            if model == Some(target) {
                return Written::NotNeeded;
            }
            let mut row = Row::default();
            row.pointer[0] = target;
            row.count[0] = 1;
            self.rows.insert(address, row);
            return Written::Inserted;
        };
        row.take(slots, target)
    }
}

/// Longest suffix a [`Ladder`] row can be keyed by: twelve bits a token and a
/// leading length bit fit ten tokens in 128 bits.
pub const MAX_ORDER: usize = 20;

/// Rows keyed by longer context suffixes than the pair (PLAN row 39, step 4
/// on the host: the key reaches back to the asked turn). A suffix of `k`
/// tokens has the exact address `1 || t_1 || ... || t_k`, twelve bits a token,
/// so no two suffixes of any length share one. Each write goes to every order
/// the window is long enough for and to the pair row; a read takes the row of
/// the longest suffix that has one.
#[derive(Debug)]
pub struct Ladder {
    /// Suffix lengths above the pair, longest first.
    orders: Vec<usize>,
    slots: usize,
    rows: HashMap<[u128; 2], Row>,
    pair: Conjunction,
}

impl Ladder {
    pub fn new(vocabulary: u32, slots: usize, orders: &[usize]) -> Result<Self> {
        if vocabulary > 4_096 {
            return Err(invalid("a ladder address holds twelve bits a token"));
        }
        let mut orders = orders.to_vec();
        orders.sort_unstable_by(|a, b| b.cmp(a));
        orders.dedup();
        if orders.iter().any(|&k| !(3..=MAX_ORDER).contains(&k)) {
            return Err(invalid("ladder orders are 3..20; the pair row is order 2"));
        }
        Ok(Self {
            orders,
            slots,
            rows: HashMap::new(),
            pair: Conjunction::new(vocabulary, slots, Order::Ordered)?,
        })
    }
    pub fn len(&self) -> usize {
        self.rows.len() + self.pair.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Two words of ten tokens each, twelve bits a token under a leading one,
    /// so suffixes of different lengths never share an address.
    pub fn address(suffix: &[u32]) -> [u128; 2] {
        let pack = |part: &[u32]| {
            part.iter()
                .fold(1u128, |address, &token| (address << 12) | u128::from(token))
        };
        let cut = suffix.len().saturating_sub(10);
        [pack(&suffix[..cut]), pack(&suffix[cut..])]
    }
    /// The rows of the window's suffixes, longest first, the pair row last.
    fn held(&self, window: &[u32]) -> Vec<Row> {
        let mut held = Vec::new();
        for &k in &self.orders {
            if window.len() >= k {
                if let Some(row) = self.rows.get(&Self::address(&window[window.len() - k..])) {
                    held.push(*row);
                }
            }
        }
        if let [.., a, b] = *window {
            if let Some(row) = self.pair.read(a, b) {
                held.push(row);
            }
        }
        held
    }
    /// Writes `target` after `window` (which ends just before it and holds no
    /// BOS past its first place). Every row is judged against the copy read
    /// before it takes the target. Returns the trust counts moved.
    pub fn write(&mut self, window: &[u32], target: u32) -> u64 {
        let copy = copy_at(window);
        let mut judged = 0u64;
        let slots = self.slots;
        for &k in &self.orders {
            if window.len() >= k {
                let row = self
                    .rows
                    .entry(Self::address(&window[window.len() - k..]))
                    .or_default();
                judged += u64::from(row.judge(copy, target));
                row.take(slots, target);
            }
        }
        if let [.., a, b] = *window {
            judged += u64::from(self.pair.judge(a, b, copy, target));
            self.pair.write(a, b, None, target);
        }
        judged
    }
    /// The next token after `window`. The longest suffix with a row gives the
    /// output. Where the copy read names another token, the longest row with
    /// any trust count decides (the output on a tie or with no count at all).
    /// With no row the copy read answers. `use_copy` off reads the rows alone.
    pub fn next(&self, window: &[u32], use_copy: bool) -> Option<u32> {
        let copy = if use_copy { copy_at(window) } else { None };
        let held = self.held(window);
        let Some(output) = held.iter().find_map(Row::output) else {
            return copy;
        };
        let Some(copy) = copy.filter(|&copy| copy != output) else {
            return Some(output);
        };
        let judge = held.iter().find(|row| row.trust != [0, 0]);
        Some(match judge {
            Some(row) if row.trust[0] > row.trust[1] => copy,
            _ => output,
        })
    }
    /// The greedy reply to `prompt`, as [`reply`].
    pub fn reply(
        &self,
        prompt: &[u32],
        use_copy: bool,
        window: usize,
        bos: u32,
        eos: u32,
        limit: usize,
    ) -> Vec<u32> {
        let mut tokens = prompt.to_vec();
        let start = tokens.len();
        while tokens.len() - start < limit {
            let seen = window_at(&tokens, tokens.len(), window, bos);
            let Some(next) = self.next(seen, use_copy) else {
                break;
            };
            tokens.push(next);
            if next == eos {
                break;
            }
        }
        tokens.split_off(start)
    }
}

/// Longest asked turn a question form holds, and the longest gap in it.
pub const FORM_TOKENS: usize = 30;
pub const FORM_GAP: usize = 12;
/// A question form is used once this many TRAIN replies stood under it.
pub const FORM_SEEN: u32 = 8;
/// Question tokens a pointer may anchor on, and places it may read ahead.
pub const POINTER_BACK: usize = 16;
pub const POINTER_AHEAD: usize = 4;
/// Kept forms of one turn a reply is written and read under.
pub const FORMS_HELD: usize = 6;
/// A held row answers once one of its slots has been right this often.
pub const ROW_RIGHT: u32 = 2;
/// Marks a form taken over class codes in place of tokens (in `gap_len`).
const CLASS_FORM: u8 = 0x80;
/// Marks a form of a turn with more left out than a gap holds: its opening
/// and closing tokens with everything between left out (in `gap_len`, over the closing
/// count).
const OPEN_FORM: u8 = 0x40;
/// Marks a form keyed by the trunk frame of its kept tokens; it never meets a
/// form over tokens or class codes.
const TRUNK_FORM: u8 = 0x20;
/// Opening and closing tokens an open form may keep, and the fewest opening
/// ones.
pub const OPEN_HEAD: usize = 16;
pub const OPEN_TAIL: usize = 8;
pub const OPEN_HEAD_LEAST: usize = 4;
/// Pointer slots one output error may add to a row.
const POINTERS_ADDED: usize = 3;
/// Previous-token symbols of a held row: the reply's first place, and any
/// place after a copied token whose own row is missing.
const AT_START: u32 = u32::MAX;
const AFTER_COPY: u32 = u32::MAX - 1;
/// The history of the row a copy reads by the token ahead alone (no reply
/// history is all zero: each word carries a leading one).
const ANY_HISTORY: [u128; 2] = [0; 2];

/// The asked turn with at most one run of tokens left out: up to thirty
/// tokens in three words (twelve bits each under a leading one) and the gap's
/// place. "What is ▢?" is one form whatever name fills the gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Form {
    words: [u128; 3],
    gap_at: u8,
    gap_len: u8,
}

impl Form {
    /// The form keyed by the trunk: the frame the trunk holds after the
    /// `count` kept tokens, under the count.
    fn framed(
        trunk: &trunk_key::TrunkKey,
        kept: impl Iterator<Item = u32>,
        count: usize,
        gap_at: u8,
        gap_len: u8,
    ) -> Self {
        let word = (1u128 << 40) | ((count as u128) << 24) | u128::from(trunk.frame(kept));
        Self {
            words: [word, 1, 1],
            gap_at,
            gap_len: gap_len | TRUNK_FORM,
        }
    }
    fn of(
        turn: &[u32],
        gap_at: usize,
        gap_len: usize,
        trunk: Option<&trunk_key::TrunkKey>,
    ) -> Self {
        let mut words = [1u128; 3];
        let kept = turn[..gap_at].iter().chain(&turn[gap_at + gap_len..]);
        if let Some(trunk) = trunk {
            let count = turn.len() - gap_len;
            return Self::framed(trunk, kept.copied(), count, gap_at as u8, gap_len as u8);
        }
        for (place, &token) in kept.enumerate() {
            let word = &mut words[place / 10];
            *word = (*word << 12) | u128::from(token);
        }
        Self {
            words,
            gap_at: gap_at as u8,
            gap_len: gap_len as u8,
        }
    }
    /// Every form of `turn`: the whole turn, then shorter gaps before longer.
    fn each<'a>(
        turn: &'a [u32],
        trunk: Option<&'a trunk_key::TrunkKey>,
    ) -> impl Iterator<Item = Self> + 'a {
        let gaps = (1..=FORM_GAP.min(turn.len())).flat_map(move |len| {
            (0..=turn.len() - len).map(move |at| Self::of(turn, at, len, trunk))
        });
        std::iter::once(Self::of(turn, 0, 0, trunk)).chain(gaps)
    }
    /// The form of a long turn that keeps its first `head` and last `tail`
    /// tokens: "Say exactly this: ▢" is one form whatever is to be said.
    fn open(turn: &[u32], head: usize, tail: usize, trunk: Option<&trunk_key::TrunkKey>) -> Self {
        let mut words = [1u128; 3];
        let kept = turn[..head].iter().chain(&turn[turn.len() - tail..]);
        if let Some(trunk) = trunk {
            let gap_len = OPEN_FORM | tail as u8;
            return Self::framed(trunk, kept.copied(), head + tail, head as u8, gap_len);
        }
        for (place, &token) in kept.enumerate() {
            let word = &mut words[place / 10];
            *word = (*word << 12) | u128::from(token);
        }
        Self {
            words,
            gap_at: head as u8,
            gap_len: OPEN_FORM | tail as u8,
        }
    }
    /// Every open form of a long `turn`, those that keep most first (the
    /// longer opening on a tie).
    fn each_open<'a>(
        turn: &'a [u32],
        trunk: Option<&'a trunk_key::TrunkKey>,
    ) -> impl Iterator<Item = Self> + 'a {
        let mut kept: Vec<(usize, usize)> = (OPEN_HEAD_LEAST..=OPEN_HEAD)
            .flat_map(|head| (0..=OPEN_TAIL).map(move |tail| (head, tail)))
            .filter(|&(head, tail)| head + tail + FORM_GAP < turn.len())
            .collect();
        kept.sort_by_key(|&(head, tail)| std::cmp::Reverse((head + tail, head)));
        kept.into_iter()
            .map(move |(head, tail)| Self::open(turn, head, tail, trunk))
    }
    /// The same form, marked as one over class codes: it never meets a form
    /// over tokens.
    fn classed(mut self) -> Self {
        self.gap_len |= CLASS_FORM;
        self
    }
}

/// How a held row's slot names the next token.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Entry {
    #[default]
    Empty,
    Token(u32),
    /// The Pick read with its query taken from the question: the token
    /// `ahead` places after the latest earlier stand of the question token
    /// `back` places before the reply (that token itself when `ahead` is 0).
    Pointer {
        back: u8,
        ahead: u8,
        /// The stand must match the question token before it as well (the
        /// Pick read's pair).
        wide: bool,
        /// The token that must stand just before the one read (the word a
        /// value follows: `is`, `to`, `in`). Unused when `ahead` is 0.
        before: u16,
    },
    /// The token after the one the last pointer, copy or `Next` read.
    Next,
    /// The exact copy read ([`copy_at`]).
    Copy,
}

/// A place in a reply: the tokens before the target, the first token the
/// reads may see, the reply's start and the place the last copy read.
#[derive(Debug, Clone, Copy)]
struct Place<'a> {
    tokens: &'a [u32],
    floor: usize,
    start: usize,
    cursor: Option<usize>,
    stand: &'a Stand,
}

/// How a pointer picks the stand it reads from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StandBy {
    /// The latest earlier stand of the question token; the word before the
    /// value is checked after.
    #[default]
    Latest,
    /// The latest stand where the question token and the word before the
    /// value both hold: the value is bound to the pair.
    Pair,
    /// As `Pair`, with the pair compared by its octonion bind address.
    Octonion,
}

/// How held rows rank and gain their slots, and whether a copy has a row
/// for the token ahead alone. All off is the reader as first written.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Reads {
    /// Slots rank by how often they were right of the times they named
    /// anything, a read before a token on a tie; off, by times right.
    pub precise: bool,
    /// A target only token slots named still adds the reads that name it,
    /// and a trusted token a read of the row also names is read from there.
    pub explain: bool,
    /// After a copied token, a row keyed by the token ahead alone is read
    /// between the place's own row and the row every copied token shares.
    pub tail_row: bool,
    /// A place is answered by the best-standing slot of any held form's
    /// row; off, by the first form (most specific first) whose row answers.
    pub across: bool,
    /// A token answered with no place it was read from takes the place a
    /// read in another held form's row names the same token from.
    pub agree: bool,
}

/// The stand choice and, for the octonion bind, its table.
#[derive(Debug, Clone, Default)]
pub struct Stand {
    by: StandBy,
    bind: bind::BindTable,
    reads: Reads,
}

/// The held row that answered a place: its key apart from the form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Hit {
    own: bool,
    history: [u128; 2],
    tail: u32,
    empty: bool,
}

/// A slot's answer and, for a copying slot, the place it read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Named {
    token: u32,
    source: Option<usize>,
    /// The slot that named it ([`Entry::Empty`] for the ladder).
    by: Entry,
}

impl Place<'_> {
    /// The token after the one the last copy read, while it lies before the
    /// reply.
    fn ahead(&self) -> Option<u32> {
        let source = self.cursor? + 1;
        (source < self.start).then(|| self.tokens[source])
    }
}

/// The place a pointer reads: `ahead` places after the latest earlier stand
/// of the question token `back` places before the reply.
/// `before` is the word that must stand just before the value; `None` takes
/// the latest stand whatever stands there.
fn pointer_source(
    at: &Place,
    back: u8,
    ahead: u8,
    wide: bool,
    before: Option<u16>,
) -> Option<usize> {
    let tokens = at.tokens;
    let anchor = at.start.checked_sub(1 + usize::from(back))?;
    let low = at.floor + usize::from(wide);
    if anchor < low {
        return None;
    }
    if ahead == 0 {
        return Some(anchor);
    }
    let reach = usize::from(ahead);
    let key = tokens[anchor];
    let wide_holds = |place: usize| !wide || tokens[place - 1] == tokens[anchor - 1];
    let by = match before {
        Some(_) => at.stand.by,
        None => StandBy::Latest,
    };
    let holds = |place: usize| match (by, before) {
        (StandBy::Pair, Some(word)) => {
            tokens[place] == key && tokens[place + reach - 1] == u32::from(word)
        }
        // One place ahead the word before the value is the stand itself.
        (StandBy::Octonion, Some(_)) if reach == 1 => tokens[place] == key,
        (StandBy::Octonion, Some(word)) => {
            let wanted = at.stand.bind.address(key, u32::from(word));
            wanted.is_some()
                && at
                    .stand
                    .bind
                    .address(tokens[place], tokens[place + reach - 1])
                    == wanted
        }
        _ => tokens[place] == key,
    };
    let source = match by {
        // The latest stand of the token, even one too near the question.
        StandBy::Latest => {
            let stand = (low..anchor)
                .rev()
                .find(|&place| wide_holds(place) && tokens[place] == key)?;
            stand + reach
        }
        _ => {
            (low..anchor)
                .rev()
                .filter(|&place| place + reach < anchor)
                .find(|&place| wide_holds(place) && holds(place))?
                + reach
        }
    };
    if source >= anchor {
        return None;
    }
    match before {
        Some(word) if by == StandBy::Latest && tokens[source - 1] != u32::from(word) => None,
        _ => Some(source),
    }
}

impl Entry {
    /// The pointer of this `back`, `ahead`, `wide` that reads `target` here,
    /// bound to the token before it. Under [`StandBy::Latest`] only the
    /// latest stand is tried; otherwise the latest stand that reads `target`.
    fn pointer(at: &Place, back: u8, ahead: u8, wide: bool, target: u32) -> Option<Entry> {
        let tokens = at.tokens;
        let before = if ahead == 0 {
            pointer_source(at, back, ahead, wide, None)?;
            0
        } else if at.stand.by == StandBy::Latest {
            let source = pointer_source(at, back, ahead, wide, None)?;
            u16::try_from(tokens[source - 1]).ok()?
        } else {
            let anchor = at.start.checked_sub(1 + usize::from(back))?;
            let low = at.floor + usize::from(wide);
            let reach = usize::from(ahead);
            let source = (low..anchor)
                .rev()
                .filter(|&place| place + reach < anchor)
                .find(|&place| {
                    tokens[place] == tokens[anchor]
                        && (!wide || tokens[place - 1] == tokens[anchor - 1])
                        && tokens[place + reach] == target
                })?
                + reach;
            u16::try_from(tokens[source - 1]).ok()?
        };
        Some(Entry::Pointer {
            back,
            ahead,
            wide,
            before,
        })
    }
    fn name(self, at: &Place) -> Option<Named> {
        let tokens = at.tokens;
        let source = match self {
            Entry::Empty => return None,
            Entry::Token(token) => {
                return Some(Named {
                    token,
                    source: None,
                    by: self,
                })
            }
            Entry::Pointer {
                back,
                ahead,
                wide,
                before,
            } => pointer_source(at, back, ahead, wide, (ahead > 0).then_some(before))?,
            Entry::Next => {
                let source = at.cursor? + 1;
                if source >= at.start {
                    return None;
                }
                source
            }
            Entry::Copy => {
                let last = tokens.len().checked_sub(1)?;
                (at.floor + 1..last).rev().find(|&place| {
                    tokens[place - 1] == tokens[last - 1] && tokens[place] == tokens[last]
                })? + 1
            }
        };
        Some(Named {
            token: tokens[source],
            source: Some(source),
            by: self,
        })
    }
}

/// An answering slot's record: times right, and times it named anything
/// plus one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Standing {
    right: u64,
    named: u64,
    token: bool,
}

impl Standing {
    /// Right more often of the times it spoke; level, a read before a token.
    fn beats(self, other: Self) -> bool {
        (self.right * other.named)
            .cmp(&(other.right * self.named))
            .then(other.token.cmp(&self.token))
            .is_gt()
    }
}

/// A row held under a question form: slots that name the next token, ranked
/// by how often each was right.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HeldRow {
    entry: [Entry; MAX_SLOTS],
    count: [u32; MAX_SLOTS],
    /// Times a slot named a token that was not the target (kept under
    /// [`Reads::precise`]).
    wrong: [u32; MAX_SLOTS],
}

impl HeldRow {
    /// Live slots with their counts, most often right first (the lower slot
    /// on a tie).
    fn ranked(&self, precise: bool) -> impl Iterator<Item = (Entry, u32)> + '_ {
        let mut slots: [usize; MAX_SLOTS] = std::array::from_fn(|slot| slot);
        // Right of the times it named anything, with one doubt added so two
        // lucky reads do not outrank a long record.
        let named = |slot: usize| u64::from(self.count[slot]) + u64::from(self.wrong[slot]) + 1;
        let is_token = |slot: usize| matches!(self.entry[slot], Entry::Token(_));
        slots.sort_by(|&a, &b| {
            let by_count = self.count[b].cmp(&self.count[a]);
            if !precise {
                return by_count;
            }
            (u64::from(self.count[b]) * named(a))
                .cmp(&(u64::from(self.count[a]) * named(b)))
                .then(is_token(a).cmp(&is_token(b)))
                .then(by_count)
        });
        slots
            .into_iter()
            .filter(|&slot| self.count[slot] > 0)
            .map(|slot| (self.entry[slot], self.count[slot]))
    }
    /// The first ranked slot, right at least `least` times, that names
    /// anything here. Its place is the one any such slot read that token
    /// from, so a token slot a copy agrees with still carries the copy on.
    fn read(&self, at: &Place, least: u32) -> Option<Named> {
        let named = self
            .ranked(at.stand.reads.precise)
            .filter(|&(_, count)| count >= least)
            .find_map(|(entry, _)| entry.name(at))?;
        Some(Named {
            // Under `explain` a read right even once that names the same
            // token as the trusted slot is where the token was read.
            source: self.source_of(
                at,
                named.token,
                if at.stand.reads.explain { 1 } else { least },
            ),
            ..named
        })
    }
    /// The row's answer with the record of the slot that gave it.
    fn best(&self, at: &Place, least: u32) -> Option<(Named, Standing)> {
        let named = self.read(at, least)?;
        let slot =
            (0..MAX_SLOTS).find(|&slot| self.count[slot] > 0 && self.entry[slot] == named.by)?;
        let right = u64::from(self.count[slot]);
        Some((
            named,
            Standing {
                right,
                named: right + u64::from(self.wrong[slot]) + 1,
                token: matches!(named.by, Entry::Token(_)),
            },
        ))
    }
    /// True when the best slot (right at least `least` times) is a pointer
    /// or the copy read and it names nothing here: the row is about to
    /// answer from a lower slot for a fact the context does not hold.
    fn empty_read(&self, at: &Place, least: u32) -> bool {
        self.ranked(at.stand.reads.precise)
            .find(|&(_, count)| count >= least)
            .is_some_and(|(entry, _)| {
                matches!(entry, Entry::Pointer { .. } | Entry::Copy) && entry.name(at).is_none()
            })
    }
    /// The place the first ranked copying slot, right at least `least`
    /// times, read `target` from.
    fn source_of(&self, at: &Place, target: u32, least: u32) -> Option<usize> {
        self.ranked(at.stand.reads.precise)
            .filter(|&(_, count)| count >= least)
            .filter_map(|(entry, _)| entry.name(at))
            .find(|named| named.token == target && named.source.is_some())
            .and_then(|named| named.source)
    }
    fn insert(&mut self, entry: Entry) {
        if let Some(slot) = (0..MAX_SLOTS).find(|&slot| self.count[slot] == 0) {
            self.entry[slot] = entry;
            self.count[slot] = 1;
            self.wrong[slot] = 0;
            return;
        }
        let weakest = (0..MAX_SLOTS)
            .rev()
            .min_by_key(|&slot| self.count[slot])
            .unwrap_or(0);
        self.count[weakest] -= 1;
        if self.count[weakest] == 0 {
            self.entry[weakest] = entry;
            self.count[weakest] = 1;
            self.wrong[weakest] = 0;
        }
    }
    /// Every slot that names `target` grows. When none does (the output
    /// error), the token and each read that names it are added. Under
    /// [`Reads::explain`] the reads are added too when only token slots
    /// named it, so a row first met with one kind of value still learns
    /// where values are read. `pointers` holds this place's pointers once
    /// found: they are the same for every row written at it.
    fn take(&mut self, at: &Place, target: u32, pointers: &mut Option<Vec<Entry>>) {
        let reads = at.stand.reads;
        let right = |entry: Entry| entry.name(at).is_some_and(|named| named.token == target);
        let mut hit = false;
        let mut read_hit = false;
        for slot in 0..MAX_SLOTS {
            if self.count[slot] == 0 {
                continue;
            }
            match self.entry[slot].name(at) {
                Some(named) if named.token == target => {
                    self.count[slot] = self.count[slot].saturating_add(1);
                    hit = true;
                    read_hit |= !matches!(self.entry[slot], Entry::Token(_));
                }
                Some(_) if reads.precise => {
                    self.wrong[slot] = self.wrong[slot].saturating_add(1);
                }
                _ => {}
            }
        }
        if hit && (read_hit || !reads.explain) {
            return;
        }
        if !hit {
            self.insert(Entry::Token(target));
        }
        for entry in [Entry::Next, Entry::Copy] {
            if right(entry) {
                self.insert(entry);
            }
        }
        // A wide pointer with no reach ahead is the narrow one; it is skipped.
        let found = pointers.get_or_insert_with(|| {
            (0..POINTER_BACK)
                .flat_map(|back| {
                    (0..=POINTER_AHEAD).flat_map(move |ahead| {
                        [true, false].map(move |wide| (back as u8, ahead as u8, wide))
                    })
                })
                .filter(|&(_, ahead, wide)| !(ahead == 0 && wide))
                .filter_map(|(back, ahead, wide)| Entry::pointer(at, back, ahead, wide, target))
                .filter(|&entry| right(entry))
                .take(POINTERS_ADDED)
                .collect()
        });
        for &entry in found.iter() {
            self.insert(entry);
        }
    }
}

/// A held row's address: the form, the reply so far, and after a copied
/// token the token the copy would read next ([`AT_START`] elsewhere,
/// [`AFTER_COPY`] for the row every copied token shares). The token ahead
/// is how a copy knows to stop: a comma, a full stop or a line end in the
/// source each has its own row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct HeldKey {
    form: u32,
    history: [u128; 2],
    tail: u32,
    /// The rows for replies whose first read came up empty.
    empty: bool,
}

/// The reply so far as a held row sees it: each token, with a run of copied
/// tokens read in a row as one symbol, so a three-piece name and a one-piece number leave
/// the same history.
#[derive(Debug, Clone, Default)]
struct History {
    symbols: Vec<u32>,
    /// The place the last token was copied from.
    last: Option<usize>,
}

impl History {
    /// The symbol that stands for a run of copied tokens (one past the
    /// twelve-bit vocabulary).
    const COPIED: u32 = 4_096;

    fn copying(&self) -> bool {
        self.symbols.last() == Some(&Self::COPIED)
    }
    /// Takes the next reply token and the place it was copied from. A run
    /// goes on only while each token is read from the place after the last:
    /// a copy that jumps elsewhere starts a new run.
    fn push(&mut self, token: u32, source: Option<usize>) {
        match source {
            None => self.symbols.push(token),
            Some(place) => {
                let follows = self.last.is_some_and(|last| last + 1 == place);
                if !(self.copying() && follows) {
                    self.symbols.push(Self::COPIED);
                }
            }
        }
        self.last = source;
    }
    /// The last sixteen symbols, thirteen bits each under a leading one.
    fn address(&self) -> [u128; 2] {
        let mut words = [1u128; 2];
        let first = self.symbols.len().saturating_sub(16);
        for (place, &symbol) in self.symbols[first..].iter().enumerate() {
            let word = &mut words[place / 8];
            *word = (*word << 13) | u128::from(symbol);
        }
        words
    }
    /// The tail of this place's own row and, after a copied token, of the
    /// shared row. `ahead` is the token the copy would read next.
    fn tails(&self, ahead: Option<u32>) -> (u32, Option<u32>) {
        match ahead {
            Some(ahead) if self.copying() => (ahead, Some(AFTER_COPY)),
            None if self.copying() => (AFTER_COPY, None),
            _ => (AT_START, None),
        }
    }
}

/// Rows held under the asked turn's form for the whole reply (the question
/// carried across the reply), each keyed by the form and the reply so far.
#[derive(Debug, Clone)]
pub struct Asked {
    eos: u32,
    bos: u32,
    context: usize,
    /// One class code per token (its writer cell), or empty: a turn none of
    /// whose token forms is kept is then read under the forms of its codes.
    codes: Vec<u16>,
    /// Read choice, not saved: a turn whose most specific class form has a
    /// shorter gap than its most specific token form is read under its class
    /// forms.
    sharper: bool,
    /// Read choice, not saved: the trace also lists every row consulted,
    /// each slot with its right and wrong counts and what it names.
    dump: bool,
    /// First-pass choice, not saved: turns are also counted under their
    /// open forms. A read uses whatever forms are kept.
    open: bool,
    /// The trunk tables question forms are keyed by, or none: forms are then
    /// keyed by their tokens.
    trunk: Option<std::sync::Arc<trunk_key::TrunkKey>>,
    stand: std::sync::Arc<Stand>,
    seen: HashMap<Form, u32>,
    rows: HashMap<HeldKey, HeldRow>,
}

impl Asked {
    pub fn new(context: usize, bos: u32, eos: u32) -> Self {
        Self {
            eos,
            bos,
            context,
            codes: Vec::new(),
            sharper: false,
            dump: false,
            open: false,
            trunk: None,
            stand: Default::default(),
            seen: HashMap::new(),
            rows: HashMap::new(),
        }
    }
    pub fn forms(&self) -> usize {
        self.seen.len()
    }
    pub fn rows(&self) -> usize {
        self.rows.len()
    }
    pub fn eos(&self) -> u32 {
        self.eos
    }
    /// Sets one class code per token, before the first pass. Codes must be
    /// below 4096 (a form holds twelve bits a place).
    pub fn with_codes(mut self, codes: Vec<u16>) -> Result<Self> {
        if codes.iter().any(|&code| code >= 4096) {
            return Err(invalid("a class code must be below 4096"));
        }
        self.codes = codes;
        Ok(self)
    }
    /// Keys question forms by the trunk frame of their kept tokens, before
    /// the first pass. Class codes are not read beside a trunk key.
    pub fn with_trunk(mut self, trunk: Option<trunk_key::TrunkKey>) -> Result<Self> {
        if trunk.is_some() && !self.codes.is_empty() {
            return Err(invalid("class codes are not read beside a trunk key"));
        }
        self.trunk = trunk.map(std::sync::Arc::new);
        Ok(self)
    }
    pub fn trunk(&self) -> Option<&trunk_key::TrunkKey> {
        self.trunk.as_deref()
    }
    /// Sets how pointers pick their stand, before the first pass. The
    /// octonion bind needs its table.
    pub fn with_stand(mut self, by: StandBy, bind: bind::BindTable) -> Result<Self> {
        if by == StandBy::Octonion && bind.words.is_empty() {
            return Err(invalid("the octonion stand needs a bind table"));
        }
        let reads = self.stand.reads;
        self.stand = std::sync::Arc::new(Stand { by, bind, reads });
        Ok(self)
    }
    /// Sets how held rows rank and gain slots, before the first pass.
    pub fn with_reads(mut self, reads: Reads) -> Self {
        self.stand = std::sync::Arc::new(Stand {
            reads,
            ..(*self.stand).clone()
        });
        self
    }
    pub fn reads(&self) -> Reads {
        self.stand.reads
    }
    pub fn stand_by(&self) -> StandBy {
        self.stand.by
    }
    pub fn bind(&self) -> &bind::BindTable {
        &self.stand.bind
    }
    /// Counts long turns under their open forms, before the first pass.
    pub fn with_open_forms(mut self, on: bool) -> Self {
        self.open = on;
        self
    }
    pub fn trace_rows(&mut self, on: bool) {
        self.dump = on;
    }
    /// Every row a place could read under `forms`, slot by slot.
    fn rows_at(&self, forms: &[u32], history: &History, at: &Place, empty: bool) -> Vec<String> {
        let (own, shared) = history.tails(at.ahead());
        let words = history.address();
        let mut lines = Vec::new();
        for (index, &form) in forms.iter().enumerate() {
            for (label, tail) in [("own", Some(own)), ("shared", shared)] {
                let Some(tail) = tail else { continue };
                let key = HeldKey {
                    form,
                    history: words,
                    tail,
                    empty,
                };
                let Some(row) = self.rows.get(&key) else {
                    continue;
                };
                let slots: Vec<String> = row
                    .ranked(at.stand.reads.precise)
                    .enumerate()
                    .map(|(slot, (entry, count))| {
                        let wrong = (0..MAX_SLOTS)
                            .find(|&s| row.entry[s] == entry && row.count[s] == count)
                            .map_or(0, |s| row.wrong[s]);
                        let names = entry.name(at).map(|named| named.token);
                        format!("{slot}: {entry:?} right {count} wrong {wrong} names {names:?}")
                    })
                    .collect();
                lines.push(format!(
                    "  rows: form {index} {label} [{}]",
                    slots.join("; ")
                ));
            }
        }
        lines
    }
    pub fn read_sharper_classes(&mut self, on: bool) {
        self.sharper = on;
    }
    pub fn codes(&self) -> &[u16] {
        &self.codes
    }
    /// The turn as class codes; `None` with no code table or a token past it.
    fn coded(&self, turn: &[u32]) -> Option<Vec<u32>> {
        if self.codes.is_empty() {
            return None;
        }
        turn.iter()
            .map(|&token| self.codes.get(token as usize).map(|&code| u32::from(code)))
            .collect()
    }
    /// The asked turn before a reply that starts at `start`: the tokens after
    /// the latest EOS, at most [`FORM_TOKENS`] of them.
    fn turn<'a>(&self, tokens: &'a [u32], start: usize) -> &'a [u32] {
        let floor = start - window_at(tokens, start, self.context, self.bos).len();
        let low = floor.max(start.saturating_sub(FORM_TOKENS));
        let first = (low..start)
            .rev()
            .find(|&place| tokens[place] == self.eos)
            .map_or(low, |place| place + 1);
        &tokens[first..start]
    }
    /// The whole asked turn before a reply that starts at `start`, when it is
    /// long enough to keep an opening and leave out more than a gap holds:
    /// the tokens after the latest EOS in the
    /// window (after the BOS where there is none).
    fn long_turn<'a>(&self, tokens: &'a [u32], start: usize) -> Option<&'a [u32]> {
        let floor = start - window_at(tokens, start, self.context, self.bos).len();
        let first = (floor..start)
            .rev()
            .find(|&place| tokens[place] == self.eos)
            .map_or(floor, |place| place + 1);
        let first = first + usize::from(first < start && tokens[first] == self.bos);
        (start - first > OPEN_HEAD_LEAST + FORM_GAP).then(|| &tokens[first..start])
    }
    /// First pass: counts every form of the turn asked before `start`.
    pub fn count(&mut self, tokens: &[u32], start: usize) {
        let turn = self.turn(tokens, start);
        let trunk = self.trunk.clone();
        for form in Form::each(turn, trunk.as_deref()) {
            *self.seen.entry(form).or_default() += 1;
        }
        if let Some(coded) = self.coded(turn) {
            for form in Form::each(&coded, None) {
                *self.seen.entry(form.classed()).or_default() += 1;
            }
        }
        if self.open {
            if let Some(long) = self.long_turn(tokens, start) {
                for form in Form::each_open(long, trunk.as_deref()) {
                    *self.seen.entry(form).or_default() += 1;
                }
            }
        }
    }
    /// Ends the first pass: forms seen fewer than [`FORM_SEEN`] times go, and
    /// each kept form's count becomes its row id.
    pub fn seal(&mut self) {
        self.seen.retain(|_, count| *count >= FORM_SEEN);
        let mut forms: Vec<Form> = self.seen.keys().copied().collect();
        forms.sort_by_key(|form| (form.words, form.gap_at, form.gap_len));
        for (id, form) in forms.into_iter().enumerate() {
            self.seen.insert(form, id as u32);
        }
    }
    /// The kept forms of the turn asked before `start`, most specific first,
    /// at most [`FORMS_HELD`].
    fn held(&self, tokens: &[u32], start: usize) -> Vec<u32> {
        Form::each(self.turn(tokens, start), self.trunk.as_deref())
            .filter_map(|form| self.seen.get(&form).copied())
            .take(FORMS_HELD)
            .collect()
    }
    /// The kept class forms of that turn, most specific first, at most
    /// [`FORMS_HELD`]. Empty with no code table.
    fn held_classes(&self, tokens: &[u32], start: usize) -> Vec<u32> {
        let Some(coded) = self.coded(self.turn(tokens, start)) else {
            return Vec::new();
        };
        Form::each(&coded, None)
            .filter_map(|form| self.seen.get(&form.classed()).copied())
            .take(FORMS_HELD)
            .collect()
    }
    /// The kept open forms of that turn when it is a long one, those that
    /// keep most first, at most [`FORMS_HELD`].
    fn held_open(&self, tokens: &[u32], start: usize) -> Vec<u32> {
        let Some(long) = self.long_turn(tokens, start) else {
            return Vec::new();
        };
        Form::each_open(long, self.trunk.as_deref())
            .filter_map(|form| self.seen.get(&form).copied())
            .take(FORMS_HELD)
            .collect()
    }
    /// The forms a reply to this turn is read under: its kept token forms,
    /// or its kept class forms when it has no token form (or, under the
    /// sharper read, when a class form leaves less of the turn out).
    fn read_under(&self, tokens: &[u32], start: usize) -> Vec<u32> {
        let forms = self.held(tokens, start);
        if forms.is_empty() {
            let classes = self.held_classes(tokens, start);
            if classes.is_empty() {
                return self.held_open(tokens, start);
            }
            return classes;
        }
        if self.sharper {
            if let (_, [Some(token), Some(class)]) = self.sharpest(&tokens[..start]) {
                if class < token {
                    return self.held_classes(tokens, start);
                }
            }
        }
        forms
    }
    /// The asked turn's length and the gap of its most specific kept form,
    /// over tokens and over class codes (`None` where no form is kept).
    pub fn sharpest(&self, prompt: &[u32]) -> (usize, [Option<u8>; 2]) {
        let turn = self.turn(prompt, prompt.len());
        let tokens = Form::each(turn, self.trunk.as_deref())
            .find(|form| self.seen.contains_key(form))
            .map(|form| form.gap_len);
        let classes = self.coded(turn).and_then(|coded| {
            Form::each(&coded, None)
                .find(|form| self.seen.contains_key(&form.classed()))
                .map(|form| form.gap_len)
        });
        (turn.len(), [tokens, classes])
    }
    /// True when a reply to `prompt` is read under class forms alone.
    pub fn reads_classes(&self, prompt: &[u32]) -> bool {
        self.held(prompt, prompt.len()).is_empty()
            && !self.held_classes(prompt, prompt.len()).is_empty()
    }
    /// True when a reply to `prompt` is read under open forms alone.
    pub fn reads_open(&self, prompt: &[u32]) -> bool {
        let end = prompt.len();
        self.held(prompt, end).is_empty()
            && self.held_classes(prompt, end).is_empty()
            && !self.held_open(prompt, end).is_empty()
    }
    /// `stand` is passed apart from `self` so rows can be written while a
    /// place is held.
    fn place<'a>(
        &self,
        stand: &'a Stand,
        tokens: &'a [u32],
        start: usize,
        cursor: Option<usize>,
    ) -> Place<'a> {
        let floor = tokens.len() - window_at(tokens, tokens.len(), self.context, self.bos).len();
        Place {
            tokens,
            floor,
            start,
            cursor,
            stand,
        }
    }
    /// The held answer at a place, the index of the form it came from and
    /// whether the row was the place's own: each form's own row, most
    /// specific form first, then each form's shared row. A row answers once
    /// a slot of it has been right [`ROW_RIGHT`] times. `empty` reads the
    /// rows written for replies whose first read came up empty, and falls to
    /// the ordinary rows where there is none. `missed` is set when a row
    /// consulted has a pointer or copy read as its best slot and that read
    /// names nothing.
    fn answer(
        &self,
        forms: &[u32],
        history: &History,
        at: &Place,
        empty: bool,
        missed: &mut bool,
    ) -> Option<(Named, usize, Hit)> {
        let (own, shared) = history.tails(at.ahead());
        let words = history.address();
        let reads = at.stand.reads;
        let mut first = |history: [u128; 2], tail: u32, is_own: bool, empty: bool| {
            let row_of = |form: u32| {
                self.rows.get(&HeldKey {
                    form,
                    history,
                    tail,
                    empty,
                })
            };
            let mut found: Option<(Named, usize, Standing)> = None;
            for (index, &form) in forms.iter().enumerate() {
                let Some(row) = row_of(form) else { continue };
                *missed |= row.empty_read(at, ROW_RIGHT);
                let Some((named, standing)) = row.best(at, ROW_RIGHT) else {
                    continue;
                };
                if !reads.across {
                    found = Some((named, index, standing));
                    break;
                }
                // The earlier, more specific form keeps a tie.
                if found.is_none_or(|(_, _, held)| standing.beats(held)) {
                    found = Some((named, index, standing));
                }
            }
            let (mut named, index, _) = found?;
            if reads.agree && named.source.is_none() {
                named.source = forms
                    .iter()
                    .filter_map(|&form| row_of(form))
                    .find_map(|row| row.source_of(at, named.token, ROW_RIGHT));
            }
            let hit = Hit {
                own: is_own,
                history,
                tail,
                empty,
            };
            Some((named, index, hit))
        };
        let mut both = |empty: bool| {
            if let Some(found) = first(words, own, true, empty) {
                return Some(found);
            }
            let shared = shared?;
            if at.stand.reads.tail_row {
                if let Some(found) = first(ANY_HISTORY, own, false, empty) {
                    return Some(found);
                }
            }
            first(words, shared, false, empty)
        };
        if empty {
            if let Some(found) = both(true) {
                return Some(found);
            }
        }
        both(false)
    }
    /// Second pass: writes the TRAIN reply `tokens[start..end]` under every
    /// held form. False when the turn has no kept form.
    pub fn write(&mut self, tokens: &[u32], start: usize, end: usize) -> bool {
        self.write_under(tokens, start, end, false)
    }
    /// Third pass, after every reply is written: where the reader's own
    /// reply to this TRAIN prompt meets an empty read, the true reply is
    /// written again under the rows for that case. True when it was.
    pub fn write_empty(
        &mut self,
        ladder: &Ladder,
        tokens: &[u32],
        start: usize,
        end: usize,
        limit: usize,
    ) -> bool {
        // The reader sees the prompt's own window, not the corpus before it.
        let floor = start - window_at(tokens, start, self.context, self.bos).len();
        let mut wrote = false;
        for forms in [
            self.held(tokens, start),
            self.held_classes(tokens, start),
            self.held_open(tokens, start),
        ] {
            if forms.is_empty() {
                continue;
            }
            let (_, _, missed) =
                self.generate(ladder, &forms, &tokens[floor..start], limit, false, None);
            if missed {
                self.write_forms(&forms, tokens, start, end, true);
                wrote = true;
            }
        }
        wrote
    }
    /// Writes the reply under the turn's token forms and, apart from them,
    /// under its class forms: a read uses one set or the other, never both.
    fn write_under(&mut self, tokens: &[u32], start: usize, end: usize, empty: bool) -> bool {
        let mut wrote = false;
        for forms in [
            self.held(tokens, start),
            self.held_classes(tokens, start),
            self.held_open(tokens, start),
        ] {
            if !forms.is_empty() {
                self.write_forms(&forms, tokens, start, end, empty);
                wrote = true;
            }
        }
        wrote
    }
    fn write_forms(
        &mut self,
        forms: &[u32],
        tokens: &[u32],
        start: usize,
        end: usize,
        empty: bool,
    ) {
        let stand = std::sync::Arc::clone(&self.stand);
        let mut cursor = None;
        let mut history = History::default();
        for position in start..end {
            let target = tokens[position];
            let at = self.place(&stand, &tokens[..position], start, cursor);
            // What the reader would answer here, before this write. Where it
            // is right its verdict (copied or not) stands, so TRAIN histories
            // are the histories the reader makes.
            let read = self.answer(forms, &history, &at, empty, &mut false);
            let (own, shared) = history.tails(at.ahead());
            let words = history.address();
            let mut written = None;
            let mut pointers = None;
            let by_tail = (stand.reads.tail_row && shared.is_some()).then_some((ANY_HISTORY, own));
            let keys = [
                Some((words, own)),
                shared.map(|tail| (words, tail)),
                by_tail,
            ];
            for &form in forms {
                for (history, tail) in keys.into_iter().flatten() {
                    let key = HeldKey {
                        form,
                        history,
                        tail,
                        empty,
                    };
                    let row = self.rows.entry(key).or_default();
                    row.take(&at, target, &mut pointers);
                    written = written.or_else(|| row.source_of(&at, target, 1));
                }
            }
            cursor = match read {
                Some((named, _, _)) if named.token == target => named.source,
                _ => written,
            };
            history.push(target, cursor);
        }
    }
    /// The greedy reply to `prompt` and whether a form was held. A place no
    /// held row answers falls to `ladder` with its copy read.
    pub fn reply(&self, ladder: &Ladder, prompt: &[u32], limit: usize) -> (Vec<u32>, bool) {
        let (reply, under_form, _) = self.reply_traced(ladder, prompt, limit);
        (reply, under_form)
    }
    /// As [`Asked::reply`], with what named each token: the form's index, its
    /// own or the shared row and the slot (`ladder` where no held row
    /// answered). A reply that met an empty read is answered again from the
    /// rows for that case; its trace opens with `empty read`.
    pub fn reply_traced(
        &self,
        ladder: &Ladder,
        prompt: &[u32],
        limit: usize,
    ) -> (Vec<u32>, bool, Vec<String>) {
        let forms = self.read_under(prompt, prompt.len());
        if forms.is_empty() {
            let reply = ladder.reply(prompt, true, self.context, self.bos, self.eos, limit);
            return (reply, false, Vec::new());
        }
        let (reply, trace, missed) = self.generate(ladder, &forms, prompt, limit, false, None);
        if !missed {
            return (reply, true, trace);
        }
        let (reply, mut trace, _) = self.generate(ladder, &forms, prompt, limit, true, None);
        trace.insert(0, "empty read".into());
        (reply, true, trace)
    }
    /// As [`Asked::reply`], with the raw words of every step
    /// ([`STEP_WORDS`] a reply token) and whether the reply was read again
    /// from the rows for an empty read. The words are what a replay
    /// compares: the row's address, the slot read, the carried cursor and
    /// the token.
    pub fn reply_words(
        &self,
        ladder: &Ladder,
        prompt: &[u32],
        limit: usize,
    ) -> (Vec<u32>, Vec<u32>, bool) {
        let forms = self.read_under(prompt, prompt.len());
        let mut words = Vec::new();
        if forms.is_empty() {
            let reply = ladder.reply(prompt, true, self.context, self.bos, self.eos, limit);
            for &token in &reply {
                no_form_words(&mut words, token);
            }
            return (reply, words, false);
        }
        let (reply, _, missed) =
            self.generate(ladder, &forms, prompt, limit, false, Some(&mut words));
        if !missed {
            return (reply, words, false);
        }
        words.clear();
        let (reply, _, _) = self.generate(ladder, &forms, prompt, limit, true, Some(&mut words));
        (reply, words, true)
    }
    /// The read half of a TRAIN pass over `reply` given after `prompt`, with
    /// nothing written: at each place the held answer is read as
    /// [`Asked::write`] reads it before its write, and the given token is
    /// carried on. The words are those of [`Asked::reply_words`]; the count
    /// is of places whose held answer was absent or another token.
    pub fn forced_words(&self, prompt: &[u32], reply: &[u32], empty: bool) -> (Vec<u32>, u64) {
        let start = prompt.len();
        let forms = self.read_under(prompt, start);
        let mut words = Vec::new();
        if forms.is_empty() {
            for &token in reply {
                no_form_words(&mut words, token);
            }
            return (words, 0);
        }
        let mut unread = 0u64;
        let mut tokens = prompt.to_vec();
        let mut cursor = None;
        let mut history = History::default();
        for &target in reply {
            let at = self.place(&self.stand, &tokens, start, cursor);
            let read = self.answer(&forms, &history, &at, empty, &mut false);
            let held = read.filter(|(named, _, _)| named.token == target);
            let named = match held {
                Some((named, _, _)) => named,
                None => {
                    unread += 1;
                    Named {
                        token: target,
                        source: None,
                        by: Entry::Empty,
                    }
                }
            };
            step_words(&mut words, &forms, held, empty, &history, &at, named);
            cursor = named.source;
            history.push(target, cursor);
            tokens.push(target);
        }
        (words, unread)
    }
    /// Every kept form's own words by its id: three key words as twelve
    /// u32, then the gap's place and length. A saved file numbers forms
    /// anew, so a replay compares these and never the id.
    pub fn forms_by_id(&self) -> HashMap<u32, [u32; 14]> {
        self.seen
            .iter()
            .map(|(form, &id)| {
                let mut words = [0u32; 14];
                for (place, word) in form.words.iter().enumerate() {
                    for part in 0..4 {
                        words[place * 4 + part] = (word >> (96 - 32 * part)) as u32;
                    }
                }
                words[12] = u32::from(form.gap_at);
                words[13] = u32::from(form.gap_len);
                (id, words)
            })
            .collect()
    }
    /// One greedy reply under `forms`, its trace and whether any place met
    /// an empty read.
    fn generate(
        &self,
        ladder: &Ladder,
        forms: &[u32],
        prompt: &[u32],
        limit: usize,
        empty: bool,
        mut words: Option<&mut Vec<u32>>,
    ) -> (Vec<u32>, Vec<String>, bool) {
        let start = prompt.len();
        let mut trace = Vec::new();
        let mut missed = false;
        let mut tokens = prompt.to_vec();
        let mut cursor = None;
        let mut history = History::default();
        while tokens.len() - start < limit {
            let at = self.place(&self.stand, &tokens, start, cursor);
            if self.dump {
                trace.extend(self.rows_at(forms, &history, &at, empty));
            }
            let held = self.answer(forms, &history, &at, empty, &mut missed);
            let named = match held {
                Some((named, index, hit)) => {
                    let row = if hit.own { "own" } else { "shared" };
                    trace.push(format!(
                        "form {index} {row} {:?} ahead {:?}",
                        named.by,
                        at.ahead()
                    ));
                    named
                }
                None => {
                    let seen = window_at(&tokens, tokens.len(), self.context, self.bos);
                    let Some(token) = ladder.next(seen, true) else {
                        break;
                    };
                    trace.push("ladder".into());
                    Named {
                        token,
                        source: None,
                        by: Entry::Empty,
                    }
                }
            };
            if let Some(words) = words.as_deref_mut() {
                step_words(words, forms, held, empty, &history, &at, named);
            }
            cursor = named.source;
            history.push(named.token, cursor);
            tokens.push(named.token);
            if named.token == self.eos {
                break;
            }
        }
        (tokens.split_off(start), trace, missed)
    }
}

/// Words a reply step is replayed by: the kind (0 a place's own row, 1 a
/// shared row, 2 the ladder, 3 no form held), the form's index among the
/// held forms and its id, whether the pass read the rows for an empty read
/// and whether the row was one of them, the row's tail and its history
/// address (eight words), the carried cursor, the token ahead of it, the
/// slot (five words), the place the slot read and the token.
pub const STEP_WORDS: usize = 23;
/// Stands for "none" in a step's words.
pub const NO_WORD: u32 = u32::MAX;

fn step_words(
    out: &mut Vec<u32>,
    forms: &[u32],
    held: Option<(Named, usize, Hit)>,
    empty: bool,
    history: &History,
    at: &Place,
    named: Named,
) {
    let place = |place: Option<usize>| place.map_or(NO_WORD, |place| place as u32);
    let (kind, index, form, row_empty, tail, address) = match held {
        Some((_, index, hit)) => (
            u32::from(!hit.own),
            index as u32,
            forms[index],
            u32::from(hit.empty),
            hit.tail,
            hit.history,
        ),
        None => (2, NO_WORD, NO_WORD, 0, NO_WORD, history.address()),
    };
    out.extend([kind, index, form, u32::from(empty), row_empty, tail]);
    for word in address {
        for part in 0..4 {
            out.push((word >> (96 - 32 * part)) as u32);
        }
    }
    out.extend([place(at.cursor), at.ahead().unwrap_or(NO_WORD)]);
    out.extend(saved::entry_fields(named.by));
    out.extend([place(named.source), named.token]);
}

fn no_form_words(out: &mut Vec<u32>, token: u32) {
    out.push(3);
    out.extend([NO_WORD; STEP_WORDS - 2]);
    out.push(token);
}

/// The pair of the target at `position`: the two tokens before it, unless the
/// nearer one is BOS or the stream is too short.
pub fn pair_at(tokens: &[u32], position: usize, bos: u32) -> Option<[u32; 2]> {
    if position < 2 || position > tokens.len() {
        return None;
    }
    let near = tokens[position - 1];
    if near == bos {
        return None;
    }
    Some([tokens[position - 2], near])
}

/// One scored DEV target: its pair, its target token and the model's output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scored {
    pub pair: Option<[u32; 2]>,
    pub target: u32,
    pub model: Option<u32>,
    /// The model's ranked outputs, best first; a value past the vocabulary
    /// marks an unused place.
    pub ranked: [u32; 8],
    /// The exact copy read ([`copy_at`]) on the target's window.
    pub copy: Option<u32>,
    /// The token after the position the model's Pick read chose.
    pub picked: Option<u32>,
}

/// The exact copy read (the Pick read at distance zero, `kn_pick`'s
/// `exact_pair_pick`): `window` ends just before the target; the latest earlier
/// place where its last two tokens stood together gives the token that
/// followed them there.
pub fn copy_at(window: &[u32]) -> Option<u32> {
    if window.len() < 3 {
        return None;
    }
    let t = window.len() - 1;
    (1..t)
        .rev()
        .find(|&p| window[p - 1] == window[t - 1] && window[p] == window[t])
        .map(|p| window[p + 1])
}

/// The window the copy read sees for the target at `position`: up to
/// `context` tokens before it, starting no earlier than its record's BOS.
pub fn window_at(tokens: &[u32], position: usize, context: usize, bos: u32) -> &[u32] {
    let position = position.min(tokens.len());
    let floor = position.saturating_sub(context);
    let mut start = position;
    while start > floor {
        start -= 1;
        if tokens[start] == bos {
            break;
        }
    }
    &tokens[start..position]
}

/// A pointer readout over scored targets: a context position (the exact copy
/// read, or the model's pick) names a token.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Pointer {
    /// Scored targets where the position names a token.
    pub present: u64,
    /// Of those, the token is the target.
    pub right: u64,
    /// Covered targets whose row holds the named token in a slot.
    pub covered_in_row: u64,
    /// Of those, the named token is the target.
    pub covered_in_row_right: u64,
    /// Covered: the named token when the row holds it, else the row output.
    /// Not covered: the model's output.
    pub slot_top1: u64,
    /// The named token wherever there is one; else the row output where
    /// covered; else the model's output.
    pub first_top1: u64,
}

impl Pointer {
    fn add(&mut self, named: Option<u32>, item: &Scored, live: Option<(Row, u32)>) {
        let model_right = item.model == Some(item.target);
        let named_right = named == Some(item.target);
        self.present += u64::from(named.is_some());
        self.right += u64::from(named_right);
        match live {
            Some((row, output)) => {
                let in_row = named.is_some_and(|token| {
                    (0..MAX_SLOTS).any(|slot| row.count[slot] > 0 && row.pointer[slot] == token)
                });
                self.covered_in_row += u64::from(in_row);
                self.covered_in_row_right += u64::from(in_row && named_right);
                let slot = if in_row { named } else { Some(output) };
                self.slot_top1 += u64::from(slot == Some(item.target));
                self.first_top1 += u64::from(named.unwrap_or(output) == item.target);
            }
            None => {
                self.slot_top1 += u64::from(model_right);
                self.first_top1 += u64::from(if named.is_some() {
                    named_right
                } else {
                    model_right
                });
            }
        }
    }
}

/// A reading of the memory beside the model on scored targets.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Reading {
    pub rows: u64,
    pub scored_targets: u64,
    pub model_top1: u64,
    /// Targets whose pair has a row with an output.
    pub covered: u64,
    /// Covered targets whose row output is the target.
    pub covered_row_right: u64,
    /// Covered targets with the target in any slot of the row.
    pub covered_row_holds: u64,
    /// Covered targets the model alone had right.
    pub covered_model_right: u64,
    /// Row output where covered, model output elsewhere.
    pub combined_top1: u64,
    /// Covered targets where the model ranks at least one of the row's slots.
    pub covered_model_ranks_a_slot: u64,
    /// As `combined_top1`, but a covered target takes the row slot the model
    /// ranks first when it ranks any, else the row output.
    pub picked_top1: u64,
    /// Pointer readout from the exact copy read.
    pub copy: Pointer,
    /// Pointer readout from the model's Pick position.
    pub pick: Pointer,
    /// Targets where both name a token and the tokens agree.
    pub pick_agrees_with_copy: u64,
    /// Covered targets where the copy read names a token other than the row
    /// output.
    pub disagreements: u64,
    /// Of those, the row's trust counts gave the answer to the copy read.
    pub disagreements_to_copy: u64,
    /// [`Row::answer`] where covered; else the copy read; else the model.
    pub answered_top1: u64,
}

pub fn read_scored(memory: &Conjunction, scored: &[Scored]) -> Reading {
    let mut reading = Reading {
        rows: memory.len() as u64,
        scored_targets: scored.len() as u64,
        ..Reading::default()
    };
    for item in scored {
        let model_right = item.model == Some(item.target);
        reading.model_top1 += u64::from(model_right);
        let row = item.pair.and_then(|[a, b]| memory.read(a, b));
        let live = row.and_then(|row| row.output().map(|output| (row, output)));
        reading.copy.add(item.copy, item, live);
        reading.pick.add(item.picked, item, live);
        reading.pick_agrees_with_copy += u64::from(item.copy.is_some() && item.copy == item.picked);
        let answer = match live {
            Some((row, output)) => {
                let answer = row.answer(item.copy);
                if item.copy.is_some_and(|copy| copy != output) {
                    reading.disagreements += 1;
                    reading.disagreements_to_copy += u64::from(answer == item.copy);
                }
                answer
            }
            None => item.copy.or(item.model),
        };
        reading.answered_top1 += u64::from(answer == Some(item.target));
        match live {
            Some((row, output)) => {
                let right = output == item.target;
                let holds = (0..MAX_SLOTS)
                    .any(|slot| row.count[slot] > 0 && row.pointer[slot] == item.target);
                reading.covered += 1;
                reading.covered_row_right += u64::from(right);
                reading.covered_row_holds += u64::from(holds);
                reading.covered_model_right += u64::from(model_right);
                reading.combined_top1 += u64::from(right);
                let picked = item.ranked.iter().copied().find(|&token| {
                    (0..MAX_SLOTS).any(|slot| row.count[slot] > 0 && row.pointer[slot] == token)
                });
                reading.covered_model_ranks_a_slot += u64::from(picked.is_some());
                reading.picked_top1 += u64::from(picked.unwrap_or(output) == item.target);
            }
            None => {
                reading.combined_top1 += u64::from(model_right);
                reading.picked_top1 += u64::from(model_right);
            }
        }
    }
    reading
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// TRAIN targets written with the model's output read on the GPU.
    pub model_targets: u64,
    /// Further TRAIN targets written with no model output (the memory's own
    /// output error decides the write).
    pub bulk_targets: u64,
    pub slots: usize,
    pub order: Order,
    /// Last TRAIN targets kept out of every write and read as a second scored
    /// set from the training corpus itself.
    pub holdout: u64,
}

/// `conjunction-probe`: write the memory from TRAIN and read it beside the
/// model on DEV.
#[cfg(feature = "cuda")]
pub fn run(
    config: crate::training::DriverConfig,
    checkpoint: Option<&std::path::Path>,
    out: &std::path::Path,
    options: Options,
) -> Result<serde_json::Value> {
    use crate::training::{DriverCheckpoint, PreparedDriver};
    use serde_json::json;
    use std::time::Instant;

    if out.exists() {
        return Err(invalid(format!("{} already exists", out.display())));
    }
    config.validate()?;
    let vocabulary = config.spec.vocabulary;
    let context = config.spec.context;
    let blocks = config.spec.blocks as usize;
    let batch_rows = config.batch_rows;
    let bos = config.bos.id;
    if context < 2 {
        return Err(invalid("needs a context of at least 2"));
    }
    let mut memory = Conjunction::new(vocabulary, options.slots, options.order)?;
    let driver = PreparedDriver::load(config)?;
    let model = match checkpoint {
        Some(path) => {
            let bytes =
                std::fs::read(path).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
            let stored: DriverCheckpoint =
                serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
            let (model, _) = driver.restore(stored, 0)?;
            model
        }
        None => driver.new_model(0)?,
    };
    let model_output = |corpus: &crate::corpus::PreparedCorpus,
                        first: u64,
                        sink: &mut dyn FnMut(usize, u32, Option<u32>, [u32; 8])|
     -> Result<u64> {
        let batch = corpus.batch(first, context, batch_rows)?;
        let carried = model.carry_windows(&batch.windows)?;
        let evaluation = model.evaluate(&carried, &batch.targets)?;
        for ((row, position), target) in evaluation
            .rows
            .iter()
            .zip(&batch.target_positions)
            .zip(&batch.targets)
        {
            let top = row.top8[0];
            sink(
                *position as usize,
                *target,
                (top < vocabulary).then_some(top),
                row.top8,
            );
        }
        Ok(batch.next_target)
    };
    let window_mismatch = std::cell::Cell::new(0u64);
    // A scored set also takes the exact copy read on each window and, from the
    // forward trace, the token after the model's Pick position (first word of
    // a row's first block record; a Pick read only).
    let score = |corpus: &crate::corpus::PreparedCorpus,
                 first: u64,
                 scored: &mut Vec<Scored>|
     -> Result<u64> {
        let batch = corpus.batch(first, context, batch_rows)?;
        let carried = model.carry_windows(&batch.windows)?;
        let forward = model.forward_traced(&carried, &batch.targets)?;
        let picks = match (forward.read_trace()?, forward.trace_stride()) {
            (Some(words), Some(stride)) if model.read_spec() == crate::ReadSpec::Pick => {
                Some((words, stride * blocks))
            }
            _ => None,
        };
        drop(forward);
        let evaluation = model.evaluate(&carried, &batch.targets)?;
        for (index, row) in evaluation.rows.iter().enumerate() {
            let window: Vec<u32> = batch.windows[index]
                .iter()
                .map(|identity| identity.token)
                .collect();
            let picked = picks.as_ref().and_then(|(words, step)| {
                let pick = *words.get(index * step)? as usize;
                window.get(pick.checked_add(1)?).copied()
            });
            let host = window_at(
                corpus.tokens(),
                batch.target_positions[index] as usize,
                context as usize,
                bos,
            );
            if copy_at(host) != copy_at(&window) {
                window_mismatch.set(window_mismatch.get() + 1);
            }
            let top = row.top8[0];
            scored.push(Scored {
                pair: pair_at(corpus.tokens(), batch.target_positions[index] as usize, bos),
                target: batch.targets[index],
                model: (top < vocabulary).then_some(top),
                ranked: row.top8,
                copy: copy_at(&window),
                picked,
            });
        }
        Ok(batch.next_target)
    };

    let started = Instant::now();
    let dev = driver.dev();
    let mut scored = Vec::with_capacity(dev.eligible_len());
    let mut first = 0u64;
    while first < dev.eligible_len() as u64 {
        first = score(dev, first, &mut scored)?;
    }
    let dev_ms = started.elapsed().as_millis() as u64;

    let train = driver.train();
    let eligible = train.eligible_positions();
    let tokens = train.tokens();
    let write_limit = (eligible.len() as u64).saturating_sub(options.holdout);
    let model_end = options.model_targets.min(write_limit);
    let bulk_end = model_end
        .saturating_add(options.bulk_targets)
        .min(write_limit);
    let mut tail = Vec::new();
    let mut first = write_limit;
    while first < eligible.len() as u64 {
        first = score(train, first, &mut tail)?;
    }
    let empty = read_scored(&memory, &scored);

    let started = Instant::now();
    let mut judged = 0u64;
    let mut model_tally = WriteTally::default();
    let mut train_model_top1 = 0u64;
    let mut first = 0u64;
    while first < model_end {
        let mut pending = Vec::with_capacity(batch_rows as usize);
        let next = model_output(train, first, &mut |position, target, output, _| {
            pending.push((position, target, output));
        })?;
        for (index, (position, target, output)) in pending.into_iter().enumerate() {
            if first + index as u64 >= model_end {
                break;
            }
            train_model_top1 += u64::from(output == Some(target));
            if let Some([a, b]) = pair_at(tokens, position, bos) {
                let copy = copy_at(window_at(tokens, position, context as usize, bos));
                judged += u64::from(memory.judge(a, b, copy, target));
                model_tally.add(memory.write(a, b, output, target));
            }
        }
        first = next;
    }
    let model_ms = started.elapsed().as_millis() as u64;
    let after_model = read_scored(&memory, &scored);
    let tail_after_model = read_scored(&memory, &tail);

    let started = Instant::now();
    let mut bulk_tally = WriteTally::default();
    let mut curve = Vec::new();
    let mut mark = 1u64 << 20;
    for (done, &position) in eligible[model_end as usize..bulk_end as usize]
        .iter()
        .enumerate()
    {
        if let Some([a, b]) = pair_at(tokens, position, bos) {
            let copy = copy_at(window_at(tokens, position, context as usize, bos));
            judged += u64::from(memory.judge(a, b, copy, tokens[position]));
            bulk_tally.add(memory.write(a, b, None, tokens[position]));
        }
        if done as u64 + 1 == mark {
            curve.push(json!({
                "bulk_targets": mark,
                "dev": read_scored(&memory, &scored),
                "tail": read_scored(&memory, &tail),
            }));
            mark <<= 1;
        }
    }
    let bulk_ms = started.elapsed().as_millis() as u64;
    let after_bulk = read_scored(&memory, &scored);
    let tail_after_bulk = read_scored(&memory, &tail);

    let report = json!({
        "schema": SCHEMA,
        "order": options.order,
        "slots": options.slots,
        "pair_rule": "the two tokens before the target; none when the nearer is BOS",
        "lineage_sha256": driver.lineage_sha256()?,
        "checkpoint": checkpoint.map(|path| path.display().to_string()),
        "model_epoch": model.epoch()?,
        "train_eligible_targets": eligible.len(),
        "tail_rule": "the last TRAIN targets, never written",
        "tail_targets": tail.len(),
        "copy_rule": "the latest earlier place in the window where the pair stood; the token after it",
        "trust_writes": judged,
        "scored_copy_differs_between_host_and_batch_window": window_mismatch.get(),
        "dev_ms": dev_ms,
        "dev_empty_memory": empty,
        "model_written": {
            "targets": model_end,
            "train_model_top1": train_model_top1,
            "writes": model_tally,
            "ms": model_ms,
            "dev": after_model,
            "tail": tail_after_model,
        },
        "bulk_written": {
            "targets": bulk_end - model_end,
            "writes": bulk_tally,
            "ms": bulk_ms,
            "dev_curve": curve,
            "dev": after_bulk,
            "tail": tail_after_bulk,
        },
    });
    std::fs::write(
        out,
        serde_json::to_vec_pretty(&report).map_err(|e| invalid(e.to_string()))?,
    )
    .map_err(|e| invalid(format!("{}: {e}", out.display())))?;
    Ok(report)
}

/// Which parts answer when a reply is generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Readout {
    /// The row output alone.
    Rows,
    /// The copy read alone.
    Copy,
    /// [`Row::answer`]: the copy read against the row, decided by trust.
    Both,
}

/// The next token after `tokens`, or none when nothing answers.
pub fn next_token(
    memory: &Conjunction,
    readout: Readout,
    tokens: &[u32],
    window: usize,
    bos: u32,
) -> Option<u32> {
    let position = tokens.len();
    let copy = match readout {
        Readout::Rows => None,
        Readout::Copy | Readout::Both => copy_at(window_at(tokens, position, window, bos)),
    };
    let row = match readout {
        Readout::Copy => None,
        Readout::Rows | Readout::Both => {
            pair_at(tokens, position, bos).and_then(|[a, b]| memory.read(a, b))
        }
    };
    match row {
        Some(row) => row.answer(copy),
        None => copy,
    }
}

/// The greedy reply to `prompt`: tokens are taken one at a time until EOS,
/// `limit` tokens, or a place where nothing answers.
pub fn reply(
    memory: &Conjunction,
    readout: Readout,
    prompt: &[u32],
    window: usize,
    bos: u32,
    eos: u32,
    limit: usize,
) -> Vec<u32> {
    let mut tokens = prompt.to_vec();
    let start = tokens.len();
    while tokens.len() - start < limit {
        let Some(next) = next_token(memory, readout, &tokens, window, bos) else {
            break;
        };
        tokens.push(next);
        if next == eos {
            break;
        }
    }
    tokens.split_off(start)
}

/// One panel row: a prompt and its reference reply, as token ids.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PanelRow {
    pub prompt: Vec<u32>,
    pub reference: Vec<u32>,
}

/// Counts of one readout's greedy replies against the references.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct ReplyCount {
    pub rows: u64,
    /// The reply is the reference, id for id.
    pub exact: u64,
    /// The reply ends on EOS.
    pub ended: u64,
    /// The reply's first id is the reference's first id.
    pub first_right: u64,
    /// Reference ids, and how many the reply matches from the start.
    pub reference_ids: u64,
    pub prefix_ids: u64,
    /// Reply ids generated.
    pub reply_ids: u64,
}

pub fn count_replies(
    memory: &Conjunction,
    readout: Readout,
    panel: &[PanelRow],
    window: usize,
    bos: u32,
    eos: u32,
    limit: usize,
) -> (ReplyCount, Vec<Vec<u32>>) {
    count_replies_by(panel, eos, &|prompt| {
        reply(memory, readout, prompt, window, bos, eos, limit)
    })
}

/// Counts the replies `answer` gives to a panel.
pub fn count_replies_by(
    panel: &[PanelRow],
    eos: u32,
    answer: &dyn Fn(&[u32]) -> Vec<u32>,
) -> (ReplyCount, Vec<Vec<u32>>) {
    let mut count = ReplyCount::default();
    let mut replies = Vec::with_capacity(panel.len());
    for row in panel {
        let got = answer(&row.prompt);
        let prefix = got
            .iter()
            .zip(&row.reference)
            .take_while(|(a, b)| a == b)
            .count();
        count.rows += 1;
        count.exact += u64::from(got == row.reference);
        count.ended += u64::from(got.last() == Some(&eos));
        count.first_right += u64::from(prefix > 0);
        count.reference_ids += row.reference.len() as u64;
        count.prefix_ids += prefix as u64;
        count.reply_ids += got.len() as u64;
        replies.push(got);
    }
    (count, replies)
}

/// `conjunction-probe replies`: write rows and trust from every TRAIN target
/// with no model, then answer a panel's prompts greedily under each readout.
#[cfg(feature = "cuda")]
pub fn run_replies(
    config: crate::training::DriverConfig,
    panel_path: &std::path::Path,
    out: &std::path::Path,
    slots: usize,
    order: Order,
    window: usize,
    limit: usize,
    eos: u32,
    orders: &[usize],
) -> Result<serde_json::Value> {
    use crate::training::PreparedDriver;
    use serde_json::json;
    use std::time::Instant;

    if out.exists() {
        return Err(invalid(format!("{} already exists", out.display())));
    }
    config.validate()?;
    let vocabulary = config.spec.vocabulary;
    let context = config.spec.context as usize;
    let bos = config.bos.id;
    let bytes =
        std::fs::read(panel_path).map_err(|e| invalid(format!("{}: {e}", panel_path.display())))?;
    let panel: Vec<PanelRow> =
        serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
    if panel
        .iter()
        .flat_map(|row| row.prompt.iter().chain(&row.reference))
        .any(|&id| id >= vocabulary)
    {
        return Err(invalid("a panel id is past the vocabulary"));
    }
    let mut memory = Conjunction::new(vocabulary, slots, order)?;
    let driver = PreparedDriver::load(config)?;
    let train = driver.train();
    let tokens = train.tokens();
    let started = Instant::now();
    let mut tally = WriteTally::default();
    let mut judged = 0u64;
    for &position in train.eligible_positions() {
        if let Some([a, b]) = pair_at(tokens, position, bos) {
            let copy = copy_at(window_at(tokens, position, context, bos));
            judged += u64::from(memory.judge(a, b, copy, tokens[position]));
            tally.add(memory.write(a, b, None, tokens[position]));
        }
    }
    let write_ms = started.elapsed().as_millis() as u64;
    let started = Instant::now();
    let mut readouts = serde_json::Map::new();
    let mut replies = serde_json::Map::new();
    for (name, readout) in [
        ("rows", Readout::Rows),
        ("copy", Readout::Copy),
        ("both", Readout::Both),
    ] {
        let (count, got) = count_replies(&memory, readout, &panel, window, bos, eos, limit);
        readouts.insert(name.into(), json!(count));
        replies.insert(name.into(), json!(got));
    }
    let mut ladder_rows = 0usize;
    let mut ladder_ms = 0u64;
    if !orders.is_empty() {
        let started = Instant::now();
        let mut ladder = Ladder::new(vocabulary, slots, orders)?;
        let mut ladder_judged = 0u64;
        for &position in train.eligible_positions() {
            ladder_judged +=
                ladder.write(window_at(tokens, position, context, bos), tokens[position]);
        }
        ladder_rows = ladder.len();
        ladder_ms = started.elapsed().as_millis() as u64;
        for (name, use_copy) in [("ladder_rows", false), ("ladder_both", true)] {
            let (count, got) = count_replies_by(&panel, eos, &|prompt| {
                ladder.reply(prompt, use_copy, window, bos, eos, limit)
            });
            readouts.insert(name.into(), json!(count));
            replies.insert(name.into(), json!(got));
        }
        readouts.insert("ladder_trust_writes".into(), json!(ladder_judged));

        // The held-question rows: one reply is one run of adjacent targets.
        let started = Instant::now();
        let eligible = train.eligible_positions();
        let mut runs: Vec<[usize; 2]> = Vec::new();
        for &position in eligible {
            match runs.last_mut() {
                Some(run) if run[1] == position => run[1] = position + 1,
                _ => runs.push([position, position + 1]),
            }
        }
        let mut asked = Asked::new(window, bos, eos);
        for run in &runs {
            asked.count(tokens, run[0]);
        }
        asked.seal();
        let mut written = 0u64;
        for run in &runs {
            written += u64::from(asked.write(tokens, run[0], run[1]));
        }
        let mut empty_reads = 0u64;
        for run in &runs {
            empty_reads += u64::from(asked.write_empty(&ladder, tokens, run[0], run[1], limit));
        }
        let held = std::cell::Cell::new(0u64);
        let traces = std::cell::RefCell::new(Vec::new());
        let (count, got) = count_replies_by(&panel, eos, &|prompt| {
            let (reply, under_form, trace) = asked.reply_traced(&ladder, prompt, limit);
            held.set(held.get() + u64::from(under_form));
            traces.borrow_mut().push(trace);
            reply
        });
        replies.insert("asked_trace".into(), json!(traces.into_inner()));
        readouts.insert("asked".into(), json!(count));
        replies.insert("asked".into(), json!(got));
        readouts.insert(
            "asked_memory".into(),
            json!({
                "train_replies": runs.len(),
                "train_replies_under_a_form": written,
                "train_replies_with_an_empty_read": empty_reads,
                "forms": asked.forms(),
                "rows": asked.rows(),
                "panel_replies_under_a_form": held.get(),
                "form_seen": FORM_SEEN,
                "form_gap": FORM_GAP,
                "ms": started.elapsed().as_millis() as u64,
            }),
        );
    }
    let report = json!({
        "schema": "uor-r4.discrete-conjunction-replies/1",
        "panel": panel_path.display().to_string(),
        "panel_sha256": crate::corpus::sha256(&bytes),
        "order": order,
        "slots": slots,
        "write_window": context,
        "reply_window": window,
        "reply_limit": limit,
        "eos": eos,
        "lineage_sha256": driver.lineage_sha256()?,
        "train_targets": train.eligible_len(),
        "memory_rows": memory.len(),
        "writes": tally,
        "trust_writes": judged,
        "write_ms": write_ms,
        "ladder_orders": orders,
        "ladder_rows": ladder_rows,
        "ladder_write_ms": ladder_ms,
        "reply_ms": started.elapsed().as_millis() as u64,
        "model": "none: rows and the copy read only",
        "readouts": readouts,
        "replies": replies,
    });
    std::fs::write(
        out,
        serde_json::to_vec(&report).map_err(|e| invalid(e.to_string()))?,
    )
    .map_err(|e| invalid(format!("{}: {e}", out.display())))?;
    Ok(json!({ "readouts": report["readouts"], "memory_rows": memory.len(), "write_ms": write_ms }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held(memory: &Conjunction, a: u32, b: u32) -> Vec<[u32; 2]> {
        let row = memory.read(a, b).unwrap_or_default();
        (0..MAX_SLOTS)
            .filter(|&slot| row.count[slot] > 0)
            .map(|slot| [row.pointer[slot], row.count[slot]])
            .collect()
    }

    #[test]
    fn unordered_address_is_the_prime_product() -> Result<()> {
        let memory = Conjunction::new(4_096, 1, Order::Unordered)?;
        assert_eq!(memory.address(0, 1), Some(6));
        assert_eq!(memory.address(1, 0), Some(6));
        assert_eq!(memory.address(3, 3), None);
        assert_eq!(memory.address(0, 4_096), None);
        assert_eq!(memory.address(4_095, 4_094), Some(38_873 * 38_867));
        Ok(())
    }

    #[test]
    fn ordered_address_gives_the_nearer_token_its_own_primes() -> Result<()> {
        let memory = Conjunction::new(4, 1, Order::Ordered)?;
        // primes 2 3 5 7 for the farther token, 11 13 17 19 for the nearer.
        assert_eq!(memory.address(0, 1), Some(2 * 13));
        assert_eq!(memory.address(1, 0), Some(3 * 11));
        assert_eq!(memory.address(3, 3), Some(7 * 19));
        assert_eq!(memory.address(4, 0), None);
        let wide = Conjunction::new(4_096, 8, Order::Ordered)?;
        let mut seen = std::collections::HashSet::new();
        for a in [0u32, 1, 2, 4_094, 4_095] {
            for b in [0u32, 1, 2, 4_094, 4_095] {
                assert!(seen.insert(wide.address(a, b)));
            }
        }
        assert!(Conjunction::new(8_192, 1, Order::Ordered).is_err());
        Ok(())
    }

    #[test]
    fn a_row_is_stored_only_on_an_output_error() -> Result<()> {
        let mut memory = Conjunction::new(64, 1, Order::Unordered)?;
        assert_eq!(memory.write(1, 2, Some(9), 9), Written::NotNeeded);
        assert!(memory.is_empty());
        assert_eq!(memory.output(1, 2, Some(9)), Some(9));
        assert_eq!(memory.write(1, 2, Some(9), 7), Written::Inserted);
        assert_eq!(memory.output(2, 1, Some(9)), Some(7));
        assert_eq!(memory.write(5, 5, None, 7), Written::NoAddress);
        Ok(())
    }

    /// A token right half the times it spoke loses to a read right every
    /// time, though the token was right more often; two lucky reads do not
    /// outrank a long record; a read goes before a token of the same record.
    #[test]
    fn slots_rank_by_how_often_they_were_right_when_they_spoke() {
        let read = Entry::Next;
        let row = |token: [u32; 2], other: [u32; 2]| {
            let mut row = HeldRow::default();
            row.entry[0] = Entry::Token(7);
            row.entry[1] = read;
            [row.count[0], row.wrong[0]] = token;
            [row.count[1], row.wrong[1]] = other;
            row
        };
        let first =
            |row: &HeldRow, precise: bool| row.ranked(precise).next().map(|(entry, _)| entry);
        let half = row([10, 10], [6, 0]);
        assert_eq!(first(&half, false), Some(Entry::Token(7)));
        assert_eq!(first(&half, true), Some(read));
        let lucky = row([500, 10], [2, 0]);
        assert_eq!(first(&lucky, true), Some(Entry::Token(7)));
        let level = row([9, 1], [9, 1]);
        assert_eq!(first(&level, false), Some(Entry::Token(7)));
        assert_eq!(first(&level, true), Some(read));
    }

    /// A sparse row under the most specific form holds only a token; a
    /// broader form's row holds a read with a long clean record that names
    /// the same token from the context. Read form by form, the token comes
    /// with no place and no copy can follow it.
    #[test]
    fn a_sparse_specific_row_does_not_hide_a_read_under_another_form() {
        let (bos, eos) = (0u32, 1u32);
        let mut asked = Asked::new(192, bos, eos);
        let history = History::default();
        let key = |form: u32| HeldKey {
            form,
            history: history.address(),
            tail: AT_START,
            empty: false,
        };
        let row = |slots: &[(Entry, u32, u32)]| {
            let mut row = HeldRow::default();
            for (slot, &(entry, right, wrong)) in slots.iter().enumerate() {
                row.entry[slot] = entry;
                row.count[slot] = right;
                row.wrong[slot] = wrong;
            }
            row
        };
        asked.rows.insert(key(10), row(&[(Entry::Token(7), 3, 1)]));
        asked.rows.insert(
            key(11),
            row(&[(Entry::Next, 50, 0), (Entry::Token(7), 5, 6)]),
        );
        asked
            .rows
            .insert(key(12), row(&[(Entry::Token(8), 400, 0)]));
        // The copy last read place 1, so `Next` reads place 2: token 7.
        let tokens = [bos, 5, 7, 9, 2];
        let answer = |reads: Reads, forms: &[u32]| {
            let stand = Stand {
                reads,
                ..Default::default()
            };
            let at = asked.place(&stand, &tokens, tokens.len(), Some(1));
            asked
                .answer(forms, &history, &at, false, &mut false)
                .map(|(named, index, _)| (named.token, named.source, index))
        };
        let off = Reads::default();
        assert_eq!(answer(off, &[10, 11]), Some((7, None, 0)));
        let agree = Reads { agree: true, ..off };
        assert_eq!(answer(agree, &[10, 11]), Some((7, Some(2), 0)));
        let across = Reads {
            across: true,
            ..off
        };
        assert_eq!(answer(across, &[10, 11]), Some((7, Some(2), 1)));
        // Across forms a better record may also change the token; an agreed
        // place never does.
        assert_eq!(answer(across, &[10, 12]), Some((8, None, 1)));
        assert_eq!(answer(agree, &[10, 12]), Some((7, None, 0)));
    }

    #[test]
    fn one_slot_gives_way_only_after_its_count_is_spent() -> Result<()> {
        let mut memory = Conjunction::new(64, 1, Order::Unordered)?;
        assert_eq!(memory.write(1, 2, None, 7), Written::Inserted);
        assert_eq!(memory.write(1, 2, None, 7), Written::Strengthened);
        assert_eq!(memory.write(1, 2, None, 8), Written::Weakened);
        assert_eq!(held(&memory, 1, 2), vec![[7, 1]]);
        assert_eq!(memory.write(1, 2, None, 8), Written::Replaced);
        assert_eq!(held(&memory, 1, 2), vec![[8, 1]]);
        Ok(())
    }

    #[test]
    fn slots_keep_the_commonest_pointer_through_other_targets() -> Result<()> {
        let mut memory = Conjunction::new(64, 2, Order::Ordered)?;
        // 7 is the commonest but never a majority of the stream.
        for target in [7u32, 8, 9, 7, 10, 11, 7, 12, 7, 13] {
            memory.write(1, 2, None, target);
        }
        assert_eq!(memory.output(1, 2, None), Some(7));
        assert_eq!(memory.output(2, 1, Some(3)), Some(3));
        let mut one = Conjunction::new(64, 1, Order::Ordered)?;
        for target in [7u32, 8, 9, 7, 10, 11, 7, 12, 7, 13] {
            one.write(1, 2, None, target);
        }
        assert_eq!(one.output(1, 2, None), Some(13));
        Ok(())
    }

    #[test]
    fn pair_and_reading_follow_the_stream() -> Result<()> {
        let bos = 0u32;
        let tokens = [0u32, 5, 6, 7, 0, 5, 6, 9];
        assert_eq!(pair_at(&tokens, 1, bos), None);
        assert_eq!(pair_at(&tokens, 2, bos), Some([0, 5]));
        assert_eq!(pair_at(&tokens, 3, bos), Some([5, 6]));
        assert_eq!(pair_at(&tokens, 5, bos), None);
        let mut memory = Conjunction::new(64, 2, Order::Unordered)?;
        memory.write(5, 6, None, 7);
        memory.write(5, 6, None, 7);
        memory.write(5, 6, None, 9);
        let scored = [
            Scored {
                pair: Some([5, 6]),
                target: 7,
                model: Some(1),
                ranked: [1, 7, 9, 64, 64, 64, 64, 64],
                copy: Some(9),
                picked: Some(7),
            },
            Scored {
                pair: Some([5, 6]),
                target: 9,
                model: Some(9),
                ranked: [9, 7, 64, 64, 64, 64, 64, 64],
                copy: Some(9),
                picked: None,
            },
            Scored {
                pair: None,
                target: 5,
                model: Some(5),
                ranked: [5, 64, 64, 64, 64, 64, 64, 64],
                copy: Some(5),
                picked: Some(3),
            },
        ];
        let reading = read_scored(&memory, &scored);
        assert_eq!(reading.model_top1, 2);
        assert_eq!(reading.covered, 2);
        assert_eq!(reading.covered_row_right, 1);
        assert_eq!(reading.covered_row_holds, 2);
        assert_eq!(reading.covered_model_right, 1);
        assert_eq!(reading.combined_top1, 2);
        // Row holds 7 (count 2) and 9 (count 1). The model ranks 7 before 9 on
        // the first target and 9 before 7 on the second, so the pick is right
        // on both; the third has no pair and keeps the model's output.
        assert_eq!(reading.covered_model_ranks_a_slot, 2);
        assert_eq!(reading.picked_top1, 3);
        // Copy names 9, 9, 5: wrong where the row also holds it (first
        // target), right on the second, right with no row on the third.
        assert_eq!(
            reading.copy,
            Pointer {
                present: 3,
                right: 2,
                covered_in_row: 2,
                covered_in_row_right: 1,
                slot_top1: 2,
                first_top1: 2,
            }
        );
        // The pick names 7, nothing, 3: the second target falls back to the
        // row output 7 (wrong), the third keeps the model under the slot rule.
        assert_eq!(
            reading.pick,
            Pointer {
                present: 2,
                right: 1,
                covered_in_row: 1,
                covered_in_row_right: 1,
                slot_top1: 2,
                first_top1: 1,
            }
        );
        assert_eq!(reading.pick_agrees_with_copy, 0);
        Ok(())
    }

    #[test]
    fn trust_counts_decide_between_the_copy_read_and_the_row() -> Result<()> {
        let mut memory = Conjunction::new(64, 2, Order::Ordered)?;
        assert!(!memory.judge(1, 2, Some(9), 9));
        memory.write(1, 2, None, 7);
        let row = |memory: &Conjunction| memory.read(1, 2).unwrap_or_default();
        assert_eq!(row(&memory).answer(None), Some(7));
        assert_eq!(row(&memory).answer(Some(7)), Some(7));
        // No evidence yet: the copy read answers.
        assert_eq!(row(&memory).answer(Some(9)), Some(9));
        // Agreement and a target neither side names move nothing.
        assert!(!memory.judge(1, 2, Some(7), 7));
        assert!(!memory.judge(1, 2, Some(9), 3));
        assert!(memory.judge(1, 2, Some(9), 7));
        assert_eq!(row(&memory).trust, [0, 1]);
        assert_eq!(row(&memory).answer(Some(9)), Some(7));
        assert!(memory.judge(1, 2, Some(9), 9));
        assert!(memory.judge(1, 2, Some(9), 9));
        assert_eq!(row(&memory).trust, [2, 1]);
        assert_eq!(row(&memory).answer(Some(9)), Some(9));
        assert_eq!(Row::default().answer(Some(4)), Some(4));
        assert_eq!(Row::default().answer(None), None);
        Ok(())
    }

    #[test]
    fn a_reply_is_generated_until_eos_the_limit_or_no_answer() -> Result<()> {
        let (bos, eos) = (0u32, 1u32);
        let mut memory = Conjunction::new(64, 2, Order::Ordered)?;
        memory.write(5, 6, None, 7);
        memory.write(6, 7, None, eos);
        let run =
            |readout, prompt: &[u32], limit| reply(&memory, readout, prompt, 192, bos, eos, limit);
        assert_eq!(run(Readout::Rows, &[0, 5, 6], 8), vec![7, 1]);
        assert_eq!(run(Readout::Rows, &[0, 5, 6], 1), vec![7]);
        assert_eq!(run(Readout::Rows, &[0, 9, 9], 8), Vec::<u32>::new());
        // The copy read repeats what followed the pair before; with no row for
        // (9, 4) and no earlier place of it, the reply stops.
        assert_eq!(
            run(Readout::Copy, &[0, 5, 6, 9, 4, 5, 6], 8),
            vec![9, 4, 5, 6, 9, 4, 5, 6]
        );
        assert_eq!(
            run(Readout::Copy, &[0, 5, 6, 9, 4, 8, 6], 8),
            Vec::<u32>::new()
        );
        // Both: the copy read names 9, the row 7, and no evidence favours the row.
        assert_eq!(run(Readout::Both, &[0, 5, 6, 9, 3, 5, 6], 1), vec![9]);
        let panel = [
            PanelRow {
                prompt: vec![0, 5, 6],
                reference: vec![7, 1],
            },
            PanelRow {
                prompt: vec![0, 5, 6],
                reference: vec![7, 8, 1],
            },
        ];
        let (count, replies) = count_replies(&memory, Readout::Rows, &panel, 192, bos, eos, 8);
        assert_eq!(replies, vec![vec![7, 1], vec![7, 1]]);
        assert_eq!(
            count,
            ReplyCount {
                rows: 2,
                exact: 1,
                ended: 2,
                first_right: 2,
                reference_ids: 5,
                prefix_ids: 3,
                reply_ids: 4,
            }
        );
        Ok(())
    }

    #[test]
    fn a_ladder_reads_the_longest_suffix_and_backs_off_to_the_pair() -> Result<()> {
        assert_ne!(Ladder::address(&[0, 5]), Ladder::address(&[5]));
        assert_ne!(Ladder::address(&[0, 0, 5]), Ladder::address(&[0, 5]));
        assert!(Ladder::new(4_096, 8, &[21]).is_err());
        assert!(Ladder::new(8_192, 8, &[3]).is_err());
        let mut ladder = Ladder::new(64, 2, &[3])?;
        // After "9 5 6" comes 7; after "8 5 6" comes 3. The pair row (5, 6)
        // holds both, 7 first.
        ladder.write(&[0, 9, 5, 6], 7);
        ladder.write(&[0, 9, 5, 6], 7);
        ladder.write(&[0, 8, 5, 6], 3);
        assert_eq!(ladder.next(&[0, 9, 5, 6], false), Some(7));
        assert_eq!(ladder.next(&[0, 8, 5, 6], false), Some(3));
        // An unseen three-token suffix falls to the pair row.
        assert_eq!(ladder.next(&[0, 4, 5, 6], false), Some(7));
        assert_eq!(ladder.next(&[0, 4, 4, 4], false), None);
        // With no trust count the row output stands against the copy read;
        // with no row the copy read answers.
        assert_eq!(ladder.next(&[5, 6, 9, 9, 5, 6], true), Some(7));
        assert_eq!(ladder.next(&[4, 4, 9, 2, 4, 4], true), Some(9));
        // The copy read (2) wins a target the row output (7) misses twice.
        let window = [5u32, 6, 2, 9, 5, 6];
        assert_eq!(ladder.write(&window, 2), 2);
        assert_eq!(ladder.write(&window, 2), 2);
        assert_eq!(ladder.next(&window, true), Some(2));
        assert_eq!(ladder.reply(&[0, 9, 5, 6], false, 192, 0, 1, 4), vec![7]);
        Ok(())
    }

    #[test]
    fn a_pointer_is_bound_to_the_word_before_its_value() -> Result<()> {
        // Stated:  BOS JOB IS v DOT EOS WHAT JOB QM MARK | YOU GREW v DOT EOS
        // Changed: BOS JOB HAS TO v DOT EOS WHAT JOB QM MARK | YOU GREW v DOT EOS
        // The value stands two places after JOB in one wording and three in
        // the other, under the same question form. A turn of zero to three
        // FILL tokens before the question moves the value, so no fixed place
        // before the reply holds it.
        let [bos, eos, is, dot, what, qm, mark] = [0u32, 1, 2, 3, 4, 5, 6];
        let [you, grew, has, to, job, fill] = [8u32, 9, 10, 11, 12, 13];
        let record = |changed: bool, v: u32, filled: usize| {
            let mut tokens = vec![bos, job];
            tokens.extend(if changed { vec![has, to] } else { vec![is] });
            tokens.extend([v, dot, eos]);
            tokens.extend(std::iter::repeat(fill).take(filled));
            tokens.extend([eos, what, job, qm, mark, you, grew, v, dot, eos]);
            tokens
        };
        let mut tokens = Vec::new();
        let mut runs = Vec::new();
        for index in 0..48u32 {
            let at = tokens.len();
            tokens.extend(record(index % 2 == 1, 20 + index, (index as usize / 2) % 4));
            runs.push([tokens.len() - 5, tokens.len()]);
            assert_eq!(tokens[at], bos);
        }
        let ladder = Ladder::new(64, 2, &[3])?;
        let mut asked = Asked::new(192, bos, eos);
        for run in &runs {
            asked.count(&tokens, run[0]);
        }
        asked.seal();
        for run in &runs {
            assert!(asked.write(&tokens, run[0], run[1]));
        }
        for changed in [false, true] {
            let whole = record(changed, 14, 2);
            let prompt = &whole[..whole.len() - 5];
            assert_eq!(
                asked.reply(&ladder, prompt, 8),
                (vec![you, grew, 14, dot, eos], true),
                "changed {changed}"
            );
        }
        Ok(())
    }

    #[test]
    fn a_held_form_answers_a_name_it_never_saw_through_a_pointer() -> Result<()> {
        // Told:   BOS x IS v DOT EOS WHAT x QM MARK | YOU GREW v DOT EOS
        // Untold: BOS EOS WHAT x QM MARK | YOU NO EOS
        // A new x and v each record; the form "WHAT ▢ QM MARK" is in all.
        let [bos, eos, is, dot, what, qm, mark] = [0u32, 1, 2, 3, 4, 5, 6];
        let [no, you, grew] = [7u32, 8, 9];
        let told = |x: u32, v: u32| {
            vec![
                bos, x, is, v, dot, eos, what, x, qm, mark, you, grew, v, dot, eos,
            ]
        };
        let untold = |x: u32| vec![bos, eos, what, x, qm, mark, you, no, eos];
        let mut tokens = Vec::new();
        let mut runs = Vec::new();
        for index in 0..12u32 {
            let at = tokens.len();
            tokens.extend(told(20 + index, 40 + index));
            runs.push([at + 10, at + 15]);
        }
        for index in 0..9u32 {
            let at = tokens.len();
            tokens.extend(untold(32 + index));
            runs.push([at + 6, at + 9]);
        }
        let ladder = Ladder::new(64, 2, &[3])?;
        let mut asked = Asked::new(192, bos, eos);
        for run in &runs {
            asked.count(&tokens, run[0]);
        }
        asked.seal();
        assert!(asked.forms() > 0);
        for run in &runs {
            assert!(asked.write(&tokens, run[0], run[1]));
        }
        let prompt = told(60, 61)[..10].to_vec();
        assert_eq!(
            asked.reply(&ladder, &prompt, 8),
            (vec![you, grew, 61, dot, eos], true)
        );
        // A fact that was never told. The commoner told reply is begun, its
        // pointer names nothing, and no token slot was right twice: the
        // reply stops short.
        let asked_untold = untold(60)[..6].to_vec();
        assert_eq!(
            asked.reply(&ladder, &asked_untold, 8),
            (vec![you, grew], true)
        );
        // Third pass: TRAIN prompts whose reply meets an empty read write
        // their true reply under the rows for that case.
        let mut met = 0;
        for run in &runs {
            met += u32::from(asked.write_empty(&ladder, &tokens, run[0], run[1], 8));
        }
        assert_eq!(met, 9);
        assert_eq!(
            asked.reply(&ladder, &asked_untold, 8),
            (vec![you, no, eos], true)
        );
        assert_eq!(
            asked.reply(&ladder, &prompt, 8),
            (vec![you, grew, 61, dot, eos], true)
        );
        // A turn of no kept form falls to the ladder.
        let other = vec![bos, 7, 7, 7, eos, 8, 8, 8, 8, 8, 8, 8, 8, 8];
        assert!(!asked.reply(&ladder, &other, 8).1);
        Ok(())
    }

    #[test]
    fn the_window_stops_at_the_record_start_and_the_context() {
        let tokens = [0u32, 5, 6, 7, 0, 5, 6, 9, 8];
        assert_eq!(window_at(&tokens, 3, 192, 0), &[0, 5, 6]);
        assert_eq!(window_at(&tokens, 8, 192, 0), &[0, 5, 6, 9]);
        assert_eq!(window_at(&tokens, 8, 2, 0), &[6, 9]);
        assert_eq!(window_at(&tokens, 0, 192, 0), &[] as &[u32]);
    }

    #[test]
    fn the_copy_read_takes_the_latest_earlier_place_of_the_pair() {
        assert_eq!(copy_at(&[5, 6]), None);
        assert_eq!(copy_at(&[1, 2, 3]), None);
        assert_eq!(copy_at(&[5, 6, 7, 1, 5, 6]), Some(7));
        assert_eq!(copy_at(&[5, 6, 7, 5, 6, 8, 5, 6]), Some(8));
        // The pair's own last place is not an earlier place.
        assert_eq!(copy_at(&[6, 5, 5, 6]), None);
    }
}
