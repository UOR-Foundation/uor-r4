//! Explicit opt-in K-coordinate row-parallel scoring handle (BVG-K Stage C).
//! Nothing selects it implicitly; no driver path calls it. It scores K
//! coordinates of one kind in one launch and returns each coordinate's sixteen
//! losses and status. It runs no chooser and applies nothing: state and epoch
//! are unchanged.
use super::*;

pub(super) const ROWSK_SOURCE: &str = include_str!(concat!(env!("OUT_DIR"), "/rowsk.cu"));
pub(super) const ROWSK_PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/rowsk.ptx"));
pub(super) const ROWSK_CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rowsk.cubin"));
pub(super) const ROWSK_RESOURCE_REPORT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/rowsk-resources.txt"));

/// Largest coordinate block one launch admits (BVG-K control V6 bound).
pub const MAX_COORDINATE_BLOCK: usize = 64;

/// Diagnostic wall time of the scoring launches, filled only when the
/// `UOR_ROWSK_PROFILE` variable is set: index build, baseline, eval, reduce
/// and the joint forward, in microseconds, then the joint forward count.
static SCORE_PHASE_MICROS: [std::sync::atomic::AtomicU64; 6] =
    [const { std::sync::atomic::AtomicU64::new(0) }; 6];

pub fn score_phase_micros() -> [u64; 6] {
    std::array::from_fn(|i| SCORE_PHASE_MICROS[i].load(std::sync::atomic::Ordering::Relaxed))
}

fn profiling() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("UOR_ROWSK_PROFILE").is_some())
}

/// Synchronize and add the time since `clock` to `phase`; nothing unless profiling.
pub(super) fn profile_mark(
    stream: &Arc<CudaStream>,
    clock: &mut Instant,
    phase: usize,
    count: bool,
) -> Result<()> {
    if profiling() {
        stream.synchronize().map_err(cuda)?;
        let spent = u64::try_from(clock.elapsed().as_micros()).unwrap_or(u64::MAX);
        SCORE_PHASE_MICROS[phase].fetch_add(spent, std::sync::atomic::Ordering::Relaxed);
        if count {
            SCORE_PHASE_MICROS[5].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        *clock = Instant::now();
    }
    Ok(())
}

pub(super) const ROWSKB_SOURCE: &str = include_str!(concat!(env!("OUT_DIR"), "/rowskb.cu"));
pub(super) const ROWSKB_CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rowskb.cubin"));

/// The bucketed twins of the baseline and eval kernels, with the index build.
struct BucketKernels {
    build: CudaFunction,
    baseline: CudaFunction,
    eval: CudaFunction,
}

/// The CSR token-cell index of one table state, device-resident.
struct BucketIndex {
    offsets: CudaSlice<u32>,
    members: CudaSlice<u32>,
    cells: CudaSlice<u32>,
    absent_count: CudaSlice<u32>,
}

pub struct RowsKScoring {
    bucket: Option<BucketKernels>,
    baseline: CudaFunction,
    eval: CudaFunction,
    reduce: CudaFunction,
    binding: Binding,
    stream: Arc<CudaStream>,
}

/// One coordinate's scoring result: the ordinary sixteen-slot losses, or the
/// status code the ordinary proposal would have refused with (non-zero).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinateScore {
    pub parameter: Parameter,
    pub status: u32,
    pub losses: Vec<u64>,
}

impl Device {
    /// Load the opt-in K-coordinate module for this device's model and stream.
    pub fn rowsk_scoring(&self) -> Result<RowsKScoring> {
        admit_cuda_target(&self.context)?;
        let module = self
            .context
            .load_module(Ptx::from_binary(ROWSK_CUBIN.to_vec()))
            .map_err(cuda)?;
        let load = |name: &str| module.load_function(name).map_err(cuda);
        Ok(RowsKScoring {
            bucket: None,
            baseline: load(crate::rowsk_source::ENTRIES[0])?,
            eval: load(crate::rowsk_source::ENTRIES[3])?,
            reduce: load(crate::rowsk_source::ENTRIES[4])?,
            binding: self.binding.clone(),
            stream: Arc::clone(&self.stream),
        })
    }

    /// The K-coordinate handle with the baseline and eval kernels on the CSR
    /// bucketed writer (exact `IdOrder` walk, #2840). Losses and statuses are
    /// those of `rowsk_scoring`; only the writer's token walk differs. A Pick
    /// read or a non-outer address metric, which the bucket text does not
    /// carry, keeps the full-scan kernels.
    pub fn rowsk_scoring_bucketed(&self) -> Result<RowsKScoring> {
        let mut scoring = self.rowsk_scoring()?;
        if !self.binding.address_metric.is_outer() {
            return Ok(scoring);
        }
        let expected = crate::rowskb_source::generate(super::writer_bucket_device::BUCKET_SOURCE, include_str!("discrete.cu"))
            .map_err(invalid)?;
        if ROWSKB_SOURCE != expected {
            return Err(invalid(
                "embedded rowskb.cu differs from rowskb_source::generate(bucket.cu)",
            ));
        }
        let module = self
            .context
            .load_module(Ptx::from_binary(ROWSKB_CUBIN.to_vec()))
            .map_err(cuda)?;
        let load = |name: &str| module.load_function(name).map_err(cuda);
        scoring.bucket = Some(BucketKernels {
            build: load(crate::rowskb_source::ENTRIES[0])?,
            baseline: load(crate::rowskb_source::ENTRIES[7])?,
            eval: load(crate::rowskb_source::ENTRIES[8])?,
        });
        Ok(scoring)
    }

    /// Build the token-cell index of the current table on this stream.
    fn rowsk_bucket_index(&self, kernels: &BucketKernels) -> Result<BucketIndex> {
        let vocabulary = self.spec().vocabulary as usize;
        let spec = self.gpu_spec(1)?;
        let mut offsets = self.stream.alloc_zeros::<u32>(257).map_err(cuda)?;
        let mut members = self.stream.alloc_zeros::<u32>(vocabulary).map_err(cuda)?;
        let mut cells = self.stream.alloc_zeros::<u32>(vocabulary).map_err(cuda)?;
        let mut absent_count = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&kernels.build);
        launch
            .arg(&self.table)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&mut offsets)
            .arg(&mut members)
            .arg(&mut cells)
            .arg(&mut absent_count)
            .arg(&mut status);
        let config = LaunchConfig {
            grid_dim: (1, 1, 1),
            block_dim: (256, 1, 1),
            shared_mem_bytes: 0,
        };
        // SAFETY: 257 offsets, vocabulary members and cells, one absent count
        // and one status word; the build kernel runs in one 256-thread block,
        // the launch shape `m1-faster-writer` qualified.
        unsafe { launch.launch(config) }.map_err(cuda)?;
        check_status(&self.stream, &status, "K-coordinate bucket build")?;
        Ok(BucketIndex {
            offsets,
            members,
            cells,
            absent_count,
        })
    }

    /// Score `parameters` (1..=64, all one kind) in one baseline launch, one
    /// eval launch over (coordinate, row, lane) and one reduce launch over
    /// (coordinate, lane). Each coordinate's losses and status equal an
    /// independent `propose_rows` for it on the same state (control V2).
    pub fn score_rows_k(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        parameters: &[Parameter],
        scoring: &RowsKScoring,
    ) -> Result<Vec<CoordinateScore>> {
        let scores = self.score_rows_k_device(batch, targets, parameters, scoring)?;
        self.stream.synchronize().map_err(cuda)?;
        let losses = self.stream.clone_dtoh(&scores.losses).map_err(cuda)?;
        let status = self.stream.clone_dtoh(&scores.status).map_err(cuda)?;
        if self.epoch()? != scores.epoch {
            return Err(invalid("K-coordinate scoring changed the model epoch"));
        }
        Ok(parameters
            .iter()
            .enumerate()
            .map(|(c, &parameter)| CoordinateScore {
                parameter,
                status: status[c],
                losses: losses[c * 16..c * 16 + 16].to_vec(),
            })
            .collect())
    }

    /// The three scoring launches of `score_rows_k`, leaving the per-coordinate
    /// losses, status words and indices on the device (queued on this stream,
    /// not yet synchronized). `epoch` is the admitted epoch.
    pub(super) fn score_rows_k_device(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        parameters: &[Parameter],
        scoring: &RowsKScoring,
    ) -> Result<DeviceScores> {
        if scoring.binding != self.binding || !Arc::ptr_eq(&scoring.stream, &self.stream) {
            return Err(invalid(
                "K-coordinate handle belongs to another model/stream",
            ));
        }
        if parameters.is_empty() || parameters.len() > MAX_COORDINATE_BLOCK {
            return Err(invalid(format!(
                "coordinate block must hold 1..={MAX_COORDINATE_BLOCK} parameters"
            )));
        }
        let mut kind = None;
        let mut indices = Vec::with_capacity(parameters.len());
        for &parameter in parameters {
            let (k, index) = self.parameter(parameter)?;
            if *kind.get_or_insert(k) != k {
                return Err(invalid("a coordinate block must hold one parameter kind"));
            }
            indices.push(index);
        }
        let kind = kind.ok_or_else(|| invalid("empty coordinate block"))?;
        let block = u32::try_from(parameters.len())
            .map_err(|_| invalid("coordinate block exceeds the CUDA index bound"))?;
        let epoch_before = self.admit_batch(batch, targets)?;
        let targets = self.stream.clone_htod(targets).map_err(cuda)?;
        let indices_device = self.stream.clone_htod(&indices).map_err(cuda)?;
        let spec = self.gpu_spec(batch.rows)?;
        let plan = batch
            .plan
            .as_ref()
            .ok_or_else(|| invalid("missing admitted row plans"))?;
        let row_count = batch.rows as usize;
        let per = batch
            .rows
            .checked_mul(16)
            .ok_or_else(|| invalid("K-coordinate pair count overflows"))?;
        let triples = per
            .checked_mul(block)
            .ok_or_else(|| invalid("K-coordinate triple count overflows"))?;
        let mut base = self
            .stream
            .alloc_zeros::<u32>(row_count * 2)
            .map_err(cuda)?;
        let mut cost = self
            .stream
            .alloc_zeros::<u64>(triples as usize)
            .map_err(cuda)?;
        let mut flag = self
            .stream
            .alloc_zeros::<u32>(triples as usize)
            .map_err(cuda)?;
        let mut losses = self
            .stream
            .alloc_zeros::<u64>(parameters.len() * 16)
            .map_err(cuda)?;
        let mut status = self
            .stream
            .alloc_zeros::<u32>(parameters.len())
            .map_err(cuda)?;

        // The index lives until this function returns; the launches that read
        // it are queued on this stream before it is released.
        let mut clock = Instant::now();
        let index = match &scoring.bucket {
            Some(kernels) => Some(self.rowsk_bucket_index(kernels)?),
            None => None,
        };
        profile_mark(&self.stream, &mut clock, 0, false)?;
        if let (Some(kernels), Some(index)) = (&scoring.bucket, &index) {
            // Each row's baseline read, stored once and reused by every lane
            // whose candidate cannot change that row's read.
            let mut qbase = self
                .stream
                .alloc_zeros::<i32>(row_count * self.spec().blocks as usize * 8)
                .map_err(cuda)?;
            let mut launch = self.stream.launch_builder(&kernels.baseline);
            launch
                .arg(&self.table)
                .arg(&self.routes)
                .arg(&self.gates)
                .arg(&batch.ids)
                .arg(&batch.vectors)
                .arg(&targets)
                .arg(plan)
                .arg(&self.geometry)
                .arg(&spec)
                .arg(&self.position_steps)
                .arg(&index.offsets)
                .arg(&index.members)
                .arg(&index.cells)
                .arg(&index.absent_count)
                .arg(&mut base)
                .arg(&mut qbase);
            // SAFETY: rows*2 base words and rows*blocks*8 baseline read words;
            // one thread per admitted row; the index holds 257 offsets and
            // vocabulary members and cells.
            unsafe { launch.launch(launch_config(batch.rows)) }.map_err(cuda)?;
            profile_mark(&self.stream, &mut clock, 1, false)?;

            let mut launch = self.stream.launch_builder(&kernels.eval);
            launch
                .arg(&self.table)
                .arg(&self.routes)
                .arg(&self.gates)
                .arg(&batch.ids)
                .arg(&batch.vectors)
                .arg(&targets)
                .arg(plan)
                .arg(&self.geometry)
                .arg(&spec)
                .arg(&self.position_steps)
                .arg(&index.offsets)
                .arg(&index.members)
                .arg(&index.cells)
                .arg(&index.absent_count)
                .arg(&kind)
                .arg(&indices_device)
                .arg(&block)
                .arg(&base)
                .arg(&qbase)
                .arg(&mut cost)
                .arg(&mut flag);
            // SAFETY: block*rows*16 cost and flag words; one thread per
            // (coordinate, row, lane), ordered after the baseline on this stream.
            unsafe { launch.launch(launch_config(triples)) }.map_err(cuda)?;
        } else {
            let mut launch = self.stream.launch_builder(&scoring.baseline);
            launch
                .arg(&self.table)
                .arg(&self.routes)
                .arg(&self.gates)
                .arg(&batch.ids)
                .arg(&batch.vectors)
                .arg(&targets)
                .arg(plan)
                .arg(&self.geometry)
                .arg(&spec)
                .arg(&self.position_steps)
                .arg(&mut base);
            // SAFETY: rows*2 base words; one thread per admitted row, as in the
            // row-parallel baseline (the identical kernel text).
            unsafe { launch.launch(launch_config(batch.rows)) }.map_err(cuda)?;

            let mut launch = self.stream.launch_builder(&scoring.eval);
            launch
                .arg(&self.table)
                .arg(&self.routes)
                .arg(&self.gates)
                .arg(&batch.ids)
                .arg(&batch.vectors)
                .arg(&targets)
                .arg(plan)
                .arg(&self.geometry)
                .arg(&spec)
                .arg(&self.position_steps)
                .arg(&kind)
                .arg(&indices_device)
                .arg(&block)
                .arg(&base)
                .arg(&mut cost)
                .arg(&mut flag);
            // SAFETY: block*rows*16 cost and flag words; one thread per
            // (coordinate, row, lane), ordered after the baseline on this stream.
            unsafe { launch.launch(launch_config(triples)) }.map_err(cuda)?;
        }

        profile_mark(&self.stream, &mut clock, 2, false)?;
        let reducers = block * 16;
        let mut launch = self.stream.launch_builder(&scoring.reduce);
        launch
            .arg(&spec)
            .arg(&kind)
            .arg(&indices_device)
            .arg(&block)
            .arg(&base)
            .arg(&cost)
            .arg(&flag)
            .arg(&mut losses)
            .arg(&mut status);
        // SAFETY: block*16 loss words and block status words; one thread per
        // (coordinate, lane), ordered after both producers on this stream.
        unsafe { launch.launch(launch_config(reducers)) }.map_err(cuda)?;
        profile_mark(&self.stream, &mut clock, 3, false)?;
        Ok(DeviceScores {
            kind,
            indices: indices_device,
            losses,
            status,
            epoch: epoch_before,
        })
    }
}

/// Device-resident K-coordinate scores: `losses[c*16 + lane]`, `status[c]`,
/// `indices[c]`, the block's CUDA kind and the epoch they were scored at.
pub(super) struct DeviceScores {
    pub(super) kind: u32,
    pub(super) indices: CudaSlice<u32>,
    pub(super) losses: CudaSlice<u64>,
    pub(super) status: CudaSlice<u32>,
    pub(super) epoch: u64,
}

/// The embedded K-coordinate source begins with the embedded row-parallel
/// source byte for byte (the rows image is unchanged; rows-K only appends).
pub fn rowsk_extends_rows_source() -> bool {
    ROWSK_SOURCE.starts_with(super::rows_device::ROWS_SOURCE)
}

/// Identity of the embedded K-coordinate image, without a CUDA context.
pub fn compiled_rowsk_image_sha256() -> [String; 4] {
    [
        format!("{:x}", Sha256::digest(ROWSK_SOURCE.as_bytes())),
        format!("{:x}", Sha256::digest(ROWSK_PTX.as_bytes())),
        format!("{:x}", Sha256::digest(ROWSK_CUBIN)),
        format!("{:x}", Sha256::digest(ROWSK_RESOURCE_REPORT.as_bytes())),
    ]
}
