//! Development-only native action-space intervention. No fitting or fresh reads.
use super::*;
use uor_r4_integer::geometric_source_realizer::SourceBoundCueMode;

/// These are contributors to a token already chosen by global alias aggregation,
/// not a single selected source. Highest individual Copy is reported separately.
fn attribution(actions: &Value) -> Result<Value> {
    let chosen = actions["chosen_token_id"]
        .as_u64()
        .ok_or_else(|| invalid("chosen token absent"))?;
    let rows = actions["actions"]
        .as_array()
        .ok_or_else(|| invalid("action rows absent"))?;
    let copies: Vec<_> = rows
        .iter()
        .filter(|r| r["action"].get("Copy").is_some())
        .collect();
    let factual = copies.iter().copied().max_by(|a, b| {
        // Preserve the earliest action on a score tie, matching factual Copy routing.
        a["score_q24"]
            .as_i64()
            .cmp(&b["score_q24"].as_i64())
            .then_with(|| {
                b["action_offset"]
                    .as_u64()
                    .cmp(&a["action_offset"].as_u64())
            })
    });
    let contributors: Vec<_> = rows
        .iter()
        .filter(|r| r["token_id"].as_u64() == Some(chosen))
        .cloned()
        .collect();
    let terminals: Vec<_> = contributors
        .iter()
        .filter(|r| r["action"] == "Period" || r["action"] == "Stop")
        .cloned()
        .collect();
    Ok(
        json!({"factual_highest_individual_Copy":factual,"chosen_token_alias_contributors":contributors,
        "chosen_token_terminal_contributors":terminals,"single_selected_source_from_alias_aggregation":false}),
    )
}
fn read(
    native: &IntegerRealizer,
    cue: &NativeCueCarrier<'_>,
    prefix: &NativePrefixTransport<'_>,
    end: &NativeSourceEndTransport<'_>,
    episode: &Episode,
    own: &[u32],
    mode: SourceBoundCueMode,
) -> Result<Value> {
    let segments = episode.segments()?;
    let out = native.read_bank_with_source_bound_actions(
        &segments,
        &episode.packet.query_ids,
        own,
        cue,
        prefix,
        end,
        mode,
    )?;
    let mut v = compact(&out.prefix_bank.cue_bank.bank)?;
    let actions = serde_json::to_value(&out.actions)?;
    v["actions"] = actions;
    Ok(
        json!({"native":v,"cue_carrier":out.prefix_bank.cue_bank.carrier,
        "prefix_transport":out.prefix_bank.prefix,"source_end":out.source_end,
        "source_bound_mode":out.mode,
        "source_cue_q24":out.source_cue_q24,"additional_cue_costs":out.additional_cue_costs,
        "discarded_legacy_action_reductions":out.discarded_legacy_action_reductions}),
    )
}
fn canonical(
    native: &IntegerRealizer,
    cue: &NativeCueCarrier<'_>,
    prefix: &NativePrefixTransport<'_>,
    end: &NativeSourceEndTransport<'_>,
    episodes: &[Episode],
    a: &Args,
    start: Instant,
    mode: SourceBoundCueMode,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut zeros = Vec::new();
    let mut count = 0;
    let mut total = 0.;
    for e in episodes {
        let mut tokens = Vec::new();
        let mut sum = 0.;
        let mut rowzero = false;
        for (step, &target) in e.target.iter().enumerate() {
            deadline(a, start)?;
            let mut trace = read(native, cue, prefix, end, e, &e.target[..step], mode)?;
            // Labels first enter after the complete target-free native read.
            let actions = &trace["native"]["actions"];
            let den = actions["total_weight_q31"]
                .as_u64()
                .ok_or_else(|| invalid("normalizer absent"))?;
            let masses = actions["token_masses"]
                .as_array()
                .ok_or_else(|| invalid("alias masses absent"))?;
            let mass = masses
                .iter()
                .filter(|m| m["token_id"].as_u64() == Some(u64::from(target)))
                .try_fold(0u64, |acc, m| -> Result<u64> {
                    acc.checked_add(
                        m["weight_q31"]
                            .as_u64()
                            .ok_or_else(|| invalid("alias weight absent"))?,
                    )
                    .ok_or_else(|| invalid("alias weight overflow").into())
                })?;
            if den == 0 || mass > den {
                return Err(invalid("invalid source-bound native probability").into());
            }
            let probability = mass as f64 / den as f64;
            let ce = if mass == 0 {
                rowzero = true;
                zeros.push(json!({"id":e.packet.id,"step":step}));
                None
            } else {
                let x = -probability.ln();
                sum += x;
                Some(x)
            };
            trace["postprediction_provenance"] = attribution(actions)?;
            trace["step"] = json!(step);
            trace["teacherforced_prefix_ids_labels_only"] = json!(&e.target[..step]);
            trace["target_label_only_after_read"] = json!(target);
            trace["native_ce"] = json!(ce);
            trace["target_probability"] = json!(probability);
            trace["target_mass_q31"] = json!(mass);
            trace["total_weight_q31"] = json!(den);
            tokens.push(trace);
            count += 1;
        }
        let mean = if rowzero {
            None
        } else {
            Some(sum / e.target.len() as f64)
        };
        if let Some(mean) = mean {
            total += mean / episodes.len() as f64;
        }
        rows.push(json!({"id":e.packet.id,"native_mean_token_ce":mean,"tokens":tokens}));
    }
    Ok(
        json!({"cases":episodes.len(),"target_positions":count,"native_equal_episode_ce":if zeros.is_empty(){Some(total)}else{None},"zero_support_positions":zeros,"rows":rows,"probability_floor":false,"infinite_native_objective_when_zero":true}),
    )
}
fn generation(
    native: &IntegerRealizer,
    cue: &NativeCueCarrier<'_>,
    prefix: &NativePrefixTransport<'_>,
    end: &NativeSourceEndTransport<'_>,
    episodes: &[Episode],
    tok: &ByteBpeTokenizer,
    a: &Args,
    start: Instant,
    mode: SourceBoundCueMode,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut complete = 0;
    for e in episodes {
        let mut ids = Vec::new();
        let mut tokens = Vec::new();
        for step in 0..a.maximum_generation_tokens {
            deadline(a, start)?;
            let mut trace = read(native, cue, prefix, end, e, &ids, mode)?;
            let actions = &trace["native"]["actions"];
            let chosen = u32::try_from(
                actions["chosen_token_id"]
                    .as_u64()
                    .ok_or_else(|| invalid("chosen ID absent"))?,
            )?;
            trace["postprediction_provenance"] = attribution(actions)?;
            trace["step"] = json!(step);
            trace["actual_prefix_ids"] = json!(&ids);
            tokens.push(trace);
            ids.push(chosen);
            if chosen == native.binding().eos_token_id() {
                break;
            }
        }
        let eos = ids.last() == Some(&native.binding().eos_token_id());
        let plain = if eos { &ids[..ids.len() - 1] } else { &ids[..] };
        let bytes = tok.decode_bytes(plain);
        let raw = String::from_utf8_lossy(&bytes);
        let text = raw.strip_prefix(' ').unwrap_or(&raw);
        let accepted = eos && String::from_utf8(bytes.clone()).is_ok() && e.answers.accepts(text);
        complete += usize::from(accepted);
        rows.push(json!({"id":e.packet.id,"generated_ids_including_eos":ids,"eos":eos,"reply_text":text,"raw_decoded_bytes_hex":hex::encode(bytes),"accepted_complete_answer":accepted,"tokens":tokens}));
    }
    Ok(
        json!({"cases":episodes.len(),"accepted_complete":complete,"maximum_generated_tokens":a.maximum_generation_tokens,"canonical_prefixes_used":false,"rows":rows}),
    )
}
fn compare(old: &Value, new: &Value) -> Result<Value> {
    let aa = old["rows"]
        .as_array()
        .ok_or_else(|| invalid("comparison rows absent"))?;
    let bb = new["rows"]
        .as_array()
        .ok_or_else(|| invalid("comparison rows absent"))?;
    if aa.len() != bb.len() {
        return Err(invalid("comparison row count differs").into());
    }
    let mut rows = Vec::new();
    let mut gain = 0;
    let mut loss = 0;
    for (a, b) in aa.iter().zip(bb) {
        if a["id"] != b["id"] {
            return Err(invalid("comparison row IDs differ").into());
        }
        let x = a["generated_ids_including_eos"]
            .as_array()
            .ok_or_else(|| invalid("generated IDs absent"))?;
        let y = b["generated_ids_including_eos"]
            .as_array()
            .ok_or_else(|| invalid("generated IDs absent"))?;
        let first = (0..x.len().max(y.len())).find(|&i| x.get(i) != y.get(i));
        let was = a["accepted_complete_answer"] == true;
        let now = b["accepted_complete_answer"] == true;
        gain += usize::from(!was && now);
        loss += usize::from(was && !now);
        rows.push(json!({"id":a["id"],"prior_complete":was,"current_complete":now,"first_generated_divergence":first,"prior_ids":x,"current_ids":y}));
    }
    Ok(json!({"gains":gain,"losses":loss,"rows":rows}))
}
// Add only diagnostic fields to the independently executed legacy trace; its
// native scores, alias masses and selected IDs remain byte-for-byte unchanged.
fn annotate(canonical: &mut Value, generation: &mut Value, episodes: &[Episode]) -> Result<()> {
    for panel in [&mut *canonical, &mut *generation] {
        let rows = panel["rows"]
            .as_array_mut()
            .ok_or_else(|| invalid("probe rows absent"))?;
        if rows.len() != episodes.len() {
            return Err(invalid("probe annotation row count differs").into());
        }
        for (row, e) in rows.iter_mut().zip(episodes) {
            if row["id"] != e.packet.id {
                return Err(invalid("probe annotation identity differs").into());
            }
            for token in row["tokens"]
                .as_array_mut()
                .ok_or_else(|| invalid("probe tokens absent"))?
            {
                let actions = &token["native"]["actions"];
                let provenance = attribution(actions)?;
                token["postprediction_provenance"] = provenance;
                if let (Some(m), Some(d)) = (
                    token["target_mass_q31"].as_u64(),
                    token["total_weight_q31"].as_u64(),
                ) {
                    if d == 0 || m > d {
                        return Err(invalid("probe probability invalid").into());
                    }
                    token["target_probability"] = json!(m as f64 / d as f64);
                }
            }
            if let Some(ids) = row["generated_ids_including_eos"].as_array() {
                let expected: Vec<_> = e.target.iter().map(|x| json!(x)).collect();
                let divergence =
                    (0..ids.len().max(expected.len())).find(|&i| ids.get(i) != expected.get(i));
                row["first_divergence_from_canonical_labels_only"] = json!(divergence);
            }
        }
    }
    Ok(())
}
pub(super) fn run(
    a: &Args,
    start: Instant,
    native: &IntegerRealizer,
    tok: &ByteBpeTokenizer,
    cue: &NativeCueCarrier<'_>,
    prefix: &NativePrefixTransport<'_>,
    end: &NativeSourceEndTransport<'_>,
    episodes: &[Episode],
    inputs: &BTreeMap<String, String>,
    seals: &BTreeSet<PathBuf>,
) -> Result<Value> {
    if episodes.len() != 512 {
        return Err(invalid("source-bound probe requires exact512 development").into());
    }
    let mut arms = Vec::new();
    let mut prior: Vec<(String, Value, Value)> = Vec::new();
    for (name, mode) in [
        ("legacy", None),
        (
            "all-source-no-terminal-cue",
            Some(SourceBoundCueMode::NoTerminalCue),
        ),
        ("all-source-shared-cue", Some(SourceBoundCueMode::SharedCue)),
    ] {
        let root = a.out.join(name);
        report_output::claim(&root)?;
        let mut canonical = match mode {
            None => source_end_fit::canonical(native, cue, prefix, end, episodes, a, start)?,
            Some(m) => canonical(native, cue, prefix, end, episodes, a, start, m)?,
        };
        let mut generation = match mode {
            None => source_end_fit::generation(native, cue, prefix, end, episodes, tok, a, start)?,
            Some(m) => generation(native, cue, prefix, end, episodes, tok, a, start, m)?,
        };
        let comparisons=prior.iter().map(|(n,c,g)| -> Result<Value>{
            let old=c["rows"].as_array().ok_or_else(||invalid("canonical comparison rows absent"))?;
            let new=canonical["rows"].as_array().ok_or_else(||invalid("canonical comparison rows absent"))?;
            if old.len()!=new.len() {return Err(invalid("canonical comparison count differs").into());}
            let losses=old.iter().zip(new).map(|(x,y)| -> Result<Value>{if x["id"]!=y["id"] {return Err(invalid("canonical comparison IDs differ").into());}Ok(json!({"id":x["id"],"prior_ce":x["native_mean_token_ce"],"current_ce":y["native_mean_token_ce"]}))}).collect::<Result<Vec<_>>>()?;
            Ok(json!({"prior_arm":n,"canonical_rows":losses,"generation":compare(g,&generation)?}))
        }).collect::<Result<Vec<_>>>()?;
        let receipt = json!({"arm":name,"cases":episodes.len(),"native_equal_episode_ce":canonical["native_equal_episode_ce"],"accepted_complete":generation["accepted_complete"],"comparisons_to_all_prior_arms":comparisons,"selection":"NONE","updates":0});
        write_json(&root, "canonical.json", &canonical)?;
        write_json(&root, "generation.json", &generation)?;
        write_json(&root, "receipt.json", &receipt)?;
        immutable(inputs, seals)?;
        report_output::seal(&root)?;
        report_output::verify(&root)?;
        arms.push(receipt);
        prior.push((name.into(), canonical, generation));
    }
    immutable(inputs, seals)?;
    Ok(
        json!({"schema":"uor-r4.cue-source-bound-action-probe/1","status":"COMPLETED","mode":a.mode,"arms":arms,"updates":0,"gradients":"NOT_RUN","fresh_predictions":"NOT_RUN","fresh_predictions_count":0,"selection":"NONE","evaluation_panel_rows_verified_only":128,"input_files_sha256":inputs,"parent_binding":native.artifact_binding(),"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"zero-update development action-space intervention; no capability or learned geometric advantage claim","cost_note":"all-source endpoint costs include retained legacy preparation; do not add that preparation twice"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alias_contributors_do_not_become_a_selected_terminal_source() -> Result<()> {
        let v = json!({"chosen_token_id":7,"actions":[
            {"action":{"Copy":{"bank_index":0,"source_offset":0}},"action_offset":0,"score_q24":9,"token_id":4,"source":{"source_ordinal":0}},
            {"action":"Period","action_offset":1,"score_q24":8,"token_id":7,"source":{"source_ordinal":0}},
            {"action":"Period","action_offset":2,"score_q24":8,"token_id":7,"source":{"source_ordinal":1}}]});
        let x = attribution(&v)?;
        assert_eq!(
            x["chosen_token_terminal_contributors"]
                .as_array()
                .map(Vec::len),
            Some(2)
        );
        assert_eq!(x["factual_highest_individual_Copy"]["token_id"], 4);
        assert_eq!(x["single_selected_source_from_alias_aggregation"], false);
        Ok(())
    }
}
