//! Wick AOT kernels for the saved question reader's TRAIN
//! (`reader_train.cu`): the pair rows, the form counts, the kept forms and
//! the held rows are computed on device 0; the host lays the work out
//! ([`crate::conjunction::trained`]) and reads the tables back.
use crate::conjunction::flat::{FORM_WORDS, KEY_WORDS, ROW_WORDS};
use crate::conjunction::free::Update;
use crate::conjunction::trained::{self, Tables, FORMS_A_TURN, JOB_WORDS, PAIR_WORDS, RUN_WORDS};
use crate::conjunction::MAX_SLOTS;
use crate::{Error, Result};
use cudarc::driver::{CudaContext, CudaSlice, LaunchConfig, Ptx, PushKernelArg};
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
/// Words of an offer in the order-free count: a row key and a slot entry.
const OFFER_WORDS: usize = KEY_WORDS + 5;
const OFFER_SLOTS_MOST: usize = 1 << 26;

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
    /// The order-free count: its passes, the most distinct offers of a pass
    /// and the offer table that held them (all 0 for the in-order TRAIN).
    pub passes: u64,
    pub offers: u64,
    pub offer_slots: u64,
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
/// the order of writing matters, and must not differ under
/// [`Update::Free`], whose passes count the replies in no order.
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
    update: Update,
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
    // Room for every form every turn could have, up to the largest table.
    // Turns share most of their forms, so a longer stream is tried in the
    // largest table; the count kernel raises its flag if that fills.
    let form_slots = (runs.len() * FORMS_A_TURN)
        .next_power_of_two()
        .clamp(1 << 16, FORM_SLOTS_MOST);

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
    if let Update::Free { passes } = update {
        // The order-free count (conjunction/free.rs): no levels. Each pass
        // offers, keeps and counts in thirteen launches whatever the order of
        // the jobs.
        struct Held {
            state: CudaSlice<u32>,
            keys: CudaSlice<u32>,
            rows: CudaSlice<u32>,
            capacity: usize,
        }
        let walk_kernel = load("free_walk")?;
        let rows_kernel = load("free_rows")?;
        let best_kernel = load("free_best")?;
        let kept_kernel = load("free_keep")?;
        let sort_kernel = load("free_sort")?;
        let jobs_gpu = put(&jobs)?;
        stats.jobs = job_count as u64;
        stats.passes = passes.max(1) as u64;
        let mut launches = 0u64;
        let mut micros = 0u64;
        let mut offers_most = 0u64;
        let mut offer_slots_most = 0u64;
        let mut pass = |old: &Held, order: &[u32], empty: u32| -> Result<Held> {
            // Most offers and rows these jobs can make: a place is written
            // under each form's own row and at most one shared row, and is
            // offered its id, two reads and three pointers.
            let places: usize = order
                .iter()
                .map(|&job| {
                    let job = &jobs[job as usize * JOB_WORDS..][..JOB_WORDS];
                    (job[1] - job[0]) as usize * job[2] as usize * 2
                })
                .sum();
            let offer_slots = (places * 6)
                .next_power_of_two()
                .clamp(1 << 12, OFFER_SLOTS_MOST);
            let order_gpu = put(order)?;
            let count = small(order.len())?;
            let mut flags_gpu = zeros(4)?;
            let mut offer_state = zeros(offer_slots)?;
            let mut offer_keys = zeros(offer_slots * OFFER_WORDS)?;
            let mut offer_counts = zeros(offer_slots)?;
            let old_capacity = small(old.capacity)?;
            let offer_capacity = small(offer_slots)?;
            ctx.synchronize().map_err(cuda)?;
            let started = std::time::Instant::now();
            // SAFETY: the old rows are only read; an offer's slot is claimed
            // by compare-and-swap before its key is written, and its count
            // grows by atomic adds.
            unsafe {
                stream
                    .launch_builder(&walk_kernel)
                    .arg(&cfg_gpu)
                    .arg(&tokens_gpu)
                    .arg(&jobs_gpu)
                    .arg(&order_gpu)
                    .arg(&old.state)
                    .arg(&old.keys)
                    .arg(&old.rows)
                    .arg(&mut offer_state)
                    .arg(&mut offer_keys)
                    .arg(&mut offer_counts)
                    .arg(&mut flags_gpu)
                    .arg(&old_capacity)
                    .arg(&offer_capacity)
                    .arg(&count)
                    .arg(&empty)
                    .arg(&0u32)
                    .launch(grid(order.len(), 64))
            }
            .map_err(cuda)?;
            ctx.synchronize().map_err(cuda)?;
            micros += started.elapsed().as_micros() as u64;
            launches += 1;
            if stream.clone_dtoh(&flags_gpu).map_err(cuda)?[1] != 0 {
                return Err(invalid("the offers do not fit the largest offer table"));
            }
            let offers = stream
                .clone_dtoh(&offer_state)
                .map_err(cuda)?
                .iter()
                .filter(|&&state| state == SLOT_READY)
                .count();
            offers_most = offers_most.max(offers as u64);
            offer_slots_most = offer_slots_most.max(offer_slots as u64);
            // A row has at least one offer.
            let row_slots = (offers.min(places) * 2)
                .next_power_of_two()
                .clamp(1 << 12, ROW_SLOTS_MOST);
            if offers.min(places) > row_slots {
                return Err(invalid(
                    "the held rows do not fit the largest row table",
                ));
            }
            let mut new = Held {
                state: zeros(row_slots)?,
                keys: zeros(row_slots * KEY_WORDS)?,
                rows: zeros(row_slots * ROW_WORDS)?,
                capacity: row_slots,
            };
            let mut offer_row = zeros(offer_slots)?;
            let mut best = stream
                .alloc_zeros::<u64>(row_slots * MAX_SLOTS)
                .map_err(cuda)?;
            let row_capacity = small(row_slots)?;
            ctx.synchronize().map_err(cuda)?;
            let started = std::time::Instant::now();
            // SAFETY: a thread an offer slot; a row's slot is claimed by
            // compare-and-swap and each thread writes its own word of
            // `offer_row`.
            unsafe {
                stream
                    .launch_builder(&rows_kernel)
                    .arg(&offer_state)
                    .arg(&offer_keys)
                    .arg(&mut new.state)
                    .arg(&mut new.keys)
                    .arg(&mut offer_row)
                    .arg(&mut flags_gpu)
                    .arg(&offer_capacity)
                    .arg(&row_capacity)
                    .launch(grid(offer_slots, 256))
            }
            .map_err(cuda)?;
            for round in 0..MAX_SLOTS as u32 {
                // SAFETY: a row's eight words of `best` grow by atomic
                // maxima; round `round` reads the word of the round before,
                // which no thread of this launch writes.
                unsafe {
                    stream
                        .launch_builder(&best_kernel)
                        .arg(&offer_state)
                        .arg(&offer_keys)
                        .arg(&offer_counts)
                        .arg(&offer_row)
                        .arg(&mut best)
                        .arg(&offer_capacity)
                        .arg(&round)
                        .launch(grid(offer_slots, 256))
                }
                .map_err(cuda)?;
            }
            // SAFETY: two offers of one row never stand equal, so each kept
            // slot of a row is written by one thread.
            unsafe {
                stream
                    .launch_builder(&kept_kernel)
                    .arg(&offer_state)
                    .arg(&offer_keys)
                    .arg(&offer_counts)
                    .arg(&offer_row)
                    .arg(&best)
                    .arg(&mut new.rows)
                    .arg(&offer_capacity)
                    .launch(grid(offer_slots, 256))
            }
            .map_err(cuda)?;
            // SAFETY: the old rows and the new rows' keys and entries are
            // only read; the new rows' counts grow by atomic adds.
            unsafe {
                stream
                    .launch_builder(&walk_kernel)
                    .arg(&cfg_gpu)
                    .arg(&tokens_gpu)
                    .arg(&jobs_gpu)
                    .arg(&order_gpu)
                    .arg(&old.state)
                    .arg(&old.keys)
                    .arg(&old.rows)
                    .arg(&mut new.state)
                    .arg(&mut new.keys)
                    .arg(&mut new.rows)
                    .arg(&mut flags_gpu)
                    .arg(&old_capacity)
                    .arg(&row_capacity)
                    .arg(&count)
                    .arg(&empty)
                    .arg(&1u32)
                    .launch(grid(order.len(), 64))
            }
            .map_err(cuda)?;
            // SAFETY: a thread a row sorts its own fifty-six words.
            unsafe {
                stream
                    .launch_builder(&sort_kernel)
                    .arg(&new.state)
                    .arg(&mut new.rows)
                    .arg(&row_capacity)
                    .launch(grid(row_slots, 256))
            }
            .map_err(cuda)?;
            ctx.synchronize().map_err(cuda)?;
            micros += started.elapsed().as_micros() as u64;
            launches += 3 + MAX_SLOTS as u64 + 1;
            if stream.clone_dtoh(&flags_gpu).map_err(cuda)?[1] != 0 {
                return Err(invalid("the held rows do not fit their row table"));
            }
            Ok(new)
        };
        let order: Vec<u32> = if reversed {
            every.iter().rev().copied().collect()
        } else {
            every.clone()
        };
        // No row before the first pass: a table of empty slots.
        let mut held = Held {
            state: zeros(16)?,
            keys: zeros(16 * KEY_WORDS)?,
            rows: zeros(16 * ROW_WORDS)?,
            capacity: 16,
        };
        for _ in 0..passes.max(1) {
            held = pass(&held, &order, 0)?;
        }
        let missed_cfg = put(&cfg(held.capacity)?)?;
        let mut work_gpu = zeros(job_count * stride)?;
        let mut missed_gpu = zeros(job_count)?;
        let job_count_word = small(job_count)?;
        ctx.synchronize().map_err(cuda)?;
        let started = std::time::Instant::now();
        // SAFETY: the held rows are only read here; each job's thread writes
        // its own work row of `stride` ids and its own word of `missed`.
        unsafe {
            stream
                .launch_builder(&missed_kernel)
                .arg(&missed_cfg)
                .arg(&tokens_gpu)
                .arg(&jobs_gpu)
                .arg(&held.state)
                .arg(&held.keys)
                .arg(&held.rows)
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
        let missed_micros = started.elapsed().as_micros() as u64;
        if stream.clone_dtoh(&flags_gpu).map_err(cuda)?[3] != 0 {
            return Err(invalid("a TRAIN prompt is longer than its work row"));
        }
        let missed = stream.clone_dtoh(&missed_gpu).map_err(cuda)?;
        let again: Vec<u32> = order
            .iter()
            .copied()
            .filter(|&job| missed[job as usize] != 0)
            .collect();
        let for_empty = if again.is_empty() {
            None
        } else {
            Some(pass(&held, &again, 1)?)
        };
        stats.jobs_with_an_empty_read = again.len() as u64;
        stats.row_slots = held.capacity as u64;
        stats.offers = offers_most;
        stats.offer_slots = offer_slots_most;
        stats.launches += launches + 1;
        stats.kernel_micros = kernel_micros + micros + missed_micros;
        let mut tables = Tables {
            forms,
            pair_keys: pairs.keys,
            pair_rows: stream.clone_dtoh(&pair_rows_gpu).map_err(cuda)?,
            ..Tables::default()
        };
        if tables.pair_keys.is_empty() {
            tables.pair_rows.clear();
        }
        for table in std::iter::once(&held).chain(for_empty.as_ref()) {
            let state = stream.clone_dtoh(&table.state).map_err(cuda)?;
            let keys = stream.clone_dtoh(&table.keys).map_err(cuda)?;
            let rows = stream.clone_dtoh(&table.rows).map_err(cuda)?;
            for slot in (0..table.capacity).filter(|&slot| state[slot] == SLOT_READY) {
                tables
                    .row_keys
                    .extend(&keys[slot * KEY_WORDS..(slot + 1) * KEY_WORDS]);
                tables
                    .row_slots
                    .extend(&rows[slot * ROW_WORDS..(slot + 1) * ROW_WORDS]);
            }
        }
        stats.held_rows = (tables.row_keys.len() / KEY_WORDS) as u64;
        return Ok((tables, stats));
    }
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
