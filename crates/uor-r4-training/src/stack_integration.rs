//! The Stage 2 integration's shared pieces (Lab 1; ROADMAP restart packet):
//! the dialogue model with the memory port, trained on literal-role dialogue
//! corpora (chat-v0, M-world) beside a relation world.
//!
//! This module starts with the user-turn flags the memory port's gate needs
//! for any literal-role token stream. The DeepSeek lab's gate (#1502) takes a
//! per-position flag: memory operations happen only where it is 1. Its world
//! episodes carry flags from their builder; corpus dialogues and generation
//! histories carry only tokens, so the flags are recovered here from the
//! protocol's marker sequences.

use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::{invalid, Result};

/// The literal-role protocol's marker token sequences, as its encoder emits
/// them: each separator and marker is encoded on its own
/// (`uor_r4_tokenizer::dialogue::DialogueEncoder`), so the sequences occur
/// verbatim in every rendered dialogue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnMarkers {
    bos: u32,
    eos: u32,
    separator: Vec<u32>,
    user: Vec<u32>,
    assistant: Vec<u32>,
    system: Vec<u32>,
}

/// Where the scan is: between turns, or inside one role's content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    /// Before any marker of a window that may start mid-turn: nothing is
    /// flagged until a marker is seen.
    Unknown,
    Boundary,
    User,
    System,
    Assistant,
}

impl TurnMarkers {
    /// The markers of the literal-role protocol under `tokenizer`.
    pub fn new(tokenizer: &ByteBpeTokenizer, bos: u32, eos: u32) -> Result<Self> {
        let encode = |text: &str| -> Result<Vec<u32>> {
            let ids = tokenizer.encode(text);
            if ids.is_empty() {
                return Err(invalid(format!("the marker {text:?} encodes to nothing")));
            }
            Ok(ids)
        };
        Ok(Self {
            bos,
            eos,
            separator: encode("\n")?,
            user: encode("User: ")?,
            assistant: encode("Assistant: ")?,
            system: encode("System: ")?,
        })
    }

    fn at(tokens: &[u32], position: usize, sequence: &[u32]) -> bool {
        tokens
            .get(position..position + sequence.len())
            .is_some_and(|window| window == sequence)
    }

    /// The role marker starting at `position`, and the state it opens.
    fn marker(&self, tokens: &[u32], position: usize) -> Option<(usize, State)> {
        [
            (&self.user, State::User),
            (&self.assistant, State::Assistant),
            (&self.system, State::System),
        ]
        .into_iter()
        .find(|(sequence, _)| Self::at(tokens, position, sequence))
        .map(|(sequence, state)| (sequence.len(), state))
    }

    /// 1 at every token of user-turn content, 0 elsewhere: at BOS, EOS,
    /// separators, role markers, system and assistant content (including a
    /// reply being generated), and before the first marker of a window that
    /// starts mid-turn.
    ///
    /// A user or system turn ends at a separator followed by a role marker,
    /// or at EOS (a document that ends on a non-assistant turn). An
    /// assistant turn ends only at its EOS, so marker text inside a reply is
    /// content.
    pub fn user_turn_flags(&self, tokens: &[u32]) -> Vec<u8> {
        let mut flags = vec![0u8; tokens.len()];
        let mut state = State::Unknown;
        let mut position = 0;
        while position < tokens.len() {
            let token = tokens[position];
            if token == self.bos {
                state = State::Boundary;
                position += 1;
                continue;
            }
            match state {
                State::Unknown | State::Boundary => {
                    if let Some((length, next)) = self.marker(tokens, position) {
                        state = next;
                        position += length;
                    } else {
                        if Self::at(tokens, position, &self.separator) {
                            position += self.separator.len();
                        } else {
                            position += 1;
                        }
                    }
                }
                State::User | State::System => {
                    if token == self.eos {
                        state = State::Boundary;
                        position += 1;
                    } else if Self::at(tokens, position, &self.separator)
                        && self
                            .marker(tokens, position + self.separator.len())
                            .is_some()
                    {
                        state = State::Boundary;
                        position += self.separator.len();
                    } else {
                        if state == State::User {
                            flags[position] = 1;
                        }
                        position += 1;
                    }
                }
                State::Assistant => {
                    if token == self.eos {
                        state = State::Boundary;
                    }
                    position += 1;
                }
            }
        }
        flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};

    /// GPT-2's byte-to-character alphabet.
    fn alphabet() -> Vec<char> {
        let mut printable: Vec<u32> = (u32::from(b'!')..=u32::from(b'~')).collect();
        printable.extend(0xA1..=0xAC);
        printable.extend(0xAE..=0xFF);
        let mut table = vec!['\0'; 256];
        let mut extra = 0;
        for byte in 0u32..256 {
            table[byte as usize] = if printable.contains(&byte) {
                char::from_u32(byte).unwrap()
            } else {
                extra += 1;
                char::from_u32(255 + extra).unwrap()
            };
        }
        table
    }

    /// A byte-level tokenizer with the three dialogue specials at ids 0-2.
    fn tokenizer() -> ByteBpeTokenizer {
        let mut vocab = serde_json::Map::new();
        for (id, surface) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
            vocab.insert((*surface).to_owned(), json!(id));
        }
        for (byte, ch) in alphabet().iter().enumerate() {
            vocab.insert(ch.to_string(), json!(byte + 3));
        }
        let added: Vec<Value> = ["<|bos|>", "<|eos|>", "<|unk|>"]
            .iter()
            .enumerate()
            .map(|(id, surface)| json!({"id": id, "content": surface}))
            .collect();
        ByteBpeTokenizer::from_tokenizer_json_bytes(
            json!({
                "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
                "added_tokens": added,
                "model": {"type": "BPE", "vocab": vocab, "merges": []},
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
    }

    fn user_text(tokenizer: &ByteBpeTokenizer, tokens: &[u32], flags: &[u8]) -> String {
        let flagged: Vec<u32> = tokens
            .iter()
            .zip(flags)
            .filter(|(_, &f)| f == 1)
            .map(|(&t, _)| t)
            .collect();
        tokenizer.decode(&flagged)
    }

    #[test]
    fn flags_mark_exactly_the_user_content() {
        let tokenizer = tokenizer();
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let markers = TurnMarkers::new(&tokenizer, protocol.bos_id, protocol.eos_id).unwrap();
        let messages = [
            Message {
                role: "system",
                content: "Be kind.",
            },
            Message {
                role: "user",
                content: "My name is Ruby.\nIt is a nice name.",
            },
            Message {
                role: "assistant",
                content: "User: this is a reply, not a user turn.",
            },
            Message {
                role: "user",
                content: "What is my name?",
            },
            Message {
                role: "assistant",
                content: "Ruby.",
            },
        ];
        let document = encoder.encode_document(&messages);
        let flags = markers.user_turn_flags(&document.tokens);
        // Only the two user turns' content, with the multi-line turn whole.
        assert_eq!(
            user_text(&tokenizer, &document.tokens, &flags),
            "My name is Ruby.\nIt is a nice name.What is my name?"
        );
        // No reply position is ever flagged.
        for (flag, mask) in flags.iter().zip(&document.response_mask) {
            assert!(!(*flag == 1 && *mask == 1));
        }
    }

    #[test]
    fn a_generation_prompt_flags_the_last_user_turn_and_not_the_reply() {
        let tokenizer = tokenizer();
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let markers = TurnMarkers::new(&tokenizer, protocol.bos_id, protocol.eos_id).unwrap();
        let prompt = encoder.encode_assistant_prefix(&[Message {
            role: "user",
            content: "Hi there!",
        }]);
        let mut tokens = prompt.tokens.clone();
        let reply = tokenizer.encode("Hello! User: again");
        tokens.extend(&reply);
        let flags = markers.user_turn_flags(&tokens);
        assert_eq!(user_text(&tokenizer, &tokens, &flags), "Hi there!");
        assert!(flags[prompt.tokens.len()..].iter().all(|&f| f == 0));
    }

    #[test]
    fn a_window_that_starts_mid_turn_waits_for_a_marker() {
        let tokenizer = tokenizer();
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let markers = TurnMarkers::new(&tokenizer, protocol.bos_id, protocol.eos_id).unwrap();
        let document = encoder.encode_document(&[
            Message {
                role: "user",
                content: "First question here.",
            },
            Message {
                role: "assistant",
                content: "An answer.",
            },
            Message {
                role: "user",
                content: "Second one.",
            },
            Message {
                role: "assistant",
                content: "Done.",
            },
        ]);
        // Start inside the first user turn, after BOS and its marker.
        let window = &document.tokens[10..];
        let flags = markers.user_turn_flags(window);
        assert_eq!(user_text(&tokenizer, window, &flags), "Second one.");
    }
}
