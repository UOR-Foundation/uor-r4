//! The trunk's position transport as the key of a question form.
//!
//! The D11 trunk carries a frame along a prefix, one block at a time:
//! `F_p = COMPOSE[step(block, root(token_p, block)), F_{p-1}]`, from the
//! identity frame. A [`TrunkKey`] holds the three tables that rule needs (a
//! token's root in each block, the step frame of each root, the composition
//! of two frames), read from the trunk artifact, and gives the frame the
//! trunk would hold after the kept tokens of an asked turn. A reader built
//! with one keys its question forms by that frame and not by the tokens, so
//! it cannot be read without the trunk it was written beside.
//!
//! Integer arithmetic only: table lookups over bytes.

use super::invalid;
use crate::Result;
use sha2::{Digest, Sha256};

/// Blocks of a trunk token.
pub const BLOCKS: usize = 4;
/// Orbit roots a block takes.
pub const ROOTS: usize = 16;
/// Frames of the transport group.
pub const FRAMES: usize = 64;
/// The frame that leaves a block as it is.
pub const IDENTITY: u8 = 0;

/// What of the trunk state a form is keyed by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Keyed {
    /// The frame held after the last kept token.
    #[default]
    Frame,
    /// The frame held after every kept token. The steps between them say
    /// the same, so each kept token gives the code of its four step frames.
    Path,
    /// The trunk address of every kept token: its root in each block.
    Address,
}

impl Keyed {
    pub const ALL: [Self; 3] = [Self::Frame, Self::Path, Self::Address];
}

/// The trunk tables a form's key is read from.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TrunkKey {
    /// A token's root in each block: the trunk's address of the token.
    pub roots: Vec<[u8; BLOCKS]>,
    /// `steps[block * 16 + root]`: the frame that root steps the block by.
    pub steps: Vec<u8>,
    /// `compose[a * 64 + b]`: the frame `a` after `b`.
    pub compose: Vec<u8>,
    /// What a form is keyed by.
    #[serde(default)]
    pub by: Keyed,
    /// A token's code under [`Keyed::Path`] or [`Keyed::Address`], filled by
    /// [`TrunkKey::checked`]: the place of its step frames, or of its roots,
    /// among the distinct ones in order.
    #[serde(skip)]
    pub codes: Vec<u16>,
}

/// Each row's place among the distinct rows in order; twelve bits hold it.
fn ranked(rows: Vec<[u8; BLOCKS]>) -> Result<Vec<u16>> {
    let distinct: std::collections::BTreeSet<[u8; BLOCKS]> = rows.iter().copied().collect();
    if distinct.len() > 1 << 12 {
        return Err(invalid("a trunk key code is past twelve bits"));
    }
    let place: std::collections::BTreeMap<[u8; BLOCKS], u16> = distinct
        .into_iter()
        .enumerate()
        .map(|(at, row)| (row, at as u16))
        .collect();
    Ok(rows.iter().map(|row| place[row]).collect())
}

impl TrunkKey {
    /// The same tables keyed by `by`.
    pub fn keyed(mut self, by: Keyed) -> Result<Self> {
        self.by = by;
        self.checked()
    }

    /// A kept token's code where forms are keyed a token at a time, twelve
    /// bits; none under [`Keyed::Frame`]. A token past the table has the
    /// code of token zero.
    pub fn code(&self, token: u32) -> Option<u32> {
        let first = *self.codes.first()?;
        Some(u32::from(*self.codes.get(token as usize).unwrap_or(&first)))
    }

    /// Refuses tables of another size, a value past its range, or a
    /// composition under which [`IDENTITY`] is not the identity.
    pub fn checked(mut self) -> Result<Self> {
        if self.roots.is_empty() {
            return Err(invalid("a trunk key needs a root row for every token"));
        }
        if self.steps.len() != BLOCKS * ROOTS || self.compose.len() != FRAMES * FRAMES {
            return Err(invalid(
                "a trunk key's step or composition table has another size",
            ));
        }
        if self
            .roots
            .iter()
            .flatten()
            .any(|&root| usize::from(root) >= ROOTS)
        {
            return Err(invalid("a trunk key root is past the sixteen orbit roots"));
        }
        if self
            .steps
            .iter()
            .chain(&self.compose)
            .any(|&frame| usize::from(frame) >= FRAMES)
        {
            return Err(invalid("a trunk key frame is past the sixty-four frames"));
        }
        let identity = usize::from(IDENTITY);
        let neutral = (0..FRAMES).all(|frame| {
            usize::from(self.compose[identity * FRAMES + frame]) == frame
                && usize::from(self.compose[frame * FRAMES + identity]) == frame
        });
        if !neutral {
            return Err(invalid(
                "frame zero is not the identity of the composition table",
            ));
        }
        self.codes = match self.by {
            Keyed::Frame => Vec::new(),
            Keyed::Path => {
                let stepped = self.roots.iter().map(|roots| {
                    std::array::from_fn(|block| {
                        self.steps[block * ROOTS + usize::from(roots[block])]
                    })
                });
                ranked(stepped.collect())?
            }
            Keyed::Address => ranked(self.roots.clone())?,
        };
        Ok(self)
    }

    /// The frames the trunk holds after `kept`, six bits a block. A token
    /// past the table steps by the roots of token zero.
    pub fn frame(&self, kept: impl Iterator<Item = u32>) -> u32 {
        let mut held = [usize::from(IDENTITY); BLOCKS];
        for token in kept {
            let roots = self.roots.get(token as usize).unwrap_or(&self.roots[0]);
            for (block, frame) in held.iter_mut().enumerate() {
                let step = usize::from(self.steps[block * ROOTS + usize::from(roots[block])]);
                *frame = usize::from(self.compose[step * FRAMES + *frame]);
            }
        }
        held.iter().enumerate().fold(0u32, |word, (block, &frame)| {
            word | ((frame as u32) << (6 * block))
        })
    }

    /// SHA-256 of the three tables in order, the root rows first.
    pub fn sha256(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update((self.roots.len() as u32).to_le_bytes());
        for row in &self.roots {
            hash.update(row);
        }
        hash.update(&self.steps);
        hash.update(&self.compose);
        // The frame key keeps the digest it had before the other keys.
        match self.by {
            Keyed::Frame => {}
            Keyed::Path => hash.update([1u8]),
            Keyed::Address => hash.update([2u8]),
        }
        hash.finalize().into()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A toy trunk over `tokens` tokens: the frames are the integers under
    /// addition modulo 64 except that the top bit turns the sum round, so
    /// the composition is not commutative; token `t` has root `(t + block) %
    /// 16` and root `r` of block `b` steps by `1 + r + 16 * b`.
    pub(crate) fn toy(tokens: usize) -> TrunkKey {
        let roots = (0..tokens)
            .map(|token| std::array::from_fn(|block| ((token + block) % ROOTS) as u8))
            .collect();
        let steps = (0..BLOCKS * ROOTS)
            .map(|at| ((1 + at) % FRAMES) as u8)
            .collect();
        let compose = (0..FRAMES * FRAMES)
            .map(|at| {
                let (a, b) = (at / FRAMES, at % FRAMES);
                // The dihedral group of order 64: a low half of turns, and a
                // top bit that mirrors what follows.
                let (turn_a, flip_a) = (a % 32, a / 32);
                let (turn_b, flip_b) = (b % 32, b / 32);
                let turn = if flip_a == 1 {
                    (turn_a + 32 - turn_b) % 32
                } else {
                    (turn_a + turn_b) % 32
                };
                (turn + 32 * (flip_a ^ flip_b)) as u8
            })
            .collect();
        TrunkKey {
            roots,
            steps,
            compose,
            by: Keyed::Frame,
            codes: Vec::new(),
        }
        .checked()
        .expect("the toy trunk key is well formed")
    }

    #[test]
    fn a_path_code_is_the_step_frames_and_an_address_code_the_roots() -> Result<()> {
        // Two roots of a block that step alike: the path cannot tell the
        // tokens apart, the address can.
        let mut key = toy(40);
        for block in 0..BLOCKS {
            key.steps[block * ROOTS + (5 + block) % ROOTS] =
                key.steps[block * ROOTS + (4 + block) % ROOTS];
        }
        let frame = key.clone().checked()?;
        assert_eq!(frame.code(4), None);
        let path = key.clone().keyed(Keyed::Path)?;
        assert_eq!(path.code(4), path.code(5));
        assert_ne!(path.code(4), path.code(6));
        // Tokens sixteen apart share every root in the toy.
        let address = key.keyed(Keyed::Address)?;
        assert_ne!(address.code(4), address.code(5));
        assert_eq!(address.code(4), address.code(20));
        // Tokens that step alike hold the same frame after them.
        assert_eq!(
            frame.frame([4u32].into_iter()),
            frame.frame([5u32].into_iter())
        );
        // Each key has its own digest; the frame key keeps the old one.
        assert_ne!(path.sha256(), frame.sha256());
        assert_ne!(address.sha256(), path.sha256());
        assert_eq!(path.code(4000), path.code(0));
        Ok(())
    }

    #[test]
    fn a_frame_is_the_trunk_rule_one_token_at_a_time() {
        let key = toy(40);
        assert_eq!(key.frame(std::iter::empty()), 0);
        let one = key.frame([7u32].into_iter());
        for block in 0..BLOCKS {
            let step = u32::from(key.steps[block * ROOTS + (7 + block) % ROOTS]);
            assert_eq!((one >> (6 * block)) & 63, step);
        }
        // Order matters: the toy composition does not commute.
        assert_ne!(
            key.frame([20u32, 3].into_iter()),
            key.frame([3u32, 20].into_iter())
        );
        // A second token composes on the left of what is held.
        let two = key.frame([7u32, 9].into_iter());
        for block in 0..BLOCKS {
            let first = (one >> (6 * block)) & 63;
            let step = u32::from(key.steps[block * ROOTS + (9 + block) % ROOTS]);
            let wanted = u32::from(key.compose[(step * 64 + first) as usize]);
            assert_eq!((two >> (6 * block)) & 63, wanted);
        }
    }

    #[test]
    fn a_malformed_key_is_refused_and_the_digest_holds_every_table() {
        let key = toy(8);
        let mut short = key.clone();
        short.steps.pop();
        assert!(short.checked().is_err());
        let mut far = key.clone();
        far.roots[3][1] = 16;
        assert!(far.checked().is_err());
        let mut skew = key.clone();
        skew.compose[5] = 6;
        assert!(skew.checked().is_err());
        let mut other = key.clone();
        other.roots[2][0] ^= 1;
        assert_ne!(other.sha256(), key.sha256());
        let mut stepped = key.clone();
        stepped.steps[9] ^= 1;
        assert_ne!(stepped.sha256(), key.sha256());
    }
}
