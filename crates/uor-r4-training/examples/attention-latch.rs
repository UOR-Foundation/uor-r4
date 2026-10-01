//! Variable-gap association on the existing geometric stack; no language claim.
use candle_core::{DType, Device, Tensor};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::geometric_stack::{
    ReadBinding, ReadBindingTarget, ReadIdentityLatch, ReadScore, StackAdamW, StackArch,
    StackConfig, StackModel,
};
use uor_r4_training::{Result, TrainingError};
fn invalid(s: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(s.into())
}
struct Rng(u64);
impl Rng {
    fn next(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct Episode {
    ids: Vec<u32>,
    roles: Vec<String>,
    source: usize,
    query: usize,
    answer: u32,
    facts: usize,
    pair: usize,
    condition: String,
    write_gaps: Vec<usize>,
    query_gap: usize,
    target_write_gap: usize,
}
fn push(ids: &mut Vec<u32>, roles: &mut Vec<String>, id: u32, role: &str) {
    ids.push(id);
    roles.push(role.into());
}
fn episode(
    facts: &[(u32, u32)],
    write_noise: &[Vec<u32>],
    query_key: u32,
    query_noise: &[u32],
    pair: usize,
    condition: &str,
) -> Episode {
    let (mut ids, mut roles) = (Vec::new(), Vec::new());
    push(&mut ids, &mut roles, 36, "bos");
    let (mut source, mut answer) = (0, 0);
    let mut target_write_gap = 0;
    for (i, &(key, value)) in facts.iter().enumerate() {
        push(&mut ids, &mut roles, 32, "fact_marker");
        push(&mut ids, &mut roles, key, "write_key");
        for &noise in &write_noise[i] {
            push(&mut ids, &mut roles, 37, "noise_marker");
            push(&mut ids, &mut roles, noise, "noise_key");
        }
        push(&mut ids, &mut roles, 35, "value_marker");
        push(&mut ids, &mut roles, value, "value");
        if key == query_key {
            source = ids.len() - 1;
            answer = value;
            target_write_gap = write_noise[i].len();
        }
    }
    push(&mut ids, &mut roles, 33, "query_marker");
    push(&mut ids, &mut roles, query_key, "query_key");
    for &noise in query_noise {
        push(&mut ids, &mut roles, 37, "noise_marker");
        push(&mut ids, &mut roles, noise, "noise_key");
    }
    push(&mut ids, &mut roles, 34, "answer_marker");
    let query = ids.len() - 1;
    Episode {
        ids,
        roles,
        source,
        query,
        answer,
        facts: facts.len(),
        pair,
        condition: condition.into(),
        write_gaps: write_noise.iter().map(Vec::len).collect(),
        query_gap: query_noise.len(),
        target_write_gap,
    }
}
fn draw(rng: &mut Rng, n: usize) -> Vec<(u32, u32)> {
    let mut keys: Vec<u32> = (0..16).collect();
    for i in (1..16).rev() {
        let j = rng.next(i + 1);
        keys.swap(i, j);
    }
    keys[..n]
        .iter()
        .map(|&k| (k, 16 + rng.next(16) as u32))
        .collect()
}
fn noise(rng: &mut Rng, max: usize) -> Vec<u32> {
    let n = rng.next(max + 1);
    (0..n).map(|_| rng.next(16) as u32).collect()
}
fn evaluation(seed: u64, max_gap: usize) -> Vec<Episode> {
    let mut rng = Rng(seed);
    let mut out = Vec::new();
    for n in [2, 4] {
        for p in 0..16 {
            let facts = draw(&mut rng, n);
            let q = rng.next(n);
            let wn: Vec<_> = (0..n).map(|_| noise(&mut rng, max_gap)).collect();
            let qn = noise(&mut rng, max_gap);
            let pair = n * 100 + p;
            out.push(episode(&facts, &wn, facts[q].0, &qn, pair, "base"));
            out.push(episode(
                &facts,
                &wn,
                facts[(q + 1) % n].0,
                &qn,
                pair,
                "changed_query",
            ));
            let mut changed = facts.clone();
            changed[q].1 = facts[(q + 1) % n].1;
            changed[(q + 1) % n].1 = facts[q].1;
            out.push(episode(
                &changed,
                &wn,
                facts[q].0,
                &qn,
                pair,
                "swapped_values",
            ));
            let mut reversed = facts.clone();
            reversed.reverse();
            let mut rn = wn.clone();
            rn.reverse();
            out.push(episode(
                &reversed,
                &rn,
                facts[q].0,
                &qn,
                pair,
                "reversed_order",
            ));
        }
    }
    out
}
fn batch(es: &[Episode]) -> (Vec<u32>, Vec<u32>, Vec<f32>, usize) {
    let time = es.iter().map(|e| e.ids.len()).max().unwrap_or(0);
    let (mut ids, mut targets, mut weights) = (
        vec![38; es.len() * time],
        vec![0; es.len() * time],
        vec![0.; es.len() * time],
    );
    for (b, e) in es.iter().enumerate() {
        ids[b * time..b * time + e.ids.len()].copy_from_slice(&e.ids);
        targets[b * time + e.query] = e.answer;
        weights[b * time + e.query] = 1.;
    }
    (ids, targets, weights, time)
}
fn binding(es: &[Episode], head: usize) -> ReadBindingTarget {
    ReadBindingTarget {
        layer: 2,
        head,
        rows: es
            .iter()
            .enumerate()
            .map(|(batch, e)| ReadBinding {
                batch,
                query: e.query,
                sources: vec![e.source],
            })
            .collect(),
    }
}
fn score(model: &StackModel, es: &[Episode]) -> Result<Vec<Value>> {
    let mut rows = Vec::new();
    for group in es.chunks(32) {
        let (ids, _, _, time) = batch(group);
        let predictions = model
            .forward(&ids, group.len(), time)?
            .argmax(candle_core::D::Minus1)?
            .to_vec1::<u32>()?;
        let a = model
            .read_binding_masses(&ids, group.len(), time, &binding(group, 0))?
            .to_vec1::<f32>()?;
        let b = model
            .read_binding_masses(&ids, group.len(), time, &binding(group, 1))?
            .to_vec1::<f32>()?;
        for (i, e) in group.iter().enumerate() {
            let prediction = predictions[i * time + e.query];
            rows.push(json!({"pair":e.pair,"condition":e.condition,"facts":e.facts,"query":e.query,"source":e.source,"answer":e.answer,"prediction":prediction,"answer_correct":prediction==e.answer,"source_masses_head0_head1":[a[i],b[i]],"source_majorities_head0_head1":[a[i]>0.5,b[i]>0.5],"write_gaps":e.write_gaps,"query_gap":e.query_gap,"target_write_gap":e.target_write_gap,"max_write_gap":e.write_gaps.iter().max()}));
        }
    }
    Ok(rows)
}
fn ablate(model: &StackModel, es: &[Episode]) -> Result<Vec<Value>> {
    let names = [
        "layers.02.read.query_identity.weight",
        "layers.02.read.key_identity.weight",
    ];
    let mut original = Vec::new();
    for name in names {
        let weight = model
            .variables()
            .get(name)
            .ok_or_else(|| invalid("identity weight missing"))?;
        original.push(weight.as_tensor().copy()?);
    }
    for name in names {
        let w = model
            .variables()
            .get(name)
            .ok_or_else(|| invalid("identity weight missing"))?;
        w.set(&Tensor::zeros(w.shape(), DType::F32, model.device())?)?;
    }
    let result = score(model, es);
    for (name, value) in names.iter().zip(&original) {
        model
            .variables()
            .get(*name)
            .ok_or_else(|| invalid("identity weight missing"))?
            .set(value)?;
    }
    result
}
fn gate_roles(model: &StackModel, es: &[Episode]) -> Result<Value> {
    let mut sums = std::collections::BTreeMap::<String, (usize, f64, f32, f32)>::new();
    for group in es.chunks(32) {
        let (ids, _, _, time) = batch(group);
        let gates = model
            .read_identity_latch_gates(&ids, group.len(), time, 2)?
            .to_vec2::<f32>()?;
        for (e, g) in group.iter().zip(gates) {
            for (role, v) in e.roles.iter().zip(g) {
                let x = sums.entry(role.clone()).or_insert((0, 0., 1., 0.));
                x.0 += 1;
                x.1 += f64::from(v);
                x.2 = x.2.min(v);
                x.3 = x.3.max(v);
            }
        }
    }
    Ok(json!(sums
        .into_iter()
        .map(|(role, (count, sum, min, max))| (
            role,
            json!({"count":count,"mean":sum/count as f64,"min":min,"max":max})
        ))
        .collect::<std::collections::BTreeMap<_, _>>()))
}
fn run(out: &Path, steps: usize, max_seconds: u64) -> Result<Value> {
    let start = Instant::now();
    let eval = evaluation(2110173, 4);
    let stress = evaluation(3110173, 8);
    fs::write(
        out.join("evaluation.json"),
        serde_json::to_vec_pretty(&eval)?,
    )?;
    fs::write(out.join("stress.json"), serde_json::to_vec_pretty(&stress)?)?;
    let mut reports = Vec::new();
    for seed in [1, 2] {
        for mode in [ReadIdentityLatch::Local, ReadIdentityLatch::Held] {
            if start.elapsed().as_secs() >= max_seconds {
                break;
            }
            let name = format!("{mode:?}-s{seed}");
            let root = out.join(&name);
            fs::create_dir(&root)?;
            let config = StackConfig {
                arch: StackArch::Geometric,
                vocab_size: 40,
                width: 32,
                heads: 2,
                mlp_hidden: 64,
                context: 128,
                pattern: "rra".into(),
                read: ReadScore::Lorentz,
                rotation: true,
                seed,
                memory: None,
                select: None,
                pointer: None,
            };
            let mut model = StackModel::new(config, &Device::Cpu)?;
            model.set_read_identity_latch(mode)?;
            let mut optimizer = StackAdamW::new(&model, 0., 1.)?;
            let mut rng = Rng(820019 + seed);
            let mut history = Vec::new();
            let mut done = 0;
            let mut input_tokens = 0;
            let mut padded_tokens = 0;
            for step in 0..steps {
                if start.elapsed().as_secs() >= max_seconds {
                    break;
                }
                let n = [2, 4][step % 2];
                let es: Vec<_> = (0..16)
                    .map(|_| {
                        let facts = draw(&mut rng, n);
                        let q = rng.next(n);
                        let wn: Vec<_> = (0..n).map(|_| noise(&mut rng, 4)).collect();
                        let qn = noise(&mut rng, 4);
                        episode(&facts, &wn, facts[q].0, &qn, 0, "train")
                    })
                    .collect();
                let (ids, targets, weights, time) = batch(&es);
                input_tokens += es.iter().map(|e| e.ids.len()).sum::<usize>();
                padded_tokens += 16 * time;
                let loss = model.weighted_loss(&ids, &targets, &weights, 16, time)?;
                let value = loss.to_scalar::<f32>()?;
                let gradients = loss.backward()?;
                let mut context_gradients = serde_json::Map::new();
                if step % 40 == 0 || step + 1 == steps {
                    for part in [
                        "identity_gate.weight",
                        "identity_gate.bias",
                        "query_identity.weight",
                        "key_identity.weight",
                    ] {
                        let name = format!("layers.02.read.{part}");
                        let variable = model
                            .variables()
                            .get(&name)
                            .ok_or_else(|| invalid("missing context variable"))?;
                        let norm = gradients
                            .get(variable.as_tensor())
                            .map(|g| -> Result<f32> {
                                Ok(g.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?)
                            })
                            .transpose()?;
                        context_gradients.insert(part.into(), json!(norm));
                    }
                }
                let norm = optimizer.update(&model, &gradients, 0.003)?;
                done = step + 1;
                if step % 40 == 0 || done == steps {
                    history.push(json!({"step":done,"answer_nll":value,"gradient_norm":norm,"pure_answer_context_gradient_l2":context_gradients,"time":time,"elapsed_seconds":start.elapsed().as_secs_f64()}));
                }
            }
            model.save(&root.join("model"))?;
            let reload = StackModel::load(&root.join("model"), &Device::Cpu)?;
            let e = &eval[0];
            let before = model
                .forward(&e.ids, 1, e.ids.len())?
                .flatten_all()?
                .to_vec1::<f32>()?;
            let after = reload
                .forward(&e.ids, 1, e.ids.len())?
                .flatten_all()?
                .to_vec1::<f32>()?;
            let measured = if start.elapsed().as_secs() < max_seconds {
                Some(
                    json!({"rows":score(&reload,&eval)?,"identity_disabled_rows":ablate(&reload,&eval)?,"stress_rows":score(&reload,&stress)?,"gate_roles":gate_roles(&reload,&eval)?}),
                )
            } else {
                None
            };
            let report = json!({"name":name,"seed":seed,"mode":format!("{mode:?}"),"steps":done,"requested_steps":steps,"config":model.config,"parameters":model.variables().values().map(|v|v.elem_count()).sum::<usize>(),"unpadded_training_positions":input_tokens,"padded_training_positions":padded_tokens,"history":history,"measured":measured,"reload_logits_identical_fixed_episode":before==after,"model_sha256":uor_r4_training::sha256_file(&root.join("model/model.safetensors"))?});
            fs::write(
                root.join("result.json"),
                serde_json::to_vec_pretty(&report)?,
            )?;
            reports.push(report);
        }
    }
    Ok(
        json!({"schema":"uor-r4.attention-latch-experiment/1","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE"),"reports":reports,"steps":steps,"max_seconds":max_seconds,"elapsed_seconds":start.elapsed().as_secs_f64(),"training_gaps":"independent uniform 0..4 noise pairs per write/query; ordinary final-answer loss only","evaluation":"32 base groups, four correlated interventions; fresh development 0..4 and postfit 0..8 gap draws, not certified final holdout","control":"same gate and current/identity projections; Local retains only preceding g*u, Held accumulates causal convex state","context":"ceiling128, training actual<=60, stress actual<=100, width32; full causal support","scope":"synthetic capture/hold/rebind, no explicit clear, language, integer serving, geometry superiority or energy qualification"}),
    )
}
fn main() -> Result<()> {
    let mut out = None;
    let mut steps = 320;
    let mut seconds = 1500;
    for arg in std::env::args().skip(1) {
        let (k, v) = arg.split_once('=').ok_or_else(|| invalid("key=value"))?;
        match k {
            "out" => out = Some(std::path::PathBuf::from(v)),
            "steps" => steps = v.parse().map_err(|_| invalid("steps"))?,
            "max_seconds" => seconds = v.parse().map_err(|_| invalid("seconds"))?,
            _ => return Err(invalid("unknown argument")),
        }
    }
    if steps == 0 || steps > 2000 || seconds == 0 || seconds > 3600 {
        return Err(invalid("steps1..2000,seconds1..3600"));
    }
    let out = out.ok_or_else(|| invalid("out required"))?;
    report_output::claim(&out)?;
    let result = run(&out, steps, seconds);
    match &result {
        Ok(r) => fs::write(out.join("report.json"), serde_json::to_vec_pretty(r)?)?,
        Err(e) => fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error":e.to_string()}))?,
        )?,
    };
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result?;
    Ok(())
}
