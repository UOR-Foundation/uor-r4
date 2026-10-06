//! CUDA-only one-update admission for joint H4 context and native vocabulary
//! prediction. Fixed synthetic next-symbol rows are an execution instrument,
//! not conversation data or learned language qualification.
use candle_core::{Device, Tensor, Var};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Component, Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    native_geometric::learner::geometric_generate::{GenerateReadCounts, NativeGeometricGenerate},
    report_output,
};
use uor_r4_integer::{
    geometric_context::NativeContextState,
    geometric_context_q4::{ContextQ4Config, NativeContextQ4},
    geometric_source_actions::SourceActionBinding,
    geometric_vocabulary_actions::NativeVocabularyActions,
    h4_tables::{H4Code, HistoricalH4Tables},
};
use uor_r4_training::{
    geometric_context::{ContextWeights, PreparedContextQ4},
    geometric_generate_learning::{vocabulary_marginal_loss, GenerateLearningWeights},
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
const VOCAB: usize = 4096;
const CAP: u64 = 64 << 20;
const MAX_SECONDS: f64 = 600.;
const CONFIG: ContextQ4Config = ContextQ4Config {
    vocab_size: VOCAB,
    heads: 1,
    lanes_per_head: 2,
};
const GEOMETRY: &[u8] = include_bytes!("../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");
#[derive(Debug)]
struct Args {
    out: PathBuf,
    exp: PathBuf,
    seed: u64,
    steps: usize,
}
fn bad(msg: &str) -> Box<dyn Error> {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, msg).into()
}
fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn args() -> Result<Args> {
    let mut fields = BTreeMap::new();
    let mut it = std::env::args().skip(1);
    while let Some(k) = it.next() {
        if !["--out", "--exp", "--seed", "--steps", "--device"].contains(&k.as_str()) {
            return Err(bad("unknown argument"));
        }
        let v = it.next().ok_or_else(|| bad("missing argument value"))?;
        if fields.insert(k, v).is_some() {
            return Err(bad("duplicate argument"));
        }
    }
    if fields.get("--device").map(String::as_str) != Some("cuda") {
        return Err(bad("explicit --device cuda required; no CPU fallback"));
    }
    let out = PathBuf::from(
        fields
            .remove("--out")
            .ok_or_else(|| bad("--out required"))?,
    );
    let exp = PathBuf::from(
        fields
            .remove("--exp")
            .ok_or_else(|| bad("--exp canonical table required"))?,
    );
    let seed = fields
        .get("--seed")
        .ok_or_else(|| bad("--seed required"))?
        .parse()?;
    let steps = fields
        .get("--steps")
        .map_or(Ok(1), |s| s.parse::<usize>())?;
    validate(&out, &exp, steps)?;
    Ok(Args {
        out,
        exp,
        seed,
        steps,
    })
}
fn validate(out: &Path, exp: &Path, steps: usize) -> Result<()> {
    if !(1..=16).contains(&steps)
        || !out.is_absolute()
        || !exp.is_absolute()
        || out
            .components()
            .chain(exp.components())
            .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(bad("absolute normalized paths and steps1..16 required"));
    }
    if exp.starts_with(out) || out.starts_with(exp) {
        return Err(bad("output/input overlap"));
    }
    if out.ancestors().any(|a| a.join("manifest.json").exists()) {
        return Err(bad("output cannot nest inside sealed report"));
    }
    Ok(())
}
#[cfg(feature = "cuda")]
fn cuda() -> Result<Device> {
    Ok(Device::new_cuda(0)?)
}
#[cfg(not(feature = "cuda"))]
fn cuda() -> Result<Device> {
    Err(bad("cuda feature required; no CPU fallback"))
}
fn tokenizer() -> Result<Vec<u8>> {
    let mut vocab = serde_json::Map::new();
    for id in 0..VOCAB {
        let s = match id {
            0 => "<|bos|>".into(),
            1 => "<|eos|>".into(),
            2 => "<|unk|>".into(),
            3 => ".".into(),
            _ => format!("symbol{id}"),
        };
        vocab.insert(s, json!(id));
    }
    Ok(serde_json::to_vec(
        &json!({"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":vocab,"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}),
    )?)
}
fn rows() -> Vec<(Vec<u32>, u32)> {
    (0..8)
        .map(|i| (vec![10 + i, 30 + i, 50 + i], 400 + i))
        .collect()
}
fn size(path: &Path) -> Result<u64> {
    let mut n = 0;
    for e in fs::read_dir(path)? {
        let p = e?.path();
        n += if p.is_dir() {
            size(&p)?
        } else {
            fs::metadata(p)?.len()
        };
    }
    Ok(n)
}
fn write(root: &Path, name: &str, v: &Value) -> Result<()> {
    let b = serde_json::to_vec_pretty(v)?;
    if size(root)? + b.len() as u64 > CAP {
        return Err(bad("report cap exceeded"));
    }
    fs::write(root.join(name), b)?;
    Ok(())
}
fn parameters(c: &ContextWeights, g: &GenerateLearningWeights) -> BTreeMap<String, Var> {
    let mut p = g.parameters();
    for (n, v) in c.parameters() {
        if n.ends_with("_transition") {
            p.insert(format!("context.{n}"), v.clone());
        }
    }
    p
}
fn seeded_context(seed: u64, device: &Device) -> Result<ContextWeights> {
    let c = ContextWeights::new_finite_choice(VOCAB, 8, 1, seed)?.into_q4()?;
    let mut rng = seed ^ 0x8c372be05974a21d;
    if rng == 0 {
        rng = 1;
    }
    for (name, var) in c.parameters() {
        if name.ends_with("_transition") {
            let values = (0..var.elem_count())
                .map(|_| {
                    rng ^= rng << 13;
                    rng ^= rng >> 7;
                    rng ^= rng << 17;
                    ((rng % 3) as i32 - 1) as f32 * 0.25
                })
                .collect::<Vec<_>>();
            var.set(&Tensor::from_vec(values, var.shape(), var.device())?)?;
        }
    }
    Ok(c.to_device(device)?)
}
fn state_graph(
    prepared: &PreparedContextQ4<'_>,
    ids: &[u32],
) -> Result<(Vec<H4Code>, Tensor, Value)> {
    let out = prepared.forward(ids, 1, ids.len(), false)?;
    let codes = out
        .trace
        .states
        .last()
        .ok_or_else(|| bad("missing final state"))?
        .iter()
        .map(|&x| H4Code::try_from(x))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let logits = out
        .state_logits
        .narrow(1, ids.len() - 1, 1)?
        .reshape((2, 120))?;
    Ok((
        codes,
        logits,
        json!({"tokens":ids,"states":out.trace.states,"actions":out.trace.actions}),
    ))
}
fn peak_rss_kib() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find(|l| l.starts_with("VmHWM:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}
fn shadow_receipt(root: &Path, label: &str, params: &BTreeMap<String, Var>) -> Result<Value> {
    let dir = root.join(format!("shadows-{label}"));
    fs::create_dir(&dir)?;
    let mut receipts = BTreeMap::new();
    for (name, var) in params {
        let values = var.flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|v| !v.is_finite()) {
            return Err(bad("nonfinite source shadow"));
        }
        let bytes = values
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>();
        if size(root)? + bytes.len() as u64 > CAP {
            return Err(bad("source shadows exceed report cap"));
        }
        fs::write(dir.join(format!("{name}.f32le")), &bytes)?;
        receipts.insert(
            name.clone(),
            json!({"sha256":hash(&bytes),"bytes":bytes.len(),"shape":var.dims()}),
        );
    }
    Ok(json!(receipts))
}
fn checkpoint(
    root: &Path,
    label: &str,
    c: &ContextWeights,
    g: &GenerateLearningWeights,
    binding: &SourceActionBinding,
) -> Result<(NativeContextQ4, NativeGeometricGenerate, Value)> {
    let packed = c.packed_coefficients()?;
    let native = g.export_native()?;
    let bytes = native.to_bytes()?;
    fs::write(root.join(format!("context-{label}-q4.bin")), &packed)?;
    fs::write(root.join(format!("generate-{label}.bin")), &bytes)?;
    // Independent disk bytes are decoded, not reused in-memory native objects.
    let re_context = NativeContextQ4::new(
        CONFIG,
        &fs::read(root.join(format!("context-{label}-q4.bin")))?,
    )?;
    let re_generate = NativeGeometricGenerate::from_bytes(
        &fs::read(root.join(format!("generate-{label}.bin")))?,
        binding,
    )?;
    if re_context.packed_coefficients() != packed || re_generate.to_bytes()? != bytes {
        return Err(bad("independent checkpoint roundtrip mismatch"));
    }
    let v = json!({"context_config":CONFIG,"context_packed_sha256":hash(&packed),"context_packed_bytes":packed.len(),"context_zero_coefficients":uor_r4_integer::geometric_context_q4::unpack_coefficients(CONFIG.coefficient_count()?,&packed)?.iter().filter(|&&q|q==0).count(),"generate_sha256":hash(&bytes),"generate_bytes":bytes.len(),"generate_metadata":native.metadata(),"independent_disk_reload":true});
    Ok((re_context, re_generate, v))
}
fn context_changes(a: &NativeContextQ4, b: &NativeContextQ4) -> Result<Value> {
    let count = CONFIG.coefficient_count()?;
    let aa =
        uor_r4_integer::geometric_context_q4::unpack_coefficients(count, a.packed_coefficients())?;
    let bb =
        uor_r4_integer::geometric_context_q4::unpack_coefficients(count, b.packed_coefficients())?;
    let mut offset = 0;
    let mut out = BTreeMap::new();
    for (name, shape) in CONFIG.coefficient_shapes()? {
        let n: usize = shape.iter().product();
        let changed = aa[offset..offset + n]
            .iter()
            .zip(&bb[offset..offset + n])
            .filter(|(x, y)| x != y)
            .count();
        if !name.ends_with("_transition") && changed != 0 {
            return Err(bad("unused observation family changed"));
        }
        let frozen = !name.ends_with("_transition");
        out.insert(
            name,
            json!({"coefficients":n,"changed":changed,"frozen":frozen}),
        );
        offset += n;
    }
    Ok(json!(out))
}
fn generate_changes(a: &NativeGeometricGenerate, b: &NativeGeometricGenerate) -> Result<Value> {
    let mut unary = 0;
    let mut pair = 0;
    for lane in 0..2 {
        for r in 0..120 {
            unary += usize::from(a.energy().get_unary(lane, r)? != b.energy().get_unary(lane, r)?);
        }
    }
    for e in 0..a.energy().edges().len() {
        for x in 0..120 {
            for y in 0..120 {
                pair += usize::from(a.energy().get_pair(e, x, y)? != b.energy().get_pair(e, x, y)?);
            }
        }
    }
    Ok(
        json!({"unary_coefficients_changed":unary,"pair_coefficients_changed":pair,"bias_packed_bytes_changed":a.packed_biases().iter().zip(b.packed_biases()).filter(|(x,y)|x!=y).count()}),
    )
}
fn rollout(
    context: &NativeContextQ4,
    g: &NativeGeometricGenerate,
    binding: &SourceActionBinding,
    exp: &[u8],
    prompt: &[u32],
) -> Result<Value> {
    let geometry = HistoricalH4Tables::from_bytes(GEOMETRY)?;
    let mut state = NativeContextState::new(1, 2)?;
    for &id in prompt {
        state.step(id as usize, context.native(), &geometry)?;
    }
    let mut pool = NativeVocabularyActions::new(binding.clone(), exp)?;
    let mut scores = vec![0; VOCAB];
    let mut tokens = Vec::new();
    let mut evidence = Vec::new();
    let mut eos = false;
    for _ in 0..32 {
        let before = state.states().iter().map(|s| s.index()).collect::<Vec<_>>();
        let mut counts = GenerateReadCounts::default();
        g.score_into(state.states(), &mut scores, &mut counts)?;
        let trace = pool.reduce_trace(&scores, &[], &[])?;
        let id = trace.summary.chosen_token_id;
        tokens.push(id);
        evidence.push(json!({"own_prefix_ids":&tokens[..tokens.len()-1],"states_before":before,"native_score_sha256":hash(&scores.iter().flat_map(|s|s.to_le_bytes()).collect::<Vec<_>>()),"pool":trace.summary,"counts":counts}));
        // Feed the actually emitted token even when it is EOS; never target feedback.
        state.step(id as usize, context.native(), &geometry)?;
        if id == binding.eos_token_id() {
            eos = true;
            break;
        }
    }
    Ok(
        json!({"prompt_ids":prompt,"generated_ids":tokens,"eos":eos,"steps":evidence,"scope":"target-free integer autoregression; no Copy candidates/no claimed language"}),
    )
}
fn run(a: &Args, start: Instant) -> Result<Value> {
    let device = cuda()?;
    let exp = fs::read(&a.exp)?;
    let tok = tokenizer()?;
    let binding = SourceActionBinding::new(&tok)?;
    let mut pool = NativeVocabularyActions::new(binding.clone(), &exp)?;
    if binding.vocab_size() != VOCAB || pool.legal_token_ids().len() != VOCAB {
        return Err(bad("fixed dense4096 binding failed"));
    }
    fs::write(a.out.join("tokenizer.json"), &tok)?;
    let data = rows();
    write(
        &a.out,
        "frozen-inputs.json",
        &json!({"scope":"synthetic next-symbol update instrument","source_only_prompts":data.iter().map(|r|&r.0).collect::<Vec<_>>(),"targets_training_labels_only":data.iter().map(|r|r.1).collect::<Vec<_>>(),"copy_sources":[],"draw":"fixed-eight/1;not seed-dependent data"}),
    )?;
    let context = seeded_context(a.seed, &device)?;
    let generate = GenerateLearningWeights::seeded(binding.clone(), 2, a.seed, &device)?;
    let params = parameters(&context, &generate);
    let initial_shadows = shadow_receipt(&a.out, "initial", &params)?;
    let mut optimizer = AdamW::new(
        params.values().cloned().collect(),
        ParamsAdamW {
            lr: 0.2,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.,
        },
    )?;
    let export_start = Instant::now();
    let (initial_context, initial_generate, initial) =
        checkpoint(&a.out, "initial", &context, &generate, &binding)?;
    let initial_export_seconds = export_start.elapsed().as_secs_f64();
    let cpu_context = seeded_context(a.seed, &Device::Cpu)?;
    let cpu_prepared = cpu_context.prepare_q4(&initial_context)?;
    let cpu_generate =
        GenerateLearningWeights::from_native(binding.clone(), &initial_generate, &Device::Cpu)?;
    let cpu_snapshot = cpu_generate.prepare_native()?;
    let (cpu_codes, cpu_logits, cpu_state) = state_graph(&cpu_prepared, &data[0].0)?;
    let cpu_out = cpu_generate.forward_prepared(&cpu_snapshot, &cpu_codes, &cpu_logits)?;
    let cpu_trace = pool.reduce_trace(&cpu_out.scores_q24, &[], &[])?;
    let mut conditional = [0i64; 120];
    initial_generate.state_conditional_scores_into(
        &cpu_codes,
        0,
        0,
        &mut conditional,
        &mut GenerateReadCounts::default(),
    )?;
    if conditional.iter().all(|&x| x == conditional[0]) {
        return Err(bad("seeded decoder is not state-sensitive"));
    }
    let state_sensitivity = json!({"token_id":0,"lane":0,"min_q24":conditional.iter().min(),"max_q24":conditional.iter().max(),"scope":"target-free fixed-token conditional native score; not prediction quality"});
    let cpu_loss = vocabulary_marginal_loss(&cpu_trace, &cpu_out.raw_scores, None, data[0].1)?
        .to_scalar::<f32>()?;
    let mut updates = Vec::new();
    let mut initial_gpu_raw = Vec::new();
    for update in 0..a.steps {
        if start.elapsed().as_secs_f64() > MAX_SECONDS {
            return Err(bad("600 second update instrument bound"));
        }
        let staging = Instant::now();
        let packed = context.packed_coefficients()?;
        let native_context = NativeContextQ4::new(CONFIG, &packed)?;
        let snapshot = generate.prepare_native()?;
        let prepared = context.prepare_q4(&native_context)?;
        device.synchronize()?;
        let staging_seconds = staging.elapsed().as_secs_f64();
        let forward = Instant::now();
        let mut losses = Vec::new();
        let mut row_reports = Vec::new();
        for (index, (ids, target)) in data.iter().enumerate() {
            let (codes, logits, state) = state_graph(&prepared, ids)?;
            let output = generate.forward_prepared(&snapshot, &codes, &logits)?;
            let trace = pool.reduce_trace(&output.scores_q24, &[], &[])?;
            // Label first enters the loss only after both target-free native stages.
            let loss = vocabulary_marginal_loss(&trace, &output.raw_scores, None, *target)?;
            if update == 0 && index == 0 {
                if codes != cpu_codes
                    || output.scores_q24 != cpu_out.scores_q24
                    || state != cpu_state
                    || trace != cpu_trace
                {
                    return Err(bad("CPU/CUDA hard causal/output/pool parity failed"));
                }
                let gpu_loss = loss.to_scalar::<f32>()?;
                if (gpu_loss - cpu_loss).abs() > 1e-6 {
                    return Err(bad("CPU/CUDA initial loss parity failed"));
                }
            }
            if update == 0 {
                initial_gpu_raw.push(output.scores_q24.clone());
            }
            row_reports.push(json!({"index":index,"actual_prefix_ids":ids,"native_state":state,"target_label_after_native":target,"native_target_mass":trace.token_masses.iter().find(|m|m.token_id==*target).map(|m|m.weight_q31),"pool":trace.summary,"decoder_costs":output.costs}));
            losses.push(loss);
        }
        let mean = Tensor::stack(&losses, 0)?.mean_all()?;
        device.synchronize()?;
        let forward_seconds = forward.elapsed().as_secs_f64();
        let backward = Instant::now();
        let mut gradients = mean.backward()?;
        device.synchronize()?;
        let backward_seconds = backward.elapsed().as_secs_f64();
        let telemetry = Instant::now();
        let mut family = BTreeMap::new();
        let mut norm_terms = Vec::new();
        let mut generate_positive = false;
        let mut context_positive = 0usize;
        for (name, var) in &params {
            if let Some(g) = gradients.get(var.as_tensor()) {
                if !g.device().same_device(&device) {
                    return Err(bad("host gradient fallback"));
                }
                let square = g.sqr()?.sum_all()?;
                let norm = square.sqrt()?.to_scalar::<f32>()?;
                if !norm.is_finite() {
                    return Err(bad("nonfinite gradient"));
                }
                if norm > 0. {
                    if name.starts_with("generate.") {
                        generate_positive = true;
                    } else {
                        context_positive += 1;
                    }
                }
                norm_terms.push(square);
                family.insert(
                    name.clone(),
                    json!({"connected":true,"elements":g.elem_count(),"device":"CUDA","l2":norm}),
                );
            } else {
                family.insert(
                    name.clone(),
                    json!({"connected":false,"elements":var.elem_count(),"l2":0.}),
                );
            }
        }
        if !generate_positive || context_positive != 3 {
            return Err(bad(
                "joint decoder/context gradient positive control failed",
            ));
        }
        let norm = Tensor::stack(&norm_terms, 0)?.sum_all()?.sqrt()?;
        let denominator = norm.clamp(1., 1e30)?;
        for var in params.values() {
            if let Some(g) = gradients.get(var.as_tensor()) {
                let clipped = g.broadcast_div(&denominator)?.detach();
                gradients.insert(var.as_tensor(), clipped);
            }
        }
        device.synchronize()?;
        let telemetry_seconds = telemetry.elapsed().as_secs_f64();
        drop(losses);
        drop(prepared);
        let step = Instant::now();
        optimizer.step(&gradients)?;
        context.project_shadow_range()?;
        generate.project_shadow_range()?;
        device.synchronize()?;
        let optimizer_seconds = step.elapsed().as_secs_f64();
        updates.push(json!({"update":update+1,"batch":8,"mean_loss_before_update":mean.to_scalar::<f32>()?,"snapshot_context_packed_sha256":hash(&packed),"snapshot_generate_metadata":snapshot.native.metadata(),"context_packed_download_bytes":packed.len(),"context_master_export_download_estimate_bytes":context.parameters().values().map(|v|v.elem_count()*4).sum::<usize>()*3,"context_export_scope":"one native codec per update; prepare_q4 validates packed source twice in addition to codec export; explicit host staging cost","generate_master_download_bytes":snapshot.downloaded_master_bytes,"gradient_receipts":family,"gradient_global_l2":norm.to_scalar::<f32>()?,"staging_and_native_snapshot_seconds":staging_seconds,"forward_including_host_oracle_synced_seconds":forward_seconds,"backward_synced_seconds":backward_seconds,"gradient_telemetry_and_device_clip_seconds":telemetry_seconds,"optimizer_projection_synced_seconds":optimizer_seconds,"rows":row_reports}));
        write(&a.out, "updates.json", &json!(updates))?;
    }
    let final_shadow_start = Instant::now();
    let final_shadows = shadow_receipt(&a.out, "final", &params)?;
    let final_shadow_download_seconds = final_shadow_start.elapsed().as_secs_f64();
    let export = Instant::now();
    let (final_context, final_generate, final_receipt) =
        checkpoint(&a.out, "final", &context, &generate, &binding)?;
    let export_seconds = export.elapsed().as_secs_f64();
    let initial_prototypes = initial_generate.prototypes();
    let prototype_changed = initial_prototypes
        .iter()
        .zip(final_generate.prototypes())
        .filter(|(a, b)| a != b)
        .count();
    let mut score_changed_rows = 0;
    let mut generations = Vec::new();
    let geometry = HistoricalH4Tables::from_bytes(GEOMETRY)?;
    for (index, (ids, _)) in data.iter().enumerate() {
        let mut st = NativeContextState::new(1, 2)?;
        for &id in ids {
            st.step(id as usize, final_context.native(), &geometry)?;
        }
        let mut final_scores = vec![0; VOCAB];
        final_generate.score_into(
            st.states(),
            &mut final_scores,
            &mut GenerateReadCounts::default(),
        )?;
        if final_scores != initial_gpu_raw[index] {
            score_changed_rows += 1;
        }
        generations.push(rollout(
            &final_context,
            &final_generate,
            &binding,
            &exp,
            ids,
        )?);
    }
    write(
        &a.out,
        "native-reloaded-generation.json",
        &json!(generations),
    )?;
    let exe = std::env::current_exe()?;
    let exe_sha = hash(&fs::read(exe)?);
    Ok(
        json!({"schema":"uor-r4.geometric-generate-update/1","status":"COMPLETED","scope":"joint CUDA one/few-update execution admission; synthetic next-symbols; not language/chat","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":exe_sha,"device":"CUDA:0","cpu_fallback":false,"seed":a.seed,"initialization":"deterministic nonsemantic transition masters .25*{-1,0,1}; Generate seeded shared potentials/base120 tuples","adamw":{"lr":0.2,"beta1":0.9,"beta2":0.999,"eps":1e-8,"weight_decay":0.0,"global_clip":1.0},"admitted_context_families":["token_transition","self_transition","neighbor_transition"],"unused_observation_families":"six root/category families frozen; not required positive","state_sensitivity":state_sensitivity,"updates":a.steps,"vocabulary":VOCAB,"lanes":2,"batch":8,"source_candidates":0,"initial_source_shadows":initial_shadows,"final_source_shadows":final_shadows,"final_shadow_download_seconds":final_shadow_download_seconds,"peak_host_rss_kib":peak_rss_kib(),"tokenizer_sha256":hash(&tok),"canonical_exp_sha256":hash(&exp),"cpu_reference_rows":1,"cpu_reference_backward":false,"initial":initial,"final":final_receipt,"initial_export_reload_seconds":initial_export_seconds,"final_export_reload_seconds":export_seconds,"native_context_family_changes":context_changes(&initial_context,&final_context)?,"native_generate_coefficient_changes":generate_changes(&initial_generate,&final_generate)?,"prototype_entries_changed":prototype_changed,"context_packed_bytes_changed":initial_context.packed_coefficients().iter().zip(final_context.packed_coefficients()).filter(|(a,b)|a!=b).count(),"generate_native_bytes_changed":initial_generate.to_bytes()?.iter().zip(final_generate.to_bytes()?.iter()).filter(|(a,b)|a!=b).count(),"first_position_score_changed_rows":score_changed_rows,"actual_native_feedback_rollouts":8,"generation_cap":32,"fresh_predictions":0,"selection":"NONE","report_cap_bytes":CAP,"elapsed_seconds":start.elapsed().as_secs_f64(),"limitation":"single update need not cross quarter-grid/prototype boundaries; frozen native identity is not failed learning; no corpus/bench2chat qualification; observation-root/category families not directly consumed by state decoder"}),
    )
}
fn main() -> Result<()> {
    let a = args()?;
    report_output::claim(&a.out)?;
    let start = Instant::now();
    let result = run(&a, start);
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.geometric-generate-update/1","status":"FAILED","error":e.to_string(),"elapsed_seconds":start.elapsed().as_secs_f64(),"fit_verdict":"NOT_QUALIFIED"})
        }
    };
    write(&a.out, "report.json", &report)?;
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn admission_rejects_cpu_overlap_and_bad_steps() {
        assert!(validate(Path::new("/tmp/out"), Path::new("/tmp/out/exp"), 1).is_err());
        assert!(validate(Path::new("out"), Path::new("/tmp/exp"), 1).is_err());
        assert!(validate(Path::new("/tmp/out"), Path::new("/tmp/exp"), 17).is_err());
    }
    #[test]
    fn tiny_prediction_has_no_target_in_prompt() {
        let r = rows();
        assert_eq!(r.len(), 8);
        for (ids, target) in r {
            assert!(!ids.contains(&target));
            assert_eq!(ids.len(), 3);
        }
    }
}
