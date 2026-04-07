#!/usr/bin/env python3
"""Canonical spin_H core v4 with composition-aware sigma mode carrier."""

from __future__ import annotations

import csv
from collections import Counter, defaultdict
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path
import sys

from geometry_native_operator_model_v10 import (
    composite_swap_component_v10,
    composite_twist_component_v10,
    coupled_torus_kick_component_v10,
    fiber_phase_lift_component_v10,
    hold_component_v10,
    torus_base_advance_component_v10,
)
from geometry_native_operator_model_v12 import (
    bounded_operator_surface_v8,
    radial_direction_v12,
    radial_phi_transport_v12,
    radial_spin_transport_map_v12,
    radial_tau_transport_map_v12,
    radial_transport_component_v12,
)
from geometry_native_spinH_candidate_v3 import NativeTauV3, PrimaryChartStateSpinHCandidateV3
from geometry_native_spinH_core_v1 import (
    HCoreV1,
    RhoCoreV1,
    ThetaCoreV1,
    _tau_from_tuple,
    _tau_str,
    _tau_tuple,
)
from geometry_native_spinH_core_v3 import (
    OUTPUT_PATH_SPINH_CORE_V3,
    _tau_from_state_tuple,
)


OUTPUT_PATH_SPINH_CORE_V4 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_sigma_mode_carrier_v1.csv"
)


COMPONENTS_V4 = (
    "hold",
    "torus_base_advance",
    "composite_swap",
    "coupled_torus_kick",
    "composite_twist",
    "fiber_phase_lift_spin_transport",
    "radial_transport_unfolding",
)


@dataclass(frozen=True)
class RegressiveModeCarrierV4:
    seed_orbit: tuple[tuple[int, ...], ...]
    generator_projection_profile: tuple[tuple[str, tuple[int, ...]], ...]
    generator_orbit_profile: tuple[tuple[str, tuple[tuple[int, ...], ...]], ...]
    generator_composition_profile: tuple[tuple[str, tuple[tuple[str, tuple[int, ...]], ...]], ...]


@dataclass(frozen=True)
class SigmaCoreV4:
    mode_carrier: RegressiveModeCarrierV4
    current_residue: int
    fiber_residue: int
    radial_residue: int


@dataclass(frozen=True)
class SpinHCoreV4:
    theta: ThetaCoreV1
    rho: RhoCoreV1
    sigma: SigmaCoreV4
    h: HCoreV1


def _word_str(word: tuple[int, ...]) -> str:
    return "".join(str(int(bit)) for bit in word)


def primary_chart_of_core_v4(state: object) -> PrimaryChartStateSpinHCandidateV3:
    return PrimaryChartStateSpinHCandidateV3(b=int(state.b), phi=int(state.phi), r=int(state.r))


@lru_cache(maxsize=None)
def _radial_targets_from_state_v4(state: object) -> tuple[int, int]:
    direction = radial_direction_v12(state)
    target_r = (state.r + direction) % 3
    target_phi = radial_phi_transport_v12(state, target_r=target_r, direction=direction)
    return target_r, target_phi


@lru_cache(maxsize=None)
def _local_sigma_words_v4(state: object) -> tuple[tuple[int, ...], tuple[int, ...], tuple[int, ...]]:
    fiber_state = fiber_phase_lift_component_v10(state)
    target_r, target_phi = _radial_targets_from_state_v4(state)
    radial_spin = radial_spin_transport_map_v12(
        state.spin_h,
        source_r=state.r,
        target_r=target_r,
        target_phi=target_phi,
        direction=radial_direction_v12(state),
        tau=state.tau,
        composite_compat_class=state.composite_compat_class,
    )
    current_word = tuple(int(bit) for bit in state.spin_h.bits)
    fiber_word = tuple(int(bit) for bit in fiber_state.spin_h.bits)
    radial_word = tuple(int(bit) for bit in radial_spin.bits)
    return current_word, fiber_word, radial_word


def _canonical_orbit_and_residues_v4(
    current_word: tuple[int, ...],
    fiber_word: tuple[int, ...],
    radial_word: tuple[int, ...],
) -> tuple[tuple[tuple[int, ...], ...], int, int, int]:
    orbit = tuple(sorted({tuple(current_word), tuple(fiber_word), tuple(radial_word)}))
    index = {word: idx for idx, word in enumerate(orbit)}
    return orbit, index[tuple(current_word)], index[tuple(fiber_word)], index[tuple(radial_word)]


@lru_cache(maxsize=None)
def _successor_by_component_v4(state: object, component: str) -> object:
    if component == "hold":
        return hold_component_v10(state)
    if component == "torus_base_advance":
        return torus_base_advance_component_v10(state)
    if component == "composite_swap":
        return composite_swap_component_v10(state)
    if component == "coupled_torus_kick":
        return coupled_torus_kick_component_v10(state)
    if component == "composite_twist":
        return composite_twist_component_v10(state)
    if component == "fiber_phase_lift_spin_transport":
        return fiber_phase_lift_component_v10(state)
    if component == "radial_transport_unfolding":
        return radial_transport_component_v12(state)
    raise ValueError(f"unknown component {component!r}")


@lru_cache(maxsize=None)
def _regressive_mode_carrier_v4(state: object) -> RegressiveModeCarrierV4:
    current_word, fiber_word, radial_word = _local_sigma_words_v4(state)
    seed_orbit, _, _, _ = _canonical_orbit_and_residues_v4(current_word, fiber_word, radial_word)

    projection_profile: list[tuple[str, tuple[int, ...]]] = []
    orbit_profile: list[tuple[str, tuple[tuple[int, ...], ...]]] = []
    composition_profile: list[tuple[str, tuple[tuple[str, tuple[int, ...]], ...]]] = []

    for first in COMPONENTS_V4:
        first_successor = _successor_by_component_v4(state, first)
        succ_current, succ_fiber, succ_radial = _local_sigma_words_v4(first_successor)
        succ_orbit, _, _, _ = _canonical_orbit_and_residues_v4(succ_current, succ_fiber, succ_radial)
        projection_profile.append((first, succ_current))
        orbit_profile.append((first, succ_orbit))

        second_profile: list[tuple[str, tuple[int, ...]]] = []
        for second in COMPONENTS_V4:
            composed_successor = _successor_by_component_v4(first_successor, second)
            comp_current, _, _ = _local_sigma_words_v4(composed_successor)
            second_profile.append((second, comp_current))
        composition_profile.append((first, tuple(second_profile)))

    return RegressiveModeCarrierV4(
        seed_orbit=seed_orbit,
        generator_projection_profile=tuple(projection_profile),
        generator_orbit_profile=tuple(orbit_profile),
        generator_composition_profile=tuple(composition_profile),
    )


def derive_mode_orbit_v4(carrier: RegressiveModeCarrierV4) -> tuple[tuple[int, ...], ...]:
    return carrier.seed_orbit


@lru_cache(maxsize=None)
def active_transport_lift_core_v4(state: object) -> SpinHCoreV4:
    direction = radial_direction_v12(state)
    target_r = (state.r + direction) % 3
    target_phi = radial_phi_transport_v12(state, target_r=target_r, direction=direction)

    fiber_state = fiber_phase_lift_component_v10(state)
    radial_spin = radial_spin_transport_map_v12(
        state.spin_h,
        source_r=state.r,
        target_r=target_r,
        target_phi=target_phi,
        direction=direction,
        tau=state.tau,
        composite_compat_class=state.composite_compat_class,
    )
    radial_tau = radial_tau_transport_map_v12(
        state.tau,
        source_r=state.r,
        target_r=target_r,
        target_phi=target_phi,
        spin_h=state.spin_h,
        direction=direction,
    )

    current_word = tuple(int(bit) for bit in state.spin_h.bits)
    fiber_word = tuple(int(bit) for bit in fiber_state.spin_h.bits)
    radial_word = tuple(int(bit) for bit in radial_spin.bits)

    carrier = _regressive_mode_carrier_v4(state)
    mode_orbit, current_residue, fiber_residue, radial_residue = _canonical_orbit_and_residues_v4(
        current_word=current_word,
        fiber_word=fiber_word,
        radial_word=radial_word,
    )
    if mode_orbit != derive_mode_orbit_v4(carrier):
        carrier = RegressiveModeCarrierV4(
            seed_orbit=mode_orbit,
            generator_projection_profile=carrier.generator_projection_profile,
            generator_orbit_profile=carrier.generator_orbit_profile,
            generator_composition_profile=carrier.generator_composition_profile,
        )

    return SpinHCoreV4(
        theta=ThetaCoreV1(
            base_angle=int(state.b),
            fiber_phase=int(state.phi),
        ),
        rho=RhoCoreV1(
            radial_class=int(state.r),
            unfolding_load=sum(int(bit) for bit in state.spin_h.bits),
            radial_direction=int(direction),
            radial_target=int(target_r),
            radial_target_phi=int(target_phi),
        ),
        sigma=SigmaCoreV4(
            mode_carrier=carrier,
            current_residue=current_residue,
            fiber_residue=fiber_residue,
            radial_residue=radial_residue,
        ),
        h=HCoreV1(
            recursive_phase=_tau_tuple(state.tau),
            fiber_recursive_phase=_tau_tuple(fiber_state.tau),
            radial_recursive_phase=_tau_tuple(radial_tau),
            holonomy_bit=int(state.twist),
        ),
    )


def project_spin_h4_v4(core: SpinHCoreV4) -> tuple[int, ...]:
    return derive_mode_orbit_v4(core.sigma.mode_carrier)[core.sigma.current_residue]


def project_tau_v4(core: SpinHCoreV4) -> NativeTauV3:
    return _tau_from_state_tuple(core.h.recursive_phase)


def project_kappa_v4(core: SpinHCoreV4) -> int:
    return int(core.h.holonomy_bit)


def component_update_signature_v4(source: object, target: object) -> tuple[int, int, int, int]:
    source_core = active_transport_lift_core_v4(source)
    target_core = active_transport_lift_core_v4(target)
    return (
        int(source_core.theta != target_core.theta),
        int(source_core.rho != target_core.rho),
        int(source_core.sigma != target_core.sigma),
        int(source_core.h != target_core.h),
    )


def _summary_metrics_from_v3(path: Path = OUTPUT_PATH_SPINH_CORE_V3) -> dict[str, float]:
    csv.field_size_limit(min(sys.maxsize, 10**9))
    rows = list(csv.DictReader(path.open("r", encoding="utf-8")))
    out: dict[str, float] = {}
    for row in rows:
        if row["scope"] in {"summary", "projection", "comparison_vs_v2"}:
            out[f"{row['metric']}__count"] = float(row["count"])
            out[f"{row['metric']}__fraction"] = float(row["fraction"])
    return out


def summarize_spinH_core_v4(depth: int = 8) -> list[dict[str, object]]:
    states, transitions = bounded_operator_surface_v8(depth=depth)

    primary_to_core: dict[PrimaryChartStateSpinHCandidateV3, set[SpinHCoreV4]] = defaultdict(set)
    core_to_primary: dict[SpinHCoreV4, set[PrimaryChartStateSpinHCandidateV3]] = defaultdict(set)
    core_transition_map: dict[SpinHCoreV4, dict[str, set[SpinHCoreV4]]] = defaultdict(lambda: defaultdict(set))

    sigma_depends_on_local_orbits = False
    less_bounded_than_v3 = True
    spin_h4_derivable = True
    tau_derivable = True
    kappa_derivable = True

    for state in states:
        primary = primary_chart_of_core_v4(state)
        core = active_transport_lift_core_v4(state)
        primary_to_core[primary].add(core)
        core_to_primary[core].add(primary)
        spin_h4_derivable &= project_spin_h4_v4(core) in derive_mode_orbit_v4(core.sigma.mode_carrier)
        tau_derivable &= project_tau_v4(core) == _tau_from_state_tuple(core.h.recursive_phase)
        kappa_derivable &= project_kappa_v4(core) == core.h.holonomy_bit
        sigma_depends_on_local_orbits |= len(core.sigma.mode_carrier.generator_composition_profile) == 0
        less_bounded_than_v3 &= len(core.sigma.mode_carrier.generator_composition_profile) == len(COMPONENTS_V4)

    component_signature_counter: Counter[tuple[str, tuple[int, int, int, int]]] = Counter()
    for transition in transitions:
        source_core = active_transport_lift_core_v4(transition.source)
        target_core = active_transport_lift_core_v4(transition.target)
        core_transition_map[source_core][transition.component].add(target_core)
        component_signature_counter[(transition.component, component_update_signature_v4(transition.source, transition.target))] += 1

    primary_count = len(primary_to_core)
    distinct_core_count = len(core_to_primary)
    collision_count = sum(max(len(preimages) - 1, 0) for preimages in core_to_primary.values())
    collision_fraction = collision_count / max(primary_count, 1)

    canonical_core_count = 0
    for component_map in core_transition_map.values():
        if all(len(targets) <= 1 for targets in component_map.values()):
            canonical_core_count += 1
    recursive_consistency_rate = canonical_core_count / max(distinct_core_count, 1)

    v3_metrics = _summary_metrics_from_v3()

    rows: list[dict[str, object]] = []
    rows.append({"scope": "summary", "metric": "primary_states_examined", "count": primary_count, "total": primary_count, "fraction": 1.0, "note": "distinct primary chart states on the bounded lawful H_v8 surface"})
    rows.append({"scope": "summary", "metric": "distinct_spin_H_core_v4_states_reached", "count": distinct_core_count, "total": primary_count, "fraction": distinct_core_count / max(primary_count, 1), "note": "distinct canonical parent states with composition-aware sigma carrier"})
    rows.append({"scope": "summary", "metric": "collision_count", "count": collision_count, "total": primary_count, "fraction": collision_fraction, "note": "many-to-one collisions from primary chart states into composition-aware parent states"})
    rows.append({"scope": "summary", "metric": "recursive_consistency_rate", "count": canonical_core_count, "total": distinct_core_count, "fraction": recursive_consistency_rate, "note": "fraction of parent states whose lawful component updates remain canonical"})

    rows.append({"scope": "projection", "metric": "spin_h4_derivable_from_parent", "count": int(spin_h4_derivable), "total": 1, "fraction": float(spin_h4_derivable), "note": "Pi_pred(spin_H_core_v4) -> spin_h4 via derived mode orbit"})
    rows.append({"scope": "projection", "metric": "tau_derivable_from_parent", "count": int(tau_derivable), "total": 1, "fraction": float(tau_derivable), "note": "Pi_rec(spin_H_core_v4) -> tau"})
    rows.append({"scope": "projection", "metric": "kappa_derivable_from_parent", "count": int(kappa_derivable), "total": 1, "fraction": float(kappa_derivable), "note": "Pi_hol(spin_H_core_v4) -> kappa"})
    rows.append({"scope": "projection", "metric": "sigma_still_depends_on_bounded_local_observable_orbit_summaries", "count": int(sigma_depends_on_local_orbits), "total": 1, "fraction": float(sigma_depends_on_local_orbits), "note": "false when sigma carrier is no longer only a one-step local observable orbit summary"})
    rows.append({"scope": "projection", "metric": "sigma_mode_carrier_less_bounded_than_global_regressive_mode_index", "count": int(less_bounded_than_v3), "total": 1, "fraction": float(less_bounded_than_v3), "note": "true when sigma carrier includes lawful generator-composition profile beyond v3 one-step profile"})

    rows.append({"scope": "comparison_vs_v3", "metric": "collision_change", "count": collision_count - int(v3_metrics["collision_count__count"]), "total": int(v3_metrics["collision_count__count"]), "fraction": collision_fraction - v3_metrics["collision_count__fraction"], "note": "change in collisions relative to spin_H_core_v3"})
    rows.append({"scope": "comparison_vs_v3", "metric": "recursive_consistency_change", "count": canonical_core_count - int(v3_metrics["recursive_consistency_rate__count"]), "total": distinct_core_count, "fraction": recursive_consistency_rate - v3_metrics["recursive_consistency_rate__fraction"], "note": "change in recursive consistency relative to spin_H_core_v3"})
    rows.append({"scope": "comparison_vs_v3", "metric": "distinct_state_count_change", "count": distinct_core_count - int(v3_metrics["distinct_spin_H_core_v3_states_reached__count"]), "total": int(v3_metrics["distinct_spin_H_core_v3_states_reached__count"]), "fraction": (distinct_core_count / max(v3_metrics["distinct_spin_H_core_v3_states_reached__count"], 1.0)) - 1.0, "note": "change in distinct parent states relative to spin_H_core_v3"})

    for component in COMPONENTS_V4:
        matching = [(sig, count) for (name, sig), count in component_signature_counter.items() if name == component]
        matching.sort(key=lambda item: (-item[1], item[0]))
        dominant_sig, dominant_count = matching[0]
        rows.append(
            {
                "scope": "update_law",
                "metric": component,
                "count": dominant_count,
                "total": sum(count for _, count in matching),
                "fraction": dominant_count / max(sum(count for _, count in matching), 1),
                "note": f"dominant_component_delta=theta{dominant_sig[0]}_rho{dominant_sig[1]}_sigma{dominant_sig[2]}_h{dominant_sig[3]}",
            }
        )

    return rows


def write_spinH_core_v4(
    rows: list[dict[str, object]],
    output_path: Path = OUTPUT_PATH_SPINH_CORE_V4,
) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=("scope", "metric", "count", "total", "fraction", "note"))
        writer.writeheader()
        writer.writerows(rows)


__all__ = [
    "OUTPUT_PATH_SPINH_CORE_V4",
    "RegressiveModeCarrierV4",
    "SigmaCoreV4",
    "SpinHCoreV4",
    "active_transport_lift_core_v4",
    "derive_mode_orbit_v4",
    "project_kappa_v4",
    "project_spin_h4_v4",
    "project_tau_v4",
    "summarize_spinH_core_v4",
    "write_spinH_core_v4",
]
