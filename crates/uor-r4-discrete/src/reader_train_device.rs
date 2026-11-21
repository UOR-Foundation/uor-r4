//! Wick AOT kernels for the saved question reader's TRAIN
//! (`reader_train.cu`): the pair rows, the form counts, the kept forms and
//! the held rows are computed on device 0; the host lays the work out
//! ([`crate::conjunction::trained`]) and reads the tables back.
use crate::conjunction::flat::{FORM_WORDS, KEY_WORDS, ROW_WORDS};
use crate::conjunction::trained::{self, Tables, FORMS_A_TURN, JOB_WORDS, PAIR_WORDS, RUN_WORDS};
use crate::conjunction::MAX_SLOTS;
use crate::{Error, Result};
use cudarc::driver::{CudaContext, LaunchConfig, Ptx, PushKernelArg};
use sha2::{Digest, Sha256};

const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/reader-train.cubin"));
const TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda-image-target.txt"));

/// Kept forms the buffer between the count and the ranking holds.
const KEPT: usize = 1 << 20;
/// Held-row slots the first try allocates, and the most a retry reaches.
const ROW_SLOTS: usize = 1 << 21;
const ROW_SLOTS_MOST: usize = 1 << 25;
const FORM_SLOTS_MOST: usize = 1 << 26;
const SLOT_READY: u32 = 2;

/// What one TRAIN on the device did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Stats {
    pub replies: u64,
    pub pairs: u64,
    pub form_slots: u64,
    pub forms_kept: u64,
    pub jobs: u64,
    pub second_pass_levels: u64,
    pub second_pass_widest_level: u64,
    pub jobs_with_an_empty_read: u64,
    pub third_pass_levels: u64,
    pub row_slots: u64,
    pub held_rows: u64,
    pub launches: u64,
    /// Launch to end of every kernel, with the host's levelling between
    /// them left out; no context, module or table load, no readback.
    pub kernel_micros: u64,
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

fn grid(threads: usize, block: u32) -> LaunchConfig {
    LaunchConfig {
        block_dim: (block, 1, 1),
        grid_dim: ((threads as u32).div_ceil(block).max(1), 1, 1),
        shared_mem_bytes: 0,
    }
}

/// TRAIN on device 0 with the choices of the saved M2 readers (no class
/// codes, no suffix orders, the pair stand, `precise,across`). `reversed`
/// writes the replies last first: a control whose reader must differ where
/// the order of writing matters.
#[allow(clippy::too_many_arguments)]
pub fn train(
    tokens: &[u32],
    eligible: &[usize],
    vocabulary: u32,
    window: usize,
    bos: u32,
    eos: u32,
    slots: usize,
    limit: usize,
    open: bool,
    reversed: bool,
) -> Result<(Tables, Stats)> {
    if vocabulary > 4_096 || tokens.iter().any(|&token| token >= vocabulary) {
        return Err(invalid(
            "TRAIN ids must lie below a vocabulary of at most 4096",
        ));
    }
    if eligible.is_empty()
        || eligible[eligible.len() - 1] >= tokens.len()
        || eligible.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(invalid("eligible targets must rise and lie in TRAIN"));
    }
    if !(1..=MAX_SLOTS).contains(&slots) || limit == 0 || window == 0 {
        return Err(invalid(
            "needs 1..8 ladder slots, a reply limit and a window",
        ));
    }
    let small = |value: usize| u32::try_from(value).map_err(|_| invalid("a size is past u32"));
    small(tokens.len())?;
    let runs = trained::runs(eligible);
    let run_start: Vec<u32> = runs.iter().map(|run| run[0] as u32).collect();
    let pairs = trained::pairs(tokens, eligible, window, bos);
    let stride = window + limit;
    let form_slots = (runs.len() * FORMS_A_TURN).next_power_of_two().max(1 << 16);
    if form_slots > FORM_SLOTS_MOST {
        return Err(invalid("more replies than the form table is sized for"));
    }

    let ctx = CudaContext::new(0).map_err(cuda)?;
    let (major, minor) = ctx.compute_capability().map_err(cuda)?;
    if TARGET.trim() != format!("sm_{major}{minor}") {
        return Err(invalid("reader-train cubin and device differ"));
    }
    let stream = ctx.default_stream();
    let module = ctx
        .load_module(Ptx::from_binary(CUBIN.to_vec()))
        .map_err(cuda)?;
    let load = |name: &str| module.load_function(name).map_err(cuda);
    let count_kernel = load("train_count")?;
    let keep_kernel = load("train_keep")?;
    let rank_kernel = load("train_rank")?;
    let forms_kernel = load("train_forms")?;
    let pairs_kernel = load("train_pairs")?;
    let write_kernel = load("train_write")?;
    let missed_kernel = load("train_missed")?;
    // An empty table is sent as one unread word.
    let put = |words: &[u32]| {
        stream
            .clone_htod(if words.is_empty() { &[0u32][..] } else { words })
            .map_err(cuda)
    };
    let zeros = |words: usize| stream.alloc_zeros::<u32>(words.max(1)).map_err(cuda);
    let cfg = |row_slots: usize| -> Result<[u32; 11]> {
        Ok([
            vocabulary,
            small(window)?,
            bos,
            eos,
            small(form_slots)?,
            small(row_slots)?,
            small(limit)?,
            small(stride)?,
            small(slots)?,
            small(KEPT)?,
            u32::from(open),
        ])
    };
    let mut stats = Stats {
        replies: runs.len() as u64,
        pairs: pairs.keys.len() as u64,
        form_slots: form_slots as u64,
        ..Stats::default()
    };
    let run_count = small(runs.len())?;
    let pair_count = small(pairs.keys.len())?;

    let tokens_gpu = put(tokens)?;
    let run_start_gpu = put(&run_start)?;
    let pair_keys_gpu = put(&pairs.keys)?;
    let offsets_gpu = put(&pairs.offsets)?;
    let positions_gpu = put(&pairs.positions)?;
    let mut pair_rows_gpu = zeros(pairs.keys.len() * PAIR_WORDS)?;
    let mut cfg_gpu = put(&cfg(ROW_SLOTS)?)?;
    let mut flags_gpu = zeros(4)?;
    let mut form_state = zeros(form_slots)?;
    let mut form_keys = zeros(form_slots * FORM_WORDS)?;
    let mut form_counts = zeros(form_slots)?;
    let mut kept_gpu = zeros(KEPT * FORM_WORDS)?;
    let mut kept_count_gpu = zeros(1)?;
    ctx.synchronize().map_err(cuda)?;

    let started = std::time::Instant::now();
    // SAFETY: every id is below the vocabulary and every position below the
    // stream's length; each pair's thread reads its own positions and writes
    // its own eighteen words.
    unsafe {
        stream
            .launch_builder(&pairs_kernel)
            .arg(&cfg_gpu)
            .arg(&tokens_gpu)
            .arg(&offsets_gpu)
            .arg(&positions_gpu)
            .arg(&mut pair_rows_gpu)
            .arg(&pair_count)
            .launch(grid(pairs.keys.len(), 128))
    }
    .map_err(cuda)?;
    // SAFETY: the form table holds `form_slots` slots of fourteen key words
    // and one count; a slot is claimed by compare-and-swap before its key is
    // written, and counts grow by atomic adds.
    unsafe {
        stream
            .launch_builder(&count_kernel)
            .arg(&cfg_gpu)
            .arg(&tokens_gpu)
            .arg(&run_start_gpu)
            .arg(&mut form_state)
            .arg(&mut form_keys)
            .arg(&mut form_counts)
            .arg(&mut flags_gpu)
            .arg(&run_count)
            .launch(grid(runs.len(), 128))
    }
    .map_err(cuda)?;
    // SAFETY: a thread a form slot; a kept form's place in the buffer comes
    // from an atomic count and is checked against the buffer's size.
    unsafe {
        stream
            .launch_builder(&keep_kernel)
            .arg(&cfg_gpu)
            .arg(&form_state)
            .arg(&form_keys)
            .arg(&form_counts)
            .arg(&mut kept_gpu)
            .arg(&mut kept_count_gpu)
            .arg(&mut flags_gpu)
            .launch(grid(form_slots, 256))
    }
    .map_err(cuda)?;
    ctx.synchronize().map_err(cuda)?;
    let mut kernel_micros = started.elapsed().as_micros() as u64;
    stats.launches += 3;
    let flags = stream.clone_dtoh(&flags_gpu).map_err(cuda)?;
    if flags[0] != 0 || flags[2] != 0 {
        return Err(invalid("the form table or the kept-form buffer is full"));
    }
    let kept = stream.clone_dtoh(&kept_count_gpu).map_err(cuda)?[0] as usize;
    if kept == 0 {
        return Err(invalid("no form was seen eight times"));
    }
    drop((form_state, form_keys, form_counts));
    stats.forms_kept = kept as u64;
    let kept_count = small(kept)?;
    let mut forms_gpu = zeros(kept * FORM_WORDS)?;
    let mut run_forms_gpu = zeros(runs.len() * RUN_WORDS)?;
    ctx.synchronize().map_err(cuda)?;
    let started = std::time::Instant::now();
    // SAFETY: kept forms are distinct, so their ranks are distinct places
    // below `kept`; each thread writes its own form's fourteen words.
    unsafe {
        stream
            .launch_builder(&rank_kernel)
            .arg(&kept_gpu)
            .arg(&mut forms_gpu)
            .arg(&kept_count)
            .launch(grid(kept, 128))
    }
    .map_err(cuda)?;
    // SAFETY: each reply's thread writes its own fourteen words.
    unsafe {
        stream
            .launch_builder(&forms_kernel)
            .arg(&cfg_gpu)
            .arg(&tokens_gpu)
            .arg(&run_start_gpu)
            .arg(&forms_gpu)
            .arg(&mut run_forms_gpu)
            .arg(&kept_count)
            .arg(&run_count)
            .launch(grid(runs.len(), 128))
    }
    .map_err(cuda)?;
    ctx.synchronize().map_err(cuda)?;
    kernel_micros += started.elapsed().as_micros() as u64;
    stats.launches += 2;
    let forms = stream.clone_dtoh(&forms_gpu).map_err(cuda)?;
    // Ranks are places only when the kept forms are distinct: say how many
    // are not, and whether the count table or the ranking made them.
    let repeats = |words: &[u32]| {
        let mut rows: Vec<&[u32]> = words.chunks_exact(FORM_WORDS).collect();
        rows.sort_unstable();
        rows.windows(2).filter(|pair| pair[0] == pair[1]).count()
    };
    let ranked_twice = repeats(&forms);
    if ranked_twice != 0 {
        let unranked = stream.clone_dtoh(&kept_gpu).map_err(cuda)?;
        return Err(invalid(format!(
            "of {kept} kept forms {} stand twice in the count table and {ranked_twice} after ranking",
            repeats(&unranked[..kept * FORM_WORDS])
        )));
    }
    drop(kept_gpu);
    let run_forms = stream.clone_dtoh(&run_forms_gpu).map_err(cuda)?;

    // The host lays the held-row work out; it computes no row.
    let ordered: Vec<[usize; 2]> = if reversed {
        runs.iter().rev().copied().collect()
    } else {
        runs.clone()
    };
    let ordered_forms: Vec<u32> = if reversed {
        run_forms
            .chunks_exact(RUN_WORDS)
            .rev()
            .flatten()
            .copied()
            .collect()
    } else {
        run_forms
    };
    let jobs = trained::jobs(&ordered, &ordered_forms)?;
    let job_count = jobs.len() / JOB_WORDS;
    if job_count == 0 {
        return Err(invalid("no reply stands under a kept form"));
    }
    let every: Vec<u32> = (0..small(job_count)?).collect();
    let (order, bounds) = trained::levels(&jobs, &every, kept)?;
    stats.jobs = job_count as u64;
    stats.second_pass_levels = bounds.len() as u64;
    stats.second_pass_widest_level = bounds
        .iter()
        .map(|bound| u64::from(bound[1]))
        .max()
        .unwrap_or(0);
    small(job_count * stride)?;
    let jobs_gpu = put(&jobs)?;
    let order_gpu = put(&order)?;
    let job_count_word = small(job_count)?;

    let mut row_slots = ROW_SLOTS;
    loop {
        cfg_gpu = put(&cfg(row_slots)?)?;
        flags_gpu = zeros(4)?;
        let mut row_state = zeros(row_slots)?;
        let mut row_keys = zeros(row_slots * KEY_WORDS)?;
        let mut row_rows = zeros(row_slots * ROW_WORDS)?;
        let mut work_gpu = zeros(job_count * stride)?;
        let mut missed_gpu = zeros(job_count)?;
        let mut launches = 0u64;
        ctx.synchronize().map_err(cuda)?;
        let started = std::time::Instant::now();
        let mut write = |order_gpu: &cudarc::driver::CudaSlice<u32>,
                         bounds: &[[u32; 2]],
                         empty: u32,
                         row_state: &mut cudarc::driver::CudaSlice<u32>,
                         row_keys: &mut cudarc::driver::CudaSlice<u32>,
                         row_rows: &mut cudarc::driver::CudaSlice<u32>,
                         flags_gpu: &mut cudarc::driver::CudaSlice<u32>|
         -> Result<()> {
            for bound in bounds {
                // SAFETY: the jobs of one level hold no form in common, so
                // no two threads of a launch read or write the same held
                // row; a row's slot is claimed by compare-and-swap; launches
                // on one stream run in order.
                unsafe {
                    stream
                        .launch_builder(&write_kernel)
                        .arg(&cfg_gpu)
                        .arg(&tokens_gpu)
                        .arg(&jobs_gpu)
                        .arg(order_gpu)
                        .arg(&mut *row_state)
                        .arg(&mut *row_keys)
                        .arg(&mut *row_rows)
                        .arg(&mut *flags_gpu)
                        .arg(&bound[0])
                        .arg(&bound[1])
                        .arg(&empty)
                        .launch(grid(bound[1] as usize, 64))
                }
                .map_err(cuda)?;
                launches += 1;
            }
            Ok(())
        };
        write(
            &order_gpu,
            &bounds,
            0,
            &mut row_state,
            &mut row_keys,
            &mut row_rows,
            &mut flags_gpu,
        )?;
        // SAFETY: the held rows are only read here; each job's thread writes
        // its own work row of `stride` ids and its own word of `missed`.
        unsafe {
            stream
                .launch_builder(&missed_kernel)
                .arg(&cfg_gpu)
                .arg(&tokens_gpu)
                .arg(&jobs_gpu)
                .arg(&mut row_state)
                .arg(&mut row_keys)
                .arg(&mut row_rows)
                .arg(&pair_keys_gpu)
                .arg(&pair_rows_gpu)
                .arg(&mut work_gpu)
                .arg(&mut missed_gpu)
                .arg(&mut flags_gpu)
                .arg(&pair_count)
                .arg(&job_count_word)
                .launch(grid(job_count, 64))
        }
        .map_err(cuda)?;
        ctx.synchronize().map_err(cuda)?;
        let mut micros = started.elapsed().as_micros() as u64;
        let missed = stream.clone_dtoh(&missed_gpu).map_err(cuda)?;
        let again: Vec<u32> = every
            .iter()
            .copied()
            .filter(|&job| missed[job as usize] != 0)
            .collect();
        let (again_order, again_bounds) = trained::levels(&jobs, &again, kept)?;
        let again_gpu = put(&again_order)?;
        ctx.synchronize().map_err(cuda)?;
        let started = std::time::Instant::now();
        write(
            &again_gpu,
            &again_bounds,
            1,
            &mut row_state,
            &mut row_keys,
            &mut row_rows,
            &mut flags_gpu,
        )?;
        ctx.synchronize().map_err(cuda)?;
        micros += started.elapsed().as_micros() as u64;
        let flags = stream.clone_dtoh(&flags_gpu).map_err(cuda)?;
        if flags[3] != 0 {
            return Err(invalid("a TRAIN prompt is longer than its work row"));
        }
        if flags[1] != 0 {
            if row_slots >= ROW_SLOTS_MOST {
                return Err(invalid("the held rows do not fit the largest row table"));
            }
            row_slots *= 2;
            continue;
        }
        stats.jobs_with_an_empty_read = again.len() as u64;
        stats.third_pass_levels = again_bounds.len() as u64;
        stats.row_slots = row_slots as u64;
        stats.launches += launches + 1;
        stats.kernel_micros = kernel_micros + micros;

        let state = stream.clone_dtoh(&row_state).map_err(cuda)?;
        let keys = stream.clone_dtoh(&row_keys).map_err(cuda)?;
        let rows = stream.clone_dtoh(&row_rows).map_err(cuda)?;
        let mut tables = Tables {
            forms,
            pair_keys: pairs.keys,
            pair_rows: stream.clone_dtoh(&pair_rows_gpu).map_err(cuda)?,
            ..Tables::default()
        };
        if tables.pair_keys.is_empty() {
            tables.pair_rows.clear();
        }
        for slot in (0..row_slots).filter(|&slot| state[slot] == SLOT_READY) {
            tables
                .row_keys
                .extend(&keys[slot * KEY_WORDS..(slot + 1) * KEY_WORDS]);
            tables
                .row_slots
                .extend(&rows[slot * ROW_WORDS..(slot + 1) * ROW_WORDS]);
        }
        stats.held_rows = (tables.row_keys.len() / KEY_WORDS) as u64;
        return Ok((tables, stats));
    }
}
