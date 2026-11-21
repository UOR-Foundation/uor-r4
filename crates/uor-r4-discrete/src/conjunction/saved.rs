//! The reader as one integer file (PLAN row 39: a count on a saved model).
//!
//! [`Reader`] is the suffix [`Ladder`] and the held-question rows ([`Asked`])
//! that `conjunction-probe replies` builds in memory. [`Reader::to_bytes`]
//! writes them as little-endian integers in key order, so one reader has one
//! byte string, and [`Reader::from_bytes`] reads them back into the same
//! structs: a loaded reader answers through the same `Asked::reply` code as
//! the one that was trained.
//!
//! What is left out is what a read can never reach: a held slot that was
//! right fewer than [`ROW_RIGHT`] times never names a token, and a held row
//! with no such slot never answers. Leaving them out changes no reply.
//! Suffix rows written fewer than `ladder_least` times are left out as well;
//! that does change replies when `ladder_least` is above one, and is the
//! caller's measured choice.
//!
//! Layout: `UORRDR01`, the header integers, the question forms in id order,
//! the held rows, the suffix rows, the pair rows, then the SHA-256 of
//! everything before it.
use super::bind::BindTable;
use super::free::Update;
use super::trunk_key::TrunkKey;
use super::{
    invalid, window_at, Asked, Entry, Form, HeldKey, HeldRow, Ladder, Reads, Row, StandBy,
    FORMS_HELD, FORM_GAP, FORM_TOKENS, MAX_ORDER, MAX_SLOTS, NO_WORD, ROW_RIGHT, STEP_WORDS,
};
use crate::Result;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub const MAGIC: &[u8; 8] = b"UORRDR01";
/// A file that also holds one class code per token, after the orders.
pub const MAGIC_CODES: &[u8; 8] = b"UORRDR02";
/// A file whose pointers pick their stand by the bound pair: after the
/// orders it holds the class code count and codes, the stand choice (1 the
/// pair, 2 its octonion bind address) and the bind table.
pub const MAGIC_STAND: &[u8; 8] = b"UORRDR03";
/// A file with any of the [`Reads`] choices on: the 03 layout (stand choice
/// 0 for the latest stand), then the choices as bits (1 precise, 2 explain,
/// 4 tail row, 8 across forms, 16 agreed place), and after each held slot's count its wrong count.
pub const MAGIC_READS: &[u8; 8] = b"UORRDR04";
/// As `UORRDR04`, with the question forms keyed by a trunk frame: the read
/// choices are followed by the SHA-256 of the trunk key the file was written
/// beside, and the file is read only with that key.
pub const MAGIC_TRUNK: &[u8; 8] = b"UORRDR05";

/// The rows a reply is read from.
#[derive(Debug)]
pub struct Reader {
    pub vocabulary: u32,
    pub ladder: Ladder,
    pub asked: Asked,
}

/// What one TRAIN pass set wrote.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Trained {
    pub targets: u64,
    pub replies: u64,
    pub replies_under_a_form: u64,
    pub replies_with_an_empty_read: u64,
    pub forms: u64,
    pub held_rows: u64,
    pub ladder_rows: u64,
}

/// What a saved file holds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Saved {
    pub bytes: u64,
    pub forms: u64,
    pub held_rows: u64,
    pub held_slots: u64,
    pub suffix_rows: u64,
    pub pair_rows: u64,
}

struct Out {
    bytes: Vec<u8>,
}

impl Out {
    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }
    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }
    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }
    fn u128(&mut self, value: u128) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }
    fn row(&mut self, row: &Row) {
        let live: Vec<usize> = (0..MAX_SLOTS).filter(|&slot| row.count[slot] > 0).collect();
        self.u8(live.len() as u8);
        self.u32(row.trust[0]);
        self.u32(row.trust[1]);
        for slot in live {
            self.u32(row.pointer[slot]);
            self.u32(row.count[slot]);
        }
    }
}

struct In<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl In<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N]> {
        let end = self.at + N;
        let part = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| invalid("the reader file ends early"))?;
        self.at = end;
        Ok(part.try_into().expect("length checked"))
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take::<1>()?[0])
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take()?))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    fn u128(&mut self) -> Result<u128> {
        Ok(u128::from_le_bytes(self.take()?))
    }
    fn row(&mut self, vocabulary: u32) -> Result<Row> {
        let live = usize::from(self.u8()?);
        if live == 0 || live > MAX_SLOTS {
            return Err(invalid("a saved row holds 1..8 slots"));
        }
        let mut row = Row {
            trust: [self.u32()?, self.u32()?],
            ..Row::default()
        };
        for slot in 0..live {
            row.pointer[slot] = self.u32()?;
            row.count[slot] = self.u32()?;
            if row.pointer[slot] >= vocabulary || row.count[slot] == 0 {
                return Err(invalid("a saved row slot is past the vocabulary or empty"));
            }
        }
        Ok(row)
    }
}

/// Kind byte, three small fields and one value: a slot's entry.
pub(super) fn entry_fields(entry: Entry) -> [u32; 5] {
    match entry {
        Entry::Empty => [0, 0, 0, 0, 0],
        Entry::Token(token) => [1, 0, 0, 0, token],
        Entry::Pointer {
            back,
            ahead,
            wide,
            before,
        } => [
            2,
            u32::from(back),
            u32::from(ahead),
            u32::from(wide),
            u32::from(before),
        ],
        Entry::Next => [3, 0, 0, 0, 0],
        Entry::Copy => [4, 0, 0, 0, 0],
    }
}

pub(super) fn entry_of(fields: [u32; 5], vocabulary: u32) -> Result<Entry> {
    let small = |value: u32| u8::try_from(value).map_err(|_| invalid("a saved slot field"));
    Ok(match fields {
        [1, 0, 0, 0, token] if token < vocabulary => Entry::Token(token),
        [2, back, ahead, wide, before] if wide < 2 && before < vocabulary => Entry::Pointer {
            back: small(back)?,
            ahead: small(ahead)?,
            wide: wide == 1,
            before: before as u16,
        },
        [3, 0, 0, 0, 0] => Entry::Next,
        [4, 0, 0, 0, 0] => Entry::Copy,
        _ => return Err(invalid("a saved slot is of no known kind")),
    })
}

fn held_order(key: &HeldKey) -> (u32, bool, u32, [u128; 2]) {
    (key.form, key.empty, key.tail, key.history)
}

fn form_order(form: &Form) -> ([u128; 3], u8, u8) {
    (form.words, form.gap_at, form.gap_len)
}

/// Step words with each form id replaced by that form's own words.
fn by_form_words(forms: &HashMap<u32, [u32; 14]>, words: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(words.len() / STEP_WORDS * (STEP_WORDS + 13));
    for step in words.chunks_exact(STEP_WORDS) {
        out.extend(&step[..2]);
        out.extend(forms.get(&step[2]).copied().unwrap_or([NO_WORD; 14]));
        out.extend(&step[3..]);
    }
    out
}

/// What a replay of one panel found. `trained` is the reader as TRAIN left
/// it in memory, `served` the reader read back from its file, `forced` the
/// read half of a TRAIN pass over the served reply, on the served reader.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct Replay {
    pub prompts: u64,
    pub reply_ids: u64,
    pub words_a_step: u64,
    /// Steps by kind: a place's own row, a shared row, the ladder, no form.
    pub steps: [u64; 4],
    pub replies_from_empty_read_rows: u64,
    pub prompts_served_differs_from_trained: u64,
    pub prompts_forced_differs_from_served: u64,
    pub forced_unread_steps: u64,
    pub trained_words_sha256: String,
    pub served_words_sha256: String,
    pub forced_words_sha256: String,
}

/// Replays `prompts` on the trained and the served reader and compares the
/// raw words of every step: the row's address (form words, history, tail),
/// the slot read, the carried cursor and the token.
pub fn replay<'a>(
    trained: &Reader,
    served: &Reader,
    prompts: impl IntoIterator<Item = &'a [u32]>,
    limit: usize,
) -> Replay {
    let trained_forms = trained.asked.forms_by_id();
    let served_forms = served.asked.forms_by_id();
    let mut hashes = [Sha256::new(), Sha256::new(), Sha256::new()];
    let mut found = Replay {
        words_a_step: (STEP_WORDS + 13) as u64,
        ..Default::default()
    };
    let take = |hash: &mut Sha256, words: &[u32]| {
        hash.update((words.len() as u64).to_le_bytes());
        for word in words {
            hash.update(word.to_le_bytes());
        }
    };
    for prompt in prompts {
        let (_, before, _) = trained.replay_words(&trained_forms, prompt, limit);
        let (reply, after, empty) = served.replay_words(&served_forms, prompt, limit);
        let (forced, unread) = served.forced_replay_words(&served_forms, prompt, &reply, empty);
        found.prompts += 1;
        found.reply_ids += reply.len() as u64;
        found.replies_from_empty_read_rows += u64::from(empty);
        for step in after.chunks_exact(STEP_WORDS + 13) {
            found.steps[step[0] as usize] += 1;
        }
        found.prompts_served_differs_from_trained += u64::from(before != after);
        found.prompts_forced_differs_from_served += u64::from(forced != after);
        found.forced_unread_steps += unread;
        let [first, second, third] = &mut hashes;
        take(first, &before);
        take(second, &after);
        take(third, &forced);
    }
    let hex = |hash: Sha256| -> String {
        hash.finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    };
    let [first, second, third] = hashes;
    found.trained_words_sha256 = hex(first);
    found.served_words_sha256 = hex(second);
    found.forced_words_sha256 = hex(third);
    found
}

impl Reader {
    /// Writes every row from TRAIN: the suffix rows from each eligible
    /// target, then the three held-question passes over each reply (a run of
    /// adjacent eligible targets). The passes are those of
    /// `conjunction-probe replies`.
    #[allow(clippy::too_many_arguments)]
    pub fn train(
        tokens: &[u32],
        eligible: &[usize],
        vocabulary: u32,
        window: usize,
        bos: u32,
        eos: u32,
        slots: usize,
        orders: &[usize],
        limit: usize,
        codes: &[u16],
        by: StandBy,
        bind: BindTable,
        open: bool,
        reads: Reads,
    ) -> Result<(Self, Trained)> {
        Self::train_by(
            tokens,
            eligible,
            vocabulary,
            window,
            bos,
            eos,
            slots,
            orders,
            limit,
            codes,
            by,
            bind,
            open,
            reads,
            Update::InOrder,
            None,
        )
    }

    /// As [`Reader::train`], with the held rows written by `update`.
    #[allow(clippy::too_many_arguments)]
    pub fn train_by(
        tokens: &[u32],
        eligible: &[usize],
        vocabulary: u32,
        window: usize,
        bos: u32,
        eos: u32,
        slots: usize,
        orders: &[usize],
        limit: usize,
        codes: &[u16],
        by: StandBy,
        bind: BindTable,
        open: bool,
        reads: Reads,
        update: Update,
        trunk: Option<TrunkKey>,
    ) -> Result<(Self, Trained)> {
        if trunk
            .as_ref()
            .is_some_and(|key| key.roots.len() != vocabulary as usize)
        {
            return Err(invalid("the trunk key must hold one root row a token"));
        }
        if !bind.words.is_empty() && bind.words.len() != vocabulary as usize {
            return Err(invalid("the bind table must hold one word a token"));
        }
        if !codes.is_empty() && codes.len() != vocabulary as usize {
            return Err(invalid("class codes must be one a token"));
        }
        if tokens.iter().any(|&token| token >= vocabulary) {
            return Err(invalid("a TRAIN id is past the vocabulary"));
        }
        if eligible
            .windows(2)
            .any(|pair| pair[0] >= pair[1] || pair[1] >= tokens.len())
        {
            return Err(invalid("eligible targets must rise and lie in TRAIN"));
        }
        let mut ladder = Ladder::new(vocabulary, slots, orders)?;
        for &position in eligible {
            ladder.write(window_at(tokens, position, window, bos), tokens[position]);
        }
        let mut runs: Vec<[usize; 2]> = Vec::new();
        for &position in eligible {
            match runs.last_mut() {
                Some(run) if run[1] == position => run[1] = position + 1,
                _ => runs.push([position, position + 1]),
            }
        }
        let mut asked = Asked::new(window, bos, eos)
            .with_codes(codes.to_vec())?
            .with_trunk(trunk)?
            .with_stand(by, bind)?
            .with_open_forms(open)
            .with_reads(reads);
        for run in &runs {
            asked.count(tokens, run[0]);
        }
        asked.seal();
        let (written, empty_reads) = match update {
            Update::InOrder => {
                let mut written = 0u64;
                for run in &runs {
                    written += u64::from(asked.write(tokens, run[0], run[1]));
                }
                let mut empty_reads = 0u64;
                for run in &runs {
                    empty_reads +=
                        u64::from(asked.write_empty(&ladder, tokens, run[0], run[1], limit));
                }
                (written, empty_reads)
            }
            Update::Free { passes } => asked.write_free(&ladder, tokens, &runs, limit, passes),
        };
        let trained = Trained {
            targets: eligible.len() as u64,
            replies: runs.len() as u64,
            replies_under_a_form: written,
            replies_with_an_empty_read: empty_reads,
            forms: asked.forms() as u64,
            held_rows: asked.rows() as u64,
            ladder_rows: ladder.len() as u64,
        };
        Ok((
            Self {
                vocabulary,
                ladder,
                asked,
            },
            trained,
        ))
    }

    /// The greedy reply to `prompt`, at most `limit` ids.
    pub fn reply(&self, prompt: &[u32], limit: usize) -> Vec<u32> {
        let (reply, _) = self.asked.reply(&self.ladder, prompt, limit);
        reply
    }

    /// As [`Reader::reply`], with what named each token.
    pub fn reply_traced(&self, prompt: &[u32], limit: usize) -> (Vec<u32>, Vec<String>) {
        let (reply, _, trace) = self.asked.reply_traced(&self.ladder, prompt, limit);
        (reply, trace)
    }

    /// The reply to `prompt` as replay words: every step's words with the
    /// form's id replaced by the form's own fourteen words
    /// (`STEP_WORDS + 13` a reply token), and whether the reply was read
    /// again from the rows for an empty read. `forms` is
    /// [`Asked::forms_by_id`] of this reader.
    pub fn replay_words(
        &self,
        forms: &HashMap<u32, [u32; 14]>,
        prompt: &[u32],
        limit: usize,
    ) -> (Vec<u32>, Vec<u32>, bool) {
        let (reply, words, empty) = self.asked.reply_words(&self.ladder, prompt, limit);
        (reply, by_form_words(forms, &words), empty)
    }

    /// The read half of a TRAIN pass over `reply`, as replay words, and the
    /// places whose held answer was absent or another token.
    pub fn forced_replay_words(
        &self,
        forms: &HashMap<u32, [u32; 14]>,
        prompt: &[u32],
        reply: &[u32],
        empty: bool,
    ) -> (Vec<u32>, u64) {
        let (words, unread) = self.asked.forced_words(prompt, reply, empty);
        (by_form_words(forms, &words), unread)
    }

    /// The reader as bytes. Suffix rows written fewer than `ladder_least`
    /// times are left out (1 keeps all of them).
    pub fn to_bytes(&self, ladder_least: u32) -> (Vec<u8>, Saved) {
        let mut out = Out { bytes: Vec::new() };
        let mut saved = Saved::default();
        let by = self.asked.stand_by();
        let reads = self.asked.reads();
        let ranked = reads != Reads::default();
        let bound = ranked || by != StandBy::Latest;
        let trunk = self.asked.trunk();
        let ranked = ranked || trunk.is_some();
        let bound = bound || trunk.is_some();
        out.bytes.extend_from_slice(if trunk.is_some() {
            MAGIC_TRUNK
        } else if ranked {
            MAGIC_READS
        } else if by != StandBy::Latest {
            MAGIC_STAND
        } else if self.asked.codes().is_empty() {
            MAGIC
        } else {
            MAGIC_CODES
        });
        for value in [
            self.vocabulary,
            self.asked.context as u32,
            self.asked.bos,
            self.asked.eos,
            self.ladder.slots as u32,
            ROW_RIGHT,
            MAX_SLOTS as u32,
            FORM_TOKENS as u32,
            FORM_GAP as u32,
            FORMS_HELD as u32,
            self.ladder.orders.len() as u32,
        ] {
            out.u32(value);
        }
        for &order in &self.ladder.orders {
            out.u32(order as u32);
        }
        if bound {
            out.u32(self.asked.codes().len() as u32);
        }
        for &code in self.asked.codes() {
            out.bytes.extend_from_slice(&code.to_le_bytes());
        }
        if bound {
            let bind = self.asked.bind();
            out.u32(match by {
                StandBy::Latest => 0,
                StandBy::Pair => 1,
                StandBy::Octonion => 2,
            });
            out.u32(bind.count as u32);
            for &word in bind.words.iter().chain(&bind.of) {
                out.bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        if ranked {
            out.u32(
                u32::from(reads.precise)
                    | u32::from(reads.explain) << 1
                    | u32::from(reads.tail_row) << 2
                    | u32::from(reads.across) << 3
                    | u32::from(reads.agree) << 4,
            );
        }
        if let Some(trunk) = trunk {
            out.bytes.extend_from_slice(&trunk.sha256());
        }

        let mut forms: Vec<(&Form, &u32)> = self.asked.seen.iter().collect();
        forms.sort_by_key(|&(_, &id)| id);
        out.u32(forms.len() as u32);
        for (form, _) in forms {
            for word in form.words {
                out.u128(word);
            }
            out.u8(form.gap_at);
            out.u8(form.gap_len);
            saved.forms += 1;
        }

        let mut held: Vec<(&HeldKey, &HeldRow)> = self
            .asked
            .rows
            .iter()
            .filter(|(_, row)| row.count.iter().any(|&count| count >= ROW_RIGHT))
            .collect();
        held.sort_by_key(|&(key, _)| held_order(key));
        out.u64(held.len() as u64);
        for (key, row) in held {
            out.u32(key.form);
            out.u8(u8::from(key.empty));
            out.u32(key.tail);
            out.u128(key.history[0]);
            out.u128(key.history[1]);
            let live: Vec<usize> = (0..MAX_SLOTS)
                .filter(|&slot| row.count[slot] >= ROW_RIGHT)
                .collect();
            out.u8(live.len() as u8);
            for slot in live {
                let [kind, back, ahead, wide, value] = entry_fields(row.entry[slot]);
                for small in [kind, back, ahead, wide] {
                    out.u8(small as u8);
                }
                out.u32(value);
                out.u32(row.count[slot]);
                if ranked {
                    out.u32(row.wrong[slot]);
                }
                saved.held_slots += 1;
            }
            saved.held_rows += 1;
        }

        let mut suffix: Vec<(&[u128; 2], &Row)> = self
            .ladder
            .rows
            .iter()
            .filter(|(_, row)| {
                row.count.iter().map(|&count| u64::from(count)).sum::<u64>()
                    >= u64::from(ladder_least)
            })
            .collect();
        suffix.sort_by_key(|&(address, _)| *address);
        out.u64(suffix.len() as u64);
        for (address, row) in suffix {
            out.u128(address[0]);
            out.u128(address[1]);
            out.row(row);
            saved.suffix_rows += 1;
        }

        let mut pairs: Vec<(&u32, &Row)> = self.ladder.pair.rows.iter().collect();
        pairs.sort_by_key(|&(address, _)| *address);
        out.u64(pairs.len() as u64);
        for (address, row) in pairs {
            out.u32(*address);
            out.row(row);
            saved.pair_rows += 1;
        }

        let digest = Sha256::digest(&out.bytes);
        out.bytes.extend_from_slice(&digest);
        saved.bytes = out.bytes.len() as u64;
        (out.bytes, saved)
    }

    /// Reads a file [`Reader::to_bytes`] wrote. Refuses a file whose digest,
    /// read constants, key order or slot kinds are not what this build
    /// writes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::from_bytes_beside(bytes, None)
    }

    /// As [`Reader::from_bytes`], for a file that may be keyed by a trunk:
    /// a `UORRDR05` file is read only with the trunk key whose SHA-256 it
    /// names, and a file of another kind only with none.
    pub fn from_bytes_beside(bytes: &[u8], trunk: Option<TrunkKey>) -> Result<Self> {
        if bytes.len() < MAGIC.len() + 32 {
            return Err(invalid("not a reader file"));
        }
        let keyed = &bytes[..MAGIC.len()] == MAGIC_TRUNK;
        match (keyed, trunk.is_some()) {
            (true, false) => {
                return Err(invalid(
                    "this reader file is keyed by a trunk and is read only beside it",
                ))
            }
            (false, true) => return Err(invalid("this reader file is not keyed by a trunk")),
            _ => {}
        }
        let (coded, bound, ranked) = match &bytes[..MAGIC.len()] {
            magic if magic == MAGIC_TRUNK => (false, true, true),
            magic if magic == MAGIC => (false, false, false),
            magic if magic == MAGIC_CODES => (true, false, false),
            magic if magic == MAGIC_STAND => (false, true, false),
            magic if magic == MAGIC_READS => (false, true, true),
            _ => return Err(invalid("not a UORRDR01 to 05 reader file")),
        };
        let (body, digest) = bytes.split_at(bytes.len() - 32);
        if Sha256::digest(body).as_slice() != digest {
            return Err(invalid("the reader file's digest does not match its bytes"));
        }
        let mut input = In {
            bytes: body,
            at: MAGIC.len(),
        };
        let vocabulary = input.u32()?;
        let context = input.u32()? as usize;
        let bos = input.u32()?;
        let eos = input.u32()?;
        let slots = input.u32()? as usize;
        let constants = [
            input.u32()?,
            input.u32()?,
            input.u32()?,
            input.u32()?,
            input.u32()?,
        ];
        if constants
            != [
                ROW_RIGHT,
                MAX_SLOTS as u32,
                FORM_TOKENS as u32,
                FORM_GAP as u32,
                FORMS_HELD as u32,
            ]
        {
            return Err(invalid(
                "the reader file was written under other read constants",
            ));
        }
        let order_count = input.u32()? as usize;
        if order_count > MAX_ORDER {
            return Err(invalid("too many ladder orders"));
        }
        let mut orders = Vec::with_capacity(order_count);
        for _ in 0..order_count {
            orders.push(input.u32()? as usize);
        }
        let mut ladder = Ladder::new(vocabulary, slots, &orders)?;
        if ladder.orders != orders {
            return Err(invalid("ladder orders must fall and not repeat"));
        }
        let code_count = match (coded, bound) {
            (_, true) => input.u32()?,
            (true, _) => vocabulary,
            _ => 0,
        };
        if code_count != 0 && code_count != vocabulary {
            return Err(invalid("class codes must be one a token"));
        }
        let mut codes = Vec::new();
        for _ in 0..code_count {
            codes.push(u16::from_le_bytes(input.take()?));
        }
        let mut asked = Asked::new(context, bos, eos).with_codes(codes)?;
        if bound {
            let by = match input.u32()? {
                0 if ranked => StandBy::Latest,
                1 => StandBy::Pair,
                2 => StandBy::Octonion,
                _ => return Err(invalid("an unknown stand choice")),
            };
            let count = input.u32()? as usize;
            if count > 16 {
                return Err(invalid("more than sixteen block vectors"));
            }
            let mut bind = BindTable {
                count,
                ..BindTable::default()
            };
            if count > 0 {
                for _ in 0..vocabulary {
                    bind.words.push(u16::from_le_bytes(input.take()?));
                }
                for _ in 0..count * count {
                    bind.of.push(u16::from_le_bytes(input.take()?));
                }
            }
            asked = asked.with_stand(by, bind.checked()?)?;
        }
        if ranked {
            let bits = input.u32()?;
            if (bits == 0 && !keyed) || bits > 31 {
                return Err(invalid("unknown read choices"));
            }
            asked = asked.with_reads(Reads {
                precise: bits & 1 != 0,
                explain: bits & 2 != 0,
                tail_row: bits & 4 != 0,
                across: bits & 8 != 0,
                agree: bits & 16 != 0,
            });
        }
        if let Some(trunk) = trunk {
            let named: [u8; 32] = input.take()?;
            // The digest names the tables and what of them keys a form, so
            // the file says which key it was written under.
            let mut found = None;
            for by in super::trunk_key::Keyed::ALL {
                let keyed = trunk.clone().keyed(by)?;
                if keyed.sha256() == named {
                    found = Some(keyed);
                }
            }
            let trunk = found
                .filter(|trunk| trunk.roots.len() == vocabulary as usize)
                .ok_or_else(|| invalid("the reader file was written beside another trunk key"))?;
            asked = asked.with_trunk(Some(trunk))?;
        }

        let form_count = input.u32()?;
        let mut last: Option<Form> = None;
        for id in 0..form_count {
            let form = Form {
                words: [input.u128()?, input.u128()?, input.u128()?],
                gap_at: input.u8()?,
                gap_len: input.u8()?,
            };
            if last.is_some_and(|last| form_order(&last) >= form_order(&form)) {
                return Err(invalid("question forms must rise"));
            }
            last = Some(form);
            asked.seen.insert(form, id);
        }

        let held_count = input.u64()?;
        let mut last: Option<HeldKey> = None;
        for _ in 0..held_count {
            let form = input.u32()?;
            let empty = match input.u8()? {
                0 => false,
                1 => true,
                _ => return Err(invalid("a held row's empty mark is 0 or 1")),
            };
            let key = HeldKey {
                form,
                empty,
                tail: input.u32()?,
                history: [input.u128()?, input.u128()?],
            };
            if key.form >= form_count
                || last.is_some_and(|last| held_order(&last) >= held_order(&key))
            {
                return Err(invalid("held rows must rise under known forms"));
            }
            last = Some(key);
            let live = usize::from(input.u8()?);
            if live == 0 || live > MAX_SLOTS {
                return Err(invalid("a held row holds 1..8 slots"));
            }
            let mut row = HeldRow::default();
            for slot in 0..live {
                let fields = [
                    u32::from(input.u8()?),
                    u32::from(input.u8()?),
                    u32::from(input.u8()?),
                    u32::from(input.u8()?),
                    input.u32()?,
                ];
                row.entry[slot] = entry_of(fields, vocabulary)?;
                row.count[slot] = input.u32()?;
                if row.count[slot] < ROW_RIGHT {
                    return Err(invalid("a held slot below the read count"));
                }
                if ranked {
                    row.wrong[slot] = input.u32()?;
                }
            }
            asked.rows.insert(key, row);
        }

        let suffix_count = input.u64()?;
        let mut last: Option<[u128; 2]> = None;
        for _ in 0..suffix_count {
            let address = [input.u128()?, input.u128()?];
            if last.is_some_and(|last| last >= address) {
                return Err(invalid("suffix rows must rise"));
            }
            last = Some(address);
            ladder.rows.insert(address, input.row(vocabulary)?);
        }

        let pair_count = input.u64()?;
        let mut last: Option<u32> = None;
        for _ in 0..pair_count {
            let address = input.u32()?;
            if last.is_some_and(|last| last >= address) {
                return Err(invalid("pair rows must rise"));
            }
            last = Some(address);
            ladder.pair.rows.insert(address, input.row(vocabulary)?);
        }
        if input.at != body.len() {
            return Err(invalid("the reader file has bytes past its rows"));
        }
        Ok(Self {
            vocabulary,
            ladder,
            asked,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Told:   BOS x IS v DOT EOS WHAT x QM MARK | YOU GREW v DOT EOS
    /// Untold: BOS EOS WHAT x QM MARK | YOU NO EOS
    fn train() -> Result<(Reader, Vec<Vec<u32>>)> {
        train_beside(None)
    }

    /// The same TRAIN with the question forms keyed by `trunk`.
    fn train_beside(trunk: Option<TrunkKey>) -> Result<(Reader, Vec<Vec<u32>>)> {
        let [bos, eos, is, dot, what, qm, mark] = [0u32, 1, 2, 3, 4, 5, 6];
        let [no, you, grew] = [7u32, 8, 9];
        let told = |x: u32, v: u32| {
            vec![
                bos, x, is, v, dot, eos, what, x, qm, mark, you, grew, v, dot, eos,
            ]
        };
        let untold = |x: u32| vec![bos, eos, what, x, qm, mark, you, no, eos];
        let mut tokens = Vec::new();
        let mut eligible = Vec::new();
        for index in 0..12u32 {
            let at = tokens.len();
            tokens.extend(told(20 + index, 40 + index));
            eligible.extend(at + 10..at + 15);
        }
        for index in 0..9u32 {
            let at = tokens.len();
            tokens.extend(untold(32 + index));
            eligible.extend(at + 6..at + 9);
        }
        // One reply seen once: its last rows are never right twice.
        let at = tokens.len();
        tokens.extend([bos, 19, is, 39, dot, eos, what, 19, qm, mark]);
        tokens.extend([you, grew, 39, dot, 10, eos]);
        eligible.extend(at + 10..at + 16);
        let (reader, trained) = Reader::train_by(
            &tokens,
            &eligible,
            64,
            192,
            bos,
            eos,
            2,
            &[3],
            8,
            &[],
            StandBy::Latest,
            BindTable::default(),
            false,
            Reads::default(),
            Update::InOrder,
            trunk,
        )?;
        assert_eq!(trained.replies, 22);
        assert_eq!(trained.replies_with_an_empty_read, 9);
        let prompts = vec![
            told(60, 61)[..10].to_vec(),
            untold(60)[..6].to_vec(),
            vec![bos, 7, 7, 7, eos, 8, 8, 8, 8, 8, 8, 8, 8, 8],
            told(21, 41)[..10].to_vec(),
        ];
        Ok((reader, prompts))
    }

    #[test]
    fn a_saved_reader_replies_as_the_trained_one_with_fewer_rows() -> Result<()> {
        let (reader, prompts) = train()?;
        let (bytes, saved) = reader.to_bytes(1);
        let loaded = Reader::from_bytes(&bytes)?;
        assert!(saved.held_rows > 0);
        assert!((loaded.asked.rows() as u64) == saved.held_rows);
        assert!(loaded.asked.rows() < reader.asked.rows());
        assert_eq!(reader.reply(&prompts[0], 8), vec![8, 9, 61, 3, 1]);
        assert_eq!(reader.reply(&prompts[1], 8), vec![8, 7, 1]);
        for prompt in &prompts {
            assert_eq!(loaded.reply(prompt, 8), reader.reply(prompt, 8));
        }
        // One reader has one byte string.
        let (again, resaved) = loaded.to_bytes(1);
        assert_eq!(again, bytes);
        assert_eq!(resaved, saved);
        Ok(())
    }

    /// Keyed by a trunk frame the reader gives the replies it gives keyed by
    /// tokens, holds no token form, and its file is read only beside the
    /// trunk key it was written beside.
    #[test]
    fn a_reader_keyed_by_the_trunk_replies_alike_and_is_read_only_beside_it() -> Result<()> {
        let key = crate::conjunction::trunk_key::tests::toy(64);
        let (plain, prompts) = train()?;
        let (reader, _) = train_beside(Some(key.clone()))?;
        assert_eq!(reader.asked.forms(), plain.asked.forms());
        assert!(reader
            .asked
            .seen
            .keys()
            .all(|form| form.gap_len & 0x20 != 0));
        assert!(plain.asked.seen.keys().all(|form| form.gap_len & 0x20 == 0));
        assert_eq!(reader.reply(&prompts[0], 8), vec![8, 9, 61, 3, 1]);
        for prompt in &prompts {
            assert_eq!(reader.reply(prompt, 8), plain.reply(prompt, 8));
        }
        let (bytes, saved) = reader.to_bytes(1);
        assert_eq!(&bytes[..8], MAGIC_TRUNK);
        let loaded = Reader::from_bytes_beside(&bytes, Some(key.clone()))?;
        for prompt in &prompts {
            assert_eq!(loaded.reply(prompt, 8), reader.reply(prompt, 8));
        }
        let (again, resaved) = loaded.to_bytes(1);
        assert_eq!(again, bytes);
        assert_eq!(resaved, saved);
        // Without the trunk, beside another trunk, and a token-keyed file
        // beside a trunk: each is refused.
        assert!(Reader::from_bytes(&bytes).is_err());
        let mut other = key.clone();
        // Token 4 is the question word every kept form holds.
        other.roots[4][0] ^= 1;
        assert!(Reader::from_bytes_beside(&bytes, Some(other.clone())).is_err());
        let (plain_bytes, _) = plain.to_bytes(1);
        assert!(Reader::from_bytes_beside(&plain_bytes, Some(key)).is_err());
        // The GPU tables hold token forms and refuse this reader.
        assert!(crate::conjunction::flat::Flat::of(&loaded).is_err());
        // Beside another trunk the same TRAIN keys its forms otherwise.
        let (moved, _) = train_beside(Some(other))?;
        assert!(moved
            .asked
            .seen
            .keys()
            .any(|form| !reader.asked.seen.contains_key(form)));
        Ok(())
    }

    /// Keyed a kept token at a time (by the frame path or by the trunk
    /// address) the reader replies alike, and its file says which key it was
    /// written under: the bare tables read it back.
    #[test]
    fn a_reader_keyed_by_the_frame_path_or_the_address_names_its_key() -> Result<()> {
        use crate::conjunction::trunk_key::Keyed;
        let tables = crate::conjunction::trunk_key::tests::toy(64);
        let (plain, prompts) = train()?;
        let (frame, _) = train_beside(Some(tables.clone()))?;
        let mut files = vec![frame.to_bytes(1).0];
        for by in [Keyed::Path, Keyed::Address] {
            let (reader, _) = train_beside(Some(tables.clone().keyed(by)?))?;
            assert_eq!(reader.asked.forms(), plain.asked.forms());
            assert!(reader
                .asked
                .seen
                .keys()
                .all(|form| form.gap_len & 0x20 != 0));
            let (bytes, _) = reader.to_bytes(1);
            let loaded = Reader::from_bytes_beside(&bytes, Some(tables.clone()))?;
            assert_eq!(loaded.asked.trunk().map(|trunk| trunk.by), Some(by));
            for prompt in &prompts {
                assert_eq!(reader.reply(prompt, 8), plain.reply(prompt, 8));
                assert_eq!(loaded.reply(prompt, 8), reader.reply(prompt, 8));
            }
            assert_eq!(loaded.to_bytes(1).0, bytes);
            assert!(Reader::from_bytes(&bytes).is_err());
            files.push(bytes);
        }
        // Three keys, three files.
        assert_ne!(files[0], files[1]);
        assert_ne!(files[1], files[2]);
        Ok(())
    }

    /// The served reader replays the trained one word for word; a reader
    /// whose held rows are gone does not, so the comparison can fail.
    #[test]
    fn a_replay_compares_raw_step_words() -> Result<()> {
        let (reader, prompts) = train()?;
        let (bytes, _) = reader.to_bytes(1);
        let served = Reader::from_bytes(&bytes)?;
        let found = replay(&reader, &served, prompts.iter().map(Vec::as_slice), 8);
        assert_eq!(found.prompts, 4);
        assert_eq!(found.words_a_step, 36);
        assert_eq!(found.steps.iter().sum::<u64>(), found.reply_ids);
        assert!(found.steps[0] > 0);
        assert!(found.replies_from_empty_read_rows > 0);
        assert_eq!(found.prompts_served_differs_from_trained, 0);
        assert_eq!(found.prompts_forced_differs_from_served, 0);
        assert_eq!(found.forced_unread_steps, found.steps[2]);
        assert_eq!(found.trained_words_sha256, found.served_words_sha256);
        assert_eq!(found.forced_words_sha256, found.served_words_sha256);
        let mut emptied = Reader::from_bytes(&bytes)?;
        emptied.asked.rows.clear();
        let other = replay(&reader, &emptied, prompts.iter().map(Vec::as_slice), 8);
        assert!(other.prompts_served_differs_from_trained > 0);
        assert_ne!(other.trained_words_sha256, other.served_words_sha256);
        assert_eq!(other.trained_words_sha256, found.trained_words_sha256);
        Ok(())
    }

    /// A fourteen-token question whose first and last words are new: no one
    /// gap of twelve covers both, so no token form is kept. With the new
    /// words in the classes of the TRAIN words the class form answers.
    #[test]
    fn a_question_in_new_words_of_known_classes_is_answered() -> Result<()> {
        let [bos, eos, is, dot, mark, you, grew] = [0u32, 1, 2, 3, 6, 8, 9];
        let asked = |first: u32, x: u32, last: u32| {
            let mut turn = vec![first, 11, 12, 13, 14, 15, x, 16, 17, 18, 19, 20, last];
            turn.push(mark);
            turn
        };
        let mut tokens = Vec::new();
        let mut eligible = Vec::new();
        for index in 0..12u32 {
            let at = tokens.len();
            let [x, v] = [30 + index, 44 + index];
            tokens.extend([bos, x, is, v, dot, eos]);
            tokens.extend(asked(10, x, 21));
            tokens.extend([you, grew, v, dot, eos]);
            eligible.extend(at + 20..at + 25);
        }
        let mut codes: Vec<u16> = (0..64).collect();
        codes[22] = codes[10];
        codes[23] = codes[21];
        let mut prompt = vec![bos, 60, is, 61, dot, eos];
        prompt.extend(asked(22, 60, 23));
        let right = vec![you, grew, 61, dot, eos];

        let train = |codes: &[u16]| {
            Reader::train(
                &tokens,
                &eligible,
                64,
                192,
                bos,
                eos,
                2,
                &[],
                8,
                codes,
                StandBy::Latest,
                BindTable::default(),
                false,
                Reads::default(),
            )
        };
        let (plain, _) = train(&[])?;
        assert!(!plain.asked.reads_classes(&prompt));
        assert_ne!(plain.reply(&prompt, 8), right);
        let (plain_bytes, _) = plain.to_bytes(1);
        assert_eq!(&plain_bytes[..8], MAGIC);

        let (reader, _) = train(&codes)?;
        assert!(reader.asked.reads_classes(&prompt));
        assert_eq!(reader.reply(&prompt, 8), right);
        // A question in TRAIN's own words is read under its token forms.
        let mut known = vec![bos, 60, is, 61, dot, eos];
        known.extend(asked(10, 60, 21));
        assert!(!reader.asked.reads_classes(&known));
        assert_eq!(reader.reply(&known, 8), plain.reply(&known, 8));
        assert_eq!(reader.reply(&known, 8), right);

        let (bytes, saved) = reader.to_bytes(1);
        assert_eq!(&bytes[..8], MAGIC_CODES);
        let loaded = Reader::from_bytes(&bytes)?;
        assert_eq!(loaded.asked.codes(), &codes[..]);
        assert_eq!(loaded.reply(&prompt, 8), right);
        assert_eq!(loaded.to_bytes(1), (bytes, saved));
        Ok(())
    }

    /// "My friend is v. / Good friend. / My dog is w. / Who is the friend?"
    /// The latest stand of `friend` is the echo, with no value after it, and
    /// the latest `is` leads to the dog. Only the stand where `friend` and
    /// `is` hold together reads the friend's name.
    #[test]
    fn a_value_is_read_where_relation_and_is_stand_together() -> Result<()> {
        let [bos, eos, is, dot, who, qm, mark] = [0u32, 1, 2, 3, 4, 5, 6];
        let [my, friend, dog, good, your, the] = [7u32, 8, 9, 10, 11, 12];
        // The two facts come in either order and up to two small turns
        // stand before the question, so no fixed place holds the name.
        let told = |v: u32, w: u32, dog_first: bool, turns: usize| {
            let mut record = vec![bos];
            if dog_first {
                record.extend([my, dog, is, w, dot, eos]);
            }
            record.extend([my, friend, is, v, dot, eos, good, friend, dot, eos]);
            if !dog_first {
                record.extend([my, dog, is, w, dot, eos]);
            }
            for _ in 0..turns {
                record.extend([good, dot, eos]);
            }
            record.extend([who, is, the, friend, qm, mark]);
            record
        };
        let mut tokens = Vec::new();
        let mut eligible = Vec::new();
        for index in 0..18u32 {
            let dog_first = index % 2 == 1;
            tokens.extend(told(20 + index, 40 + index, dog_first, index as usize % 3));
            let at = tokens.len();
            // The reply does not say "friend is" again, so the copy read
            // has no pair to find.
            tokens.extend([your, 20 + index, dot, eos]);
            eligible.extend(at..at + 4);
        }
        let prompt = told(60, 61, false, 2);
        let right = vec![your, 60, dot, eos];
        // A codebook of four block vectors: token t has blocks t%4, t/4%4,
        // t/16, 0, so every token has its own word.
        let rows: [[i32; 8]; 4] = [
            [2, 2, 0, 0, 0, 0, 0, 0],
            [0, 2, -2, 0, 0, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 2, 0],
            [2, 0, 0, 0, 0, -2, 0, 0],
        ];
        let table: Vec<i32> = (0..64usize)
            .flat_map(|t| [rows[t % 4], rows[t / 4 % 4], rows[t / 16], rows[0]])
            .flatten()
            .collect();
        let bind = BindTable::from_codebook(&table, 64, 4)?;
        let train = |by: StandBy, bind: BindTable| {
            Reader::train(
                &tokens,
                &eligible,
                64,
                192,
                bos,
                eos,
                2,
                &[],
                8,
                &[],
                by,
                bind,
                false,
                Reads::default(),
            )
        };

        let (latest, _) = train(StandBy::Latest, BindTable::default())?;
        assert_ne!(latest.reply(&prompt, 8), right);
        let (latest_bytes, _) = latest.to_bytes(1);
        assert_eq!(&latest_bytes[..8], MAGIC);

        for (by, bind) in [
            (StandBy::Pair, BindTable::default()),
            (StandBy::Octonion, bind),
        ] {
            let (reader, _) = train(by, bind)?;
            assert_eq!(reader.reply(&prompt, 8), right, "{by:?}");
            let (bytes, saved) = reader.to_bytes(1);
            assert_eq!(&bytes[..8], MAGIC_STAND);
            let loaded = Reader::from_bytes(&bytes)?;
            assert_eq!(loaded.asked.stand_by(), by);
            assert_eq!(loaded.reply(&prompt, 8), right, "{by:?} loaded");
            assert_eq!(loaded.to_bytes(1), (bytes, saved));
        }
        assert!(train(StandBy::Octonion, BindTable::default()).is_err());
        Ok(())
    }

    /// "User: say exactly this: ▢" with thirty or more tokens to say is longer
    /// than a form holds, so no form over its last thirty tokens is ever seen
    /// twice. The open form keeps the turn's opening and closing tokens, and
    /// under it the reply is the copy of what stands between.
    #[test]
    fn a_long_turn_is_held_by_its_opening_and_copied() -> Result<()> {
        let [bos, eos, user, colon, say, exactly, this, line, mark] =
            [0u32, 1, 2, 3, 4, 5, 6, 7, 8];
        let [good, dot] = [9u32, 10];
        let mut state = 12_345u32;
        let mut said = |length: usize| -> Vec<u32> {
            (0..length)
                .map(|_| {
                    state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                    20 + (state >> 16) % 44
                })
                .collect()
        };
        // The turn opens a record or follows a small turn, and what is to be
        // said has no fixed length.
        let asked = |payload: &[u32], after_a_turn: bool| {
            let mut record = vec![bos];
            if after_a_turn {
                record.extend([
                    user, colon, good, dot, line, mark, colon, good, dot, eos, line,
                ]);
            }
            record.extend([user, colon, say, exactly, this, colon]);
            record.extend(payload);
            record.extend([line, mark, colon]);
            record
        };
        let mut tokens = Vec::new();
        let mut eligible = Vec::new();
        for index in 0..24usize {
            let payload = said(if index % 6 == 5 {
                13 + index % 5
            } else {
                28 + index % 9
            });
            tokens.extend(asked(&payload, index % 2 == 1));
            let at = tokens.len();
            tokens.extend(&payload);
            tokens.push(eos);
            eligible.extend(at..at + payload.len() + 1);
        }
        let train = |open: bool| {
            Reader::train(
                &tokens,
                &eligible,
                64,
                192,
                bos,
                eos,
                2,
                &[],
                48,
                &[],
                StandBy::Latest,
                BindTable::default(),
                open,
                Reads::default(),
            )
        };
        let (closed, _) = train(false)?;
        let (reader, _) = train(true)?;
        let (bytes, saved) = reader.to_bytes(1);
        assert_eq!(&bytes[..8], MAGIC);
        let loaded = Reader::from_bytes(&bytes)?;
        assert_eq!(loaded.to_bytes(1), (bytes, saved));
        for (length, after_a_turn) in [
            (30, false),
            (33, true),
            (36, false),
            (29, true),
            (14, false),
        ] {
            let payload = said(length);
            let prompt = asked(&payload, after_a_turn);
            let mut right = payload.clone();
            right.push(eos);
            assert!(!closed.asked.reads_open(&prompt));
            assert_ne!(closed.reply(&prompt, 48), right, "{length} closed");
            assert!(reader.asked.reads_open(&prompt));
            assert_eq!(reader.reply(&prompt, 48), right, "{length}");
            assert_eq!(loaded.reply(&prompt, 48), right, "{length} loaded");
        }
        Ok(())
    }

    /// A reader with read choices on is written as an 04 file with its
    /// wrong counts and reads back to the same bytes and replies; with none
    /// on the file is the 01 file it always was.
    #[test]
    fn read_choices_are_saved_with_the_wrong_counts() -> Result<()> {
        let [bos, eos, name, is, dot, what, mark] = [0u32, 1, 2, 3, 4, 5, 6];
        let mut tokens = Vec::new();
        let mut eligible = Vec::new();
        for index in 0..16u32 {
            // Half the values are one fixed token, so its token slot is right
            // half the times it speaks.
            let value = if index % 2 == 0 { 9 } else { 20 + index };
            tokens.extend([bos, name, is, value, dot, eos, what, name, mark]);
            let at = tokens.len();
            tokens.extend([value, dot, eos]);
            eligible.extend(at..at + 3);
        }
        let prompt = vec![bos, name, is, 50, dot, eos, what, name, mark];
        let train = |reads: Reads| {
            Reader::train(
                &tokens,
                &eligible,
                64,
                192,
                bos,
                eos,
                2,
                &[],
                8,
                &[],
                StandBy::Latest,
                BindTable::default(),
                false,
                reads,
            )
        };
        let (plain, _) = train(Reads::default())?;
        let (plain_bytes, _) = plain.to_bytes(1);
        assert_eq!(&plain_bytes[..8], MAGIC);
        for bits in 1..32u32 {
            let reads = Reads {
                precise: bits & 1 != 0,
                explain: bits & 2 != 0,
                tail_row: bits & 4 != 0,
                across: bits & 8 != 0,
                agree: bits & 16 != 0,
            };
            let (reader, _) = train(reads)?;
            assert_eq!(reader.reply(&prompt, 8), vec![50, dot, eos], "{reads:?}");
            let (bytes, saved) = reader.to_bytes(1);
            assert_eq!(&bytes[..8], MAGIC_READS);
            let loaded = Reader::from_bytes(&bytes)?;
            assert_eq!(loaded.asked.reads(), reads);
            assert_eq!(loaded.reply(&prompt, 8), vec![50, dot, eos]);
            assert_eq!(loaded.to_bytes(1), (bytes, saved));
        }
        Ok(())
    }

    #[test]
    fn a_changed_or_cut_file_is_refused() -> Result<()> {
        let (reader, _) = train()?;
        let (bytes, _) = reader.to_bytes(1);
        for place in [0, 9, bytes.len() / 2, bytes.len() - 1] {
            let mut changed = bytes.clone();
            changed[place] ^= 1;
            assert!(Reader::from_bytes(&changed).is_err());
        }
        assert!(Reader::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        assert!(Reader::from_bytes(&bytes[..40]).is_err());
        Ok(())
    }
}
