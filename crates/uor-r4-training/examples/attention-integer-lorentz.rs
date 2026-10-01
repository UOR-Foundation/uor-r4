//! Saved-model integer Lorentz distance replay; no fitting or serving claim.
use candle_core::{Device, Tensor, D};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path, time::Instant};
use uor_r4_core::report_output;
use uor_r4_integer::{
    codec::Grouped4BitRow,
    identity_latch::{DyadicBias, DyadicVector, IdentityGate, IdentityLatch, LatchMode},
    paired_projection::{PairedProjection, PROJECTION_OUTPUT_EXPONENT},
    stack::IntegerLorentzDistance,
};
use uor_r4_training::geometric_stack::{
    ReadBinding, ReadBindingTarget, ReadIdentityLatch, ReadScore, StackArch, StackModel,
};
use uor_r4_training::{sha256_file, Result, TrainingError};
fn invalid(s: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(s.into())
}
#[derive(Deserialize)]
struct Episode {
    ids: Vec<u32>,
    roles: Vec<String>,
    source: usize,
    query: usize,
    answer: u32,
    facts: usize,
    pair: usize,
    condition: String,
    target_write_gap: usize,
    query_gap: usize,
}
#[derive(Serialize, Deserialize)]
struct GateArtifact {
    schema: String,
    model_sha256: String,
    current: Vec<i8>,
    current_exp: i32,
    previous: Vec<i8>,
    previous_exp: i32,
    bias: i64,
    bias_exp: i32,
    max_weight_error: f32,
    policy: String,
}
impl GateArtifact {
    fn gate(&self) -> Result<IdentityGate> {
        if self.schema != "uor-r4.integer-identity-gate/1" {
            return Err(invalid("gate schema"));
        }
        IdentityGate::new(
            self.current.clone(),
            self.current_exp,
            self.previous.clone(),
            self.previous_exp,
            DyadicBias {
                mantissa: self.bias,
                exponent: self.bias_exp,
            },
        )
        .map_err(|e| invalid(e.to_string()))
    }
}
fn export_gate(model: &StackModel, model_sha256: String) -> Result<GateArtifact> {
    let w = model
        .variables()
        .get("layers.02.read.identity_gate.weight")
        .ok_or_else(|| invalid("gateweight"))?
        .flatten_all()?
        .to_vec1::<f32>()?;
    if w.len() != 64 {
        return Err(invalid("gatewidth"));
    }
    let mut halves = Vec::new();
    let mut max_weight_error = 0f32;
    for half in w.chunks(32) {
        let row = Grouped4BitRow::encode_f32(half, 32).map_err(|e| invalid(e.to_string()))?;
        let codes = (0..32)
            .map(|i| {
                let n = (row.packed_codes[i >> 1] >> ((i & 1) << 2)) & 15;
                (n as i8) << 4 >> 4
            })
            .collect::<Vec<_>>();
        for (a, b) in half.iter().zip(row.dequantize_f32()) {
            max_weight_error = max_weight_error.max((a - b).abs());
        }
        halves.push((codes, i32::from(row.group_exponents[0])));
    }
    let b = model
        .variables()
        .get("layers.02.read.identity_gate.bias")
        .ok_or_else(|| invalid("gatebias"))?
        .to_vec1::<f32>()?[0];
    let scaled = f64::from(b) * 2f64.powi(24);
    if !scaled.is_finite() || scaled.abs() >= i64::MAX as f64 {
        return Err(invalid("biasrange"));
    }
    let (previous, previous_exp) = halves.pop().ok_or_else(|| invalid("previoushalf"))?;
    let (current, current_exp) = halves.pop().ok_or_else(|| invalid("currenthalf"))?;
    Ok(GateArtifact{schema:"uor-r4.integer-identity-gate/1".into(),model_sha256,current,current_exp,previous,previous_exp,bias:scaled.round() as i64,bias_exp:-24,max_weight_error,policy:"existing Grouped4BitRow group32 signed[-7,7] dyadic maxabs-only exp[-24,16]; weights/input/bias nearest tiesaway; fixed Q24bias; gained inputs; distinct from stack_export grid".into()})
}

#[derive(Serialize, Deserialize)]
struct RowArtifact {
    group_size: usize,
    group_exponents: Vec<i16>,
    packed_codes: Vec<u8>,
    elements: usize,
}

impl RowArtifact {
    fn row(&self) -> Grouped4BitRow {
        Grouped4BitRow {
            group_size: self.group_size,
            group_exponents: self.group_exponents.clone(),
            packed_codes: self.packed_codes.clone().into(),
            elements: self.elements,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct MatrixArtifact {
    source_tensor: String,
    rows: Vec<RowArtifact>,
    max_weight_error: f32,
    relative_rms_error: f64,
}

impl MatrixArtifact {
    fn rows(&self, expected: &str) -> Result<Vec<Grouped4BitRow>> {
        if self.source_tensor != expected
            || self.rows.len() != 32
            || self.rows.iter().any(|row| row.elements != 32)
        {
            return Err(invalid("projection artifact tensor or shape differs"));
        }
        Ok(self.rows.iter().map(RowArtifact::row).collect())
    }
}

#[derive(Serialize, Deserialize)]
struct ComponentArtifact {
    schema: String,
    model_sha256: String,
    config_sha256: String,
    gate: GateArtifact,
    query_current: MatrixArtifact,
    query_identity: MatrixArtifact,
    key_current: MatrixArtifact,
    key_identity: MatrixArtifact,
    output_exponent: i32,
    policy: String,
}

struct Components {
    gate: IdentityGate,
    query: PairedProjection,
    key: PairedProjection,
}

impl ComponentArtifact {
    fn components(&self) -> Result<Components> {
        if self.schema != "uor-r4.integer-qk-components/1"
            || self.gate.model_sha256 != self.model_sha256
            || self.output_exponent != PROJECTION_OUTPUT_EXPONENT
        {
            return Err(invalid(
                "component artifact schema, binding or output scale",
            ));
        }
        let gate = self.gate.gate()?;
        if gate.width() != 32 {
            return Err(invalid("component gate width differs"));
        }
        let query = PairedProjection::new(
            self.query_current.rows("layers.02.read.query.weight")?,
            self.query_identity
                .rows("layers.02.read.query_identity.weight")?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let key = PairedProjection::new(
            self.key_current.rows("layers.02.read.key.weight")?,
            self.key_identity
                .rows("layers.02.read.key_identity.weight")?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        Ok(Components { gate, query, key })
    }
}

fn export_matrix(model: &StackModel, name: &str) -> Result<MatrixArtifact> {
    let weights = model.variables().get(name).ok_or_else(|| invalid(name))?;
    if weights.dims() != [32, 32] {
        return Err(invalid("projection weight shape differs"));
    }
    let weights = weights.to_vec2::<f32>()?;
    let (mut squared_error, mut energy) = (0f64, 0f64);
    let mut max_weight_error = 0f32;
    let mut rows = Vec::with_capacity(32);
    for values in weights {
        let row = Grouped4BitRow::encode_f32(&values, 32).map_err(|e| invalid(e.to_string()))?;
        for (value, rounded) in values.iter().zip(row.dequantize_f32()) {
            let error = f64::from(*value) - f64::from(rounded);
            squared_error += error * error;
            energy += f64::from(*value) * f64::from(*value);
            max_weight_error = max_weight_error.max((*value - rounded).abs());
        }
        rows.push(RowArtifact {
            group_size: row.group_size,
            group_exponents: row.group_exponents,
            packed_codes: row.packed_codes.to_vec(),
            elements: row.elements,
        });
    }
    Ok(MatrixArtifact {
        source_tensor: name.to_owned(),
        rows,
        max_weight_error,
        relative_rms_error: if energy == 0. {
            0.
        } else {
            (squared_error / energy).sqrt()
        },
    })
}

fn export_components(
    model: &StackModel,
    model_sha256: String,
    config_sha256: String,
) -> Result<ComponentArtifact> {
    Ok(ComponentArtifact {
        schema: "uor-r4.integer-qk-components/1".into(),
        gate: export_gate(model, model_sha256.clone())?,
        model_sha256,
        config_sha256,
        query_current: export_matrix(model, "layers.02.read.query.weight")?,
        query_identity: export_matrix(model, "layers.02.read.query_identity.weight")?,
        key_current: export_matrix(model, "layers.02.read.key.weight")?,
        key_identity: export_matrix(model, "layers.02.read.key_identity.weight")?,
        output_exponent: PROJECTION_OUTPUT_EXPONENT,
        policy: "four saved maps; existing Grouped4BitRow perrow group32 signed[-7,7] dyadic maxabs-only exp[-24,16]; no calibration or answer tuning; gained current and prior input; exact paired accumulation then one Q16 nearest/ties-away rounding; remaining reader float".into(),
    })
}

fn quantize(u: &[f32]) -> Result<(Vec<i16>, i32)> {
    if u.iter().any(|x| !x.is_finite()) {
        return Err(invalid("nonfiniteinput"));
    }
    let max = u.iter().fold(0f64, |a, x| a.max(f64::from(x.abs())));
    let exp = if max == 0. {
        0
    } else {
        (max / 32767.).log2().ceil() as i32
    };
    if !(-64..=64).contains(&exp) {
        return Err(invalid("input exponent"));
    }
    let scale = 2f64.powi(exp);
    let mut out = Vec::new();
    for x in u {
        let a = (f64::from(*x) / scale).round();
        if !(-32767. ..=32767.).contains(&a) {
            return Err(invalid("input clipping"));
        }
        out.push(a as i16);
    }
    Ok((out, exp))
}
#[derive(Clone, Copy)]
enum Replay {
    IntegerQk,
    IntegerDistance,
}

fn score_rows(
    model: &StackModel,
    episodes: &[Episode],
    ids: &[u32],
    time: usize,
    query: &Tensor,
    key: &Tensor,
    distances: &Tensor,
    replay: Replay,
) -> Result<Vec<Value>> {
    let batch = episodes.len();
    let logits = match replay {
        Replay::IntegerQk => model.forward_read_projection_replay(ids, batch, time, query, key)?,
        Replay::IntegerDistance => {
            model.forward_read_distance_replay(ids, batch, time, query, key, distances)?
        }
    };
    let predictions = logits.argmax(D::Minus1)?.to_vec1::<u32>()?;
    let values = logits.to_vec2::<f32>()?;
    if values.iter().flatten().any(|x| !x.is_finite()) {
        return Err(invalid("nonfinite answer logits"));
    }
    let mut masses = Vec::new();
    for head in [0, 1] {
        let target = ReadBindingTarget {
            layer: 2,
            head,
            rows: episodes
                .iter()
                .enumerate()
                .map(|(batch, e)| ReadBinding {
                    batch,
                    query: e.query,
                    sources: vec![e.source],
                })
                .collect(),
        };
        let mass = match replay {
            Replay::IntegerQk => model
                .read_binding_masses_projection_replay(ids, batch, time, &target, query, key)?,
            Replay::IntegerDistance => model.read_binding_masses_distance_replay(
                ids, batch, time, &target, query, key, distances,
            )?,
        }
        .to_vec1::<f32>()?;
        if mass
            .iter()
            .any(|x| !x.is_finite() || !(0. ..=1.).contains(x))
        {
            return Err(invalid("nonfinite or out-of-range source probability"));
        }
        masses.push(mass);
    }
    Ok(episodes.iter().enumerate().map(|(i, e)| {
        let prediction = predictions[i * time + e.query];
        json!({"pair":e.pair,"condition":e.condition,"facts":e.facts,"source":e.source,"query":e.query,"answer":e.answer,"prediction":prediction,"answer_correct":prediction==e.answer,"answer_logits":values[i*time+e.query],"source_masses_head0_head1":[masses[0][i],masses[1][i]],"source_majorities_head0_head1":[masses[0][i]>0.5,masses[1][i]>0.5],"target_write_gap":e.target_write_gap,"query_gap":e.query_gap,"positions":e.ids.len()})
    }).collect())
}

fn float_projection(
    model: &StackModel,
    input: &Tensor,
    prior: &Tensor,
    part: &str,
) -> Result<Tensor> {
    let (batch, time, width) = input.dims3()?;
    let current_name = format!("layers.02.read.{part}.weight");
    let identity_name = format!("layers.02.read.{part}_identity.weight");
    let current = model
        .variables()
        .get(&current_name)
        .ok_or_else(|| invalid(&current_name))?;
    let identity = model
        .variables()
        .get(&identity_name)
        .ok_or_else(|| invalid(&identity_name))?;
    Ok(input
        .reshape((batch * time, width))?
        .matmul(&current.t()?)?
        .add(
            &prior
                .reshape((batch * time, width))?
                .matmul(&identity.t()?)?,
        )?
        .reshape((batch, time, width))?)
}

fn max_error(left: &[f32], right: &[f32]) -> f64 {
    left.iter().zip(right).fold(0f64, |maximum, (a, b)| {
        maximum.max((f64::from(*a) - f64::from(*b)).abs())
    })
}

#[derive(Serialize, Deserialize, PartialEq)]
struct TableArtifact {
    schema: String,
    file: String,
    sha256: String,
    entries: usize,
    coordinate_exponent: i32,
    lift_exponent: i32,
    distance_exponent: i32,
    minimum_excess_code_q32: u128,
    producer: String,
    policy: String,
}

fn export_table(out: &Path) -> Result<(Vec<u32>, TableArtifact)> {
    let table = uor_r4_training::lut_export::arcosh_table();
    let file = "arcosh-q24.le-u32";
    let mut bytes = Vec::with_capacity(table.len() * 4);
    for entry in &table {
        bytes.extend_from_slice(&entry.to_le_bytes());
    }
    let path = out.join(file);
    fs::write(&path, &bytes)?;
    let metadata = TableArtifact {
        schema: "uor-r4.integer-lorentz-table/1".into(),
        file: file.into(),
        sha256: sha256_file(&path)?,
        entries: table.len(),
        coordinate_exponent: -16,
        lift_exponent: -32,
        distance_exponent: -24,
        minimum_excess_code_q32: 429,
        producer: "uor_r4_training::lut_export::arcosh_table".into(),
        policy: "existing octave-grid arcosh(1+excess), LE u32; Q24 nearest table values; integer floor-Q32 lifts, signed Q64 excess, nonnegative floor-Q32 argument clamped to 429; unchanged existing integer stack interpolation".into(),
    };
    let metadata_path = out.join("arcosh-table.json");
    fs::write(&metadata_path, serde_json::to_vec_pretty(&metadata)?)?;
    let reloaded: TableArtifact = serde_json::from_slice(&fs::read(&metadata_path)?)?;
    if reloaded != metadata || sha256_file(&out.join(&reloaded.file))? != reloaded.sha256 {
        return Err(invalid(
            "distance table metadata or hash differs after reload",
        ));
    }
    let loaded_bytes = fs::read(out.join(&reloaded.file))?;
    if loaded_bytes.len() != reloaded.entries * 4 || loaded_bytes != bytes {
        return Err(invalid("distance table bytes differ after reload"));
    }
    let loaded = loaded_bytes
        .chunks_exact(4)
        .map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
        .collect::<Vec<_>>();
    if loaded != table {
        return Err(invalid("distance table entries differ after reload"));
    }
    Ok((loaded, reloaded))
}

// Match geometric_stack::dot's sixteen f32 partial sums for the norm.
// At this fixed head width 16 the product reduction also equals the reader's
// ascending-coordinate f32 tile_product reduction. Do not replace by an f64 dot.
fn reader_dot(a: &[f32], b: &[f32]) -> f32 {
    let mut partial = [0f32; 16];
    for i in 0..16 {
        partial[i] += a[i] * b[i];
    }
    partial.iter().sum()
}

fn reader_distance(excess: f64) -> f64 {
    let e = excess.max(1e-7);
    (e + (e * (e + 2.0)).sqrt()).ln_1p()
}

#[allow(clippy::too_many_arguments)]
fn distance_inputs(
    kernel: &mut IntegerLorentzDistance,
    episodes: &[Episode],
    time: usize,
    query_raw: &[i32],
    key_raw: &[i32],
    query_float: &[Vec<Vec<f32>>],
    key_float: &[Vec<Vec<f32>>],
    device: &Device,
    start: Instant,
    limit: u64,
) -> Result<(Tensor, Vec<Value>)> {
    // These are the actual paired projection codes, before F32 reconstruction.
    // Future entries stay canonical zero; every causal entry includes self.
    if kernel.width() != 16
        || query_raw.len() != episodes.len() * time * 32
        || key_raw.len() != query_raw.len()
    {
        return Err(invalid("distance replay coordinate layout"));
    }
    let mut distances = vec![0f64; episodes.len() * 2 * time * time];
    let mut diagnostics = Vec::new();
    for (b, episode) in episodes.iter().enumerate() {
        let (mut pair_count, mut integer_clamped, mut float_clamped) = (0usize, 0usize, 0usize);
        let (mut clamp_disagreements, mut negative_integer, mut negative_float) =
            (0usize, 0usize, 0usize);
        let mut floor_code_count = 0usize;
        let (mut max_error, mut max_self_error, mut max_nonself_error) = (0f64, 0f64, 0f64);
        let (mut max_excess_error, mut max_lift_error, mut max_float_norm2) = (0f64, 0f64, 0f64);
        let mut worst_pair = Value::Null;
        let mut selected_source_pairs = Vec::new();
        for head in 0..2 {
            if start.elapsed().as_secs() >= limit {
                return Err(invalid("distance evaluation wall ceiling reached"));
            }
            let mut queries = Vec::with_capacity(time);
            let mut keys = Vec::with_capacity(time);
            let mut query_norms = Vec::with_capacity(time);
            let mut key_norms = Vec::with_capacity(time);
            for t in 0..time {
                let offset = (b * time + t) * 32 + head * 16;
                queries.push(
                    kernel
                        .lift(&query_raw[offset..offset + 16])
                        .map_err(|e| invalid(e.to_string()))?,
                );
                keys.push(
                    kernel
                        .lift(&key_raw[offset..offset + 16])
                        .map_err(|e| invalid(e.to_string()))?,
                );
                let q = &query_float[b][t][head * 16..head * 16 + 16];
                let k = &key_float[b][t][head * 16..head * 16 + 16];
                query_norms.push(f64::from(reader_dot(q, q)));
                key_norms.push(f64::from(reader_dot(k, k)));
            }
            for t in 0..time {
                let q = &query_float[b][t][head * 16..head * 16 + 16];
                let query_lift = (1.0 + query_norms[t]).sqrt();
                for j in 0..=t {
                    let result = kernel
                        .evaluate(queries[t], keys[j])
                        .map_err(|e| invalid(e.to_string()))?;
                    let integer_distance = f64::from(result.distance_q24) * 2f64.powi(-24);
                    distances[((b * 2 + head) * time + t) * time + j] = integer_distance;
                    // Padding is needed by the replay tensor, but never counted as evidence.
                    if t >= episode.ids.len() {
                        continue;
                    }
                    let k = &key_float[b][j][head * 16..head * 16 + 16];
                    let key_lift = (1.0 + key_norms[j]).sqrt();
                    let float_dot = f64::from(reader_dot(q, k));
                    let float_excess = query_lift * key_lift - float_dot - 1.0;
                    let float_distance = reader_distance(float_excess);
                    let absolute_error = (integer_distance - float_distance).abs();
                    let integer_excess = result.signed_excess_q64 as f64 * 2f64.powi(-64);
                    let excess_error = (integer_excess - float_excess).abs();
                    let q_lift_error =
                        (queries[t].lift_q32() as f64 * 2f64.powi(-32) - query_lift).abs();
                    let k_lift_error =
                        (keys[j].lift_q32() as f64 * 2f64.powi(-32) - key_lift).abs();
                    if [
                        float_dot,
                        float_excess,
                        float_distance,
                        absolute_error,
                        integer_excess,
                        excess_error,
                        q_lift_error,
                        k_lift_error,
                    ]
                    .iter()
                    .any(|value| !value.is_finite())
                    {
                        return Err(invalid("nonfinite distance diagnostic"));
                    }
                    pair_count += 1;
                    integer_clamped += usize::from(result.clamped);
                    let reference_clamped = float_excess < 1e-7;
                    float_clamped += usize::from(reference_clamped);
                    clamp_disagreements += usize::from(reference_clamped != result.clamped);
                    floor_code_count += usize::from(result.code_q32 == 429);
                    negative_integer += usize::from(result.signed_excess_q64 < 0);
                    negative_float += usize::from(float_excess < 0.);
                    max_excess_error = max_excess_error.max(excess_error);
                    max_lift_error = max_lift_error.max(q_lift_error).max(k_lift_error);
                    max_float_norm2 = max_float_norm2.max(query_norms[t]).max(key_norms[j]);
                    if t == j {
                        max_self_error = max_self_error.max(absolute_error);
                    } else {
                        max_nonself_error = max_nonself_error.max(absolute_error);
                    }
                    let new_worst = worst_pair.is_null() || absolute_error > max_error;
                    let selected_source = t == episode.query && j == episode.source;
                    if new_worst || selected_source {
                        let pair = json!({
                            "head":head,"query_position":t,"source_position":j,"same_position":t==j,
                            "query_q16":queries[t].coordinates(),"key_q16":keys[j].coordinates(),
                            "float_query_norm2":query_norms[t],"float_key_norm2":key_norms[j],
                            "float_query_lift":query_lift,"float_key_lift":key_lift,
                            "integer_query_lift_q32":queries[t].lift_q32(),"integer_key_lift_q32":keys[j].lift_q32(),
                            "float_dot":float_dot,"float_signed_excess":float_excess,
                            "integer_signed_excess_q64":result.signed_excess_q64.to_string(),
                            "integer_code_q32":result.code_q32.to_string(),
                            "float_clamped":reference_clamped,"integer_clamped":result.clamped,
                            "float_distance":float_distance,"integer_distance_q24":result.distance_q24,
                            "integer_distance":integer_distance,"absolute_distance_error":absolute_error,
                            "absolute_excess_error":excess_error
                        });
                        if new_worst {
                            worst_pair = pair.clone();
                        }
                        if selected_source {
                            selected_source_pairs.push(pair);
                        }
                    }
                    max_error = max_error.max(absolute_error);
                }
            }
        }
        if selected_source_pairs.len() != 2 {
            return Err(invalid("missing selected source distance diagnostic"));
        }
        diagnostics.push(json!({
            "pair":episode.pair,"condition":episode.condition,"positions":episode.ids.len(),
            "causal_pairs_both_heads":pair_count,"computed_causal_pairs_including_padding":time*(time+1),
            "integer_clamp_count":integer_clamped,"float_clamp_count":float_clamped,
            "clamp_disagreement_count":clamp_disagreements,"integer_floor_code_count":floor_code_count,
            "negative_integer_excess_count":negative_integer,"negative_float_excess_count":negative_float,
            "max_absolute_distance_error":max_error,"max_same_position_distance_error":max_self_error,
            "max_different_position_distance_error":max_nonself_error,
            "max_absolute_excess_error":max_excess_error,"max_absolute_lift_error":max_lift_error,
            "max_float_norm2":max_float_norm2,"worst_distance_pair":worst_pair,
            "selected_source_pairs_head0_head1":selected_source_pairs
        }));
    }
    Ok((
        Tensor::from_vec(distances, (episodes.len(), 2, time, time), device)?,
        diagnostics,
    ))
}

fn score_panel(
    model: &StackModel,
    episodes: &[Episode],
    components: &mut Components,
    distance_kernel: &mut IntegerLorentzDistance,
    start: Instant,
    limit: u64,
) -> Result<Value> {
    let mut reference_rows = Vec::new();
    let mut integer_rows = Vec::new();
    let mut disabled_rows = Vec::new();
    let mut diagnostic_rows = Vec::new();
    let mut distance_rows = Vec::new();
    for group in episodes.chunks(32) {
        if start.elapsed().as_secs() >= limit {
            return Err(invalid("evaluation wall ceiling reached"));
        }
        let time = group
            .iter()
            .map(|e| e.ids.len())
            .max()
            .ok_or_else(|| invalid("empty batch"))?;
        let mut ids = vec![38; group.len() * time];
        for (i, e) in group.iter().enumerate() {
            ids[i * time..i * time + e.ids.len()].copy_from_slice(&e.ids);
        }
        let input_tensor = model.read_identity_latch_replay_input(&ids, group.len(), time)?;
        let input = input_tensor.to_vec3::<f32>()?;
        let float_gates = model
            .read_identity_latch_gate_logits(&ids, group.len(), time, 2)?
            .to_vec2::<f32>()?;
        let mut prior_values = Vec::with_capacity(group.len() * time * 32);
        let mut query_values = Vec::with_capacity(group.len() * time * 32);
        let mut key_values = Vec::with_capacity(group.len() * time * 32);
        let mut query_raw = Vec::with_capacity(group.len() * time * 32);
        let mut key_raw = Vec::with_capacity(group.len() * time * 32);
        let mut diagnostics = Vec::new();
        let output_scale = 2f64.powi(PROJECTION_OUTPUT_EXPONENT);
        for (b, sequence) in input.iter().enumerate() {
            let mode = if model.read_identity_latch() == Some(ReadIdentityLatch::Held) {
                LatchMode::Held
            } else {
                LatchMode::Local
            };
            let mut latch = IdentityLatch::new(components.gate.clone(), mode)
                .map_err(|e| invalid(e.to_string()))?;
            let mut reference = vec![0f32; 32];
            let mut events = Vec::new();
            for (t, current_float) in sequence.iter().enumerate() {
                let held = latch.identity();
                let held_exp = held.exponent;
                let prior = held
                    .mantissas
                    .iter()
                    .map(|x| (f64::from(*x) * 2f64.powi(held.exponent)) as f32)
                    .collect::<Vec<_>>();
                let content_error = max_error(&prior, &reference);
                prior_values.extend(prior);
                let (mantissas, exponent) = quantize(current_float)?;
                let current = DyadicVector {
                    mantissas: &mantissas,
                    exponent,
                };
                let (mut query_codes, mut key_codes) = ([0i32; 32], [0i32; 32]);
                // Both maps consume the old captured identity before this token's step.
                components
                    .query
                    .project(current, held, &mut query_codes)
                    .map_err(|e| invalid(e.to_string()))?;
                components
                    .key
                    .project(current, held, &mut key_codes)
                    .map_err(|e| invalid(e.to_string()))?;
                query_raw.extend_from_slice(&query_codes);
                key_raw.extend_from_slice(&key_codes);
                query_values.extend(query_codes.map(|x| (f64::from(x) * output_scale) as f32));
                key_values.extend(key_codes.map(|x| (f64::from(x) * output_scale) as f32));
                let decision = latch.step(current).map_err(|e| invalid(e.to_string()))?;
                let integer_logit =
                    decision.logit.mantissa as f64 * 2f64.powi(decision.logit.exponent);
                let capture = float_gates[b][t] >= 0.;
                if capture {
                    reference.copy_from_slice(current_float);
                } else if mode == LatchMode::Local {
                    reference.fill(0.);
                }
                if t < group[b].ids.len() {
                    events.push(json!({"position":t,"role":group[b].roles[t],"reference_capture":capture,"integer_capture":decision.capture,"action_agrees":capture==decision.capture,"float_gate_logit":float_gates[b][t],"integer_gate_logit":integer_logit,"absolute_gate_logit_error":(integer_logit-f64::from(float_gates[b][t])).abs(),"prior_content_max_error":content_error,"prior_exponent":held_exp,"input_exponent":exponent}));
                }
            }
            diagnostics.push(events);
        }
        let shape = (group.len(), time, 32);
        let prior = Tensor::from_vec(prior_values, shape, model.device())?;
        let query = Tensor::from_vec(query_values, shape, model.device())?;
        let key = Tensor::from_vec(key_values, shape, model.device())?;
        let query_float =
            float_projection(model, &input_tensor, &prior, "query")?.to_vec3::<f32>()?;
        let key_float = float_projection(model, &input_tensor, &prior, "key")?.to_vec3::<f32>()?;
        let query_integer = query.to_vec3::<f32>()?;
        let key_integer = key.to_vec3::<f32>()?;
        if [&query_float, &key_float, &query_integer, &key_integer]
            .iter()
            .any(|values| values.iter().flatten().flatten().any(|x| !x.is_finite()))
        {
            return Err(invalid("nonfinite projection values"));
        }
        for (i, e) in group.iter().enumerate() {
            let q_errors = (0..e.ids.len())
                .map(|t| max_error(&query_float[i][t], &query_integer[i][t]))
                .collect::<Vec<_>>();
            let k_errors = (0..e.ids.len())
                .map(|t| max_error(&key_float[i][t], &key_integer[i][t]))
                .collect::<Vec<_>>();
            if q_errors.iter().chain(&k_errors).any(|e| !e.is_finite()) {
                return Err(invalid("nonfinite projection error"));
            }
            diagnostic_rows.push(json!({"pair":e.pair,"condition":e.condition,"positions":e.ids.len(),"max_query_projection_error":q_errors.iter().copied().fold(0f64,f64::max),"max_key_projection_error":k_errors.iter().copied().fold(0f64,f64::max),"answer_query_projection_error":q_errors[e.query],"source_key_projection_error":k_errors[e.source],"query_errors":q_errors,"key_errors":k_errors,"events":diagnostics[i]}));
        }
        let (distances, distance_diagnostics) = distance_inputs(
            distance_kernel,
            group,
            time,
            &query_raw,
            &key_raw,
            &query_integer,
            &key_integer,
            model.device(),
            start,
            limit,
        )?;
        distance_rows.extend(distance_diagnostics);
        let reference = score_rows(
            model,
            group,
            &ids,
            time,
            &query,
            &key,
            &distances,
            Replay::IntegerQk,
        )?;
        let integer = score_rows(
            model,
            group,
            &ids,
            time,
            &query,
            &key,
            &distances,
            Replay::IntegerDistance,
        )?;
        // Explicit read-output ablation, not a forced NoRead probability.
        let output_weight = model
            .variables()
            .get("layers.02.read.out.weight")
            .ok_or_else(|| invalid("read output map"))?;
        let original = output_weight.as_tensor().copy()?;
        output_weight.set(&Tensor::zeros(
            output_weight.shape(),
            candle_core::DType::F32,
            model.device(),
        )?)?;
        let disabled = score_rows(
            model,
            group,
            &ids,
            time,
            &query,
            &key,
            &distances,
            Replay::IntegerDistance,
        );
        output_weight.set(&original)?;
        let disabled = disabled?;
        let after = score_rows(
            model,
            group,
            &ids,
            time,
            &query,
            &key,
            &distances,
            Replay::IntegerQk,
        )?;
        if reference != after {
            return Err(invalid(
                "distance replay or output ablation changed reference",
            ));
        }
        reference_rows.extend(reference);
        integer_rows.extend(integer);
        disabled_rows.extend(disabled);
    }
    Ok(
        json!({"integer_qk_reference":reference_rows,"integer_distance":integer_rows,"read_output_disabled":disabled_rows,"projection_diagnostics":diagnostic_rows,"distance_diagnostics":distance_rows,"reference_after_identical":true}),
    )
}

fn run(out: &Path, parent: &Path, limit: u64) -> Result<Value> {
    let start = Instant::now();
    let (table, table_artifact) = export_table(out)?;
    let mut panels = Vec::new();
    for file in ["evaluation.json", "stress.json"] {
        let bytes = fs::read(parent.join(file))?;
        let es: Vec<Episode> = serde_json::from_slice(&bytes)?;
        if es.len() != 128
            || es.iter().any(|e| {
                e.ids.is_empty()
                    || e.ids.len() > 128
                    || e.roles.len() != e.ids.len()
                    || e.source >= e.query
                    || e.query >= e.ids.len()
                    || e.answer >= 40
            })
        {
            return Err(invalid("panel"));
        }
        fs::write(out.join(file), bytes)?;
        let input_sha256 = sha256_file(&out.join(file))?;
        if input_sha256 != sha256_file(&parent.join(file))? {
            return Err(invalid("panel changed during copy"));
        }
        panels.push((file, es, input_sha256));
    }
    let mut reports = Vec::new();
    for name in ["Held-s1", "Held-s2", "Local-s1", "Local-s2"] {
        let path = parent.join(name).join("model");
        let model_sha256 = sha256_file(&path.join("model.safetensors"))?;
        let config_sha256 = sha256_file(&path.join("config.json"))?;
        let latch_record_sha256 = sha256_file(&path.join("read-identity-latch.json"))?;
        let model = StackModel::load(&path, &Device::Cpu)?;
        if model.config.arch != StackArch::Geometric
            || model.config.pattern != "rra"
            || model.config.width != 32
            || model.config.context != 128
            || model.config.heads != 2
            || model.config.vocab_size != 40
            || model.config.read != ReadScore::Lorentz
            || !model.config.rotation
            || model.config.pointer.is_some()
            || model.config.memory.is_some()
            || model.config.select.is_some()
            || model.read_identity_latch()
                != Some(if name.starts_with("Held") {
                    ReadIdentityLatch::Held
                } else {
                    ReadIdentityLatch::Local
                })
        {
            return Err(invalid("modelcontract"));
        }
        let a = export_components(&model, model_sha256.clone(), config_sha256.clone())?;
        let file = out.join(format!("{name}-components.json"));
        fs::write(&file, serde_json::to_vec_pretty(&a)?)?;
        let reloaded: ComponentArtifact = serde_json::from_slice(&fs::read(&file)?)?;
        if reloaded.model_sha256 != model_sha256 || reloaded.config_sha256 != config_sha256 {
            return Err(invalid("component model binding differs"));
        }
        let artifact_sha256 = sha256_file(&file)?;
        let mut components = reloaded.components()?;
        let mut distance_kernel =
            IntegerLorentzDistance::new(16, table.clone()).map_err(|e| invalid(e.to_string()))?;
        let mut measured = Vec::new();
        for (file, es, input_sha256) in &panels {
            let mut panel = score_panel(
                &model,
                es,
                &mut components,
                &mut distance_kernel,
                start,
                limit,
            )?;
            panel["panel"] = json!(file);
            if sha256_file(&parent.join(file))? != *input_sha256 {
                return Err(invalid("saved panel changed during evaluation"));
            }
            panel["input_sha256"] = json!(input_sha256);
            measured.push(panel);
        }
        if sha256_file(&path.join("model.safetensors"))? != model_sha256
            || sha256_file(&path.join("config.json"))? != config_sha256
            || sha256_file(&path.join("read-identity-latch.json"))? != latch_record_sha256
        {
            return Err(invalid("saved model files changed"));
        }
        reports.push(json!({"name":name,"model_root":path,"model_sha256":model_sha256,"config_sha256":config_sha256,"latch_record_sha256":latch_record_sha256,"component_artifact":file,"component_artifact_sha256":artifact_sha256,"components":reloaded,"distance_table_sha256":table_artifact.sha256,"measurements":measured}));
    }
    Ok(
        json!({"schema":"uor-r4.integer-lorentz-replay/1","distance_table":table_artifact,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE"),"reports":reports,"elapsed_seconds":start.elapsed().as_secs_f64(),"max_seconds":limit,"training_updates":0,"projection_output_exponent":PROJECTION_OUTPUT_EXPONENT,"distance_diagnostic_reference":"actual F32 dequantized q/k passed to parent replay; norm uses sixteen F32 partial sums, cross-dot uses ascending F32 features (same reduction at this fixed head width16), F64 sqrt(1+norm), signed excess and log1p arcosh with 1e-7 floor; not an ideal F64 dot; padding excluded from diagnostic counts","ablation":"read.out map zeroed then restored; recurrent residual remains; not a forced NoRead gate","scope":"actual integer gate/16bit gained retained identity/4bit paired qk maps with one Q16 rounding and existing integer floor-Q32 lift/Q24 arcosh distance; float trunk/input conversion/beta/offset/age/NoRead/softmax/value/head; same authored development/fullprefix; no whole-serving/language/energy qualification"}),
    )
}
fn main() -> Result<()> {
    let (mut out, mut parent) = (None, None);
    let mut limit = 900u64;
    for arg in std::env::args().skip(1) {
        let (k, v) = arg.split_once('=').ok_or_else(|| invalid("key=value"))?;
        match k {
            "out" => out = Some(std::path::PathBuf::from(v)),
            "parent" => parent = Some(std::path::PathBuf::from(v)),
            "max_seconds" => limit = v.parse().map_err(|_| invalid("limit"))?,
            _ => return Err(invalid("argument")),
        }
    }
    let out = out.ok_or_else(|| invalid("out"))?;
    let parent = parent.ok_or_else(|| invalid("parent"))?;
    if limit == 0 || limit > 900 {
        return Err(invalid("limit1..900"));
    }
    report_output::claim(&out)?;
    let result = run(&out, &parent, limit);
    match &result {
        Ok(v) => fs::write(out.join("report.json"), serde_json::to_vec_pretty(v)?)?,
        Err(e) => fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error":e.to_string()}))?,
        )?,
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result?;
    Ok(())
}
