//! One coherent, no-following capture of a recorded corpus and the source
//! execution identity that produced it.
//!
//! New dense-present generations publish [`RECORDED_CORPUS_BINDING_FILE`]
//! last. That canonical commit record binds every provenance/corpus member by
//! exact filename, typed presence, length, and BLAKE3 content address.
//! Compilation consumes the captured bytes directly, so arbitrary cross-file
//! replacement cannot assemble an operator era and corpus from two committed
//! generations. Markerless legacy and attention-only corpora remain readable
//! through the explicitly weaker compatibility path.

use crate::observation::{self, ObservationManifest};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs::{Metadata, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use uor_r4_model_source::SourceUnavailable;
use uor_r4_model_source::attention::AttentionOperatorSpec;
use uor_r4_model_source::dense::DenseOperatorSpec;

pub const ATTENTION_OPERATOR_BINDING_FILE: &str = "attention_operator.json";
pub const DENSE_OPERATOR_BINDING_FILE: &str = "dense_operator.json";
pub const RECORDED_CORPUS_BINDING_FILE: &str = "recorded_corpus_binding.json";
pub const RECORDED_CORPUS_BINDING_SCHEMA: &str = "uor-r4-recorded-corpus-binding/1";
pub const RECORDED_CORPUS_COMPILE_ATTEMPT_FILE: &str = "recorded_corpus_compile_attempt.json";
pub const RECORDED_CORPUS_COMPILE_ATTEMPT_SCHEMA: &str = "uor-r4-recorded-corpus-compile-attempt/1";
pub const PLANNED_OUTPUT_RESERVED_PREFIX: &str = ".uor-r4-planned-output--";
const RECORDED_CORPUS_PRODUCER_COORDINATION_DIR: &str = ".uor-r4-recorded-corpus-producers";
const STREAM_BINDING_BUFFER_BYTES: usize = 64 * 1024;

static RECORDED_CORPUS_BINDING_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static RECORDED_CORPUS_COMPILE_ATTEMPT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// The two canonical member pairs that a recorded-corpus binding may commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordedCorpusRole {
    /// Compiler/subsample corpus: `corpus.meta` plus `corpus.records`.
    Compile,
    /// Observation corpus: `state.bin` plus `merged.bin`.
    Observation,
}

impl RecordedCorpusRole {
    fn member_names(self) -> (&'static str, &'static str) {
        match self {
            Self::Compile => ("corpus.meta", "corpus.records"),
            Self::Observation => (observation::STATE_FILE, "merged.bin"),
        }
    }
}

/// A deterministic compile-style output member with an owner-only staging
/// namespace. The enum and parser live here so every producer agrees on the
/// complete reserved namespace, even when the publisher itself lives in the
/// graph CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannedOutputMember {
    Artifact,
    Store,
    Calibration,
    HierarchicalCodes,
    Records,
    Hidden,
    Metadata,
}

impl PlannedOutputMember {
    pub const ALL: [Self; 7] = [
        Self::Artifact,
        Self::Store,
        Self::Calibration,
        Self::HierarchicalCodes,
        Self::Records,
        Self::Hidden,
        Self::Metadata,
    ];

    pub const fn stable_name(self) -> &'static str {
        match self {
            Self::Artifact => "tless_artifacts.bin",
            Self::Store => "tless_store.bin",
            Self::Calibration => "hamming_calibration.json",
            Self::HierarchicalCodes => "hierarchical_codes.json",
            Self::Records => "corpus.records",
            Self::Hidden => "corpus.records.hidden",
            Self::Metadata => "corpus.meta",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedCorpusFileBinding {
    name: String,
    present: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    length: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    blake3: Option<String>,
}

impl RecordedCorpusFileBinding {
    fn from_bytes(name: &str, bytes: Option<&[u8]>) -> Result<Self, SourceUnavailable> {
        let length = bytes
            .map(|bytes| {
                u64::try_from(bytes.len()).map_err(|_| {
                    SourceUnavailable::new(format!(
                        "recorded corpus member {name} is too large for its u64 binding length"
                    ))
                })
            })
            .transpose()?;
        Ok(Self {
            name: name.to_owned(),
            present: bytes.is_some(),
            length,
            blake3: bytes.map(|bytes| format!("blake3:{}", blake3::hash(bytes).to_hex())),
        })
    }

    fn validate(
        &self,
        expected_name: &str,
        bytes: Option<&[u8]>,
        context: &str,
    ) -> Result<(), SourceUnavailable> {
        if self.name != expected_name {
            return Err(SourceUnavailable::new(format!(
                "{context} binds filename {:?}, expected exact member {expected_name:?}",
                self.name
            )));
        }
        let expected = Self::from_bytes(expected_name, bytes)?;
        if self != &expected {
            return Err(SourceUnavailable::new(format!(
                "{context} does not match the typed presence, length, and BLAKE3 of {expected_name}"
            )));
        }
        Ok(())
    }

    fn validate_record(
        &self,
        expected_name: &str,
        actual: &Self,
        context: &str,
    ) -> Result<(), SourceUnavailable> {
        if self.name != expected_name || actual.name != expected_name || self != actual {
            return Err(SourceUnavailable::new(format!(
                "{context} does not match the exact streamed presence, length, and BLAKE3 of {expected_name}"
            )));
        }
        Ok(())
    }
}

/// Canonical commit record for one recorded-corpus generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedCorpusBinding {
    schema: String,
    manifest: RecordedCorpusFileBinding,
    attention_operator: RecordedCorpusFileBinding,
    dense_operator: RecordedCorpusFileBinding,
    metadata: RecordedCorpusFileBinding,
    records: RecordedCorpusFileBinding,
    hidden: RecordedCorpusFileBinding,
}

impl RecordedCorpusBinding {
    fn canonical_bytes(&self) -> Result<Vec<u8>, SourceUnavailable> {
        let mut bytes = serde_json::to_vec_pretty(self).map_err(SourceUnavailable::new)?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// Content address of the canonical binding bytes. This is a derived
    /// corpus identity, never a source-manifest kappa.
    pub fn declared_digest(&self) -> Result<String, SourceUnavailable> {
        Ok(format!(
            "blake3:{}",
            blake3::hash(&self.canonical_bytes()?).to_hex()
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedCorpusCompileAttempt {
    schema: String,
    role: String,
}

impl RecordedCorpusCompileAttempt {
    fn compile() -> Self {
        Self {
            schema: RECORDED_CORPUS_COMPILE_ATTEMPT_SCHEMA.to_owned(),
            role: "compile".to_owned(),
        }
    }

    fn canonical_bytes(&self) -> Result<Vec<u8>, SourceUnavailable> {
        let mut bytes = serde_json::to_vec_pretty(self).map_err(SourceUnavailable::new)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

fn validate_binding_role(
    binding: &RecordedCorpusBinding,
    role: RecordedCorpusRole,
    path: &Path,
) -> Result<(), SourceUnavailable> {
    let (metadata, records) = role.member_names();
    if binding.metadata.name != metadata || binding.records.name != records {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus binding {} commits canonical pair {}/{}, but this producer requires {metadata}/{records}; refusing cross-role mutation",
            path.display(),
            binding.metadata.name,
            binding.records.name
        )));
    }
    Ok(())
}

fn validate_role_inventory(root: &Path, role: RecordedCorpusRole) -> Result<(), SourceUnavailable> {
    const COMPILE_ONLY: [&str; 19] = [
        "corpus.meta",
        "corpus.records",
        "corpus.records.hidden",
        "tokenizer.bin",
        "tless_artifacts.bin",
        "tless_store.bin",
        "hamming_calibration.json",
        "hierarchical_codes.json",
        "space_manifest.json",
        "compile_report.json",
        "tokenizer_adapter.json",
        RECORDED_CORPUS_COMPILE_ATTEMPT_FILE,
        "source_compile_preflight.json",
        "source_manifest_kappa.json",
        "compiled_bundle_completion.json",
        ".compiled_bundle_stage.json",
        "graph",
        "graph-cover",
        "instruction-eval.json",
    ];
    fn compile_only_name(name: &str) -> bool {
        if COMPILE_ONLY.contains(&name) {
            return true;
        }
        // Server/source publishers use the same stable-name + numeric owner
        // convention for their recoverable identity/completion temporaries.
        // A temp-only crash prefix is just as role-defining as its stable
        // destination and must keep an observation writer out.
        COMPILE_ONLY.iter().any(|stable| {
            let prefix = format!(".{stable}.");
            name.starts_with(&prefix)
                && (name.ends_with(".tmp")
                    || name.ends_with(".writing")
                    || name.ends_with(".replace.tmp"))
        })
    }

    if role == RecordedCorpusRole::Compile {
        for entry in std::fs::read_dir(root).map_err(SourceUnavailable::new)? {
            let entry = entry.map_err(SourceUnavailable::new)?;
            let raw_name = entry.file_name();
            let Some(name) = raw_name.to_str().map(str::to_owned) else {
                #[cfg(unix)]
                {
                    use std::os::unix::ffi::OsStrExt;
                    let bytes = raw_name.as_bytes();
                    if [
                        b"manifest.json".as_slice(),
                        b"state.bin".as_slice(),
                        b"raw-committed.bin".as_slice(),
                        b"committed.bin".as_slice(),
                        b"merged.bin".as_slice(),
                        b"stories.jsonl".as_slice(),
                        b"stories.tmp".as_slice(),
                        b"shard-".as_slice(),
                        b".manifest.json.tmp".as_slice(),
                        b".raw-committed.bin.tmp".as_slice(),
                        b".state.bin.tmp".as_slice(),
                        b".committed.bin.tmp".as_slice(),
                    ]
                    .iter()
                    .any(|prefix| bytes.starts_with(prefix))
                    {
                        return Err(SourceUnavailable::new(format!(
                            "recorded corpus root {} contains a non-UTF-8 entry in a reserved observation-role namespace",
                            root.display()
                        )));
                    }
                }
                continue;
            };
            if observation::is_observation_exclusive_entry_name(&name) {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} contains foreign observation-role member {name}; refusing compile mutation",
                    root.display()
                )));
            }
        }
    } else {
        for entry in std::fs::read_dir(root).map_err(SourceUnavailable::new)? {
            let entry = entry.map_err(SourceUnavailable::new)?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                // An unrelated non-Unicode file remains compatible, but a
                // reserved ASCII prefix represented by non-Unicode bytes is
                // terminal on Unix. The lossy form is used only as a prefix
                // classifier and never as a path to follow.
                #[cfg(unix)]
                {
                    use std::os::unix::ffi::OsStrExt;
                    let raw = entry.file_name();
                    if COMPILE_ONLY.iter().any(|stable| {
                        raw.as_bytes() == stable.as_bytes()
                            || raw.as_bytes().starts_with(format!(".{stable}.").as_bytes())
                    }) {
                        return Err(SourceUnavailable::new(format!(
                            "recorded corpus root {} contains a non-UTF-8 entry in a reserved compile-role namespace",
                            root.display()
                        )));
                    }
                }
                continue;
            };
            if compile_only_name(name) {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} contains foreign compile-role member {name}; refusing observation mutation",
                    root.display()
                )));
            }
        }
    }
    Ok(())
}

/// Parse a JSON document solely to reject duplicate object keys at every
/// nesting level. `serde_json::Value` is last-key-wins, which is unsuitable
/// for provenance records where duplicated fields could spell two arithmetic
/// eras in one document.
pub struct DuplicateRejectingJson;

impl<'de> serde::Deserialize<'de> for DuplicateRejectingJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = DuplicateRejectingJson;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("JSON without duplicate object keys")
            }

            fn visit_bool<E>(self, _: bool) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_f64<E>(self, _: f64) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_str<E>(self, _: &str) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_string<E>(self, _: String) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                <DuplicateRejectingJson as serde::Deserialize>::deserialize(deserializer)
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                while sequence.next_element::<DuplicateRejectingJson>()?.is_some() {}
                Ok(DuplicateRejectingJson)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut keys = std::collections::BTreeSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    if !keys.insert(key.clone()) {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate JSON field {key:?}"
                        )));
                    }
                    map.next_value::<DuplicateRejectingJson>()?;
                }
                Ok(DuplicateRejectingJson)
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

/// Reject duplicate object keys recursively before any typed/`Value` parse.
pub fn reject_duplicate_json(bytes: &[u8], context: &str) -> Result<(), SourceUnavailable> {
    serde_json::from_slice::<DuplicateRejectingJson>(bytes)
        .map(|_| ())
        .map_err(|error| SourceUnavailable::new(format!("{context}: malformed JSON: {error}")))
}

/// Exact registered source-execution pair captured beside a recorded corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedCorpusExecutionIdentity {
    pub attention_operator: Option<AttentionOperatorSpec>,
    pub dense_operator: Option<DenseOperatorSpec>,
}

/// Corpus bytes and execution provenance captured and reverified as one unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedCorpusSnapshot {
    pub execution: RecordedCorpusExecutionIdentity,
    /// Exact captured `attention_operator.json` bytes, when present. These
    /// bytes are the same generation that was registry- and binding-checked;
    /// callers must not reopen the pathname to derive durable evidence.
    pub attention_operator_bytes: Option<Vec<u8>>,
    /// Exact captured `dense_operator.json` bytes, when present.
    pub dense_operator_bytes: Option<Vec<u8>>,
    pub meta_bytes: Vec<u8>,
    pub records_bytes: Vec<u8>,
    pub hidden_bytes: Option<Vec<u8>>,
    /// Exact canonical generation binding, when the producer committed one.
    pub binding: Option<RecordedCorpusBinding>,
    /// BLAKE3 CID of the exact canonical binding bytes. This is derived
    /// corpus provenance and is deliberately distinct from source kappa.
    pub binding_cid: Option<String>,
}

/// One no-follow corpus member whose complete content address was validated
/// while this exact handle remained open. Consumers may seek/read it without
/// materializing the body and must call [`Self::verify_generation`] after the
/// final read before committing derived output.
#[derive(Debug)]
pub struct VerifiedCorpusMember {
    path: PathBuf,
    file: std::fs::File,
    initial: Metadata,
    binding: RecordedCorpusFileBinding,
    context: &'static str,
}

/// Bounded-memory length/content address of one exact regular member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedCorpusMemberSummary {
    pub length: u64,
    pub blake3: String,
}

impl VerifiedCorpusMember {
    pub fn len(&self) -> u64 {
        self.binding.length.unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn declared_digest(&self) -> &str {
        self.binding.blake3.as_deref().unwrap_or("")
    }

    pub fn summary(&self) -> RecordedCorpusMemberSummary {
        RecordedCorpusMemberSummary {
            length: self.len(),
            blake3: self.declared_digest().to_owned(),
        }
    }

    pub fn verify_generation(&self) -> Result<(), SourceUnavailable> {
        let final_file = self.file.metadata().map_err(SourceUnavailable::new)?;
        let final_path = std::fs::symlink_metadata(&self.path).map_err(|error| {
            SourceUnavailable::new(format!(
                "{} {} changed or disappeared: {error}",
                self.context,
                self.path.display()
            ))
        })?;
        if !final_path.file_type().is_file()
            || !opened_file_generation_matches(&self.initial, &final_file)
            || !opened_file_identity_matches(&final_path, &final_file)
        {
            return Err(SourceUnavailable::new(format!(
                "{} {} changed generation after its verified stream was opened",
                self.context,
                self.path.display()
            )));
        }
        Ok(())
    }
}

impl Read for VerifiedCorpusMember {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buffer)
    }
}

impl Seek for VerifiedCorpusMember {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(position)
    }
}

/// Small provenance/metadata plus retained verified handles for large corpus
/// members. This is the bounded-memory source for derivation/status paths.
#[derive(Debug)]
pub struct RecordedCorpusStreamSnapshot {
    pub execution: RecordedCorpusExecutionIdentity,
    pub attention_operator_bytes: Option<Vec<u8>>,
    pub dense_operator_bytes: Option<Vec<u8>>,
    pub meta_bytes: Vec<u8>,
    pub records: VerifiedCorpusMember,
    pub hidden: Option<VerifiedCorpusMember>,
    pub binding_cid: Option<String>,
    root: PathBuf,
    provenance: ProvenanceBytes,
    meta_path: PathBuf,
    binding_path: PathBuf,
    binding_bytes: Option<Vec<u8>>,
    marker_path: PathBuf,
    marker_bytes: Option<Vec<u8>>,
    hidden_path: PathBuf,
}

impl RecordedCorpusStreamSnapshot {
    /// Final generation check after a consumer finishes reading/seeking every
    /// retained member and before it publishes any derived generation.
    pub fn verify_generation(&self) -> Result<(), SourceUnavailable> {
        self.records.verify_generation()?;
        if let Some(hidden) = self.hidden.as_ref() {
            hidden.verify_generation()?;
        } else {
            verify_captured_optional_regular_file(
                &self.hidden_path,
                "recorded corpus hidden stream",
                None,
            )?;
        }
        verify_captured_optional_regular_file(
            &self.meta_path,
            "recorded corpus metadata",
            Some(&self.meta_bytes),
        )?;
        verify_provenance(&self.root, &self.provenance)?;
        verify_captured_optional_regular_file(
            &self.binding_path,
            "recorded corpus generation binding",
            self.binding_bytes.as_deref(),
        )?;
        verify_captured_optional_regular_file(
            &self.marker_path,
            "recorded corpus compile-attempt marker",
            self.marker_bytes.as_deref(),
        )?;
        require_no_binding_temporaries(&self.root)
    }
}

/// Nonblocking, process-crash-safe exclusive guard for one canonical recorded
/// corpus root.
///
/// The permanent sibling inode is coordination metadata, not corpus payload.
/// Every producer acquires any writer-specific outer session first and this
/// guard second, then holds both from the first corpus/provenance mutation
/// through final [`publish_binding`]. No producer may acquire another outer
/// session or a second recorded-corpus guard while this value is live.
#[derive(Debug)]
pub struct RecordedCorpusProducerGuard {
    root: PathBuf,
    _file: std::fs::File,
    root_file: Option<std::fs::File>,
    _coordination_file: std::fs::File,
    _parent_file: std::fs::File,
}

impl RecordedCorpusProducerGuard {
    /// Try to acquire exclusive producer ownership without waiting. A live
    /// cooperating producer returns an explicit `BUSY` error; malformed,
    /// symlinked, special, or identity-raced coordination entries fail closed.
    pub fn try_acquire(root: impl AsRef<Path>) -> Result<Self, SourceUnavailable> {
        Self::try_acquire_with_root_hook(root.as_ref(), || {})
    }

    fn try_acquire_with_root_hook<F>(
        root: &Path,
        after_root_open: F,
    ) -> Result<Self, SourceUnavailable>
    where
        F: FnOnce(),
    {
        let root_name = root.file_name().ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus producer subject {} must end in one exact directory name, not '.', '..', or a filesystem root",
                root.display()
            ))
        })?;
        if root_name == OsStr::new(".") || root_name == OsStr::new("..") {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus producer subject {} cannot use '.' or '..' as its root name",
                root.display()
            )));
        }
        if root_name == OsStr::new(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR) {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus producer root name {RECORDED_CORPUS_PRODUCER_COORDINATION_DIR:?} is reserved for sibling coordination"
            )));
        }
        let requested_parent = root
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let canonical_parent = std::fs::canonicalize(requested_parent).map_err(|error| {
            SourceUnavailable::new(format!(
                "recorded corpus producer parent {} cannot be canonicalized: {error}",
                requested_parent.display()
            ))
        })?;
        let parent_file = open_directory_nofollow(
            &canonical_parent,
            "recorded corpus producer canonical parent",
        )?;
        verify_directory_handle(
            &canonical_parent,
            &parent_file,
            "recorded corpus producer canonical parent",
        )?;
        let requested_root = canonical_parent.join(root_name);
        let root_file = match std::fs::symlink_metadata(&requested_root) {
            Ok(metadata) if metadata.file_type().is_dir() => Some(open_directory_nofollow(
                &requested_root,
                "recorded corpus producer root",
            )?),
            Ok(_) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus producer root {} is not a real non-symlink directory",
                    requested_root.display()
                )));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(SourceUnavailable::new(error)),
        };
        if let Some(file) = root_file.as_ref() {
            verify_directory_handle(&requested_root, file, "recorded corpus producer root")?;
        }
        after_root_open();
        verify_directory_handle(
            &canonical_parent,
            &parent_file,
            "recorded corpus producer canonical parent",
        )?;
        verify_optional_root_generation(&requested_root, root_file.as_ref())?;
        let canonical = match root_file.as_ref() {
            Some(file) => canonical_existing_root_path(&canonical_parent, &requested_root, file)?,
            None => requested_root,
        };
        let canonical_name = canonical.file_name().ok_or_else(|| {
            SourceUnavailable::new("canonical recorded corpus root has no final component")
        })?;
        let coordination_path = canonical_parent.join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR);
        let coordination_file =
            open_or_create_coordination_directory(&canonical_parent, &parent_file)?;
        let lock_path = coordination_path.join(canonical_name);
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;

            options
                .mode(0o600)
                .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let file = options.open(&lock_path).map_err(|error| {
            SourceUnavailable::new(format!(
                "recorded corpus producer coordination {} cannot be opened without following links: {error}",
                lock_path.display()
            ))
        })?;
        let initial_path = std::fs::symlink_metadata(&lock_path).map_err(|error| {
            SourceUnavailable::new(format!(
                "recorded corpus producer coordination {} cannot be inspected: {error}",
                lock_path.display()
            ))
        })?;
        let initial_file = file.metadata().map_err(SourceUnavailable::new)?;
        if !initial_path.file_type().is_file()
            || !initial_file.file_type().is_file()
            || !opened_file_identity_matches(&initial_path, &initial_file)
        {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus producer coordination {} is not one stable regular non-symlink inode",
                lock_path.display()
            )));
        }
        match file.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} is BUSY under another active producer session",
                    canonical.display()
                )));
            }
            Err(std::fs::TryLockError::Error(error)) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus producer coordination {} cannot be locked: {error}",
                    lock_path.display()
                )));
            }
        }
        let final_path = std::fs::symlink_metadata(&lock_path).map_err(SourceUnavailable::new)?;
        let final_file = file.metadata().map_err(SourceUnavailable::new)?;
        if !final_path.file_type().is_file()
            || !final_file.file_type().is_file()
            || !opened_file_identity_matches(&initial_file, &final_file)
            || !opened_file_identity_matches(&final_path, &final_file)
        {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus producer coordination {} changed identity or type while its lock was acquired",
                lock_path.display()
            )));
        }
        verify_directory_handle(
            &canonical_parent,
            &parent_file,
            "recorded corpus producer canonical parent",
        )?;
        verify_directory_handle(
            &coordination_path,
            &coordination_file,
            "recorded corpus producer coordination directory",
        )?;
        verify_optional_root_generation(&canonical, root_file.as_ref())?;
        Ok(Self {
            root: canonical,
            _file: file,
            root_file,
            _coordination_file: coordination_file,
            _parent_file: parent_file,
        })
    }

    /// Canonical root protected by this guard.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Whether the logical root existed when acquired (or has since been
    /// created by [`Self::ensure_root`]).
    pub fn root_exists(&self) -> bool {
        self.root_file.is_some()
    }

    /// Validate every stable/reserved binding entry before a caller mutates
    /// any generation member. Returns `true` when a canonical, fully parsed
    /// publication temporary needs exact-generation recovery by
    /// [`publish_binding`]. Non-authoritative `.writing` residue may be
    /// reclaimed only by that publisher while this guard remains live.
    pub fn preflight_publication_namespace(&self) -> Result<bool, SourceUnavailable> {
        self.verify_root()?;
        let stable_path = binding_path(&self.root);
        if let Some(bytes) =
            capture_optional_regular_file(&stable_path, "recorded corpus generation binding")?
        {
            let _ = parse_binding_bytes(&stable_path, &bytes)?;
        }
        let residues = binding_publication_residues(&self.root, true)?;
        self.verify_root()?;
        Ok(!residues.temporaries.is_empty())
    }

    /// Validate the binding namespace and require every stable or recoverable
    /// commit record to name the caller's canonical member pair. This keeps an
    /// observation writer from mutating a compile/subsample root (and vice
    /// versa) before binding-last publication discovers the role conflict.
    pub fn preflight_publication_namespace_for(
        &self,
        role: RecordedCorpusRole,
    ) -> Result<bool, SourceUnavailable> {
        self.verify_root()?;
        validate_role_inventory(&self.root, role)?;
        validate_compile_attempt_namespace(&self.root, role)?;
        let stable_path = binding_path(&self.root);
        if let Some(bytes) =
            capture_optional_regular_file(&stable_path, "recorded corpus generation binding")?
        {
            let binding = parse_binding_bytes(&stable_path, &bytes)?;
            validate_binding_role(&binding, role, &stable_path)?;
        }
        let residues = binding_publication_residues(&self.root, true)?;
        for (path, bytes) in &residues.temporaries {
            let binding = parse_binding_bytes(path, bytes)?;
            validate_binding_role(&binding, role, path)?;
        }
        self.verify_root()?;
        Ok(!residues.temporaries.is_empty())
    }

    /// Durably declare a compile-style mutation attempt before the first
    /// sidecar or corpus member is published. The fixed marker disambiguates
    /// sidecar-only crash prefixes from observation roots without changing
    /// the canonical recorded-corpus binding schema.
    pub fn begin_compile_attempt(&self) -> Result<(), SourceUnavailable> {
        self.verify_root()?;
        self.preflight_planned_output_scope(&PlannedOutputMember::ALL)?;
        let _ = self.preflight_publication_namespace_for(RecordedCorpusRole::Compile)?;
        publish_compile_attempt_marker(self)?;
        self.verify_root()
    }

    /// Remove the compile-attempt marker only after a stable same-role corpus
    /// binding exists. A crash before removal leaves a readable committed
    /// generation plus explicit retry evidence; removal and directory sync
    /// are idempotent under the retained guard.
    pub fn finish_compile_attempt(&self) -> Result<(), SourceUnavailable> {
        self.verify_root()?;
        let stable_binding = binding_path(&self.root);
        let bytes = capture_optional_regular_file(
            &stable_binding,
            "recorded corpus generation binding",
        )?
        .ok_or_else(|| {
            SourceUnavailable::new(
                "cannot finish recorded-corpus compile attempt before its stable generation binding",
            )
        })?;
        let binding = parse_binding_bytes(&stable_binding, &bytes)?;
        validate_binding_role(&binding, RecordedCorpusRole::Compile, &stable_binding)?;
        finish_compile_attempt_marker(self)?;
        self.verify_root()
    }

    /// Whether this compile-role root carries the exact durable attempt
    /// marker. The namespace is fully validated before the answer is
    /// returned, so malformed or special residue remains terminal.
    pub fn compile_attempt_active(&self) -> Result<bool, SourceUnavailable> {
        self.verify_root()?;
        validate_compile_attempt_namespace(&self.root, RecordedCorpusRole::Compile)?;
        let path = compile_attempt_path(&self.root);
        let active =
            capture_optional_regular_file(&path, "recorded corpus compile-attempt marker")?
                .map(|bytes| parse_compile_attempt_bytes(&path, &bytes))
                .transpose()?
                .is_some();
        self.verify_root()?;
        Ok(active)
    }

    /// Validate the complete shared planned-output staging namespace and
    /// reject residue owned by any member outside `allowed`. All recognized
    /// entries are opened nofollow and bound to one regular inode before the
    /// owner filter is applied; malformed or special entries are terminal.
    pub fn preflight_planned_output_scope(
        &self,
        allowed: &[PlannedOutputMember],
    ) -> Result<(), SourceUnavailable> {
        let residues = self.planned_output_residues()?;
        if let Some(residue) = residues
            .iter()
            .find(|residue| !allowed.contains(&residue.member))
        {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} contains reserved staging residue {} for foreign member {}; refusing a partial-plan publication",
                self.root.display(),
                residue.path.display(),
                residue.member.stable_name()
            )));
        }
        Ok(())
    }

    /// Reject stable deterministic members outside a partial producer's
    /// declared plan. Staging ownership alone is insufficient: a subsample
    /// must not commit new corpus bytes beside stale compile artifacts.
    pub fn preflight_planned_output_stable_scope(
        &self,
        allowed: &[PlannedOutputMember],
    ) -> Result<(), SourceUnavailable> {
        self.verify_root()?;
        for member in PlannedOutputMember::ALL {
            if allowed.contains(&member) {
                continue;
            }
            let path = self.root.join(member.stable_name());
            match std::fs::symlink_metadata(&path) {
                Ok(_) => {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus root {} contains foreign stable planned member {}; refusing a partial-plan publication",
                        self.root.display(),
                        member.stable_name()
                    )));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(SourceUnavailable::new(error)),
            }
        }
        self.verify_root()
    }

    /// Require a deterministic compile-style destination to contain only the
    /// exact members that this command can reproduce, plus the shared
    /// execution/binding/attempt namespaces. This is deliberately stricter
    /// than the broad Compile role: source bundles and server stages carry
    /// additional valid compile-role leaves that a corpus derivation must not
    /// silently preserve beside a newly committed generation.
    pub fn preflight_deterministic_compile_inventory(
        &self,
        allowed: &[PlannedOutputMember],
    ) -> Result<(), SourceUnavailable> {
        self.verify_root()?;
        self.preflight_planned_output_scope(allowed)?;
        let _ = self.preflight_publication_namespace_for(RecordedCorpusRole::Compile)?;
        for entry in std::fs::read_dir(&self.root).map_err(SourceUnavailable::new)? {
            let entry = entry.map_err(SourceUnavailable::new)?;
            let os_name = entry.file_name();
            let Some(name) = os_name.to_str() else {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} contains a non-Unicode entry outside this deterministic compile plan",
                    self.root.display()
                )));
            };
            let stable_allowed = allowed.iter().any(|member| member.stable_name() == name);
            let shared_stable = matches!(
                name,
                ATTENTION_OPERATOR_BINDING_FILE
                    | DENSE_OPERATOR_BINDING_FILE
                    | RECORDED_CORPUS_BINDING_FILE
                    | RECORDED_CORPUS_COMPILE_ATTEMPT_FILE
            );
            let shared_reserved = name.starts_with(&format!(".{RECORDED_CORPUS_BINDING_FILE}."))
                || name.starts_with(&format!(".{RECORDED_CORPUS_COMPILE_ATTEMPT_FILE}."))
                || name.starts_with(PLANNED_OUTPUT_RESERVED_PREFIX);
            if stable_allowed || shared_stable || shared_reserved {
                continue;
            }
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} contains unowned compile-style entry {name}; refusing deterministic corpus publication",
                self.root.display()
            )));
        }
        self.verify_root()
    }

    /// Whether a validated regular staging residue exists for `member`.
    pub fn has_planned_output_residue(
        &self,
        member: PlannedOutputMember,
    ) -> Result<bool, SourceUnavailable> {
        Ok(self
            .planned_output_residues()?
            .iter()
            .any(|residue| residue.member == member))
    }

    /// Reclaim only validated regular residue owned by `member`. The initial
    /// generation is rechecked through a nofollow handle immediately before
    /// unlink so process death recovery is O(1) in staged payload size.
    pub fn reclaim_planned_output_residues(
        &self,
        member: PlannedOutputMember,
    ) -> Result<(), SourceUnavailable> {
        let residues = self.planned_output_residues()?;
        for residue in residues
            .into_iter()
            .filter(|residue| residue.member == member)
        {
            remove_planned_output_residue(&residue)?;
        }
        self.verify_root()
    }

    fn planned_output_residues(&self) -> Result<Vec<PlannedOutputResidue>, SourceUnavailable> {
        self.verify_root()?;
        let mut residues = Vec::new();
        for entry in std::fs::read_dir(&self.root).map_err(SourceUnavailable::new)? {
            let entry = entry.map_err(SourceUnavailable::new)?;
            let Some(name) = reserved_planned_output_entry_name(&self.root, &entry.file_name())?
            else {
                continue;
            };
            let member = PlannedOutputMember::ALL
                .into_iter()
                .find(|member| planned_output_staging_name(*member, &name))
                .ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "recorded corpus root {} contains unrecognized reserved planned-output staging entry {name:?}",
                        self.root.display()
                    ))
                })?;
            let path = entry.path();
            let mut options = OpenOptions::new();
            options.read(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
            }
            let file = options.open(&path).map_err(|error| {
                SourceUnavailable::new(format!(
                    "planned-output staging entry {} cannot be opened without following links: {error}",
                    path.display()
                ))
            })?;
            let path_metadata = std::fs::symlink_metadata(&path).map_err(SourceUnavailable::new)?;
            let file_metadata = file.metadata().map_err(SourceUnavailable::new)?;
            if !path_metadata.file_type().is_file()
                || !file_metadata.file_type().is_file()
                || !opened_file_identity_matches(&path_metadata, &file_metadata)
            {
                return Err(SourceUnavailable::new(format!(
                    "planned-output staging entry {} is not one stable regular non-symlink inode",
                    path.display()
                )));
            }
            residues.push(PlannedOutputResidue {
                member,
                path,
                metadata: file_metadata,
            });
        }
        residues.sort_by(|left, right| left.path.cmp(&right.path));
        self.verify_root()?;
        Ok(residues)
    }

    /// Create and bind a previously absent logical root after all caller
    /// semantic/input preflights have succeeded. Existing roots are merely
    /// reverified. This is the only supported transition from a path lock to
    /// a mutable recorded-corpus directory.
    pub fn ensure_root(&mut self) -> Result<&Path, SourceUnavailable> {
        self.verify_parent()?;
        if self.root_file.is_none() {
            std::fs::create_dir(&self.root).map_err(|error| {
                SourceUnavailable::new(format!(
                    "recorded corpus producer root {} cannot be created under exclusive ownership: {error}",
                    self.root.display()
                ))
            })?;
            self._parent_file
                .sync_all()
                .map_err(SourceUnavailable::new)?;
            self.root_file = Some(open_directory_nofollow(
                &self.root,
                "recorded corpus producer root",
            )?);
        }
        self.verify_root()?;
        Ok(&self.root)
    }

    fn verify_parent(&self) -> Result<(), SourceUnavailable> {
        verify_directory_handle(
            self.root.parent().ok_or_else(|| {
                SourceUnavailable::new("recorded corpus producer root has no canonical parent")
            })?,
            &self._parent_file,
            "recorded corpus producer canonical parent",
        )?;
        verify_directory_handle(
            &self
                .root
                .parent()
                .expect("validated canonical root parent")
                .join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR),
            &self._coordination_file,
            "recorded corpus producer coordination directory",
        )
    }

    fn verify_root(&self) -> Result<(), SourceUnavailable> {
        let file = self.root_file.as_ref().ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus producer root {} is still absent; call ensure_root only after semantic preflight and before mutation",
                self.root.display()
            ))
        })?;
        verify_directory_handle(&self.root, file, "recorded corpus producer root")
    }

    /// Revalidate the guarded root handle before a higher-level producer
    /// publishes another generation member.
    pub fn verify_owned_root(&self) -> Result<(), SourceUnavailable> {
        self.verify_root()
    }

    /// Compare another directory to the guarded root by opened directory
    /// identity, not caller spelling. This catches case/Unicode aliases on
    /// filesystems whose canonicalize operation preserves the input case.
    pub fn protects_directory(&self, directory: &Path) -> Result<bool, SourceUnavailable> {
        if self.root_file.is_none() {
            self.verify_parent()?;
            verify_optional_root_generation(&self.root, None)?;
            return Ok(false);
        }
        self.verify_root()?;
        let other = open_directory_nofollow(directory, "recorded corpus comparison directory")?;
        #[cfg(unix)]
        {
            let guarded = self
                .root_file
                .as_ref()
                .expect("verified recorded corpus root")
                .metadata()
                .map_err(SourceUnavailable::new)?;
            let other = other.metadata().map_err(SourceUnavailable::new)?;
            return Ok(opened_file_identity_matches(&guarded, &other));
        }
        #[cfg(not(unix))]
        {
            let canonical = std::fs::canonicalize(directory).map_err(SourceUnavailable::new)?;
            Ok(canonical == self.root)
        }
    }
}

/// Non-mutating shared coordination for a bounded recorded-corpus read.
///
/// When a producer coordination inode already exists, readers take its shared
/// lock and therefore return `BUSY` while an exclusive producer is live. A
/// copied/read-only legacy archive may have no coordination directory or lock;
/// this guard never creates either and instead pins their typed absence plus
/// the parent/root directory handles for a final recheck.
#[derive(Debug)]
pub struct RecordedCorpusReaderGuard {
    root: PathBuf,
    root_file: std::fs::File,
    parent: PathBuf,
    parent_file: std::fs::File,
    coordination_path: PathBuf,
    coordination_file: Option<std::fs::File>,
    lock_path: PathBuf,
    lock_file: Option<std::fs::File>,
}

impl RecordedCorpusReaderGuard {
    pub fn try_acquire(root: impl AsRef<Path>) -> Result<Self, SourceUnavailable> {
        let requested = root.as_ref();
        let root_name = requested.file_name().ok_or_else(|| {
            SourceUnavailable::new("recorded corpus reader root has no final component")
        })?;
        let requested_parent = requested
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent = std::fs::canonicalize(requested_parent).map_err(SourceUnavailable::new)?;
        let parent_file =
            open_directory_nofollow(&parent, "recorded corpus reader canonical parent")?;
        let requested_root = parent.join(root_name);
        let root_file = open_directory_nofollow(&requested_root, "recorded corpus reader root")?;
        let root = canonical_existing_root_path(&parent, &requested_root, &root_file)?;
        verify_directory_handle(&root, &root_file, "recorded corpus reader root")?;

        let coordination_path = parent.join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR);
        let coordination_file = match std::fs::symlink_metadata(&coordination_path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(SourceUnavailable::new(error)),
            Ok(metadata) if metadata.file_type().is_dir() => Some(open_directory_nofollow(
                &coordination_path,
                "recorded corpus reader coordination directory",
            )?),
            Ok(_) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus reader coordination {} is not a real non-symlink directory",
                    coordination_path.display()
                )));
            }
        };
        let lock_path = coordination_path.join(root.file_name().ok_or_else(|| {
            SourceUnavailable::new("canonical recorded corpus reader root has no name")
        })?);
        let lock_file = if coordination_file.is_some() {
            match std::fs::symlink_metadata(&lock_path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => return Err(SourceUnavailable::new(error)),
                Ok(metadata) if metadata.file_type().is_file() => {
                    let mut options = OpenOptions::new();
                    options.read(true);
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::OpenOptionsExt;
                        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
                    }
                    let file = options.open(&lock_path).map_err(SourceUnavailable::new)?;
                    let opened = file.metadata().map_err(SourceUnavailable::new)?;
                    let path =
                        std::fs::symlink_metadata(&lock_path).map_err(SourceUnavailable::new)?;
                    if !opened.file_type().is_file()
                        || !path.file_type().is_file()
                        || !opened_file_identity_matches(&path, &opened)
                    {
                        return Err(SourceUnavailable::new(format!(
                            "recorded corpus reader coordination {} changed identity or type",
                            lock_path.display()
                        )));
                    }
                    match file.try_lock_shared() {
                        Ok(()) => Some(file),
                        Err(std::fs::TryLockError::WouldBlock) => {
                            return Err(SourceUnavailable::new(format!(
                                "recorded corpus root {} is BUSY under another active producer session",
                                root.display()
                            )));
                        }
                        Err(std::fs::TryLockError::Error(error)) => {
                            return Err(SourceUnavailable::new(format!(
                                "recorded corpus reader coordination {} cannot be locked: {error}",
                                lock_path.display()
                            )));
                        }
                    }
                }
                Ok(_) => {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus reader coordination {} is not a regular non-symlink file",
                        lock_path.display()
                    )));
                }
            }
        } else {
            None
        };

        let guard = Self {
            root,
            root_file,
            parent,
            parent_file,
            coordination_path,
            coordination_file,
            lock_path,
            lock_file,
        };
        guard.verify()?;
        Ok(guard)
    }

    pub fn verify(&self) -> Result<(), SourceUnavailable> {
        verify_directory_handle(
            &self.parent,
            &self.parent_file,
            "recorded corpus reader canonical parent",
        )?;
        verify_directory_handle(&self.root, &self.root_file, "recorded corpus reader root")?;
        match self.coordination_file.as_ref() {
            Some(file) => verify_directory_handle(
                &self.coordination_path,
                file,
                "recorded corpus reader coordination directory",
            )?,
            None => match std::fs::symlink_metadata(&self.coordination_path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(SourceUnavailable::new(error)),
                Ok(_) => {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus reader coordination {} appeared during an uncoordinated legacy read",
                        self.coordination_path.display()
                    )));
                }
            },
        }
        match self.lock_file.as_ref() {
            Some(file) => {
                let path =
                    std::fs::symlink_metadata(&self.lock_path).map_err(SourceUnavailable::new)?;
                let opened = file.metadata().map_err(SourceUnavailable::new)?;
                if !path.file_type().is_file()
                    || !opened.file_type().is_file()
                    || !opened_file_identity_matches(&path, &opened)
                {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus reader coordination {} changed identity or type",
                        self.lock_path.display()
                    )));
                }
            }
            None if self.coordination_file.is_some() => {
                match std::fs::symlink_metadata(&self.lock_path) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(SourceUnavailable::new(error)),
                    Ok(_) => {
                        return Err(SourceUnavailable::new(format!(
                            "recorded corpus reader coordination {} appeared during an uncoordinated legacy read",
                            self.lock_path.display()
                        )));
                    }
                }
            }
            None => {}
        }
        Ok(())
    }
}

#[derive(Debug)]
struct PlannedOutputResidue {
    member: PlannedOutputMember,
    path: PathBuf,
    metadata: Metadata,
}

fn planned_output_staging_name(member: PlannedOutputMember, name: &str) -> bool {
    let prefix = format!("{PLANNED_OUTPUT_RESERVED_PREFIX}{}--", member.stable_name());
    let Some(sequence) = name
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix(".writing"))
    else {
        return false;
    };
    let mut parts = sequence.split('.');
    parts
        .next()
        .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none()
}

fn reserved_planned_output_entry_name(
    root: &Path,
    name: &OsStr,
) -> Result<Option<String>, SourceUnavailable> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        let bytes = name.as_bytes();
        if !bytes.starts_with(PLANNED_OUTPUT_RESERVED_PREFIX.as_bytes()) {
            return Ok(None);
        }
        return std::str::from_utf8(bytes)
            .map(|name| Some(name.to_owned()))
            .map_err(|_| {
                SourceUnavailable::new(format!(
                    "recorded corpus root {} contains a non-UTF-8 entry in the reserved planned-output staging namespace",
                    root.display()
                ))
            });
    }

    #[cfg(not(unix))]
    {
        let lossy = name.to_string_lossy();
        if !lossy.starts_with(PLANNED_OUTPUT_RESERVED_PREFIX) {
            return Ok(None);
        }
        name.to_str().map(|name| Some(name.to_owned())).ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus root {} contains a non-Unicode entry in the reserved planned-output staging namespace",
                root.display()
            ))
        })
    }
}

fn remove_planned_output_residue(residue: &PlannedOutputResidue) -> Result<(), SourceUnavailable> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(&residue.path).map_err(|error| {
        SourceUnavailable::new(format!(
            "planned-output staging entry {} cannot be reopened without following links: {error}",
            residue.path.display()
        ))
    })?;
    let path_metadata = std::fs::symlink_metadata(&residue.path).map_err(SourceUnavailable::new)?;
    let file_metadata = file.metadata().map_err(SourceUnavailable::new)?;
    if !path_metadata.file_type().is_file()
        || !file_metadata.file_type().is_file()
        || !opened_file_identity_matches(&path_metadata, &file_metadata)
        || !opened_file_generation_matches(&residue.metadata, &file_metadata)
    {
        return Err(SourceUnavailable::new(format!(
            "planned-output staging entry {} changed identity, type, or generation before guarded recovery",
            residue.path.display()
        )));
    }
    std::fs::remove_file(&residue.path).map_err(SourceUnavailable::new)
}

fn open_directory_nofollow(path: &Path, context: &str) -> Result<std::fs::File, SourceUnavailable> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        options.custom_flags(
            libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_NONBLOCK,
        );
    }
    let file = options.open(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot be opened without following links: {error}",
            path.display()
        ))
    })?;
    if !file
        .metadata()
        .map_err(SourceUnavailable::new)?
        .file_type()
        .is_dir()
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} is not a directory",
            path.display()
        )));
    }
    Ok(file)
}

fn verify_directory_handle(
    path: &Path,
    file: &std::fs::File,
    context: &str,
) -> Result<(), SourceUnavailable> {
    let path_metadata = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    let file_metadata = file.metadata().map_err(SourceUnavailable::new)?;
    if !path_metadata.file_type().is_dir()
        || !file_metadata.file_type().is_dir()
        || !opened_file_identity_matches(&path_metadata, &file_metadata)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed identity or is not a real non-symlink directory",
            path.display()
        )));
    }
    Ok(())
}

fn open_or_create_coordination_directory(
    canonical_parent: &Path,
    parent_file: &std::fs::File,
) -> Result<std::fs::File, SourceUnavailable> {
    verify_directory_handle(
        canonical_parent,
        parent_file,
        "recorded corpus producer canonical parent",
    )?;
    let path = canonical_parent.join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR);
    match std::fs::create_dir(&path) {
        Ok(()) => parent_file.sync_all().map_err(SourceUnavailable::new)?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus producer coordination directory {} cannot be created: {error}",
                path.display()
            )));
        }
    }
    let file = open_directory_nofollow(&path, "recorded corpus producer coordination directory")?;
    verify_directory_handle(
        &path,
        &file,
        "recorded corpus producer coordination directory",
    )?;
    verify_directory_handle(
        canonical_parent,
        parent_file,
        "recorded corpus producer canonical parent",
    )?;
    Ok(file)
}

fn verify_optional_root_generation(
    root: &Path,
    file: Option<&std::fs::File>,
) -> Result<(), SourceUnavailable> {
    match file {
        Some(file) => verify_directory_handle(root, file, "recorded corpus producer root"),
        None => match std::fs::symlink_metadata(root) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(SourceUnavailable::new(error)),
            Ok(_) => Err(SourceUnavailable::new(format!(
                "recorded corpus producer root {} appeared while its absent logical path was being locked",
                root.display()
            ))),
        },
    }
}

fn canonical_existing_root_path(
    canonical_parent: &Path,
    requested_root: &Path,
    file: &std::fs::File,
) -> Result<PathBuf, SourceUnavailable> {
    #[cfg(unix)]
    {
        let file_metadata = file.metadata().map_err(SourceUnavailable::new)?;
        let mut matches = Vec::new();
        for entry in std::fs::read_dir(canonical_parent).map_err(SourceUnavailable::new)? {
            let entry = entry.map_err(SourceUnavailable::new)?;
            let metadata = match std::fs::symlink_metadata(entry.path()) {
                Ok(metadata) => metadata,
                // Unrelated siblings may be removed concurrently. The opened
                // target handle and its exact requested path are rechecked
                // below, so a vanished nonmatching sibling is irrelevant.
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(SourceUnavailable::new(error)),
            };
            if metadata.file_type().is_dir()
                && opened_file_identity_matches(&metadata, &file_metadata)
            {
                matches.push(entry.path());
            }
        }
        matches.sort();
        if matches.len() != 1 {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus producer root {} resolves to {} real directory entries in {}; refusing an ambiguous OS-equivalence class",
                requested_root.display(),
                matches.len(),
                canonical_parent.display()
            )));
        }
        return Ok(matches.remove(0));
    }

    #[cfg(not(unix))]
    std::fs::canonicalize(requested_root).map_err(SourceUnavailable::new)
}

#[cfg(test)]
fn producer_lock_path(canonical_parent: &Path, root_name: &OsStr) -> PathBuf {
    canonical_parent
        .join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR)
        .join(root_name)
}

#[cfg(unix)]
fn opened_file_identity_matches(path: &Metadata, file: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    path.dev() == file.dev() && path.ino() == file.ino()
}

#[cfg(not(unix))]
fn opened_file_identity_matches(_path: &Metadata, _file: &Metadata) -> bool {
    true
}

#[cfg(unix)]
fn opened_file_generation_matches(initial: &Metadata, final_metadata: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    opened_file_identity_matches(initial, final_metadata)
        && initial.len() == final_metadata.len()
        && initial.mtime() == final_metadata.mtime()
        && initial.mtime_nsec() == final_metadata.mtime_nsec()
        && initial.ctime() == final_metadata.ctime()
        && initial.ctime_nsec() == final_metadata.ctime_nsec()
}

#[cfg(not(unix))]
fn opened_file_generation_matches(initial: &Metadata, final_metadata: &Metadata) -> bool {
    opened_file_identity_matches(initial, final_metadata)
        && initial.len() == final_metadata.len()
        && initial.modified().ok() == final_metadata.modified().ok()
}

fn capture_optional_regular_file(
    path: &Path,
    context: &str,
) -> Result<Option<Vec<u8>>, SourceUnavailable> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "{context} {} is not a regular file or cannot be opened without following links: {error}",
                path.display()
            )));
        }
    };
    let initial_file = file.metadata().map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} opened handle cannot be inspected: {error}",
            path.display()
        ))
    })?;
    let initial_path = std::fs::symlink_metadata(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot be inspected after open: {error}",
            path.display()
        ))
    })?;
    if !initial_file.file_type().is_file()
        || !initial_path.file_type().is_file()
        || !opened_file_identity_matches(&initial_path, &initial_file)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} is not the opened regular non-symlink file",
            path.display()
        )));
    }

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot be read: {error}",
            path.display()
        ))
    })?;
    if initial_file.len() != bytes.len() as u64 {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed length while it was captured",
            path.display()
        )));
    }
    file.seek(SeekFrom::Start(0)).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot be rewound for verification: {error}",
            path.display()
        ))
    })?;
    verify_opened_bytes(&mut file, path, context, &bytes, &initial_file)?;
    Ok(Some(bytes))
}

/// Capture only the typed presence/length/content address of a regular file.
/// The payload is streamed through one no-follow handle and never retained;
/// initial/final path and handle generation checks bind the digest to that
/// exact inode generation.
fn stream_regular_file_binding(
    path: &Path,
    expected_name: &str,
    context: &str,
) -> Result<RecordedCorpusFileBinding, SourceUnavailable> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return RecordedCorpusFileBinding::from_bytes(expected_name, None);
        }
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "{context} {} cannot be opened without following links: {error}",
                path.display()
            )));
        }
    };
    let initial_file = file.metadata().map_err(SourceUnavailable::new)?;
    let initial_path = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    if !initial_file.file_type().is_file()
        || !initial_path.file_type().is_file()
        || !opened_file_identity_matches(&initial_path, &initial_file)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} is not one regular non-symlink inode",
            path.display()
        )));
    }
    let mut hasher = blake3::Hasher::new();
    let mut length = 0u64;
    let mut buffer = [0u8; STREAM_BINDING_BUFFER_BYTES];
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            SourceUnavailable::new(format!(
                "{context} {} cannot be streamed: {error}",
                path.display()
            ))
        })?;
        if read == 0 {
            break;
        }
        length = length
            .checked_add(read as u64)
            .ok_or_else(|| SourceUnavailable::new(format!("{context} length overflows u64")))?;
        hasher.update(&buffer[..read]);
    }
    let final_file = file.metadata().map_err(SourceUnavailable::new)?;
    let final_path = std::fs::symlink_metadata(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} changed or disappeared while hashed: {error}",
            path.display()
        ))
    })?;
    if length != initial_file.len()
        || !final_path.file_type().is_file()
        || !opened_file_generation_matches(&initial_file, &final_file)
        || !opened_file_identity_matches(&final_path, &final_file)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed generation while it was streamed",
            path.display()
        )));
    }
    Ok(RecordedCorpusFileBinding {
        name: expected_name.to_owned(),
        present: true,
        length: Some(length),
        blake3: Some(format!("blake3:{}", hasher.finalize().to_hex())),
    })
}

fn open_verified_corpus_member(
    path: &Path,
    expected_name: &str,
    context: &'static str,
    expected: Option<&RecordedCorpusFileBinding>,
) -> Result<Option<VerifiedCorpusMember>, SourceUnavailable> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let actual = RecordedCorpusFileBinding::from_bytes(expected_name, None)?;
            if let Some(expected) = expected {
                expected.validate_record(expected_name, &actual, context)?;
            }
            return Ok(None);
        }
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "{context} {} cannot be opened without following links: {error}",
                path.display()
            )));
        }
    };
    let initial = file.metadata().map_err(SourceUnavailable::new)?;
    let path_metadata = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    if !initial.file_type().is_file()
        || !path_metadata.file_type().is_file()
        || !opened_file_identity_matches(&path_metadata, &initial)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} is not one regular non-symlink inode",
            path.display()
        )));
    }
    let mut hasher = blake3::Hasher::new();
    let mut length = 0u64;
    let mut buffer = [0u8; STREAM_BINDING_BUFFER_BYTES];
    loop {
        let read = file.read(&mut buffer).map_err(SourceUnavailable::new)?;
        if read == 0 {
            break;
        }
        length = length
            .checked_add(read as u64)
            .ok_or_else(|| SourceUnavailable::new(format!("{context} length overflows u64")))?;
        hasher.update(&buffer[..read]);
    }
    let binding = RecordedCorpusFileBinding {
        name: expected_name.to_owned(),
        present: true,
        length: Some(length),
        blake3: Some(format!("blake3:{}", hasher.finalize().to_hex())),
    };
    if let Some(expected) = expected {
        expected.validate_record(expected_name, &binding, context)?;
    }
    file.seek(SeekFrom::Start(0))
        .map_err(SourceUnavailable::new)?;
    let member = VerifiedCorpusMember {
        path: path.to_path_buf(),
        file,
        initial,
        binding,
        context,
    };
    member.verify_generation()?;
    Ok(Some(member))
}

fn verify_opened_bytes(
    file: &mut std::fs::File,
    path: &Path,
    context: &str,
    expected: &[u8],
    initial_file: &Metadata,
) -> Result<(), SourceUnavailable> {
    let mut offset = 0usize;
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(|error| {
            SourceUnavailable::new(format!(
                "{context} {} cannot be verified: {error}",
                path.display()
            ))
        })?;
        if count == 0 {
            break;
        }
        let end = offset
            .checked_add(count)
            .ok_or_else(|| SourceUnavailable::new(format!("{context} byte count overflow")))?;
        if expected.get(offset..end) != Some(&buffer[..count]) {
            return Err(SourceUnavailable::new(format!(
                "{context} {} changed content during capture",
                path.display()
            )));
        }
        offset = end;
    }
    let final_file = file.metadata().map_err(SourceUnavailable::new)?;
    let final_path = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    if offset != expected.len()
        || !final_file.file_type().is_file()
        || !final_path.file_type().is_file()
        || !opened_file_generation_matches(initial_file, &final_file)
        || !opened_file_identity_matches(&final_path, &final_file)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed identity, type, length, content, or generation during capture",
            path.display()
        )));
    }
    Ok(())
}

fn verify_captured_optional_regular_file(
    path: &Path,
    context: &str,
    expected: Option<&[u8]>,
) -> Result<(), SourceUnavailable> {
    let Some(expected) = expected else {
        return match std::fs::symlink_metadata(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(SourceUnavailable::new(format!(
                "{context} {} cannot be reinspected: {error}",
                path.display()
            ))),
            Ok(_) => Err(SourceUnavailable::new(format!(
                "{context} {} appeared after typed absence was captured",
                path.display()
            ))),
        };
    };
    let actual = capture_optional_regular_file(path, context)?.ok_or_else(|| {
        SourceUnavailable::new(format!(
            "{context} {} disappeared after capture",
            path.display()
        ))
    })?;
    if actual != expected {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed after capture",
            path.display()
        )));
    }
    Ok(())
}

fn canonical_parent(path: &Path, label: &str) -> Result<PathBuf, SourceUnavailable> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{label} {} cannot be inspected: {error}",
            path.display()
        ))
    })?;
    if !metadata.file_type().is_file() {
        return Err(SourceUnavailable::new(format!(
            "{label} {} is not a regular non-symlink file",
            path.display()
        )));
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let canonical = std::fs::canonicalize(parent).map_err(|error| {
        SourceUnavailable::new(format!(
            "{label} parent {} cannot be resolved: {error}",
            parent.display()
        ))
    })?;
    if !std::fs::metadata(&canonical)
        .map_err(SourceUnavailable::new)?
        .is_dir()
    {
        return Err(SourceUnavailable::new(format!(
            "{label} parent {} is not a directory",
            canonical.display()
        )));
    }
    Ok(canonical)
}

fn resolved_corpus_paths(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<(PathBuf, PathBuf, PathBuf), SourceUnavailable> {
    let meta_root = canonical_parent(corpus_meta, "corpus metadata")?;
    let records_root = canonical_parent(corpus_records, "corpus records")?;
    if meta_root != records_root {
        return Err(SourceUnavailable::new(format!(
            "corpus metadata and records have different canonical parent roots ({} versus {}); no cryptographic cross-directory pairing exists",
            meta_root.display(),
            records_root.display()
        )));
    }
    let meta_name = corpus_meta
        .file_name()
        .ok_or_else(|| SourceUnavailable::new("corpus metadata path has no filename"))?;
    let records_name = corpus_records
        .file_name()
        .ok_or_else(|| SourceUnavailable::new("corpus records path has no filename"))?;
    Ok((
        meta_root.clone(),
        meta_root.join(meta_name),
        meta_root.join(records_name),
    ))
}

fn is_canonical_pair(corpus_meta: &Path, corpus_records: &Path) -> bool {
    matches!(
        (corpus_meta.file_name(), corpus_records.file_name()),
        (Some(meta), Some(records))
            if (meta == OsStr::new("corpus.meta") && records == OsStr::new("corpus.records"))
                || (meta == OsStr::new(observation::STATE_FILE)
                    && records == OsStr::new("merged.bin"))
    )
}

fn hidden_path(corpus_records: &Path) -> PathBuf {
    uor_r4_core::transformerless::compiler::corpus_hidden_path(corpus_records)
}

type ProvenanceBytes = [Option<Vec<u8>>; 3];

fn provenance_paths(root: &Path) -> [PathBuf; 3] {
    [
        root.join(observation::MANIFEST_FILE),
        root.join(ATTENTION_OPERATOR_BINDING_FILE),
        root.join(DENSE_OPERATOR_BINDING_FILE),
    ]
}

fn capture_provenance(root: &Path) -> Result<ProvenanceBytes, SourceUnavailable> {
    let paths = provenance_paths(root);
    let mut bytes: ProvenanceBytes = [None, None, None];
    for (slot, path) in paths.iter().enumerate() {
        bytes[slot] = capture_optional_regular_file(path, "recorded corpus provenance")?;
    }
    Ok(bytes)
}

fn verify_provenance(root: &Path, bytes: &ProvenanceBytes) -> Result<(), SourceUnavailable> {
    for (path, expected) in provenance_paths(root).iter().zip(bytes.iter()) {
        verify_captured_optional_regular_file(
            path,
            "recorded corpus provenance",
            expected.as_deref(),
        )?;
    }
    Ok(())
}

fn utf8_file_name(path: &Path, label: &str) -> Result<String, SourceUnavailable> {
    path.file_name()
        .and_then(OsStr::to_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "{label} {} has no UTF-8 filename for canonical binding",
                path.display()
            ))
        })
}

fn binding_path(root: &Path) -> PathBuf {
    root.join(RECORDED_CORPUS_BINDING_FILE)
}

fn parse_binding_bytes(
    path: &Path,
    bytes: &[u8],
) -> Result<RecordedCorpusBinding, SourceUnavailable> {
    reject_duplicate_json(
        bytes,
        &format!("recorded corpus binding {}", path.display()),
    )?;
    let binding: RecordedCorpusBinding = serde_json::from_slice(bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "invalid recorded corpus binding {}: {error}",
            path.display()
        ))
    })?;
    if binding.schema != RECORDED_CORPUS_BINDING_SCHEMA {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus binding {} has unsupported schema {:?}",
            path.display(),
            binding.schema
        )));
    }
    if binding.canonical_bytes()? != bytes {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus binding {} is not canonical pretty JSON with one trailing newline",
            path.display()
        )));
    }
    Ok(binding)
}

fn compile_attempt_path(root: &Path) -> PathBuf {
    root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE)
}

fn parse_compile_attempt_bytes(path: &Path, bytes: &[u8]) -> Result<(), SourceUnavailable> {
    reject_duplicate_json(
        bytes,
        &format!("recorded corpus compile-attempt marker {}", path.display()),
    )?;
    let marker: RecordedCorpusCompileAttempt = serde_json::from_slice(bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "invalid recorded corpus compile-attempt marker {}: {error}",
            path.display()
        ))
    })?;
    let expected = RecordedCorpusCompileAttempt::compile();
    if marker != expected || marker.canonical_bytes()? != bytes {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus compile-attempt marker {} is not the exact canonical compile/1 record",
            path.display()
        )));
    }
    Ok(())
}

fn compile_attempt_writing_name(name: &str) -> bool {
    let prefix = format!(".{RECORDED_CORPUS_COMPILE_ATTEMPT_FILE}.");
    let Some(sequence) = name
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix(".writing"))
    else {
        return false;
    };
    let mut parts = sequence.split('.');
    parts
        .next()
        .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none()
}

fn reserved_compile_attempt_entry_name(
    root: &Path,
    name: &OsStr,
) -> Result<Option<String>, SourceUnavailable> {
    let prefix = format!(".{RECORDED_CORPUS_COMPILE_ATTEMPT_FILE}.");
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let bytes = name.as_bytes();
        if !bytes.starts_with(prefix.as_bytes()) {
            return Ok(None);
        }
        return std::str::from_utf8(bytes)
            .map(|name| Some(name.to_owned()))
            .map_err(|_| {
                SourceUnavailable::new(format!(
                    "recorded corpus root {} contains a non-UTF-8 entry in the reserved compile-attempt namespace",
                    root.display()
                ))
            });
    }
    #[cfg(not(unix))]
    {
        let lossy = name.to_string_lossy();
        if !lossy.starts_with(&prefix) {
            return Ok(None);
        }
        name.to_str().map(|name| Some(name.to_owned())).ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus root {} contains a non-Unicode entry in the reserved compile-attempt namespace",
                root.display()
            ))
        })
    }
}

fn compile_attempt_residues(root: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>, SourceUnavailable> {
    let mut residues = Vec::new();
    for entry in std::fs::read_dir(root).map_err(SourceUnavailable::new)? {
        let entry = entry.map_err(SourceUnavailable::new)?;
        let Some(name) = reserved_compile_attempt_entry_name(root, &entry.file_name())? else {
            continue;
        };
        if !compile_attempt_writing_name(&name) {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} contains unrecognized reserved compile-attempt entry {name:?}",
                root.display()
            )));
        }
        let path = entry.path();
        let bytes =
            capture_optional_regular_file(&path, "recorded corpus compile-attempt staging")?
                .ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "recorded corpus compile-attempt staging {} disappeared",
                        path.display()
                    ))
                })?;
        residues.push((path, bytes));
    }
    residues.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(residues)
}

fn validate_compile_attempt_namespace(
    root: &Path,
    role: RecordedCorpusRole,
) -> Result<(), SourceUnavailable> {
    let stable = compile_attempt_path(root);
    let stable_present =
        capture_optional_regular_file(&stable, "recorded corpus compile-attempt marker")?;
    if let Some(bytes) = stable_present.as_deref() {
        parse_compile_attempt_bytes(&stable, bytes)?;
    }
    let residues = compile_attempt_residues(root)?;
    if role == RecordedCorpusRole::Observation && (stable_present.is_some() || !residues.is_empty())
    {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus root {} contains compile-attempt evidence; refusing observation mutation",
            root.display()
        )));
    }
    Ok(())
}

fn publish_compile_attempt_marker(
    guard: &RecordedCorpusProducerGuard,
) -> Result<(), SourceUnavailable> {
    validate_compile_attempt_namespace(&guard.root, RecordedCorpusRole::Compile)?;
    let expected = RecordedCorpusCompileAttempt::compile().canonical_bytes()?;
    let stable = compile_attempt_path(&guard.root);
    if let Some(bytes) =
        capture_optional_regular_file(&stable, "recorded corpus compile-attempt marker")?
    {
        if bytes != expected {
            return Err(SourceUnavailable::new(
                "recorded corpus compile-attempt marker conflicts with compile/1",
            ));
        }
        for (path, bytes) in compile_attempt_residues(&guard.root)? {
            remove_exact_binding_residue(&path, &bytes, "compile-attempt staging")?;
        }
        return sync_binding_directory(&guard.root);
    }
    for (path, bytes) in compile_attempt_residues(&guard.root)? {
        remove_exact_binding_residue(&path, &bytes, "compile-attempt staging")?;
    }
    let (staging, mut file) = loop {
        let sequence = RECORDED_CORPUS_COMPILE_ATTEMPT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = guard.root.join(format!(
            ".{RECORDED_CORPUS_COMPILE_ATTEMPT_FILE}.{}.{}.writing",
            std::process::id(),
            sequence
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options
                .mode(0o600)
                .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        match options.open(&candidate) {
            Ok(file) => break (candidate, file),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(SourceUnavailable::new(error)),
        }
    };
    file.write_all(&expected)
        .and_then(|()| file.sync_all())
        .map_err(SourceUnavailable::new)?;
    guard.verify_root()?;
    match std::fs::hard_link(&staging, &stable) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let current =
                capture_optional_regular_file(&stable, "recorded corpus compile-attempt marker")?
                    .ok_or_else(|| SourceUnavailable::new("compile-attempt marker disappeared"))?;
            if current != expected {
                return Err(SourceUnavailable::new(
                    "recorded corpus compile-attempt marker appeared with conflicting bytes",
                ));
            }
        }
        Err(error) => return Err(SourceUnavailable::new(error)),
    }
    std::fs::remove_file(&staging).map_err(SourceUnavailable::new)?;
    sync_binding_directory(&guard.root)?;
    let current = capture_optional_regular_file(&stable, "recorded corpus compile-attempt marker")?
        .ok_or_else(|| SourceUnavailable::new("compile-attempt marker disappeared"))?;
    parse_compile_attempt_bytes(&stable, &current)
}

fn finish_compile_attempt_marker(
    guard: &RecordedCorpusProducerGuard,
) -> Result<(), SourceUnavailable> {
    validate_compile_attempt_namespace(&guard.root, RecordedCorpusRole::Compile)?;
    let stable = compile_attempt_path(&guard.root);
    if let Some(bytes) =
        capture_optional_regular_file(&stable, "recorded corpus compile-attempt marker")?
    {
        parse_compile_attempt_bytes(&stable, &bytes)?;
        remove_exact_binding_residue(&stable, &bytes, "recorded corpus compile-attempt marker")?;
    }
    for (path, bytes) in compile_attempt_residues(&guard.root)? {
        remove_exact_binding_residue(&path, &bytes, "compile-attempt staging")?;
    }
    sync_binding_directory(&guard.root)
}

fn binding_temporary_name(name: &str) -> bool {
    let prefix = format!(".{RECORDED_CORPUS_BINDING_FILE}.");
    let Some(sequence) = name
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix(".tmp"))
    else {
        return false;
    };
    let mut parts = sequence.split('.');
    parts
        .next()
        .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none()
}

fn binding_writing_name(name: &str) -> bool {
    let prefix = format!(".{RECORDED_CORPUS_BINDING_FILE}.");
    let Some(sequence) = name
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix(".writing"))
    else {
        return false;
    };
    let mut parts = sequence.split('.');
    parts
        .next()
        .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none()
}

fn reserved_binding_entry_name(
    root: &Path,
    name: &OsStr,
) -> Result<Option<String>, SourceUnavailable> {
    let prefix = format!(".{RECORDED_CORPUS_BINDING_FILE}.");

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        let bytes = name.as_bytes();
        if !bytes.starts_with(prefix.as_bytes()) {
            return Ok(None);
        }
        let name = std::str::from_utf8(bytes).map_err(|_| {
            SourceUnavailable::new(format!(
                "recorded corpus root {} contains a non-UTF-8 entry in the reserved binding publication namespace",
                root.display()
            ))
        })?;
        Ok(Some(name.to_owned()))
    }

    #[cfg(not(unix))]
    {
        let lossy = name.to_string_lossy();
        if !lossy.starts_with(&prefix) {
            return Ok(None);
        }
        let name = name.to_str().ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus root {} contains a non-Unicode entry in the reserved binding publication namespace",
                root.display()
            ))
        })?;
        Ok(Some(name.to_owned()))
    }
}

#[derive(Default)]
struct BindingPublicationResidues {
    temporaries: Vec<(PathBuf, Vec<u8>)>,
    writings: Vec<(PathBuf, Vec<u8>)>,
}

fn binding_publication_residues(
    root: &Path,
    capture_writings: bool,
) -> Result<BindingPublicationResidues, SourceUnavailable> {
    let mut residues = BindingPublicationResidues::default();
    for entry in std::fs::read_dir(root).map_err(SourceUnavailable::new)? {
        let entry = entry.map_err(SourceUnavailable::new)?;
        let name = entry.file_name();
        let Some(name) = reserved_binding_entry_name(root, &name)? else {
            continue;
        };
        if binding_temporary_name(&name) {
            let path = entry.path();
            let bytes = capture_optional_regular_file(&path, "recorded corpus binding temporary")?
                .ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "recorded corpus binding temporary {} disappeared during validation",
                        path.display()
                    ))
                })?;
            let _ = parse_binding_bytes(&path, &bytes)?;
            residues.temporaries.push((path, bytes));
        } else if binding_writing_name(&name) {
            // A `.writing` inode is deliberately non-authoritative: readers
            // ignore it, while the exclusive publisher may validate and
            // reclaim it before beginning a new publication attempt.
            if capture_writings {
                let path = entry.path();
                let bytes = capture_optional_regular_file(
                    &path,
                    "recorded corpus non-authoritative binding staging file",
                )?
                .ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "recorded corpus binding staging file {} disappeared during validation",
                        path.display()
                    ))
                })?;
                residues.writings.push((path, bytes));
            }
        } else {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} contains unrecognized entry {name:?} in the reserved binding publication namespace",
                root.display()
            )));
        }
    }
    residues
        .temporaries
        .sort_by(|left, right| left.0.cmp(&right.0));
    residues
        .writings
        .sort_by(|left, right| left.0.cmp(&right.0));
    Ok(residues)
}

fn binding_temporaries(root: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>, SourceUnavailable> {
    Ok(binding_publication_residues(root, false)?.temporaries)
}

fn require_no_binding_temporaries(root: &Path) -> Result<(), SourceUnavailable> {
    let temporaries = binding_temporaries(root)?;
    if temporaries.is_empty() {
        return Ok(());
    }
    Err(SourceUnavailable::new(format!(
        "recorded corpus root {} contains an unpublished canonical binding temporary; writer recovery is required before reading",
        root.display()
    )))
}

fn make_streaming_binding(
    meta_path: &Path,
    records_path: &Path,
    hidden_path: &Path,
    provenance: &ProvenanceBytes,
    meta_bytes: &[u8],
) -> Result<RecordedCorpusBinding, SourceUnavailable> {
    let metadata_name = utf8_file_name(meta_path, "recorded corpus metadata")?;
    let records_name = utf8_file_name(records_path, "recorded corpus records")?;
    let hidden_name = utf8_file_name(hidden_path, "recorded corpus hidden stream")?;
    Ok(RecordedCorpusBinding {
        schema: RECORDED_CORPUS_BINDING_SCHEMA.to_owned(),
        manifest: RecordedCorpusFileBinding::from_bytes(
            observation::MANIFEST_FILE,
            provenance[0].as_deref(),
        )?,
        attention_operator: RecordedCorpusFileBinding::from_bytes(
            ATTENTION_OPERATOR_BINDING_FILE,
            provenance[1].as_deref(),
        )?,
        dense_operator: RecordedCorpusFileBinding::from_bytes(
            DENSE_OPERATOR_BINDING_FILE,
            provenance[2].as_deref(),
        )?,
        metadata: RecordedCorpusFileBinding::from_bytes(&metadata_name, Some(meta_bytes))?,
        records: stream_regular_file_binding(
            records_path,
            &records_name,
            "recorded corpus records",
        )?,
        hidden: stream_regular_file_binding(
            hidden_path,
            &hidden_name,
            "recorded corpus hidden stream",
        )?,
    })
}

fn validate_finalized_corpus_shape(
    meta_bytes: &[u8],
    records: &RecordedCorpusFileBinding,
) -> Result<(), SourceUnavailable> {
    if meta_bytes.len() != 25 || meta_bytes[24] != 1 {
        return Err(SourceUnavailable::new(
            "recorded corpus binding requires one exact finalized 25-byte metadata checkpoint",
        ));
    }
    let count = u64::from_le_bytes(
        meta_bytes[0..8]
            .try_into()
            .map_err(|_| SourceUnavailable::new("invalid finalized corpus count"))?,
    );
    let length = records.length.ok_or_else(|| {
        SourceUnavailable::new("recorded corpus records are absent during binding publication")
    })?;
    if count == 0
        || ![88u64, 48, 32, 12]
            .into_iter()
            .any(|width| count.checked_mul(width) == Some(length))
    {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus has {count} rows and {length} record bytes, not one exact registered finalized layout"
        )));
    }
    Ok(())
}

struct CapturedBindingMembers<'a> {
    meta_path: &'a Path,
    records_path: &'a Path,
    hidden_path: &'a Path,
    provenance: &'a ProvenanceBytes,
    meta_bytes: &'a [u8],
    records_bytes: &'a [u8],
    hidden_bytes: Option<&'a [u8]>,
}

fn validate_binding_generation(
    binding: &RecordedCorpusBinding,
    members: &CapturedBindingMembers<'_>,
    context: &str,
) -> Result<(), SourceUnavailable> {
    binding.manifest.validate(
        observation::MANIFEST_FILE,
        members.provenance[0].as_deref(),
        context,
    )?;
    binding.attention_operator.validate(
        ATTENTION_OPERATOR_BINDING_FILE,
        members.provenance[1].as_deref(),
        context,
    )?;
    binding.dense_operator.validate(
        DENSE_OPERATOR_BINDING_FILE,
        members.provenance[2].as_deref(),
        context,
    )?;
    binding.metadata.validate(
        &utf8_file_name(members.meta_path, "recorded corpus metadata")?,
        Some(members.meta_bytes),
        context,
    )?;
    binding.records.validate(
        &utf8_file_name(members.records_path, "recorded corpus records")?,
        Some(members.records_bytes),
        context,
    )?;
    binding.hidden.validate(
        &utf8_file_name(members.hidden_path, "recorded corpus hidden stream")?,
        members.hidden_bytes,
        context,
    )
}

fn parse_execution_identity(
    root: &Path,
    corpus_meta: &Path,
    corpus_records: &Path,
    bytes: &ProvenanceBytes,
) -> Result<RecordedCorpusExecutionIdentity, SourceUnavailable> {
    let manifest_path = root.join(observation::MANIFEST_FILE);
    let manifest = bytes[0]
        .as_deref()
        .map(|bytes| {
            reject_duplicate_json(
                bytes,
                &format!("observation manifest {}", manifest_path.display()),
            )
            .map_err(|mut error| {
                error.reason = format!(
                    "{}: malformed observation manifest: {}",
                    manifest_path.display(),
                    error.reason
                );
                error
            })?;
            ObservationManifest::parse_captured_execution_identity(root, &manifest_path, bytes)
                .map_err(|mut error| {
                    error.reason = format!(
                        "{}: malformed observation manifest: {}",
                        manifest_path.display(),
                        error.reason
                    );
                    error
                })
        })
        .transpose()?;
    let manifest_present = manifest.is_some();

    let attention_path = root.join(ATTENTION_OPERATOR_BINDING_FILE);
    let sidecar_attention = bytes[1]
        .as_deref()
        .map(|bytes| {
            reject_duplicate_json(
                bytes,
                &format!("attention operator sidecar {}", attention_path.display()),
            )?;
            let operator: AttentionOperatorSpec =
                serde_json::from_slice(bytes).map_err(|error| {
                    SourceUnavailable::new(format!(
                        "invalid attention operator sidecar {}: {error}",
                        attention_path.display()
                    ))
                })?;
            observation::validate_registered_source_attention_operator(&operator)?;
            Ok::<_, SourceUnavailable>(operator)
        })
        .transpose()?;
    let dense_path = root.join(DENSE_OPERATOR_BINDING_FILE);
    let sidecar_dense = bytes[2]
        .as_deref()
        .map(|bytes| {
            reject_duplicate_json(
                bytes,
                &format!("dense operator sidecar {}", dense_path.display()),
            )?;
            let operator: DenseOperatorSpec = serde_json::from_slice(bytes).map_err(|error| {
                SourceUnavailable::new(format!(
                    "invalid dense operator sidecar {}: {error}",
                    dense_path.display()
                ))
            })?;
            observation::validate_registered_source_dense_operator(&operator)?;
            Ok::<_, SourceUnavailable>(operator)
        })
        .transpose()?;
    let manifest_attention = manifest
        .as_ref()
        .and_then(|manifest| manifest.attention_operator.clone());
    let manifest_dense = manifest
        .as_ref()
        .and_then(|manifest| manifest.dense_operator.clone());

    if (manifest_present || sidecar_attention.is_some() || sidecar_dense.is_some())
        && !is_canonical_pair(corpus_meta, corpus_records)
    {
        return Err(SourceUnavailable::new(format!(
            "recorded-corpus provenance in {} applies only to the canonical corpus.meta/corpus.records or state.bin/merged.bin pair; refusing to attach it to {}/{}",
            root.display(),
            corpus_meta.display(),
            corpus_records.display()
        )));
    }
    if let Some(sidecar) = sidecar_attention.as_ref()
        && manifest_present
        && manifest_attention.is_none()
    {
        return Err(SourceUnavailable::new(format!(
            "{} declares attention operator {}/{} but {} records the legacy operatorless era",
            attention_path.display(),
            sidecar.id,
            sidecar.version,
            manifest_path.display()
        )));
    }
    if let Some(sidecar) = sidecar_dense.as_ref()
        && manifest_present
        && manifest_dense.is_none()
    {
        return Err(SourceUnavailable::new(format!(
            "{} declares dense operator {}/{} but {} records the dense-operator-absent era",
            dense_path.display(),
            sidecar.id,
            sidecar.version,
            manifest_path.display()
        )));
    }

    let attention_operator = match (sidecar_attention, manifest_attention) {
        (Some(sidecar), Some(manifest)) if sidecar != manifest => {
            return Err(SourceUnavailable::new(format!(
                "{} and {} declare different attention operators",
                attention_path.display(),
                manifest_path.display()
            )));
        }
        (Some(sidecar), _) => Some(sidecar),
        (None, manifest) => manifest,
    };
    let dense_operator = match (sidecar_dense, manifest_dense) {
        (Some(sidecar), Some(manifest)) if sidecar != manifest => {
            return Err(SourceUnavailable::new(format!(
                "{} and {} declare different dense operators",
                dense_path.display(),
                manifest_path.display()
            )));
        }
        (Some(sidecar), _) => Some(sidecar),
        (None, manifest) => manifest,
    };
    if attention_operator.is_none()
        && let Some(dense) = dense_operator.as_ref()
    {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus declares dense operator {}/{} without an attention operator",
            dense.id, dense.version
        )));
    }
    observation::validate_source_execution_identity(
        attention_operator.as_ref(),
        dense_operator.as_ref(),
        "recorded corpus provenance",
    )?;
    Ok(RecordedCorpusExecutionIdentity {
        attention_operator,
        dense_operator,
    })
}

/// Require the caller-declared pair to equal the exact recorded pair. Typed
/// absence is significant: a current dense corpus cannot be relabeled by
/// omitting `--dense-operator`, and a legacy corpus cannot acquire one.
pub fn require_execution_identity(
    recorded: &RecordedCorpusExecutionIdentity,
    attention: Option<&AttentionOperatorSpec>,
    dense: Option<&DenseOperatorSpec>,
    context: &str,
) -> Result<(), SourceUnavailable> {
    if let Some(attention) = attention {
        observation::validate_registered_source_attention_operator(attention)?;
    }
    if let Some(dense) = dense {
        observation::validate_registered_source_dense_operator(dense)?;
    }
    observation::validate_source_execution_identity(attention, dense, context)?;
    if recorded.attention_operator.as_ref() != attention {
        return Err(SourceUnavailable::new(format!(
            "{context}: requested attention execution identity does not match the recorded corpus; refusing before output mutation"
        )));
    }
    if recorded.dense_operator.as_ref() != dense {
        return Err(SourceUnavailable::new(format!(
            "{context}: requested dense execution identity does not match the recorded corpus; refusing before output mutation"
        )));
    }
    Ok(())
}

pub(crate) fn capture_with_hooks<F, G>(
    corpus_meta: &Path,
    corpus_records: &Path,
    after_provenance_capture: F,
    after_corpus_capture: G,
) -> Result<RecordedCorpusSnapshot, SourceUnavailable>
where
    F: FnOnce(),
    G: FnOnce(),
{
    let (root, resolved_meta, resolved_records) =
        resolved_corpus_paths(corpus_meta, corpus_records)?;
    require_no_binding_temporaries(&root)?;
    let stable_binding_path = binding_path(&root);
    let binding_bytes =
        capture_optional_regular_file(&stable_binding_path, "recorded corpus generation binding")?;
    let binding = binding_bytes
        .as_deref()
        .map(|bytes| parse_binding_bytes(&stable_binding_path, bytes))
        .transpose()?;
    let compile_attempt_path = compile_attempt_path(&root);
    let compile_attempt = capture_optional_regular_file(
        &compile_attempt_path,
        "recorded corpus compile-attempt marker",
    )?;
    if let Some(bytes) = compile_attempt.as_deref() {
        parse_compile_attempt_bytes(&compile_attempt_path, bytes)?;
        match binding.as_ref() {
            Some(binding) => {
                validate_binding_role(binding, RecordedCorpusRole::Compile, &stable_binding_path)?
            }
            None => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} has an unfinished compile attempt without a stable generation binding",
                    root.display()
                )));
            }
        }
    }

    let provenance = capture_provenance(&root)?;
    after_provenance_capture();

    let meta_bytes = capture_optional_regular_file(&resolved_meta, "recorded corpus metadata")?
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus metadata {} disappeared during capture",
                resolved_meta.display()
            ))
        })?;
    let records_bytes =
        capture_optional_regular_file(&resolved_records, "recorded corpus records")?.ok_or_else(
            || {
                SourceUnavailable::new(format!(
                    "recorded corpus records {} disappeared during capture",
                    resolved_records.display()
                ))
            },
        )?;
    let hidden_path = hidden_path(&resolved_records);
    let hidden_bytes =
        capture_optional_regular_file(&hidden_path, "recorded corpus hidden stream")?;

    after_corpus_capture();
    let execution =
        parse_execution_identity(&root, &resolved_meta, &resolved_records, &provenance)?;

    let binding_cid = if let (Some(binding), Some(bytes)) = (&binding, binding_bytes.as_deref()) {
        validate_binding_generation(
            binding,
            &CapturedBindingMembers {
                meta_path: &resolved_meta,
                records_path: &resolved_records,
                hidden_path: &hidden_path,
                provenance: &provenance,
                meta_bytes: &meta_bytes,
                records_bytes: &records_bytes,
                hidden_bytes: hidden_bytes.as_deref(),
            },
            &format!("recorded corpus binding {}", stable_binding_path.display()),
        )?;
        Some(format!("blake3:{}", blake3::hash(bytes).to_hex()))
    } else {
        // Markerless compatibility retains the historical repeated-read
        // defense. It is intentionally not advertised as an adversarial
        // cross-file transaction: only the canonical binding supplies that
        // guarantee.
        verify_provenance(&root, &provenance)?;
        verify_captured_optional_regular_file(
            &resolved_meta,
            "recorded corpus metadata",
            Some(meta_bytes.as_slice()),
        )?;
        verify_captured_optional_regular_file(
            &resolved_records,
            "recorded corpus records",
            Some(records_bytes.as_slice()),
        )?;
        verify_captured_optional_regular_file(
            &hidden_path,
            "recorded corpus hidden stream",
            hidden_bytes.as_deref(),
        )?;
        verify_captured_optional_regular_file(
            &stable_binding_path,
            "recorded corpus generation binding",
            None,
        )?;
        require_no_binding_temporaries(&root)?;
        if let Some(dense) = execution.dense_operator.as_ref() {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus in {} declares dense operator {}/{} but has no canonical {}; dense-present generations must be committed last before use",
                root.display(),
                dense.id,
                dense.version,
                RECORDED_CORPUS_BINDING_FILE
            )));
        }
        None
    };
    Ok(RecordedCorpusSnapshot {
        execution,
        attention_operator_bytes: provenance[1].clone(),
        dense_operator_bytes: provenance[2].clone(),
        meta_bytes,
        records_bytes,
        hidden_bytes,
        binding,
        binding_cid,
    })
}

/// Capture and reverify provenance and corpus bytes as one transaction.
pub fn capture(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusSnapshot, SourceUnavailable> {
    capture_with_hooks(corpus_meta, corpus_records, || {}, || {})
}

/// Open one bounded-memory recorded-corpus generation.
///
/// Large records and hidden-state members are hashed through retained
/// no-follow handles and rewound; their bodies are never accumulated here.
/// A consumer may seek/read those handles and must call
/// [`RecordedCorpusStreamSnapshot::verify_generation`] after its final read
/// and immediately before publishing any derived generation. Markerless
/// compatibility callers additionally hold a [`RecordedCorpusProducerGuard`]
/// for the source while opening, reading, and performing that final check.
pub fn open_stream(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusStreamSnapshot, SourceUnavailable> {
    let (root, resolved_meta, resolved_records) =
        resolved_corpus_paths(corpus_meta, corpus_records)?;
    require_no_binding_temporaries(&root)?;
    let stable_path = binding_path(&root);
    let binding_bytes =
        capture_optional_regular_file(&stable_path, "recorded corpus generation binding")?;
    let binding = binding_bytes
        .as_deref()
        .map(|bytes| parse_binding_bytes(&stable_path, bytes))
        .transpose()?;
    let marker_path = compile_attempt_path(&root);
    let marker_bytes =
        capture_optional_regular_file(&marker_path, "recorded corpus compile-attempt marker")?;
    if let Some(bytes) = marker_bytes.as_deref() {
        parse_compile_attempt_bytes(&marker_path, bytes)?;
        match binding.as_ref() {
            Some(binding) => {
                validate_binding_role(binding, RecordedCorpusRole::Compile, &stable_path)?
            }
            None => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} has an unfinished compile attempt without a stable generation binding",
                    root.display()
                )));
            }
        }
    }

    let provenance = capture_provenance(&root)?;
    let meta_bytes = capture_optional_regular_file(&resolved_meta, "recorded corpus metadata")?
        .ok_or_else(|| SourceUnavailable::new("recorded corpus metadata is absent"))?;
    let hidden = hidden_path(&resolved_records);
    let records_name = utf8_file_name(&resolved_records, "recorded corpus records")?;
    let hidden_name = utf8_file_name(&hidden, "recorded corpus hidden stream")?;
    let records = open_verified_corpus_member(
        &resolved_records,
        &records_name,
        "recorded corpus records",
        binding.as_ref().map(|binding| &binding.records),
    )?
    .ok_or_else(|| SourceUnavailable::new("recorded corpus records are absent"))?;
    let hidden_member = open_verified_corpus_member(
        &hidden,
        &hidden_name,
        "recorded corpus hidden stream",
        binding.as_ref().map(|binding| &binding.hidden),
    )?;
    let execution =
        parse_execution_identity(&root, &resolved_meta, &resolved_records, &provenance)?;

    let binding_cid = if let (Some(binding), Some(bytes)) =
        (binding.as_ref(), binding_bytes.as_deref())
    {
        let context = format!("recorded corpus binding {}", stable_path.display());
        binding.manifest.validate(
            observation::MANIFEST_FILE,
            provenance[0].as_deref(),
            &context,
        )?;
        binding.attention_operator.validate(
            ATTENTION_OPERATOR_BINDING_FILE,
            provenance[1].as_deref(),
            &context,
        )?;
        binding.dense_operator.validate(
            DENSE_OPERATOR_BINDING_FILE,
            provenance[2].as_deref(),
            &context,
        )?;
        binding.metadata.validate(
            &utf8_file_name(&resolved_meta, "recorded corpus metadata")?,
            Some(&meta_bytes),
            &context,
        )?;
        Some(format!("blake3:{}", blake3::hash(bytes).to_hex()))
    } else {
        if let Some(dense) = execution.dense_operator.as_ref() {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus in {} declares dense operator {}/{} but has no canonical {}; dense-present generations must be committed last before use",
                root.display(),
                dense.id,
                dense.version,
                RECORDED_CORPUS_BINDING_FILE
            )));
        }
        None
    };

    let snapshot = RecordedCorpusStreamSnapshot {
        execution,
        attention_operator_bytes: provenance[1].clone(),
        dense_operator_bytes: provenance[2].clone(),
        meta_bytes,
        records,
        hidden: hidden_member,
        binding_cid,
        root,
        provenance,
        meta_path: resolved_meta,
        binding_path: stable_path,
        binding_bytes,
        marker_path,
        marker_bytes,
        hidden_path: hidden,
    };
    snapshot.verify_generation()?;
    Ok(snapshot)
}

/// Bounded-memory identity resolver for status/copy callers. It validates the
/// complete bound generation while retaining no corpus body in memory.
pub fn execution_identity(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusExecutionIdentity, SourceUnavailable> {
    let (root, _, _) = resolved_corpus_paths(corpus_meta, corpus_records)?;
    let source_guard = RecordedCorpusReaderGuard::try_acquire(&root)?;
    let snapshot = open_stream(corpus_meta, corpus_records)?;
    snapshot.verify_generation()?;
    source_guard.verify()?;
    Ok(snapshot.execution)
}

/// Open a stream while the caller already owns the exact producer root. This
/// avoids self-contention in source/server publication paths while preserving
/// the same root-identity and final-generation checks.
pub fn open_stream_under_guard(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusStreamSnapshot, SourceUnavailable> {
    let (root, _, _) = resolved_corpus_paths(corpus_meta, corpus_records)?;
    if !guard.protects_directory(&root)? {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus producer guard for {} does not own stream root {}",
            guard.root().display(),
            root.display()
        )));
    }
    let snapshot = open_stream(corpus_meta, corpus_records)?;
    snapshot.verify_generation()?;
    guard.verify_owned_root()?;
    Ok(snapshot)
}

fn sync_binding_directory(root: &Path) -> Result<(), SourceUnavailable> {
    std::fs::File::open(root)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            SourceUnavailable::new(format!(
                "recorded corpus root {} cannot be synchronized after binding publication: {error}",
                root.display()
            ))
        })
}

fn create_binding_temporary(root: &Path, bytes: &[u8]) -> Result<PathBuf, SourceUnavailable> {
    loop {
        let sequence = RECORDED_CORPUS_BINDING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let writing = root.join(format!(
            ".{RECORDED_CORPUS_BINDING_FILE}.{}.{}.writing",
            std::process::id(),
            sequence
        ));
        let temporary = root.join(format!(
            ".{RECORDED_CORPUS_BINDING_FILE}.{}.{}.tmp",
            std::process::id(),
            sequence
        ));
        match std::fs::symlink_metadata(&temporary) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => continue,
            Err(error) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus binding temporary {} cannot be inspected: {error}",
                    temporary.display()
                )));
            }
        }
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&writing)
        {
            Ok(mut file) => {
                if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
                    let _ = std::fs::remove_file(&writing);
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus binding staging file {} cannot be written: {error}",
                        writing.display()
                    )));
                }
                drop(file);
                std::fs::rename(&writing, &temporary).map_err(|error| {
                    SourceUnavailable::new(format!(
                        "recorded corpus binding staging promotion {} -> {} failed: {error}",
                        writing.display(),
                        temporary.display()
                    ))
                })?;
                sync_binding_directory(root)?;
                return Ok(temporary);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus binding staging file {} cannot be created: {error}",
                    writing.display()
                )));
            }
        }
    }
}

fn remove_exact_binding_residue(
    path: &Path,
    expected: &[u8],
    context: &str,
) -> Result<(), SourceUnavailable> {
    let current = capture_optional_regular_file(path, context)?.ok_or_else(|| {
        SourceUnavailable::new(format!(
            "{context} {} disappeared before recovery",
            path.display()
        ))
    })?;
    if current != expected {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed before recovery",
            path.display()
        )));
    }
    std::fs::remove_file(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot be reclaimed: {error}",
            path.display()
        ))
    })
}

/// Read-only verification that every stable/recoverable binding record, when
/// present, is the canonical binding for the exact current member bytes.
/// Deterministic producers call this after all destination-member preflights
/// and before publishing their compile-attempt marker, so a terminal recovery
/// conflict leaves the corpus root byte-identical.
pub fn preflight_binding_evidence_matches_current(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<(), SourceUnavailable> {
    preflight_binding_evidence_matches_current_inner(guard, corpus_meta, corpus_records, true)
}

/// Publication state accepted for a resumable source-generation update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceUpdatePublicationState {
    /// An exact canonical binding temporary commits the current member bytes
    /// and must be promoted before the generator changes them again.
    pub recoverable_binding: bool,
    /// A stable compile-attempt marker proves an interrupted same-role update.
    pub attempt_active: bool,
}

/// Validate source-update binding evidence without confusing an honest
/// binding-last crash with a deterministic-output conflict.
///
/// With no active compile marker, every stable binding and temporary must
/// commit the exact current member bytes. With an active marker, a stable
/// binding may remain the canonical last-good generation while the
/// authoritative resume checkpoint has advanced; canonical temporaries must
/// still match the current bytes exactly. The caller is responsible for
/// validating that checkpoint before invoking this seam.
pub fn preflight_source_update_publication(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<SourceUpdatePublicationState, SourceUnavailable> {
    guard.verify_root()?;
    let recoverable_binding =
        guard.preflight_publication_namespace_for(RecordedCorpusRole::Compile)?;
    let marker = compile_attempt_path(&guard.root);
    let attempt_active =
        capture_optional_regular_file(&marker, "recorded corpus compile-attempt marker")?
            .map(|bytes| parse_compile_attempt_bytes(&marker, &bytes))
            .transpose()?
            .is_some();
    let stable = capture_optional_regular_file(
        &binding_path(&guard.root),
        "recorded corpus generation binding",
    )?
    .is_some();
    if recoverable_binding || (stable && !attempt_active) {
        preflight_binding_evidence_matches_current_inner(
            guard,
            corpus_meta,
            corpus_records,
            !attempt_active,
        )?;
    }
    Ok(SourceUpdatePublicationState {
        recoverable_binding,
        attempt_active,
    })
}

fn preflight_binding_evidence_matches_current_inner(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
    check_stable: bool,
) -> Result<(), SourceUnavailable> {
    guard.verify_root()?;
    let (root, resolved_meta, resolved_records) =
        resolved_corpus_paths(corpus_meta, corpus_records)?;
    if root != guard.root {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus producer guard owns {}, but binding preflight resolved to {}",
            guard.root.display(),
            root.display()
        )));
    }
    if !is_canonical_pair(&resolved_meta, &resolved_records) {
        return Err(SourceUnavailable::new(
            "binding preflight requires a canonical corpus.meta/corpus.records or state.bin/merged.bin pair",
        ));
    }
    let role = if resolved_meta.file_name() == Some(OsStr::new("corpus.meta")) {
        RecordedCorpusRole::Compile
    } else {
        RecordedCorpusRole::Observation
    };
    validate_role_inventory(&root, role)?;
    validate_compile_attempt_namespace(&root, role)?;
    let provenance = capture_provenance(&root)?;
    let meta_bytes = capture_optional_regular_file(&resolved_meta, "recorded corpus metadata")?
        .ok_or_else(|| SourceUnavailable::new("binding preflight metadata is absent"))?;
    let resolved_hidden = hidden_path(&resolved_records);
    let execution =
        parse_execution_identity(&root, &resolved_meta, &resolved_records, &provenance)?;
    let binding = make_streaming_binding(
        &resolved_meta,
        &resolved_records,
        &resolved_hidden,
        &provenance,
        &meta_bytes,
    )?;
    if !binding.records.present {
        return Err(SourceUnavailable::new(
            "binding preflight records are absent",
        ));
    }
    if execution.dense_operator.is_some() {
        validate_finalized_corpus_shape(&meta_bytes, &binding.records)?;
    }
    let expected = binding.canonical_bytes()?;
    let stable_path = binding_path(&root);
    if let Some(bytes) =
        capture_optional_regular_file(&stable_path, "recorded corpus generation binding")?
            .filter(|_| check_stable)
    {
        let binding = parse_binding_bytes(&stable_path, &bytes)?;
        validate_binding_role(&binding, role, &stable_path)?;
        if bytes != expected {
            return Err(SourceUnavailable::new(format!(
                "stable recorded corpus binding {} does not commit the exact planned generation",
                stable_path.display()
            )));
        }
    }
    let residues = binding_publication_residues(&root, true)?;
    for (path, bytes) in residues.temporaries {
        let binding = parse_binding_bytes(&path, &bytes)?;
        validate_binding_role(&binding, role, &path)?;
        if bytes != expected {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus binding temporary {} does not commit the exact planned generation",
                path.display()
            )));
        }
    }
    guard.verify_root()
}

/// Publish or repair the canonical generation binding after a producer has
/// completed a canonical corpus pair. The binding is the final commit point.
///
/// Canonical publisher temporaries are recovered only when every byte exactly
/// matches the intended current generation. Malformed, nonregular, unknown,
/// or conflicting canonical residue remains untouched and terminal.
///
/// The reserved `.{RECORDED_CORPUS_BINDING_FILE}.<pid>.<sequence>.writing`
/// namespace is non-authoritative and ignored by readers. This function may
/// reclaim registry-shaped regular `.writing` residue, including zero or
/// partial bytes left by process death, only under this API's hard precondition
/// that its caller holds exclusive producer ownership of the corpus root for
/// the entire publication attempt. Callers must not use this function as an
/// unlocked cleanup utility.
pub fn publish_binding(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<String, SourceUnavailable> {
    guard.verify_root()?;
    let (root, resolved_meta, resolved_records) =
        resolved_corpus_paths(corpus_meta, corpus_records)?;
    if root != guard.root {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus producer guard owns {}, but binding publication resolved to {}; refusing cross-root publication before mutation",
            guard.root.display(),
            root.display()
        )));
    }
    guard.verify_root()?;
    if !is_canonical_pair(&resolved_meta, &resolved_records) {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus generation bindings are supported only for canonical corpus.meta/corpus.records or state.bin/merged.bin pairs, not {}/{}",
            resolved_meta.display(),
            resolved_records.display()
        )));
    }
    let intended_role = if resolved_meta.file_name() == Some(OsStr::new("corpus.meta")) {
        RecordedCorpusRole::Compile
    } else {
        RecordedCorpusRole::Observation
    };
    validate_role_inventory(&root, intended_role)?;
    validate_compile_attempt_namespace(&root, intended_role)?;
    if intended_role == RecordedCorpusRole::Compile {
        let marker_path = compile_attempt_path(&root);
        let marker = capture_optional_regular_file(
            &marker_path,
            "recorded corpus compile-attempt marker",
        )?
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "compile-role binding publication in {} requires an exact stable {RECORDED_CORPUS_COMPILE_ATTEMPT_FILE} marker before any commit mutation",
                root.display()
            ))
        })?;
        parse_compile_attempt_bytes(&marker_path, &marker)?;
    }
    let provenance = capture_provenance(&root)?;
    let meta_bytes = capture_optional_regular_file(&resolved_meta, "recorded corpus metadata")?
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus metadata {} is absent during binding publication",
                resolved_meta.display()
            ))
        })?;
    let resolved_hidden = hidden_path(&resolved_records);
    let execution =
        parse_execution_identity(&root, &resolved_meta, &resolved_records, &provenance)?;
    let binding = make_streaming_binding(
        &resolved_meta,
        &resolved_records,
        &resolved_hidden,
        &provenance,
        &meta_bytes,
    )?;
    if !binding.records.present {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus records {} are absent during binding publication",
            resolved_records.display()
        )));
    }
    if execution.dense_operator.is_some() {
        validate_finalized_corpus_shape(&meta_bytes, &binding.records).map_err(|mut error| {
            error.reason = format!(
                "dense-present recorded corpus in {} is incomplete or malformed: {}",
                root.display(),
                error.reason
            );
            error
        })?;
    }
    let expected = binding.canonical_bytes()?;
    let expected_cid = format!("blake3:{}", blake3::hash(&expected).to_hex());
    let stable_path = binding_path(&root);
    let stable = capture_optional_regular_file(&stable_path, "recorded corpus generation binding")?;
    if let Some(bytes) = stable.as_deref() {
        let existing = parse_binding_bytes(&stable_path, bytes)?;
        validate_binding_role(&existing, intended_role, &stable_path)?;
    }
    let residues = binding_publication_residues(&root, true)?;
    let temporaries = residues.temporaries;
    let writings = residues.writings;
    if let Some((path, _)) = temporaries
        .iter()
        .find(|(_, bytes)| bytes.as_slice() != expected.as_slice())
    {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus binding temporary {} conflicts with the intended current generation; refusing recovery before mutation",
            path.display()
        )));
    }

    if stable.as_deref() == Some(expected.as_slice()) {
        for (path, bytes) in temporaries {
            remove_exact_binding_residue(&path, &bytes, "recorded corpus binding temporary")?;
        }
        for (path, bytes) in writings {
            remove_exact_binding_residue(
                &path,
                &bytes,
                "recorded corpus non-authoritative binding staging file",
            )?;
        }
        sync_binding_directory(&root)?;
        return Ok(expected_cid);
    }

    for (path, bytes) in writings {
        remove_exact_binding_residue(
            &path,
            &bytes,
            "recorded corpus non-authoritative binding staging file",
        )?;
    }

    let selected = match temporaries.first() {
        Some((path, bytes)) => {
            let current = capture_optional_regular_file(path, "recorded corpus binding temporary")?
                .ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "recorded corpus binding temporary {} disappeared before publication",
                        path.display()
                    ))
                })?;
            if current.as_slice() != bytes.as_slice() || current != expected {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus binding temporary {} changed before publication",
                    path.display()
                )));
            }
            path.clone()
        }
        None => create_binding_temporary(&root, &expected)?,
    };

    #[cfg(windows)]
    if stable.is_some() {
        std::fs::remove_file(&stable_path).map_err(SourceUnavailable::new)?;
    }
    std::fs::rename(&selected, &stable_path).map_err(|error| {
        SourceUnavailable::new(format!(
            "recorded corpus binding publication {} -> {} failed: {error}",
            selected.display(),
            stable_path.display()
        ))
    })?;
    for (path, bytes) in temporaries {
        if path != selected {
            remove_exact_binding_residue(&path, &bytes, "recorded corpus binding temporary")?;
        }
    }
    sync_binding_directory(&root)?;
    let published = capture_optional_regular_file(
        &stable_path,
        "published recorded corpus generation binding",
    )?
    .ok_or_else(|| {
        SourceUnavailable::new(format!(
            "published recorded corpus binding {} disappeared",
            stable_path.display()
        ))
    })?;
    if published != expected {
        return Err(SourceUnavailable::new(format!(
            "published recorded corpus binding {} does not match the intended current generation",
            stable_path.display()
        )));
    }
    Ok(expected_cid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

    fn unique_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "uor-r4-recorded-corpus-{label}-{}-{}",
            std::process::id(),
            NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn write_operator<T: serde::Serialize>(path: &Path, operator: &T) {
        let mut bytes = serde_json::to_vec_pretty(operator).expect("operator JSON");
        bytes.push(b'\n');
        std::fs::write(path, bytes).expect("operator sidecar");
    }

    fn write_corpus_pair(root: &Path, meta_name: &str, records_name: &str) -> (PathBuf, PathBuf) {
        std::fs::create_dir_all(root).expect("root");
        let meta = root.join(meta_name);
        let records = root.join(records_name);
        std::fs::write(&meta, b"captured metadata").expect("metadata");
        std::fs::write(&records, b"captured records").expect("records");
        (meta, records)
    }

    fn complete_source_corpus(root: &Path, next: u32) -> (PathBuf, PathBuf) {
        std::fs::create_dir_all(root).expect("root");
        let meta = root.join("corpus.meta");
        let records = root.join("corpus.records");
        let mut meta_bytes = [0u8; 25];
        meta_bytes[0..8].copy_from_slice(&1u64.to_le_bytes());
        meta_bytes[8..16].copy_from_slice(&1u64.to_le_bytes());
        meta_bytes[16..24].copy_from_slice(&7u64.to_le_bytes());
        meta_bytes[24] = 1;
        let mut record = [0u8; 48];
        record[4..8].copy_from_slice(&next.to_le_bytes());
        record[20..24].copy_from_slice(&34u32.to_le_bytes());
        record[24..28].copy_from_slice(&33u32.to_le_bytes());
        record[28..32].copy_from_slice(&33u32.to_le_bytes());
        record[36..40].copy_from_slice(&1u32.to_le_bytes());
        record[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
        record[44..48].copy_from_slice(&u32::MAX.to_le_bytes());
        std::fs::write(&meta, meta_bytes).expect("complete metadata");
        std::fs::write(&records, record).expect("complete records");
        (meta, records)
    }

    fn write_execution_pair(root: &Path, version: u32) {
        let attention = if version == 1 {
            AttentionOperatorSpec::learned_absolute_v1()
        } else {
            AttentionOperatorSpec::learned_absolute_v2()
        };
        let dense = if version == 1 {
            DenseOperatorSpec::gpt2_v1()
        } else {
            DenseOperatorSpec::gpt2_v2()
        };
        write_operator(&root.join(ATTENTION_OPERATOR_BINDING_FILE), &attention);
        write_operator(&root.join(DENSE_OPERATOR_BINDING_FILE), &dense);
    }

    fn producer_guard(root: &Path) -> RecordedCorpusProducerGuard {
        std::fs::create_dir_all(root).expect("producer root");
        RecordedCorpusProducerGuard::try_acquire(root).expect("producer guard")
    }

    fn publish_compile_binding(
        guard: &RecordedCorpusProducerGuard,
        meta: &Path,
        records: &Path,
    ) -> String {
        guard
            .begin_compile_attempt()
            .expect("begin compile attempt");
        publish_binding(guard, meta, records).expect("publish compile binding")
    }

    #[test]
    fn canonical_binding_v1_bytes_and_cid_are_literal_pinned() {
        let binding = RecordedCorpusBinding {
            schema: "uor-r4-recorded-corpus-binding/1".to_owned(),
            manifest: RecordedCorpusFileBinding {
                name: "observation.manifest.json".to_owned(),
                present: false,
                length: None,
                blake3: None,
            },
            attention_operator: RecordedCorpusFileBinding {
                name: "attention_operator.json".to_owned(),
                present: false,
                length: None,
                blake3: None,
            },
            dense_operator: RecordedCorpusFileBinding {
                name: "dense_operator.json".to_owned(),
                present: false,
                length: None,
                blake3: None,
            },
            metadata: RecordedCorpusFileBinding {
                name: "corpus.meta".to_owned(),
                present: true,
                length: Some(1),
                blake3: Some(
                    "blake3:83d9dab06011479163c7c4d5fa735f911bac86729f56aaa115b7eed2eb66e022"
                        .to_owned(),
                ),
            },
            records: RecordedCorpusFileBinding {
                name: "corpus.records".to_owned(),
                present: true,
                length: Some(1),
                blake3: Some(
                    "blake3:b2dea48d667b2821a9bcf69eded39a2458a1d8165ca7fcac64c3557b69a7ea08"
                        .to_owned(),
                ),
            },
            hidden: RecordedCorpusFileBinding {
                name: "corpus.records.hidden".to_owned(),
                present: false,
                length: None,
                blake3: None,
            },
        };
        let expected = b"{\n  \"schema\": \"uor-r4-recorded-corpus-binding/1\",\n  \"manifest\": {\n    \"name\": \"observation.manifest.json\",\n    \"present\": false\n  },\n  \"attention_operator\": {\n    \"name\": \"attention_operator.json\",\n    \"present\": false\n  },\n  \"dense_operator\": {\n    \"name\": \"dense_operator.json\",\n    \"present\": false\n  },\n  \"metadata\": {\n    \"name\": \"corpus.meta\",\n    \"present\": true,\n    \"length\": 1,\n    \"blake3\": \"blake3:83d9dab06011479163c7c4d5fa735f911bac86729f56aaa115b7eed2eb66e022\"\n  },\n  \"records\": {\n    \"name\": \"corpus.records\",\n    \"present\": true,\n    \"length\": 1,\n    \"blake3\": \"blake3:b2dea48d667b2821a9bcf69eded39a2458a1d8165ca7fcac64c3557b69a7ea08\"\n  },\n  \"hidden\": {\n    \"name\": \"corpus.records.hidden\",\n    \"present\": false\n  }\n}\n";
        assert_eq!(
            binding.canonical_bytes().expect("canonical bytes"),
            expected
        );
        assert_eq!(
            binding.declared_digest().expect("binding CID"),
            "blake3:aea2551ba82356375580a480d338b076488354c4b6bcab59bb22c26af9bcdafc"
        );
    }

    #[test]
    fn producer_guard_is_alias_stable_nonblocking_and_reacquirable() {
        let root = unique_root("producer-guard-busy");
        std::fs::create_dir_all(&root).expect("producer root");
        let first = RecordedCorpusProducerGuard::try_acquire(&root).expect("first guard");
        let before: Vec<_> = std::fs::read_dir(&root)
            .expect("root inventory")
            .map(|entry| entry.expect("entry").file_name())
            .collect();
        let alias = root
            .parent()
            .expect("root parent")
            .join(".")
            .join(root.file_name().expect("root name"));
        let busy = RecordedCorpusProducerGuard::try_acquire(&alias)
            .expect_err("alias contender is nonblocking");
        assert!(busy.reason.contains("BUSY"), "{busy}");
        let after: Vec<_> = std::fs::read_dir(&root)
            .expect("root inventory after BUSY")
            .map(|entry| entry.expect("entry").file_name())
            .collect();
        assert_eq!(after, before, "BUSY refusal cannot mutate corpus root");
        drop(first);
        let retry = RecordedCorpusProducerGuard::try_acquire(&alias).expect("retry after drop");
        assert_eq!(retry.root(), std::fs::canonicalize(&root).unwrap());
        drop(retry);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn producer_guard_case_aliases_contend_when_the_filesystem_equates_them() {
        let root = unique_root("producer-guard-case-alias");
        std::fs::create_dir_all(&root).expect("producer root");
        let alias = root.parent().expect("root parent").join(
            root.file_name()
                .expect("root name")
                .to_string_lossy()
                .to_uppercase(),
        );
        let Ok(alias_metadata) = std::fs::symlink_metadata(&alias) else {
            // Case-sensitive filesystems have no equivalent alias to test.
            let _ = std::fs::remove_dir_all(root);
            return;
        };
        let root_metadata = std::fs::symlink_metadata(&root).expect("root metadata");
        if !opened_file_identity_matches(&root_metadata, &alias_metadata) {
            let _ = std::fs::remove_dir_all(root);
            return;
        }

        let first = RecordedCorpusProducerGuard::try_acquire(&root).expect("first spelling");
        let busy = RecordedCorpusProducerGuard::try_acquire(&alias)
            .expect_err("equivalent final-component spelling must contend");
        assert!(busy.reason.contains("BUSY"), "{busy}");
        assert_eq!(first.root(), std::fs::canonicalize(&root).unwrap());
        drop(first);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn producer_guard_refuses_cross_root_publication_before_mutation() {
        let owned = unique_root("producer-guard-owned");
        let other = unique_root("producer-guard-other");
        let guard = producer_guard(&owned);
        let (meta, records) = complete_source_corpus(&other, 37);
        write_execution_pair(&other, 2);
        let meta_before = std::fs::read(&meta).expect("metadata before");
        let records_before = std::fs::read(&records).expect("records before");
        let error =
            publish_binding(&guard, &meta, &records).expect_err("cross-root guard is terminal");
        assert!(error.reason.contains("cross-root"), "{error}");
        assert_eq!(std::fs::read(&meta).unwrap(), meta_before);
        assert_eq!(std::fs::read(&records).unwrap(), records_before);
        assert!(!other.join(RECORDED_CORPUS_BINDING_FILE).exists());
        let _ = std::fs::remove_dir_all(owned);
        let _ = std::fs::remove_dir_all(other);
    }

    #[cfg(unix)]
    #[test]
    fn producer_guard_rejects_symlink_coordination_without_following() {
        let root = unique_root("producer-guard-symlink");
        std::fs::create_dir_all(&root).expect("producer root");
        let canonical = std::fs::canonicalize(&root).expect("canonical root");
        let lock_path = producer_lock_path(
            canonical.parent().expect("canonical parent"),
            canonical.file_name().expect("canonical root name"),
        );
        std::fs::create_dir_all(lock_path.parent().expect("coordination parent"))
            .expect("coordination directory");
        let sentinel = root.parent().expect("root parent").join(format!(
            "{}.sentinel",
            root.file_name().unwrap().to_string_lossy()
        ));
        std::fs::write(&sentinel, b"sentinel").expect("sentinel");
        std::os::unix::fs::symlink(&sentinel, &lock_path).expect("coordination symlink");
        let error = RecordedCorpusProducerGuard::try_acquire(&root)
            .expect_err("coordination symlink is terminal");
        assert!(error.reason.contains("without following"), "{error}");
        assert_eq!(
            std::fs::read(&sentinel).expect("sentinel preserved"),
            b"sentinel"
        );
        assert!(
            std::fs::symlink_metadata(&lock_path)
                .expect("symlink preserved")
                .file_type()
                .is_symlink()
        );
        std::fs::remove_file(lock_path).expect("remove test coordination symlink");
        let _ = std::fs::remove_file(sentinel);
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn producer_guard_rejects_root_replaced_by_symlink_during_acquisition() {
        let root = unique_root("producer-guard-root-swap");
        let displaced = root.with_extension("displaced");
        let other = unique_root("producer-guard-root-swap-target");
        std::fs::create_dir_all(&root).expect("producer root");
        std::fs::create_dir_all(&other).expect("other root");
        std::fs::write(other.join("sentinel"), b"other").expect("other sentinel");

        let error = RecordedCorpusProducerGuard::try_acquire_with_root_hook(&root, || {
            std::fs::rename(&root, &displaced).expect("displace opened root");
            std::os::unix::fs::symlink(&other, &root).expect("replace root with symlink");
        })
        .expect_err("root replacement is terminal");
        assert!(
            error.reason.contains("changed identity")
                || error.reason.contains("real non-symlink directory"),
            "{error}"
        );
        assert_eq!(std::fs::read(other.join("sentinel")).unwrap(), b"other");
        assert!(
            std::fs::symlink_metadata(&root)
                .expect("replacement preserved")
                .file_type()
                .is_symlink()
        );

        std::fs::remove_file(&root).expect("remove test symlink");
        std::fs::rename(&displaced, &root).expect("restore root");
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(other);
    }

    #[test]
    fn producer_guard_locks_absent_logical_root_before_exact_creation() {
        let root = unique_root("producer-guard-absent");
        assert!(!root.exists());
        let mut first =
            RecordedCorpusProducerGuard::try_acquire(&root).expect("lock absent logical root");
        assert!(!root.exists(), "lock acquisition alone is failure-atomic");
        let busy = RecordedCorpusProducerGuard::try_acquire(&root)
            .expect_err("absent-root contender is nonblocking");
        assert!(busy.reason.contains("BUSY"), "{busy}");
        assert!(!root.exists(), "BUSY cannot create the logical root");
        assert_eq!(
            first.ensure_root().expect("create root after preflight"),
            std::fs::canonicalize(&root).unwrap()
        );
        drop(first);
        let retry = RecordedCorpusProducerGuard::try_acquire(&root).expect("retry existing root");
        assert_eq!(retry.root(), std::fs::canonicalize(&root).unwrap());
        drop(retry);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn dense_generation_requires_and_roundtrips_canonical_binding() {
        let root = unique_root("dense-binding-roundtrip");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 17);
        write_execution_pair(&root, 2);

        let missing = capture(&meta, &records).expect_err("dense without commit is terminal");
        assert!(
            missing.reason.contains(RECORDED_CORPUS_BINDING_FILE),
            "{missing}"
        );
        let cid = publish_compile_binding(&guard, &meta, &records);
        let snapshot = capture(&meta, &records).expect("capture committed generation");
        assert_eq!(snapshot.binding_cid.as_deref(), Some(cid.as_str()));
        assert_eq!(
            snapshot.execution,
            RecordedCorpusExecutionIdentity {
                attention_operator: Some(AttentionOperatorSpec::learned_absolute_v2()),
                dense_operator: Some(DenseOperatorSpec::gpt2_v2()),
            }
        );
        assert_eq!(
            snapshot
                .binding
                .as_ref()
                .expect("typed binding")
                .declared_digest()
                .expect("binding digest"),
            cid
        );

        let bytes = std::fs::read(root.join(RECORDED_CORPUS_BINDING_FILE))
            .expect("canonical binding bytes");
        assert_eq!(
            parse_binding_bytes(&root.join(RECORDED_CORPUS_BINDING_FILE), &bytes)
                .expect("canonical binding")
                .canonical_bytes()
                .expect("canonical bytes"),
            bytes
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn role_preflight_and_publication_reject_cross_role_generations() {
        let compile_root = unique_root("compile-role-binding");
        let compile_guard = producer_guard(&compile_root);
        let (compile_meta, compile_records) = complete_source_corpus(&compile_root, 41);
        publish_compile_binding(&compile_guard, &compile_meta, &compile_records);
        compile_guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Compile)
            .expect("matching compile role");
        let error = compile_guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Observation)
            .expect_err("observation cannot enter a compile root");
        assert!(error.reason.contains("compile-role"), "{error}");
        let compile_binding =
            std::fs::read(compile_root.join(RECORDED_CORPUS_BINDING_FILE)).unwrap();
        let (state, merged) =
            write_corpus_pair(&compile_root, observation::STATE_FILE, "merged.bin");
        let error = publish_binding(&compile_guard, &state, &merged)
            .expect_err("public publisher cannot replace a compile binding with observation role");
        assert!(
            error.reason.contains("cross-role") || error.reason.contains("compile-role"),
            "{error}"
        );
        assert_eq!(
            std::fs::read(compile_root.join(RECORDED_CORPUS_BINDING_FILE)).unwrap(),
            compile_binding
        );

        let observation_root = unique_root("observation-role-binding");
        let observation_guard = producer_guard(&observation_root);
        let (state, merged) =
            write_corpus_pair(&observation_root, observation::STATE_FILE, "merged.bin");
        publish_binding(&observation_guard, &state, &merged).expect("observation-role binding");
        observation_guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Observation)
            .expect("matching observation role");
        let error = observation_guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Compile)
            .expect_err("compile cannot enter an observation root");
        assert!(error.reason.contains("observation-role"), "{error}");
        let observation_binding =
            std::fs::read(observation_root.join(RECORDED_CORPUS_BINDING_FILE)).unwrap();
        let marker = RecordedCorpusCompileAttempt::compile()
            .canonical_bytes()
            .expect("marker bytes");
        std::fs::write(
            observation_root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE),
            marker,
        )
        .expect("inject wrong-role marker");
        let error = capture(&state, &merged)
            .expect_err("an observation binding plus compile marker is not one generation");
        assert!(
            error.reason.contains("requires corpus.meta/corpus.records"),
            "{error}"
        );
        std::fs::remove_file(observation_root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE))
            .expect("remove injected marker");
        let (meta, records) = complete_source_corpus(&observation_root, 43);
        let error = publish_binding(&observation_guard, &meta, &records)
            .expect_err("public publisher cannot replace an observation binding with compile role");
        assert!(
            error.reason.contains("cross-role") || error.reason.contains("observation-role"),
            "{error}"
        );
        assert_eq!(
            std::fs::read(observation_root.join(RECORDED_CORPUS_BINDING_FILE)).unwrap(),
            observation_binding
        );

        let _ = std::fs::remove_dir_all(compile_root);
        let _ = std::fs::remove_dir_all(observation_root);
    }

    #[test]
    fn compile_attempt_marker_bytes_digest_and_lifecycle_are_pinned() {
        let expected = b"{\n  \"schema\": \"uor-r4-recorded-corpus-compile-attempt/1\",\n  \"role\": \"compile\"\n}\n";
        let canonical = RecordedCorpusCompileAttempt::compile()
            .canonical_bytes()
            .expect("canonical marker");
        assert_eq!(canonical, expected);
        assert_eq!(
            format!("blake3:{}", blake3::hash(&canonical).to_hex()),
            "blake3:06c2257b2d374860717cac12285e7ba0cf9a414f88d89596d5380168e1e0ac52"
        );

        let root = unique_root("compile-attempt-lifecycle");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 53);
        guard.begin_compile_attempt().expect("begin marker");
        assert_eq!(
            std::fs::read(root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE)).unwrap(),
            expected
        );
        let error = capture(&meta, &records).expect_err("attempt without commit is terminal");
        assert!(
            error.reason.contains("unfinished compile attempt"),
            "{error}"
        );
        let cid = publish_binding(&guard, &meta, &records).expect("publish under marker");
        assert_eq!(
            capture(&meta, &records)
                .expect("committed generation remains readable before cleanup")
                .binding_cid
                .as_deref(),
            Some(cid.as_str())
        );
        guard.finish_compile_attempt().expect("finish marker");
        assert!(!root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE).exists());
        assert_eq!(
            capture(&meta, &records)
                .expect("finished generation")
                .binding_cid,
            Some(cid)
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_update_requires_marker_for_same_role_generation_transition() {
        let interrupted = unique_root("source-update-interrupted");
        let guard = producer_guard(&interrupted);
        let (meta, records) = complete_source_corpus(&interrupted, 53);
        let old_cid = publish_compile_binding(&guard, &meta, &records);
        guard.finish_compile_attempt().expect("finish generation A");
        guard.begin_compile_attempt().expect("begin generation B");
        let _ = complete_source_corpus(&interrupted, 71);
        let state = preflight_source_update_publication(&guard, &meta, &records)
            .expect("active marker permits canonical last-good binding A beside resumed members B");
        assert!(state.attempt_active);
        assert!(!state.recoverable_binding);
        let new_cid = publish_binding(&guard, &meta, &records).expect("commit generation B");
        assert_ne!(new_cid, old_cid);
        guard.finish_compile_attempt().expect("finish generation B");
        assert_eq!(capture(&meta, &records).unwrap().binding_cid, Some(new_cid));

        let unowned = unique_root("source-update-without-marker");
        let guard = producer_guard(&unowned);
        let (meta, records) = complete_source_corpus(&unowned, 53);
        publish_compile_binding(&guard, &meta, &records);
        guard.finish_compile_attempt().expect("finish generation A");
        let _ = complete_source_corpus(&unowned, 71);
        let before_binding = std::fs::read(unowned.join(RECORDED_CORPUS_BINDING_FILE)).unwrap();
        let before_meta = std::fs::read(&meta).unwrap();
        let before_records = std::fs::read(&records).unwrap();
        let error = preflight_source_update_publication(&guard, &meta, &records)
            .expect_err("member drift without an active attempt marker is terminal");
        assert!(error.reason.contains("does not commit"), "{error}");
        assert_eq!(
            std::fs::read(unowned.join(RECORDED_CORPUS_BINDING_FILE)).unwrap(),
            before_binding
        );
        assert_eq!(std::fs::read(&meta).unwrap(), before_meta);
        assert_eq!(std::fs::read(&records).unwrap(), before_records);

        let _ = std::fs::remove_dir_all(interrupted);
        let _ = std::fs::remove_dir_all(unowned);
    }

    #[test]
    fn compile_attempt_crash_residue_is_recovered_but_observation_refuses_it() {
        let root = unique_root("compile-attempt-writing");
        let guard = producer_guard(&root);
        let writing = root.join(format!(
            ".{RECORDED_CORPUS_COMPILE_ATTEMPT_FILE}.999.1.writing"
        ));
        std::fs::write(&writing, b"partial").expect("partial marker staging");
        let error = guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Observation)
            .expect_err("observation rejects compile staging evidence");
        assert!(error.reason.contains("compile-attempt"), "{error}");
        assert_eq!(std::fs::read(&writing).unwrap(), b"partial");
        guard
            .begin_compile_attempt()
            .expect("compile retry reclaims staging");
        assert!(!writing.exists());
        assert!(root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE).exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn role_preflight_rejects_foreign_prefix_and_temporary_without_mutation() {
        for foreign in [
            "shard-00.bin",
            "shard-not-a-number.bin",
            observation::RAW_COMMITTED_FILE,
            "tokenizer_adapter.json",
            ".source_compile_preflight.json.800.1.tmp",
            ".compiled_bundle_completion.json.801.2.tmp",
        ] {
            let root = unique_root(&format!("foreign-role-{}", foreign.replace('.', "-")));
            let guard = producer_guard(&root);
            let path = root.join(foreign);
            std::fs::write(&path, b"foreign crash prefix").expect("foreign prefix");
            let before = std::fs::read(&path).unwrap();
            let role =
                if foreign.starts_with("shard-") || foreign == observation::RAW_COMMITTED_FILE {
                    RecordedCorpusRole::Compile
                } else {
                    RecordedCorpusRole::Observation
                };
            let error = guard
                .preflight_publication_namespace_for(role)
                .expect_err("foreign role prefix is terminal");
            assert!(error.reason.contains("foreign"), "{error}");
            assert_eq!(std::fs::read(&path).unwrap(), before);
            let _ = std::fs::remove_dir_all(root);
        }

        let root = unique_root("cross-role-binding-temp");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 47);
        publish_compile_binding(&guard, &meta, &records);
        let stable = root.join(RECORDED_CORPUS_BINDING_FILE);
        let temporary = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.777.1.tmp"));
        std::fs::rename(&stable, &temporary).expect("binding crash temp");
        let before = std::fs::read(&temporary).unwrap();
        let error = guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Observation)
            .expect_err("cross-role canonical temp is terminal");
        assert!(error.reason.contains("cross-role") || error.reason.contains("compile-role"));
        assert_eq!(std::fs::read(&temporary).unwrap(), before);
        assert!(!stable.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn shared_planned_output_scope_is_global_and_recovery_is_owner_exact() {
        let root = unique_root("planned-output-scope");
        let guard = producer_guard(&root);
        let records_residue = root.join(format!(
            "{PLANNED_OUTPUT_RESERVED_PREFIX}{}--999.1.writing",
            PlannedOutputMember::Records.stable_name()
        ));
        std::fs::write(&records_residue, b"partial records").expect("records residue");
        guard
            .preflight_planned_output_scope(&[PlannedOutputMember::Records])
            .expect("declared owner accepts its regular residue");
        assert!(
            guard
                .has_planned_output_residue(PlannedOutputMember::Records)
                .unwrap()
        );
        let error = guard
            .preflight_planned_output_scope(&[])
            .expect_err("partial-plan writer owns no residue");
        assert!(error.reason.contains("foreign member"), "{error}");
        assert_eq!(std::fs::read(&records_residue).unwrap(), b"partial records");
        guard
            .reclaim_planned_output_residues(PlannedOutputMember::Records)
            .expect("owner reclaims exact regular residue");
        assert!(!records_residue.exists());

        let artifact_residue = root.join(format!(
            "{PLANNED_OUTPUT_RESERVED_PREFIX}{}--999.2.writing",
            PlannedOutputMember::Artifact.stable_name()
        ));
        std::fs::write(&artifact_residue, b"foreign artifact").expect("artifact residue");
        let error = guard
            .preflight_planned_output_scope(&[
                PlannedOutputMember::Records,
                PlannedOutputMember::Hidden,
                PlannedOutputMember::Metadata,
            ])
            .expect_err("subsample cannot ignore another plan's residue");
        assert!(error.reason.contains("tless_artifacts.bin"), "{error}");
        assert_eq!(
            std::fs::read(&artifact_residue).unwrap(),
            b"foreign artifact"
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn shared_planned_output_scope_rejects_symlink_and_nonregular_entries() {
        let root = unique_root("planned-output-special");
        let guard = producer_guard(&root);
        let symlink = root.join(format!(
            "{PLANNED_OUTPUT_RESERVED_PREFIX}{}--999.3.writing",
            PlannedOutputMember::Hidden.stable_name()
        ));
        std::os::unix::fs::symlink("missing-target", &symlink).expect("reserved symlink");
        let error = guard
            .preflight_planned_output_scope(&PlannedOutputMember::ALL)
            .expect_err("reserved symlink is terminal");
        assert!(error.reason.contains("without following links"), "{error}");
        assert!(
            std::fs::symlink_metadata(&symlink)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        std::fs::remove_file(&symlink).unwrap();

        let directory = root.join(format!(
            "{PLANNED_OUTPUT_RESERVED_PREFIX}{}--999.4.writing",
            PlannedOutputMember::Metadata.stable_name()
        ));
        std::fs::create_dir(&directory).expect("reserved directory");
        let error = guard
            .preflight_planned_output_scope(&PlannedOutputMember::ALL)
            .expect_err("reserved directory is terminal");
        assert!(
            error.reason.contains("without following links") || error.reason.contains("regular")
        );
        assert!(
            std::fs::symlink_metadata(&directory)
                .unwrap()
                .file_type()
                .is_dir()
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn binding_rejects_stronger_pair_corpus_aba_even_when_pair_and_marker_restore() {
        let root = unique_root("binding-aba");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 11);
        write_execution_pair(&root, 1);
        publish_compile_binding(&guard, &meta, &records);
        let attention_a =
            std::fs::read(root.join(ATTENTION_OPERATOR_BINDING_FILE)).expect("attention A");
        let dense_a = std::fs::read(root.join(DENSE_OPERATOR_BINDING_FILE)).expect("dense A");
        let binding_a = std::fs::read(root.join(RECORDED_CORPUS_BINDING_FILE)).expect("binding A");

        let error = capture_with_hooks(
            &meta,
            &records,
            || {
                write_execution_pair(&root, 2);
                let _ = complete_source_corpus(&root, 29);
                publish_binding(&guard, &meta, &records).expect("publish B");
            },
            || {
                std::fs::write(root.join(ATTENTION_OPERATOR_BINDING_FILE), &attention_a)
                    .expect("restore attention A");
                std::fs::write(root.join(DENSE_OPERATOR_BINDING_FILE), &dense_a)
                    .expect("restore dense A");
                std::fs::write(root.join(RECORDED_CORPUS_BINDING_FILE), &binding_a)
                    .expect("restore binding A");
            },
        )
        .expect_err("binding A cannot authorize corpus B");
        assert!(
            error.reason.contains("does not match")
                && (error.reason.contains("metadata") || error.reason.contains("records")),
            "unexpected error: {error}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn markerless_attention_only_remains_explicit_compatibility() {
        let root = unique_root("markerless-attention-only");
        let (meta, records) = complete_source_corpus(&root, 5);
        write_operator(
            &root.join(ATTENTION_OPERATOR_BINDING_FILE),
            &AttentionOperatorSpec::learned_absolute_v2(),
        );
        let snapshot = capture(&meta, &records).expect("attention-only compatibility");
        assert!(snapshot.binding.is_none());
        assert!(snapshot.binding_cid.is_none());
        assert_eq!(snapshot.execution.dense_operator, None);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn binding_temporary_recovery_is_exact_and_conflicts_are_preserved() {
        let root = unique_root("binding-temp-recovery");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 7);
        write_execution_pair(&root, 2);
        publish_compile_binding(&guard, &meta, &records);
        let stable = root.join(RECORDED_CORPUS_BINDING_FILE);
        let expected = std::fs::read(&stable).expect("stable binding");
        let recoverable = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.999.1.tmp"));
        std::fs::rename(&stable, &recoverable).expect("simulate pre-publish crash");
        let recovered_cid =
            publish_binding(&guard, &meta, &records).expect("recover exact temporary");
        assert_eq!(
            recovered_cid,
            format!("blake3:{}", blake3::hash(&expected).to_hex())
        );
        assert!(!recoverable.exists());

        let conflicting = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.999.2.tmp"));
        let mut conflict = expected.clone();
        let schema = RECORDED_CORPUS_BINDING_SCHEMA.as_bytes();
        let offset = conflict
            .windows(schema.len())
            .position(|window| window == schema)
            .expect("schema bytes");
        conflict[offset] = b'X';
        std::fs::write(&conflicting, &conflict).expect("conflicting temporary");
        let before = std::fs::read(&conflicting).expect("conflict before");
        let error =
            publish_binding(&guard, &meta, &records).expect_err("malformed temp is terminal");
        assert!(error.reason.contains("binding"), "{error}");
        assert_eq!(
            std::fs::read(&conflicting).expect("conflict preserved"),
            before
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn binding_writing_crash_residue_is_ignored_and_reclaimed_by_exact_retry() {
        let root = unique_root("binding-writing-recovery");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 13);
        write_execution_pair(&root, 2);
        let zero = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.999.10.writing"));
        let partial = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.999.11.writing"));
        std::fs::write(&zero, b"").expect("zero-byte staging residue");
        std::fs::write(&partial, b"partial canonical JSON").expect("partial staging residue");

        publish_compile_binding(&guard, &meta, &records);
        assert!(!zero.exists());
        assert!(!partial.exists());

        let ignored = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.999.12.writing"));
        std::fs::write(&ignored, b"unfinished").expect("reader-ignored staging residue");
        capture(&meta, &records).expect("reader ignores non-authoritative staging residue");
        assert_eq!(
            std::fs::read(&ignored).expect("reader preserves staging residue"),
            b"unfinished"
        );
        publish_binding(&guard, &meta, &records).expect("publisher reclaims ignored residue");
        assert!(!ignored.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn unrecognized_binding_publication_residue_is_terminal_and_preserved() {
        let root = unique_root("binding-unrecognized-residue");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 19);
        write_execution_pair(&root, 2);
        publish_compile_binding(&guard, &meta, &records);
        let unrecognized = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.unowned.writing"));
        std::fs::write(&unrecognized, b"unowned").expect("unrecognized residue");
        let before = std::fs::read(&unrecognized).expect("residue before");
        let error =
            publish_binding(&guard, &meta, &records).expect_err("unrecognized residue is terminal");
        assert!(error.reason.contains("reserved"), "{error}");
        assert_eq!(
            std::fs::read(&unrecognized).expect("residue preserved"),
            before
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn binding_namespace_ignores_unrelated_non_utf8_but_rejects_reserved_non_utf8() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let root = unique_root("binding-non-utf8");
        let (meta, records) = complete_source_corpus(&root, 23);
        write_operator(
            &root.join(ATTENTION_OPERATOR_BINDING_FILE),
            &AttentionOperatorSpec::learned_absolute_v2(),
        );
        let unrelated = root.join(OsString::from_vec(vec![b'u', 0xff, b'x']));
        std::fs::write(&unrelated, b"unrelated").expect("unrelated non-UTF-8 entry");
        capture(&meta, &records).expect("unrelated non-UTF-8 entry is outside namespace");

        let mut reserved = format!(".{RECORDED_CORPUS_BINDING_FILE}.").into_bytes();
        reserved.extend_from_slice(&[0xff, b'.', b't', b'm', b'p']);
        let reserved = root.join(OsString::from_vec(reserved));
        std::fs::write(&reserved, b"reserved").expect("reserved non-UTF-8 entry");
        let error = capture(&meta, &records).expect_err("reserved non-UTF-8 entry is terminal");
        assert!(error.reason.contains("non-UTF-8"), "{error}");
        assert_eq!(
            std::fs::read(&reserved).expect("reserved entry preserved"),
            b"reserved"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn publisher_rejects_symlink_binding_writing_residue_without_mutation() {
        let root = unique_root("binding-writing-symlink");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 31);
        write_execution_pair(&root, 2);
        publish_compile_binding(&guard, &meta, &records);
        let writing = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.999.20.writing"));
        std::os::unix::fs::symlink("missing-target", &writing).expect("staging symlink");
        capture(&meta, &records).expect("reader ignores non-authoritative staging symlink");
        let error = publish_binding(&guard, &meta, &records)
            .expect_err("publisher rejects staging symlink");
        assert!(
            error.reason.contains("not a regular file")
                || error.reason.contains("without following links"),
            "{error}"
        );
        assert!(
            std::fs::symlink_metadata(&writing)
                .expect("symlink preserved")
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            std::fs::read_link(&writing).expect("symlink target preserved"),
            PathBuf::from("missing-target")
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn lower_hidden_path_changes_only_the_filename() {
        let records = Path::new("/tmp/parent_recs.bin/corpus_recs.bin");
        assert_eq!(
            hidden_path(records),
            PathBuf::from("/tmp/parent_recs.bin/corpus_hidden.bin")
        );
    }

    #[test]
    fn composite_capture_rejects_execution_swap_between_provenance_and_corpus() {
        let root = unique_root("inter-phase-swap");
        std::fs::create_dir_all(&root).expect("root");
        let meta = root.join("corpus.meta");
        let records = root.join("corpus.records");
        std::fs::write(&meta, b"corpus A metadata").expect("metadata A");
        std::fs::write(&records, b"corpus A records").expect("records A");
        let attention = root.join(ATTENTION_OPERATOR_BINDING_FILE);
        let dense = root.join(DENSE_OPERATOR_BINDING_FILE);
        write_operator(&attention, &AttentionOperatorSpec::learned_absolute_v1());
        write_operator(&dense, &DenseOperatorSpec::gpt2_v1());

        let error = capture_with_hooks(
            &meta,
            &records,
            || {
                write_operator(&attention, &AttentionOperatorSpec::learned_absolute_v2());
                write_operator(&dense, &DenseOperatorSpec::gpt2_v2());
                std::fs::write(&meta, b"corpus B metadata").expect("metadata B");
                std::fs::write(&records, b"corpus B records").expect("records B");
            },
            || {},
        )
        .expect_err("provenance A cannot label corpus B");
        assert!(
            error.reason.contains("changed after capture"),
            "unexpected error: {error}"
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn exact_pair_reconciliation_treats_absence_and_version_as_identity() {
        let recorded = RecordedCorpusExecutionIdentity {
            attention_operator: Some(AttentionOperatorSpec::learned_absolute_v2()),
            dense_operator: Some(DenseOperatorSpec::gpt2_v2()),
        };
        assert!(
            require_execution_identity(
                &recorded,
                Some(&AttentionOperatorSpec::learned_absolute_v2()),
                Some(&DenseOperatorSpec::gpt2_v2()),
                "test compile",
            )
            .is_ok()
        );
        let absent_dense = require_execution_identity(
            &recorded,
            Some(&AttentionOperatorSpec::learned_absolute_v2()),
            None,
            "test compile",
        )
        .expect_err("dense absence cannot relabel v2");
        assert!(absent_dense.reason.contains("dense execution identity"));
        let wrong_era = require_execution_identity(
            &recorded,
            Some(&AttentionOperatorSpec::learned_absolute_v1()),
            Some(&DenseOperatorSpec::gpt2_v1()),
            "test compile",
        )
        .expect_err("v1 cannot relabel v2");
        assert!(wrong_era.reason.contains("attention execution identity"));
    }

    #[test]
    fn recursive_duplicate_keys_are_terminal_in_both_sidecars_and_manifest() {
        let attention_v2 = AttentionOperatorSpec::learned_absolute_v2();
        let dense_v2 = DenseOperatorSpec::gpt2_v2();

        let attention_root = unique_root("duplicate-attention-sidecar");
        let (meta, records) = write_corpus_pair(&attention_root, "corpus.meta", "corpus.records");
        let attention_json = serde_json::to_string(&attention_v2).expect("attention JSON");
        let duplicate_attention =
            attention_json.replacen("\"version\":2", "\"version\":2,\"version\":1", 1);
        assert_ne!(duplicate_attention, attention_json);
        std::fs::write(
            attention_root.join(ATTENTION_OPERATOR_BINDING_FILE),
            duplicate_attention,
        )
        .expect("duplicate attention sidecar");
        write_operator(&attention_root.join(DENSE_OPERATOR_BINDING_FILE), &dense_v2);
        let error = capture(&meta, &records).expect_err("duplicate attention key is terminal");
        assert!(error.reason.contains("duplicate JSON field"), "{error}");

        let dense_root = unique_root("duplicate-dense-sidecar");
        let (meta, records) = write_corpus_pair(&dense_root, "corpus.meta", "corpus.records");
        write_operator(
            &dense_root.join(ATTENTION_OPERATOR_BINDING_FILE),
            &attention_v2,
        );
        let dense_json = serde_json::to_string(&dense_v2).expect("dense JSON");
        let duplicate_dense =
            dense_json.replacen("\"version\":2", "\"version\":2,\"version\":1", 1);
        assert_ne!(duplicate_dense, dense_json);
        std::fs::write(
            dense_root.join(DENSE_OPERATOR_BINDING_FILE),
            duplicate_dense,
        )
        .expect("duplicate dense sidecar");
        let error = capture(&meta, &records).expect_err("duplicate dense key is terminal");
        assert!(error.reason.contains("duplicate JSON field"), "{error}");

        let nested_manifest_root = unique_root("duplicate-nested-manifest");
        let (state, merged) =
            write_corpus_pair(&nested_manifest_root, observation::STATE_FILE, "merged.bin");
        let mut manifest = ObservationManifest::new(1);
        manifest.attention_operator = Some(attention_v2.clone());
        manifest.dense_operator = Some(dense_v2.clone());
        let manifest_json = serde_json::to_string(&manifest).expect("manifest JSON");
        let duplicate_nested =
            manifest_json.replacen("\"version\":2", "\"version\":2,\"version\":1", 1);
        assert_ne!(duplicate_nested, manifest_json);
        std::fs::write(
            nested_manifest_root.join(observation::MANIFEST_FILE),
            duplicate_nested,
        )
        .expect("nested duplicate manifest");
        let error = capture(&state, &merged).expect_err("nested duplicate key is terminal");
        assert!(error.reason.contains("duplicate JSON field"), "{error}");

        let top_manifest_root = unique_root("duplicate-top-manifest");
        let (state, merged) =
            write_corpus_pair(&top_manifest_root, observation::STATE_FILE, "merged.bin");
        let mut duplicate_top = manifest_json
            .strip_suffix('}')
            .expect("manifest object")
            .to_owned();
        duplicate_top.push_str(&format!(",\"dense_operator\":{dense_json}}}"));
        std::fs::write(
            top_manifest_root.join(observation::MANIFEST_FILE),
            duplicate_top,
        )
        .expect("top-level duplicate manifest");
        let error = capture(&state, &merged).expect_err("top-level duplicate key is terminal");
        assert!(error.reason.contains("duplicate JSON field"), "{error}");

        for root in [
            attention_root,
            dense_root,
            nested_manifest_root,
            top_manifest_root,
        ] {
            let _ = std::fs::remove_dir_all(root);
        }
    }
}
