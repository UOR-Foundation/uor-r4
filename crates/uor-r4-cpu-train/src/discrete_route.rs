//! Independent CPU Route read for four-table discrete artifacts.

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
        if !(2..=8192).contains(&vocab)
            || !(2..=16).contains(&blocks)
            || !(1..=243).contains(&context)
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
        section_u32(bytes, &header, "discrete.cells", vocab, "u32")?;
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
        let routes = routes
            .chunks_exact(stages)
            .map(|row| row.to_vec())
            .collect();
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

fn root_id(block: &[i32; 8]) -> Option<usize> {
    (0..16u8)
        .find(|&root| orbit_root_coords(root).ok().as_ref() == Some(block))
        .map(usize::from)
}

fn expected_stages(context: usize) -> Option<usize> {
    if !(1..=243).contains(&context) {
        return None;
    }
    let mut stages = 1usize;
    let mut span = 3usize;
    while span <= context {
        stages += 1;
        span += span + span;
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
            nodes.push(BinderNode {
                children: Some(children),
                atom: None,
                level: level + 1,
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
    let end = 16 + usize::try_from(len).map_err(|_| invalid("Route header length"))?;
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
    let header_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let start = (16 + header_len as usize).div_ceil(64) * 64
        + usize::try_from(entry["span"]["offset"].as_u64().unwrap_or(u64::MAX))
            .map_err(|_| invalid("section offset"))?;
    let len = usize::try_from(entry["span"]["bytes"].as_u64().unwrap_or(0))
        .map_err(|_| invalid("section bytes"))?;
    let end = start
        .checked_add(len)
        .ok_or_else(|| invalid(format!("{name} end overflows")))?;
    bytes
        .get(start..end)
        .ok_or_else(|| invalid(format!("{name} outside artifact")))
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
    use serde_json::json;
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

    fn serialized_route() -> Vec<u8> {
        let roots = [[0u8, 1], [2, 3], [4, 5]];
        let mut table = Vec::new();
        let mut cells = Vec::new();
        for blocks in roots {
            for root in blocks {
                table.extend_from_slice(&orbit_root_coords(root).unwrap());
            }
            cells.push(u32::from(blocks[0] << 4 | blocks[1]));
        }
        let routes = vec![1u32, 2, 3, 4];
        let gates = vec![0i32; 4];
        let mut blob = Vec::new();
        let table_span = append(&mut blob, &ints_i32(&table));
        let cells_span = append(&mut blob, &ints_u32(&cells));
        let routes_span = append(&mut blob, &ints_u32(&routes));
        let gates_span = append(&mut blob, &ints_i32(&gates));
        let header = json!({
            "schema": "uor-r4.lut-discrete/1",
            "shape": {
                "vocabulary": 3, "blocks": 2, "context": 3, "kept_cells": 2,
                "stages": 2, "read_mode": "route", "read_kept": 0,
                "read_sweeps": 0, "objective_policy": "nearest_address_margin_v1",
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
