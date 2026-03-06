#!/usr/bin/env python3
import argparse
import json
import math
import os
import sys
import time
from typing import Dict, List, Sequence, Tuple

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

import hyperbolic_router_so8 as hr
from tasks.wikitext2_proxy import contexts_targets, hashed_targets


def _subset_with_index(X: np.ndarray, Y: np.ndarray, T: np.ndarray, max_n: int, seed: int):
    if max_n <= 0 or X.shape[0] <= max_n:
        idx = np.arange(X.shape[0], dtype=np.int64)
        return X, Y, T, idx
    rs = np.random.RandomState(seed)
    idx = rs.permutation(X.shape[0])[:max_n]
    return X[idx], Y[idx], T[idx], idx


def _normalize_rows(X: np.ndarray) -> np.ndarray:
    n = np.linalg.norm(X, axis=1, keepdims=True)
    n = np.maximum(n, 1e-8)
    return (X / n).astype(np.float64)


def apply_retrieval_fast_dev(args: argparse.Namespace):
    if int(args.fast_dev) != 1:
        return
    args.max_train = min(int(args.max_train), 3000)
    args.max_eval = min(int(args.max_eval), 1500)
    args.chart_iters = min(int(args.chart_iters), 40)
    args.kmeans_iters = min(int(args.kmeans_iters), 12)
    args.so8_candidates = min(int(args.so8_candidates), 2)
    args.scale_candidates = min(int(args.scale_candidates), 2)
    args.probe_buckets = min(int(args.probe_buckets), 2)


def compute_amortized_retrieval_metrics(offline_total: float, online_total: float, query_repeats: int) -> Tuple[float, float]:
    reps = max(1, int(query_repeats))
    return float(online_total / reps), float((offline_total + online_total) / reps)


def parse_args():
    ap = argparse.ArgumentParser()
    ap.add_argument("--input", type=str, default="data/wikitext2_proxy/wikitext2_proxy.npz")
    ap.add_argument("--tokens_input", type=str, default="data/wikitext2_proxy/wikitext2_tokens.npz")
    ap.add_argument("--proxy_meta", type=str, default="data/wikitext2_proxy/wikitext2_proxy_meta.json")
    ap.add_argument("--eval_split", type=str, default="test", choices=["test", "val"])
    ap.add_argument("--max_train", type=int, default=12000)
    ap.add_argument("--max_eval", type=int, default=6000)

    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--retrieval_backend", type=str, default="routed_probe", choices=["dense_exact", "routed_probe"])
    ap.add_argument("--topk", type=int, default=8)
    ap.add_argument("--probe_buckets", type=int, default=1)
    ap.add_argument("--query_repeats", type=int, default=1)
    ap.add_argument("--route_key_mode", type=str, default="hopf_bucket", choices=["hopf_bucket", "hopf_plus_complex"])
    ap.add_argument("--complex_key_roots", type=int, default=8)
    ap.add_argument("--complex_key_radius_bins", type=int, default=1)
    ap.add_argument("--complex_backfill_items", type=int, default=0)
    ap.add_argument("--complex_backfill_mode", type=str, default="always", choices=["always", "small_bucket", "low_margin"])
    ap.add_argument("--complex_backfill_max_exact", type=int, default=0)
    ap.add_argument("--complex_backfill_margin_threshold", type=float, default=0.0)

    ap.add_argument("--K", type=int, default=8)
    ap.add_argument("--delta_r", type=float, default=3.0)
    ap.add_argument("--kmeans_iters", type=int, default=25)

    ap.add_argument("--epochs", type=int, default=1)
    ap.add_argument("--eta_p", type=float, default=0.04)
    ap.add_argument("--eta_m", type=float, default=0.08)
    ap.add_argument("--extra_budget", type=int, default=32)
    ap.add_argument("--max_slots_per_bucket", type=int, default=4)
    ap.add_argument("--split_rounds", type=int, default=40)
    ap.add_argument("--min_split_gain", type=float, default=1e-4)

    ap.add_argument("--sector_mode", type=str, default="kmeans", choices=["kmeans", "phase2", "phase4d", "phase4d_adaptive", "phase4d_hopf", "phase4d_hopf_iso", "phase4d_hopf_ball", "phase4d_hopf_chi", "phase4d_hopf_fib", "phase4d_hopf_fib_rung", "phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound", "phase4d_hopf_blend", "phase4d_complex_local", "complex2"])
    ap.add_argument("--phase_dims", type=str, default="0,1")
    ap.add_argument("--phase4_dims", type=str, default="0,2,4,6")
    ap.add_argument("--complex_dims", type=str, default="0,1")
    ap.add_argument("--hybrid_local_k", type=int, default=4)
    ap.add_argument("--hybrid_complex_roots", type=int, default=4)
    ap.add_argument("--hybrid_local_min_k", type=int, default=1)
    ap.add_argument("--hybrid_local_target", type=float, default=0.60)
    ap.add_argument("--hybrid_local_hysteresis", type=float, default=0.05)
    ap.add_argument("--hybrid_local_converge_lambda", type=float, default=1.0)
    ap.add_argument("--adaptive_min_pair_bins", type=int, default=2)
    ap.add_argument("--adaptive_time_growth", type=float, default=1.0)
    ap.add_argument("--adaptive_balance", type=float, default=1.0)
    ap.add_argument("--adaptive_angle_growth", type=float, default=0.35)
    ap.add_argument("--adaptive_shell_growth", type=float, default=0.0)
    ap.add_argument("--adaptive_shell_balance", type=float, default=0.0)
    ap.add_argument("--adaptive_converge_lambda", type=float, default=0.0)
    ap.add_argument("--adaptive_converge_target", type=float, default=1.0)
    ap.add_argument("--adaptive_converge_hysteresis", type=float, default=0.1)
    ap.add_argument("--adaptive_converge_mode", type=str, default="fixed", choices=["fixed", "phi_ratio", "phi_ladder"])
    ap.add_argument("--shell_mode", type=str, default="linear", choices=["linear", "phi_log", "phi_phase"])
    ap.add_argument("--shell_phase_coupling", type=float, default=0.0)
    ap.add_argument("--fib_rung_gate_threshold", type=float, default=0.0)
    ap.add_argument("--route_scale_lambda", type=float, default=1.0)
    ap.add_argument("--memory_coord_mode", type=str, default="route_chart", choices=["route_chart", "full_chart"])
    ap.add_argument("--hopf_chi_bins", type=int, default=2)
    ap.add_argument("--hopf_blend_lambda", type=float, default=0.8)
    ap.add_argument("--hopf_blend_chi_weight", type=float, default=1.0)
    ap.add_argument("--hopf_blend_shell_weight", type=float, default=0.5)
    ap.add_argument("--time_pressure_lambda", type=float, default=0.0)
    ap.add_argument("--train_route_mode", type=str, default="final_static", choices=["dynamic", "final_static"])

    ap.add_argument("--learn_so8", type=int, default=0, choices=[0, 1])
    ap.add_argument("--learn_scale", type=int, default=1, choices=[0, 1])
    ap.add_argument("--scale_mode", type=str, default="radial", choices=["global", "radial"])
    ap.add_argument("--radial_bins", type=int, default=10)
    ap.add_argument("--radial_rmax", type=float, default=0.0)
    ap.add_argument("--radial_update_frac", type=float, default=0.25)
    ap.add_argument("--radial_l2", type=float, default=0.0)

    ap.add_argument("--chart_iters", type=int, default=40)
    ap.add_argument("--chart_alpha", type=float, default=0.01)
    ap.add_argument("--chart_beta", type=float, default=0.0)
    ap.add_argument("--so8_step", type=float, default=0.10)
    ap.add_argument("--so8_candidates", type=int, default=2)
    ap.add_argument("--scale_step", type=float, default=0.08)
    ap.add_argument("--scale_candidates", type=int, default=2)
    ap.add_argument("--scale_clip", type=float, default=2.0)
    ap.add_argument("--recluster_after_chart", type=int, default=1, choices=[0, 1])

    ap.add_argument("--fast_dev", type=int, default=0, choices=[0, 1])
    ap.add_argument("--early_stop_patience", type=int, default=0)
    ap.add_argument("--early_stop_min_delta", type=float, default=0.0)
    ap.add_argument("--cache_dir", type=str, default="results/cache")
    ap.add_argument("--cache_chart", type=int, default=0, choices=[0, 1])
    ap.add_argument("--cache_routes", type=int, default=0, choices=[0, 1])
    ap.add_argument("--run_tag", type=str, default="")
    return ap.parse_args()


def load_token_targets(tokens_input: str, proxy_meta_path: str, eval_split: str) -> Tuple[np.ndarray, np.ndarray]:
    with open(proxy_meta_path, "r", encoding="utf-8") as f:
        meta = json.load(f)
    context_len = int(meta["context_len"])
    n_train = int(meta["n_train"])
    n_eval = int(meta["n_val"] if eval_split == "val" else meta["n_test"])
    z = np.load(tokens_input)
    _, y_train_tok = contexts_targets(z["train_ids"], context_len, n_train)
    split_key = "valid_ids" if eval_split == "val" else "test_ids"
    _, y_eval_tok = contexts_targets(z[split_key], context_len, n_eval)
    return y_train_tok.astype(np.int32), y_eval_tok.astype(np.int32)


def key_stats(keys: Sequence[Tuple[int, ...]]) -> Tuple[float, float, float, int, int, int]:
    if not keys:
        return 0.0, 0.0, 0.0, 0, 0, 0
    sh = np.array([k[0] for k in keys], dtype=np.int64)
    se = np.array([k[1] for k in keys], dtype=np.int64)
    key_hash = sh * 1000003 + se
    _, counts = np.unique(key_hash, return_counts=True)
    pmax = float(np.max(counts) / np.sum(counts)) if len(counts) else 0.0
    entropy = hr.entropy_from_counts(counts) if len(counts) else 0.0
    shell_vals = np.unique(sh)
    sector_vals = np.unique(se)
    shell_counts = np.unique(sh, return_counts=True)[1]
    sector_counts = np.unique(se, return_counts=True)[1]
    shell_pmax = float(np.max(shell_counts) / np.sum(shell_counts)) if len(shell_counts) else 0.0
    sector_pmax = float(np.max(sector_counts) / np.sum(sector_counts)) if len(sector_counts) else 0.0
    return pmax, entropy, shell_pmax, int(len(counts)), int(len(shell_vals)), int(len(sector_vals))


def global_mean_mse(y_train: np.ndarray, y_eval: np.ndarray) -> float:
    mean_vec = np.mean(y_train, axis=0, keepdims=True)
    dif = y_eval - mean_vec
    return float(np.mean(dif * dif))


def vote_top1(token_ids: np.ndarray) -> int:
    vals, counts = np.unique(token_ids, return_counts=True)
    return int(vals[np.argmax(counts)])


def topk_reduce(
    sim: np.ndarray,
    cand_y: np.ndarray,
    cand_tok: np.ndarray,
    topk: int,
) -> Tuple[np.ndarray, np.ndarray]:
    kk = min(topk, cand_y.shape[0])
    idx = np.argpartition(-sim, kth=kk - 1, axis=1)[:, :kk]
    part = np.take_along_axis(sim, idx, axis=1)
    order = np.argsort(-part, axis=1)
    idx = np.take_along_axis(idx, order, axis=1)
    yhat = np.mean(cand_y[idx], axis=1)
    pred_tok = np.zeros((idx.shape[0],), dtype=np.int32)
    for j in range(idx.shape[0]):
        pred_tok[j] = vote_top1(cand_tok[idx[j]])
    return yhat, pred_tok


def dense_retrieval(
    train_z: np.ndarray,
    train_y: np.ndarray,
    train_tok: np.ndarray,
    eval_z: np.ndarray,
    topk: int,
    block_size: int = 256,
):
    tr_u = _normalize_rows(train_z)
    ev_u = _normalize_rows(eval_z)
    yhat = np.zeros((ev_u.shape[0], train_y.shape[1]), dtype=np.float64)
    pred_tok = np.zeros((ev_u.shape[0],), dtype=np.int32)
    for start in range(0, ev_u.shape[0], block_size):
        stop = min(start + block_size, ev_u.shape[0])
        sim = ev_u[start:stop] @ tr_u.T
        y_block, tok_block = topk_reduce(sim, train_y, train_tok, topk=topk)
        yhat[start:stop] = y_block
        pred_tok[start:stop] = tok_block
    candidate_count = float(train_z.shape[0])
    return yhat, pred_tok, candidate_count, candidate_count / max(1.0, float(train_z.shape[0]))


def build_bucket_index(keys: Sequence[Tuple[int, ...]]) -> Dict[Tuple[int, ...], np.ndarray]:
    bucket_to_idx: Dict[Tuple[int, ...], List[int]] = {}
    for i, key in enumerate(keys):
        bucket_to_idx.setdefault(key, []).append(i)
    return {k: np.array(v, dtype=np.int64) for k, v in bucket_to_idx.items()}


def complex_key_ids(field: np.ndarray, dim_i: int, dim_j: int, roots: int, radius_bins: int) -> np.ndarray:
    roots = max(1, int(roots))
    radius_bins = max(1, int(radius_bins))
    qi = field[:, dim_i]
    qj = field[:, dim_j]
    theta = np.mod(np.arctan2(qj, qi), 2.0 * math.pi)
    angle_ids = np.minimum((theta * (float(roots) / (2.0 * math.pi))).astype(np.int64), roots - 1)
    if radius_bins <= 1 or field.shape[0] == 0:
        radius_ids = np.zeros((field.shape[0],), dtype=np.int64)
    else:
        radius = np.sqrt(qi * qi + qj * qj)
        edges = np.quantile(radius, np.linspace(0.0, 1.0, radius_bins + 1)[1:-1])
        radius_ids = np.searchsorted(edges, radius, side="right").astype(np.int64)
    return angle_ids + roots * radius_ids


def augment_route_keys_with_complex(
    base_keys: Sequence[Tuple[int, int]],
    field: np.ndarray,
    dim_i: int,
    dim_j: int,
    roots: int,
    radius_bins: int,
) -> Tuple[List[Tuple[int, int, int]], int]:
    complex_ids = complex_key_ids(field, dim_i=dim_i, dim_j=dim_j, roots=roots, radius_bins=radius_bins)
    keys = [(int(key[0]), int(key[1]), int(complex_ids[i])) for i, key in enumerate(base_keys)]
    return keys, int(len(np.unique(complex_ids))) if complex_ids.size else 0


def primary_route_key(key: Tuple[int, ...]) -> Tuple[int, int]:
    return int(key[0]), int(key[1])


def routed_retrieval_grouped_same_bucket(
    train_keys: Sequence[Tuple[int, ...]],
    eval_keys: Sequence[Tuple[int, ...]],
    train_z: np.ndarray,
    train_y: np.ndarray,
    train_tok: np.ndarray,
    eval_z: np.ndarray,
    topk: int,
    complex_backfill_items: int = 0,
    complex_backfill_mode: str = "always",
    complex_backfill_max_exact: int = 0,
    complex_backfill_margin_threshold: float = 0.0,
) -> Tuple[np.ndarray, np.ndarray, float, float, float, float, float, float]:
    bucket_to_train_idx = build_bucket_index(train_keys)
    bucket_to_eval_idx = build_bucket_index(eval_keys)
    tr_u = _normalize_rows(train_z)
    ev_u = _normalize_rows(eval_z)
    yhat = np.zeros((ev_u.shape[0], train_y.shape[1]), dtype=np.float64)
    pred_tok = np.zeros((ev_u.shape[0],), dtype=np.int32)
    candidate_counts = np.zeros((ev_u.shape[0],), dtype=np.float64)
    fallback = np.zeros((ev_u.shape[0],), dtype=np.float64)
    backfill_trigger = np.zeros((ev_u.shape[0],), dtype=np.float64)
    backfill_added = np.zeros((ev_u.shape[0],), dtype=np.float64)

    full_idx = np.arange(train_z.shape[0], dtype=np.int64)
    use_backfill = int(complex_backfill_items) > 0 and bool(train_keys) and len(train_keys[0]) > 2
    primary_to_train_idx = build_bucket_index([primary_route_key(k) for k in train_keys]) if use_backfill else {}
    extra_pool_by_key: Dict[Tuple[int, ...], np.ndarray] = {}
    if use_backfill:
        for key, exact_idx in bucket_to_train_idx.items():
            if len(key) <= 2:
                continue
            base_idx = primary_to_train_idx.get(primary_route_key(key))
            if base_idx is None or base_idx.size == 0:
                extra_pool_by_key[key] = np.zeros((0,), dtype=np.int64)
            else:
                extra_pool_by_key[key] = np.setdiff1d(base_idx, exact_idx, assume_unique=False)
    for key, ev_idx in bucket_to_eval_idx.items():
        cand_idx = bucket_to_train_idx.get(key)
        if not use_backfill:
            if cand_idx is None or cand_idx.size == 0:
                cand_idx = full_idx
                fallback[ev_idx] = 1.0
            sim = ev_u[ev_idx] @ tr_u[cand_idx].T
            y_block, tok_block = topk_reduce(sim, train_y[cand_idx], train_tok[cand_idx], topk=topk)
            yhat[ev_idx] = y_block
            pred_tok[ev_idx] = tok_block
            candidate_counts[ev_idx] = float(cand_idx.shape[0])
            continue

        base_idx = primary_to_train_idx.get(primary_route_key(key))
        if base_idx is None or base_idx.size == 0:
            cand_idx_row = full_idx
            fallback[ev_idx] = 1.0
            sim = ev_u[ev_idx] @ tr_u[cand_idx_row].T
            y_block, tok_block = topk_reduce(sim, train_y[cand_idx_row], train_tok[cand_idx_row], topk=topk)
            yhat[ev_idx] = y_block
            pred_tok[ev_idx] = tok_block
            candidate_counts[ev_idx] = float(cand_idx_row.shape[0])
            continue
        if cand_idx is None or cand_idx.size == 0:
            fallback[ev_idx] = 1.0
            sim = ev_u[ev_idx] @ tr_u[base_idx].T
            y_block, tok_block = topk_reduce(sim, train_y[base_idx], train_tok[base_idx], topk=topk)
            yhat[ev_idx] = y_block
            pred_tok[ev_idx] = tok_block
            candidate_counts[ev_idx] = float(base_idx.shape[0])
            continue
        extra_pool = extra_pool_by_key.get(key)
        sim_exact = ev_u[ev_idx] @ tr_u[cand_idx].T
        y_block, tok_block = topk_reduce(sim_exact, train_y[cand_idx], train_tok[cand_idx], topk=topk)
        yhat[ev_idx] = y_block
        pred_tok[ev_idx] = tok_block
        candidate_counts[ev_idx] = float(cand_idx.shape[0])
        if extra_pool is None or extra_pool.size == 0:
            continue
        if complex_backfill_mode == "small_bucket":
            max_exact = max(1, int(complex_backfill_max_exact))
            trigger_mask = np.full((ev_idx.shape[0],), cand_idx.shape[0] <= max_exact, dtype=bool)
        elif complex_backfill_mode == "low_margin":
            if cand_idx.shape[0] <= 1:
                trigger_mask = np.ones((ev_idx.shape[0],), dtype=bool)
            else:
                top2 = min(2, cand_idx.shape[0])
                best2 = np.argpartition(-sim_exact, kth=top2 - 1, axis=1)[:, :top2]
                top2_vals = np.take_along_axis(sim_exact, best2, axis=1)
                top2_vals.sort(axis=1)
                margins = top2_vals[:, -1] - top2_vals[:, -2]
                trigger_mask = margins <= float(complex_backfill_margin_threshold)
        else:
            trigger_mask = np.ones((ev_idx.shape[0],), dtype=bool)
        if not np.any(trigger_mask):
            continue
        backfill_n = min(int(complex_backfill_items), int(extra_pool.size))
        chosen_eval_idx = ev_idx[trigger_mask]
        extra_sim = ev_u[chosen_eval_idx] @ tr_u[extra_pool].T
        extra_idx = np.argpartition(-extra_sim, kth=backfill_n - 1, axis=1)[:, :backfill_n]
        part = np.take_along_axis(extra_sim, extra_idx, axis=1)
        order = np.argsort(-part, axis=1)
        extra_idx = np.take_along_axis(extra_idx, order, axis=1)
        for local_row, row_idx in enumerate(chosen_eval_idx):
            cand_idx_row = np.unique(np.concatenate([cand_idx, extra_pool[extra_idx[local_row]]]))
            sim = ev_u[row_idx:row_idx + 1] @ tr_u[cand_idx_row].T
            y_block, tok_block = topk_reduce(sim, train_y[cand_idx_row], train_tok[cand_idx_row], topk=topk)
            yhat[row_idx:row_idx + 1] = y_block
            pred_tok[row_idx] = tok_block[0]
            candidate_counts[row_idx] = float(cand_idx_row.shape[0])
            backfill_trigger[row_idx] = 1.0
            backfill_added[row_idx] = float(cand_idx_row.shape[0] - cand_idx.shape[0])

    return (
        yhat,
        pred_tok,
        float(np.mean(candidate_counts)),
        float(np.mean(candidate_counts) / max(1.0, float(train_z.shape[0]))),
        1.0,
        float(np.mean(fallback)),
        float(np.mean(backfill_trigger)),
        float(np.mean(backfill_added)),
    )


def routed_retrieval(
    train_keys: Sequence[Tuple[int, ...]],
    eval_keys: Sequence[Tuple[int, ...]],
    train_z: np.ndarray,
    train_y: np.ndarray,
    train_tok: np.ndarray,
    eval_z: np.ndarray,
    topk: int,
    probe_buckets: int,
    complex_backfill_items: int = 0,
    complex_backfill_mode: str = "always",
    complex_backfill_max_exact: int = 0,
    complex_backfill_margin_threshold: float = 0.0,
):
    bucket_to_idx = build_bucket_index(train_keys)
    bucket_keys = list(bucket_to_idx.keys())
    if not bucket_keys:
        yhat, pred_tok, cand_mean, cand_frac = dense_retrieval(train_z, train_y, train_tok, eval_z, topk=topk)
        return yhat, pred_tok, cand_mean, cand_frac, 0.0, 1.0, 0.0, 0.0
    if probe_buckets <= 1:
        return routed_retrieval_grouped_same_bucket(
            train_keys,
            eval_keys,
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=topk,
            complex_backfill_items=complex_backfill_items,
            complex_backfill_mode=complex_backfill_mode,
            complex_backfill_max_exact=complex_backfill_max_exact,
            complex_backfill_margin_threshold=complex_backfill_margin_threshold,
        )

    tr_u = _normalize_rows(train_z)
    ev_u = _normalize_rows(eval_z)
    centroids = np.stack([np.mean(tr_u[idx], axis=0) for idx in bucket_to_idx.values()], axis=0)
    centroids_u = _normalize_rows(centroids)
    yhat = np.zeros((ev_u.shape[0], train_y.shape[1]), dtype=np.float64)
    pred_tok = np.zeros((ev_u.shape[0],), dtype=np.int32)
    candidate_counts = np.zeros((ev_u.shape[0],), dtype=np.float64)
    probe_counts = np.zeros((ev_u.shape[0],), dtype=np.float64)
    fallback = np.zeros((ev_u.shape[0],), dtype=np.float64)

    for i in range(ev_u.shape[0]):
        q = ev_u[i]
        if probe_buckets <= 1:
            selected = [eval_keys[i]] if eval_keys[i] in bucket_to_idx else []
        else:
            sim = centroids_u @ q
            kk = min(probe_buckets, len(bucket_keys))
            probe_idx = np.argpartition(-sim, kth=kk - 1)[:kk]
            probe_idx = probe_idx[np.argsort(-sim[probe_idx])]
            selected = [bucket_keys[j] for j in probe_idx]
            if eval_keys[i] in bucket_to_idx and eval_keys[i] not in selected:
                selected = [eval_keys[i]] + selected[:-1]
        cand_parts = [bucket_to_idx[k] for k in selected if k in bucket_to_idx]
        if not cand_parts:
            cand_idx = np.arange(train_z.shape[0], dtype=np.int64)
            fallback[i] = 1.0
        else:
            cand_idx = np.unique(np.concatenate(cand_parts))
        local_sim = tr_u[cand_idx] @ q
        kk = min(topk, cand_idx.shape[0])
        best = np.argpartition(-local_sim, kth=kk - 1)[:kk]
        best = best[np.argsort(-local_sim[best])]
        top_idx = cand_idx[best]
        yhat[i] = np.mean(train_y[top_idx], axis=0)
        pred_tok[i] = vote_top1(train_tok[top_idx])
        candidate_counts[i] = float(cand_idx.shape[0])
        probe_counts[i] = float(len(selected)) if selected else 1.0

    return (
        yhat,
        pred_tok,
        float(np.mean(candidate_counts)),
        float(np.mean(candidate_counts) / max(1.0, float(train_z.shape[0]))),
        float(np.mean(probe_counts)),
        float(np.mean(fallback)),
        0.0,
        0.0,
    )


def main():
    args = parse_args()
    t_total_start = time.perf_counter()
    timings = {
        "dataset": 0.0,
        "chart_opt": 0.0,
        "routing_eval": 0.0,
        "route_index_build": 0.0,
        "query_route": 0.0,
        "retrieval_search": 0.0,
        "offline_total": 0.0,
        "online_total": 0.0,
        "training_route": 0.0,
        "training_update": 0.0,
        "training_ema": 0.0,
        "growth": 0.0,
        "total": 0.0,
    }
    notes: List[str] = []
    artifacts = {
        "input": args.input,
        "tokens_input": args.tokens_input,
        "eval_split": args.eval_split,
        "chart_cache_file": "",
        "route_cache_file": "",
        "run_tag": args.run_tag,
    }

    apply_retrieval_fast_dev(args)
    args.query_repeats = max(1, int(args.query_repeats))

    t0 = time.perf_counter()
    data = np.load(args.input)
    x_train = data["x_train"].astype(np.float64)
    y_train = data["y_train"].astype(np.float64)
    if args.eval_split == "val":
        x_eval = data["x_val"].astype(np.float64)
        y_eval = data["y_val"].astype(np.float64)
    else:
        x_eval = data["x_test"].astype(np.float64)
        y_eval = data["y_test"].astype(np.float64)
    y_train_tok_all, y_eval_tok_all = load_token_targets(args.tokens_input, args.proxy_meta, args.eval_split)
    timings["dataset"] = time.perf_counter() - t0

    v_tr, y_tr, y_tr_tok, _ = _subset_with_index(x_train, y_train, y_train_tok_all, args.max_train, args.seed + 1)
    v_ev, y_ev, y_ev_tok, _ = _subset_with_index(x_eval, y_eval, y_eval_tok_all, args.max_eval, args.seed + 2)

    d = int(v_tr.shape[1])
    dy = int(y_tr.shape[1])
    phase_dim_i, phase_dim_j = hr.parse_pair_dims(args.phase_dims, "--phase_dims")
    phase4_dim_i, phase4_dim_j, phase4_dim_k, phase4_dim_l = hr.parse_quad_dims(args.phase4_dims, "--phase4_dims")
    complex_dim_i, complex_dim_j = hr.parse_pair_dims(args.complex_dims, "--complex_dims")
    hr.ensure_dims_in_range([phase_dim_i, phase_dim_j], d, "--phase_dims")
    hr.ensure_dims_in_range([phase4_dim_i, phase4_dim_j, phase4_dim_k, phase4_dim_l], d, "--phase4_dims")
    hr.ensure_dims_in_range([complex_dim_i, complex_dim_j], d, "--complex_dims")

    train_label_sse_per = global_mean_mse(y_tr, y_tr)
    eval_label_sse_per = global_mean_mse(y_tr, y_ev)
    mse_before = eval_label_sse_per

    chart = hr.Chart(R=np.eye(d, dtype=np.float64), s_global=None, S_radial=None, scale_mode="global")
    C_used = None
    z_tr = v_tr
    z_ev = v_ev
    keys_tr: List[Tuple[int, ...]] = []
    keys_ev: List[Tuple[int, ...]] = []

    yhat = np.zeros((v_ev.shape[0], dy), dtype=np.float64)
    pred_tok = np.zeros((v_ev.shape[0],), dtype=np.int32)
    candidate_count_mean = 0.0
    candidate_fraction_mean = 1.0
    probe_bucket_mean = 0.0
    bucket_fallback_rate = 0.0
    backfill_trigger_rate = 0.0
    backfill_extra_candidates_mean = 0.0
    secondary_key_count = 0

    if args.retrieval_backend == "routed_probe":
        if args.sector_mode == "kmeans":
            U0 = hr.normalize_rows(v_tr)
            C_used = hr.spherical_kmeans(U0, K=args.K, iters=args.kmeans_iters, seed=args.seed + 11)

        learned_chart = (args.learn_so8 == 1 or args.learn_scale == 1)
        if learned_chart:
            t_chart = time.perf_counter()
            opt_res = hr.optimize_chart(
                v_train=v_tr,
                y_train=y_tr,
                delta_r=args.delta_r,
                C0=C_used,
                learn_so8=args.learn_so8,
                learn_scale=args.learn_scale,
                scale_mode=args.scale_mode,
                radial_bins=args.radial_bins,
                radial_rmax=args.radial_rmax,
                radial_update_frac=args.radial_update_frac,
                radial_l2=args.radial_l2,
                iters=args.chart_iters,
                so8_step=args.so8_step,
                so8_candidates=args.so8_candidates,
                scale_step=args.scale_step,
                scale_candidates=args.scale_candidates,
                scale_clip=args.scale_clip,
                alpha_overload=args.chart_alpha,
                beta_bucketcount=args.chart_beta,
                sector_mode=args.sector_mode,
                phase_dim_i=phase_dim_i,
                phase_dim_j=phase_dim_j,
                phase4_dim_i=phase4_dim_i,
                phase4_dim_j=phase4_dim_j,
                phase4_dim_k=phase4_dim_k,
                phase4_dim_l=phase4_dim_l,
                complex_dim_i=complex_dim_i,
                complex_dim_j=complex_dim_j,
                K=args.K,
                seed=args.seed,
                early_stop_patience=args.early_stop_patience,
                early_stop_min_delta=args.early_stop_min_delta,
                adaptive_min_pair_bins=args.adaptive_min_pair_bins,
                adaptive_time_growth=args.adaptive_time_growth,
                adaptive_balance=args.adaptive_balance,
                adaptive_angle_growth=args.adaptive_angle_growth,
                adaptive_shell_growth=args.adaptive_shell_growth,
                adaptive_shell_balance=args.adaptive_shell_balance,
                adaptive_converge_lambda=args.adaptive_converge_lambda,
                adaptive_converge_target=args.adaptive_converge_target,
                adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
                adaptive_converge_mode=args.adaptive_converge_mode,
                fib_rung_gate_threshold=args.fib_rung_gate_threshold,
                route_scale_lambda=args.route_scale_lambda,
                memory_coord_mode=args.memory_coord_mode,
                shell_mode=args.shell_mode,
                shell_phase_coupling=args.shell_phase_coupling,
                hopf_chi_bins=args.hopf_chi_bins,
                hopf_blend_lambda=args.hopf_blend_lambda,
                hopf_blend_chi_weight=args.hopf_blend_chi_weight,
                hopf_blend_shell_weight=args.hopf_blend_shell_weight,
                hybrid_local_k=args.hybrid_local_k,
                hybrid_complex_roots=args.hybrid_complex_roots,
                hybrid_local_min_k=args.hybrid_local_min_k,
                hybrid_local_target=args.hybrid_local_target,
                hybrid_local_hysteresis=args.hybrid_local_hysteresis,
                hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
            )
            chart = opt_res.chart
            timings["chart_opt"] = time.perf_counter() - t_chart
            notes.append(f"retrieval chart iters={len(opt_res.loss_hist) - 1}")
        t_route_tr = time.perf_counter()
        shell_tr, sector_tr, _, z_tr = hr.route_addresses(
            v_tr, delta_r=args.delta_r, C=C_used, chart=chart,
            sector_mode=args.sector_mode,
            phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
            phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
            phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
            complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
            K=args.K,
            time_pressure_lambda=args.time_pressure_lambda, tau=1.0,
            adaptive_min_pair_bins=args.adaptive_min_pair_bins,
            adaptive_time_growth=args.adaptive_time_growth,
            adaptive_balance=args.adaptive_balance,
            adaptive_angle_growth=args.adaptive_angle_growth,
            adaptive_shell_growth=args.adaptive_shell_growth,
            adaptive_shell_balance=args.adaptive_shell_balance,
            adaptive_converge_lambda=args.adaptive_converge_lambda,
            adaptive_converge_target=args.adaptive_converge_target,
            adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
            adaptive_converge_mode=args.adaptive_converge_mode,
            fib_rung_gate_threshold=args.fib_rung_gate_threshold,
            route_scale_lambda=args.route_scale_lambda,
            memory_coord_mode=args.memory_coord_mode,
            shell_mode=args.shell_mode,
            shell_phase_coupling=args.shell_phase_coupling,
            hopf_chi_bins=args.hopf_chi_bins,
            hopf_blend_lambda=args.hopf_blend_lambda,
            hopf_blend_chi_weight=args.hopf_blend_chi_weight,
            hopf_blend_shell_weight=args.hopf_blend_shell_weight,
            hybrid_local_k=args.hybrid_local_k,
            hybrid_complex_roots=args.hybrid_complex_roots,
            hybrid_local_min_k=args.hybrid_local_min_k,
            hybrid_local_target=args.hybrid_local_target,
            hybrid_local_hysteresis=args.hybrid_local_hysteresis,
            hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
        )
        timings["route_index_build"] = time.perf_counter() - t_route_tr
        keys_tr = [hr.make_bucket_key(int(shell_tr[i]), int(sector_tr[i])) for i in range(shell_tr.shape[0])]
        if args.route_key_mode == "hopf_plus_complex":
            keys_tr, secondary_key_count_tr = augment_route_keys_with_complex(
                base_keys=keys_tr,
                field=z_tr,
                dim_i=complex_dim_i,
                dim_j=complex_dim_j,
                roots=args.complex_key_roots,
                radius_bins=args.complex_key_radius_bins,
            )
            secondary_key_count = max(secondary_key_count, int(secondary_key_count_tr))
        train_key_set = set(keys_tr)

        def run_routed_query_pass():
            t_route_ev = time.perf_counter()
            shell_ev, sector_ev, _, z_ev_local = hr.route_addresses(
                v_ev, delta_r=args.delta_r, C=C_used, chart=chart,
                sector_mode=args.sector_mode,
                phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
                phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
                phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
                complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
                K=args.K,
                time_pressure_lambda=args.time_pressure_lambda, tau=1.0,
                adaptive_min_pair_bins=args.adaptive_min_pair_bins,
                adaptive_time_growth=args.adaptive_time_growth,
                adaptive_balance=args.adaptive_balance,
                adaptive_angle_growth=args.adaptive_angle_growth,
                adaptive_shell_growth=args.adaptive_shell_growth,
                adaptive_shell_balance=args.adaptive_shell_balance,
                adaptive_converge_lambda=args.adaptive_converge_lambda,
                adaptive_converge_target=args.adaptive_converge_target,
                adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
                adaptive_converge_mode=args.adaptive_converge_mode,
                fib_rung_gate_threshold=args.fib_rung_gate_threshold,
                route_scale_lambda=args.route_scale_lambda,
                memory_coord_mode=args.memory_coord_mode,
                shell_mode=args.shell_mode,
                shell_phase_coupling=args.shell_phase_coupling,
                hopf_chi_bins=args.hopf_chi_bins,
                hopf_blend_lambda=args.hopf_blend_lambda,
                hopf_blend_chi_weight=args.hopf_blend_chi_weight,
                hopf_blend_shell_weight=args.hopf_blend_shell_weight,
                hybrid_local_k=args.hybrid_local_k,
                hybrid_complex_roots=args.hybrid_complex_roots,
                hybrid_local_min_k=args.hybrid_local_min_k,
                hybrid_local_target=args.hybrid_local_target,
                hybrid_local_hysteresis=args.hybrid_local_hysteresis,
                hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
            )
            query_route_sec = time.perf_counter() - t_route_ev
            keys_ev_local = [hr.make_bucket_key(int(shell_ev[i]), int(sector_ev[i])) for i in range(shell_ev.shape[0])]
            secondary_key_count_local = 0
            if args.route_key_mode == "hopf_plus_complex":
                keys_ev_local, secondary_key_count_local = augment_route_keys_with_complex(
                    base_keys=keys_ev_local,
                    field=z_ev_local,
                    dim_i=complex_dim_i,
                    dim_j=complex_dim_j,
                    roots=args.complex_key_roots,
                    radius_bins=args.complex_key_radius_bins,
                )
            t_retr = time.perf_counter()
            (
                yhat_local,
                pred_tok_local,
                candidate_count_local,
                candidate_fraction_local,
                probe_bucket_local,
                fallback_local,
                backfill_trigger_local,
                backfill_added_local,
            ) = routed_retrieval(
                keys_tr, keys_ev_local, z_tr, y_tr, y_tr_tok, z_ev_local,
                topk=args.topk,
                probe_buckets=args.probe_buckets,
                complex_backfill_items=args.complex_backfill_items,
                complex_backfill_mode=args.complex_backfill_mode,
                complex_backfill_max_exact=args.complex_backfill_max_exact,
                complex_backfill_margin_threshold=args.complex_backfill_margin_threshold,
            )
            retrieval_search_sec = time.perf_counter() - t_retr
            return {
                "shell_ev": shell_ev,
                "sector_ev": sector_ev,
                "z_ev": z_ev_local,
                "keys_ev": keys_ev_local,
                "query_route_sec": query_route_sec,
                "retrieval_search_sec": retrieval_search_sec,
                "yhat": yhat_local,
                "pred_tok": pred_tok_local,
                "candidate_count_mean": candidate_count_local,
                "candidate_fraction_mean": candidate_fraction_local,
                "probe_bucket_mean": probe_bucket_local,
                "bucket_fallback_rate": fallback_local,
                "backfill_trigger_rate": backfill_trigger_local,
                "backfill_extra_candidates_mean": backfill_added_local,
                "secondary_key_count": secondary_key_count_local,
            }

        first_pass = run_routed_query_pass()
        z_ev = first_pass["z_ev"]
        keys_ev = first_pass["keys_ev"]
        yhat = first_pass["yhat"]
        pred_tok = first_pass["pred_tok"]
        candidate_count_mean = float(first_pass["candidate_count_mean"])
        candidate_fraction_mean = float(first_pass["candidate_fraction_mean"])
        probe_bucket_mean = float(first_pass["probe_bucket_mean"])
        bucket_fallback_rate = float(first_pass["bucket_fallback_rate"])
        backfill_trigger_rate = float(first_pass["backfill_trigger_rate"])
        backfill_extra_candidates_mean = float(first_pass["backfill_extra_candidates_mean"])
        secondary_key_count = max(int(secondary_key_count), int(first_pass["secondary_key_count"]))
        timings["query_route"] += float(first_pass["query_route_sec"])
        timings["retrieval_search"] += float(first_pass["retrieval_search_sec"])

        unseen_rate = hr.unseen_key_rate(keys_ev, train_key_set)
        pmax_after, entropy_after, shell_pmax, buckets, eval_shells, eval_sectors = key_stats(keys_ev)
        sector_pmax = 0.0
        if keys_ev:
            sectors = np.array([k[1] for k in keys_ev], dtype=np.int64)
            _, sector_counts = np.unique(sectors, return_counts=True)
            sector_pmax = float(np.max(sector_counts) / np.sum(sector_counts)) if len(sector_counts) else 0.0
        train_label_sse = hr.label_coherence_sse(
            v_tr, y_tr, delta_r=args.delta_r, C=C_used, chart=chart,
            sector_mode=args.sector_mode,
            phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
            phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
            phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
            complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
            K=args.K,
            adaptive_min_pair_bins=args.adaptive_min_pair_bins,
            adaptive_time_growth=args.adaptive_time_growth,
            adaptive_balance=args.adaptive_balance,
            adaptive_angle_growth=args.adaptive_angle_growth,
            adaptive_shell_growth=args.adaptive_shell_growth,
            adaptive_shell_balance=args.adaptive_shell_balance,
            adaptive_converge_lambda=args.adaptive_converge_lambda,
            adaptive_converge_target=args.adaptive_converge_target,
            adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
            adaptive_converge_mode=args.adaptive_converge_mode,
            fib_rung_gate_threshold=args.fib_rung_gate_threshold,
            route_scale_lambda=args.route_scale_lambda,
            memory_coord_mode=args.memory_coord_mode,
            shell_mode=args.shell_mode,
            shell_phase_coupling=args.shell_phase_coupling,
            hopf_chi_bins=args.hopf_chi_bins,
            hopf_blend_lambda=args.hopf_blend_lambda,
            hopf_blend_chi_weight=args.hopf_blend_chi_weight,
            hopf_blend_shell_weight=args.hopf_blend_shell_weight,
            hybrid_local_k=args.hybrid_local_k,
            hybrid_complex_roots=args.hybrid_complex_roots,
            hybrid_local_min_k=args.hybrid_local_min_k,
            hybrid_local_target=args.hybrid_local_target,
            hybrid_local_hysteresis=args.hybrid_local_hysteresis,
            hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
        )
        eval_label_sse = hr.label_coherence_sse(
            v_ev, y_ev, delta_r=args.delta_r, C=C_used, chart=chart,
            sector_mode=args.sector_mode,
            phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
            phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
            phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
            complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
            K=args.K,
            adaptive_min_pair_bins=args.adaptive_min_pair_bins,
            adaptive_time_growth=args.adaptive_time_growth,
            adaptive_balance=args.adaptive_balance,
            adaptive_angle_growth=args.adaptive_angle_growth,
            adaptive_shell_growth=args.adaptive_shell_growth,
            adaptive_shell_balance=args.adaptive_shell_balance,
            adaptive_converge_lambda=args.adaptive_converge_lambda,
            adaptive_converge_target=args.adaptive_converge_target,
            adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
            adaptive_converge_mode=args.adaptive_converge_mode,
            fib_rung_gate_threshold=args.fib_rung_gate_threshold,
            route_scale_lambda=args.route_scale_lambda,
            memory_coord_mode=args.memory_coord_mode,
            shell_mode=args.shell_mode,
            shell_phase_coupling=args.shell_phase_coupling,
            hopf_chi_bins=args.hopf_chi_bins,
            hopf_blend_lambda=args.hopf_blend_lambda,
            hopf_blend_chi_weight=args.hopf_blend_chi_weight,
            hopf_blend_shell_weight=args.hopf_blend_shell_weight,
            hybrid_local_k=args.hybrid_local_k,
            hybrid_complex_roots=args.hybrid_complex_roots,
            hybrid_local_min_k=args.hybrid_local_min_k,
            hybrid_local_target=args.hybrid_local_target,
            hybrid_local_hysteresis=args.hybrid_local_hysteresis,
            hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
        )
        train_label_sse_per = float(train_label_sse / max(1, v_tr.shape[0]))
        eval_label_sse_per = float(eval_label_sse / max(1, v_ev.shape[0]))
        alignment = hr.poincare_alignment_diagnostics(v_ev, hr.route_coordinate(v_ev, chart, args.sector_mode, args.route_scale_lambda))

        for _ in range(1, args.query_repeats):
            repeat_pass = run_routed_query_pass()
            timings["query_route"] += float(repeat_pass["query_route_sec"])
            timings["retrieval_search"] += float(repeat_pass["retrieval_search_sec"])
            bucket_fallback_rate = float(max(bucket_fallback_rate, float(repeat_pass["bucket_fallback_rate"])))
            backfill_trigger_rate = float(max(backfill_trigger_rate, float(repeat_pass["backfill_trigger_rate"])))
            backfill_extra_candidates_mean = float(max(backfill_extra_candidates_mean, float(repeat_pass["backfill_extra_candidates_mean"])))
            secondary_key_count = max(int(secondary_key_count), int(repeat_pass["secondary_key_count"]))
    else:
        unseen_rate = 0.0
        pmax_after = 1.0
        entropy_after = 0.0
        shell_pmax = 1.0
        sector_pmax = 1.0
        buckets = 1
        eval_shells = 1
        eval_sectors = 1
        alignment = hr.poincare_alignment_diagnostics(v_ev, v_ev)
        notes.append("dense exact retrieval baseline")
        backfill_trigger_rate = 0.0
        backfill_extra_candidates_mean = 0.0
        for _ in range(args.query_repeats):
            t_retr = time.perf_counter()
            yhat, pred_tok, candidate_count_mean, candidate_fraction_mean = dense_retrieval(
                z_tr, y_tr, y_tr_tok, z_ev, topk=args.topk
            )
            timings["retrieval_search"] += time.perf_counter() - t_retr

    timings["routing_eval"] = timings["route_index_build"] + timings["query_route"] + timings["retrieval_search"]
    timings["offline_total"] = timings["chart_opt"] + timings["route_index_build"]
    timings["online_total"] = timings["query_route"] + timings["retrieval_search"]
    online_per_repeat_sec, amortized_per_repeat_sec = compute_amortized_retrieval_metrics(
        timings["offline_total"], timings["online_total"], args.query_repeats
    )

    mse_after = float(np.mean((yhat - y_ev) ** 2))
    top1_after = float(np.mean(pred_tok == y_ev_tok)) if y_ev_tok.size else 0.0
    timings["total"] = time.perf_counter() - t_total_start
    notes.append(f"query_repeats={args.query_repeats}")

    summary = {
        "schema_version": "1.0",
        "parsed": True,
        "args": {k: v for k, v in vars(args).items()},
        "metrics": {
            "test_mse_before": float(mse_before),
            "test_mse_after": float(mse_after),
            "train_label_sse_per": float(train_label_sse_per),
            "test_label_sse_per": float(eval_label_sse_per),
            "buckets": int(buckets),
            "slots_used": int(buckets),
            "test_unseen_rate": float(unseen_rate),
            "pmax_before": 1.0,
            "pmax_after": float(pmax_after),
            "entropy_before": 0.0,
            "entropy_after": float(entropy_after),
            "eval_shells": int(eval_shells),
            "eval_sectors": int(eval_sectors),
            "shell_pmax": float(shell_pmax),
            "sector_pmax": float(sector_pmax),
            "shell_entropy": 0.0,
            "sector_entropy": 0.0,
            "poincare_alignment_pairs_used": int(alignment["poincare_alignment_pairs_used"]),
            "poincare_alignment_radial_mae": float(alignment["poincare_alignment_radial_mae"]),
            "poincare_alignment_radial_rel_mean": float(alignment["poincare_alignment_radial_rel_mean"]),
            "poincare_alignment_radial_corr": float(alignment["poincare_alignment_radial_corr"]),
            "poincare_alignment_pair_mae": float(alignment["poincare_alignment_pair_mae"]),
            "poincare_alignment_pair_rel_mean": float(alignment["poincare_alignment_pair_rel_mean"]),
            "poincare_alignment_pair_corr": float(alignment["poincare_alignment_pair_corr"]),
            "new_slots": 0,
            "accepted_splits": 0,
            "n_buckets_total": int(buckets),
            "test_top1_after": float(top1_after),
            "retrieval_candidate_count_mean": float(candidate_count_mean),
            "retrieval_candidate_fraction_mean": float(candidate_fraction_mean),
            "retrieval_probe_bucket_mean": float(probe_bucket_mean),
            "retrieval_bucket_fallback_rate": float(bucket_fallback_rate),
            "retrieval_backfill_trigger_rate": float(backfill_trigger_rate),
            "retrieval_backfill_extra_candidates_mean": float(backfill_extra_candidates_mean),
            "retrieval_secondary_key_count": int(secondary_key_count),
            "retrieval_train_items": int(v_tr.shape[0]),
            "retrieval_eval_items": int(v_ev.shape[0]),
            "retrieval_query_repeats": int(args.query_repeats),
            "retrieval_offline_total_sec": float(timings["offline_total"]),
            "retrieval_online_total_sec": float(timings["online_total"]),
            "retrieval_online_total_per_repeat_sec": float(online_per_repeat_sec),
            "retrieval_total_amortized_per_repeat_sec": float(amortized_per_repeat_sec),
        },
        "timings_sec": {k: float(v) for k, v in timings.items()},
        "artifacts": artifacts,
        "git": hr.maybe_git_info(),
        "notes": notes,
    }
    print("__JSON_SUMMARY__ " + json.dumps(summary, sort_keys=True))


if __name__ == "__main__":
    main()
