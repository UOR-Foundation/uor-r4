//! M1 control 7: actual FBS A/value/P, S1 and one native vector writer.
//! This is a CUDA fixture, not training or a CPU reference. The detached
//! writer table is constructed once per explicitly declared K1/K0 regime.
//! The K1 credit-on/off comparison uses the identical fixed table. K0 is a
//! separate fixed-objective wake-up control, not a cross-table loss comparison.
//! FBS query gradients are the production geometric secant surrogate, not
//! derivatives of its discontinuous hard routes. No FBS secant bar is added.

use super::murmuration::{
    murmuration_read, selection_spec, AddressSink, Chirality, MurmurationInputs,
};
use super::{geometry, unit_address, Credit, NativeStarling, CELLS};
use crate::native_vector_writer::{NativeVectorWriter, WriterSpec};
use crate::spiralcore_fbs::ProjectionPlan;
use crate::{invalid, Result};
use candle_core::{DType, Device, Tensor, Var};
use std::sync::{Arc, Mutex};

const TIME: usize = 4;
const WIDTH: usize = 16;
const HEADS: usize = 4;
const TARGET: u32 = 511;

// One integration seam: the repaired selector excludes null tokens before
// storage/ranking and internally provisions the K+1 selection window.
fn selection(
    spec: &NativeStarling,
    address: &Tensor,
    norm: &Tensor,
    reset: &Tensor,
) -> Result<Tensor> {
    super::murmuration::murmuration_select(spec, HEADS, address, norm, reset)
}

fn near(a: &Tensor, b: &Tensor, tolerance: f64, label: &str) -> Result<()> {
    let worst = a.sub(b)?.abs()?.max_all()?.to_scalar::<f32>()?;
    assert!(
        worst.is_finite() && f64::from(worst) <= tolerance,
        "{label}: CUDA maximum error {worst}"
    );
    Ok(())
}

fn finite_nonzero(t: &Tensor, label: &str) -> Result<()> {
    assert!(t.device().is_cuda(), "{label} left CUDA");
    let size = t.abs()?.sum_all()?.to_scalar::<f32>()?;
    assert!(size.is_finite() && size > 1e-7, "{label}: {size}");
    Ok(())
}

fn gradient(grads: &candle_core::backprop::GradStore, var: &Var) -> Result<Tensor> {
    match grads.get(var.as_tensor()) {
        Some(t) => Ok(t.clone()),
        // A is disconnected with capture disabled. This is the mathematical
        // zero of that declared absence, never a substitute model output.
        None => Ok(var.as_tensor().zeros_like()?),
    }
}

struct Parameters {
    a: Var,
    value: Var,
    p: Var,
    coupling: Var,
}
impl Parameters {
    fn new(a: &Arc<ProjectionPlan>, p: &Arc<ProjectionPlan>, k: f32, d: &Device) -> Result<Self> {
        let points = |plan: &Arc<ProjectionPlan>, index: usize| -> Result<Var> {
            let root = plan.geometry.roots[index].map(f32::from);
            Ok(Var::from_vec(
                root.into_iter()
                    .cycle()
                    .take(plan.query_shape().iter().product())
                    .collect(),
                &plan.query_shape()[..],
                d,
            )?)
        };
        Ok(Self {
            a: points(a, 0)?,
            value: points(a, 1)?,
            p: points(p, 0)?,
            coupling: Var::from_vec(vec![k, k, 2. * k, 2. * k], HEADS, d)?,
        })
    }
}

struct Fixture {
    a: Arc<ProjectionPlan>,
    p: Arc<ProjectionPlan>,
    spec: NativeStarling,
    reset: Tensor,
    off_home: Tensor,
    writer: NativeVectorWriter,
}

struct Observation {
    delta: Tensor,
    query: Tensor,
    loss: Tensor,
    raw_address: Tensor,
    input: Tensor,
    sink: AddressSink,
}

impl Fixture {
    fn new(d: &Device, anchor_k: f32) -> Result<Self> {
        // Static icosian/index metadata only. All transport/fixture arithmetic
        // below, including the helpful off-home value, executes on CUDA.
        let (roots, adjacent) = geometry()?;
        let home = 30;
        let other = adjacent[home * 12] as usize;
        let roots = Tensor::from_vec(roots, (CELLS, 4), d)?;
        let c_a = roots.narrow(0, home, 1)?.reshape(4)?;
        let c_b = roots.narrow(0, other, 1)?.reshape(4)?;
        let w = Tensor::new(&[1f32, -1., 1., 1.], d)?.affine(32., 0.)?;
        let conjugation = Tensor::new(&[1f32, -1., -1., -1.], d)?;
        let rel = super::murmuration::relative_table()?;
        let g = roots
            .narrow(0, rel[home * CELLS + other] as usize, 1)?
            .reshape(4)?;
        let v_b = super::murmuration::hamilton(&g.mul(&conjugation)?, &w.neg()?)?;
        let address = Tensor::stack(&[&c_a, &c_b, &c_a, &c_a], 0)?;
        // Kept home values w; the reader's .4w makes Delta and K wake-up
        // nonzero. Off-home exploration is carried to -w in the reader frame.
        let value = Tensor::stack(&[&w, &v_b, &w, &w.affine(0.4, 0.)?], 0)?;
        let first = Tensor::cat(&[&address, &address], 1)?;
        let desired_value = Tensor::cat(&[&value, &value], 1)?;
        let a = ProjectionPlan::new(WIDTH, WIDTH)?;
        let p = ProjectionPlan::new(WIDTH, WIDTH)?;
        assert_eq!(a.frontier().len(), 2);
        assert_eq!(p.frontier().len(), 2);
        // Exact signed-axis metadata for atom1's BFS action. Invert it on the
        // GPU to make V's selected output the declared transported values.
        let g = &a.geometry;
        let state = g.compose[g.bfs_actions[1] as usize * 64 + g.identity as usize] as usize;
        let mut inverse = [0u32; 8];
        for (out, &code) in g.axes[state * 8..state * 8 + 8].iter().enumerate() {
            inverse[(code & 7) as usize] = out as u32 | (code & 8);
        }
        let second = signed_axes(&desired_value, &inverse)?;
        let seed = Tensor::cat(&[&first, &second], 1)?.reshape((1, TIME, WIDTH))?;
        let spec = NativeStarling {
            depth: 4,
            kept: 2,
            explore: 2,
            bos_id: 0,
            credit: Credit::FIXTURE,
        };
        let reset = Tensor::new(&[[1u32, 0, 0, 0]], d)?;
        let parameters = Parameters::new(&a, &p, anchor_k, d)?;
        // Device fixture preparation at the one frozen nonzero-coupling
        // anchor. No table is reanchored for credit or coupling comparisons.
        let raw = heads(&a.forward(parameters.a.as_tensor(), &seed)?)?;
        let unit = unit_address(&raw)?;
        let norm = raw.sqr()?.sum(2)?.sqrt()?;
        let values = heads(&a.forward(parameters.value.as_tensor(), &seed)?)?;
        near(
            &values.narrow(0, 0, 1)?.squeeze(0)?,
            &value,
            0.,
            "V exact BFS inverse",
        )?;
        let selected = selection(&spec, &unit, &norm, &reset)?;
        let delta = murmuration_read(
            &spec,
            &MurmurationInputs {
                address: &unit,
                address_norm: &norm,
                value: &values,
                coupling: parameters.coupling.as_tensor(),
                selection: &selected,
                heads: HEADS,
                chirality: Chirality::Left,
            },
            false,
            None,
        )?
        .delta;
        let anchor = seed
            .add(&p.forward(parameters.p.as_tensor(), &concat_heads(&delta)?)?)?
            .narrow(1, TIME - 1, 1)?
            .reshape((1, WIDTH))?
            .detach();
        let sign = w.gt(0.)?.to_dtype(DType::F32)?.affine(2., -1.)?;
        let direction = Tensor::cat(&[&sign, &sign, &sign, &sign], 0)?.reshape((1, WIDTH))?;
        let shift = direction.affine(0.25, 0.)?;
        let rivals = anchor.add(&shift)?.broadcast_as((507, WIDTH))?;
        let target = anchor.sub(&shift)?;
        let table = Tensor::cat(&[&seed.reshape((TIME, WIDTH))?, &rivals, &target], 0)?.detach();
        let writer = NativeVectorWriter::new(
            &table,
            WriterSpec {
                vocabulary: 512,
                width: WIDTH,
                kept_cells: 16,
            },
        )?;
        near(
            &writer.encode(&[0, 1, 2, 3])?,
            &seed.reshape((TIME, WIDTH))?,
            0.,
            "same input table",
        )?;
        Ok(Self {
            a,
            p,
            spec,
            reset,
            off_home: c_b,
            writer,
        })
    }

    fn run(&self, params: &Parameters, credit: bool) -> Result<Observation> {
        // The same writer/table supplies token vectors and scores the output.
        let input = self
            .writer
            .encode(&[0, 1, 2, 3])?
            .reshape((1, TIME, WIDTH))?;
        let raw = heads(&self.a.forward(params.a.as_tensor(), &input)?)?;
        let address = unit_address(&raw)?;
        let norm = raw.sqr()?.sum(2)?.sqrt()?;
        let value = heads(&self.a.forward(params.value.as_tensor(), &input)?)?;
        let picked = selection(&self.spec, &address, &norm, &self.reset)?;
        // This is a real selector result, not a manually authored slot list.
        let selector = selection_spec(&self.spec)?;
        let live = picked
            .narrow(1, TIME - 1, 1)?
            .narrow(2, 0, selector.slots())?;
        assert_eq!(
            live.eq(1u32)?
                .to_dtype(DType::U32)?
                .sum_all()?
                .to_scalar::<u32>()?,
            HEADS as u32,
            "helpful off-home token must be an actual candidate"
        );
        let exploration = live.narrow(2, selector.kept, self.spec.explore)?;
        assert_eq!(
            exploration
                .eq(1u32)?
                .to_dtype(DType::U32)?
                .sum_all()?
                .to_scalar::<u32>()?,
            HEADS as u32,
            "helpful off-home token must lie in the real zero-forward exploration slots"
        );
        let kept_window = live.narrow(2, 0, selector.kept)?;
        for token in [0u32, 2] {
            assert_eq!(
                kept_window
                    .eq(token)?
                    .to_dtype(DType::U32)?
                    .sum_all()?
                    .to_scalar::<u32>()?,
                HEADS as u32,
                "both home tokens must lie in the real kept window"
            );
        }
        let sink: AddressSink = Arc::new(Mutex::new(Vec::new()));
        let read = murmuration_read(
            &self.spec,
            &MurmurationInputs {
                address: &address,
                address_norm: &norm,
                value: &value,
                coupling: params.coupling.as_tensor(),
                selection: &picked,
                heads: HEADS,
                chirality: Chirality::Left,
            },
            credit,
            Some(Arc::clone(&sink)),
        )?;
        assert_eq!(
            read.flock
                .narrow(1, TIME - 1, 1)?
                .sum_all()?
                .to_scalar::<f32>()?,
            2. * HEADS as f32
        );
        let query = input
            .add(
                &self
                    .p
                    .forward(params.p.as_tensor(), &concat_heads(&read.delta)?)?,
            )?
            .narrow(1, TIME - 1, 1)?
            .reshape((1, WIDTH))?;
        let candidates = self.writer.candidates(&query)?;
        let margins = self.writer.margin_rows(&candidates, &[TARGET], 1)?;
        let loss = margins
            .token_margin
            .add(&margins.address_margin)?
            .sum_all()?;
        Ok(Observation {
            delta: read.delta,
            query,
            loss,
            raw_address: raw,
            input,
            sink,
        })
    }
}

fn heads(vector: &Tensor) -> Result<Tensor> {
    Ok(vector
        .reshape((1, TIME, HEADS, 4))?
        .transpose(1, 2)?
        .contiguous()?
        .reshape((HEADS, TIME, 4))?)
}

fn concat_heads(delta: &Tensor) -> Result<Tensor> {
    // Exactly P(concat_heads Delta), with no padding, dummy or probe carrier.
    Ok(delta
        .reshape((1, HEADS, TIME, 4))?
        .transpose(1, 2)?
        .contiguous()?
        .reshape((1, TIME, WIDTH))?)
}

fn signed_axes(input: &Tensor, axes: &[u32; 8]) -> Result<Tensor> {
    let pieces: Vec<Tensor> = axes
        .iter()
        .map(|&code| {
            Ok(input
                .narrow(1, (code & 7) as usize, 1)?
                .affine(if code & 8 == 0 { 1. } else { -1. }, 0.)?)
        })
        .collect::<Result<_>>()?;
    Ok(Tensor::cat(&pieces, 1)?)
}

/// Exact scoped name: starling_native::murmuration_composed::murm_composed_credit.
/// The production operator is expected to have the corrected per-reader .1
/// address law. The old /queries implementation deliberately fails this test.
#[test]
#[cfg_attr(not(feature = "cuda"), ignore = "GPU control: admitted CUDA pod only")]
fn murm_composed_credit() -> Result<()> {
    let device = Device::new_cuda(0)?;
    let fixture = Fixture::new(&device, 1.)?;
    let on = Parameters::new(&fixture.a, &fixture.p, 1., &device)?;
    let off = Parameters::new(&fixture.a, &fixture.p, 1., &device)?;
    let with = fixture.run(&on, true)?;
    let without = fixture.run(&off, false)?;
    near(&with.delta, &without.delta, 0., "capture forward Delta")?;
    near(
        &with.query,
        &without.query,
        0.,
        "capture forward writer input",
    )?;
    near(
        &with.loss,
        &without.loss,
        0.,
        "capture forward native objective",
    )?;
    let go = with.loss.backward()?;
    let gf = without.loss.backward()?;
    for (label, a, b) in [
        ("value", &on.value, &off.value),
        ("P", &on.p, &off.p),
        ("K", &on.coupling, &off.coupling),
    ] {
        let actual = gradient(&go, a)?;
        finite_nonzero(&actual, label)?;
        near(&actual, &gradient(&gf, b)?, 1e-6, label)?;
    }
    finite_nonzero(&gradient(&go, &on.a)?, "A")?;
    near(
        &gradient(&gf, &off.a)?,
        &off.a.as_tensor().zeros_like()?,
        0.,
        "capture-off A",
    )?;
    let reports = with
        .sink
        .lock()
        .map_err(|_| invalid("composed address sink poisoned"))?;
    assert_eq!(reports.len(), 1);
    assert_eq!(
        reports[0].credited.to_scalar::<f32>()?,
        2.,
        "actual P/writer utility must trigger the two selected-atom readers"
    );
    drop(reports);

    // Independent CUDA analytic tangent: d[-.1 <raw/sqrt(raw²+eps), c_b>]/draw.
    // Pull this exact raw-address law through the real A projection, so the
    // amplitude check reaches its learned FBS routing points as well.
    let raw = with
        .raw_address
        .detach()
        .narrow(0, 0, 2)?
        .narrow(1, TIME - 1, 1)?;
    let root = fixture.off_home.reshape((1, 1, 4))?;
    let n = raw.sqr()?.sum_keepdim(2)?.affine(1., 1e-12)?.sqrt()?;
    let n3 = n.sqr()?.mul(&n)?;
    let expected = root
        .broadcast_div(&n)?
        .sub(&raw.broadcast_mul(&raw.broadcast_mul(&root)?.sum_keepdim(2)?.div(&n3)?)?)?
        .affine(-0.1, 0.)?;
    let expected = Tensor::cat(
        &[
            &Tensor::zeros((2, TIME - 1, 4), DType::F32, &device)?,
            &expected,
        ],
        1,
    )?;
    let expected = Tensor::cat(
        &[
            &expected,
            &Tensor::zeros((2, TIME, 4), DType::F32, &device)?,
        ],
        0,
    )?;
    let reference_a = Parameters::new(&fixture.a, &fixture.p, 1., &device)?;
    let reference = heads(
        &fixture
            .a
            .forward(reference_a.a.as_tensor(), &with.input.detach())?,
    )?
    .mul(&expected)?
    .sum_all()?
    .backward()?;
    let want = gradient(&reference, &reference_a.a)?;
    finite_nonzero(&want, "analytic A amplitude")?;
    near(
        &gradient(&go, &on.a)?,
        &want,
        1e-5,
        "per-reader .1 address amplitude through A",
    )?;

    let zero = Parameters::new(&fixture.a, &fixture.p, 0., &device)?;
    let zero_fixture = Fixture::new(&device, 0.)?;
    let zero_read = zero_fixture.run(&zero, true)?;
    near(
        &zero_read.delta,
        &zero_read.delta.zeros_like()?,
        0.,
        "K0 read identity",
    )?;
    near(
        &zero_read.query,
        &zero_read
            .input
            .narrow(1, TIME - 1, 1)?
            .reshape((1, WIDTH))?,
        0.,
        "K0 P residual identity",
    )?;
    let gz = zero_read.loss.backward()?;
    for (label, parameter) in [
        ("K0 A", &zero.a),
        ("K0 value", &zero.value),
        ("K0 P", &zero.p),
    ] {
        near(
            &gradient(&gz, parameter)?,
            &parameter.as_tensor().zeros_like()?,
            0.,
            label,
        )?;
    }
    finite_nonzero(
        &gradient(&gz, &zero.coupling)?,
        "K0 wake-up through P/writer",
    )?;
    let reports = zero_read
        .sink
        .lock()
        .map_err(|_| invalid("composed address sink poisoned"))?;
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].credited.to_scalar::<f32>()?, 0.);
    Ok(())
}
