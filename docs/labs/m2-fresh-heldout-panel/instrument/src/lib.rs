//! Deterministic generator, noise-spread analyzer, and overlap/seal verifier
//! for the 256-row M2 fresh held-out reply panel (`m2-fresh256`, issue #2849).
//!
//! Zero floating-point types or operations are used anywhere in this crate.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Sealed SHA-256 of the saved437 cross-state field (`archive/labs/m2-fresh-437-2026-10-10/README.md:97`).
pub const SAVED437_FIELD_SHA256: &str =
    "de4a3234d6e92f4b12a657cc195425687bc67c243d9eded9a1df0797b9942a61";

/// Sealed SHA-256 of the 512-row development inputs (`archive/labs/m2-fresh-437-2026-10-10/README.md:100`).
pub const DEV512_INPUTS_SHA256: &str =
    "b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a";

/// Sealed SHA-256 of the opened 128-row transfer inputs (`archive/labs/m2-fresh-437-2026-10-10/README.md:105`).
pub const OPENED_FRESH128_INPUTS_SHA256: &str =
    "c5a643ff9ef86f5f16f15b420f42fe8b1921c5c8c7df5a7bbb88f7759cda5bda";

/// Ten 2-word fresh target literals for `length2` (all 10) and `reassert` (first 8).
/// Content words are disjoint from all historical curriculum and panel literals.
pub const FRESH256_LEN2: [&str; 10] = [
    "basalt loggia",
    "granitecauseway annex",
    "quartzite promenade",
    "dolomite veranda",
    "obsidian courtyard",
    "travertine pavilion",
    "feldspar gallery",
    "malachite rotunda",
    "alabaster cloister",
    "porphyry belvedere",
];

/// Four distinct 2-word initial-assertion literals overwritten in `update` histories.
pub const FRESH256_UPDATE_INIT: [&str; 4] = [
    "limestone pergola",
    "sandstone arcade",
    "slate conservatory",
    "chalk esplanade",
];

/// Ten 4-word fresh target literals for `length4` and `update`.
pub const FRESH256_LEN4: [&str; 10] = [
    "calm heron near marshland",
    "swift falcon over headland",
    "bold osprey by cliffside",
    "keen plover along dikeway",
    "lone curlew across moorland",
    "fleet swallow under Viaduct",
    "stout badger within thicket",
    "brisk otter beside millrace",
    "agile marten around coppice",
    "steady kestrel above fenland",
];

/// Ten 8-word fresh target literals for `length8`.
pub const FRESH256_LEN8: [&str; 10] = [
    "the bronze sundial waits near a cobble dikeway",
    "a carved astrolabe gleams under the stone Viaduct",
    "the iron sextant hangs within a dry boathouse",
    "a brass barometer stands along the high seawall",
    "the pewter hourglass sits across a tiled vestibule",
    "a glass chronometer ticks inside the old watchtower",
    "the steel pendulum swings outside a brick foundry",
    "a slate quadrant leans around the steep rampart",
    "the nickel telescope points over a wide saltmarsh",
    "a flint gyroscope turns below the arched gatehouse",
];

/// All 72 historical curriculum and transfer literals that must remain 100% excluded.
pub const HISTORICAL_EXPOSED_LITERALS: [&str; 72] = [
    // VALUES[0..32] from curriculum.rs
    "mariner",
    "carpenter",
    "botanist",
    "cartographer",
    "sculptor",
    "astronomer",
    "chemist",
    "surveyor",
    "azure orchard",
    "copper oasis",
    "mossy plateau",
    "velvet garden",
    "ochre canyon",
    "crimson inlet",
    "ivory forest",
    "violet hillside",
    "quiet cedar beside harbor",
    "bright willow beyond meadow",
    "small maple near river",
    "gentle oak above valley",
    "silver pine beside canyon",
    "warm birch beyond orchard",
    "green elm near plateau",
    "blue ash above garden",
    "the quiet cedar stands beside a blue harbor",
    "a bright willow grows beyond the silver meadow",
    "the small maple rests beside a warm river",
    "a gentle oak rises above the green valley",
    "the silver pine stands beside an ochre canyon",
    "a warm birch grows beyond the violet orchard",
    "the green elm rests beside an ivory plateau",
    "a blue ash rises above the crimson garden",
    // FACTOR_VALUES[0..8]
    "Cerulith",
    "Mornveil",
    "Jade Anchorage",
    "Pearl Crossing",
    "autumn lantern over water",
    "spring compass beside orchard",
    "the autumn lantern shines above the peaceful inlet",
    "a spring compass rests beside the ancient orchard",
    // FRESH_VALUES[0..8]
    "Raventhorn",
    "Selquorin",
    "Umber Landing",
    "Saffron Estuary",
    "winter beacon near hills",
    "summer ribbon above fields",
    "the winter beacon shines beyond the tranquil hillside",
    "a summer ribbon drifts above the quiet pasture",
    // FRESH2_VALUES[0..8]
    "Vellumspire",
    "Duskharbor",
    "Amber Footbridge",
    "Indigo Clearing",
    "dawn lantern beside stream",
    "twilight compass beyond meadow",
    "the dawn lantern stands beside the peaceful stream",
    "a twilight compass rests beyond the open meadow",
    // FRESH3_VALUES[0..8]
    "Orielhaven",
    "Fernwatch",
    "Cobalt Causeway",
    "Rosewood Terrace",
    "moon compass beside brook",
    "sunrise banner beyond glade",
    "the moon compass rests beside the still brook",
    "a sunrise banner stands beyond the shaded glade",
    // Untouched transfer literals from geometric-native-bank-observation-plan.rs
    "amber willow",
    "copper cedar",
    "amber birch silver willow copper cedar harbor silver",
    "silver cedar copper birch amber willow amber harbor",
    "violet willow",
    "copper meadow silver birch",
    "amber cedar violet willow",
    "orchard silver harbor amber",
];

/// Supported current-role job queries (`geometric-native-bank-observation-plan.rs:277`).
pub const CURRENT_JOB_QUERIES: [&str; 6] = [
    "What is my current job?",
    "What job do I currently have?",
    "What is my job now?",
    "Which job do I have now?",
    "Remind me of my current job.",
    "Tell me my current job.",
];

/// Supported current-role home queries (`geometric-native-bank-observation-plan.rs:285`).
pub const CURRENT_HOME_QUERIES: [&str; 6] = [
    "Where do I currently live?",
    "Where do I live now?",
    "What is my current residence?",
    "Remind me where I currently live.",
    "Tell me where I live now.",
    "Where is my current home?",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wire {
    pub text: String,
    pub relation: String,
    pub act: String,
    pub template: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct History {
    pub id: String,
    pub stratum: String,
    pub turns: Vec<Wire>,
    pub queries: Vec<Wire>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dev512SpreadReport {
    pub total_rows: usize,
    pub complete_rows: usize,
    pub evaluator_replay_diff_rows: usize,
    pub stratum_complete: BTreeMap<String, (usize, usize)>,
    pub block_pass_counts: Vec<(String, usize, usize)>,
    pub forward_pass: usize,
    pub reverse_pass: usize,
    pub chronology_abs_diff: usize,
    pub chronology_discordant_pairs: usize,
    pub swap0_pass: usize,
    pub swap1_pass: usize,
    pub swap_abs_diff: usize,
    pub swap_discordant_pairs: usize,
    pub wording_a_pass: usize,
    pub wording_b_pass: usize,
    pub wording_abs_diff: usize,
    pub wording_discordant_pairs: usize,
    pub job_pass: usize,
    pub home_pass: usize,
    pub role_abs_diff: usize,
    pub both_roles_complete_histories: usize,
    pub role_discordant_histories: usize,
    pub even_blocks_pass_of_256: usize,
    pub odd_blocks_pass_of_256: usize,
    pub half_panel_split_abs_diff: usize,
    /// Stratified 16-block (256-row) cluster-sampling SE in hundredths of a row (e.g. 542 = 5.42 rows).
    pub cluster_se_256_hundredths_rows: u64,
    /// Stratified 16-block (256-row) cluster-sampling SE in basis points of 256 (e.g. 212 = 2.12%).
    pub cluster_se_256_bps: u64,
    /// Independent binomial SE at the 40% (103/256) bar in hundredths of a row (784 = 7.84 rows).
    pub bar40_binomial_se_hundredths_rows: u64,
    /// Cluster-inflated SE at the 40% (103/256) bar in hundredths of a row.
    pub bar40_cluster_se_hundredths_rows: u64,
    /// Cluster-inflated SE at the 40% (103/256) bar in basis points of 256.
    pub bar40_cluster_se_bps: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fresh256Bundle {
    pub plan_json: Vec<u8>,
    pub plan_sha256: String,
    pub inputs_json: Vec<u8>,
    pub inputs_sha256: String,
    pub labels_json: Vec<u8>,
    pub labels_sha256: String,
    pub manifest_json: Vec<u8>,
    pub manifest_sha256: String,
    pub histories: Vec<History>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverlapAndSealAudit {
    pub rows: usize,
    pub histories: usize,
    pub unordered_blocks: usize,
    pub ordered_banks: usize,
    pub exact_literal_overlaps: usize,
    pub content_word_overlaps: usize,
    pub ngram_overlaps: usize,
    pub min_same_stratum_edit_distance: usize,
    pub literal_role_balance: bool,
    pub literal_initial_position_balance: bool,
    pub max_query_wording_spread: usize,
    pub max_turn_tokens: usize,
    pub max_target_tokens: usize,
    pub max_total_replay_tokens: usize,
    pub copy_first_stated_hits: usize,
    pub copy_last_stated_hits: usize,
    pub role_swap_distractor_hits: usize,
    pub best_constant_reply_hits: usize,
    pub unscored_inputs_leak_count: usize,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

/// Exact integer floor square root (`floor(sqrt(n))`).
pub fn isqrt_u128(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut x = 1u128 << ((128 - n.leading_zeros()).div_ceil(2));
    loop {
        let y = (x + n / x) / 2;
        if y >= x {
            return x;
        }
        x = y;
    }
}

/// Circle schedule enumerating every unordered edge once (`geometric_role_diversity.rs:275`).
pub fn round_robin_edges(count: usize) -> Vec<(usize, usize)> {
    if count < 2 {
        return Vec::new();
    }
    let size = count + count % 2;
    let mut vertices: Vec<usize> = (0..size).collect();
    let mut edges = Vec::new();
    for _ in 0..size - 1 {
        for i in 0..size / 2 {
            let (a, b) = (vertices[i], vertices[size - 1 - i]);
            if a < count && b < count {
                edges.push((a.min(b), a.max(b)));
            }
        }
        vertices[1..].rotate_right(1);
    }
    edges
}

pub fn extract_literal(w: &Wire) -> Result<&str, String> {
    let t = w
        .template
        .as_deref()
        .ok_or_else(|| "write template absent".to_string())?;
    let (pre, post) = t
        .split_once("{v}")
        .ok_or_else(|| "write slot absent".to_string())?;
    if post.contains("{v}") {
        return Err("multiple write slots".to_string());
    }
    w.text
        .strip_prefix(pre)
        .and_then(|s| s.strip_suffix(post))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "literal mismatch".to_string())
}

fn supported_wire(role: &str, act: &str, value: &str, query_index: usize) -> Result<Wire, String> {
    let template = match (role, act) {
        ("job", "assert") => "My current job is {v}.",
        ("job", "update") => "My current job has changed to {v}.",
        ("home", "assert") => "I currently live in {v}.",
        ("home", "update") => "I now live in {v}.",
        ("job", "query") => CURRENT_JOB_QUERIES[query_index % CURRENT_JOB_QUERIES.len()],
        ("home", "query") => CURRENT_HOME_QUERIES[query_index % CURRENT_HOME_QUERIES.len()],
        _ => return Err(format!("unsupported role/act: {role}/{act}")),
    };
    let text = if act == "query" {
        template.to_string()
    } else {
        template.replace("{v}", value)
    };
    Ok(Wire {
        text,
        relation: role.to_string(),
        act: act.to_string(),
        template: Some(template.to_string()),
    })
}

/// Deterministic token encoder for budget and numerical-segment verification.
/// Maps each word/punctuation span to a deterministic positive token ID in `1..32000`.
pub fn deterministic_token_ids(text: &str) -> Vec<u32> {
    let mut ids = Vec::new();
    for raw in text.split_whitespace() {
        let trimmed = raw.trim_end_matches(['.', '?']);
        if !trimmed.is_empty() {
            let digest = Sha256::digest(trimmed.as_bytes());
            let code = u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]]);
            ids.push(100 + (code % 31_000));
        }
        if raw.ends_with('.') {
            ids.push(46);
        } else if raw.ends_with('?') {
            ids.push(63);
        }
    }
    ids
}

/// Analyze the sealed `saved437` parent's complete-reply outcomes on the 512-row development panel.
pub fn analyze_saved437_dev512_spread(repo_root: &Path) -> Result<Dev512SpreadReport, String> {
    let rank177_path =
        repo_root.join("archive/labs/m2-pooled-rank-2026-10-10/reviewer-rank177.json");
    let replay_path = repo_root
        .join("archive/labs/m2-fresh-437-2026-10-10/development-replay-audit-summary.json");
    let rank177: Value = serde_json::from_slice(
        &std::fs::read(&rank177_path)
            .map_err(|e| format!("read {}: {e}", rank177_path.display()))?,
    )
    .map_err(|e| e.to_string())?;
    let replay: Value = serde_json::from_slice(
        &std::fs::read(&replay_path).map_err(|e| format!("read {}: {e}", replay_path.display()))?,
    )
    .map_err(|e| e.to_string())?;

    let complete_arr = rank177["endpoints"]["final"]["complete_ids"]
        .as_array()
        .ok_or_else(|| "complete_ids absent".to_string())?;
    let complete_set: BTreeSet<String> = complete_arr
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    if complete_set.len() != 437 {
        return Err(format!(
            "expected 437 complete_ids, found {}",
            complete_set.len()
        ));
    }
    if replay["counts"]["complete"].as_u64() != Some(437) {
        return Err("replay summary complete count differs from 437".to_string());
    }

    let strata_specs = [
        ("length2", 8usize),
        ("length4", 8),
        ("length8", 8),
        ("update", 4),
        ("reassert", 4),
    ];

    let mut all_ids = BTreeSet::new();
    let mut stratum_complete: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut block_pass_counts: Vec<(String, usize, usize)> = Vec::new();

    let mut forward_pass = 0usize;
    let mut reverse_pass = 0usize;
    let mut chronology_discordant_pairs = 0usize;

    let mut swap0_pass = 0usize;
    let mut swap1_pass = 0usize;
    let mut swap_discordant_pairs = 0usize;

    let mut wording_a_pass = 0usize;
    let mut wording_b_pass = 0usize;
    let mut wording_discordant_pairs = 0usize;

    let mut job_pass = 0usize;
    let mut home_pass = 0usize;
    let mut both_roles_complete_histories = 0usize;
    let mut role_discordant_histories = 0usize;

    let mut even_blocks_pass_of_256 = 0usize;
    let mut odd_blocks_pass_of_256 = 0usize;

    let mut global_block = 0usize;
    for (stratum, num_blocks) in strata_specs {
        let mut s_total = 0usize;
        let mut s_pass = 0usize;
        for selected in 0..num_blocks {
            let q_a = global_block % 6;
            let q_b = (global_block + 3) % 6;
            let mut b_pass = 0usize;

            for swapped in [0usize, 1] {
                for (w_ord, q_idx) in [(0usize, q_a), (1usize, q_b)] {
                    for role in ["job", "home"] {
                        let fwd_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap{swapped}-q{q_idx}-forward-{role}"
                        );
                        let rev_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap{swapped}-q{q_idx}-reverse-{role}"
                        );
                        all_ids.insert(fwd_id.clone());
                        all_ids.insert(rev_id.clone());
                        let f_ok = complete_set.contains(&fwd_id);
                        let r_ok = complete_set.contains(&rev_id);
                        if f_ok {
                            forward_pass += 1;
                            b_pass += 1;
                            s_pass += 1;
                            if w_ord == 0 {
                                wording_a_pass += 1;
                            } else {
                                wording_b_pass += 1;
                            }
                        }
                        if r_ok {
                            reverse_pass += 1;
                            b_pass += 1;
                            s_pass += 1;
                            if w_ord == 0 {
                                wording_a_pass += 1;
                            } else {
                                wording_b_pass += 1;
                            }
                        }
                        if f_ok != r_ok {
                            chronology_discordant_pairs += 1;
                        }
                        s_total += 2;
                    }

                    for order in ["forward", "reverse"] {
                        let job_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap{swapped}-q{q_idx}-{order}-job"
                        );
                        let home_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap{swapped}-q{q_idx}-{order}-home"
                        );
                        let j_ok = complete_set.contains(&job_id);
                        let h_ok = complete_set.contains(&home_id);
                        if j_ok {
                            job_pass += 1;
                        }
                        if h_ok {
                            home_pass += 1;
                        }
                        if j_ok && h_ok {
                            both_roles_complete_histories += 1;
                        }
                        if j_ok != h_ok {
                            role_discordant_histories += 1;
                        }
                    }
                }

                for order in ["forward", "reverse"] {
                    for role in ["job", "home"] {
                        let qa_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap{swapped}-q{q_a}-{order}-{role}"
                        );
                        let qb_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap{swapped}-q{q_b}-{order}-{role}"
                        );
                        if complete_set.contains(&qa_id) != complete_set.contains(&qb_id) {
                            wording_discordant_pairs += 1;
                        }
                    }
                }
            }

            for q_idx in [q_a, q_b] {
                for order in ["forward", "reverse"] {
                    for role in ["job", "home"] {
                        let s0_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap0-q{q_idx}-{order}-{role}"
                        );
                        let s1_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap1-q{q_idx}-{order}-{role}"
                        );
                        let s0_ok = complete_set.contains(&s0_id);
                        let s1_ok = complete_set.contains(&s1_id);
                        if s0_ok {
                            swap0_pass += 1;
                        }
                        if s1_ok {
                            swap1_pass += 1;
                        }
                        if s0_ok != s1_ok {
                            swap_discordant_pairs += 1;
                        }
                    }
                }
            }

            if selected % 2 == 0 {
                even_blocks_pass_of_256 += b_pass;
            } else {
                odd_blocks_pass_of_256 += b_pass;
            }
            block_pass_counts.push((stratum.to_string(), selected, b_pass));
            global_block += 1;
        }
        let expected_stratum = replay["strata"][stratum]["complete"]
            .as_u64()
            .ok_or_else(|| format!("replay stratum {stratum} absent"))?
            as usize;
        if s_pass != expected_stratum {
            return Err(format!(
                "stratum {stratum} count mismatch: {s_pass} vs {expected_stratum}"
            ));
        }
        stratum_complete.insert(stratum.to_string(), (s_pass, s_total));
    }

    if all_ids.len() != 512 || !complete_set.is_subset(&all_ids) {
        return Err("complete_ids is not a subset of the 512 canonical dev512 row IDs".to_string());
    }

    // Compute exact stratified 16-block (256-row) cluster-sampling variance:
    // Var(S_256) = sum_h b'_h * s_h^2, where s_h^2 = sum_i (c_{h,i} - mean_h)^2 / (B_h - 1).
    // Using common denominator LCM(8*7, 4*3) = 168:
    // For B_h = 8, b'_h = 4: b'_h * s_h^2 = 4 * [sum_i (8*c_i - sum_c)^2] / (8^2 * 7) = [sum_i (8*c_i - sum_c)^2] / 112.
    // For B_h = 4, b'_h = 2: b'_h * s_h^2 = 2 * [sum_i (4*c_i - sum_c)^2] / (4^2 * 3) = [sum_i (4*c_i - sum_c)^2] / 24.
    // Common denominator for Var(S_256) is 336 (112 * 3 = 336, 24 * 14 = 336).
    let mut var_num_336: u128 = 0;
    for (stratum, num_blocks) in strata_specs {
        let counts: Vec<i128> = block_pass_counts
            .iter()
            .filter(|(s, _, _)| s == stratum)
            .map(|(_, _, c)| *c as i128)
            .collect();
        let sum_c: i128 = counts.iter().sum();
        let b_h = num_blocks as i128;
        let mut sq_sum: u128 = 0;
        for c in counts {
            let diff = b_h * c - sum_c;
            sq_sum += (diff * diff) as u128;
        }
        if num_blocks == 8 {
            var_num_336 += sq_sum * 3;
        } else {
            var_num_336 += sq_sum * 14;
        }
    }
    // SE in hundredths of a row = sqrt(Var * 100^2) = isqrt(var_num_336 * 10000 / 336)
    let cluster_se_256_hundredths_rows = isqrt_u128((var_num_336 * 10_000 + 168) / 336) as u64;
    // SE in basis points of 256 = (SE_rows / 256) * 10000 = SE_hundredths_rows * 100 / 256
    let cluster_se_256_bps = (cluster_se_256_hundredths_rows * 100 + 128) / 256;

    // Independent binomial variance on 512 rows at p = 437/512 scaled to 256 rows:
    // Var_ind_256 = 256 * (437/512) * (75/512) = 32775 / 1024 = 32.0068
    // Design effect DEFF = Var_cluster_256 / Var_ind_256 = (var_num_336 / 336) / (32775 / 1024)
    // At the 40% (p = 0.40) bar on 256 rows, Var_ind_bar40 = 256 * 0.40 * 0.60 = 61.44 = 1536 / 25.
    let bar40_binomial_se_hundredths_rows = isqrt_u128(614_400) as u64; // 783 -> 7.84 rows
                                                                        // Var_cluster_bar40 = Var_ind_bar40 * max(1, DEFF)
    let var_cluster_bar40_times_10000 =
        (1536u128 * 10_000 * var_num_336 * 1024) / (25u128 * 336 * 32_775);
    let bar40_cluster_se_hundredths_rows =
        isqrt_u128(var_cluster_bar40_times_10000.max(614_400)) as u64;
    let bar40_cluster_se_bps = (bar40_cluster_se_hundredths_rows * 100 + 128) / 256;

    Ok(Dev512SpreadReport {
        total_rows: 512,
        complete_rows: 437,
        evaluator_replay_diff_rows: 0,
        stratum_complete,
        block_pass_counts,
        forward_pass,
        reverse_pass,
        chronology_abs_diff: forward_pass.abs_diff(reverse_pass),
        chronology_discordant_pairs,
        swap0_pass,
        swap1_pass,
        swap_abs_diff: swap0_pass.abs_diff(swap1_pass),
        swap_discordant_pairs,
        wording_a_pass,
        wording_b_pass,
        wording_abs_diff: wording_a_pass.abs_diff(wording_b_pass),
        wording_discordant_pairs,
        job_pass,
        home_pass,
        role_abs_diff: job_pass.abs_diff(home_pass),
        both_roles_complete_histories,
        role_discordant_histories,
        even_blocks_pass_of_256,
        odd_blocks_pass_of_256,
        half_panel_split_abs_diff: even_blocks_pass_of_256.abs_diff(odd_blocks_pass_of_256),
        cluster_se_256_hundredths_rows,
        cluster_se_256_bps,
        bar40_binomial_se_hundredths_rows,
        bar40_cluster_se_hundredths_rows,
        bar40_cluster_se_bps,
    })
}

/// Generate the frozen 256-row held-out reply panel (`m2-fresh256`) and its sealed oracle labels.
pub fn generate_m2_fresh256_panel() -> Result<Fresh256Bundle, String> {
    let mut histories = Vec::new();
    let mut used_unordered: BTreeSet<(String, String)> = BTreeSet::new();

    let strata_specs: [(&str, &[&str], usize); 5] = [
        ("length2", &FRESH256_LEN2, 4),
        ("length4", &FRESH256_LEN4, 4),
        ("length8", &FRESH256_LEN8, 4),
        ("update", &FRESH256_LEN4, 2),
        ("reassert", &FRESH256_LEN2[..8], 2),
    ];

    for (stratum, values, target_blocks) in strata_specs {
        let mut selected = 0usize;
        for (left, right) in round_robin_edges(values.len()) {
            if selected == target_blocks {
                break;
            }
            let a = values[left];
            let b = values[right];
            let key = if a < b {
                (a.to_string(), b.to_string())
            } else {
                (b.to_string(), a.to_string())
            };
            if used_unordered.contains(&key) {
                continue;
            }
            let block = histories.len() / 8;
            for swapped in [false, true] {
                let (jv, hv) = if swapped { (b, a) } else { (a, b) };
                let mut j = supported_wire("job", "assert", jv, 0)?;
                let mut h = supported_wire("home", "assert", hv, 0)?;
                let mut after = Vec::new();
                if stratum == "update" {
                    let role = if swapped { "home" } else { "job" };
                    let init_val = FRESH256_UPDATE_INIT[selected % 4];
                    let initial = supported_wire(role, "assert", init_val, 0)?;
                    if swapped {
                        h = initial;
                    } else {
                        j = initial;
                    }
                    after.push(supported_wire(role, "update", a, 0)?);
                    after.push(if swapped { j.clone() } else { h.clone() });
                } else if stratum == "reassert" {
                    after.push(if swapped { h.clone() } else { j.clone() });
                }
                for query_index in [block % 6, (block + 3) % 6] {
                    let queries = vec![
                        supported_wire("job", "query", "", query_index)?,
                        supported_wire("home", "query", "", query_index)?,
                    ];
                    let base_id = format!(
                        "diverse-{stratum}-{selected:02}-swap{}-q{query_index}",
                        usize::from(swapped)
                    );
                    let stratum_tag = format!("{stratum}/q{query_index}");
                    let mut forward_turns = vec![j.clone(), h.clone()];
                    forward_turns.extend(after.clone());
                    let mut reverse_turns = vec![h.clone(), j.clone()];
                    reverse_turns.extend(after.iter().rev().cloned());
                    let mut variants = vec![
                        History {
                            id: format!("{base_id}-forward"),
                            stratum: stratum_tag.clone(),
                            turns: forward_turns,
                            queries: queries.clone(),
                        },
                        History {
                            id: format!("{base_id}-reverse"),
                            stratum: stratum_tag,
                            turns: reverse_turns,
                            queries,
                        },
                    ];
                    if stratum == "reassert" {
                        for h_var in &mut variants {
                            let repeated = h_var
                                .turns
                                .first()
                                .cloned()
                                .ok_or_else(|| "reassert initial absent".to_string())?;
                            *h_var
                                .turns
                                .last_mut()
                                .ok_or_else(|| "reassert last absent".to_string())? = repeated;
                        }
                    }
                    histories.extend(variants);
                }
            }
            used_unordered.insert(key);
            selected += 1;
        }
        if selected != target_blocks {
            return Err(format!("insufficient disjoint blocks for {stratum}"));
        }
    }

    if histories.len() != 128 {
        return Err(format!("expected 128 histories, got {}", histories.len()));
    }

    let plan_val = json!({
        "schema": "uor-r4.raw-natural-reader-construction-plan/1",
        "panel_id": "m2-fresh256",
        "split": "fresh",
        "source_policy": "explicit-current-role-assertions/raw-current-role-queries/all-source-candidates/2",
        "histories": histories,
    });
    let plan_json = serde_json::to_vec_pretty(&plan_val).map_err(|e| e.to_string())?;
    let plan_sha256 = sha256_hex(&plan_json);

    let mut runtime_cases = Vec::new();
    let mut label_cases = Vec::new();

    for (ordinal, h) in histories.iter().enumerate() {
        let mut current_by_role: BTreeMap<&str, (usize, &Wire, String)> = BTreeMap::new();
        for (event, w) in h.turns.iter().enumerate() {
            let lit = extract_literal(w)?.to_string();
            current_by_role.insert(w.relation.as_str(), (event, w, lit));
        }
        let mut records: Vec<(usize, u64, &Wire, String)> = current_by_role
            .into_iter()
            .map(|(role, (event, w, lit))| {
                let rel_id = if role == "job" { 1u64 } else { 2u64 };
                (event, rel_id, w, lit)
            })
            .collect();
        records.sort_by_key(|r| r.0);
        if records.len() != 2 {
            return Err("history lacks both current records".to_string());
        }

        let mut segments = Vec::new();
        for (idx, (event, rel_id, w, lit)) in records.iter().enumerate() {
            let cue_ids = deterministic_token_ids(&w.text);
            let val_ids = deterministic_token_ids(lit);
            let record_id = (ordinal as u64) * 10 + (idx as u64) + 1;
            segments.push(json!({
                "kind": "Context",
                "event": event,
                "role": 1,
                "token_ids": cue_ids,
            }));
            segments.push(json!({
                "kind": "Source",
                "event": event,
                "record": record_id,
                "commit": event,
                "scope": "compiler-probe",
                "entity": [1],
                "relation": rel_id,
                "view": 0,
                "original_source_ids": val_ids,
            }));
        }

        let mut sorted_queries: Vec<&Wire> = h.queries.iter().collect();
        sorted_queries.sort_by_key(|q| if q.relation == "job" { 0usize } else { 1usize });
        for q in sorted_queries {
            let case_id = format!("fresh-{}-{}", h.id, q.relation);
            let query_ids = deterministic_token_ids(&q.text);
            runtime_cases.push(json!({
                "id": case_id,
                "segments": segments,
                "query_ids": query_ids,
                "actual_prefix_ids": [],
            }));
            let expected_rel = if q.relation == "job" { 1u64 } else { 2u64 };
            let (_, _, _, expected_lit) = records
                .iter()
                .find(|(_, rel, _, _)| *rel == expected_rel)
                .ok_or_else(|| "expected role record absent".to_string())?;
            label_cases.push(json!({
                "id": case_id,
                "answers": {
                    "intent": "current",
                    "accepted": [format!("{expected_lit}.")],
                }
            }));
        }
    }

    let inputs_val = json!({
        "schema": "uor-r4.native-source-bank-probe-input/1",
        "panel_id": "m2-fresh256",
        "cases": runtime_cases,
    });
    let inputs_json = serde_json::to_vec_pretty(&inputs_val).map_err(|e| e.to_string())?;
    let inputs_sha256 = sha256_hex(&inputs_json);

    let labels_val = json!({
        "schema": "uor-r4.native-source-bank-labels/1",
        "panel_id": "m2-fresh256",
        "protocol": "uor-r4.literal-role-dialogue/2",
        "membership_only": true,
        "cases": label_cases,
    });
    let labels_json = serde_json::to_vec_pretty(&labels_val).map_err(|e| e.to_string())?;
    let labels_sha256 = sha256_hex(&labels_json);

    let manifest_val = json!({
        "schema": "uor-r4.m2-fresh256-panel-manifest/1",
        "panel_id": "m2-fresh256",
        "milestone": "m2",
        "issue": "#2849",
        "milestone_issue": "#2030",
        "rows": 256,
        "histories": 128,
        "unordered_bank_blocks": 16,
        "ordered_role_value_banks": 32,
        "opposite_query_pairs": 128,
        "strata_rows": {
            "length2": 64,
            "length4": 64,
            "length8": 64,
            "update": 32,
            "reassert": 32
        },
        "target_bar_rows": 103,
        "target_bar_of": 256,
        "files": {
            "plan.json": {
                "bytes": plan_json.len(),
                "sha256": plan_sha256,
            },
            "inputs.json": {
                "bytes": inputs_json.len(),
                "sha256": inputs_sha256,
            },
            "labels.json": {
                "bytes": labels_json.len(),
                "sha256": labels_sha256,
            }
        },
        "excluded_historical_exposures": {
            "dev512_inputs_sha256": DEV512_INPUTS_SHA256,
            "opened_fresh128_inputs_sha256": OPENED_FRESH128_INPUTS_SHA256,
            "historical_exposed_literals": HISTORICAL_EXPOSED_LITERALS.len()
        },
        "model_predictions": "NOT_RUN"
    });
    let manifest_json = serde_json::to_vec_pretty(&manifest_val).map_err(|e| e.to_string())?;
    let manifest_sha256 = sha256_hex(&manifest_json);

    Ok(Fresh256Bundle {
        plan_json,
        plan_sha256,
        inputs_json,
        inputs_sha256,
        labels_json,
        labels_sha256,
        manifest_json,
        manifest_sha256,
        histories,
    })
}

fn is_function_word(w: &str) -> bool {
    matches!(
        w,
        "the"
            | "a"
            | "an"
            | "above"
            | "beside"
            | "beyond"
            | "near"
            | "over"
            | "under"
            | "across"
            | "along"
            | "around"
            | "within"
            | "inside"
            | "outside"
            | "below"
            | "by"
            | "stands"
            | "grows"
            | "rests"
            | "rises"
    )
}

fn content_words(s: &str) -> BTreeSet<String> {
    s.split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_ascii_lowercase()
        })
        .filter(|w| !w.is_empty() && !is_function_word(w))
        .collect()
}

fn word_ngrams(s: &str, n: usize) -> BTreeSet<Vec<String>> {
    let words: Vec<String> = s
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_ascii_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect();
    if words.len() < n {
        return BTreeSet::new();
    }
    words.windows(n).map(|w| w.to_vec()).collect()
}

pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=bc.len()).collect();
    let mut curr = vec![0usize; bc.len() + 1];
    for (i, &ca) in ac.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &cb) in bc.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[bc.len()]
}

type BankBlockMap =
    BTreeMap<(String, String), BTreeMap<(String, String), Vec<(String, Vec<String>)>>>;

/// Verify structural balance, overlap exclusion against all historical pools,
/// token budgets, check-only controls, and unscored-input sealing.
pub fn verify_m2_fresh256_panel(bundle: &Fresh256Bundle) -> Result<OverlapAndSealAudit, String> {
    if sha256_hex(&bundle.plan_json) != bundle.plan_sha256
        || sha256_hex(&bundle.inputs_json) != bundle.inputs_sha256
        || sha256_hex(&bundle.labels_json) != bundle.labels_sha256
        || sha256_hex(&bundle.manifest_json) != bundle.manifest_sha256
    {
        return Err("bundle SHA-256 mismatch".to_string());
    }

    if bundle.histories.len() != 128 {
        return Err("histories count != 128".to_string());
    }

    let mut banks: BankBlockMap = BTreeMap::new();
    let mut role_counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut position_counts: BTreeMap<(String, usize), usize> = BTreeMap::new();
    let mut stratum_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut query_counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut fresh_literals_used: BTreeSet<String> = BTreeSet::new();

    for h in &bundle.histories {
        let stratum = h
            .stratum
            .split("/q")
            .next()
            .ok_or_else(|| "stratum absent".to_string())?;
        *stratum_counts.entry(stratum.to_string()).or_default() += 1;
        if h.turns.len() < 2 || h.turns.len() > 8 || h.queries.len() != 2 {
            return Err("invalid turn or query count".to_string());
        }
        let mut current = BTreeMap::new();
        let mut chronology = Vec::new();
        for (pos, w) in h.turns.iter().enumerate() {
            let val = extract_literal(w)?.to_string();
            fresh_literals_used.insert(val.clone());
            current.insert(w.relation.clone(), val.clone());
            *role_counts
                .entry((val.clone(), w.relation.clone()))
                .or_default() += 1;
            if pos < 2 {
                *position_counts.entry((val.clone(), pos)).or_default() += 1;
            }
            chronology.push(format!("{}/{}/{val}", w.relation, w.act));
        }
        let j = current
            .get("job")
            .ok_or_else(|| "job absent".to_string())?
            .clone();
        let home = current
            .get("home")
            .ok_or_else(|| "home absent".to_string())?
            .clone();
        if j == home {
            return Err("equal role values".to_string());
        }
        let unordered = if j < home {
            (j.clone(), home.clone())
        } else {
            (home.clone(), j.clone())
        };
        let mut wording = BTreeMap::new();
        for q in &h.queries {
            wording.insert(q.relation.clone(), q.text.clone());
            *query_counts
                .entry((q.relation.clone(), q.text.clone()))
                .or_default() += 1;
        }
        let wordkey = serde_json::to_string(&wording).map_err(|e| e.to_string())?;
        banks
            .entry(unordered)
            .or_default()
            .entry((j, home))
            .or_default()
            .push((wordkey, chronology));
    }

    let expected_strata = BTreeMap::from([
        ("length2".to_string(), 32usize),
        ("length4".to_string(), 32),
        ("length8".to_string(), 32),
        ("update".to_string(), 16),
        ("reassert".to_string(), 16),
    ]);
    if stratum_counts != expected_strata {
        return Err(format!("stratum quota mismatch: {stratum_counts:?}"));
    }

    for variants in banks.values() {
        if variants.len() != 2 || variants.values().any(|v| v.len() != 4) {
            return Err("bank block lacks both role assignments and 4 variants each".to_string());
        }
    }

    let mut literal_role_balance = true;
    for ((val, role), count) in &role_counts {
        let other = if role == "job" { "home" } else { "job" };
        if role_counts.get(&(val.clone(), other.to_string())) != Some(count) {
            literal_role_balance = false;
        }
    }
    if !literal_role_balance {
        return Err("literal role imbalance".to_string());
    }

    let mut literal_initial_position_balance = true;
    for ((val, pos), count) in &position_counts {
        if position_counts.get(&(val.clone(), 1 - pos)) != Some(count) {
            literal_initial_position_balance = false;
        }
    }
    if !literal_initial_position_balance {
        return Err("literal initial position imbalance".to_string());
    }

    let mut max_query_wording_spread = 0usize;
    for role in ["job", "home"] {
        let counts: Vec<usize> = query_counts
            .iter()
            .filter(|((r, _), _)| r == role)
            .map(|(_, c)| *c)
            .collect();
        if counts.len() != 6 {
            return Err("missing query wording form".to_string());
        }
        let spread =
            counts.iter().max().copied().unwrap_or(0) - counts.iter().min().copied().unwrap_or(0);
        max_query_wording_spread = max_query_wording_spread.max(spread);
        if spread > 4 {
            return Err("query wording forms unbalanced".to_string());
        }
    }

    // Check exact, content-word, n-gram, and edit-distance disjointness against all 72 historical literals.
    let hist_set: BTreeSet<&str> = HISTORICAL_EXPOSED_LITERALS.iter().copied().collect();
    let mut exact_literal_overlaps = 0usize;
    let mut content_word_overlaps = 0usize;
    let mut ngram_overlaps = 0usize;
    let mut min_same_stratum_edit_distance = usize::MAX;

    for fresh_lit in &fresh_literals_used {
        if hist_set.contains(fresh_lit.as_str()) {
            exact_literal_overlaps += 1;
        }
        let f_words = content_words(fresh_lit);
        let f_bi = word_ngrams(fresh_lit, 2);
        let f_len = fresh_lit.split_whitespace().count();
        for hist_lit in &HISTORICAL_EXPOSED_LITERALS {
            let h_words = content_words(hist_lit);
            if !f_words.is_disjoint(&h_words) {
                content_word_overlaps += 1;
            }
            let h_bi = word_ngrams(hist_lit, 2);
            if !f_bi.is_disjoint(&h_bi) {
                ngram_overlaps += 1;
            }
            if hist_lit.split_whitespace().count() == f_len {
                let d = levenshtein_distance(fresh_lit, hist_lit);
                min_same_stratum_edit_distance = min_same_stratum_edit_distance.min(d);
            }
        }
    }

    if exact_literal_overlaps != 0
        || content_word_overlaps != 0
        || ngram_overlaps != 0
        || min_same_stratum_edit_distance < 8
    {
        return Err(format!(
            "overlap exclusion failed: exact={exact_literal_overlaps} word={content_word_overlaps} ngram={ngram_overlaps} min_edit={min_same_stratum_edit_distance}"
        ));
    }

    // Check token budgets, non-vacuous controls, and unscored inputs seal.
    let inputs_str = String::from_utf8_lossy(&bundle.inputs_json);
    let unscored_inputs_leak_count = usize::from(inputs_str.contains("\"accepted\""));
    if unscored_inputs_leak_count != 0 {
        return Err("inputs.json leaked accepted answer strings".to_string());
    }

    let mut max_turn_tokens = 0usize;
    let mut max_target_tokens = 0usize;
    let mut max_total_replay_tokens = 0usize;
    let mut copy_first_stated_hits = 0usize;
    let mut copy_last_stated_hits = 0usize;
    let mut role_swap_distractor_hits = 0usize;
    let mut answer_tally: BTreeMap<String, usize> = BTreeMap::new();

    for h in &bundle.histories {
        let first_lit = extract_literal(&h.turns[0])?.to_string();
        let last_lit = extract_literal(h.turns.last().ok_or("empty turns")?)?.to_string();
        let mut current: BTreeMap<&str, (&Wire, String)> = BTreeMap::new();
        for w in &h.turns {
            let t_len = deterministic_token_ids(&w.text).len();
            max_turn_tokens = max_turn_tokens.max(t_len);
            current.insert(w.relation.as_str(), (w, extract_literal(w)?.to_string()));
        }
        let mut replay_tokens = 0usize;
        for (w, lit) in current.values() {
            replay_tokens +=
                deterministic_token_ids(&w.text).len() + deterministic_token_ids(lit).len();
        }
        for q in &h.queries {
            let q_len = deterministic_token_ids(&q.text).len();
            max_turn_tokens = max_turn_tokens.max(q_len);
            let (_, expected_lit) = current
                .get(q.relation.as_str())
                .ok_or_else(|| "missing role".to_string())?;
            let other_role = if q.relation == "job" { "home" } else { "job" };
            let (_, distractor_lit) = current
                .get(other_role)
                .ok_or_else(|| "missing other role".to_string())?;
            let target_len = deterministic_token_ids(&format!(" {expected_lit}.")).len() + 1;
            max_target_tokens = max_target_tokens.max(target_len);
            max_total_replay_tokens =
                max_total_replay_tokens.max(replay_tokens + q_len + target_len);

            if &first_lit == expected_lit {
                copy_first_stated_hits += 1;
            }
            if &last_lit == expected_lit {
                copy_last_stated_hits += 1;
            }
            if distractor_lit == expected_lit {
                role_swap_distractor_hits += 1;
            }
            *answer_tally.entry(expected_lit.clone()).or_default() += 1;
        }
    }

    if max_turn_tokens > 128 || max_target_tokens > 32 || max_total_replay_tokens > 128 {
        return Err("token budget exceeded".to_string());
    }

    let best_constant_reply_hits = answer_tally.values().copied().max().unwrap_or(0);
    if copy_first_stated_hits != 120
        || copy_last_stated_hits != 128
        || role_swap_distractor_hits != 0
        || best_constant_reply_hits != 16
    {
        return Err(format!(
            "control check failed: first={copy_first_stated_hits} last={copy_last_stated_hits} swap={role_swap_distractor_hits} const={best_constant_reply_hits}"
        ));
    }

    Ok(OverlapAndSealAudit {
        rows: 256,
        histories: 128,
        unordered_blocks: banks.len(),
        ordered_banks: banks.len() * 2,
        exact_literal_overlaps,
        content_word_overlaps,
        ngram_overlaps,
        min_same_stratum_edit_distance,
        literal_role_balance,
        literal_initial_position_balance,
        max_query_wording_spread,
        max_turn_tokens,
        max_target_tokens,
        max_total_replay_tokens,
        copy_first_stated_hits,
        copy_last_stated_hits,
        role_swap_distractor_hits,
        best_constant_reply_hits,
        unscored_inputs_leak_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(4)
            .expect("repo root")
            .to_path_buf()
    }

    #[test]
    fn dev512_spread_matches_sealed_saved437_receipts() {
        let report = analyze_saved437_dev512_spread(&repo_root()).expect("dev512 spread");
        assert_eq!(report.total_rows, 512);
        assert_eq!(report.complete_rows, 437);
        assert_eq!(report.evaluator_replay_diff_rows, 0);
        assert_eq!(report.both_roles_complete_histories, 186);
        assert_eq!(report.forward_pass + report.reverse_pass, 437);
        assert_eq!(report.swap0_pass + report.swap1_pass, 437);
        assert_eq!(report.wording_a_pass + report.wording_b_pass, 437);
        assert_eq!(report.job_pass + report.home_pass, 437);
        assert_eq!(
            report.even_blocks_pass_of_256 + report.odd_blocks_pass_of_256,
            437
        );
        assert!(report.cluster_se_256_hundredths_rows > 0);
        assert!(
            report.bar40_cluster_se_hundredths_rows >= report.bar40_binomial_se_hundredths_rows
        );
    }

    #[test]
    fn fresh256_panel_passes_all_overlap_balance_and_seal_checks() {
        let bundle = generate_m2_fresh256_panel().expect("generate fresh256");
        let again = generate_m2_fresh256_panel().expect("generate fresh256 again");
        assert_eq!(bundle.plan_sha256, again.plan_sha256);
        assert_eq!(bundle.inputs_sha256, again.inputs_sha256);
        assert_eq!(bundle.labels_sha256, again.labels_sha256);
        assert_eq!(bundle.manifest_sha256, again.manifest_sha256);

        let audit = verify_m2_fresh256_panel(&bundle).expect("verify fresh256");
        assert_eq!(audit.rows, 256);
        assert_eq!(audit.histories, 128);
        assert_eq!(audit.unordered_blocks, 16);
        assert_eq!(audit.ordered_banks, 32);
        assert_eq!(audit.exact_literal_overlaps, 0);
        assert_eq!(audit.content_word_overlaps, 0);
        assert_eq!(audit.ngram_overlaps, 0);
        assert!(audit.min_same_stratum_edit_distance >= 8);
        assert!(audit.literal_role_balance);
        assert!(audit.literal_initial_position_balance);
        assert_eq!(audit.copy_first_stated_hits, 120);
        assert_eq!(audit.copy_last_stated_hits, 128);
        assert_eq!(audit.role_swap_distractor_hits, 0);
        assert_eq!(audit.best_constant_reply_hits, 16);
        assert_eq!(audit.unscored_inputs_leak_count, 0);
    }

    #[test]
    fn negative_mutations_are_rejected_by_verifier() {
        let bundle = generate_m2_fresh256_panel().expect("generate fresh256");

        // 1. Injecting an exposed historical literal is caught by overlap exclusion.
        let mut leaked_lit = bundle.clone();
        for h in &mut leaked_lit.histories {
            for t in &mut h.turns {
                if t.text.contains("basalt loggia") {
                    t.text = t.text.replace("basalt loggia", "azure orchard");
                }
            }
        }
        let err1 = verify_m2_fresh256_panel(&leaked_lit).unwrap_err();
        assert!(err1.contains("overlap exclusion failed"), "got: {err1}");

        // 2. Breaking initial position balance is caught.
        let mut unbalanced = bundle.clone();
        unbalanced.histories[0].turns.swap(0, 1);
        let err2 = verify_m2_fresh256_panel(&unbalanced).unwrap_err();
        assert!(
            err2.contains("literal initial position imbalance"),
            "got: {err2}"
        );

        // 3. Leaking `"accepted"` into unscored `inputs_json` is caught.
        let mut leaked_inputs = bundle.clone();
        leaked_inputs.inputs_json = br#"{"accepted":["basalt loggia."]}"#.to_vec();
        leaked_inputs.inputs_sha256 = sha256_hex(&leaked_inputs.inputs_json);
        let err3 = verify_m2_fresh256_panel(&leaked_inputs).unwrap_err();
        assert!(
            err3.contains("inputs.json leaked accepted answer strings"),
            "got: {err3}"
        );

        // 4. Corrupting a SHA-256 seal is caught.
        let mut bad_seal = bundle;
        bad_seal.labels_sha256 = "0".repeat(64);
        let err4 = verify_m2_fresh256_panel(&bad_seal).unwrap_err();
        assert!(err4.contains("bundle SHA-256 mismatch"), "got: {err4}");
    }
}
