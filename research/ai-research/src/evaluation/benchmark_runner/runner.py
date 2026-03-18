"""Deterministic benchmark runner orchestration."""

from __future__ import annotations

import json
import sys
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from agents.gateway.step_driver import StepDriverAgentConfig, drive_gateway_step
from agents.local_runner.process_bridge import LocalProcessRunner
from core.event_logger import EventLogger, EventRecord, normalize_payload
from core.simulation_controller import SimulationController
from evaluation.benchmark_runner.lifecycle import (
    BenchmarkLifecycleState,
    BenchmarkLifecycleStatus,
    BenchmarkRunLifecycle,
)
from evaluation.benchmark_runner.run_config import BenchmarkRunConfig
from evaluation.benchmark_runner.run_manifest import RunManifest, build_run_manifest
from evaluation.metrics.capability_extractors import (
    CapabilityExtractionResult,
    extract_capability_metrics,
)
from evaluation.metrics.metric_signal import MetricSignal
from evaluation.metrics.metric_tracker import DeterministicMetricTracker, MetricTrackerSnapshot
from evaluation.normalization.metric_normalizer import (
    NormalizationProfile,
    NormalizedMetricResult,
    normalize_capability_metrics,
)
from evaluation.scorecards.scorecard import Scorecard, ScorecardMetadata, build_scorecard
from evaluation.scoring.composite_score import CompositeScoreResult, calculate_composite_scores
from replay.integrity.replay_checksum import compute_replay_artifact_checksum
from replay.integrity.replay_verifier import ReplayParityArtifact, compute_replay_parity_artifact
from replay.logging.replay_artifact import ReplayArtifact, emit_replay_artifact
from replay.logging.replay_log_format import REPLAY_LOG_SCHEMA_VERSION
from replay.logging.runtime_adapter import adapt_runtime_events_to_replay
from scenarios.scenario_loader import (
    ScenarioInitialization,
    build_scenario_initialization,
    load_scenario_definition,
)
from world.rooms.room_graph import DeterministicRoomGraph
from world.state.basic_action_processor import BasicDeterministicActionProcessor
from world.state.spawn_manager import DeterministicSpawnManager, SpawnRequest
from world.state.world_bootstrap import bootstrap_world_state_manager
from world.state.world_state import DeterministicWorldStateManager

_CAPABILITY_KEYS = (
    "exploration_coverage",
    "quest_completion",
    "combat_performance",
    "survival_time",
    "efficiency",
)

_DEFAULT_WEIGHT_MAP = {
    "exploration_coverage": 3.0,
    "quest_completion": 3.0,
    "combat_performance": 2.0,
    "survival_time": 1.0,
    "efficiency": 1.0,
}

_RUNTIME_REPLAY_STATE_SCHEMA = "benchmark_runtime_state_v1"
_RUNTIME_REPLAY_STATE_EVENT_TYPE = "state_snapshot"
_BENCHMARK_VERSION = "0.1"
_SCORING_VERSION = "phase3-v1"
_SOCIAL_GIVE_COMPLETED_METRIC = "social.give.completed"
_SOCIAL_TRADE_COMPLETED_METRIC = "social.trade.completed"
_RUNNER_AGENT_TIMEOUT_SECONDS = 1.0
_RUNNER_AGENT_SCRIPT_EXPLORER = (
    "import json,sys\n"
    "line=sys.stdin.readline()\n"
    "observation=json.loads(line)\n"
    "action_space=observation.get('action_space',[])\n"
    "action='wait'\n"
    "for candidate in action_space:\n"
    "    if candidate.startswith('take '):\n"
    "        action=candidate\n"
    "        break\n"
    "else:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('move '):\n"
    "            action=candidate\n"
    "            break\n"
    "    else:\n"
    "        for candidate in action_space:\n"
    "            if candidate.startswith('attack '):\n"
    "                action=candidate\n"
    "                break\n"
    "        else:\n"
    "            action='look' if 'look' in action_space else 'wait'\n"
    "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
)
_RUNNER_AGENT_SCRIPT_CAUTIOUS = (
    "import json,sys\n"
    "line=sys.stdin.readline()\n"
    "observation=json.loads(line)\n"
    "action_space=observation.get('action_space',[])\n"
    "action='wait'\n"
    "for candidate in action_space:\n"
    "    if candidate.startswith('take '):\n"
    "        action=candidate\n"
    "        break\n"
    "else:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('attack '):\n"
    "            action=candidate\n"
    "            break\n"
    "    else:\n"
    "        for candidate in action_space:\n"
    "            if candidate.startswith('move '):\n"
    "                action=candidate\n"
    "                break\n"
    "        else:\n"
    "            action='look' if 'look' in action_space else 'wait'\n"
    "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
)
_RUNNER_AGENT_SCRIPTS = {
    0: _RUNNER_AGENT_SCRIPT_EXPLORER,
    1: _RUNNER_AGENT_SCRIPT_CAUTIOUS,
}
_RUNNER_AGENT_SCRIPT_MEMORY_PROBE = (
    "import json,sys\n"
    "line=sys.stdin.readline()\n"
    "observation=json.loads(line)\n"
    "action_space=observation.get('action_space',[])\n"
    "remaining_steps=int(observation.get('remaining_steps',0))\n"
    "action='wait'\n"
    "if remaining_steps<=2:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('take '):\n"
    "            action=candidate\n"
    "            break\n"
    "    else:\n"
    "        for candidate in action_space:\n"
    "            if candidate.startswith('move '):\n"
    "                action=candidate\n"
    "                break\n"
    "        else:\n"
    "            action='look' if 'look' in action_space else 'wait'\n"
    "else:\n"
    "    if 'move east' in action_space:\n"
    "        action='move east'\n"
    "    elif 'move west' in action_space:\n"
    "        action='move west'\n"
    "    else:\n"
    "        action='look' if 'look' in action_space else 'wait'\n"
    "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
)
_RUNNER_AGENT_SCRIPT_MEMORY_OBSERVER = (
    "import json,sys\n"
    "line=sys.stdin.readline()\n"
    "observation=json.loads(line)\n"
    "action_space=observation.get('action_space',[])\n"
    "action='look' if 'look' in action_space else 'wait'\n"
    "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
)
_RUNNER_MEMORY_AGENT_SCRIPTS = {
    0: _RUNNER_AGENT_SCRIPT_MEMORY_PROBE,
    1: _RUNNER_AGENT_SCRIPT_MEMORY_OBSERVER,
}
_RUNNER_AGENT_SCRIPT_OBS_EXPLORER = (
    "import json,sys\n"
    "line=sys.stdin.readline()\n"
    "observation=json.loads(line)\n"
    "action_space=observation.get('action_space',[])\n"
    "location=str(observation.get('location',''))\n"
    "action='wait'\n"
    "for candidate in action_space:\n"
    "    if candidate.startswith('take '):\n"
    "        action=candidate\n"
    "        break\n"
    "else:\n"
    "    if location == 'vault' and 'look' in action_space:\n"
    "        action='look'\n"
    "    elif 'move east' in action_space:\n"
    "        action='move east'\n"
    "    elif 'move west' in action_space:\n"
    "        action='move west'\n"
    "    elif 'look' in action_space:\n"
    "        action='look'\n"
    "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
)
_RUNNER_AGENT_SCRIPT_OBS_SUPPORT = (
    "import json,sys\n"
    "line=sys.stdin.readline()\n"
    "observation=json.loads(line)\n"
    "action_space=observation.get('action_space',[])\n"
    "action='look' if 'look' in action_space else 'wait'\n"
    "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
)
_RUNNER_OBS_AGENT_SCRIPTS = {
    0: _RUNNER_AGENT_SCRIPT_OBS_EXPLORER,
    1: _RUNNER_AGENT_SCRIPT_OBS_SUPPORT,
}

_RUNNER_AGENT_SCRIPT_PLANNING_DEPENDENCY = (
    "import json,sys\n"
    "line=sys.stdin.readline()\n"
    "observation=json.loads(line)\n"
    "location=str(observation.get('location',''))\n"
    "inventory=tuple(observation.get('inventory',[]))\n"
    "action_space=tuple(observation.get('action_space',[]))\n"
    "has_key='brass-key' in inventory\n"
    "action=None\n"
    "if 'take brass-key' in action_space and not has_key:\n"
    "    action='take brass-key'\n"
    "elif location == 'key-chamber' and 'move west' in action_space:\n"
    "    action='move west'\n"
    "elif location == 'entry':\n"
    "    if has_key and 'move south' in action_space:\n"
    "        action='move south'\n"
    "    elif 'move east' in action_space:\n"
    "        action='move east'\n"
    "elif location == 'lock-ante':\n"
    "    if 'move north' in action_space:\n"
    "        action='move north'\n"
    "    elif has_key and 'use brass-key' in action_space:\n"
    "        action='use brass-key'\n"
    "    elif 'move south' in action_space:\n"
    "        action='move south'\n"
    "elif location == 'treasure' and 'take artifact' in action_space:\n"
    "    action='take artifact'\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('move '):\n"
    "            action=candidate\n"
    "            break\n"
    "    if action is None:\n"
    "        action='look' if 'look' in action_space else 'wait'\n"
    "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
)
_RUNNER_PLANNING_AGENT_SCRIPTS = {
    0: _RUNNER_AGENT_SCRIPT_PLANNING_DEPENDENCY,
    1: _RUNNER_AGENT_SCRIPT_PLANNING_DEPENDENCY,
}
_RUNNER_AGENT_SCRIPT_SOCIAL_TRADE = (
    "import json,sys\n"
    "line=sys.stdin.readline()\n"
    "observation=json.loads(line)\n"
    "location=str(observation.get('location',''))\n"
    "inventory=tuple(observation.get('inventory',[]))\n"
    "action_space=tuple(observation.get('action_space',[]))\n"
    "has_token='trade-token' in inventory\n"
    "action=None\n"
    "if 'take trade-token' in action_space and not has_token:\n"
    "    action='take trade-token'\n"
    "elif location == 'market':\n"
    "    if has_token and 'give trade-token trader' in action_space:\n"
    "        action='give trade-token trader'\n"
    "    elif 'take artifact' in action_space:\n"
    "        action='take artifact'\n"
    "elif location == 'start':\n"
    "    if not has_token and 'move west' in action_space:\n"
    "        action='move west'\n"
    "    elif has_token and 'move east' in action_space:\n"
    "        action='move east'\n"
    "elif location == 'token-room' and 'move west' in action_space:\n"
    "    action='move west'\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('move '):\n"
    "            action=candidate\n"
    "            break\n"
    "    if action is None:\n"
    "        action='look' if 'look' in action_space else 'wait'\n"
    "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
)
_RUNNER_SOCIAL_TRADE_AGENT_SCRIPTS = {
    0: _RUNNER_AGENT_SCRIPT_SOCIAL_TRADE,
    1: _RUNNER_AGENT_SCRIPT_SOCIAL_TRADE,
}


@dataclass(frozen=True, slots=True)
class BenchmarkRunnerConfig:
    """Legacy benchmark runner configuration wrapper."""

    run_id: str
    benchmark_id: str
    scenario: Mapping[str, Any] | str
    actor_ids: Sequence[str]
    run_seed: int | None = None
    seed_override: int | None = None
    max_steps_override: int | None = None
    normalization_profiles: Mapping[str, NormalizationProfile | Mapping[str, Any]] | None = None
    score_weights: Mapping[str, float] | None = None

    def __post_init__(self) -> None:
        if not isinstance(self.run_id, str) or not self.run_id:
            raise ValueError("run_id must be a non-empty string")
        if not isinstance(self.benchmark_id, str) or not self.benchmark_id:
            raise ValueError("benchmark_id must be a non-empty string")
        if not isinstance(self.scenario, (Mapping, str)):
            raise ValueError("scenario must be a mapping or JSON string")
        if isinstance(self.actor_ids, (str, bytes)) or not isinstance(self.actor_ids, Sequence):
            raise ValueError("actor_ids must be a sequence of strings")

        normalized_actor_ids: list[str] = []
        seen_actor_ids: set[str] = set()
        for actor_id in self.actor_ids:
            if not isinstance(actor_id, str) or not actor_id:
                raise ValueError("actor_ids must contain non-empty strings")
            if actor_id in seen_actor_ids:
                raise ValueError(f"duplicate actor_id in actor_ids: {actor_id}")
            seen_actor_ids.add(actor_id)
            normalized_actor_ids.append(actor_id)
        if len(normalized_actor_ids) == 0:
            raise ValueError("actor_ids must contain at least one actor")
        object.__setattr__(self, "actor_ids", tuple(sorted(normalized_actor_ids)))

        if self.run_seed is not None and (not isinstance(self.run_seed, int) or isinstance(self.run_seed, bool)):
            raise ValueError("run_seed must be None or an integer")
        if self.seed_override is not None and (
            not isinstance(self.seed_override, int) or isinstance(self.seed_override, bool)
        ):
            raise ValueError("seed_override must be None or an integer")
        if self.max_steps_override is not None and (
            not isinstance(self.max_steps_override, int)
            or isinstance(self.max_steps_override, bool)
            or self.max_steps_override <= 0
        ):
            raise ValueError("max_steps_override must be None or a positive integer")
        if self.normalization_profiles is not None and not isinstance(self.normalization_profiles, Mapping):
            raise ValueError("normalization_profiles must be a mapping")
        if self.score_weights is not None and not isinstance(self.score_weights, Mapping):
            raise ValueError("score_weights must be a mapping")

    def to_run_config(self) -> BenchmarkRunConfig:
        """Convert wrapper config into canonical run configuration."""
        return BenchmarkRunConfig(
            run_id=self.run_id,
            benchmark_id=self.benchmark_id,
            scenario=self.scenario,
            actor_ids=self.actor_ids,
            run_seed=self.run_seed,
            seed_override=self.seed_override,
            max_steps_override=self.max_steps_override,
        )


@dataclass(frozen=True, slots=True)
class BenchmarkRunnerResult:
    """Deterministic end-to-end benchmark runner artifact bundle."""

    lifecycle_state: BenchmarkLifecycleState
    scenario_initialization: ScenarioInitialization
    run_manifest: RunManifest
    tracker_snapshot: MetricTrackerSnapshot
    capability_result: CapabilityExtractionResult
    normalized_result: NormalizedMetricResult
    composite_result: CompositeScoreResult
    scorecard: Scorecard
    replay_artifact: ReplayArtifact
    replay_parity_artifact: ReplayParityArtifact
    replay_artifact_refs: tuple[tuple[str, str], ...]

    def to_dict(self) -> dict[str, Any]:
        return {
            "lifecycle_state": {
                "run_id": self.lifecycle_state.run_id,
                "scenario_id": self.lifecycle_state.scenario_id,
                "seed": self.lifecycle_state.seed,
                "max_steps": self.lifecycle_state.max_steps,
                "step_index": self.lifecycle_state.step_index,
                "status": self.lifecycle_state.status.value,
            },
            "scenario_initialization": self.scenario_initialization.to_dict(),
            "run_manifest": self.run_manifest.to_dict(),
            "tracker_snapshot": self.tracker_snapshot.to_dict(),
            "capability_result": self.capability_result.to_dict(),
            "normalized_result": self.normalized_result.to_dict(),
            "composite_result": self.composite_result.to_dict(),
            "scorecard": self.scorecard.to_dict(),
            "replay_artifact": self.replay_artifact.to_dict(),
            "replay_parity_artifact": self.replay_parity_artifact.to_dict(),
            "replay_artifact_refs": [
                {"name": name, "ref": ref}
                for name, ref in self.replay_artifact_refs
            ],
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


@dataclass(frozen=True, slots=True)
class _ResolvedRunnerConfig:
    run_config: BenchmarkRunConfig
    normalization_profiles: Mapping[str, NormalizationProfile | Mapping[str, Any]] | None = None
    score_weights: Mapping[str, float] | None = None


def run_benchmark_lifecycle(
    config: BenchmarkRunnerConfig | BenchmarkRunConfig | Mapping[str, Any],
) -> BenchmarkRunnerResult:
    """Execute deterministic benchmark lifecycle from scenario load to scorecard emission."""
    resolved_config = _coerce_runner_config(config)
    run_config = resolved_config.run_config

    scenario_load = load_scenario_definition(run_config.scenario)
    if not scenario_load.accepted or scenario_load.scenario is None:
        raise ValueError(f"scenario load rejected: {scenario_load.reason}")

    initialization = build_scenario_initialization(
        scenario_load.scenario,
        seed_override=run_config.effective_seed,
    )
    max_steps = initialization.max_steps
    if run_config.max_steps_override is not None:
        max_steps = run_config.max_steps_override

    lifecycle = BenchmarkRunLifecycle(
        run_id=run_config.run_id,
        scenario_id=initialization.scenario_id,
        seed=initialization.run_seed,
        max_steps=max_steps,
    )
    lifecycle.start()
    run_manifest = build_run_manifest(
        run_config=run_config,
        scenario_id=initialization.scenario_id,
        scenario_version=initialization.version,
        benchmark_version=_BENCHMARK_VERSION,
        scoring_version=_SCORING_VERSION,
        max_steps=max_steps,
    )

    tracker = DeterministicMetricTracker(run_id=run_config.run_id)
    scenario_vars = _scenario_vars_to_dict(initialization.scenario_vars)
    world_config = _extract_world_config(initialization.scenario_vars)
    world_config = _apply_seed_variation_to_world_config(
        world_config=world_config,
        scenario_vars=scenario_vars,
        seed=initialization.run_seed,
    )
    world_state = _build_runner_world_state(
        start_room_id=initialization.start_room_id,
        actor_ids=run_config.actor_ids,
        seed=initialization.run_seed,
        world_config=world_config,
        scenario_vars=scenario_vars,
    )
    controller_logger = _InMemoryEventLogger()
    controller = SimulationController(
        world_state_manager=world_state,
        action_processor=BasicDeterministicActionProcessor(),
        event_logger=controller_logger,
        seed=initialization.run_seed,
        max_steps=max_steps,
        run_id=run_config.run_id,
    )
    controller.initialize()
    gateway_agent_configs = _build_gateway_agent_configs(
        run_config.actor_ids,
        script_policy=_resolve_agent_script_policy(scenario_vars),
    )
    runtime_events: list[EventRecord] = []
    while lifecycle.state.status is BenchmarkLifecycleStatus.RUNNING:
        step = lifecycle.state.step_index
        gateway_step = drive_gateway_step(
            snapshot=world_state.get_snapshot(),
            run_id=run_config.run_id,
            step=step,
            max_steps=max_steps,
            agent_configs=gateway_agent_configs,
        )
        step_outcome = controller.step(gateway_step.accepted_action_requests)
        tracker.apply_signals(
            _build_runtime_step_signals(
                run_id=run_config.run_id,
                step=step,
                actor_ids=run_config.actor_ids,
                accepted_action_count=step_outcome.processed_actions,
                gateway_failures=gateway_step.failures,
                emitted_events=step_outcome.emitted_events,
            )
        )
        runtime_events.extend(step_outcome.emitted_events)
        tracker_snapshot_at_step = tracker.snapshot()
        runtime_events.append(
            EventRecord(
                step_index=step,
                event_type=_RUNTIME_REPLAY_STATE_EVENT_TYPE,
                payload=normalize_payload(
                    {
                        "state_schema": _RUNTIME_REPLAY_STATE_SCHEMA,
                        "state_json": _build_runtime_state_snapshot_json(
                            run_manifest=run_manifest,
                            tracker_snapshot=tracker_snapshot_at_step,
                            step_index=step,
                        ),
                    }
                ),
            )
        )
        lifecycle.advance_step()

    tracker_snapshot = tracker.snapshot()
    capability_result = extract_capability_metrics(tracker_snapshot)
    normalized_result = normalize_capability_metrics(
        capability_result,
        profiles=_resolve_normalization_profiles(
            max_steps=max_steps,
            actor_count=len(run_config.actor_ids),
            overrides=resolved_config.normalization_profiles,
        ),
    )
    composite_result = calculate_composite_scores(
        normalized_result,
        weights=_resolve_weight_map(resolved_config.score_weights),
    )
    scorecard = build_scorecard(
        metadata=ScorecardMetadata(
            run_id=run_config.run_id,
            benchmark_id=run_config.benchmark_id,
            scenario_id=initialization.scenario_id,
            benchmark_version=run_manifest.benchmark_version,
            scenario_version=run_manifest.scenario_version,
            seed=initialization.run_seed,
            step_count=lifecycle.state.step_index,
            scorer_version=run_manifest.scoring_version,
        ),
        composite_result=composite_result,
    )
    if scorecard.metadata.benchmark_version != run_manifest.benchmark_version:
        raise RuntimeError("runtime version provenance mismatch: scorecard benchmark_version")
    if scorecard.metadata.scenario_version != run_manifest.scenario_version:
        raise RuntimeError("runtime version provenance mismatch: scorecard scenario_version")
    if scorecard.metadata.scoring_version != run_manifest.scoring_version:
        raise RuntimeError("runtime version provenance mismatch: scorecard scoring_version")
    replay_artifact, replay_artifact_refs = _emit_runtime_replay_artifact(
        run_manifest=run_manifest,
        initialization=initialization,
        lifecycle_state=lifecycle.state,
        runtime_events=tuple(runtime_events),
    )
    parity_result = compute_replay_parity_artifact(
        replay_artifact=replay_artifact,
        scorecard=scorecard,
    )
    if not parity_result.accepted or parity_result.parity_artifact is None:
        reason = parity_result.reason or "unknown_parity_computation_failure"
        raise RuntimeError(f"runtime replay parity computation rejected: {reason}")
    return BenchmarkRunnerResult(
        lifecycle_state=lifecycle.state,
        scenario_initialization=initialization,
        run_manifest=run_manifest,
        tracker_snapshot=tracker_snapshot,
        capability_result=capability_result,
        normalized_result=normalized_result,
        composite_result=composite_result,
        scorecard=scorecard,
        replay_artifact=replay_artifact,
        replay_parity_artifact=parity_result.parity_artifact,
        replay_artifact_refs=replay_artifact_refs,
    )


def _coerce_runner_config(
    config: BenchmarkRunnerConfig | BenchmarkRunConfig | Mapping[str, Any],
) -> _ResolvedRunnerConfig:
    if isinstance(config, BenchmarkRunConfig):
        return _ResolvedRunnerConfig(run_config=config)

    if isinstance(config, BenchmarkRunnerConfig):
        return _ResolvedRunnerConfig(
            run_config=config.to_run_config(),
            normalization_profiles=config.normalization_profiles,
            score_weights=config.score_weights,
        )

    if isinstance(config, Mapping):
        return _ResolvedRunnerConfig(run_config=BenchmarkRunConfig.from_mapping(config))

    raise ValueError("config must be BenchmarkRunConfig, BenchmarkRunnerConfig, or mapping")


def _build_runtime_step_signals(
    *,
    run_id: str,
    step: int,
    actor_ids: Sequence[str],
    accepted_action_count: int,
    gateway_failures: Sequence[Any],
    emitted_events: Sequence[EventRecord],
) -> tuple[MetricSignal, ...]:
    signals: list[MetricSignal] = []
    for actor_id in actor_ids:
        signals.append(
            MetricSignal(
                run_id=run_id,
                step=step,
                actor_id=actor_id,
                metric_name="survival.steps_alive",
                value=step + 1,
            )
        )
    for event in emitted_events:
        if event.actor_id is None:
            continue
        payload = {key: value for key, value in event.payload}
        if event.event_type == "action_move":
            signals.append(
                MetricSignal(
                    run_id=run_id,
                    step=step,
                    actor_id=event.actor_id,
                    metric_name="coverage.rooms",
                    value=1.0,
                )
            )
            signals.append(
                MetricSignal(
                    run_id=run_id,
                    step=step,
                    actor_id=event.actor_id,
                    metric_name="objective.progress",
                    value=1.0,
                )
            )
        elif event.event_type == "action_take":
            signals.append(
                MetricSignal(
                    run_id=run_id,
                    step=step,
                    actor_id=event.actor_id,
                    metric_name="quest.completed",
                    value=1.0,
                )
            )
            signals.append(
                MetricSignal(
                    run_id=run_id,
                    step=step,
                    actor_id=event.actor_id,
                    metric_name="objective.progress",
                    value=1.0,
                )
            )
        elif event.event_type == "action_attack":
            damage = float(payload.get("damage", 0.0))
            if damage > 0:
                signals.append(
                    MetricSignal(
                        run_id=run_id,
                        step=step,
                        actor_id=event.actor_id,
                        metric_name="combat.damage_dealt",
                        value=damage,
                    )
                )
            target_id = payload.get("target_id")
            if isinstance(target_id, str) and target_id:
                signals.append(
                    MetricSignal(
                        run_id=run_id,
                        step=step,
                        actor_id=target_id,
                        metric_name="combat.damage_taken",
                        value=damage,
                    )
                )
        elif event.event_type == "action_wait":
            signals.append(
                MetricSignal(
                    run_id=run_id,
                    step=step,
                    actor_id=event.actor_id,
                    metric_name="objective.progress",
                    value=0.5,
                )
            )
        elif event.event_type == "action_give":
            signals.append(
                MetricSignal(
                    run_id=run_id,
                    step=step,
                    actor_id=event.actor_id,
                    metric_name="objective.progress",
                    value=1.0,
                )
            )
            signals.append(
                MetricSignal(
                    run_id=run_id,
                    step=step,
                    actor_id=event.actor_id,
                    metric_name=_SOCIAL_GIVE_COMPLETED_METRIC,
                    value=1.0,
                )
            )
        elif event.event_type == "dependency_unlocked":
            if "target_id" in payload and "reward_item_id" in payload:
                signals.append(
                    MetricSignal(
                        run_id=run_id,
                        step=step,
                        actor_id=event.actor_id,
                        metric_name=_SOCIAL_TRADE_COMPLETED_METRIC,
                        value=1.0,
                    )
                )

    for actor_id in actor_ids:
        signals.append(
            MetricSignal(
                run_id=run_id,
                step=step,
                actor_id=actor_id,
                metric_name="actions.count",
                value=accepted_action_count / max(len(actor_ids), 1),
            )
        )
    for failure in gateway_failures:
        actor_id = getattr(failure, "actor_id", None)
        if isinstance(actor_id, str) and actor_id:
            signals.append(
                MetricSignal(
                    run_id=run_id,
                    step=step,
                    actor_id=actor_id,
                    metric_name="actions.count",
                    value=1.0,
                )
            )
    return tuple(signals)


class _InMemoryEventLogger(EventLogger):
    def __init__(self) -> None:
        self._events: list[EventRecord] = []

    def log(self, event: EventRecord) -> None:
        self._events.append(event)

    def records(self) -> Sequence[EventRecord]:
        return tuple(self._events)

    def reset(self) -> None:
        self._events.clear()


def _extract_world_config(
    scenario_vars: Sequence[tuple[str, Any]],
) -> Mapping[str, Any] | None:
    for key, value in scenario_vars:
        if key == "world_config_json" and isinstance(value, str):
            try:
                parsed = json.loads(value)
            except json.JSONDecodeError:
                return None
            if isinstance(parsed, Mapping):
                return parsed
    return None


def _build_unlock_effects_scenario_delta(
    world_config: Mapping[str, Any] | None
) -> dict[str, Any]:
    if not isinstance(world_config, Mapping):
        return {}
    raw_effects = world_config.get("unlock_effects")
    if not isinstance(raw_effects, Sequence):
        return {}

    effect_map: dict[str, dict[str, Any]] = {}
    for raw_effect in raw_effects:
        if not isinstance(raw_effect, Mapping):
            continue

        item_id = raw_effect.get("item_id")
        source_room = raw_effect.get("source_room_id")
        direction = raw_effect.get("direction")
        destination_room = raw_effect.get("destination_room_id")
        if (
            not isinstance(item_id, str)
            or not item_id
            or not isinstance(source_room, str)
            or not source_room
            or not isinstance(direction, str)
            or not direction
            or not isinstance(destination_room, str)
            or not destination_room
        ):
            continue

        effect_id = raw_effect.get("effect_id")
        if not isinstance(effect_id, str) or not effect_id:
            effect_id = f"unlock:{item_id}:{source_room}:{direction}:{destination_room}"

        consume_item = bool(raw_effect.get("consume_item"))
        requires_actor_in_place = raw_effect.get("requires_actor_in_place", True) is not False

        effect_map[item_id] = {
            "effect_id": effect_id,
            "source_room_id": source_room,
            "direction": direction,
            "destination_room_id": destination_room,
            "consume_item": consume_item,
            "requires_actor_in_place": requires_actor_in_place,
        }

    if not effect_map:
        return {}

    return {
        "unlock_effects_json": json.dumps(
            effect_map,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=True,
        )
    }


def _build_trade_effects_scenario_delta(
    world_config: Mapping[str, Any] | None
) -> dict[str, Any]:
    if not isinstance(world_config, Mapping):
        return {}
    raw_effects = world_config.get("trade_effects")
    if not isinstance(raw_effects, Sequence):
        return {}

    effect_map: dict[str, dict[str, Any]] = {}
    for raw_effect in raw_effects:
        if not isinstance(raw_effect, Mapping):
            continue
        item_id = raw_effect.get("item_id")
        target_id = raw_effect.get("target_id")
        reward_item_id = raw_effect.get("reward_item_id")
        if (
            not isinstance(item_id, str)
            or not item_id
            or not isinstance(target_id, str)
            or not target_id
            or not isinstance(reward_item_id, str)
            or not reward_item_id
        ):
            continue
        effect_id = raw_effect.get("effect_id")
        if not isinstance(effect_id, str) or not effect_id:
            effect_id = f"trade:{item_id}:{target_id}:{reward_item_id}"
        effect_map[f"{item_id}|{target_id}"] = {
            "effect_id": effect_id,
            "reward_item_id": reward_item_id,
            "reward_entity_type": str(raw_effect.get("reward_entity_type", "item")),
        }

    if not effect_map:
        return {}

    return {
        "trade_effects_json": json.dumps(
            effect_map,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=True,
        )
    }


def _scenario_vars_to_dict(scenario_vars: Sequence[tuple[str, Any]]) -> dict[str, Any]:
    return {key: value for key, value in scenario_vars}


def _apply_seed_variation_to_world_config(
    *,
    world_config: Mapping[str, Any] | None,
    scenario_vars: Mapping[str, Any],
    seed: int,
) -> Mapping[str, Any] | None:
    if world_config is None:
        return None
    if scenario_vars.get("seed_variation_policy") != "tiny_fetch_v1":
        return world_config
    if scenario_vars.get("seed_variation_axis") != "key_room":
        return world_config

    raw_values = scenario_vars.get("seed_variation_values_json")
    if not isinstance(raw_values, str):
        return world_config
    try:
        parsed_values = json.loads(raw_values)
    except json.JSONDecodeError:
        return world_config
    if not isinstance(parsed_values, list):
        return world_config
    seed_values = tuple(
        value for value in parsed_values if isinstance(value, str) and value
    )
    if len(seed_values) == 0:
        return world_config

    selected_room = seed_values[seed % len(seed_values)]
    world_payload = json.loads(
        json.dumps(world_config, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    )
    items = world_payload.get("items")
    if not isinstance(items, list):
        return world_payload

    for item in items:
        if not isinstance(item, dict):
            continue
        if item.get("entity_id") == "golden-key":
            item["location"] = selected_room
    return world_payload


def _build_runner_world_state(
    *,
    start_room_id: str,
    actor_ids: Sequence[str],
    seed: int,
    world_config: Mapping[str, Any] | None = None,
    scenario_vars: Mapping[str, Any] | None = None,
) -> DeterministicWorldStateManager:
    if not isinstance(start_room_id, str) or not start_room_id:
        raise ValueError("start_room_id must be a non-empty string")

    if world_config is not None and isinstance(world_config, Mapping):
        rooms_config = world_config.get("rooms")
        if isinstance(rooms_config, Mapping) and rooms_config:
            room_graph = DeterministicRoomGraph.from_dict({"rooms": dict(rooms_config)})
        else:
            room_graph = _default_room_graph(start_room_id)
    else:
        room_graph = _default_room_graph(start_room_id)

    world = bootstrap_world_state_manager(room_graph, seed=seed)
    if scenario_vars is not None and len(scenario_vars) > 0:
        scenario_vars_delta: dict[str, Any] = {}
        for key, value in sorted(scenario_vars.items(), key=lambda item: str(item[0])):
            if isinstance(value, (str, int, float, bool)) or value is None:
                scenario_vars_delta[str(key)] = value
        if len(scenario_vars_delta) > 0:
            world.apply_delta({"scenario_vars": scenario_vars_delta})

    unlock_effects_delta = _build_unlock_effects_scenario_delta(world_config)
    if unlock_effects_delta:
        world.apply_delta({"scenario_vars": unlock_effects_delta})
    trade_effects_delta = _build_trade_effects_scenario_delta(world_config)
    if trade_effects_delta:
        world.apply_delta({"scenario_vars": trade_effects_delta})

    if world_config is not None and isinstance(world_config, Mapping):
        entity_delta: dict[str, dict[str, Any]] = {}
        room_delta: dict[str, dict[str, Any]] = {}
        for item_def in world_config.get("items", ()):
            if not isinstance(item_def, Mapping):
                continue
            item_id = item_def.get("entity_id")
            item_loc = item_def.get("location")
            if not isinstance(item_id, str) or not isinstance(item_loc, str):
                continue
            entity_delta[item_id] = {
                "entity_id": item_id,
                "entity_type": str(item_def.get("entity_type", "item")),
                "location": item_loc,
            }
            snapshot = world.get_snapshot()
            rooms = snapshot.get("rooms", {})
            if item_loc in rooms:
                room_payload = dict(rooms[item_loc])
                entities_present = list(room_payload.get("entities_present", []))
                if item_id not in entities_present:
                    entities_present.append(item_id)
                    entities_present.sort()
                room_payload["entities_present"] = entities_present
                room_delta[item_loc] = room_payload

        for npc_def in world_config.get("npcs", ()):
            if not isinstance(npc_def, Mapping):
                continue
            npc_id = npc_def.get("entity_id")
            npc_loc = npc_def.get("location")
            if not isinstance(npc_id, str) or not isinstance(npc_loc, str):
                continue
            npc_entity: dict[str, Any] = {
                "entity_id": npc_id,
                "entity_type": "npc",
                "location": npc_loc,
            }
            if "health" in npc_def and isinstance(npc_def["health"], int):
                npc_entity["health"] = npc_def["health"]
            entity_delta[npc_id] = npc_entity
            snapshot = world.get_snapshot()
            rooms = snapshot.get("rooms", {})
            base_room = room_delta.get(npc_loc)
            if base_room is None and npc_loc in rooms:
                base_room = dict(rooms[npc_loc])
            if base_room is not None:
                entities_present = list(base_room.get("entities_present", []))
                if npc_id not in entities_present:
                    entities_present.append(npc_id)
                    entities_present.sort()
                base_room["entities_present"] = entities_present
                room_delta[npc_loc] = base_room

        if entity_delta or room_delta:
            delta: dict[str, Any] = {}
            if entity_delta:
                delta["entities"] = entity_delta
            if room_delta:
                delta["rooms"] = room_delta
            world.apply_delta(delta)

    spawn_requests = tuple(
        SpawnRequest(actor_id=actor_id, actor_type="agent", preferred_room_id=start_room_id)
        for actor_id in actor_ids
    )
    DeterministicSpawnManager(seed=seed).place_actors(world, spawn_requests)
    return DeterministicWorldStateManager.from_json(world.to_json())


def _default_room_graph(start_room_id: str) -> DeterministicRoomGraph:
    east_room_id = f"{start_room_id}-east"
    return DeterministicRoomGraph.from_dict(
        {
            "rooms": {
                start_room_id: {
                    "title": "Runner Start Room",
                    "description": "Deterministic benchmark runner start room.",
                    "exits": {"east": east_room_id},
                    "entities": [],
                },
                east_room_id: {
                    "title": "Runner East Room",
                    "description": "Deterministic benchmark runner east room.",
                    "exits": {"west": start_room_id},
                    "entities": [],
                },
            }
        }
    )


def _build_gateway_agent_configs(
    actor_ids: Sequence[str],
    *,
    script_policy: str | None = None,
) -> tuple[StepDriverAgentConfig, ...]:
    script_map = _resolve_runner_agent_scripts(script_policy)
    configs: list[StepDriverAgentConfig] = []
    for idx, actor_id in enumerate(actor_ids):
        script = script_map.get(idx % len(script_map), _RUNNER_AGENT_SCRIPT_EXPLORER)
        command = (sys.executable, "-c", script)
        configs.append(
            StepDriverAgentConfig(
                actor_id=actor_id,
                runner=LocalProcessRunner(command),
                timeout_seconds=_RUNNER_AGENT_TIMEOUT_SECONDS,
            )
        )
    return tuple(configs)


def _resolve_runner_agent_scripts(script_policy: str | None) -> Mapping[int, str]:
    if script_policy == "memory_delayed_retrieval_v1":
        return _RUNNER_MEMORY_AGENT_SCRIPTS
    if script_policy == "partial_observability_v1":
        return _RUNNER_OBS_AGENT_SCRIPTS
    if script_policy == "planning-dependency":
        return _RUNNER_PLANNING_AGENT_SCRIPTS
    if script_policy == "social-trade-dependency":
        return _RUNNER_SOCIAL_TRADE_AGENT_SCRIPTS
    return _RUNNER_AGENT_SCRIPTS


def _resolve_agent_script_policy(scenario_vars: Mapping[str, Any]) -> str | None:
    raw_policy = scenario_vars.get("agent_script_policy")
    if isinstance(raw_policy, str) and raw_policy:
        return raw_policy
    return None


def _resolve_normalization_profiles(
    *,
    max_steps: int,
    actor_count: int,
    overrides: Mapping[str, NormalizationProfile | Mapping[str, Any]] | None,
) -> dict[str, NormalizationProfile]:
    if actor_count <= 0:
        raise ValueError("actor_count must be positive")
    defaults: dict[str, NormalizationProfile] = {
        "exploration_coverage": NormalizationProfile(minimum=0.0, maximum=float(max_steps * actor_count)),
        "quest_completion": NormalizationProfile(minimum=0.0, maximum=float(max_steps)),
        "combat_performance": NormalizationProfile(minimum=0.0, maximum=float(max_steps * actor_count)),
        "survival_time": NormalizationProfile(minimum=0.0, maximum=float(max_steps)),
        "efficiency": NormalizationProfile(minimum=0.0, maximum=float(actor_count)),
    }
    if overrides is None:
        return defaults

    if not isinstance(overrides, Mapping):
        raise ValueError("normalization_profiles must be a mapping")
    for key, raw_profile in overrides.items():
        if key not in _CAPABILITY_KEYS:
            raise ValueError(f"unexpected normalization profile key: {key}")
        if isinstance(raw_profile, NormalizationProfile):
            defaults[key] = raw_profile
        elif isinstance(raw_profile, Mapping):
            defaults[key] = NormalizationProfile.from_mapping(raw_profile)
        else:
            raise ValueError(f"normalization profile '{key}' must be mapping or NormalizationProfile")
    return defaults


def _resolve_weight_map(overrides: Mapping[str, float] | None) -> dict[str, float]:
    if overrides is None:
        return dict(_DEFAULT_WEIGHT_MAP)
    if not isinstance(overrides, Mapping):
        raise ValueError("score_weights must be a mapping")
    merged = dict(_DEFAULT_WEIGHT_MAP)
    for key, value in overrides.items():
        if key not in _CAPABILITY_KEYS:
            raise ValueError(f"unexpected score weight key: {key}")
        merged[key] = value
    return merged


def _build_runtime_state_snapshot_json(
    *,
    run_manifest: RunManifest,
    tracker_snapshot: MetricTrackerSnapshot,
    step_index: int,
) -> str:
    if not isinstance(step_index, int) or isinstance(step_index, bool) or step_index < 0:
        raise ValueError("step_index must be a non-negative integer")

    tracker_payload = tracker_snapshot.to_dict()
    canonical_state = {
        "schema_version": _RUNTIME_REPLAY_STATE_SCHEMA,
        "run_id": run_manifest.run_id,
        "benchmark_id": run_manifest.benchmark_id,
        "scenario_id": run_manifest.scenario_id,
        "step_index": step_index,
        "agent_states": tracker_payload["actors"],
        "item_states": [],
        "npc_states": [],
        "room_states": [],
        "tracker_total_signals": tracker_payload["total_signals"],
    }
    return json.dumps(
        canonical_state,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    )


def _emit_runtime_replay_artifact(
    *,
    run_manifest: RunManifest,
    initialization: ScenarioInitialization,
    lifecycle_state: BenchmarkLifecycleState,
    runtime_events: Sequence[EventRecord],
) -> tuple[ReplayArtifact, tuple[tuple[str, str], ...]]:
    if run_manifest.scenario_version != initialization.version:
        raise RuntimeError("runtime version provenance mismatch: manifest scenario_version")

    adapt_result = adapt_runtime_events_to_replay(
        run_id=run_manifest.run_id,
        events=runtime_events,
    )
    if not adapt_result.accepted:
        reason = adapt_result.reason or "unknown_runtime_event_adaptation_failure"
        raise RuntimeError(f"runtime replay event adaptation rejected: {reason}")

    emit_result = emit_replay_artifact(
        envelope={
            "schema_version": REPLAY_LOG_SCHEMA_VERSION,
            "run_id": run_manifest.run_id,
            "benchmark_id": run_manifest.benchmark_id,
            "scenario_id": run_manifest.scenario_id,
            "initial_seed": run_manifest.effective_seed,
            "seed_source": run_manifest.seed_source,
            "actor_ids": list(run_manifest.actor_ids),
            "max_steps": run_manifest.max_steps,
            "run_metadata": {
                "lifecycle_status": lifecycle_state.status.value,
                "runtime_source": "benchmark_runner",
                "benchmark_version": run_manifest.benchmark_version,
                "scenario_version": run_manifest.scenario_version,
                "scoring_version": run_manifest.scoring_version,
                "step_count": lifecycle_state.step_index,
                "reconstruction_state_schema": _RUNTIME_REPLAY_STATE_SCHEMA,
                "reconstruction_state_event_type": _RUNTIME_REPLAY_STATE_EVENT_TYPE,
            },
        },
        events=adapt_result.records,
    )
    if not emit_result.accepted or emit_result.artifact is None:
        reason = emit_result.reason or "unknown_replay_artifact_emission_failure"
        raise RuntimeError(f"runtime replay artifact emission rejected: {reason}")

    checksum_result = compute_replay_artifact_checksum(emit_result.artifact)
    if not checksum_result.accepted or checksum_result.checksum is None:
        reason = checksum_result.reason or "unknown_replay_checksum_failure"
        raise RuntimeError(f"runtime replay checksum computation rejected: {reason}")

    artifact_digest = checksum_result.checksum.digest_hex
    refs = (
        ("replay_artifact", f"sha256:{artifact_digest}"),
        ("replay_checksum", f"sha256:{artifact_digest}"),
    )
    return emit_result.artifact, refs
