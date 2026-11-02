//! Independent CPU Route and Flock reads for four-table discrete artifacts.

use uor_r4_core::spiralcore_operator::{cl06_finite_composition_table, Cl06FiniteCompositionTable};
use uor_r4_discrete::normalization_mechanism::{orbit_root_coords, NormFrameTable};

use crate::CpuTrainError;

type Result<T> = std::result::Result<T, CpuTrainError>;

fn invalid(message: impl Into<String>) -> CpuTrainError {
    CpuTrainError::Invalid(message.into())
}

const BFS: [u8; 256] = [
    0, 16, 7, 42, 3, 8, 9, 27, 5, 24, 4, 47, 2, 12, 1, 34, 16, 0, 34, 1, 12, 2, 47, 4, 24, 5, 27,
    9, 8, 3, 42, 7, 34, 7, 0, 26, 11, 9, 14, 10, 6, 4, 5, 2, 47, 1, 3, 16, 1, 42, 3, 0, 15, 4, 5,
    8, 13, 9, 2, 10, 27, 7, 16, 48, 12, 3, 1, 7, 0, 5, 6, 9, 2, 41, 13, 4, 35, 16, 15, 11, 2, 8,
    47, 27, 24, 0, 7, 1, 3, 12, 42, 34, 16, 5, 4, 9, 47, 9, 2, 10, 4, 34, 0, 3, 11, 1, 26, 16, 7,
    6, 5, 14, 4, 27, 5, 2, 13, 42, 48, 0, 7, 15, 16, 3, 1, 9, 8, 10, 35, 5, 4, 9, 41, 12, 1, 15, 0,
    16, 7, 11, 3, 2, 13, 6, 5, 35, 6, 13, 2, 3, 11, 7, 16, 0, 15, 1, 12, 41, 9, 4, 27, 4, 10, 8, 9,
    1, 3, 16, 15, 7, 0, 48, 42, 13, 2, 5, 9, 47, 14, 5, 6, 7, 16, 26, 1, 11, 3, 0, 34, 4, 10, 2, 8,
    2, 9, 4, 5, 16, 34, 42, 12, 3, 1, 7, 0, 24, 27, 47, 3, 12, 11, 15, 16, 35, 4, 13, 41, 2, 9, 6,
    5, 0, 7, 1, 42, 1, 48, 16, 7, 27, 10, 2, 9, 13, 8, 5, 4, 15, 0, 3, 7, 34, 16, 3, 1, 47, 2, 5,
    4, 6, 10, 14, 9, 11, 26, 0,
];

const FLOCK_SNAPSHOT_SCHEMA: &str = "uor-r4.discrete-spiralcore-fbs/2";
const FLOCK_OPERATOR_KAPPA: &str =
    "blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b";
const FLOCK_COMPOSITION_KAPPA: &str =
    "blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0";
const RELATION_EDGE_TYPES: usize = 5;

const BRANCH: [[u8; 3]; 16] = [
    [0, 14, 3],
    [1, 3, 14],
    [2, 13, 4],
    [3, 0, 1],
    [4, 2, 15],
    [5, 7, 10],
    [6, 9, 8],
    [7, 12, 5],
    [8, 6, 11],
    [9, 11, 6],
    [10, 5, 12],
    [11, 8, 9],
    [12, 10, 7],
    [13, 15, 2],
    [14, 1, 0],
    [15, 4, 13],
];

#[derive(Clone)]
pub struct CpuDiscreteRoute {
    context: usize,
    blocks: usize,
    stages: usize,
    token_blocks: Vec<Vec<[i32; 8]>>,
    routes: Vec<Vec<u32>>,
    composition: Cl06FiniteCompositionTable,
}

#[derive(Clone)]
struct BinderNode {
    children: Option<[usize; 3]>,
    atom: Option<usize>,
    level: usize,
    first: usize,
    last: usize,
}

#[derive(Clone)]
struct RowPlan {
    nodes: Vec<BinderNode>,
    frontier: Vec<usize>,
}

impl CpuDiscreteRoute {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let header = header(bytes)?;
        if header["schema"].as_str() != Some("uor-r4.lut-discrete/1") {
            return Err(invalid("Route CPU read requires uor-r4.lut-discrete/1"));
        }
        let shape = &header["shape"];
        if shape["read_mode"].as_str() != Some("route")
            || shape["read_initialization"].as_str() != Some("zero_gates_v1")
        {
            return Err(invalid("Route CPU read refuses non-Route initialization"));
        }
        let vocab = usize::try_from(shape["vocabulary"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("vocabulary"))?;
        let blocks = usize::try_from(shape["blocks"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("blocks"))?;
        let context = usize::try_from(shape["context"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("context"))?;
        let stages = usize::try_from(shape["stages"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("stages"))?;
        let kept_cells = usize::try_from(shape["kept_cells"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("kept_cells"))?;
        if !(2..=8192).contains(&vocab)
            || !(2..=16).contains(&blocks)
            || !(1..=243).contains(&context)
            || !(1..=16).contains(&kept_cells)
            || kept_cells > vocab
            || shape["read_kept"].as_u64() != Some(0)
            || shape["read_sweeps"].as_u64() != Some(0)
            || expected_stages(context) != Some(stages)
        {
            return Err(invalid("Route shape differs"));
        }
        let tables = header["tables"]
            .as_array()
            .ok_or_else(|| invalid("Route artifact tables differ"))?;
        if tables.len() != 4 {
            return Err(invalid("Route artifact requires four tables"));
        }
        let table = section_i32(bytes, &header, "discrete.table", vocab * blocks * 8, "i32")?;
        let cells = section_u32(bytes, &header, "discrete.cells", vocab, "u32")?;
        let routes = section_u32(bytes, &header, "discrete.routes", blocks * stages, "u32")?;
        let gates = section_i32(bytes, &header, "discrete.gates", blocks * stages, "i32")?;
        if gates.iter().any(|gate| *gate != 0) {
            return Err(invalid("Route initialization requires zero gates"));
        }
        if routes.iter().any(|&action| action >= 64) {
            return Err(invalid("Route action outside 0..64"));
        }
        let mut token_blocks = Vec::with_capacity(vocab);
        for token in 0..vocab {
            let mut values = Vec::with_capacity(blocks);
            for block in 0..blocks {
                let start = (token * blocks + block) * 8;
                let coords: [i32; 8] = table[start..start + 8]
                    .try_into()
                    .map_err(|_| invalid("Route codebook width"))?;
                if root_id(&coords).is_none() {
                    return Err(invalid("Route codebook block outside orbit"));
                }
                values.push(coords);
            }
            token_blocks.push(values);
        }
        for (token, (blocks, &cell)) in token_blocks.iter().zip(&cells).enumerate() {
            let first = root_id(&blocks[0]).unwrap() as u32;
            let last = root_id(blocks.last().unwrap()).unwrap() as u32;
            if cell != first * 16 + last {
                return Err(invalid(format!("Route cell for token {token} differs")));
            }
        }
        validate_layout(bytes, &header)?;
        let routes = routes
            .chunks_exact(stages)
            .map(|row| row.to_vec())
            .collect();
        Self::from_parts(context, token_blocks, routes)
    }

    pub fn from_parts(
        context: usize,
        token_blocks: Vec<Vec<[i32; 8]>>,
        routes: Vec<Vec<u32>>,
    ) -> Result<Self> {
        let blocks = token_blocks.first().map_or(0, Vec::len);
        let stages = routes.first().map_or(0, Vec::len);
        if !(2..=8192).contains(&token_blocks.len())
            || !(2..=16).contains(&blocks)
            || expected_stages(context) != Some(stages)
            || routes.len() != blocks
            || routes
                .iter()
                .any(|row| row.len() != stages || row.iter().any(|&value| value >= 64))
            || token_blocks.iter().any(|token| {
                token.len() != blocks || token.iter().any(|block| root_id(block).is_none())
            })
        {
            return Err(invalid("Route parts differ"));
        }
        let composition =
            cl06_finite_composition_table().map_err(|error| invalid(error.to_string()))?;
        Ok(Self {
            context,
            blocks,
            stages,
            token_blocks,
            routes,
            composition,
        })
    }

    pub fn read(&self, prefix: &[u32]) -> Result<Vec<i32>> {
        if prefix.is_empty() || prefix.len() > self.context {
            return Err(invalid("Route prefix outside context"));
        }
        if prefix
            .iter()
            .any(|&token| token as usize >= self.token_blocks.len())
        {
            return Err(invalid("Route prefix token outside vocabulary"));
        }
        let plan = build_row_plan(prefix.len())?;
        let last = prefix[prefix.len() - 1] as usize;
        let mut out = Vec::with_capacity(self.blocks * 8);
        for block in 0..self.blocks {
            let query = &self.token_blocks[last][block];
            let mut node = 0usize;
            let mut root = 0usize;
            let mut total = 0usize;
            let mut resolved = None;
            for stage in 0..self.stages {
                let q = self.act(self.routes[block][stage] as usize, query)?;
                let count = if stage == 0 { plan.frontier.len() } else { 3 };
                let mut best = 0usize;
                let mut best_distance = u32::MAX;
                let mut best_hamming = u32::MAX;
                let mut goal = 0usize;
                #[allow(clippy::needless_range_loop)]
                for port in 0..count {
                    let r = if stage == 0 {
                        port
                    } else {
                        usize::from(BRANCH[root][port])
                    };
                    let (distance, hamming) = root_cost(&q, r);
                    if distance < best_distance
                        || (distance == best_distance && hamming < best_hamming)
                    {
                        best = port;
                        best_distance = distance;
                        best_hamming = hamming;
                        goal = r;
                    }
                }
                total = usize::from(
                    self.composition
                        .compose_index(BFS[(root << 4) + goal], total as u8)
                        .ok_or_else(|| invalid("Route composition failed"))?,
                );
                root = goal;
                let next = if stage == 0 {
                    plan.frontier[best]
                } else {
                    plan.nodes[node]
                        .children
                        .ok_or_else(|| invalid("Route stepped past atom"))?[best]
                };
                if let Some(atom) = plan.nodes[next].atom {
                    let carrier = &self.token_blocks[prefix[atom] as usize][block];
                    resolved = Some(self.act(total, carrier)?);
                    break;
                }
                node = next;
            }
            out.extend_from_slice(
                &resolved.ok_or_else(|| invalid("Route did not reach an atom leaf"))?,
            );
        }
        Ok(out)
    }

    fn act(&self, action: usize, input: &[i32; 8]) -> Result<[i32; 8]> {
        let root: [i8; 8] = std::array::from_fn(|index| (input[index] / 2) as i8);
        let out = self
            .composition
            .state(action as u8)
            .ok_or_else(|| invalid("Route action state absent"))?
            .apply_root_key(root)
            .map_err(|error| invalid(error.to_string()))?;
        Ok(out.map(|value| i32::from(value) * 2))
    }
}

#[derive(Clone)]
enum ActiveLeafMode {
    Flock,
    Relation {
        sweeps: usize,
        relation_gates: Vec<[i32; RELATION_EDGE_TYPES]>,
    },
}

#[derive(Clone, Copy)]
enum ActiveLeafKind {
    Flock,
    Relation,
}

impl ActiveLeafKind {
    fn label(self) -> &'static str {
        match self {
            Self::Flock => "Flock",
            Self::Relation => "Relation",
        }
    }

    fn mode_name(self) -> &'static str {
        match self {
            Self::Flock => "flock",
            Self::Relation => "relation",
        }
    }
}

#[derive(Clone)]
struct ActiveLeafModel {
    context: usize,
    blocks: usize,
    stages: usize,
    kept: usize,
    token_blocks: Vec<Vec<[i32; 8]>>,
    level_gates: Vec<Vec<i32>>,
    mode: ActiveLeafMode,
    composition: Cl06FiniteCompositionTable,
}

#[derive(Clone)]
pub struct CpuDiscreteFlock(ActiveLeafModel);

#[derive(Clone)]
pub struct CpuDiscreteRelation(ActiveLeafModel);

impl ActiveLeafModel {
    fn from_bytes(bytes: &[u8], kind: ActiveLeafKind) -> Result<Self> {
        let label = kind.label();
        let header = header(bytes)?;
        if header["schema"].as_str() != Some("uor-r4.lut-discrete/1") {
            return Err(invalid(format!(
                "{label} CPU read requires uor-r4.lut-discrete/1"
            )));
        }
        let shape = &header["shape"];
        if shape["read_mode"].as_str() != Some(kind.mode_name())
            || shape["read_initialization"].as_str() != Some("active_leaf_v1")
        {
            return Err(invalid(format!(
                "{label} CPU read refuses non-{label} initialization"
            )));
        }
        let vocab = usize::try_from(shape["vocabulary"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("vocabulary"))?;
        let blocks = usize::try_from(shape["blocks"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("blocks"))?;
        let context = usize::try_from(shape["context"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("context"))?;
        let stages = usize::try_from(shape["stages"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("stages"))?;
        let kept_cells = usize::try_from(shape["kept_cells"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("kept_cells"))?;
        let kept = usize::try_from(shape["read_kept"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("read_kept"))?;
        let sweeps = usize::try_from(shape["read_sweeps"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("read_sweeps"))?;
        let sweeps_valid = match kind {
            ActiveLeafKind::Flock => sweeps == 0,
            ActiveLeafKind::Relation => (1..=5).contains(&sweeps),
        };
        if !(2..=8192).contains(&vocab)
            || !(2..=16).contains(&blocks)
            || !(1..=243).contains(&context)
            || !(1..=16).contains(&kept_cells)
            || kept_cells > vocab
            || !(1..=8).contains(&kept)
            || !sweeps_valid
            || expected_stages(context) != Some(stages)
            || !matches!(
                shape["objective_policy"].as_str(),
                Some("nearest_address_margin_v1" | "occupied_cell_rank_hinge_v1")
            )
        {
            return Err(invalid(format!("{label} shape differs")));
        }
        if header["matrices"]
            .as_array()
            .is_none_or(|matrices| !matrices.is_empty())
        {
            return Err(invalid(format!(
                "{label} artifact requires an empty matrices array"
            )));
        }
        let source = &header["source"];
        if source["snapshot_schema"].as_str() != Some(FLOCK_SNAPSHOT_SCHEMA)
            || source["operator_kappa"].as_str() != Some(FLOCK_OPERATOR_KAPPA)
            || source["composition_kappa"].as_str() != Some(FLOCK_COMPOSITION_KAPPA)
        {
            return Err(invalid(format!("{label} artifact source geometry differs")));
        }
        validate_header_padding(bytes)?;
        let tables = header["tables"]
            .as_array()
            .ok_or_else(|| invalid(format!("{label} artifact tables differ")))?;
        if tables.len() != 4 {
            return Err(invalid(format!("{label} artifact requires four tables")));
        }
        let table = section_i32(bytes, &header, "discrete.table", vocab * blocks * 8, "i32")?;
        let cells = section_u32(bytes, &header, "discrete.cells", vocab, "u32")?;
        let routes = section_u32(bytes, &header, "discrete.routes", blocks * stages, "u32")?;
        let base_gate_count = blocks * stages;
        let gate_count = base_gate_count
            + match kind {
                ActiveLeafKind::Flock => 0,
                ActiveLeafKind::Relation => RELATION_EDGE_TYPES * stages,
            };
        let gates = section_i32(bytes, &header, "discrete.gates", gate_count, "i32")?;
        if routes.iter().any(|&action| action >= 64) {
            return Err(invalid(format!("{label} route action outside 0..64")));
        }
        if gates.iter().any(|&gate| !(-1..=1).contains(&gate)) {
            return Err(invalid(format!("{label} gate trit outside -1..=1")));
        }
        let token_blocks = decode_token_blocks(&table, &cells, vocab, blocks, label)?;
        validate_layout(bytes, &header)?;
        let level_gates = gates[..base_gate_count]
            .chunks_exact(stages)
            .map(<[i32]>::to_vec)
            .collect();
        let mode = match kind {
            ActiveLeafKind::Flock => ActiveLeafMode::Flock,
            ActiveLeafKind::Relation => {
                let relation_gates = (0..stages)
                    .map(|stage| {
                        let start = base_gate_count + stage * RELATION_EDGE_TYPES;
                        gates[start..start + RELATION_EDGE_TYPES]
                            .try_into()
                            .expect("the relation gate slice has five entries")
                    })
                    .collect();
                ActiveLeafMode::Relation {
                    sweeps,
                    relation_gates,
                }
            }
        };
        Self::from_parts(label, context, kept, token_blocks, level_gates, mode)
    }

    fn from_parts(
        label: &str,
        context: usize,
        kept: usize,
        token_blocks: Vec<Vec<[i32; 8]>>,
        level_gates: Vec<Vec<i32>>,
        mode: ActiveLeafMode,
    ) -> Result<Self> {
        let blocks = token_blocks.first().map_or(0, Vec::len);
        let stages = level_gates.first().map_or(0, Vec::len);
        let mode_valid = match &mode {
            ActiveLeafMode::Flock => true,
            ActiveLeafMode::Relation {
                sweeps,
                relation_gates,
            } => {
                (1..=5).contains(sweeps)
                    && relation_gates.len() == stages
                    && relation_gates
                        .iter()
                        .all(|row| row.iter().all(|gate| (-1..=1).contains(gate)))
            }
        };
        if !(2..=8192).contains(&token_blocks.len())
            || !(2..=16).contains(&blocks)
            || !(1..=8).contains(&kept)
            || expected_stages(context) != Some(stages)
            || level_gates.len() != blocks
            || level_gates
                .iter()
                .any(|row| row.len() != stages || row.iter().any(|gate| !(-1..=1).contains(gate)))
            || token_blocks.iter().any(|token| {
                token.len() != blocks || token.iter().any(|block| root_id(block).is_none())
            })
            || !mode_valid
        {
            return Err(invalid(format!("{label} parts differ")));
        }
        let composition =
            cl06_finite_composition_table().map_err(|error| invalid(error.to_string()))?;
        Ok(Self {
            context,
            blocks,
            stages,
            kept,
            token_blocks,
            level_gates,
            mode,
            composition,
        })
    }

    fn read(&self, prefix: &[u32]) -> Result<Vec<i32>> {
        let label = match self.mode {
            ActiveLeafMode::Flock => "Flock",
            ActiveLeafMode::Relation { .. } => "Relation",
        };
        if prefix.is_empty() || prefix.len() > self.context {
            return Err(invalid(format!("{label} prefix outside context")));
        }
        if prefix
            .iter()
            .any(|&token| token as usize >= self.token_blocks.len())
        {
            return Err(invalid(format!("{label} prefix token outside vocabulary")));
        }
        let plan = build_row_plan(prefix.len())?;
        let length = prefix.len();
        let last_token = prefix[length - 1] as usize;
        let mut out = Vec::with_capacity(self.blocks * 8);
        for block in 0..self.blocks {
            let frame_block = if block == 0 { self.blocks - 1 } else { 0 };
            let query_frame = &self.token_blocks[last_token][frame_block];
            let query_frame_root = root_id(query_frame)
                .ok_or_else(|| invalid("active-leaf query frame outside orbit"))?;
            let mut value = self.token_blocks[last_token][block];
            let mut terms = 1usize;

            let mut frames = vec![0usize; plan.nodes.len()];
            let mut payload = vec![[0i32; 8]; plan.nodes.len()];
            for (node_index, node) in plan.nodes.iter().enumerate() {
                if let Some(atom) = node.atom {
                    let token = prefix[atom] as usize;
                    frames[node_index] = root_id(&self.token_blocks[token][frame_block])
                        .ok_or_else(|| invalid("active-leaf atom frame outside orbit"))?;
                    payload[node_index] = self.token_blocks[token][block];
                } else {
                    let children = node
                        .children
                        .ok_or_else(|| invalid("active-leaf non-atom missing children"))?;
                    frames[node_index] = frames[children[0]];
                    let mut sum = [0i32; 8];
                    for &child in &children {
                        let action = usize::from(BFS[(frames[child] << 4) + frames[children[0]]]);
                        let carried = self.act(action, &payload[child])?;
                        for dimension in 0..8 {
                            sum[dimension] += carried[dimension];
                        }
                    }
                    payload[node_index] = orbit_root_coords(snap_root(&sum, 3)? as u8)
                        .map_err(|error| invalid(error.to_string()))?;
                }
            }

            let mut relation_scores = vec![0i32; plan.nodes.len()];
            if let ActiveLeafMode::Relation {
                sweeps,
                relation_gates,
            } = &self.mode
            {
                let mut previous = (0..plan.nodes.len())
                    .map(|node| {
                        8i32 - i32::try_from(root_cost(query_frame, frames[node]).0)
                            .expect("root distance fits i32")
                    })
                    .collect::<Vec<_>>();
                for _ in 0..*sweeps {
                    for node in 0..plan.nodes.len() {
                        if plan.nodes[node].last >= length {
                            continue;
                        }
                        let level = plan.nodes[node].level;
                        for (edge, &gate) in relation_gates[level].iter().enumerate() {
                            if gate == 0 {
                                continue;
                            }
                            let mut sum = 0i32;
                            match edge {
                                0 => {
                                    if let Some(children) = plan.nodes[node].children {
                                        for child in children {
                                            if plan.nodes[child].last < length {
                                                sum += previous[child];
                                            }
                                        }
                                    }
                                    for (parent, &previous_parent) in previous
                                        .iter()
                                        .enumerate()
                                        .take(plan.nodes.len())
                                        .skip(node + 1)
                                    {
                                        if plan.nodes[parent].last < length
                                            && plan.nodes[parent]
                                                .children
                                                .is_some_and(|children| children.contains(&node))
                                        {
                                            sum += previous_parent;
                                        }
                                    }
                                }
                                1 => {
                                    for parent in (node + 1)..plan.nodes.len() {
                                        let Some(children) = plan.nodes[parent].children else {
                                            continue;
                                        };
                                        if plan.nodes[parent].last >= length {
                                            continue;
                                        }
                                        if node == children[0]
                                            && plan.nodes[children[1]].last < length
                                        {
                                            sum += previous[children[1]];
                                        } else if node == children[1] {
                                            if plan.nodes[children[0]].last < length {
                                                sum += previous[children[0]];
                                            }
                                            if plan.nodes[children[2]].last < length {
                                                sum += previous[children[2]];
                                            }
                                        } else if node == children[2]
                                            && plan.nodes[children[1]].last < length
                                        {
                                            sum += previous[children[1]];
                                        }
                                    }
                                }
                                2 => {
                                    for (sibling, &previous_sibling) in
                                        previous.iter().enumerate().take(node)
                                    {
                                        if plan.nodes[sibling].last < length
                                            && plan.nodes[sibling].level == level
                                            && plan.nodes[sibling].last + 1
                                                == plan.nodes[node].first
                                        {
                                            sum += previous_sibling;
                                        }
                                    }
                                }
                                3 => {
                                    if level == 0 {
                                        let token = prefix[plan.nodes[node].last];
                                        for peer in 0..node {
                                            if plan.nodes[peer].last < length
                                                && plan.nodes[peer].level == 0
                                                && prefix[plan.nodes[peer].last] == token
                                            {
                                                sum = previous[peer];
                                            }
                                        }
                                    }
                                }
                                4 => {
                                    for peer in 0..node {
                                        if plan.nodes[peer].last < length
                                            && plan.nodes[peer].level == level
                                            && frames[peer] == frames[node]
                                        {
                                            sum = previous[peer];
                                        }
                                    }
                                }
                                _ => unreachable!("the edge loop has five entries"),
                            }
                            relation_scores[node] += if gate > 0 { sum } else { -sum };
                        }
                    }
                    previous.copy_from_slice(&relation_scores);
                }
            }

            for level in 0..self.stages {
                let gate = self.level_gates[block][level];
                if gate == 0 {
                    continue;
                }
                let mut top = [0usize; 8];
                let mut key_relation = [0i32; 8];
                let mut key_distance = [u32::MAX; 8];
                let mut key_hamming = [u32::MAX; 8];
                let mut found = 0usize;
                for (node, candidate) in plan.nodes.iter().enumerate() {
                    if candidate.level != level || candidate.last + 1 >= length {
                        continue;
                    }
                    let (distance, hamming) = root_cost(query_frame, frames[node]);
                    let relation = relation_scores[node];
                    let mut at = found;
                    while at > 0 {
                        let previous = top[at - 1];
                        let better = relation > key_relation[at - 1]
                            || (relation == key_relation[at - 1]
                                && (distance < key_distance[at - 1]
                                    || (distance == key_distance[at - 1]
                                        && (hamming < key_hamming[at - 1]
                                            || (hamming == key_hamming[at - 1]
                                                && (plan.nodes[node].last
                                                    > plan.nodes[previous].last
                                                    || (plan.nodes[node].last
                                                        == plan.nodes[previous].last
                                                        && node < previous)))))));
                        if !better {
                            break;
                        }
                        if at < self.kept {
                            top[at] = previous;
                            key_relation[at] = key_relation[at - 1];
                            key_distance[at] = key_distance[at - 1];
                            key_hamming[at] = key_hamming[at - 1];
                        }
                        at -= 1;
                    }
                    if at < self.kept {
                        top[at] = node;
                        key_relation[at] = relation;
                        key_distance[at] = distance;
                        key_hamming[at] = hamming;
                        found += usize::from(found < self.kept);
                    }
                }
                if found == 0 {
                    continue;
                }
                let mut sum = [0i32; 8];
                for &node in &top[..found] {
                    let action = usize::from(BFS[(frames[node] << 4) + query_frame_root]);
                    let carried = self.act(action, &payload[node])?;
                    for dimension in 0..8 {
                        sum[dimension] += carried[dimension];
                    }
                }
                let consensus = snap_root(&sum, found)?;
                let root = orbit_root_coords(consensus as u8)
                    .map_err(|error| invalid(error.to_string()))?;
                for dimension in 0..8 {
                    value[dimension] += if gate > 0 {
                        root[dimension]
                    } else {
                        -root[dimension]
                    };
                }
                terms += 1;
            }
            let output_root = snap_root(&value, terms)?;
            out.extend_from_slice(
                &orbit_root_coords(output_root as u8)
                    .map_err(|error| invalid(error.to_string()))?,
            );
        }
        Ok(out)
    }

    fn act(&self, action: usize, input: &[i32; 8]) -> Result<[i32; 8]> {
        let root: [i8; 8] = std::array::from_fn(|index| (input[index] / 2) as i8);
        let out = self
            .composition
            .state(action as u8)
            .ok_or_else(|| invalid("active-leaf action state absent"))?
            .apply_root_key(root)
            .map_err(|error| invalid(error.to_string()))?;
        Ok(out.map(|value| i32::from(value) * 2))
    }
}

impl CpuDiscreteFlock {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        ActiveLeafModel::from_bytes(bytes, ActiveLeafKind::Flock).map(Self)
    }

    pub fn from_parts(
        context: usize,
        kept: usize,
        token_blocks: Vec<Vec<[i32; 8]>>,
        level_gates: Vec<Vec<i32>>,
    ) -> Result<Self> {
        ActiveLeafModel::from_parts(
            "Flock",
            context,
            kept,
            token_blocks,
            level_gates,
            ActiveLeafMode::Flock,
        )
        .map(Self)
    }

    pub fn read(&self, prefix: &[u32]) -> Result<Vec<i32>> {
        self.0.read(prefix)
    }
}

impl CpuDiscreteRelation {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        ActiveLeafModel::from_bytes(bytes, ActiveLeafKind::Relation).map(Self)
    }

    pub fn from_parts(
        context: usize,
        kept: usize,
        sweeps: usize,
        token_blocks: Vec<Vec<[i32; 8]>>,
        level_gates: Vec<Vec<i32>>,
        relation_gates: Vec<[i32; RELATION_EDGE_TYPES]>,
    ) -> Result<Self> {
        ActiveLeafModel::from_parts(
            "Relation",
            context,
            kept,
            token_blocks,
            level_gates,
            ActiveLeafMode::Relation {
                sweeps,
                relation_gates,
            },
        )
        .map(Self)
    }

    pub fn read(&self, prefix: &[u32]) -> Result<Vec<i32>> {
        self.0.read(prefix)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuDiscreteProposalMode {
    Route,
    Flock,
    Relation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuDiscreteProposalKind {
    Route,
    Table,
    Gate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpuDiscreteProposal {
    pub lane: usize,
    pub current_loss: u64,
    pub selected_loss: u64,
    pub epoch: u64,
    pub sensitive: bool,
}

#[derive(Clone)]
pub struct CpuDiscreteProposalChooser {
    mode: CpuDiscreteProposalMode,
    blocks: usize,
    stages: usize,
    routes: Vec<u32>,
    token_blocks: Vec<Vec<[i32; 8]>>,
    gate_count: usize,
    move_actions: [u8; 16],
    composition: Cl06FiniteCompositionTable,
}

impl CpuDiscreteProposalChooser {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let header = header(bytes)?;
        let shape = &header["shape"];
        let blocks = usize::try_from(shape["blocks"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("proposal chooser blocks"))?;
        let stages = usize::try_from(shape["stages"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("proposal chooser stages"))?;
        let gate_count = blocks
            .checked_mul(stages)
            .ok_or_else(|| invalid("proposal chooser gate count overflows"))?;
        let (mode, routes, token_blocks, gate_count) = match shape["read_mode"].as_str() {
            Some("route") => {
                let route = CpuDiscreteRoute::from_bytes(bytes)?;
                (
                    CpuDiscreteProposalMode::Route,
                    route.routes.into_iter().flatten().collect(),
                    route.token_blocks,
                    gate_count,
                )
            }
            Some("flock") => {
                let flock = ActiveLeafModel::from_bytes(bytes, ActiveLeafKind::Flock)?;
                (
                    CpuDiscreteProposalMode::Flock,
                    Vec::new(),
                    flock.token_blocks,
                    gate_count,
                )
            }
            Some("relation") => {
                let relation = ActiveLeafModel::from_bytes(bytes, ActiveLeafKind::Relation)?;
                (
                    CpuDiscreteProposalMode::Relation,
                    Vec::new(),
                    relation.token_blocks,
                    gate_count + RELATION_EDGE_TYPES * stages,
                )
            }
            _ => return Err(invalid("proposal chooser read mode differs")),
        };
        let composition =
            cl06_finite_composition_table().map_err(|error| invalid(error.to_string()))?;
        let mut move_actions = [0u8; 16];
        for (candidate, action) in move_actions.iter_mut().enumerate() {
            *action = NormFrameTable::new(16, vec![candidate as u8; 16])
                .map_err(|error| invalid(error.to_string()))?
                .block_frame(0)
                .map_err(|error| invalid(error.to_string()))?;
        }
        Ok(Self {
            mode,
            blocks,
            stages,
            routes,
            token_blocks,
            gate_count,
            move_actions,
            composition,
        })
    }

    pub fn choose(
        &self,
        kind: CpuDiscreteProposalKind,
        index: usize,
        losses: &[u64; 16],
        epoch: u64,
    ) -> Result<CpuDiscreteProposal> {
        match kind {
            CpuDiscreteProposalKind::Gate if self.mode == CpuDiscreteProposalMode::Route => {
                return Err(invalid("Gate proposal requires Flock or Relation mode"));
            }
            CpuDiscreteProposalKind::Route if self.mode != CpuDiscreteProposalMode::Route => {
                return Err(invalid("Route proposal requires Route mode"));
            }
            _ => {}
        }
        match kind {
            CpuDiscreteProposalKind::Gate if index >= self.gate_count => {
                return Err(invalid("Gate proposal index outside gate inventory"));
            }
            CpuDiscreteProposalKind::Route if index >= self.blocks * self.stages => {
                return Err(invalid("Route proposal index outside map inventory"));
            }
            CpuDiscreteProposalKind::Table if index >= self.token_blocks.len() * self.blocks => {
                return Err(invalid("Table proposal index outside codebook inventory"));
            }
            _ => {}
        }

        let active = if kind == CpuDiscreteProposalKind::Gate {
            3
        } else {
            16
        };
        let sensitive = losses[1..active].iter().any(|&loss| loss != losses[0]);
        let mut best = 0usize;
        if kind == CpuDiscreteProposalKind::Gate {
            for lane in 1..3 {
                if losses[lane] < losses[best] {
                    best = lane;
                }
            }
        } else {
            for lane in 1..16 {
                let tie = best != 0
                    && losses[lane] == losses[best]
                    && self.resulting_port(kind, index, lane)?
                        < self.resulting_port(kind, index, best)?;
                if losses[lane] < losses[best] || tie {
                    best = lane;
                }
            }
        }
        Ok(CpuDiscreteProposal {
            lane: best,
            current_loss: losses[0],
            selected_loss: losses[best],
            epoch,
            sensitive,
        })
    }

    fn resulting_port(
        &self,
        kind: CpuDiscreteProposalKind,
        index: usize,
        candidate: usize,
    ) -> Result<u32> {
        let action = self.move_actions[candidate];
        match kind {
            CpuDiscreteProposalKind::Route => {
                let route = *self
                    .routes
                    .get(index)
                    .ok_or_else(|| invalid("Route proposal index absent"))?;
                self.composition
                    .compose_index(action, route as u8)
                    .map(u32::from)
                    .ok_or_else(|| invalid("Route proposal composition failed"))
            }
            CpuDiscreteProposalKind::Table => {
                let token = index / self.blocks;
                let block = index % self.blocks;
                let point = self
                    .token_blocks
                    .get(token)
                    .and_then(|token| token.get(block))
                    .ok_or_else(|| invalid("Table proposal index absent"))?;
                let root: [i8; 8] = std::array::from_fn(|dimension| (point[dimension] / 2) as i8);
                let moved = self
                    .composition
                    .state(action)
                    .ok_or_else(|| invalid("Table proposal action state absent"))?
                    .apply_root_key(root)
                    .map_err(|error| invalid(error.to_string()))?;
                let moved = moved.map(|value| i32::from(value) * 2);
                root_id(&moved)
                    .map(|root| root as u32)
                    .ok_or_else(|| invalid("Table proposal moved outside orbit"))
            }
            CpuDiscreteProposalKind::Gate => {
                Err(invalid("Gate proposals do not have resulting ports"))
            }
        }
    }
}

fn root_id(block: &[i32; 8]) -> Option<usize> {
    (0..16u8)
        .find(|&root| orbit_root_coords(root).ok().as_ref() == Some(block))
        .map(usize::from)
}

fn decode_token_blocks(
    table: &[i32],
    cells: &[u32],
    vocab: usize,
    blocks: usize,
    mode: &str,
) -> Result<Vec<Vec<[i32; 8]>>> {
    let mut token_blocks = Vec::with_capacity(vocab);
    for token in 0..vocab {
        let mut values = Vec::with_capacity(blocks);
        for block in 0..blocks {
            let start = (token * blocks + block) * 8;
            let coords: [i32; 8] = table[start..start + 8]
                .try_into()
                .map_err(|_| invalid(format!("{mode} codebook width")))?;
            if root_id(&coords).is_none() {
                return Err(invalid(format!("{mode} codebook block outside orbit")));
            }
            values.push(coords);
        }
        token_blocks.push(values);
    }
    for (token, (blocks, &cell)) in token_blocks.iter().zip(cells).enumerate() {
        let first = root_id(&blocks[0]).expect("decoded codebook block is an orbit root") as u32;
        let last = root_id(blocks.last().unwrap()).expect("decoded codebook block is an orbit root")
            as u32;
        if cell != first * 16 + last {
            return Err(invalid(format!("{mode} cell for token {token} differs")));
        }
    }
    Ok(token_blocks)
}

fn expected_stages(context: usize) -> Option<usize> {
    if !(1..=243).contains(&context) {
        return None;
    }
    let mut stages = 1usize;
    let mut span = 3usize;
    while span <= context {
        stages += 1;
        span = span.saturating_mul(3);
    }
    Some(stages)
}

fn root_cost(block: &[i32; 8], root: usize) -> (u32, u32) {
    let target = orbit_root_coords(root as u8).expect("root index is 0..16");
    let mut distance = 0u32;
    let mut hamming = 0u32;
    for (left, right) in block.iter().zip(target) {
        distance += left.abs_diff(right);
        hamming += u32::from(*left != right);
    }
    (distance, hamming)
}

fn snap_root(block: &[i32; 8], multiplicity: usize) -> Result<usize> {
    let mut best = 0usize;
    let mut best_distance = u32::MAX;
    let mut best_hamming = u32::MAX;
    for root in 0..16 {
        let target = orbit_root_coords(root as u8)
            .map_err(|error| invalid(format!("Flock snap root: {error}")))?;
        let mut distance = 0u32;
        let mut hamming = 0u32;
        for (value, target) in block.iter().zip(target) {
            let scaled = target
                .checked_mul(i32::try_from(multiplicity).unwrap_or(i32::MAX))
                .ok_or_else(|| invalid("Flock snap multiplicity overflows"))?;
            distance = distance.saturating_add(value.abs_diff(scaled));
            hamming += u32::from(*value != scaled);
        }
        if distance < best_distance || (distance == best_distance && hamming < best_hamming) {
            best = root;
            best_distance = distance;
            best_hamming = hamming;
        }
    }
    Ok(best)
}

fn build_row_plan(length: usize) -> Result<RowPlan> {
    if length == 0 || length > 243 {
        return Err(invalid("Route plan length outside context"));
    }
    let mut nodes = Vec::new();
    let mut frontier = Vec::new();
    for atom in 0..length {
        frontier.push(nodes.len());
        nodes.push(BinderNode {
            children: None,
            atom: Some(atom),
            level: 0,
            first: atom,
            last: atom,
        });
        loop {
            let n = frontier.len();
            if n < 3 {
                break;
            }
            let children = [frontier[n - 3], frontier[n - 2], frontier[n - 1]];
            let level = nodes[children[0]].level;
            if children.iter().any(|&child| nodes[child].level != level) {
                break;
            }
            let parent = nodes.len();
            let first = nodes[children[0]].first;
            let last = nodes[children[2]].last;
            nodes.push(BinderNode {
                children: Some(children),
                atom: None,
                level: level + 1,
                first,
                last,
            });
            frontier.truncate(n - 3);
            frontier.push(parent);
        }
    }
    Ok(RowPlan { nodes, frontier })
}

fn header(bytes: &[u8]) -> Result<serde_json::Value> {
    if bytes.len() < 16 || &bytes[..8] != b"UORLUT01" {
        return Err(invalid("Route artifact magic differs"));
    }
    let len = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let end = 16usize
        .checked_add(usize::try_from(len).map_err(|_| invalid("Route header length"))?)
        .ok_or_else(|| invalid("Route header end overflows"))?;
    serde_json::from_slice(
        bytes
            .get(16..end)
            .ok_or_else(|| invalid("Route header truncated"))?,
    )
    .map_err(|error| invalid(format!("Route header: {error}")))
}

fn section<'a>(
    bytes: &'a [u8],
    header: &serde_json::Value,
    name: &str,
    expected_len: usize,
    expected_kind: &str,
) -> Result<&'a [u8]> {
    let mut matches = header["tables"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|entry| entry["name"].as_str() == Some(name));
    let entry = matches
        .next()
        .ok_or_else(|| invalid(format!("missing {name}")))?;
    if matches.next().is_some() {
        return Err(invalid(format!("duplicate {name}")));
    }
    if entry["len"].as_u64() != Some(expected_len as u64) {
        return Err(invalid(format!("{name} length differs")));
    }
    if entry["kind"].as_str() != Some(expected_kind) {
        return Err(invalid(format!("{name} kind differs")));
    }
    let width = if expected_kind == "i16" { 2 } else { 4 };
    if entry["span"]["bytes"].as_u64() != Some((expected_len * width) as u64) {
        return Err(invalid(format!("{name} span byte length differs")));
    }
    let header_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let offset = usize::try_from(entry["span"]["offset"].as_u64().unwrap_or(u64::MAX))
        .map_err(|_| invalid("section offset"))?;
    if offset % 64 != 0 {
        return Err(invalid(format!("{name} is not 64-byte aligned")));
    }
    let header_end = 16usize
        .checked_add(
            usize::try_from(header_len)
                .map_err(|_| invalid("Route header length exceeds usize"))?,
        )
        .ok_or_else(|| invalid("Route header end overflows"))?;
    let data_start = header_end
        .checked_add(63)
        .ok_or_else(|| invalid("data start overflow"))?
        / 64
        * 64;
    let start = data_start
        .checked_add(offset)
        .ok_or_else(|| invalid("Route section start overflows"))?;
    let len = usize::try_from(entry["span"]["bytes"].as_u64().unwrap_or(0))
        .map_err(|_| invalid("section bytes"))?;
    let end = start
        .checked_add(len)
        .ok_or_else(|| invalid(format!("{name} end overflows")))?;
    bytes
        .get(start..end)
        .ok_or_else(|| invalid(format!("{name} outside artifact")))
}

fn validate_layout(bytes: &[u8], header: &serde_json::Value) -> Result<()> {
    let mut spans: Vec<(usize, usize, &str)> = header["tables"]
        .as_array()
        .ok_or_else(|| invalid("Route tables differ"))?
        .iter()
        .map(|entry| {
            let offset = usize::try_from(entry["span"]["offset"].as_u64().unwrap_or(u64::MAX))
                .map_err(|_| invalid("layout offset"))?;
            let len = usize::try_from(entry["span"]["bytes"].as_u64().unwrap_or(u64::MAX))
                .map_err(|_| invalid("layout bytes"))?;
            let end = offset
                .checked_add(len)
                .ok_or_else(|| invalid("layout end overflow"))?;
            Ok((offset, end, entry["name"].as_str().unwrap_or("")))
        })
        .collect::<Result<_>>()?;
    spans.sort_unstable_by_key(|span| span.0);
    let header_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let header_end = 16usize
        .checked_add(
            usize::try_from(header_len)
                .map_err(|_| invalid("Route header length exceeds usize"))?,
        )
        .ok_or_else(|| invalid("Route header end overflows"))?;
    let data_start = header_end
        .checked_add(63)
        .ok_or_else(|| invalid("data start overflow"))?
        / 64
        * 64;
    let data = bytes
        .get(data_start..)
        .ok_or_else(|| invalid("Route data start outside artifact"))?;
    let mut cursor = 0usize;
    for (start, end, name) in spans {
        if start < cursor || end > data.len() || data[cursor..start].iter().any(|byte| *byte != 0) {
            return Err(invalid(format!(
                "{name} layout overlaps or has nonzero padding"
            )));
        }
        cursor = end;
    }
    if data[cursor..].iter().any(|byte| *byte != 0) {
        return Err(invalid("Route artifact has nonzero trailing padding"));
    }
    Ok(())
}

fn validate_header_padding(bytes: &[u8]) -> Result<()> {
    let header_len = u64::from_le_bytes(
        bytes[8..16]
            .try_into()
            .map_err(|_| invalid("Flock header length differs"))?,
    );
    let unaligned = 16usize
        .checked_add(
            usize::try_from(header_len)
                .map_err(|_| invalid("Flock header length exceeds usize"))?,
        )
        .ok_or_else(|| invalid("Flock header end overflows"))?;
    let data_start = unaligned
        .checked_add(63)
        .ok_or_else(|| invalid("Flock data start overflow"))?
        / 64
        * 64;
    let padding = bytes
        .get(unaligned..data_start)
        .ok_or_else(|| invalid("Flock header padding is truncated"))?;
    if padding.iter().any(|byte| *byte != 0) {
        return Err(invalid("Flock header padding is nonzero"));
    }
    Ok(())
}

fn section_i32(
    bytes: &[u8],
    header: &serde_json::Value,
    name: &str,
    len: usize,
    kind: &str,
) -> Result<Vec<i32>> {
    Ok(section(bytes, header, name, len, kind)?
        .chunks_exact(4)
        .map(|v| i32::from_le_bytes(v.try_into().unwrap()))
        .collect())
}

fn section_u32(
    bytes: &[u8],
    header: &serde_json::Value,
    name: &str,
    len: usize,
    kind: &str,
) -> Result<Vec<u32>> {
    Ok(section(bytes, header, name, len, kind)?
        .chunks_exact(4)
        .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

    fn identity_route() -> CpuDiscreteRoute {
        let root = orbit_root_coords(0).unwrap();
        CpuDiscreteRoute {
            context: 3,
            blocks: 2,
            stages: 2,
            token_blocks: vec![vec![root; 2]; 3],
            routes: vec![vec![0; 2]; 2],
            composition: cl06_finite_composition_table().unwrap(),
        }
    }

    #[test]
    fn identity_route_is_deterministic_across_context_lengths() {
        let route = identity_route();
        let expected = orbit_root_coords(0).unwrap();
        let one = route.read(&[0]).unwrap();
        let three = route.read(&[0, 1, 2]).unwrap();
        assert_eq!(
            one,
            expected.into_iter().chain(expected).collect::<Vec<_>>()
        );
        assert_eq!(one, three);
        assert_eq!(
            route.read(&[]).unwrap_err().to_string(),
            "Route prefix outside context"
        );
        assert!(route.read(&[3]).is_err());
    }

    #[test]
    fn malformed_artifacts_are_refused() {
        assert!(CpuDiscreteRoute::from_bytes(b"bad").is_err());
        assert!(root_id(&[1; 8]).is_none());
    }

    #[test]
    fn serialized_nonzero_route_matches_integer_discrete_model() {
        let artifact = serialized_route();
        let cpu = CpuDiscreteRoute::from_bytes(&artifact).unwrap();
        let oracle = IntegerDiscreteModel::from_bytes(&artifact).unwrap();
        for prefix in [&[0u32][..], &[0, 1][..], &[0, 1, 2][..], &[1, 2, 0][..]] {
            assert_eq!(
                cpu.read(prefix).unwrap(),
                oracle.evaluate_prefix(prefix).unwrap().read_vector
            );
        }
    }

    #[test]
    fn serialized_flock_matches_integer_discrete_model() {
        let artifact = serialized_flock_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[0, 0, 1, 0],
            2,
        );
        let cpu = CpuDiscreteFlock::from_bytes(&artifact).unwrap();
        let oracle = IntegerDiscreteModel::from_bytes(&artifact).unwrap();
        for prefix in [
            &[0u32][..],
            &[0, 1][..],
            &[0, 1, 2, 3, 4, 1][..],
            &[4, 3, 2, 1, 0][..],
        ] {
            assert_eq!(
                cpu.read(prefix).unwrap(),
                oracle.evaluate_prefix(prefix).unwrap().read_vector
            );
        }
    }

    #[test]
    fn flock_zero_gate_fallback_and_gate_trits_are_distinct() {
        let roots = [[1, 0], [0, 0], [0, 0]];
        let mut zero_fixture = DiscreteFixture::flock();
        zero_fixture.roots = roots;
        let mut positive_fixture = DiscreteFixture::flock();
        positive_fixture.roots = roots;
        positive_fixture.gates = vec![1, 0, 1, 0];
        let mut negative_fixture = DiscreteFixture::flock();
        negative_fixture.roots = roots;
        negative_fixture.gates = vec![-1, 0, -1, 0];
        let zero_artifact = serialized_discrete(zero_fixture);
        let positive_artifact = serialized_discrete(positive_fixture);
        let negative_artifact = serialized_discrete(negative_fixture);
        let zero = CpuDiscreteFlock::from_bytes(&zero_artifact).unwrap();
        let positive = CpuDiscreteFlock::from_bytes(&positive_artifact).unwrap();
        let negative = CpuDiscreteFlock::from_bytes(&negative_artifact).unwrap();
        let zero_oracle = IntegerDiscreteModel::from_bytes(&zero_artifact).unwrap();
        let positive_oracle = IntegerDiscreteModel::from_bytes(&positive_artifact).unwrap();
        let negative_oracle = IntegerDiscreteModel::from_bytes(&negative_artifact).unwrap();
        let mut changed = false;
        for prefix in [&[0u32, 1][..], &[0, 1, 2][..], &[2, 1, 0][..]] {
            let zero_read = zero.read(prefix).unwrap();
            let positive_read = positive.read(prefix).unwrap();
            let negative_read = negative.read(prefix).unwrap();
            assert_eq!(
                zero_read,
                zero_oracle.evaluate_prefix(prefix).unwrap().read_vector
            );
            assert_eq!(
                positive_read,
                positive_oracle.evaluate_prefix(prefix).unwrap().read_vector
            );
            assert_eq!(
                negative_read,
                negative_oracle.evaluate_prefix(prefix).unwrap().read_vector
            );
            let last = prefix[prefix.len() - 1] as usize;
            assert_eq!(
                zero_read,
                roots[last]
                    .iter()
                    .flat_map(|&root| orbit_root_coords(root).unwrap())
                    .collect::<Vec<_>>()
            );
            changed |= positive_read != zero_read;
        }
        assert!(changed);
    }

    #[test]
    fn serialized_multilevel_flock_matches_integer_discrete_model() {
        let roots = [
            [0, 1],
            [2, 3],
            [4, 5],
            [6, 7],
            [8, 9],
            [10, 11],
            [12, 13],
            [14, 15],
        ];
        let artifact = serialized_flock_fixture(9, &roots, &[1, 1, -1, 1, -1, 1], 3);
        let cpu = CpuDiscreteFlock::from_bytes(&artifact).unwrap();
        let oracle = IntegerDiscreteModel::from_bytes(&artifact).unwrap();
        for prefix in [
            &[0u32][..],
            &[0, 1][..],
            &[0, 1, 2, 3, 4][..],
            &[0, 1, 2, 3, 4, 5, 6, 7, 0][..],
            &[7, 6, 5, 4, 3, 2, 1, 0, 5][..],
        ] {
            assert_eq!(
                cpu.read(prefix).unwrap(),
                oracle.evaluate_prefix(prefix).unwrap().read_vector
            );
        }
        let active = cpu.read(&[0, 1, 2, 3, 4, 5, 6, 7, 0]).unwrap();
        let zero = CpuDiscreteFlock::from_bytes(&serialized_flock_fixture(9, &roots, &[0; 6], 3))
            .unwrap()
            .read(&[0, 1, 2, 3, 4, 5, 6, 7, 0])
            .unwrap();
        assert_ne!(active, zero);
    }

    #[test]
    fn flock_plan_retains_spans_across_an_unequal_frontier() {
        let plan = build_row_plan(5).unwrap();
        assert_eq!(
            plan.frontier
                .iter()
                .map(|&node| plan.nodes[node].level)
                .collect::<Vec<_>>(),
            [1, 0, 0]
        );
        let parent = plan.frontier[0];
        assert_eq!((plan.nodes[parent].first, plan.nodes[parent].last), (0, 2));
    }

    #[test]
    #[ignore = "release benchmark"]
    fn release_flock_read_benchmark() {
        use std::time::Instant;

        const VOCAB: usize = 4_096;
        const BLOCKS: usize = 4;
        const CONTEXT: usize = 192;
        const PREFIX: usize = 64;
        const KEPT: usize = 3;
        const ROWS: usize = 1_024;

        let token_blocks = (0..VOCAB)
            .map(|token| {
                (0..BLOCKS)
                    .map(|block| orbit_root_coords(((token + block * 7) & 15) as u8).unwrap())
                    .collect()
            })
            .collect();
        let stages = expected_stages(CONTEXT).unwrap();
        let gates = (0..BLOCKS)
            .map(|block| {
                (0..stages)
                    .map(|level| ((block + level) % 3) as i32 - 1)
                    .collect()
            })
            .collect();
        let flock = CpuDiscreteFlock::from_parts(CONTEXT, KEPT, token_blocks, gates).unwrap();
        let prefix = (0..PREFIX as u32)
            .map(|index| (index * 997 + 17) % VOCAB as u32)
            .collect::<Vec<_>>();
        for _ in 0..4 {
            flock.read(&prefix).unwrap();
        }
        let started = Instant::now();
        let mut checksum = 0u64;
        for _ in 0..ROWS {
            for value in flock.read(&prefix).unwrap() {
                checksum = checksum.rotate_left(7).wrapping_add(value as i64 as u64);
            }
        }
        let seconds = started.elapsed().as_secs_f64();
        println!(
            "benchmark=flock vocab={VOCAB} blocks={BLOCKS} stages={stages} kept={KEPT} prefix={PREFIX} rows={ROWS} elapsed_ms={:.3} rows_per_second={:.3} checksum={checksum:016x}",
            seconds * 1_000.0,
            ROWS as f64 / seconds
        );
    }

    #[test]
    fn malformed_flock_artifacts_are_refused() {
        let mut route = DiscreteFixture::flock();
        route.mode = "route";
        route.initialization = "zero_gates_v1";
        route.kept = 0;
        route.gates.fill(0);
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(route)).is_err());

        let mut relation = DiscreteFixture::flock();
        relation.mode = "relation";
        relation.sweeps = 1;
        relation.gates = vec![0; 14];
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(relation)).is_err());

        for kept in [0, 9] {
            let mut fixture = DiscreteFixture::flock();
            fixture.kept = kept;
            assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(fixture)).is_err());
        }

        let mut sweeps = DiscreteFixture::flock();
        sweeps.sweeps = 1;
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(sweeps)).is_err());

        let mut initialization = DiscreteFixture::flock();
        initialization.initialization = "zero_gates_v1";
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(initialization)).is_err());

        let mut bad_gate_count = DiscreteFixture::flock();
        bad_gate_count.gates.pop();
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(bad_gate_count)).is_err());

        let mut bad_gate = DiscreteFixture::flock();
        bad_gate.gates[0] = 2;
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(bad_gate)).is_err());

        let mut bad_map = DiscreteFixture::flock();
        bad_map.routes[0] = 64;
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(bad_map)).is_err());

        let mut non_orbit = DiscreteFixture::flock();
        non_orbit.table = Some(vec![1; 3 * 2 * 8]);
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(non_orbit)).is_err());

        let mut bad_cell = DiscreteFixture::flock();
        bad_cell.cells = Some(vec![0, 0xff, 0]);
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(bad_cell)).is_err());

        let mut short_span = DiscreteFixture::flock();
        short_span.short_span = true;
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(short_span)).is_err());

        let artifact = serialized_discrete(DiscreteFixture::flock());
        let wrong_kind = rewrite_header(artifact.clone(), |header| {
            header["tables"][0]["kind"] = json!("u32");
        });
        assert!(CpuDiscreteFlock::from_bytes(&wrong_kind).is_err());

        let misaligned = rewrite_header(artifact.clone(), |header| {
            header["tables"][0]["span"]["offset"] = json!(1);
        });
        assert!(CpuDiscreteFlock::from_bytes(&misaligned).is_err());

        let duplicate = rewrite_header(artifact.clone(), |header| {
            header["tables"][1]["name"] = json!("discrete.table");
        });
        assert!(CpuDiscreteFlock::from_bytes(&duplicate).is_err());

        let nonempty_matrices = rewrite_header(artifact.clone(), |header| {
            header["matrices"] = json!([{"name": "unexpected"}]);
        });
        assert!(CpuDiscreteFlock::from_bytes(&nonempty_matrices).is_err());

        let wrong_snapshot = rewrite_header(artifact.clone(), |header| {
            header["source"]["snapshot_schema"] = json!("uor-r4.wrong/1");
        });
        assert!(CpuDiscreteFlock::from_bytes(&wrong_snapshot).is_err());

        let wrong_operator = rewrite_header(artifact.clone(), |header| {
            header["source"]["operator_kappa"] = json!("blake3:wrong");
        });
        assert!(CpuDiscreteFlock::from_bytes(&wrong_operator).is_err());

        let wrong_composition = rewrite_header(artifact.clone(), |header| {
            header["source"]["composition_kappa"] = json!("blake3:wrong");
        });
        assert!(CpuDiscreteFlock::from_bytes(&wrong_composition).is_err());

        let unknown_objective = rewrite_header(artifact.clone(), |header| {
            header["shape"]["objective_policy"] = json!("unknown");
        });
        assert!(CpuDiscreteFlock::from_bytes(&unknown_objective).is_err());

        let missing_objective = rewrite_header(artifact.clone(), |header| {
            header["shape"]
                .as_object_mut()
                .unwrap()
                .remove("objective_policy");
        });
        assert!(CpuDiscreteFlock::from_bytes(&missing_objective).is_err());

        let mut nonzero_header_padding = artifact.clone();
        let header_len =
            u64::from_le_bytes(nonzero_header_padding[8..16].try_into().unwrap()) as usize;
        let unaligned = 16 + header_len;
        let data_start = unaligned.div_ceil(64) * 64;
        assert!(unaligned < data_start);
        nonzero_header_padding[unaligned] = 1;
        assert!(CpuDiscreteFlock::from_bytes(&nonzero_header_padding).is_err());

        let mut nonzero_padding = artifact;
        nonzero_padding.push(1);
        assert!(CpuDiscreteFlock::from_bytes(&nonzero_padding).is_err());

        let reader =
            CpuDiscreteFlock::from_bytes(&serialized_discrete(DiscreteFixture::flock())).unwrap();
        assert!(reader.read(&[]).is_err());
        assert!(reader.read(&[3]).is_err());
        assert!(reader.read(&[0, 1, 2, 0]).is_err());
    }

    #[test]
    fn serialized_relation_matches_host_across_sweeps_and_levels() {
        let roots = [
            [0, 1],
            [2, 3],
            [4, 5],
            [6, 7],
            [8, 9],
            [10, 11],
            [12, 13],
            [14, 15],
        ];
        let level_gates = [1, 1, -1, 1, -1, 1];
        let relation_gates = [[1, -1, 1, -1, 1], [-1, 1, -1, 1, -1], [1, 1, -1, -1, 1]];
        for sweeps in [1, 2, 5] {
            let artifact =
                serialized_relation_fixture(9, &roots, &level_gates, &relation_gates, 3, sweeps);
            let cpu = CpuDiscreteRelation::from_bytes(&artifact).unwrap();
            let oracle = IntegerDiscreteModel::from_bytes(&artifact).unwrap();
            for prefix in [
                &[0u32][..],
                &[0, 1][..],
                &[0, 1, 2, 3, 4][..],
                &[0, 1, 2, 3, 4, 5, 6, 7, 0][..],
                &[7, 6, 5, 4, 3, 2, 1, 0, 5][..],
            ] {
                assert_eq!(
                    cpu.read(prefix).unwrap(),
                    oracle.evaluate_prefix(prefix).unwrap().read_vector,
                    "sweeps={sweeps} prefix={prefix:?}"
                );
            }
        }
    }

    #[test]
    fn each_relation_edge_kind_matches_host() {
        let roots = [
            [0, 1],
            [2, 3],
            [4, 5],
            [6, 7],
            [8, 9],
            [10, 11],
            [12, 13],
            [14, 15],
        ];
        let level_gates = [1, 1, -1, 1, -1, 1];
        let prefix = [0u32, 1, 2, 3, 4, 5, 6, 7, 0];
        let zero_artifact = serialized_relation_fixture(
            9,
            &roots,
            &level_gates,
            &[[0; RELATION_EDGE_TYPES]; 3],
            2,
            2,
        );
        let zero_read = CpuDiscreteRelation::from_bytes(&zero_artifact)
            .unwrap()
            .read(&prefix)
            .unwrap();
        let mut changed = false;
        for edge in 0..RELATION_EDGE_TYPES {
            let mut relation_gates = [[0i32; RELATION_EDGE_TYPES]; 3];
            relation_gates[0][edge] = 1;
            let artifact =
                serialized_relation_fixture(9, &roots, &level_gates, &relation_gates, 2, 2);
            let cpu = CpuDiscreteRelation::from_bytes(&artifact).unwrap();
            let oracle = IntegerDiscreteModel::from_bytes(&artifact).unwrap();
            let read = cpu.read(&prefix).unwrap();
            assert_eq!(
                read,
                oracle.evaluate_prefix(&prefix).unwrap().read_vector,
                "edge={edge}"
            );
            changed |= read != zero_read;
        }
        assert!(changed);
    }

    #[test]
    fn zero_relation_gates_equal_the_corresponding_flock_read() {
        let roots = [
            [0, 1],
            [2, 3],
            [4, 5],
            [6, 7],
            [8, 9],
            [10, 11],
            [12, 13],
            [14, 15],
        ];
        let level_gates = [1, 1, -1, 1, -1, 1];
        let relation = serialized_relation_fixture(
            9,
            &roots,
            &level_gates,
            &[[0; RELATION_EDGE_TYPES]; 3],
            3,
            2,
        );
        let flock = serialized_flock_fixture(9, &roots, &level_gates, 3);
        let relation = CpuDiscreteRelation::from_bytes(&relation).unwrap();
        let flock = CpuDiscreteFlock::from_bytes(&flock).unwrap();
        for prefix in [
            &[0u32, 1][..],
            &[0, 1, 2, 3, 4][..],
            &[0, 1, 2, 3, 4, 5, 6, 7, 0][..],
        ] {
            assert_eq!(relation.read(prefix).unwrap(), flock.read(prefix).unwrap());
        }
    }

    #[test]
    #[ignore = "release benchmark"]
    fn release_relation_read_benchmark() {
        use std::time::Instant;

        const VOCAB: usize = 4_096;
        const BLOCKS: usize = 4;
        const CONTEXT: usize = 192;
        const PREFIX: usize = 64;
        const KEPT: usize = 3;
        const SWEEPS: usize = 2;
        const ROWS: usize = 1_024;

        let token_blocks = (0..VOCAB)
            .map(|token| {
                (0..BLOCKS)
                    .map(|block| orbit_root_coords(((token + block * 7) & 15) as u8).unwrap())
                    .collect()
            })
            .collect();
        let stages = expected_stages(CONTEXT).unwrap();
        let level_gates = (0..BLOCKS)
            .map(|block| {
                (0..stages)
                    .map(|level| ((block + level) % 3) as i32 - 1)
                    .collect()
            })
            .collect();
        let relation_gates = (0..stages)
            .map(|level| std::array::from_fn(|edge| ((level + edge) % 3) as i32 - 1))
            .collect();
        let relation = CpuDiscreteRelation::from_parts(
            CONTEXT,
            KEPT,
            SWEEPS,
            token_blocks,
            level_gates,
            relation_gates,
        )
        .unwrap();
        let prefix = (0..PREFIX as u32)
            .map(|index| (index * 997 + 17) % VOCAB as u32)
            .collect::<Vec<_>>();
        for _ in 0..4 {
            relation.read(&prefix).unwrap();
        }
        let started = Instant::now();
        let mut checksum = 0u64;
        for _ in 0..ROWS {
            for value in relation.read(&prefix).unwrap() {
                checksum = checksum.rotate_left(7).wrapping_add(value as i64 as u64);
            }
        }
        let seconds = started.elapsed().as_secs_f64();
        println!(
            "benchmark=relation vocab={VOCAB} blocks={BLOCKS} stages={stages} kept={KEPT} sweeps={SWEEPS} prefix={PREFIX} rows={ROWS} elapsed_ms={:.3} rows_per_second={:.3} checksum={checksum:016x}",
            seconds * 1_000.0,
            ROWS as f64 / seconds
        );
    }

    #[test]
    fn non_relation_and_malformed_relation_artifacts_are_refused() {
        let roots = [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]];
        let level_gates = [0, 1, 0, 1];
        let relation_gates = [[0; RELATION_EDGE_TYPES]; 2];
        let artifact = serialized_relation_fixture(6, &roots, &level_gates, &relation_gates, 2, 2);

        for mode in ["flock", "route"] {
            let wrong_mode = rewrite_header(artifact.clone(), |header| {
                header["shape"]["read_mode"] = json!(mode);
            });
            assert!(CpuDiscreteRelation::from_bytes(&wrong_mode).is_err());
        }
        for sweeps in [0, 6] {
            let wrong_sweeps = rewrite_header(artifact.clone(), |header| {
                header["shape"]["read_sweeps"] = json!(sweeps);
            });
            assert!(CpuDiscreteRelation::from_bytes(&wrong_sweeps).is_err());
        }
        for kept in [0, 9] {
            let wrong_kept = rewrite_header(artifact.clone(), |header| {
                header["shape"]["read_kept"] = json!(kept);
            });
            assert!(CpuDiscreteRelation::from_bytes(&wrong_kept).is_err());
        }
        let wrong_initialization = rewrite_header(artifact.clone(), |header| {
            header["shape"]["read_initialization"] = json!("zero_gates_v1");
        });
        assert!(CpuDiscreteRelation::from_bytes(&wrong_initialization).is_err());

        let mut bad_level_gate = level_gates;
        bad_level_gate[0] = 2;
        assert!(
            CpuDiscreteRelation::from_bytes(&serialized_relation_fixture(
                6,
                &roots,
                &bad_level_gate,
                &relation_gates,
                2,
                2
            ))
            .is_err()
        );

        let mut bad_relation_gates = relation_gates;
        bad_relation_gates[0][0] = 2;
        assert!(
            CpuDiscreteRelation::from_bytes(&serialized_relation_fixture(
                6,
                &roots,
                &level_gates,
                &bad_relation_gates,
                2,
                2
            ))
            .is_err()
        );

        let short_gate_table = rewrite_header(artifact.clone(), |header| {
            header["tables"][3]["len"] = json!(header["tables"][3]["len"].as_u64().unwrap() - 1);
            header["tables"][3]["span"]["bytes"] =
                json!(header["tables"][3]["span"]["bytes"].as_u64().unwrap() - 4);
        });
        assert!(CpuDiscreteRelation::from_bytes(&short_gate_table).is_err());

        let mut bad_map = artifact.clone();
        set_table_u32(&mut bad_map, "discrete.routes", 0, 64);
        assert!(CpuDiscreteRelation::from_bytes(&bad_map).is_err());

        let relation = CpuDiscreteRelation::from_bytes(&serialized_relation_fixture(
            6,
            &roots,
            &level_gates,
            &relation_gates,
            2,
            2,
        ))
        .unwrap();
        assert!(relation.read(&[]).is_err());
        assert!(relation.read(&[5]).is_err());
        assert!(relation.read(&[0, 1, 2, 3, 4, 0, 1]).is_err());

        assert!(CpuDiscreteRelation::from_bytes(&serialized_flock_fixture(
            6,
            &roots,
            &level_gates,
            2
        ))
        .is_err());
    }

    #[test]
    fn gate_chooser_uses_strict_improvement_and_ignores_inactive_lanes() {
        let artifact = serialized_flock_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[0, 1, 0, 1],
            2,
        );
        let chooser = CpuDiscreteProposalChooser::from_bytes(&artifact).unwrap();
        let mut improved = [12u64; 16];
        improved[1] = 10;
        improved[2] = 9;
        let selected = chooser
            .choose(CpuDiscreteProposalKind::Gate, 0, &improved, 7)
            .unwrap();
        assert_eq!(selected.lane, 2);
        assert!(selected.sensitive);

        let mut tied = [11u64; 16];
        tied[3] = 0;
        let selected = chooser
            .choose(CpuDiscreteProposalKind::Gate, 0, &tied, 7)
            .unwrap();
        assert_eq!(selected.lane, 0);
        assert!(!selected.sensitive);

        let mut all_worse = [10u64; 16];
        all_worse[1] = 11;
        all_worse[2] = 12;
        let selected = chooser
            .choose(CpuDiscreteProposalKind::Gate, 0, &all_worse, 8)
            .unwrap();
        assert_eq!(selected.lane, 0);
        assert!(selected.sensitive);
        assert_eq!(selected.epoch, 8);
    }

    #[test]
    fn route_and_table_ties_choose_the_lower_resulting_port() {
        const ROUTE_PORTS: [u32; 15] = [16, 27, 34, 41, 47, 2, 3, 4, 5, 21, 22, 23, 24, 25, 26];
        const TABLE_PORTS: [u32; 15] = [0, 10, 2, 5, 6, 5, 13, 7, 9, 11, 0, 2, 8, 7, 4];
        let route_artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let route = CpuDiscreteProposalChooser::from_bytes(&route_artifact).unwrap();
        let observed_route_ports = std::array::from_fn::<_, 15, _>(|lane| {
            route
                .resulting_port(CpuDiscreteProposalKind::Route, 0, lane + 1)
                .unwrap()
        });
        assert_eq!(observed_route_ports, ROUTE_PORTS);
        let mut losses = [1u64; 16];
        losses[0] = 2;
        let route_expected = 1 + ROUTE_PORTS
            .iter()
            .enumerate()
            .min_by_key(|&(lane, &port)| (port, lane))
            .unwrap()
            .0;
        assert_eq!(
            route
                .choose(CpuDiscreteProposalKind::Route, 0, &losses, 3)
                .unwrap()
                .lane,
            route_expected
        );
        let tied_to_current = [1u64; 16];
        assert_eq!(
            route
                .choose(CpuDiscreteProposalKind::Route, 0, &tied_to_current, 3)
                .unwrap()
                .lane,
            0
        );
        let mut lower_loss = [1u64; 16];
        lower_loss[7] = 0;
        assert_eq!(
            route
                .choose(CpuDiscreteProposalKind::Route, 0, &lower_loss, 3)
                .unwrap()
                .lane,
            7
        );

        let table_artifact = serialized_flock_fixture(
            6,
            &[[3, 2], [1, 0], [7, 4], [9, 8], [5, 6]],
            &[1, 0, 1, 0],
            2,
        );
        let table = CpuDiscreteProposalChooser::from_bytes(&table_artifact).unwrap();
        let observed_table_ports = std::array::from_fn::<_, 15, _>(|lane| {
            table
                .resulting_port(CpuDiscreteProposalKind::Table, 0, lane + 1)
                .unwrap()
        });
        assert_eq!(observed_table_ports, TABLE_PORTS);
        let table_expected = 1 + TABLE_PORTS
            .iter()
            .enumerate()
            .min_by_key(|&(lane, &port)| (port, lane))
            .unwrap()
            .0;
        assert_eq!(
            table
                .choose(CpuDiscreteProposalKind::Table, 0, &losses, 3)
                .unwrap()
                .lane,
            table_expected
        );
        assert_eq!(
            table
                .choose(CpuDiscreteProposalKind::Table, 0, &tied_to_current, 3)
                .unwrap()
                .lane,
            0
        );
        assert_eq!(
            table
                .choose(CpuDiscreteProposalKind::Table, 0, &lower_loss, 3)
                .unwrap()
                .lane,
            7
        );
    }

    #[test]
    fn proposal_chooser_refuses_invalid_modes_and_inventory() {
        let route_artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let route = CpuDiscreteProposalChooser::from_bytes(&route_artifact).unwrap();
        let losses = [0u64; 16];
        assert!(route
            .choose(CpuDiscreteProposalKind::Gate, 0, &losses, 0)
            .is_err());
        assert!(route
            .choose(CpuDiscreteProposalKind::Route, 4, &losses, 0)
            .is_err());
        assert!(route
            .choose(CpuDiscreteProposalKind::Table, 6, &losses, 0)
            .is_err());

        let relation_artifact = serialized_relation_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[0, 1, 0, 1],
            &[[0; RELATION_EDGE_TYPES]; 2],
            2,
            1,
        );
        let relation = CpuDiscreteProposalChooser::from_bytes(&relation_artifact).unwrap();
        assert!(relation
            .choose(CpuDiscreteProposalKind::Route, 0, &losses, 0)
            .is_err());
        assert!(relation
            .choose(CpuDiscreteProposalKind::Gate, 14, &losses, 0)
            .is_err());
        assert!(CpuDiscreteProposalChooser::from_bytes(b"bad").is_err());
    }

    #[test]
    #[ignore = "release benchmark"]
    fn release_proposal_chooser_benchmark() {
        use std::time::Instant;

        const ROWS: usize = 1_000_000;
        let artifact = serialized_relation_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[0, 1, 0, 1],
            &[[1, -1, 1, -1, 1], [-1, 1, -1, 1, -1]],
            2,
            2,
        );
        let chooser = CpuDiscreteProposalChooser::from_bytes(&artifact).unwrap();
        let losses = [12, 11, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        for _ in 0..4 {
            chooser
                .choose(CpuDiscreteProposalKind::Gate, 0, &losses, 1)
                .unwrap();
        }
        let started = Instant::now();
        let mut checksum = 0u64;
        for row in 0..ROWS {
            let selected = chooser
                .choose(CpuDiscreteProposalKind::Gate, 0, &losses, row as u64)
                .unwrap();
            checksum = checksum
                .rotate_left(7)
                .wrapping_add(selected.lane as u64)
                .wrapping_add(selected.current_loss)
                .wrapping_add(selected.selected_loss)
                .wrapping_add(selected.epoch);
        }
        let seconds = started.elapsed().as_secs_f64();
        println!(
            "benchmark=proposal-chooser rows={ROWS} elapsed_ms={:.3} decisions_per_second={:.3} checksum={checksum:016x}",
            seconds * 1_000.0,
            ROWS as f64 / seconds
        );
    }

    #[test]
    fn non_route_and_malformed_route_artifacts_are_refused() {
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "relation",
            vec![1, 2, 3, 4],
            2,
            1,
            2,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "flock",
            vec![1, 2, 3, 4],
            2,
            0,
            2,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![64, 2, 3, 4],
            0,
            0,
            2,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![1, 2, 3],
            0,
            0,
            2,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![1, 2, 3, 4],
            0,
            0,
            2,
            Some(vec![1; 3 * 2 * 8]),
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![1, 2, 3, 4],
            1,
            0,
            2,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![1, 2, 3, 4],
            0,
            1,
            2,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![1, 2, 3, 4],
            0,
            0,
            0,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![1, 2, 3, 4],
            0,
            0,
            2,
            None,
            true
        ))
        .is_err());
    }

    fn serialized_route() -> Vec<u8> {
        serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false)
    }

    #[allow(clippy::too_many_arguments)]
    fn serialized_route_with(
        mode: &str,
        routes: Vec<u32>,
        read_kept: u32,
        read_sweeps: u32,
        kept_cells: u32,
        table_override: Option<Vec<i32>>,
        short_span: bool,
    ) -> Vec<u8> {
        let roots = [[0u8, 1], [2, 3], [4, 5]];
        let mut table = Vec::new();
        let mut cells = Vec::new();
        for blocks in roots {
            for root in blocks {
                table.extend_from_slice(&orbit_root_coords(root).unwrap());
            }
            cells.push(u32::from(blocks[0] << 4 | blocks[1]));
        }
        if let Some(table_override) = table_override {
            table = table_override;
        }
        let gates = vec![0i32; 4];
        let mut blob = Vec::new();
        let table_span = append(&mut blob, &ints_i32(&table));
        let cells_span = append(&mut blob, &ints_u32(&cells));
        let mut routes_span = append(&mut blob, &ints_u32(&routes));
        if short_span {
            routes_span["bytes"] = json!(1);
        }
        let gates_span = append(&mut blob, &ints_i32(&gates));
        let header = json!({
            "schema": "uor-r4.lut-discrete/1",
            "shape": {
                "vocabulary": 3, "blocks": 2, "context": 3, "kept_cells": kept_cells,
                "stages": 2, "read_mode": mode, "read_kept": read_kept,
                "read_sweeps": read_sweeps, "objective_policy": "nearest_address_margin_v1",
                "read_initialization": "zero_gates_v1", "seed": 1, "epoch": 0
            },
            "matrices": [],
            "tables": [
                {"name":"discrete.table","kind":"i32","len":table.len(),"span":table_span},
                {"name":"discrete.cells","kind":"u32","len":cells.len(),"span":cells_span},
                {"name":"discrete.routes","kind":"u32","len":routes.len(),"span":routes_span},
                {"name":"discrete.gates","kind":"i32","len":gates.len(),"span":gates_span}
            ],
            "source": {
                "snapshot_schema":"uor-r4.discrete-spiralcore-fbs/2",
                "snapshot_sha256":"00".repeat(32),
                "operator_kappa":"blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b",
                "composition_kappa":"blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0",
                "kernel_sha256":"11".repeat(32),
                "source_sha256":"22".repeat(32),
                "geometry_sha256":"33".repeat(32)
            }
        });
        let header = serde_json::to_vec(&header).unwrap();
        let mut out = Vec::new();
        out.extend_from_slice(b"UORLUT01");
        out.extend_from_slice(&(header.len() as u64).to_le_bytes());
        out.extend_from_slice(&header);
        out.resize(out.len().div_ceil(64) * 64, 0);
        out.extend_from_slice(&blob);
        out
    }

    struct DiscreteFixture {
        mode: &'static str,
        initialization: &'static str,
        kept: u32,
        sweeps: u32,
        routes: Vec<u32>,
        gates: Vec<i32>,
        roots: [[u8; 2]; 3],
        table: Option<Vec<i32>>,
        cells: Option<Vec<u32>>,
        short_span: bool,
    }

    impl DiscreteFixture {
        fn flock() -> Self {
            Self {
                mode: "flock",
                initialization: "active_leaf_v1",
                kept: 2,
                sweeps: 0,
                routes: vec![1, 2, 3, 4],
                gates: vec![0; 4],
                roots: [[0, 1], [2, 3], [4, 5]],
                table: None,
                cells: None,
                short_span: false,
            }
        }
    }

    fn serialized_discrete(fixture: DiscreteFixture) -> Vec<u8> {
        let mut table = Vec::new();
        let mut cells = Vec::new();
        for blocks in fixture.roots {
            for root in blocks {
                table.extend_from_slice(&orbit_root_coords(root).unwrap());
            }
            cells.push(u32::from(blocks[0] << 4 | blocks[1]));
        }
        let table = fixture.table.unwrap_or(table);
        let cells = fixture.cells.unwrap_or(cells);
        let mut blob = Vec::new();
        let table_span = append(&mut blob, &ints_i32(&table));
        let cells_span = append(&mut blob, &ints_u32(&cells));
        let routes_span = append(&mut blob, &ints_u32(&fixture.routes));
        let mut gates_span = append(&mut blob, &ints_i32(&fixture.gates));
        if fixture.short_span {
            gates_span["bytes"] = json!(1);
        }
        let header = json!({
            "schema": "uor-r4.lut-discrete/1",
            "shape": {
                "vocabulary": 3, "blocks": 2, "context": 3, "kept_cells": 2,
                "stages": 2, "read_mode": fixture.mode,
                "read_kept": fixture.kept, "read_sweeps": fixture.sweeps,
                "objective_policy": "nearest_address_margin_v1",
                "read_initialization": fixture.initialization,
                "seed": 7, "epoch": 0
            },
            "matrices": [],
            "tables": [
                {"name":"discrete.table","kind":"i32","len":table.len(),"span":table_span},
                {"name":"discrete.cells","kind":"u32","len":cells.len(),"span":cells_span},
                {"name":"discrete.routes","kind":"u32","len":fixture.routes.len(),"span":routes_span},
                {"name":"discrete.gates","kind":"i32","len":fixture.gates.len(),"span":gates_span}
            ],
            "source": {
                "snapshot_schema":"uor-r4.discrete-spiralcore-fbs/2",
                "snapshot_sha256":"00".repeat(32),
                "operator_kappa":"blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b",
                "composition_kappa":"blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0",
                "kernel_sha256":"11".repeat(32),
                "source_sha256":"22".repeat(32),
                "geometry_sha256":"33".repeat(32)
            }
        });
        let header = serde_json::to_vec(&header).unwrap();
        let mut out = Vec::new();
        out.extend_from_slice(b"UORLUT01");
        out.extend_from_slice(&(header.len() as u64).to_le_bytes());
        out.extend_from_slice(&header);
        out.resize(out.len().div_ceil(64) * 64, 0);
        out.extend_from_slice(&blob);
        out
    }

    fn rewrite_header(artifact: Vec<u8>, edit: impl FnOnce(&mut Value)) -> Vec<u8> {
        let header_len = u64::from_le_bytes(artifact[8..16].try_into().unwrap()) as usize;
        let data_start = (16 + header_len).div_ceil(64) * 64;
        let mut header: Value = serde_json::from_slice(&artifact[16..16 + header_len]).unwrap();
        edit(&mut header);
        let header = serde_json::to_vec(&header).unwrap();
        let mut out = Vec::new();
        out.extend_from_slice(b"UORLUT01");
        out.extend_from_slice(&(header.len() as u64).to_le_bytes());
        out.extend_from_slice(&header);
        out.resize(out.len().div_ceil(64) * 64, 0);
        out.extend_from_slice(&artifact[data_start..]);
        out
    }

    fn set_table_u32(artifact: &mut [u8], name: &str, index: usize, value: u32) {
        let parsed = header(artifact).unwrap();
        let header_len = u64::from_le_bytes(artifact[8..16].try_into().unwrap()) as usize;
        let data_start = (16 + header_len).div_ceil(64) * 64;
        let table = parsed["tables"]
            .as_array()
            .unwrap()
            .iter()
            .find(|table| table["name"].as_str() == Some(name))
            .unwrap();
        let offset = table["span"]["offset"].as_u64().unwrap() as usize;
        let start = data_start + offset + index * 4;
        artifact[start..start + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn serialized_flock_fixture(
        context: u32,
        roots: &[[u8; 2]],
        gates: &[i32],
        kept: u32,
    ) -> Vec<u8> {
        let stages = expected_stages(context as usize).unwrap() as u32;
        assert_eq!(gates.len(), 2 * stages as usize);
        let mut table = Vec::new();
        let mut cells = Vec::new();
        for blocks in roots {
            for &root in blocks {
                table.extend_from_slice(&orbit_root_coords(root).unwrap());
            }
            cells.push(u32::from(blocks[0] << 4 | blocks[1]));
        }
        let routes = vec![0u32; 2 * stages as usize];
        let mut blob = Vec::new();
        let table_span = append(&mut blob, &ints_i32(&table));
        let cells_span = append(&mut blob, &ints_u32(&cells));
        let routes_span = append(&mut blob, &ints_u32(&routes));
        let gates_span = append(&mut blob, &ints_i32(gates));
        let header = json!({
            "schema": "uor-r4.lut-discrete/1",
            "shape": {
                "vocabulary": roots.len(), "blocks": 2, "context": context,
                "kept_cells": 2, "stages": stages, "read_mode": "flock",
                "read_kept": kept, "read_sweeps": 0,
                "objective_policy": "nearest_address_margin_v1",
                "read_initialization": "active_leaf_v1", "seed": 7, "epoch": 0
            },
            "matrices": [],
            "tables": [
                {"name":"discrete.table","kind":"i32","len":table.len(),"span":table_span},
                {"name":"discrete.cells","kind":"u32","len":cells.len(),"span":cells_span},
                {"name":"discrete.routes","kind":"u32","len":routes.len(),"span":routes_span},
                {"name":"discrete.gates","kind":"i32","len":gates.len(),"span":gates_span}
            ],
            "source": {
                "snapshot_schema":"uor-r4.discrete-spiralcore-fbs/2",
                "snapshot_sha256":"00".repeat(32),
                "operator_kappa":"blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b",
                "composition_kappa":"blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0",
                "kernel_sha256":"11".repeat(32),
                "source_sha256":"22".repeat(32),
                "geometry_sha256":"33".repeat(32)
            }
        });
        let header = serde_json::to_vec(&header).unwrap();
        let mut out = Vec::new();
        out.extend_from_slice(b"UORLUT01");
        out.extend_from_slice(&(header.len() as u64).to_le_bytes());
        out.extend_from_slice(&header);
        out.resize(out.len().div_ceil(64) * 64, 0);
        out.extend_from_slice(&blob);
        out
    }

    fn serialized_relation_fixture(
        context: u32,
        roots: &[[u8; 2]],
        level_gates: &[i32],
        relation_gates: &[[i32; RELATION_EDGE_TYPES]],
        kept: u32,
        sweeps: u32,
    ) -> Vec<u8> {
        let stages = expected_stages(context as usize).unwrap();
        assert_eq!(level_gates.len(), 2 * stages);
        assert_eq!(relation_gates.len(), stages);
        let stages = stages as u32;
        let mut table = Vec::new();
        let mut cells = Vec::new();
        for blocks in roots {
            for &root in blocks {
                table.extend_from_slice(&orbit_root_coords(root).unwrap());
            }
            cells.push(u32::from(blocks[0] << 4 | blocks[1]));
        }
        let routes = vec![0u32; 2 * stages as usize];
        let mut gates = level_gates.to_vec();
        gates.extend(relation_gates.iter().flatten());
        let mut blob = Vec::new();
        let table_span = append(&mut blob, &ints_i32(&table));
        let cells_span = append(&mut blob, &ints_u32(&cells));
        let routes_span = append(&mut blob, &ints_u32(&routes));
        let gates_span = append(&mut blob, &ints_i32(&gates));
        let header = json!({
            "schema": "uor-r4.lut-discrete/1",
            "shape": {
                "vocabulary": roots.len(), "blocks": 2, "context": context,
                "kept_cells": 2, "stages": stages, "read_mode": "relation",
                "read_kept": kept, "read_sweeps": sweeps,
                "objective_policy": "nearest_address_margin_v1",
                "read_initialization": "active_leaf_v1", "seed": 7, "epoch": 0
            },
            "matrices": [],
            "tables": [
                {"name":"discrete.table","kind":"i32","len":table.len(),"span":table_span},
                {"name":"discrete.cells","kind":"u32","len":cells.len(),"span":cells_span},
                {"name":"discrete.routes","kind":"u32","len":routes.len(),"span":routes_span},
                {"name":"discrete.gates","kind":"i32","len":gates.len(),"span":gates_span}
            ],
            "source": {
                "snapshot_schema":"uor-r4.discrete-spiralcore-fbs/2",
                "snapshot_sha256":"00".repeat(32),
                "operator_kappa":"blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b",
                "composition_kappa":"blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0",
                "kernel_sha256":"11".repeat(32),
                "source_sha256":"22".repeat(32),
                "geometry_sha256":"33".repeat(32)
            }
        });
        let header = serde_json::to_vec(&header).unwrap();
        let mut out = Vec::new();
        out.extend_from_slice(b"UORLUT01");
        out.extend_from_slice(&(header.len() as u64).to_le_bytes());
        out.extend_from_slice(&header);
        out.resize(out.len().div_ceil(64) * 64, 0);
        out.extend_from_slice(&blob);
        out
    }

    fn append(blob: &mut Vec<u8>, bytes: &[u8]) -> serde_json::Value {
        blob.resize(blob.len().div_ceil(64) * 64, 0);
        let offset = blob.len();
        blob.extend_from_slice(bytes);
        json!({"offset": offset, "bytes": bytes.len()})
    }

    fn ints_i32(values: &[i32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn ints_u32(values: &[u32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }
}
