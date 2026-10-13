//! Native eight-dimensional projection: retained ordered ternary binders select
//! an exact carrier; SpiralCore BFS supplies its actual signed operator route.
//!
//! This is an explicit coupling of the FBS binder abstraction to the v63 E8
//! action graph, not a derivation of a metric from the full external FBS object.
//! No input/output coefficient grid exists. Old quaternion arrays cannot be
//! reinterpreted as these routing points. Model arithmetic is CUDA-only.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Arc, OnceLock};

#[cfg(any(test, feature = "cuda"))]
use candle_core::Device;
use candle_core::{CpuStorage, CustomOp2, DType, Layout, Shape, Tensor};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uor_r4_core::spiralcore_operator::{
    cl06_bivectors, cl06_finite_composition_table, SignedMatrix8,
};

use crate::{invalid, Result};

pub const SCHEMA: &str = "spiralcore-fbs-projection/1";
pub const FBS_SOURCE_SHA256: &str =
    "917cb942811971d63cbe499f3a1641d3360c4dde7ebdc6b87e298c3cf1bbd747";
pub const DIMENSION: usize = 8;
pub const ORBIT: usize = 16;
pub const MAX_LEVELS: usize = 8;
pub const MAX_ATOMS: usize = 6560; // 3^8 - 1: at most sixteen frontier roots.
pub const ABSENT: u32 = u32::MAX;

/// A complete child-bearing binder, never a padded or reduced terminal label.
/// The canonical atom uses tau=2, g=1, support length=8; support length is not
/// asserted to be a metric or a semantic dimension of the external reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Binder {
    pub children: Option<[u32; 3]>,
    pub atom: Option<u32>,
    pub level: u32,
    pub support_length: u64,
    pub first_atom: u32,
    pub atom_count: u32,
    pub parent: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct Geometry {
    pub operator_kappa: String,
    pub composition_kappa: String,
    pub roots: [[i8; DIMENSION]; ORBIT],
    /// Packed signed axes: low three bits select a coordinate; bit3 negates.
    pub axes: Vec<u32>,
    pub compose: Vec<u32>,
    pub root_of_state: Vec<u32>,
    /// Actual BFS operator compositions and retained edge histories, by root pair.
    pub bfs_actions: Vec<u32>,
    pub bfs_edges: Vec<Vec<u8>>,
    pub branch_targets: Vec<u32>,
    pub identity: u32,
}

fn geometry() -> Result<Arc<Geometry>> {
    static VALUE: OnceLock<std::result::Result<Arc<Geometry>, String>> = OnceLock::new();
    VALUE
        .get_or_init(|| compile_geometry().map(Arc::new).map_err(|e| e.to_string()))
        .as_ref()
        .map(Arc::clone)
        .map_err(|e| invalid(e.clone()))
}

fn compile_geometry() -> Result<Geometry> {
    let table = cl06_finite_composition_table().map_err(|e| invalid(e.to_string()))?;
    let anchor = [2, 2, 0, 0, 0, 0, 0, 0];
    let mut points = BTreeSet::new();
    for state in table.states() {
        points.insert(
            state
                .apply_root_key(anchor)
                .map_err(|e| invalid(e.to_string()))?,
        );
    }
    let mut ordered = vec![anchor];
    ordered.extend(points.into_iter().filter(|root| *root != anchor));
    let roots: [[i8; DIMENSION]; ORBIT] = ordered
        .try_into()
        .map_err(|_| invalid("SpiralCore anchor orbit is not sixteen roots"))?;
    let indices: BTreeMap<_, _> = roots
        .iter()
        .copied()
        .enumerate()
        .map(|(i, root)| (root, i as u32))
        .collect();
    let state_index = |matrix: &SignedMatrix8| -> Result<u32> {
        table
            .states()
            .iter()
            .position(|s| s == matrix)
            .map(|i| i as u32)
            .ok_or_else(|| invalid("BFS action is outside the bound SpiralCore group"))
    };
    let identity = state_index(&SignedMatrix8::identity())?;
    let mut axes = Vec::with_capacity(64 * DIMENSION);
    for matrix in table.states() {
        for row in matrix.entries() {
            let mut selected = None;
            for (column, &value) in row.iter().enumerate() {
                if value == 0 {
                    continue;
                }
                if selected.is_some() || !matches!(value, -1 | 1) {
                    return Err(invalid("non-permutation SpiralCore action"));
                }
                selected = Some(column as u32 | if value < 0 { 8 } else { 0 });
            }
            axes.push(selected.ok_or_else(|| invalid("empty SpiralCore axis"))?);
        }
    }
    let mut compose = Vec::with_capacity(4096);
    for left in table.states() {
        for right in table.states() {
            compose.push(state_index(
                &left
                    .checked_mul(*right)
                    .map_err(|e| invalid(e.to_string()))?,
            )?);
        }
    }
    let root_of_state = table
        .states()
        .iter()
        .map(|state| {
            let root = state
                .apply_root_key(anchor)
                .map_err(|e| invalid(e.to_string()))?;
            indices
                .get(&root)
                .copied()
                .ok_or_else(|| invalid("missing orbit root"))
        })
        .collect::<Result<Vec<_>>>()?;
    let bivectors = cl06_bivectors().map_err(|e| invalid(e.to_string()))?;
    let edge_actions = bivectors
        .iter()
        .map(|b| state_index(&b.matrix))
        .collect::<Result<Vec<_>>>()?;
    let mut neighbors = Vec::new();
    for root in &roots {
        let mut adjacent = Vec::new();
        for b in &bivectors {
            let image = b
                .matrix
                .apply_root_key(*root)
                .map_err(|e| invalid(e.to_string()))?;
            adjacent.push(
                *indices
                    .get(&image)
                    .ok_or_else(|| invalid("edge leaves orbit"))?,
            );
        }
        neighbors.push(adjacent);
    }
    let mut bfs_actions = Vec::with_capacity(ORBIT * ORBIT);
    let mut bfs_edges = Vec::with_capacity(ORBIT * ORBIT);
    for start in 0..ORBIT {
        let mut paths: Vec<Option<(u32, Vec<u8>)>> = vec![None; ORBIT];
        paths[start] = Some((identity, Vec::new()));
        let mut queue = VecDeque::from([start]);
        while let Some(from) = queue.pop_front() {
            let (state, history) = paths[from]
                .clone()
                .ok_or_else(|| invalid("missing BFS prefix"))?;
            for (edge, &to) in neighbors[from].iter().enumerate() {
                if paths[to as usize].is_some() {
                    continue;
                }
                let mut next = history.clone();
                next.push(edge as u8);
                let action = compose[edge_actions[edge] as usize * 64 + state as usize];
                paths[to as usize] = Some((action, next));
                queue.push_back(to as usize);
            }
        }
        for path in paths {
            let (action, history) = path.ok_or_else(|| invalid("disconnected SpiralCore orbit"))?;
            bfs_actions.push(action);
            bfs_edges.push(history);
        }
    }
    let generator = bivectors
        .first()
        .ok_or_else(|| invalid("missing bivector"))?
        .matrix;
    let inverse = generator
        .checked_power(3)
        .map_err(|e| invalid(e.to_string()))?;
    let mut branch_targets = Vec::with_capacity(ORBIT * 3);
    for root in &roots {
        let mut ports = BTreeSet::new();
        for action in [SignedMatrix8::identity(), generator, inverse] {
            let image = action
                .apply_root_key(*root)
                .map_err(|e| invalid(e.to_string()))?;
            let port = *indices
                .get(&image)
                .ok_or_else(|| invalid("port leaves orbit"))?;
            ports.insert(port);
            branch_targets.push(port);
        }
        if ports.len() != 3 {
            return Err(invalid("ternary geometry ports coincide"));
        }
    }
    let report = table.validate().map_err(|e| invalid(e.to_string()))?;
    Ok(Geometry {
        operator_kappa: report.operator_kappa,
        composition_kappa: table
            .reproduce_kappa()
            .map_err(|e| invalid(e.to_string()))?,
        roots,
        axes,
        compose,
        root_of_state,
        bfs_actions,
        bfs_edges,
        branch_targets,
        identity,
    })
}

#[derive(Debug, Serialize)]
pub struct ProjectionPlan {
    pub(crate) schema: &'static str,
    pub(crate) fbs_source_sha256: &'static str,
    pub(crate) atom_mode: [u32; 3], // tau, g, atomic support length
    pub(crate) compiler_sha256: String,
    pub(crate) kernel_sha256: String,
    pub(crate) input_blocks: usize,
    pub(crate) output_blocks: usize,
    pub(crate) stages: usize,
    pub(crate) binders: Vec<Binder>,
    pub(crate) frontier: Vec<u32>,
    #[serde(serialize_with = "serialize_geometry")]
    pub(crate) geometry: Arc<Geometry>,
    #[serde(skip)]
    pub(crate) fingerprint: String,
}

fn serialize_geometry<S: serde::Serializer>(
    value: &Arc<Geometry>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    value.as_ref().serialize(serializer)
}

impl ProjectionPlan {
    /// Static construction only. No model tensor or learned activation is read.
    pub fn new(input_width: usize, output_width: usize) -> Result<Arc<Self>> {
        if input_width == 0 || output_width == 0 || input_width % 8 != 0 || output_width % 8 != 0 {
            return Err(invalid(
                "SpiralCore/FBS projection requires nonzero whole 8D carriers",
            ));
        }
        let input_blocks = input_width / 8;
        if input_blocks > MAX_ATOMS {
            return Err(invalid(
                "FBS projection exceeds the bound eight-rung frontier",
            ));
        }
        let mut binders: Vec<Binder> = Vec::new();
        let mut frontier: Vec<u32> = Vec::new();
        for atom in 0..input_blocks {
            let id = binders.len() as u32;
            binders.push(Binder {
                children: None,
                atom: Some(atom as u32),
                level: 0,
                support_length: 8,
                first_atom: atom as u32,
                atom_count: 1,
                parent: None,
            });
            frontier.push(id);
            loop {
                if frontier.len() < 3 {
                    break;
                }
                let n = frontier.len();
                let children = [frontier[n - 3], frontier[n - 2], frontier[n - 1]];
                let level = binders[children[0] as usize].level;
                if children.iter().any(|&c| binders[c as usize].level != level) {
                    break;
                }
                let parent = binders.len() as u32;
                let span = children
                    .iter()
                    .map(|&c| binders[c as usize].support_length)
                    .sum::<u64>()
                    + 1;
                let count = children
                    .iter()
                    .map(|&c| binders[c as usize].atom_count)
                    .sum();
                let first_atom = binders[children[0] as usize].first_atom;
                for child in children {
                    binders[child as usize].parent = Some(parent);
                }
                binders.push(Binder {
                    children: Some(children),
                    atom: None,
                    level: level + 1,
                    support_length: span,
                    first_atom,
                    atom_count: count,
                    parent: None,
                });
                frontier.truncate(n - 3);
                frontier.push(parent);
            }
        }
        let stages = 1 + binders.iter().map(|b| b.level as usize).max().unwrap_or(0);
        if frontier.len() > ORBIT || stages > MAX_LEVELS {
            return Err(invalid(
                "FBS frontier cannot be represented by the bound SpiralCore orbit",
            ));
        }
        let mut plan = Self {
            schema: SCHEMA,
            fbs_source_sha256: FBS_SOURCE_SHA256,
            atom_mode: [2, 1, 8],
            compiler_sha256: hex::encode(Sha256::digest(include_bytes!("spiralcore_fbs.rs"))),
            kernel_sha256: hex::encode(Sha256::digest(include_bytes!("spiralcore_fbs.cu"))),
            input_blocks,
            output_blocks: output_width / 8,
            stages,
            binders,
            frontier,
            geometry: geometry()?,
            fingerprint: String::new(),
        };
        plan.fingerprint = hex::encode(Sha256::digest(serde_json::to_vec(&plan)?));
        Ok(Arc::new(plan))
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
    pub fn binders(&self) -> &[Binder] {
        &self.binders
    }
    pub fn frontier(&self) -> &[u32] {
        &self.frontier
    }

    pub fn query_shape(&self) -> [usize; 3] {
        [self.output_blocks, self.stages, DIMENSION]
    }

    /// Kernel ABI: real nodes only; ABSENT is a type marker, never a padded atom.
    pub fn node_words(&self) -> Vec<u32> {
        self.binders
            .iter()
            .flat_map(|b| {
                let c = b.children.unwrap_or([ABSENT; 3]);
                [c[0], c[1], c[2], b.atom.unwrap_or(ABSENT)]
            })
            .collect()
    }

    pub fn forward(self: &Arc<Self>, queries: &Tensor, input: &Tensor) -> Result<Tensor> {
        if queries.device().is_cpu() || input.device().is_cpu() {
            return Err(invalid(
                "SpiralCore/FBS model projection is GPU-only; CPU execution is forbidden",
            ));
        }
        if queries.device().location() != input.device().location() {
            return Err(invalid(
                "SpiralCore/FBS queries and carriers must share one GPU",
            ));
        }
        if queries.dtype() != DType::F32 || !matches!(input.dtype(), DType::F32 | DType::BF16) {
            return Err(invalid(
                "native routing points are F32 masters; carriers are F32 or BF16",
            ));
        }
        if queries.dims() != self.query_shape()
            || input.dims().last() != Some(&(8 * self.input_blocks))
        {
            return Err(invalid(
                "SpiralCore/FBS tensors disagree with the bound projection plan",
            ));
        }
        if input.elem_count() == 0 {
            return Err(invalid("empty SpiralCore/FBS carrier batch"));
        }
        Ok(queries
            .contiguous()?
            .apply_op2(&input.contiguous()?, NativeProjection(Arc::clone(self)))?)
    }
}

struct NativeProjection(Arc<ProjectionPlan>);

impl CustomOp2 for NativeProjection {
    fn name(&self) -> &'static str {
        "spiralcore-fbs-native-projection"
    }
    fn cpu_fwd(
        &self,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        candle_core::bail!("CPU model projection is forbidden; no host fallback")
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        q: &candle_core::CudaStorage,
        ql: &Layout,
        x: &candle_core::CudaStorage,
        xl: &Layout,
    ) -> candle_core::Result<(candle_core::CudaStorage, Shape)> {
        crate::geometric_stack::cuda_ops::spiralcore_fbs_fwd(&self.0, q, ql, x, xl)
    }
    #[cfg(feature = "metal")]
    fn metal_fwd(
        &self,
        _: &candle_core::MetalStorage,
        _: &Layout,
        _: &candle_core::MetalStorage,
        _: &Layout,
    ) -> candle_core::Result<(candle_core::MetalStorage, Shape)> {
        candle_core::bail!("SpiralCore/FBS has no Metal kernel; host execution is forbidden")
    }
    fn bwd(
        &self,
        q: &Tensor,
        x: &Tensor,
        _: &Tensor,
        dy: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        #[cfg(feature = "cuda")]
        if let Device::Cuda(device) = q.device() {
            let (dq, dx) =
                crate::geometric_stack::cuda_ops::spiralcore_fbs_bwd(&self.0, device, q, x, dy)?;
            return Ok((Some(dq), Some(dx)));
        }
        let _ = (q, x, dy, &self.0);
        candle_core::bail!("SpiralCore/FBS backward is GPU-only; no host adjoint")
    }
}

#[cfg(test)]
mod static_plan_tests {
    use super::*;

    #[test]
    fn retained_frontier_has_every_atom_once_without_padding() -> Result<()> {
        for atoms in [1usize, 2, 3, 4, 8, 16, 32, 64, 128, 512, MAX_ATOMS] {
            let plan = ProjectionPlan::new(atoms * 8, 16)?;
            let mut covered = Vec::new();
            fn visit(p: &ProjectionPlan, id: u32, out: &mut Vec<u32>) {
                let b = &p.binders[id as usize];
                if let Some(c) = b.children {
                    assert_eq!(
                        b.support_length,
                        c.iter()
                            .map(|&i| p.binders[i as usize].support_length)
                            .sum::<u64>()
                            + 1
                    );
                    for child in c {
                        assert_eq!(p.binders[child as usize].parent, Some(id));
                        visit(p, child, out);
                    }
                } else {
                    out.push(b.atom.unwrap());
                }
            }
            for &root in &plan.frontier {
                assert_eq!(plan.binders[root as usize].parent, None);
                visit(&plan, root, &mut covered);
            }
            assert_eq!(covered, (0..atoms as u32).collect::<Vec<_>>());
            assert!(plan.frontier.len() <= ORBIT && plan.stages <= MAX_LEVELS);
            assert_eq!(
                plan.fingerprint,
                ProjectionPlan::new(atoms * 8, 16)?.fingerprint
            );
        }
        assert!(ProjectionPlan::new((MAX_ATOMS + 1) * 8, 16).is_err());
        assert!(ProjectionPlan::new(12, 16).is_err());
        Ok(())
    }

    #[test]
    fn bfs_keeps_actual_edge_history_operator_and_inverse() -> Result<()> {
        let g = geometry()?;
        let table = cl06_finite_composition_table().map_err(|e| invalid(e.to_string()))?;
        let edges = cl06_bivectors().map_err(|e| invalid(e.to_string()))?;
        for from in 0..ORBIT {
            for to in 0..ORBIT {
                let mut matrix = SignedMatrix8::identity();
                let mut root = g.roots[from];
                for &edge in &g.bfs_edges[from * ORBIT + to] {
                    let action = edges[edge as usize].matrix;
                    root = action
                        .apply_root_key(root)
                        .map_err(|e| invalid(e.to_string()))?;
                    matrix = action
                        .checked_mul(matrix)
                        .map_err(|e| invalid(e.to_string()))?;
                }
                assert_eq!(root, g.roots[to]);
                assert_eq!(
                    table.states()[g.bfs_actions[from * ORBIT + to] as usize],
                    matrix
                );
                let inverse = table
                    .inverse_index(g.bfs_actions[from * ORBIT + to] as u8)
                    .ok_or_else(|| invalid("missing inverse"))?;
                assert_eq!(
                    table.states()[inverse as usize]
                        .apply_root_key(root)
                        .map_err(|e| invalid(e.to_string()))?,
                    g.roots[from]
                );
            }
        }
        for ports in g.branch_targets.chunks_exact(3) {
            assert_eq!(ports.iter().copied().collect::<BTreeSet<_>>().len(), 3);
        }
        Ok(())
    }

    #[test]
    fn cpu_model_entry_is_refused_before_any_carrier_arithmetic() -> Result<()> {
        let plan = ProjectionPlan::new(16, 8)?;
        let q = Tensor::zeros(&plan.query_shape(), DType::F32, &Device::Cpu)?;
        let x = Tensor::zeros((2, 16), DType::F32, &Device::Cpu)?;
        assert!(plan
            .forward(&q, &x)
            .unwrap_err()
            .to_string()
            .contains("GPU-only"));
        Ok(())
    }
}
