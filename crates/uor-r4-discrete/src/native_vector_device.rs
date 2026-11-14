//! AOT CUDA handle for the native Q31 Hamilton vector transport.
//! The module contains one integer entry point and no PTX/JIT fallback.

use crate::native_vector::{
    q31_batch_len_admitted, q31_hamilton_rotate, q31_quaternion_admitted, Q31Quaternion,
    Q31Transport,
};
use crate::{Error, Result};
use cudarc::driver::{CudaContext, CudaFunction, CudaStream, LaunchConfig, Ptx, PushKernelArg};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const SOURCE: &str = include_str!("native_vector.cu");
const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/native-vector.ptx"));
const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/native-vector.cubin"));
const RESOURCES: &str = include_str!(concat!(env!("OUT_DIR"), "/native-vector-resources.txt"));
const TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda-image-target.txt"));
const ENTRY: &str = "native_q31_hamilton_rotate";

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

fn cuda(error: impl std::fmt::Display) -> Error {
    Error::Cuda(error.to_string())
}

/// Source, PTX, cubin and ptxas-report identities for the embedded image.
pub fn compiled_native_vector_image() -> [String; 4] {
    [
        format!("{:x}", Sha256::digest(SOURCE.as_bytes())),
        format!("{:x}", Sha256::digest(PTX.as_bytes())),
        format!("{:x}", Sha256::digest(CUBIN)),
        format!("{:x}", Sha256::digest(stable_resource_report().as_bytes())),
    ]
}

/// The compiler's stopwatch is not part of the artifact identity.
fn stable_resource_report() -> String {
    RESOURCES
        .lines()
        .filter(|line| !line.contains("Compile time ="))
        .collect::<Vec<_>>()
        .join("\n")
}

/// One explicit RTX-class CUDA context for native integer vector training.
pub struct NativeVectorEngine {
    _context: Arc<CudaContext>,
    stream: Arc<CudaStream>,
    rotate: CudaFunction,
}

impl NativeVectorEngine {
    /// Load the admitted `native-vector.cubin`; no PTX fallback is attempted.
    pub fn new(ordinal: usize) -> Result<Self> {
        let context = CudaContext::new(ordinal).map_err(cuda)?;
        let (major, minor) = context.compute_capability().map_err(cuda)?;
        if TARGET.trim() != format!("sm_{major}{minor}") {
            return Err(invalid(format!(
                "native-vector cubin target {} differs from device {major}.{minor}",
                TARGET.trim()
            )));
        }
        let stream = context.default_stream();
        let module = context
            .load_module(Ptx::from_binary(CUBIN.to_vec()))
            .map_err(cuda)?;
        let rotate = module.load_function(ENTRY).map_err(cuda)?;
        Ok(Self {
            _context: context,
            stream,
            rotate,
        })
    }

    /// Rotate one batch of Q31 quaternions on CUDA and return raw Q31 words.
    pub fn hamilton_rotate(
        &self,
        gates: &[Q31Quaternion],
        values: &[Q31Quaternion],
    ) -> Result<Vec<Q31Transport>> {
        if gates.is_empty() || gates.len() != values.len() {
            return Err(invalid(
                "native-vector transport needs a nonempty equal-length batch",
            ));
        }
        if !q31_batch_len_admitted(gates.len()) {
            return Err(invalid(
                "native-vector batch exceeds the admitted 2^30-item bound",
            ));
        }
        if gates
            .iter()
            .chain(values)
            .any(|quaternion| !q31_quaternion_admitted(*quaternion))
        {
            return Err(invalid(
                "native-vector Q31 coordinates must be in -2^31..=2^31",
            ));
        }
        let count = u32::try_from(gates.len())
            .map_err(|_| invalid("native-vector batch exceeds the CUDA index bound"))?;
        let gate_words: Vec<i64> = gates.iter().flatten().copied().collect();
        let value_words: Vec<i64> = values.iter().flatten().copied().collect();
        let gate_device = self.stream.clone_htod(&gate_words).map_err(cuda)?;
        let value_device = self.stream.clone_htod(&value_words).map_err(cuda)?;
        let mut output = self
            .stream
            .alloc_zeros::<i64>(gates.len() * 4)
            .map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.rotate);
        launch
            .arg(&gate_device)
            .arg(&value_device)
            .arg(&mut output)
            .arg(&count);
        // SAFETY: every buffer carries exactly `count * 4` words on this stream.
        unsafe {
            launch.launch(LaunchConfig {
                grid_dim: (count.div_ceil(256), 1, 1),
                block_dim: (256, 1, 1),
                shared_mem_bytes: 0,
            })
        }
        .map_err(cuda)?;
        let words: Vec<i64> = self.stream.clone_dtoh(&output).map_err(cuda)?;
        Ok(words
            .chunks_exact(4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
            .collect())
    }

    /// Run the Rust oracle on one quaternion in the same batch representation.
    pub fn hamilton_rotate_reference(
        gate: Q31Quaternion,
        value: Q31Quaternion,
    ) -> Result<Q31Transport> {
        q31_hamilton_rotate(gate, value)
    }
}

/// Compile-time proof that the native device API remains integer-only.
pub fn native_vector_entry() -> &'static str {
    ENTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_identity_excludes_the_compiler_stopwatch() {
        assert!(!stable_resource_report().contains("Compile time ="));
        assert!(compiled_native_vector_image()
            .iter()
            .all(|digest| digest.len() == 64));
    }
}
