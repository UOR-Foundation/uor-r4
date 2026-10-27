// Deterministic backward of the SpiralCore/FBS projection (d11-exporter-float-trunk, claude-141).
// This module is built on its own (build.rs) from the arm prelude, the pinned
// src/spiralcore_fbs.cu (included unchanged, for its route helpers) and this file. F32 only.
//
// It is the adjoint of the pinned `spiralcore_fbs_bwd`, term for term, with one change: the pinned
// kernel adds each (row, output) item's terms into shared sums with float `atomicAdd`, so the
// order of the additions follows the GPU's thread order and two runs can differ (claude-140).
// Here every sum is a gather in a fixed order. A route depends only on its output's queries, never
// on the row, so:
//   - `spiralcore_fbs_det_bwd_input` runs one thread per row and adds that row's input adjoint
//     over the outputs in index order;
//   - `spiralcore_fbs_det_bwd_query` runs one thread per output and adds that output's query
//     pullback over the rows in index order.
// Each thread owns the sums it writes, so no float atomics occur; the error word keeps the pinned
// kernel's codes and conditions (1 non-finite query, 2 absent atom, 3 non-finite carrier or
// gradient, 4 non-finite sum, contrast or degenerate edge).

extern "C" __global__ void spiralcore_fbs_det_bwd_input(
    const float* queries, const act_t* input, const act_t* grad,
    const uint* nodes, const uint* frontier, const uint* axes,
    const uint* compose, const uint* root_of_state, const uint* bfs_actions,
    const uint* branch_targets, const float* roots,
    float* d_queries, float* d_input, uint* error,
    uint rows, uint outputs, uint inputs, uint stages, uint frontier_count, uint identity
) {
    (void)d_queries;
    uint row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= rows) return;
    ScFbsTables t = {nodes, frontier, axes, compose, root_of_state,
        bfs_actions, branch_targets, roots, stages, frontier_count, identity};
    const act_t* x = input + (u64)row * inputs * 8;
    float* dx = d_input + (u64)row * inputs * 8;
    for (uint output = 0; output < outputs; ++output) {
        const float* q = queries + (u64)output * stages * 8;
        for (uint i = 0; i < stages * 8; ++i) {
            if (!isfinite(q[i])) { atomicExch(error, 1u); return; }
        }
        ScFbsRoute selected = sc_fbs_route(t, q, SC_FBS_ABSENT, 0);
        if (selected.atom == SC_FBS_ABSENT || selected.atom >= inputs) { atomicExch(error, 2u); return; }
        u64 item = (u64)row * outputs + output;
        for (uint axis = 0; axis < 8; ++axis) {
            float dy = act_to_f(grad[item * 8 + axis]);
            float y = sc_fbs_coordinate(t, x, selected.atom, selected.state, axis);
            if (!isfinite(dy) || !isfinite(y)) { atomicExch(error, 3u); return; }
            uint code = axes[selected.state * 8 + axis];
            float* slot = dx + selected.atom * 8 + (code & 7);
            float sum = *slot + (code & 8 ? -dy : dy);
            if (!isfinite(sum)) { atomicExch(error, 4u); return; }
            *slot = sum;
        }
    }
}

extern "C" __global__ void spiralcore_fbs_det_bwd_query(
    const float* queries, const act_t* input, const act_t* grad,
    const uint* nodes, const uint* frontier, const uint* axes,
    const uint* compose, const uint* root_of_state, const uint* bfs_actions,
    const uint* branch_targets, const float* roots,
    float* d_queries, float* d_input, uint* error,
    uint rows, uint outputs, uint inputs, uint stages, uint frontier_count, uint identity
) {
    (void)d_input;
    uint output = blockIdx.x * blockDim.x + threadIdx.x;
    if (output >= outputs) return;
    if (stages > SC_FBS_LEVELS) { atomicExch(error, 2u); return; }
    ScFbsTables t = {nodes, frontier, axes, compose, root_of_state,
        bfs_actions, branch_targets, roots, stages, frontier_count, identity};
    const float* q = queries + (u64)output * stages * 8;
    for (uint i = 0; i < stages * 8; ++i) {
        if (!isfinite(q[i])) { atomicExch(error, 1u); return; }
    }
    ScFbsRoute selected = sc_fbs_route(t, q, SC_FBS_ABSENT, 0);
    if (selected.atom == SC_FBS_ABSENT || selected.atom >= inputs) { atomicExch(error, 2u); return; }
    if (frontier_count > SC_FBS_ORBIT) { atomicExch(error, 2u); return; }
    // Every alternative route depends only on the queries, so it is found once per output, before
    // the rows (the pinned kernel recomputes it in every row's thread; the routes are the same).
    uint alt_atom[SC_FBS_LEVELS][SC_FBS_ORBIT];
    uint alt_state[SC_FBS_LEVELS][SC_FBS_ORBIT];
    for (uint stage = 0; stage < selected.count; ++stage) {
        uint count = stage == 0 ? frontier_count : 3;
        for (uint sibling = 0; sibling < count; ++sibling) {
            if (sibling == selected.choice[stage]) continue;
            ScFbsRoute alternative = sc_fbs_route(t, q, stage, sibling);
            if (alternative.atom == SC_FBS_ABSENT || alternative.atom >= inputs) {
                atomicExch(error, 2u); return;
            }
            alt_atom[stage][sibling] = alternative.atom;
            alt_state[stage][sibling] = alternative.state;
        }
    }
    float acc[SC_FBS_LEVELS * 8];
    for (uint i = 0; i < stages * 8; ++i) acc[i] = 0.0f;
    for (uint row = 0; row < rows; ++row) {
        const act_t* x = input + (u64)row * inputs * 8;
        u64 item = (u64)row * outputs + output;
        float dy[8], y[8];
        for (uint axis = 0; axis < 8; ++axis) {
            dy[axis] = act_to_f(grad[item * 8 + axis]);
            y[axis] = sc_fbs_coordinate(t, x, selected.atom, selected.state, axis);
            if (!isfinite(dy[axis]) || !isfinite(y[axis])) { atomicExch(error, 3u); return; }
        }
        for (uint stage = 0; stage < selected.count; ++stage) {
            uint count = stage == 0 ? frontier_count : 3;
            uint root = root_of_state[selected.before[stage]];
            uint old_goal = selected.goal[stage];
            for (uint sibling = 0; sibling < count; ++sibling) {
                if (sibling == selected.choice[stage]) continue;
                uint goal = stage == 0 ? sibling : branch_targets[root * 3 + sibling];
                float contrast = 0.0f, squared_edge = 0.0f;
                for (uint axis = 0; axis < 8; ++axis) {
                    float other = sc_fbs_coordinate(
                        t, x, alt_atom[stage][sibling], alt_state[stage][sibling], axis);
                    if (!isfinite(other)) { atomicExch(error, 3u); return; }
                    contrast += dy[axis] * (other - y[axis]);
                    float edge = roots[goal * 8 + axis] - roots[old_goal * 8 + axis];
                    squared_edge += edge * edge;
                }
                if (squared_edge <= 0.0f || !isfinite(contrast)) { atomicExch(error, 4u); return; }
                float directional = contrast / squared_edge;
                for (uint axis = 0; axis < 8; ++axis) {
                    float edge = roots[goal * 8 + axis] - roots[old_goal * 8 + axis];
                    // The pinned kernel rounds this product before its atomic add; `__fmul_rn`
                    // keeps that rounding here (no fused multiply-add).
                    float sum = __fadd_rn(acc[stage * 8 + axis], __fmul_rn(directional, edge));
                    if (!isfinite(sum)) { atomicExch(error, 4u); return; }
                    acc[stage * 8 + axis] = sum;
                }
            }
        }
    }
    float* dq = d_queries + (u64)output * stages * 8;
    for (uint i = 0; i < stages * 8; ++i) dq[i] = acc[i];
}
