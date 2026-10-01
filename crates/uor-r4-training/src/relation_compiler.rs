//! E3 of the log-sieve design (`docs/integration/log-sieve-retrieval-design-2026-10-01.md`
//! §2.3): can a learned head name the relation and the act of a user turn
//! under paraphrase? This is the language-to-atom compiler #958 lacked.
//!
//! Every user turn of M-world v2 is labelled from the generator's typed intent:
//! its relation (one of [`relation_names`], or [`NONE`]) and its act
//! (assert, update, query, or [`NONE`]; a query of an unstated relation is a
//! query). Two softmax classifiers are fitted on turns drawn with *training*
//! phrasings and scored on turns drawn with *development* phrasings, whose
//! templates the fit never sees:
//!
//! - **trunk**: the frozen stack's final normalized states for the turn read
//!   alone (BOS, the user prefix and the assistant marker), last position and
//!   mean over positions;
//! - **lexical**: the words of the turn (a control: development phrasings
//!   change content words, so words unseen in training carry nothing).
//!
//! The fit is full-batch gradient descent on standardized features, in f64,
//! with no randomness, so a run is a function of its inputs.

use std::collections::BTreeMap;

use serde_json::{json, Value};
use uor_r4_tokenizer::dialogue::DialogueEncoder;

use crate::geometric_stack::StackModel;
use crate::milestone_world::words;
use crate::milestone_world_v2::{relation_names, Cell, MWorld2, Turn2};
use crate::stack_tracking::Rng;
use crate::{invalid, Result};

/// The label of a turn that states, updates or asks no relation.
pub const NONE: &str = "none";
/// The acts, in label order.
pub const ACTS: [&str; 4] = ["assert", "update", "query", NONE];

/// A user turn and its labels.
#[derive(Clone, Debug)]
pub struct Example {
    pub text: String,
    pub relation: String,
    pub act: &'static str,
}

/// A turn's relation and act from its typed intent.
pub fn label(turn: &Turn2) -> (String, &'static str) {
    for name in relation_names() {
        if let Some(act) = turn
            .intent
            .strip_prefix(name)
            .and_then(|rest| rest.strip_prefix('_'))
        {
            let act = match act {
                "assert" => "assert",
                "update" => "update",
                "query" | "absent" => "query",
                _ => continue,
            };
            return (name.to_owned(), act);
        }
    }
    (NONE.to_owned(), NONE)
}

/// Every user turn of `conversations` episodes of `cell`, labelled.
pub fn collect(
    world: &mut MWorld2<'_>,
    rng: &mut Rng,
    cell: Cell,
    conversations: usize,
) -> Result<Vec<Example>> {
    let mut examples = Vec::new();
    for _ in 0..conversations {
        for turn in world.conversation_in(rng, cell)?.turns {
            let (relation, act) = label(&turn);
            examples.push(Example {
                text: turn.user,
                relation,
                act,
            });
        }
    }
    Ok(examples)
}

/// The frozen trunk's features of a user turn read alone: the final
/// normalized state at the last position (the assistant marker) and the mean
/// over all positions, `2 * width` values.
pub fn trunk_features(
    model: &StackModel,
    encoder: &DialogueEncoder<'_>,
    bos: u32,
    text: &str,
) -> Result<Vec<f64>> {
    let prefix = encoder.encode_user_prefix(text, false);
    if prefix.emitted_turns != 1 || prefix.special_token_occurrences != 0 {
        return Err(invalid("a user turn did not encode as one plain turn"));
    }
    let mut ids = vec![bos];
    ids.extend(&prefix.tokens);
    if ids.len() > model.config.context {
        return Err(invalid("a user turn is longer than the model's context"));
    }
    let width = model.config.width;
    let states = model
        .hidden(&ids, 1, ids.len())?
        .flatten_all()?
        .to_vec1::<f32>()?;
    let rows = states.len() / width;
    let mut features = vec![0f64; 2 * width];
    for (i, value) in states[(rows - 1) * width..].iter().enumerate() {
        features[i] = f64::from(*value);
    }
    for row in states.chunks(width) {
        for (i, value) in row.iter().enumerate() {
            features[width + i] += f64::from(*value) / rows as f64;
        }
    }
    Ok(features)
}

/// Binary bag-of-words features over a fixed vocabulary.
pub struct Lexicon {
    index: BTreeMap<String, usize>,
}

impl Lexicon {
    /// The vocabulary of `texts` (lowercased words without punctuation).
    pub fn fit<'a>(texts: impl IntoIterator<Item = &'a str>) -> Self {
        let mut index = BTreeMap::new();
        for text in texts {
            for word in words(text) {
                let next = index.len();
                index.entry(word).or_insert(next);
            }
        }
        Self { index }
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// The features of `text`; words outside the vocabulary carry nothing.
    pub fn features(&self, text: &str) -> Vec<f64> {
        let mut features = vec![0f64; self.index.len()];
        for word in words(text) {
            if let Some(&i) = self.index.get(&word) {
                features[i] = 1.0;
            }
        }
        features
    }
}

/// A softmax classifier on standardized features.
pub struct Softmax {
    classes: usize,
    dim: usize,
    mean: Vec<f64>,
    scale: Vec<f64>,
    weights: Vec<f64>,
    bias: Vec<f64>,
}

impl Softmax {
    /// Fit by `steps` full-batch gradient steps of size `rate` on the mean
    /// cross-entropy plus `l2` times the squared weights.
    pub fn fit(
        x: &[Vec<f64>],
        y: &[usize],
        classes: usize,
        steps: usize,
        rate: f64,
        l2: f64,
    ) -> Result<Self> {
        let dim = x.first().map_or(0, Vec::len);
        if x.is_empty()
            || x.len() != y.len()
            || classes < 2
            || y.iter().any(|&label| label >= classes)
            || x.iter().any(|row| row.len() != dim)
        {
            return Err(invalid("a classifier needs labelled rows of one width"));
        }
        let n = x.len() as f64;
        let mut mean = vec![0f64; dim];
        for row in x {
            for (m, v) in mean.iter_mut().zip(row) {
                *m += v / n;
            }
        }
        let mut scale = vec![0f64; dim];
        for row in x {
            for ((s, v), m) in scale.iter_mut().zip(row).zip(&mean) {
                *s += (v - m) * (v - m) / n;
            }
        }
        for s in &mut scale {
            *s = if *s > 1e-12 { s.sqrt() } else { 1.0 };
        }
        let mut model = Self {
            classes,
            dim,
            mean,
            scale,
            weights: vec![0f64; classes * dim],
            bias: vec![0f64; classes],
        };
        let standardized: Vec<Vec<f64>> = x.iter().map(|row| model.standardize(row)).collect();
        for _ in 0..steps {
            let mut grad_w = vec![0f64; classes * dim];
            let mut grad_b = vec![0f64; classes];
            for (row, &label) in standardized.iter().zip(y) {
                let p = model.probabilities_standardized(row);
                for c in 0..classes {
                    let g = (p[c] - f64::from(u8::from(c == label))) / n;
                    grad_b[c] += g;
                    for (w, v) in grad_w[c * dim..(c + 1) * dim].iter_mut().zip(row) {
                        *w += g * v;
                    }
                }
            }
            for (w, g) in model.weights.iter_mut().zip(&grad_w) {
                *w -= rate * (g + l2 * *w);
            }
            for (b, g) in model.bias.iter_mut().zip(&grad_b) {
                *b -= rate * g;
            }
        }
        Ok(model)
    }

    fn standardize(&self, row: &[f64]) -> Vec<f64> {
        row.iter()
            .zip(&self.mean)
            .zip(&self.scale)
            .map(|((v, m), s)| (v - m) / s)
            .collect()
    }

    fn probabilities_standardized(&self, row: &[f64]) -> Vec<f64> {
        let logits: Vec<f64> = (0..self.classes)
            .map(|c| {
                self.bias[c]
                    + self.weights[c * self.dim..(c + 1) * self.dim]
                        .iter()
                        .zip(row)
                        .map(|(w, v)| w * v)
                        .sum::<f64>()
            })
            .collect();
        let top = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let exp: Vec<f64> = logits.iter().map(|l| (l - top).exp()).collect();
        let total: f64 = exp.iter().sum();
        exp.iter().map(|e| e / total).collect()
    }

    /// The most probable class of a raw feature row (ties to the lower class).
    pub fn predict(&self, row: &[f64]) -> usize {
        let p = self.probabilities_standardized(&self.standardize(row));
        let mut best = 0;
        for c in 1..p.len() {
            if p[c] > p[best] {
                best = c;
            }
        }
        best
    }
}

/// Accuracy of `predicted` against `truth` over the rows `keep` selects,
/// with per-label tallies and the most frequent confusions.
pub fn score(
    labels: &[String],
    truth: &[usize],
    predicted: &[usize],
    keep: impl Fn(usize) -> bool,
) -> Value {
    let (mut pass, mut of) = (0usize, 0usize);
    let mut per: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    let mut confusions: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for (&t, &p) in truth.iter().zip(predicted) {
        if !keep(t) {
            continue;
        }
        of += 1;
        let tally = per.entry(labels[t].as_str()).or_default();
        tally.1 += 1;
        if t == p {
            pass += 1;
            tally.0 += 1;
        } else {
            *confusions
                .entry((labels[t].as_str(), labels[p].as_str()))
                .or_default() += 1;
        }
    }
    let mut top: Vec<((&str, &str), usize)> = confusions.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    top.truncate(8);
    json!({
        "pass": pass,
        "of": of,
        "rate": if of == 0 { 0.0 } else { pass as f64 / of as f64 },
        "per_label": per
            .iter()
            .map(|(label, (p, o))| (label.to_string(), json!({"pass": p, "of": o})))
            .collect::<BTreeMap<_, _>>(),
        "top_confusions": top
            .iter()
            .map(|((t, p), n)| json!({"truth": t, "predicted": p, "count": n}))
            .collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::milestone_world_v2::{Category2, Tag};

    fn turn(intent: &str) -> Turn2 {
        Turn2 {
            intent: intent.into(),
            category: Category2::Relation,
            user: String::new(),
            reply: String::new(),
            checks: Vec::new(),
            tag: Tag::default(),
        }
    }

    #[test]
    fn turns_are_labelled_from_their_typed_intent() {
        assert_eq!(
            label(&turn("friend_name_assert")),
            ("friend_name".into(), "assert")
        );
        assert_eq!(
            label(&turn("user_name_update")),
            ("user_name".into(), "update")
        );
        assert_eq!(
            label(&turn("lucky_number_query")),
            ("lucky_number".into(), "query")
        );
        // A query of a relation never stated is still a query of it.
        assert_eq!(
            label(&turn("hometown_absent")),
            ("hometown".into(), "query")
        );
        for other in ["mqar_query", "mqar_assert", "copy", "capital", "thanks"] {
            assert_eq!(label(&turn(other)), (NONE.into(), NONE), "{other}");
        }
        assert!(relation_names().len() >= 10);
    }

    #[test]
    fn the_softmax_fit_separates_classes_and_is_deterministic() -> Result<()> {
        let x: Vec<Vec<f64>> = (0..30)
            .map(|i| {
                let class = i % 3;
                vec![class as f64 + 0.1 * (i % 5) as f64, (i % 7) as f64]
            })
            .collect();
        let y: Vec<usize> = (0..30).map(|i| i % 3).collect();
        let a = Softmax::fit(&x, &y, 3, 400, 0.5, 1e-4)?;
        let b = Softmax::fit(&x, &y, 3, 400, 0.5, 1e-4)?;
        let predicted: Vec<usize> = x.iter().map(|row| a.predict(row)).collect();
        assert_eq!(
            predicted,
            x.iter().map(|row| b.predict(row)).collect::<Vec<_>>()
        );
        assert_eq!(predicted, y);
        assert!(Softmax::fit(&x, &y[..29], 3, 1, 0.5, 0.0).is_err());
        let labels: Vec<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        let report = score(&labels, &y, &predicted, |label| label != 2);
        assert_eq!(report["of"], 20);
        assert_eq!(report["rate"], 1.0);
        Ok(())
    }

    #[test]
    fn the_lexicon_ignores_words_it_never_saw() {
        let lexicon = Lexicon::fit(["My friend is Sam.", "What is my name?"]);
        assert_eq!(lexicon.len(), 6);
        let features = lexicon.features("My buddy is Tam.");
        assert_eq!(features.iter().filter(|&&v| v == 1.0).count(), 2);
    }
}
