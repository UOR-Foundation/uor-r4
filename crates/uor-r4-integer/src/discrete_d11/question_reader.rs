//! The saved question reader carried by a D11 artifact.
//!
//! The reader is the discrete crate's conjunction rows (`UORRDR0x`): the
//! suffix ladder and the held-question rows TRAIN writes. This carrier puts
//! that file, byte for byte, in front of an unchanged parent `UORLUT01`
//! artifact under one header that names both by SHA-256. The loaded model
//! answers a prompt through `uor_r4_reader_runtime`, which is the same source
//! file the trainer compiles, so the read that TRAIN checked and the read
//! served here are one definition.
use super::*;
use std::sync::Arc;
use uor_r4_reader_runtime::conjunction::saved::{Reader, MAGIC_TRUNK};
use uor_r4_reader_runtime::conjunction::trunk_key::TrunkKey;

pub(super) const SCHEMA: &str = "uor-r4.lut-discrete-question-reader/1";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: String,
    parent_sha256: String,
    reader_sha256: String,
    reader_bytes: u64,
}

/// The reader a model carries, with the SHA-256 of its file.
#[derive(Clone, Debug)]
pub(super) struct Carried {
    reader: Arc<Reader>,
    sha256: String,
}

/// Puts a saved reader file in front of a parent D11 artifact. The result is
/// read back before it is returned, so a reader or parent the loader refuses
/// is refused here.
pub fn pack_question_reader(parent: &[u8], reader: &[u8]) -> Result<Vec<u8>> {
    let header = serde_json::to_vec(&Envelope {
        schema: SCHEMA.into(),
        parent_sha256: hex::encode(Sha256::digest(parent)),
        reader_sha256: hex::encode(Sha256::digest(reader)),
        reader_bytes: reader.len() as u64,
    })?;
    let mut bytes = Vec::with_capacity(16 + header.len() + reader.len() + parent.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&(header.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(reader);
    bytes.extend_from_slice(parent);
    IntegerDiscreteModel::from_bytes(&bytes)?;
    Ok(bytes)
}

pub(super) fn read(
    bytes: &[u8],
    header: &[u8],
    header_end: usize,
    allow_role: bool,
) -> Result<IntegerDiscreteModel> {
    if !allow_role {
        return Err(invalid("nested question reader carrier"));
    }
    let envelope: Envelope = serde_json::from_slice(header)?;
    let reader_len = usize::try_from(envelope.reader_bytes)
        .map_err(|_| invalid("question reader length out of range"))?;
    let (reader_file, parent) = bytes[header_end..]
        .split_at_checked(reader_len)
        .ok_or_else(|| invalid("truncated question reader"))?;
    if Sha256::digest(reader_file).as_slice() != read_role_digest(&envelope.reader_sha256)?
        || Sha256::digest(parent).as_slice() != read_role_digest(&envelope.parent_sha256)?
    {
        return Err(invalid("question reader or parent binding mismatch"));
    }
    let mut model = IntegerDiscreteModel::from_bytes(parent)?;
    // A reader keyed by a trunk is read with this parent's own tables, and
    // is refused unless they are the tables it was written beside.
    let trunk = if reader_file.starts_with(MAGIC_TRUNK) {
        Some(model.question_trunk_key()?)
    } else {
        None
    };
    let reader = Reader::from_bytes_beside(reader_file, trunk)
        .map_err(|error| invalid(format!("question reader file: {error}")))?;
    if model.question_reader.is_some() {
        return Err(invalid("nested question reader carrier"));
    }
    if reader.vocabulary != model.shape.vocabulary {
        return Err(invalid(
            "question reader and parent are over different vocabularies",
        ));
    }
    model.question_reader = Some(Carried {
        reader: Arc::new(reader),
        sha256: envelope.reader_sha256,
    });
    model.artifact_sha256 = hex::encode(Sha256::digest(bytes));
    Ok(model)
}

impl IntegerDiscreteModel {
    /// The trunk tables a question reader keys its forms by: each token's
    /// root in the four blocks, the step frame of each root and the
    /// composition of two frames. They are this artifact's position
    /// transport (`F_p = COMPOSE[step(b, root(p, b)), F_{p-1}]`).
    pub fn question_trunk_key(&self) -> Result<TrunkKey> {
        let trunk = self
            .trunk
            .as_ref()
            .ok_or_else(|| invalid("this artifact has no trunk tables"))?;
        if self.codebook_roots.len() != self.shape.vocabulary as usize {
            return Err(invalid("this artifact has no root row for every token"));
        }
        TrunkKey {
            roots: self.codebook_roots.clone(),
            steps: trunk.step_candidates.to_vec(),
            compose: COMPOSE.to_vec(),
        }
        .checked()
        .map_err(|error| invalid(format!("trunk key: {error}")))
    }

    /// The carried question reader, if this artifact has one.
    pub fn question_reader(&self) -> Option<&Reader> {
        self.question_reader.as_ref().map(|c| c.reader.as_ref())
    }

    /// SHA-256 of the carried reader file, as the header binds it.
    pub fn question_reader_sha256(&self) -> Option<&str> {
        self.question_reader.as_ref().map(|c| c.sha256.as_str())
    }

    /// Answers a prompt from the carried reader's rows: at most `limit`
    /// token ids, ending at the reader's end token when it is reached.
    pub fn question_reply(&self, prompt: &[u32], limit: usize) -> Result<Vec<u32>> {
        let reader = self
            .question_reader()
            .ok_or_else(|| invalid("this artifact carries no question reader"))?;
        if prompt.iter().any(|&id| id >= reader.vocabulary) {
            return Err(invalid("a prompt id is past the vocabulary"));
        }
        Ok(reader.reply(prompt, limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_reader_runtime::conjunction::{bind::BindTable, free::Update, Reads, StandBy};

    /// Told: BOS x IS v DOT EOS WHAT x QM MARK | YOU GREW v DOT EOS
    fn told(x: u32, v: u32) -> Vec<u32> {
        vec![0, x, 2, v, 3, 1, 4, x, 5, 6, 8, 9, v, 3, 1]
    }

    fn reader_file() -> Vec<u8> {
        reader_file_beside(None)
    }

    fn reader_file_beside(trunk: Option<TrunkKey>) -> Vec<u8> {
        let mut tokens = Vec::new();
        let mut eligible = Vec::new();
        for index in 0..12u32 {
            let at = tokens.len();
            tokens.extend(told(20 + index, 40 + index));
            eligible.extend(at + 10..at + 15);
        }
        let (reader, _) = Reader::train_by(
            &tokens,
            &eligible,
            64,
            192,
            0,
            1,
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
        )
        .unwrap();
        let (bytes, _) = reader.to_bytes(1);
        bytes
    }

    fn route_artifact(vocabulary: u32) -> Vec<u8> {
        super::super::tests::plain_artifact(vocabulary)
    }

    /// A reader keyed by a trunk is read with the parent's own tables, so a
    /// parent that has none cannot carry it.
    #[test]
    fn a_trunk_keyed_reader_is_refused_behind_a_parent_without_its_trunk() {
        let key = TrunkKey {
            roots: (0..64usize).map(|token| [(token % 16) as u8; 4]).collect(),
            steps: (0..64u8).collect(),
            compose: COMPOSE.to_vec(),
        }
        .checked()
        .unwrap();
        let file = reader_file_beside(Some(key.clone()));
        assert!(file.starts_with(MAGIC_TRUNK));
        let prompt = told(60, 61)[..10].to_vec();
        let beside = Reader::from_bytes_beside(&file, Some(key)).unwrap();
        assert_eq!(beside.reply(&prompt, 8), vec![8, 9, 61, 3, 1]);
        assert!(Reader::from_bytes(&file).is_err());
        let parent = route_artifact(64);
        let plain = IntegerDiscreteModel::from_bytes(&parent).unwrap();
        assert!(plain.question_trunk_key().is_err());
        assert!(pack_question_reader(&parent, &file).is_err());
    }

    #[test]
    fn a_carried_reader_replies_as_its_file_and_bad_bindings_are_refused() {
        let file = reader_file();
        let parent = route_artifact(64);
        let alone = Reader::from_bytes(&file).unwrap();
        // New names in a told turn: the value is copied from the prompt.
        let prompt = told(60, 61)[..10].to_vec();
        assert_eq!(alone.reply(&prompt, 8), vec![8, 9, 61, 3, 1]);

        let plain = IntegerDiscreteModel::from_bytes(&parent).unwrap();
        assert!(plain.question_reader().is_none());
        assert!(plain.question_reply(&prompt, 8).is_err());

        let packed = pack_question_reader(&parent, &file).unwrap();
        let model = IntegerDiscreteModel::from_bytes(&packed).unwrap();
        assert_eq!(model.artifact_schema(), SCHEMA);
        assert_eq!(
            model.artifact_sha256(),
            hex::encode(Sha256::digest(&packed))
        );
        assert_eq!(
            model.question_reader_sha256(),
            Some(hex::encode(Sha256::digest(&file)).as_str())
        );
        assert_eq!(
            model.question_reply(&prompt, 8).unwrap(),
            vec![8, 9, 61, 3, 1]
        );
        assert!(model.question_reply(&[0, 64], 8).is_err());
        // The parent's own tables are the parent's.
        assert_eq!(model.shape().vocabulary, plain.shape().vocabulary);

        // One changed reader byte, one changed parent byte, a reader over
        // another vocabulary, and a carrier inside a carrier are refused.
        let at = packed.len() - parent.len() - file.len();
        let mut reader_bit = packed.clone();
        reader_bit[at + 40] ^= 1;
        assert!(IntegerDiscreteModel::from_bytes(&reader_bit).is_err());
        let mut parent_bit = packed.clone();
        let last = parent_bit.len() - 1;
        parent_bit[last] ^= 1;
        assert!(IntegerDiscreteModel::from_bytes(&parent_bit).is_err());
        assert!(pack_question_reader(&route_artifact(2), &file).is_err());
        assert!(pack_question_reader(&packed, &file).is_err());
        assert!(pack_question_reader(&parent, &file[..file.len() - 1]).is_err());
    }
}
