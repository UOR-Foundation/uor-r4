//! Independent CPU Route and Flock reads for four-table discrete artifacts.

use uor_r4_core::spiralcore_operator::{cl06_finite_composition_table, Cl06FiniteCompositionTable};
use uor_r4_discrete::normalization_mechanism::orbit_root_coords;

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
pub struct CpuDiscreteFlock {
    context: usize,
    blocks: usize,
    stages: usize,
    kept: usize,
    token_blocks: Vec<Vec<[i32; 8]>>,
    level_gates: Vec<Vec<i32>>,
    composition: Cl06FiniteCompositionTable,
}

impl CpuDiscreteFlock {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let header = header(bytes)?;
        if header["schema"].as_str() != Some("uor-r4.lut-discrete/1") {
            return Err(invalid("Flock CPU read requires uor-r4.lut-discrete/1"));
        }
        let shape = &header["shape"];
        if shape["read_mode"].as_str() != Some("flock")
            || shape["read_initialization"].as_str() != Some("active_leaf_v1")
        {
            return Err(invalid("Flock CPU read refuses non-Flock initialization"));
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
        if !(2..=8192).contains(&vocab)
            || !(2..=16).contains(&blocks)
            || !(1..=243).contains(&context)
            || !(1..=16).contains(&kept_cells)
            || kept_cells > vocab
            || !(1..=8).contains(&kept)
            || shape["read_sweeps"].as_u64() != Some(0)
            || expected_stages(context) != Some(stages)
        {
            return Err(invalid("Flock shape differs"));
        }
        let tables = header["tables"]
            .as_array()
            .ok_or_else(|| invalid("Flock artifact tables differ"))?;
        if tables.len() != 4 {
            return Err(invalid("Flock artifact requires four tables"));
        }
        let table = section_i32(bytes, &header, "discrete.table", vocab * blocks * 8, "i32")?;
        let cells = section_u32(bytes, &header, "discrete.cells", vocab, "u32")?;
        let routes = section_u32(bytes, &header, "discrete.routes", blocks * stages, "u32")?;
        let gates = section_i32(bytes, &header, "discrete.gates", blocks * stages, "i32")?;
        if routes.iter().any(|&action| action >= 64) {
            return Err(invalid("Flock route action outside 0..64"));
        }
        if gates.iter().any(|&gate| !(-1..=1).contains(&gate)) {
            return Err(invalid("Flock level gate outside -1..=1"));
        }
        let token_blocks = decode_token_blocks(&table, &cells, vocab, blocks, "Flock")?;
        validate_layout(bytes, &header)?;
        let level_gates = gates.chunks_exact(stages).map(|row| row.to_vec()).collect();
        Self::from_parts(context, kept, token_blocks, level_gates)
    }

    pub fn from_parts(
        context: usize,
        kept: usize,
        token_blocks: Vec<Vec<[i32; 8]>>,
        level_gates: Vec<Vec<i32>>,
    ) -> Result<Self> {
        let blocks = token_blocks.first().map_or(0, Vec::len);
        let stages = level_gates.first().map_or(0, Vec::len);
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
        {
            return Err(invalid("Flock parts differ"));
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
            composition,
        })
    }

    pub fn read(&self, prefix: &[u32]) -> Result<Vec<i32>> {
        if prefix.is_empty() || prefix.len() > self.context {
            return Err(invalid("Flock prefix outside context"));
        }
        if prefix
            .iter()
            .any(|&token| token as usize >= self.token_blocks.len())
        {
            return Err(invalid("Flock prefix token outside vocabulary"));
        }
        let plan = build_row_plan(prefix.len())?;
        let length = prefix.len();
        let last_token = prefix[length - 1] as usize;
        let mut out = Vec::with_capacity(self.blocks * 8);
        for block in 0..self.blocks {
            let frame_block = if block == 0 { self.blocks - 1 } else { 0 };
            let query_frame = &self.token_blocks[last_token][frame_block];
            let query_frame_root =
                root_id(query_frame).ok_or_else(|| invalid("Flock query frame outside orbit"))?;
            let mut value = self.token_blocks[last_token][block];
            let mut terms = 1usize;

            let mut frames = vec![0usize; plan.nodes.len()];
            let mut payload = vec![[0i32; 8]; plan.nodes.len()];
            for (node_index, node) in plan.nodes.iter().enumerate() {
                if let Some(atom) = node.atom {
                    let token = prefix[atom] as usize;
                    frames[node_index] = root_id(&self.token_blocks[token][frame_block])
                        .ok_or_else(|| invalid("Flock atom frame outside orbit"))?;
                    payload[node_index] = self.token_blocks[token][block];
                } else {
                    let children = node
                        .children
                        .ok_or_else(|| invalid("Flock non-atom node missing children"))?;
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

            for level in 0..self.stages {
                let gate = self.level_gates[block][level];
                if gate == 0 {
                    continue;
                }
                let mut top = [0usize; 8];
                let mut key_distance = [u32::MAX; 8];
                let mut key_hamming = [u32::MAX; 8];
                let mut found = 0usize;
                for (node, candidate) in plan.nodes.iter().enumerate() {
                    if candidate.level != level || candidate.last + 1 >= length {
                        continue;
                    }
                    let (distance, hamming) = root_cost(query_frame, frames[node]);
                    let mut at = found;
                    while at > 0 {
                        let previous = top[at - 1];
                        let better = distance < key_distance[at - 1]
                            || (distance == key_distance[at - 1]
                                && (hamming < key_hamming[at - 1]
                                    || (hamming == key_hamming[at - 1]
                                        && (plan.nodes[node].last > plan.nodes[previous].last
                                            || (plan.nodes[node].last
                                                == plan.nodes[previous].last
                                                && node < previous)))));
                        if !better {
                            break;
                        }
                        if at < self.kept {
                            top[at] = previous;
                            key_distance[at] = key_distance[at - 1];
                            key_hamming[at] = key_hamming[at - 1];
                        }
                        at -= 1;
                    }
                    if at < self.kept {
                        top[at] = node;
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
            .ok_or_else(|| invalid("Flock action state absent"))?
            .apply_root_key(root)
            .map_err(|error| invalid(error.to_string()))?;
        Ok(out.map(|value| i32::from(value) * 2))
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
