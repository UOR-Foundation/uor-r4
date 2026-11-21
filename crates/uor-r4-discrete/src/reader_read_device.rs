//! Wick AOT kernel for the saved question reader's read (`reader_read.cu`):
//! one thread a prompt replies from the flat tables of
//! [`crate::conjunction::flat`] and writes the raw words of every step.
use crate::conjunction::flat::{Flat, FORM_WORDS, KEY_WORDS, PAIR_READ_WORDS, ROW_WORDS};
use crate::{Error, Result};
use cudarc::driver::{CudaContext, LaunchConfig, Ptx, PushKernelArg};
use sha2::{Digest, Sha256};

const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/reader-read.cubin"));
const TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda-image-target.txt"));

/// Words a step writes: `STEP_WORDS` of the host with the form's id replaced
/// by the form's fourteen words.
pub const STEP_WORDS: usize = 36;
/// Longest reply the kernel holds a history for.
pub const MAX_LIMIT: usize = 128;
/// Every place was read from a held row.
pub const STATUS_READ: u32 = 0;
/// A place no held row answers, in a reader whose ladder has a suffix order
/// above the pair: the host reads its suffix ladder there.
pub const STATUS_LADDER: u32 = 1;
/// The asked turn has no kept form.
pub const STATUS_NO_FORM: u32 = 2;

/// One prompt's read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Read {
    pub reply: Vec<u32>,
    /// [`STEP_WORDS`] a reply id.
    pub words: Vec<u32>,
    pub status: u32,
    /// The reply was read again from the rows for an empty read.
    pub empty: bool,
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}
fn cuda(error: impl std::fmt::Display) -> Error {
    Error::Cuda(error.to_string())
}

pub fn kernel_hash() -> String {
    format!("{:x}", Sha256::digest(CUBIN))
}

/// Replies to every prompt on device 0, at most `limit` ids each, and the
/// microseconds from the launch to the kernel's end (no context, table or
/// prompt load, no readback).
pub fn read(flat: &Flat, prompts: &[&[u32]], limit: usize) -> Result<(Vec<Read>, u64)> {
    if limit == 0 || limit > MAX_LIMIT {
        return Err(invalid("the reply limit must be 1..=128"));
    }
    if prompts.is_empty() {
        return Ok((Vec::new(), 0));
    }
    if prompts
        .iter()
        .any(|prompt| prompt.iter().any(|&id| id >= flat.vocabulary))
    {
        return Err(invalid("a prompt id is past the vocabulary"));
    }
    if flat.forms.len() != flat.form_count() * FORM_WORDS
        || flat.row_keys.len() != flat.row_count() * KEY_WORDS
        || flat.row_slots.len() != flat.row_count() * ROW_WORDS
        || (!flat.codes.is_empty() && flat.codes.len() != flat.vocabulary as usize)
        || (flat.trunk && flat.codes.is_empty())
        || flat.pair_reads.len() != flat.pair_keys.len() * PAIR_READ_WORDS
    {
        return Err(invalid("the flat tables are not of their stated shape"));
    }
    let longest = prompts.iter().map(|prompt| prompt.len()).max().unwrap_or(0);
    let stride = longest + limit;
    let count = prompts.len();
    let small = |value: usize| u32::try_from(value).map_err(|_| invalid("a size is past u32"));
    small(count * stride)?;
    small(count * limit * STEP_WORDS)?;
    let params = [
        flat.vocabulary,
        flat.context,
        flat.bos,
        flat.eos,
        small(flat.form_count())?,
        small(flat.row_count())?,
        small(flat.codes.len())?,
        small(limit)?,
        small(stride)?,
        small(count)?,
        u32::from(flat.trunk),
        small(flat.pair_keys.len())?,
        u32::from(flat.ladder),
    ];
    let mut work = vec![0u32; count * stride];
    let mut lengths = Vec::with_capacity(count);
    for (row, prompt) in work.chunks_exact_mut(stride).zip(prompts) {
        row[..prompt.len()].copy_from_slice(prompt);
        lengths.push(prompt.len() as u32);
    }

    let ctx = CudaContext::new(0).map_err(cuda)?;
    let (major, minor) = ctx.compute_capability().map_err(cuda)?;
    if TARGET.trim() != format!("sm_{major}{minor}") {
        return Err(invalid("reader-read cubin and device differ"));
    }
    let stream = ctx.default_stream();
    let module = ctx
        .load_module(Ptx::from_binary(CUBIN.to_vec()))
        .map_err(cuda)?;
    let kernel = module.load_function("reader_read").map_err(cuda)?;
    // An empty table is sent as one unread word.
    let put = |words: &[u32]| {
        stream
            .clone_htod(if words.is_empty() { &[0u32][..] } else { words })
            .map_err(cuda)
    };
    let params_gpu = put(&params)?;
    let forms_gpu = put(&flat.forms)?;
    let keys_gpu = put(&flat.row_keys)?;
    let slots_gpu = put(&flat.row_slots)?;
    let codes_gpu = put(&flat.codes)?;
    let pair_keys_gpu = put(&flat.pair_keys)?;
    let pair_reads_gpu = put(&flat.pair_reads)?;
    let lengths_gpu = put(&lengths)?;
    let mut work_gpu = stream.clone_htod(&work).map_err(cuda)?;
    let mut words_gpu = stream
        .alloc_zeros::<u32>(count * limit * STEP_WORDS)
        .map_err(cuda)?;
    let mut meta_gpu = stream.alloc_zeros::<u32>(count * 4).map_err(cuda)?;
    let threads = params[9];
    ctx.synchronize().map_err(cuda)?;
    let started = std::time::Instant::now();
    // SAFETY: every prompt id is below the vocabulary, the tables have the
    // shapes params states, each thread writes only its own work row
    // (stride ids, prompt + limit within it), its own limit * 36 words and
    // its own four meta words; the kernel checks limit and stride itself.
    unsafe {
        stream
            .launch_builder(&kernel)
            .arg(&params_gpu)
            .arg(&forms_gpu)
            .arg(&keys_gpu)
            .arg(&slots_gpu)
            .arg(&codes_gpu)
            .arg(&pair_keys_gpu)
            .arg(&pair_reads_gpu)
            .arg(&mut work_gpu)
            .arg(&lengths_gpu)
            .arg(&mut words_gpu)
            .arg(&mut meta_gpu)
            .launch(LaunchConfig {
                block_dim: (128, 1, 1),
                grid_dim: (threads.div_ceil(128), 1, 1),
                shared_mem_bytes: 0,
            })
    }
    .map_err(cuda)?;
    ctx.synchronize().map_err(cuda)?;
    let kernel_micros = started.elapsed().as_micros() as u64;
    let work = stream.clone_dtoh(&work_gpu).map_err(cuda)?;
    let words = stream.clone_dtoh(&words_gpu).map_err(cuda)?;
    let meta = stream.clone_dtoh(&meta_gpu).map_err(cuda)?;
    let mut reads = Vec::with_capacity(count);
    for (index, prompt) in prompts.iter().enumerate() {
        let [ids, status, empty, _missed] = [0, 1, 2, 3].map(|field| meta[index * 4 + field]);
        let ids = ids as usize;
        if ids > limit || status > STATUS_NO_FORM {
            return Err(invalid("the kernel returned a reply past its limit"));
        }
        let from = index * stride + prompt.len();
        let at = index * limit * STEP_WORDS;
        reads.push(Read {
            reply: work[from..from + ids].to_vec(),
            words: words[at..at + ids * STEP_WORDS].to_vec(),
            status,
            empty: empty == 1,
        });
    }
    Ok((reads, kernel_micros))
}
