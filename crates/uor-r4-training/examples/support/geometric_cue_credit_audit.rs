//! Fixed native checkpoint signed-credit attribution. No optimizer or proposal.
use super::*;

#[derive(Clone)]
struct Credit {
    rows: usize,
    sum: Vec<f64>,
    sum_abs: Vec<f64>,
    positive: Vec<usize>,
    negative: Vec<usize>,
    sum_squares: Vec<f64>,
    maximum_abs: Vec<f64>,
}
impl Credit {
    fn new(width: usize) -> Self {
        Self {
            rows: 0,
            sum: vec![0.; width],
            sum_abs: vec![0.; width],
            positive: vec![0; width],
            negative: vec![0; width],
            sum_squares: vec![0.; width],
            maximum_abs: vec![0.; width],
        }
    }
    fn add(&mut self, gradient: &[f32]) -> Result<()> {
        if gradient.len() != self.sum.len() || gradient.iter().any(|g| !g.is_finite()) {
            return Err(invalid("cue credit shape/nonfinite row differs").into());
        }
        self.rows += 1;
        for (i, &g) in gradient.iter().enumerate() {
            self.sum[i] += f64::from(g);
            self.sum_abs[i] += f64::from(g).abs();
            self.positive[i] += usize::from(g > 0.);
            self.negative[i] += usize::from(g < 0.);
            self.sum_squares[i] += f64::from(g).powi(2);
            self.maximum_abs[i] = self.maximum_abs[i].max(f64::from(g).abs());
        }
        Ok(())
    }
    fn report(&self) -> Value {
        let rows = (0..self.sum.len()).map(|i| json!({
            "coefficient_index":i,"lane":i/120,"angular_bin":i%120,
            "signed_sum":self.sum[i],"sum_abs":self.sum_abs[i],
            "mean":if self.rows>0 {Some(self.sum[i]/self.rows as f64)} else {None},
            "positive_rows":self.positive[i],"negative_rows":self.negative[i],
            "positive_sum":(self.sum_abs[i]+self.sum[i])/2.,"negative_magnitude_sum":(self.sum_abs[i]-self.sum[i])/2.,
            "effective_rows":if self.sum_squares[i]>0. {Some(self.sum_abs[i].powi(2)/self.sum_squares[i])} else {None},
            "maximum_row_abs_credit_fraction":if self.sum_abs[i]>0. {Some(self.maximum_abs[i]/self.sum_abs[i])} else {None},
            "cancellation_fraction":if self.sum_abs[i]>0. {Some((1.-self.sum[i].abs()/self.sum_abs[i]).clamp(0.,1.))} else {None}
        })).collect::<Vec<_>>();
        json!({"rows":self.rows,"coefficients":rows,"zero_credit_cancellation":"undefined/null; no threshold classifier"})
    }
}
fn vector(batch: &Batch, width: usize) -> Result<Vec<f32>> {
    if batch.gradients.len() != 1 {
        return Err(invalid("cue credit expected exactly one coefficient family").into());
    }
    let g = batch
        .gradients
        .values()
        .next()
        .ok_or_else(|| invalid("cue credit family absent"))?
        .flatten_all()?
        .to_vec1::<f32>()?;
    if g.len() != width || g.iter().any(|x| !x.is_finite()) {
        return Err(invalid("cue credit coefficient width/nonfinite differs").into());
    }
    Ok(g)
}
// These public construction descriptors only stratify already computed credit.
fn groups(id: &str, role: &str) -> Result<Vec<String>> {
    let parts = id
        .strip_prefix("development-diverse-")
        .ok_or_else(|| invalid("cue credit public construction ID absent"))?
        .split('-')
        .collect::<Vec<_>>();
    if parts.len() != 6
        || !matches!(
            parts[0],
            "length2" | "length4" | "length8" | "update" | "reassert"
        )
        || !matches!(parts[2], "swap0" | "swap1")
        || !matches!(parts[3], "q0" | "q1" | "q2" | "q3" | "q4" | "q5")
        || !matches!(parts[4], "forward" | "reverse")
        || !matches!((role, parts[5]), ("job", "job") | ("where", "home"))
    {
        return Err(invalid("cue credit public role/frame/swap/order/length differs").into());
    }
    Ok(vec![
        format!("query_role/{role}"),
        format!("history_kind/{}", parts[0]),
        format!("assignment/{}", parts[2]),
        format!("query_frame/{}", parts[3]),
        format!("chronology/{}", parts[4]),
    ])
}
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    a: &Args,
    start: Instant,
    source: &SourceRealizerWeights,
    parent: &NativeSourceRealizer,
    integer: &IntegerRealizer,
    weights: &CueAngularWeights,
    f: &Frozen,
    development: &[Episode],
    inputs: &BTreeMap<String, String>,
    seals: &BTreeSet<PathBuf>,
    receipts: &Value,
) -> Result<Value> {
    let config = a
        .cue_credit_audit
        .as_ref()
        .ok_or_else(|| invalid("cue credit bound configuration absent"))?;
    report_output::verify(&config.checkpoint_root)?;
    if sha256_file(&config.checkpoint_root.join("manifest.json"))?
        != config.checkpoint_manifest_sha256
    {
        return Err(invalid("cue credit selected checkpoint manifest differs").into());
    }
    let warm = warm(a)?;
    if warm.initial_cue_bundle != config.checkpoint_root.join("cue")
        || a.frozen_prefix_bundle.as_ref() != Some(&config.checkpoint_root.join("prefix"))
        || warm.frozen_end_bundle != config.checkpoint_root.join("source-end")
    {
        return Err(invalid("cue credit donor paths not exact selected checkpoint").into());
    }
    let reference_seal = nearest_seal(&config.reference_canonical)?;
    report_output::verify(&reference_seal)?;
    if sha256_file(&config.reference_canonical)? != config.reference_canonical_sha256 {
        return Err(invalid("cue credit canonical reference hash differs").into());
    }
    let reference = read_json(&config.reference_canonical)?;
    let reference_rows = reference["rows"]
        .as_array()
        .ok_or_else(|| invalid("cue credit reference rows absent"))?;
    let mut bound_inputs = inputs.clone();
    let mut bound_seals = seals.clone();
    bound_inputs.insert(
        config
            .checkpoint_root
            .join("manifest.json")
            .to_string_lossy()
            .into_owned(),
        config.checkpoint_manifest_sha256.clone(),
    );
    bound_inputs.insert(
        config.reference_canonical.to_string_lossy().into_owned(),
        config.reference_canonical_sha256.clone(),
    );
    bound_inputs.insert(
        reference_seal
            .join("manifest.json")
            .to_string_lossy()
            .into_owned(),
        sha256_file(&reference_seal.join("manifest.json"))?,
    );
    bound_seals.insert(config.checkpoint_root.clone());
    bound_seals.insert(reference_seal);
    let inputs = &bound_inputs;
    let seals = &bound_seals;
    let (count, _) = panel_counts(a);
    if development.len() != count || reference_rows.len() != count || count != 512 {
        return Err(invalid("cue credit requires complete fixed512 development panel").into());
    }
    let width = weights.config().coefficient_count()?;
    if width != 960 {
        return Err(invalid("cue credit fixed native960 coefficient width differs").into());
    }
    let initial_receipts = parameter_receipts(&weights.parameters())?;
    let initial_packed = weights.packed_coefficients()?;
    let actual_cue = integer.compile_cue_carrier(weights.native()?)?;
    let actual_cue_metadata = serde_json::to_value(actual_cue.metadata())?;
    for (episode, row) in development.iter().zip(reference_rows) {
        if row["id"] != episode.packet.id
            || row["tokens"][0]["cue_carrier"]["metadata"] != actual_cue_metadata
        {
            return Err(
                invalid("cue credit saved angular incidence checkpoint metadata differs").into(),
            );
        }
    }

    let data = read_json(&a.development_panel.join("context-data.json"))?;
    let labels = data["cases"]
        .as_array()
        .ok_or_else(|| invalid("cue credit public descriptors absent"))?;
    if labels.len() != count {
        return Err(invalid("cue credit public descriptor count differs").into());
    }
    immutable(inputs, seals)?;
    let full = batch(
        &(0..count).collect::<Vec<_>>(),
        development,
        source,
        parent,
        Some(integer),
        weights,
        f,
        false,
        a,
        start,
    )?;
    let full_vector = vector(&full, width)?;
    let reference_ce = reference["native_equal_episode_ce"]
        .as_f64()
        .filter(|x| x.is_finite())
        .ok_or_else(|| invalid("cue credit reference objective absent/nonfinite"))?;
    let full_ce = full.report["native_equal_episode_ce"]
        .as_f64()
        .ok_or_else(|| invalid("cue credit full objective absent"))?;
    if (full_ce - reference_ce).abs() > 1e-8 + 1e-8 * reference_ce.abs() {
        return Err(
            invalid("cue credit full native objective differs from bound reference").into(),
        );
    }
    write_json(&a.out, "fullbatch-positive-control.json", &full.report)?;
    let mut total = Credit::new(width);
    let mut by_group = BTreeMap::<String, Credit>::new();
    let mut rows = Vec::new();
    let mut mean_ce = 0.;
    let mut first_read_rows = vec![0usize; width];
    let mut first_read_differential_rows = vec![0usize; width];
    for (index, (episode, label)) in development.iter().zip(labels).enumerate() {
        deadline(a, start)?;
        if label["id"] != episode.packet.id {
            return Err(invalid("cue credit descriptor row ID differs").into());
        }
        let refrow = &reference_rows[index];
        if refrow["id"] != episode.packet.id {
            return Err(invalid("cue credit reference row ID differs").into());
        }
        let lanes = refrow["tokens"][0]["cue_carrier"]["angular_indices"]
            .as_array()
            .ok_or_else(|| invalid("cue credit saved firstread angular lanes absent"))?;
        if lanes.len() != width / 120 {
            return Err(invalid("cue credit saved angular lane width differs").into());
        }
        let candidate_count = lanes
            .first()
            .and_then(Value::as_array)
            .map(Vec::len)
            .ok_or_else(|| invalid("cue credit saved angular candidates absent"))?;
        if candidate_count == 0 {
            return Err(invalid("cue credit saved empty candidate geometry").into());
        }
        let mut incidence = vec![0usize; width];
        for (lane, values) in lanes.iter().enumerate() {
            let values = values
                .as_array()
                .ok_or_else(|| invalid("cue credit saved lane absent"))?;
            if values.len() != candidate_count {
                return Err(invalid("cue credit saved candidate width differs").into());
            }
            for v in values {
                if !v.is_null() {
                    let bin = v
                        .as_u64()
                        .filter(|b| *b < 120)
                        .ok_or_else(|| invalid("cue credit saved bin invalid"))?
                        as usize;
                    incidence[lane * 120 + bin] += 1;
                }
            }
        }
        let active = incidence
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        let differential = incidence
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0 && **n < candidate_count)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        for &i in &active {
            first_read_rows[i] += 1;
        }
        for &i in &differential {
            first_read_differential_rows[i] += 1;
        }
        let role = label["query_role"]
            .as_str()
            .ok_or_else(|| invalid("cue credit role descriptor absent"))?;
        let axes = groups(&episode.packet.id, role)?;
        let row = batch(
            &[index],
            development,
            source,
            parent,
            Some(integer),
            weights,
            f,
            false,
            a,
            start,
        )?;
        let g = vector(&row, width)?;
        let ce = row.report["native_equal_episode_ce"]
            .as_f64()
            .filter(|v| v.is_finite())
            .ok_or_else(|| invalid("cue credit row objective nonfinite"))?;
        let ref_ce = refrow["native_mean_token_ce"]
            .as_f64()
            .filter(|v| v.is_finite())
            .ok_or_else(|| invalid("cue credit reference row CE absent/nonfinite"))?;
        if (ce - ref_ce).abs() > 1e-8 + 1e-8 * ref_ce.abs() {
            return Err(invalid("cue credit row native CE differs from saved reference").into());
        }
        mean_ce += ce / count as f64;
        total.add(&g)?;
        for axis in &axes {
            by_group
                .entry(axis.clone())
                .or_insert_with(|| Credit::new(width))
                .add(&g)?;
        }
        let sparse = g
            .iter()
            .enumerate()
            .filter(|(_, g)| **g != 0.)
            .map(|(i, g)| json!([i, g]))
            .collect::<Vec<_>>();
        rows.push(json!({"id":episode.packet.id,"episode_index":index,"target_positions":episode.target.len(),"native_mean_token_ce":ce,"posthoc_public_groups":axes,"nonzero_signed_coefficient_credit":sparse,"saved_firstread_consumed_coordinates":active,"saved_firstread_occurrence_candidate_heterogeneous_coordinates":differential,"coefficient_width":width,"omitted_coefficient_credit":0.,"independent_native_trace_parity":row.report["independent_native_trace_parity"]}));
        if (index + 1) % 32 == 0 {
            write_json(
                &a.out,
                "progress.json",
                &json!({"rows_completed":index+1,"optimizer_updates":0,"proposal_count":0,"elapsed_seconds":start.elapsed().as_secs_f64()}),
            )?;
        }
    }
    let mut max_error = 0.0_f64;
    for (i, &full_g) in full_vector.iter().enumerate() {
        let mean = total.sum[i] / count as f64;
        let error = (mean - f64::from(full_g)).abs();
        max_error = max_error.max(error);
        if error > 1e-6 + 2e-4 * f64::from(full_g).abs() {
            return Err(invalid(
                "cue credit rowaverage/fullbatch gradient positivecontrol differs",
            )
            .into());
        }
    }
    let full_ce = full.report["native_equal_episode_ce"]
        .as_f64()
        .ok_or_else(|| invalid("cue credit full objective absent"))?;
    if (mean_ce - full_ce).abs() > 1e-8 + 1e-8 * full_ce.abs() {
        return Err(invalid("cue credit rowaverage/fullbatch CE differs").into());
    }
    if parameter_receipts(&weights.parameters())? != initial_receipts
        || weights.packed_coefficients()? != initial_packed
    {
        return Err(invalid("cue credit frozen cue changed").into());
    }
    frozen(source, receipts)?;
    immutable(inputs, seals)?;
    write_json(
        &a.out,
        "row-credit.json",
        &json!({"rows":rows,"scope":"ordinary final native CE; equalwithinrow target+EOS normalization; no labels steer gradients; rowindices preserve all512"}),
    )?;
    let mut credit = total.report();
    credit["saved_firstread_incidence"] = json!({"consumed_row_counts":first_read_rows,"occurrence_candidate_heterogeneous_row_counts":first_read_differential_rows,"scope":"saved bound canonical token0 geometry; occurrence heterogeneity only, not typed record contrast; gradient credit spans all teacherforced positions; zero gradient does not imply absent representation"});
    write_json(&a.out, "coefficient-credit.json", &credit)?;
    let reports = by_group
        .into_iter()
        .map(|(k, v)| (k, v.report()))
        .collect::<BTreeMap<_, _>>();
    write_json(
        &a.out,
        "group-credit.json",
        &json!({"groups":reports,"scope":"posthoc separate public axes, not optimization objectives or selection gates"}),
    )?;
    Ok(
        json!({"schema":"uor-r4.geometric-cue-credit-audit/1","mode":a.mode,"status":"completed","cases":count,"coefficient_width":width,"optimizer_updates":0,"adam_updates":0,"proposal_count":0,"accepted_discrete_updates":0,"gradient_passes":count+1,"positive_control":{"row_average_matches_fullbatch":true,"maximum_coefficient_absolute_error":max_error,"row_average_native_ce":mean_ce,"fullbatch_native_ce":full_ce,"coefficient_absolute_tolerance":1e-6,"coefficient_relative_tolerance":2e-4},"initial_cue_packed_sha256":sha256_bytes(&initial_packed),"frozen_cue_parameter_receipts":initial_receipts,"frozen_payloads":f.hashes(),"frozen_source_receipts":receipts,"input_manifests_sha256":inputs,"cue_credit_audit":config,"reference_native_ce":reference_ce,"elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib(),"evaluation_predictions":0,"claim":"fixedcheckpoint signed ordinaryCE attribution; no learning, representational impossibility, heldout transfer or generalchat conclusion"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_separates_opposing_credit_from_zero_gradient_credit() -> Result<()> {
        let mut c = Credit::new(3);
        c.add(&[2., 1., 0.])?;
        c.add(&[-2., 1., 0.])?;
        let r = c.report();
        assert_eq!(r["coefficients"][0]["cancellation_fraction"], 1.);
        assert_eq!(r["coefficients"][0]["positive_sum"], 2.);
        assert_eq!(r["coefficients"][0]["negative_magnitude_sum"], 2.);
        assert_eq!(r["coefficients"][0]["effective_rows"], 2.);
        assert_eq!(r["coefficients"][0]["maximum_row_abs_credit_fraction"], 0.5);
        assert_eq!(r["coefficients"][1]["cancellation_fraction"], 0.);
        assert!(r["coefficients"][2]["cancellation_fraction"].is_null());
        assert!(c.add(&[f32::NAN, 0., 0.]).is_err());
        let mut tiny = Credit::new(1);
        tiny.add(&[1e-20])?;
        tiny.add(&[-1e-20])?;
        let small = tiny.report();
        assert_eq!(small["coefficients"][0]["effective_rows"], 2.);
        assert_eq!(
            small["coefficients"][0]["maximum_row_abs_credit_fraction"],
            0.5
        );
        Ok(())
    }
    #[test]
    fn public_groups_do_not_silently_drop_mismatched_roles_or_frames() -> Result<()> {
        let id = "development-diverse-update-00-swap1-q5-reverse-home";
        assert_eq!(groups(id, "where")?.len(), 5);
        assert!(groups(id, "job").is_err());
        assert!(groups(&id.replace("q5", "q6"), "where").is_err());
        Ok(())
    }
}
