//! Native R4G1 graph-runtime adapter used by the HTTP server.
//!
//! The graph scorer is intentionally kept separate from the exploratory f64
//! router. It derives the packed input signature with the transformerless
//! artifact, then selects a token from the validated R4G1 graph.

use std::path::{Path, PathBuf};

use uor_r4_core::transformerless::compiler::{self, Compiled, WINDOW};
use uor_r4_core::transformerless::runtime;
use uor_r4_core::transformerless::scenarios::Tokenizer;
use uor_r4_core::transformerless::score::DEFAULT_EXCT_TOP_X;
use uor_r4_core::transformerless::score::DEFAULT_ROOT_TOP_B;
use uor_r4_core::transformerless::score_runtime::GraphScorer;

/// A loaded, CID-verified scored graph and the teacher artifact needed to
/// derive input signatures from token ids.
pub struct R4g1State {
    artifacts: Compiled,
    scorer: GraphScorer,
    rotations: [usize; WINDOW + 1],
    tokenizer: Option<Tokenizer>,
}

impl R4g1State {
    /// Load and validate a scored graph. The teacher artifact supplies the
    /// compressed token rows used to derive input signatures. EXCT is not
    /// enabled because its reference implementation performs probe-time
    /// floating-point quantization.
    pub fn load(graph_path: &Path, teacher_path: &Path) -> Result<Self, String> {
        let graph_bytes = std::fs::read(graph_path)
            .map_err(|error| format!("{}: {error}", graph_path.display()))?;
        let teacher_bytes = std::fs::read(teacher_path)
            .map_err(|error| format!("{}: {error}", teacher_path.display()))?;
        let artifacts = compiler::parse_artifacts(&teacher_bytes).ok_or_else(|| {
            format!(
                "{}: not a TLA3/TLA4/TLA5 teacher artifact",
                teacher_path.display()
            )
        })?;
        // EXCT is a compiler-era carryover that requires probe-time
        // log-quantization in the reference scorer. The deployed R4G1 path
        // uses the packed HEAD/NODE/EDGE/ROUT/EMIT sections only, preserving
        // the integer-only runtime contract. The teacher bytes are still
        // needed below to decode token rows and derive input signatures.
        let scorer =
            GraphScorer::from_artifact(&graph_bytes, None, DEFAULT_ROOT_TOP_B, DEFAULT_EXCT_TOP_X)
                .map_err(|error| format!("{}: {error}", graph_path.display()))?;
        let tokenizer = teacher_path
            .parent()
            .map(|parent| parent.join("tokenizer.bin"))
            .filter(|path| path.is_file())
            .and_then(|path| Tokenizer::try_load(path).ok());

        Ok(Self {
            artifacts,
            scorer,
            rotations: compiler::derive_rotations(),
            tokenizer,
        })
    }

    /// Encode with the bundle-matched tokenizer when one is available.
    pub fn encode_into(&self, text: &str, out: &mut [u32]) -> Option<usize> {
        self.tokenizer.as_ref()?.encode_into(text, out).ok()
    }

    /// Decode with the bundle-matched tokenizer when one is available.
    pub fn decode_into(&self, tokens: &[u32], out: &mut [u8]) -> Option<usize> {
        self.tokenizer.as_ref()?.decode_into(tokens, out).ok()
    }

    /// Score one token window using the validated graph artifact.
    pub fn predict_window(&self, window: &[u32]) -> Result<u32, String> {
        let bundle = runtime::bundle_window_plain(&self.artifacts, &self.rotations, window);
        let signature = runtime::sig_plain(&self.artifacts, &bundle);
        self.scorer
            .score_candidates(&signature)
            .map(|outcome| outcome.selected)
    }

    /// Generate a greedy continuation from a token seed. This mirrors the
    /// legacy runtime's fixed-width window behavior while replacing its
    /// graded-store lookup with R4G1 graph scoring.
    pub fn generate_into(&self, seed: &[u32], out: &mut [u32]) -> Result<usize, String> {
        let mut window = [0u32; WINDOW];
        let seed = &seed[seed.len().saturating_sub(WINDOW)..];
        let mut window_len = seed.len();
        window[..window_len].copy_from_slice(seed);

        for (generated, token) in out.iter_mut().enumerate() {
            let next = self.predict_window(&window[..window_len])?;
            *token = next;
            if next == 1 || next == 2 {
                return Ok(generated);
            }
            if window_len < WINDOW {
                window[window_len] = next;
                window_len += 1;
            } else {
                window.copy_within(1.., 0);
                window[WINDOW - 1] = next;
            }
        }
        Ok(out.len())
    }
}

/// Resolve the graph path from an explicit setting or the conventional
/// compiled-bundle location beside `tless_artifacts.bin`.
pub fn discover_path(explicit: Option<&str>, teacher_path: &Path) -> Option<PathBuf> {
    explicit.map(PathBuf::from).or_else(|| {
        teacher_path
            .parent()
            .map(|parent| parent.join("graph/score.r4g1"))
    })
}
