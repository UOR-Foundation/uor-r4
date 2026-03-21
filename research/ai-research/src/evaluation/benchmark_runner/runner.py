"""Deterministic benchmark runner orchestration."""

from __future__ import annotations

import json
import shutil
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Mapping, Sequence

from agents.gateway.action_pipeline import run_action_pipeline
from agents.gateway.observation_builder import build_observation_for_actor
from agents.llm_runtime import BenchmarkLLMTurnResult, run_mock_benchmark_llm_turn
from agents.protocol.action import ActionSubmission
from agents.protocol.observation import Observation
from agents.gateway.step_driver import StepDriverAgentConfig, drive_gateway_step
from agents.local_runner.process_bridge import LocalProcessRunner
from agents.local_runner.session_manager import DeterministicLocalRunnerSessionManager
from agents.local_runner.session_manager import LocalRunnerSessionRequest
from core.event_logger import EventLogger, EventRecord, normalize_payload
from core.simulation_controller import SimulationController
from core.run_state import RunStatus
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
from world.state.shard_state import ShardState
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
_LLM_RUNTIME_TELEMETRY_SCHEMA = "llm_runtime_turn_v1"
_LLM_RUNTIME_TELEMETRY_EVENT_TYPE = "agent_runtime_telemetry"
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
_RUNNER_AGENT_SCRIPT_GUARDED_RELIC = (
    "import json,sys\n"
    "line=sys.stdin.readline()\n"
    "observation=json.loads(line)\n"
    "step=int(observation.get('step',0))\n"
    "location=str(observation.get('location',''))\n"
    "inventory=tuple(observation.get('inventory',[]))\n"
    "entities=tuple(observation.get('entities',[]))\n"
    "action_space=tuple(observation.get('action_space',[]))\n"
    "has_key='relic-key' in inventory\n"
    "sentinel_present=False\n"
    "for entity in entities:\n"
    "    if isinstance(entity,dict) and entity.get('type') == 'npc' and entity.get('name') == 'sentinel':\n"
    "        sentinel_present=True\n"
    "        break\n"
    "action=None\n"
    "if 'take relic-key' in action_space and not has_key:\n"
    "    action='take relic-key'\n"
    "elif location == 'watch-post' and sentinel_present and step <= 3 and 'attack sentinel' in action_space:\n"
    "    action='attack sentinel'\n"
    "elif location == 'seal-door':\n"
    "    if 'move north' in action_space:\n"
    "        action='move north'\n"
    "    elif has_key and 'use relic-key' in action_space:\n"
    "        action='use relic-key'\n"
    "    elif 'move west' in action_space:\n"
    "        action='move west'\n"
    "elif location == 'reliquary' and 'take relic' in action_space:\n"
    "    action='take relic'\n"
    "elif location == 'camp':\n"
    "    if not has_key and 'move east' in action_space:\n"
    "        action='move east'\n"
    "    elif 'move north' in action_space:\n"
    "        action='move north'\n"
    "elif location == 'armory':\n"
    "    if not has_key and 'take relic-key' in action_space:\n"
    "        action='take relic-key'\n"
    "    elif 'move north' in action_space:\n"
    "        action='move north'\n"
    "    elif 'move west' in action_space:\n"
    "        action='move west'\n"
    "elif location == 'watch-post':\n"
    "    if 'move east' in action_space:\n"
    "        action='move east'\n"
    "    elif 'move south' in action_space:\n"
    "        action='move south'\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('take '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('move '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('attack '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    action='look' if 'look' in action_space else 'wait'\n"
    "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
)
_RUNNER_GUARDED_RELIC_AGENT_SCRIPTS = {
    0: _RUNNER_AGENT_SCRIPT_GUARDED_RELIC,
    1: _RUNNER_AGENT_SCRIPT_GUARDED_RELIC,
}
_RUNNER_AGENT_SCRIPT_HAZARD_TRADEOFF = (
    "import json,sys\n"
    "line=sys.stdin.readline()\n"
    "observation=json.loads(line)\n"
    "location=str(observation.get('location',''))\n"
    "inventory=tuple(observation.get('inventory',[]))\n"
    "entities=tuple(observation.get('entities',[]))\n"
    "action_space=tuple(observation.get('action_space',[]))\n"
    "has_kit='bridge-kit' in inventory\n"
    "raider_present=False\n"
    "for entity in entities:\n"
    "    if isinstance(entity,dict) and entity.get('type') == 'npc' and entity.get('name') == 'raider':\n"
    "        raider_present=True\n"
    "        break\n"
    "action=None\n"
    "if 'take storm-core' in action_space:\n"
    "    action='take storm-core'\n"
    "elif 'take bridge-kit' in action_space and not has_kit:\n"
    "    action='take bridge-kit'\n"
    "elif location == 'camp':\n"
    "    if not has_kit and 'move east' in action_space:\n"
    "        action='move east'\n"
    "    elif has_kit and 'move east' in action_space:\n"
    "        action='move east'\n"
    "    elif raider_present and 'move north' in action_space:\n"
    "        action='move north'\n"
    "elif location == 'supply-cache':\n"
    "    if has_kit and 'move north' in action_space:\n"
    "        action='move north'\n"
    "    elif 'move west' in action_space:\n"
    "        action='move west'\n"
    "elif location == 'bridge-approach':\n"
    "    if has_kit and 'move north' in action_space:\n"
    "        action='move north'\n"
    "    elif has_kit and 'use bridge-kit' in action_space:\n"
    "        action='use bridge-kit'\n"
    "    elif 'move south' in action_space:\n"
    "        action='move south'\n"
    "elif location == 'ember-pass':\n"
    "    if raider_present and 'attack raider' in action_space:\n"
    "        action='attack raider'\n"
    "    elif 'move north' in action_space:\n"
    "        action='move north'\n"
    "elif location == 'vault' and 'take storm-core' in action_space:\n"
    "    action='take storm-core'\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('take '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('use '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('move '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('attack '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    action='look' if 'look' in action_space else 'wait'\n"
    "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
)
_RUNNER_HAZARD_TRADEOFF_AGENT_SCRIPTS = {
    0: _RUNNER_AGENT_SCRIPT_HAZARD_TRADEOFF,
    1: _RUNNER_AGENT_SCRIPT_HAZARD_TRADEOFF,
}
_RUNNER_AGENT_SCRIPT_DELAYED_COST = (
    "import json,sys\n"
    "line=sys.stdin.readline()\n"
    "observation=json.loads(line)\n"
    "location=str(observation.get('location',''))\n"
    "inventory=tuple(observation.get('inventory',[]))\n"
    "action_space=tuple(observation.get('action_space',[]))\n"
    "has_cell='power-cell' in inventory\n"
    "action=None\n"
    "if 'take archive-ledger' in action_space:\n"
    "    action='take archive-ledger'\n"
    "elif 'take power-cell' in action_space and not has_cell:\n"
    "    action='take power-cell'\n"
    "elif location == 'camp':\n"
    "    if not has_cell and 'move east' in action_space:\n"
    "        action='move east'\n"
    "    elif has_cell and 'move east' in action_space:\n"
    "        action='move east'\n"
    "elif location == 'depot':\n"
    "    if has_cell and 'move east' in action_space:\n"
    "        action='move east'\n"
    "    elif 'move west' in action_space:\n"
    "        action='move west'\n"
    "elif location == 'service-tunnel':\n"
    "    if 'move north' in action_space:\n"
    "        action='move north'\n"
    "    elif 'move west' in action_space:\n"
    "        action='move west'\n"
    "elif location == 'vault-door':\n"
    "    if 'move north' in action_space:\n"
    "        action='move north'\n"
    "    elif has_cell and 'use power-cell' in action_space:\n"
    "        action='use power-cell'\n"
    "    elif 'move south' in action_space:\n"
    "        action='move south'\n"
    "elif location == 'tram-hub':\n"
    "    if 'move south' in action_space:\n"
    "        action='move south'\n"
    "elif location == 'vault' and 'take archive-ledger' in action_space:\n"
    "    action='take archive-ledger'\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('take '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('use '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('move '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    action='look' if 'look' in action_space else 'wait'\n"
    "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
)
_RUNNER_DELAYED_COST_AGENT_SCRIPTS = {
    0: _RUNNER_AGENT_SCRIPT_DELAYED_COST,
    1: _RUNNER_AGENT_SCRIPT_DELAYED_COST,
}
_RUNNER_AGENT_SCRIPT_CONTEXT_PRESSURE = (
    "import json,sys\n"
    "line=sys.stdin.readline()\n"
    "observation=json.loads(line)\n"
    "location=str(observation.get('location',''))\n"
    "inventory=tuple(observation.get('inventory',[]))\n"
    "action_space=tuple(observation.get('action_space',[]))\n"
    "has_cell='coolant-cell' in inventory\n"
    "has_valve='valve-handle' in inventory\n"
    "action=None\n"
    "if 'take archive-prism' in action_space:\n"
    "    action='take archive-prism'\n"
    "elif 'take coolant-cell' in action_space and not has_cell:\n"
    "    action='take coolant-cell'\n"
    "elif 'take valve-handle' in action_space and not has_valve:\n"
    "    action='take valve-handle'\n"
    "elif location == 'camp':\n"
    "    if not has_cell and 'move east' in action_space:\n"
    "        action='move east'\n"
    "    elif has_cell and not has_valve and 'move south' in action_space:\n"
    "        action='move south'\n"
    "    elif has_cell and has_valve and 'move south' in action_space:\n"
    "        action='move south'\n"
    "    elif 'move north' in action_space:\n"
    "        action='move north'\n"
    "elif location == 'depot':\n"
    "    if not has_cell and 'take coolant-cell' in action_space:\n"
    "        action='take coolant-cell'\n"
    "    elif 'move west' in action_space:\n"
    "        action='move west'\n"
    "elif location == 'workshop':\n"
    "    if not has_valve and 'take valve-handle' in action_space:\n"
    "        action='take valve-handle'\n"
    "    elif 'move east' in action_space:\n"
    "        action='move east'\n"
    "    elif 'move north' in action_space:\n"
    "        action='move north'\n"
    "elif location == 'service-bay':\n"
    "    if 'move north' in action_space:\n"
    "        action='move north'\n"
    "    elif has_valve and 'use valve-handle' in action_space:\n"
    "        action='use valve-handle'\n"
    "    elif 'move west' in action_space:\n"
    "        action='move west'\n"
    "elif location == 'spillway':\n"
    "    if 'move north' in action_space:\n"
    "        action='move north'\n"
    "    elif 'move south' in action_space:\n"
    "        action='move south'\n"
    "elif location == 'seal-door':\n"
    "    if 'move north' in action_space:\n"
    "        action='move north'\n"
    "    elif has_cell and 'use coolant-cell' in action_space:\n"
    "        action='use coolant-cell'\n"
    "    elif 'move south' in action_space:\n"
    "        action='move south'\n"
    "elif location == 'relay-hall':\n"
    "    if 'move south' in action_space:\n"
    "        action='move south'\n"
    "    elif 'attack marauder' in action_space:\n"
    "        action='attack marauder'\n"
    "    elif 'move north' in action_space:\n"
    "        action='move north'\n"
    "elif location == 'decoy-annex' and 'move west' in action_space:\n"
    "    action='move west'\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('take '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('use '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('move '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    for candidate in action_space:\n"
    "        if candidate.startswith('attack '):\n"
    "            action=candidate\n"
    "            break\n"
    "if action is None:\n"
    "    action='look' if 'look' in action_space else 'wait'\n"
    "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
)
_RUNNER_CONTEXT_PRESSURE_AGENT_SCRIPTS = {
    0: _RUNNER_AGENT_SCRIPT_CONTEXT_PRESSURE,
    1: _RUNNER_AGENT_SCRIPT_CONTEXT_PRESSURE,
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
    external_agent_command: Sequence[str] | None = None
    external_agent_commands_by_actor: Mapping[str, Sequence[str]] | None = None
    external_agent_actor_id: str | None = None
    persistent_agent_session: bool = False
    external_agent_timeout_seconds: float | None = None
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
        if self.external_agent_command is not None:
            if isinstance(self.external_agent_command, (str, bytes)) or not isinstance(
                self.external_agent_command, Sequence
            ):
                raise ValueError("external_agent_command must be a sequence of strings")
            normalized_command: list[str] = []
            for token in self.external_agent_command:
                if not isinstance(token, str) or not token:
                    raise ValueError("external_agent_command must contain non-empty strings")
                normalized_command.append(token)
            object.__setattr__(self, "external_agent_command", tuple(normalized_command))
        if self.external_agent_commands_by_actor is not None:
            if not isinstance(self.external_agent_commands_by_actor, Mapping):
                raise ValueError("external_agent_commands_by_actor must be a mapping")
            normalized_commands_by_actor: dict[str, tuple[str, ...]] = {}
            for actor_id, command in self.external_agent_commands_by_actor.items():
                if not isinstance(actor_id, str) or not actor_id:
                    raise ValueError("external_agent_commands_by_actor keys must be non-empty strings")
                if actor_id not in self.actor_ids:
                    raise ValueError("external_agent_commands_by_actor keys must be present in actor_ids")
                if isinstance(command, (str, bytes)) or not isinstance(command, Sequence):
                    raise ValueError("external_agent_commands_by_actor values must be sequences of strings")
                normalized_command = tuple(str(token) for token in command)
                if len(normalized_command) == 0 or any(token == "" for token in normalized_command):
                    raise ValueError("external_agent_commands_by_actor values must contain non-empty strings")
                normalized_commands_by_actor[actor_id] = normalized_command
            object.__setattr__(self, "external_agent_commands_by_actor", normalized_commands_by_actor)
        if self.external_agent_command is not None and self.external_agent_commands_by_actor is not None:
            raise ValueError("external_agent_command and external_agent_commands_by_actor cannot both be provided")
        if self.external_agent_actor_id is not None:
            if not isinstance(self.external_agent_actor_id, str) or not self.external_agent_actor_id:
                raise ValueError("external_agent_actor_id must be None or a non-empty string")
            if self.external_agent_command is None:
                raise ValueError("external_agent_actor_id requires external_agent_command")
            if self.external_agent_actor_id not in self.actor_ids:
                raise ValueError("external_agent_actor_id must be present in actor_ids")
        if not isinstance(self.persistent_agent_session, bool):
            raise ValueError("persistent_agent_session must be a boolean")
        if self.external_agent_timeout_seconds is not None and (
            not isinstance(self.external_agent_timeout_seconds, (int, float))
            or isinstance(self.external_agent_timeout_seconds, bool)
            or float(self.external_agent_timeout_seconds) <= 0.0
        ):
            raise ValueError("external_agent_timeout_seconds must be None or a positive number")
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
    runtime_telemetry_records: tuple[dict[str, Any], ...] = ()

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
            "runtime_telemetry": {
                "records": [dict(record) for record in self.runtime_telemetry_records],
                "summary_by_actor": _build_runtime_telemetry_summary(self.runtime_telemetry_records),
            },
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _extract_final_metric_sum_for_actor(
    result: BenchmarkRunnerResult,
    *,
    actor_id: str,
    metric_name: str,
) -> float:
    if not isinstance(actor_id, str) or not actor_id:
        raise ValueError("actor_id must be a non-empty string")
    if not isinstance(metric_name, str) or not metric_name:
        raise ValueError("metric_name must be a non-empty string")

    replay_events = result.replay_artifact.to_dict()["events"]
    snapshots = [event for event in replay_events if event["event_type"] == _RUNTIME_REPLAY_STATE_EVENT_TYPE]
    if len(snapshots) == 0:
        return 0.0

    final_state_json = snapshots[-1]["payload"]["state_json"]  # type: ignore[index]
    state_payload = json.loads(str(final_state_json))
    for actor_state in state_payload.get("agent_states", []):
        if actor_state.get("actor_id") != actor_id:
            continue
        for metric in actor_state.get("metrics", []):
            if metric.get("metric_name") == metric_name:
                return float(metric.get("value_sum", 0.0))
        return 0.0
    return 0.0


def build_playable_slice_comparison_entry(
    result: BenchmarkRunnerResult,
    *,
    mode: str,
    agent_identity: str,
    actor_id: str,
) -> dict[str, Any]:
    """Build one compact deterministic playable-slice comparison entry."""
    if not isinstance(result, BenchmarkRunnerResult):
        raise ValueError("result must be a BenchmarkRunnerResult")
    if not isinstance(mode, str) or not mode:
        raise ValueError("mode must be a non-empty string")
    if not isinstance(agent_identity, str) or not agent_identity:
        raise ValueError("agent_identity must be a non-empty string")
    if not isinstance(actor_id, str) or not actor_id:
        raise ValueError("actor_id must be a non-empty string")

    scorecard_payload = result.scorecard.to_dict()
    actor_scorecard = next(
        (actor for actor in scorecard_payload["actors"] if actor["actor_id"] == actor_id),
        None,
    )
    if actor_scorecard is None:
        raise ValueError(f"missing_actor_scorecard:{actor_id}")

    runtime_summary = next(
        (
            entry
            for entry in _build_runtime_telemetry_summary(result.runtime_telemetry_records)
            if entry["actor_id"] == actor_id
        ),
        None,
    )
    runtime_telemetry = None
    if runtime_summary is not None:
        runtime_telemetry = {
            "turn_count": int(runtime_summary["turn_count"]),
            "repair_used_count": int(runtime_summary["repair_used_count"]),
            "fail_closed_used_count": int(runtime_summary["fail_closed_used_count"]),
            "provider_request_count_total": int(runtime_summary["provider_request_count_total"]),
            "provider_latency_ms_total": float(runtime_summary["provider_latency_ms_total"]),
            "final_parse_status_counts": dict(runtime_summary["final_parse_status_counts"]),
            "failure_reasons": list(runtime_summary["failure_reasons"]),
        }

    return {
        "scenario_id": result.lifecycle_state.scenario_id,
        "mode": mode,
        "agent_identity": agent_identity,
        "actor_id": actor_id,
        "objective_completed": _extract_final_metric_sum_for_actor(
            result,
            actor_id=actor_id,
            metric_name="quest.completed",
        )
        >= 1.0,
        "aggregate_score": float(scorecard_payload["aggregate_score"]),
        "composite_score": float(actor_scorecard["composite_score"]),
        "runtime_telemetry": runtime_telemetry,
    }


def build_tiny_suite_baseline_report(
    results: Sequence[BenchmarkRunnerResult],
) -> dict[str, Any]:
    """Build a compact deterministic report for baseline runs across tiny scenarios."""
    if isinstance(results, (str, bytes)) or not isinstance(results, Sequence):
        raise ValueError("results must be a sequence of BenchmarkRunnerResult")

    entries: list[dict[str, Any]] = []
    scenario_ids: set[str] = set()
    benchmark_ids: set[str] = set()

    for result in results:
        if not isinstance(result, BenchmarkRunnerResult):
            raise ValueError("results must contain only BenchmarkRunnerResult instances")

        scenario_ids.add(result.lifecycle_state.scenario_id)
        benchmark_ids.add(result.run_manifest.benchmark_id)
        replay_ref = _find_replay_ref(result.replay_artifact_refs, ref_name="replay_artifact")
        scorecard_payload = result.scorecard.to_dict()
        actor_payloads = {
            str(actor["actor_id"]): actor for actor in scorecard_payload["actors"]
        }
        parity_ref = {
            "terminal_state_hash": result.replay_parity_artifact.terminal_state_hash,
            "applied_steps_hash": result.replay_parity_artifact.applied_steps_hash,
            "score_summary_hash": result.replay_parity_artifact.score_summary_hash,
        }

        for actor_id in result.run_manifest.actor_ids:
            actor_scorecard = actor_payloads.get(actor_id)
            if actor_scorecard is None:
                raise RuntimeError(f"scorecard missing configured actor: {actor_id}")
            entries.append(
                {
                    "scenario_id": result.lifecycle_state.scenario_id,
                    "agent_id": actor_id,
                    "aggregate_score": scorecard_payload["aggregate_score"],
                    "composite_score": actor_scorecard["composite_score"],
                    "normalized_metrics": actor_scorecard["normalized_metrics"],
                    "contributions": actor_scorecard["contributions"],
                    "replay_ref": replay_ref,
                    "parity_ref": parity_ref,
                }
            )

    entries.sort(key=lambda entry: (str(entry["scenario_id"]), str(entry["agent_id"])))
    return {
        "schema_version": "tiny_suite_baseline_report_v1",
        "benchmark_ids": sorted(benchmark_ids),
        "scenario_count": len(scenario_ids),
        "entry_count": len(entries),
        "entries": entries,
    }


def build_tiny_suite_comparison_report(
    results: Sequence[BenchmarkRunnerResult],
    *,
    baseline_agent_id: str,
    candidate_agent_id: str,
) -> dict[str, Any]:
    """Build a compact deterministic comparison report across tiny-suite scenarios."""
    baseline_report = build_tiny_suite_baseline_report(results)
    entries = baseline_report["entries"]

    if not isinstance(baseline_agent_id, str) or not baseline_agent_id:
        raise ValueError("baseline_agent_id must be a non-empty string")
    if not isinstance(candidate_agent_id, str) or not candidate_agent_id:
        raise ValueError("candidate_agent_id must be a non-empty string")
    if baseline_agent_id == candidate_agent_id:
        raise ValueError("baseline_agent_id and candidate_agent_id must differ")

    entries_by_scenario_agent = {
        (str(entry["scenario_id"]), str(entry["agent_id"])): entry
        for entry in entries
    }
    scenario_ids = sorted({str(entry["scenario_id"]) for entry in entries})

    comparisons: list[dict[str, Any]] = []
    baseline_total = 0.0
    candidate_total = 0.0
    for scenario_id in scenario_ids:
        baseline_entry = entries_by_scenario_agent.get((scenario_id, baseline_agent_id))
        candidate_entry = entries_by_scenario_agent.get((scenario_id, candidate_agent_id))
        if baseline_entry is None:
            raise ValueError(f"baseline agent missing from suite report: {baseline_agent_id}")
        if candidate_entry is None:
            raise ValueError(f"candidate agent missing from suite report: {candidate_agent_id}")

        baseline_score = float(baseline_entry["composite_score"])
        candidate_score = float(candidate_entry["composite_score"])
        baseline_total += baseline_score
        candidate_total += candidate_score
        comparisons.append(
            {
                "scenario_id": scenario_id,
                "baseline": baseline_entry,
                "candidate": candidate_entry,
                "composite_score_difference": candidate_score - baseline_score,
            }
        )

    scenario_count = len(comparisons)
    baseline_average = baseline_total / scenario_count if scenario_count > 0 else 0.0
    candidate_average = candidate_total / scenario_count if scenario_count > 0 else 0.0
    return {
        "schema_version": "tiny_suite_comparison_report_v1",
        "benchmark_ids": baseline_report["benchmark_ids"],
        "baseline_agent_id": baseline_agent_id,
        "candidate_agent_id": candidate_agent_id,
        "scenario_count": scenario_count,
        "comparisons": comparisons,
        "summary": {
            "baseline_composite_score_total": baseline_total,
            "candidate_composite_score_total": candidate_total,
            "composite_score_difference_total": candidate_total - baseline_total,
            "baseline_composite_score_average": baseline_average,
            "candidate_composite_score_average": candidate_average,
            "composite_score_difference_average": candidate_average - baseline_average,
        },
    }


def build_tiny_suite_external_comparison_report(
    baseline_results: Sequence[BenchmarkRunnerResult],
    external_results: Sequence[BenchmarkRunnerResult],
    *,
    compared_actor_id: str,
    external_agent_id: str,
) -> dict[str, Any]:
    """Build a compact deterministic comparison report for built-in vs external tiny-suite runs."""
    baseline_report = build_tiny_suite_baseline_report(baseline_results)
    external_report = build_tiny_suite_baseline_report(external_results)

    if not isinstance(compared_actor_id, str) or not compared_actor_id:
        raise ValueError("compared_actor_id must be a non-empty string")
    if not isinstance(external_agent_id, str) or not external_agent_id:
        raise ValueError("external_agent_id must be a non-empty string")
    if compared_actor_id == external_agent_id:
        raise ValueError("compared_actor_id and external_agent_id must differ")

    baseline_entries_by_scenario_agent = {
        (str(entry["scenario_id"]), str(entry["agent_id"])): entry
        for entry in baseline_report["entries"]
    }
    external_entries_by_scenario_agent = {
        (str(entry["scenario_id"]), str(entry["agent_id"])): entry
        for entry in external_report["entries"]
    }
    baseline_scenario_ids = sorted({str(entry["scenario_id"]) for entry in baseline_report["entries"]})
    external_scenario_ids = sorted({str(entry["scenario_id"]) for entry in external_report["entries"]})
    if baseline_scenario_ids != external_scenario_ids:
        raise ValueError("baseline_results and external_results must cover identical scenario_ids")

    comparisons: list[dict[str, Any]] = []
    baseline_total = 0.0
    candidate_total = 0.0
    for scenario_id in baseline_scenario_ids:
        baseline_entry = baseline_entries_by_scenario_agent.get((scenario_id, compared_actor_id))
        external_source_entry = external_entries_by_scenario_agent.get((scenario_id, compared_actor_id))
        if baseline_entry is None:
            raise ValueError(f"baseline actor missing from suite report: {compared_actor_id}")
        if external_source_entry is None:
            raise ValueError(f"external comparison actor missing from suite report: {compared_actor_id}")

        external_entry = dict(external_source_entry)
        external_entry["agent_id"] = external_agent_id

        baseline_score = float(baseline_entry["composite_score"])
        candidate_score = float(external_entry["composite_score"])
        baseline_total += baseline_score
        candidate_total += candidate_score
        comparisons.append(
            {
                "scenario_id": scenario_id,
                "baseline": baseline_entry,
                "candidate": external_entry,
                "composite_score_difference": candidate_score - baseline_score,
            }
        )

    scenario_count = len(comparisons)
    baseline_average = baseline_total / scenario_count if scenario_count > 0 else 0.0
    candidate_average = candidate_total / scenario_count if scenario_count > 0 else 0.0
    benchmark_ids = sorted(
        {str(benchmark_id) for benchmark_id in baseline_report["benchmark_ids"]}
        | {str(benchmark_id) for benchmark_id in external_report["benchmark_ids"]}
    )
    return {
        "schema_version": "tiny_suite_comparison_report_v1",
        "benchmark_ids": benchmark_ids,
        "baseline_agent_id": compared_actor_id,
        "candidate_agent_id": external_agent_id,
        "scenario_count": scenario_count,
        "comparisons": comparisons,
        "summary": {
            "baseline_composite_score_total": baseline_total,
            "candidate_composite_score_total": candidate_total,
            "composite_score_difference_total": candidate_total - baseline_total,
            "baseline_composite_score_average": baseline_average,
            "candidate_composite_score_average": candidate_average,
            "composite_score_difference_average": candidate_average - baseline_average,
        },
    }


def build_tiny_suite_external_profile_comparison_report(
    baseline_profile_results: Sequence[BenchmarkRunnerResult],
    candidate_profile_results: Sequence[BenchmarkRunnerResult],
    *,
    compared_actor_id: str,
    baseline_external_agent_id: str,
    candidate_external_agent_id: str,
) -> dict[str, Any]:
    """Build a compact deterministic comparison report for external profile vs external profile tiny-suite runs."""
    baseline_report = build_tiny_suite_baseline_report(baseline_profile_results)
    candidate_report = build_tiny_suite_baseline_report(candidate_profile_results)

    if not isinstance(compared_actor_id, str) or not compared_actor_id:
        raise ValueError("compared_actor_id must be a non-empty string")
    if not isinstance(baseline_external_agent_id, str) or not baseline_external_agent_id:
        raise ValueError("baseline_external_agent_id must be a non-empty string")
    if not isinstance(candidate_external_agent_id, str) or not candidate_external_agent_id:
        raise ValueError("candidate_external_agent_id must be a non-empty string")
    if baseline_external_agent_id == candidate_external_agent_id:
        raise ValueError("baseline_external_agent_id and candidate_external_agent_id must differ")

    baseline_entries_by_scenario_agent = {
        (str(entry["scenario_id"]), str(entry["agent_id"])): entry
        for entry in baseline_report["entries"]
    }
    candidate_entries_by_scenario_agent = {
        (str(entry["scenario_id"]), str(entry["agent_id"])): entry
        for entry in candidate_report["entries"]
    }
    baseline_scenario_ids = sorted({str(entry["scenario_id"]) for entry in baseline_report["entries"]})
    candidate_scenario_ids = sorted({str(entry["scenario_id"]) for entry in candidate_report["entries"]})
    if baseline_scenario_ids != candidate_scenario_ids:
        raise ValueError("baseline_profile_results and candidate_profile_results must cover identical scenario_ids")

    comparisons: list[dict[str, Any]] = []
    baseline_total = 0.0
    candidate_total = 0.0
    for scenario_id in baseline_scenario_ids:
        baseline_source_entry = baseline_entries_by_scenario_agent.get((scenario_id, compared_actor_id))
        candidate_source_entry = candidate_entries_by_scenario_agent.get((scenario_id, compared_actor_id))
        if baseline_source_entry is None:
            raise ValueError(f"baseline external profile actor missing from suite report: {compared_actor_id}")
        if candidate_source_entry is None:
            raise ValueError(f"candidate external profile actor missing from suite report: {compared_actor_id}")

        baseline_entry = dict(baseline_source_entry)
        baseline_entry["agent_id"] = baseline_external_agent_id
        candidate_entry = dict(candidate_source_entry)
        candidate_entry["agent_id"] = candidate_external_agent_id

        baseline_score = float(baseline_entry["composite_score"])
        candidate_score = float(candidate_entry["composite_score"])
        baseline_total += baseline_score
        candidate_total += candidate_score
        comparisons.append(
            {
                "scenario_id": scenario_id,
                "baseline": baseline_entry,
                "candidate": candidate_entry,
                "composite_score_difference": candidate_score - baseline_score,
            }
        )

    scenario_count = len(comparisons)
    baseline_average = baseline_total / scenario_count if scenario_count > 0 else 0.0
    candidate_average = candidate_total / scenario_count if scenario_count > 0 else 0.0
    benchmark_ids = sorted(
        {str(benchmark_id) for benchmark_id in baseline_report["benchmark_ids"]}
        | {str(benchmark_id) for benchmark_id in candidate_report["benchmark_ids"]}
    )
    return {
        "schema_version": "tiny_suite_comparison_report_v1",
        "benchmark_ids": benchmark_ids,
        "baseline_agent_id": baseline_external_agent_id,
        "candidate_agent_id": candidate_external_agent_id,
        "scenario_count": scenario_count,
        "comparisons": comparisons,
        "summary": {
            "baseline_composite_score_total": baseline_total,
            "candidate_composite_score_total": candidate_total,
            "composite_score_difference_total": candidate_total - baseline_total,
            "baseline_composite_score_average": baseline_average,
            "candidate_composite_score_average": candidate_average,
            "composite_score_difference_average": candidate_average - baseline_average,
        },
    }


def build_tiny_suite_mixed_external_comparison_report(
    shared_results: Sequence[BenchmarkRunnerResult],
    *,
    baseline_agent_id: str,
    external_actor_id: str,
    external_agent_id: str,
) -> dict[str, Any]:
    """Build a compact deterministic comparison report from mixed shared-run tiny-suite results."""
    shared_report = build_tiny_suite_baseline_report(shared_results)
    entries = shared_report["entries"]

    if not isinstance(baseline_agent_id, str) or not baseline_agent_id:
        raise ValueError("baseline_agent_id must be a non-empty string")
    if not isinstance(external_actor_id, str) or not external_actor_id:
        raise ValueError("external_actor_id must be a non-empty string")
    if not isinstance(external_agent_id, str) or not external_agent_id:
        raise ValueError("external_agent_id must be a non-empty string")
    if baseline_agent_id == external_actor_id:
        raise ValueError("baseline_agent_id and external_actor_id must differ")

    entries_by_scenario_agent = {
        (str(entry["scenario_id"]), str(entry["agent_id"])): entry
        for entry in entries
    }
    scenario_ids = sorted({str(entry["scenario_id"]) for entry in entries})

    comparisons: list[dict[str, Any]] = []
    baseline_total = 0.0
    candidate_total = 0.0
    for scenario_id in scenario_ids:
        baseline_entry = entries_by_scenario_agent.get((scenario_id, baseline_agent_id))
        external_source_entry = entries_by_scenario_agent.get((scenario_id, external_actor_id))
        if baseline_entry is None:
            raise ValueError(f"baseline agent missing from suite report: {baseline_agent_id}")
        if external_source_entry is None:
            raise ValueError(f"external actor missing from suite report: {external_actor_id}")

        candidate_entry = dict(external_source_entry)
        candidate_entry["agent_id"] = external_agent_id

        baseline_score = float(baseline_entry["composite_score"])
        candidate_score = float(candidate_entry["composite_score"])
        baseline_total += baseline_score
        candidate_total += candidate_score
        comparisons.append(
            {
                "scenario_id": scenario_id,
                "baseline": baseline_entry,
                "candidate": candidate_entry,
                "composite_score_difference": candidate_score - baseline_score,
            }
        )

    scenario_count = len(comparisons)
    baseline_average = baseline_total / scenario_count if scenario_count > 0 else 0.0
    candidate_average = candidate_total / scenario_count if scenario_count > 0 else 0.0
    return {
        "schema_version": "tiny_suite_comparison_report_v1",
        "benchmark_ids": shared_report["benchmark_ids"],
        "baseline_agent_id": baseline_agent_id,
        "candidate_agent_id": external_agent_id,
        "scenario_count": scenario_count,
        "comparisons": comparisons,
        "summary": {
            "baseline_composite_score_total": baseline_total,
            "candidate_composite_score_total": candidate_total,
            "composite_score_difference_total": candidate_total - baseline_total,
            "baseline_composite_score_average": baseline_average,
            "candidate_composite_score_average": candidate_average,
            "composite_score_difference_average": candidate_average - baseline_average,
        },
    }


def build_tiny_suite_mixed_external_profile_comparison_report(
    shared_results: Sequence[BenchmarkRunnerResult],
    *,
    baseline_actor_id: str,
    candidate_actor_id: str,
    baseline_external_agent_id: str,
    candidate_external_agent_id: str,
) -> dict[str, Any]:
    """Build a compact deterministic comparison report from shared-run external profile vs external profile results."""
    shared_report = build_tiny_suite_baseline_report(shared_results)
    entries = shared_report["entries"]

    if not isinstance(baseline_actor_id, str) or not baseline_actor_id:
        raise ValueError("baseline_actor_id must be a non-empty string")
    if not isinstance(candidate_actor_id, str) or not candidate_actor_id:
        raise ValueError("candidate_actor_id must be a non-empty string")
    if not isinstance(baseline_external_agent_id, str) or not baseline_external_agent_id:
        raise ValueError("baseline_external_agent_id must be a non-empty string")
    if not isinstance(candidate_external_agent_id, str) or not candidate_external_agent_id:
        raise ValueError("candidate_external_agent_id must be a non-empty string")
    if baseline_actor_id == candidate_actor_id:
        raise ValueError("baseline_actor_id and candidate_actor_id must differ")
    if baseline_external_agent_id == candidate_external_agent_id:
        raise ValueError("baseline_external_agent_id and candidate_external_agent_id must differ")

    entries_by_scenario_agent = {
        (str(entry["scenario_id"]), str(entry["agent_id"])): entry
        for entry in entries
    }
    scenario_ids = sorted({str(entry["scenario_id"]) for entry in entries})

    comparisons: list[dict[str, Any]] = []
    baseline_total = 0.0
    candidate_total = 0.0
    for scenario_id in scenario_ids:
        baseline_source_entry = entries_by_scenario_agent.get((scenario_id, baseline_actor_id))
        candidate_source_entry = entries_by_scenario_agent.get((scenario_id, candidate_actor_id))
        if baseline_source_entry is None:
            raise ValueError(f"baseline external profile actor missing from suite report: {baseline_actor_id}")
        if candidate_source_entry is None:
            raise ValueError(f"candidate external profile actor missing from suite report: {candidate_actor_id}")

        baseline_entry = dict(baseline_source_entry)
        baseline_entry["agent_id"] = baseline_external_agent_id
        candidate_entry = dict(candidate_source_entry)
        candidate_entry["agent_id"] = candidate_external_agent_id

        baseline_score = float(baseline_entry["composite_score"])
        candidate_score = float(candidate_entry["composite_score"])
        baseline_total += baseline_score
        candidate_total += candidate_score
        comparisons.append(
            {
                "scenario_id": scenario_id,
                "baseline": baseline_entry,
                "candidate": candidate_entry,
                "composite_score_difference": candidate_score - baseline_score,
            }
        )

    scenario_count = len(comparisons)
    baseline_average = baseline_total / scenario_count if scenario_count > 0 else 0.0
    candidate_average = candidate_total / scenario_count if scenario_count > 0 else 0.0
    return {
        "schema_version": "tiny_suite_comparison_report_v1",
        "benchmark_ids": shared_report["benchmark_ids"],
        "baseline_agent_id": baseline_external_agent_id,
        "candidate_agent_id": candidate_external_agent_id,
        "scenario_count": scenario_count,
        "comparisons": comparisons,
        "summary": {
            "baseline_composite_score_total": baseline_total,
            "candidate_composite_score_total": candidate_total,
            "composite_score_difference_total": candidate_total - baseline_total,
            "baseline_composite_score_average": baseline_average,
            "candidate_composite_score_average": candidate_average,
            "composite_score_difference_average": candidate_average - baseline_average,
        },
    }


@dataclass(frozen=True, slots=True)
class _ResolvedRunnerConfig:
    run_config: BenchmarkRunConfig
    external_agent_command: Sequence[str] | None = None
    external_agent_commands_by_actor: Mapping[str, Sequence[str]] | None = None
    external_agent_actor_id: str | None = None
    persistent_agent_session: bool = False
    external_agent_timeout_seconds: float | None = None
    normalization_profiles: Mapping[str, NormalizationProfile | Mapping[str, Any]] | None = None
    score_weights: Mapping[str, float] | None = None


@dataclass(frozen=True, slots=True)
class BenchmarkPlayableSession:
    """Minimal initialized benchmark runtime session for bounded local play surfaces."""

    scenario_initialization: ScenarioInitialization
    scenario_vars: Mapping[str, Any]
    world_state: DeterministicWorldStateManager
    controller: SimulationController


@dataclass(frozen=True, slots=True)
class SharedShardParticipantBinding:
    """Deterministic participant-to-shard identity binding for shared local shard loops."""

    actor_id: str
    account_id: str
    character_id: str
    session_id: str


@dataclass(frozen=True, slots=True)
class SharedShardLoopStepResult:
    """Compact deterministic step result for the shared shard loop."""

    step_index: int
    accepted_actions: tuple[tuple[str, str], ...]
    emitted_event_types: tuple[str, ...]
    active_actor_ids: tuple[str, ...]
    shard_mutation_generation: int
    world_tick_count: int
    world_tick_heartbeat: str
    world_npc_stance_phase: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "step_index": self.step_index,
            "accepted_actions": [
                {"actor_id": actor_id, "action": action}
                for actor_id, action in self.accepted_actions
            ],
            "emitted_event_types": list(self.emitted_event_types),
            "active_actor_ids": list(self.active_actor_ids),
            "shard_mutation_generation": self.shard_mutation_generation,
            "world_tick_count": self.world_tick_count,
            "world_tick_heartbeat": self.world_tick_heartbeat,
            "world_npc_stance_phase": self.world_npc_stance_phase,
        }


@dataclass(slots=True)
class SharedShardLoopSession:
    """Minimal in-process shared shard loop over one persistent world/controller pair."""

    shard_state: ShardState
    scenario_initialization: ScenarioInitialization
    scenario_vars: Mapping[str, Any]
    world_state: DeterministicWorldStateManager
    controller: SimulationController
    run_id: str
    participant_bindings: tuple[SharedShardParticipantBinding, ...]
    agent_actor_ids: tuple[str, ...] = ()
    external_agent_runners_by_actor: Mapping[str, LocalProcessRunner] = field(default_factory=dict)
    external_agent_session_manager: DeterministicLocalRunnerSessionManager = field(
        default_factory=DeterministicLocalRunnerSessionManager
    )

    @property
    def shard_id(self) -> str:
        return self.shard_state.shard_id

    @property
    def current_tick(self) -> int:
        return self.controller.run_state.step_index

    @property
    def max_steps(self) -> int:
        return self.controller.run_state.max_steps

    @property
    def world_tick_count(self) -> int:
        return self.shard_state.metadata.world_tick_count

    @property
    def world_npc_stance_phase(self) -> str:
        return self.shard_state.metadata.npc_stance_phase

    def get_participant_binding(self, actor_id: str) -> SharedShardParticipantBinding:
        for binding in self.participant_bindings:
            if binding.actor_id == actor_id:
                return binding
        raise ValueError(f"unknown_shared_shard_actor:{actor_id}")

    def is_agent_participant(self, actor_id: str) -> bool:
        self.get_participant_binding(actor_id)
        return actor_id in self.agent_actor_ids

    def is_external_agent_participant(self, actor_id: str) -> bool:
        self.get_participant_binding(actor_id)
        return actor_id in self.external_agent_runners_by_actor

    def session_is_active(self, actor_id: str) -> bool:
        binding = self.get_participant_binding(actor_id)
        return self.shard_state.get_session(binding.session_id).status == "active"

    def active_actor_ids(self) -> tuple[str, ...]:
        return tuple(
            binding.actor_id
            for binding in self.participant_bindings
            if self.session_is_active(binding.actor_id)
        )

    def get_observation(self, actor_id: str) -> Observation:
        if not self.session_is_active(actor_id):
            raise ValueError(f"inactive_shared_shard_session:{actor_id}")
        observation = build_observation_for_actor(
            self.world_state.get_snapshot(),
            actor_id=actor_id,
            run_id=self.run_id,
            step=self.current_tick,
            max_steps=self.max_steps,
            messages=(),
        )
        return Observation(
            run_id=observation.run_id,
            step=observation.step,
            location=observation.location,
            description=observation.description,
            exits=observation.exits,
            entities=observation.entities,
            inventory=observation.inventory,
            health=observation.health,
            messages=self.shard_state.get_world_tick_observation_messages(
                location=observation.location,
            ),
            action_space=self.shard_state.get_world_phase_filtered_action_space(
                location=observation.location,
                action_space=observation.action_space,
            ),
            remaining_steps=observation.remaining_steps,
            protocol_version=observation.protocol_version,
        )

    def open_participant_session(self, actor_id: str) -> None:
        binding = self.get_participant_binding(actor_id)
        try:
            session_record = self.shard_state.get_session(binding.session_id)
        except ValueError:
            session_record = None
        if session_record is not None:
            self.shard_state = self.shard_state.activate_session(binding.session_id)
            return
        self.shard_state = self.shard_state.open_character_session(
            session_id=binding.session_id,
            character_id=binding.character_id,
            account_id=binding.account_id,
        )

    def close_participant_session(self, actor_id: str) -> None:
        binding = self.get_participant_binding(actor_id)
        self.shard_state = self.shard_state.close_character_session(binding.session_id)

    def build_mock_agent_turn(self, actor_id: str) -> BenchmarkLLMTurnResult:
        if not self.is_agent_participant(actor_id):
            raise ValueError(f"shared_shard_actor_not_agent_controlled:{actor_id}")
        observation = self.get_observation(actor_id)
        return run_mock_benchmark_llm_turn(
            observation,
            actor_id=actor_id,
        )

    def request_external_agent_action(self, actor_id: str) -> ActionSubmission:
        if not self.is_external_agent_participant(actor_id):
            raise ValueError(f"shared_shard_actor_not_external_agent_controlled:{actor_id}")
        runner = self.external_agent_runners_by_actor[actor_id]
        observation = self.get_observation(actor_id)
        result = self.external_agent_session_manager.request_actions(
            (
                LocalRunnerSessionRequest(
                    actor_id=actor_id,
                    runner=runner,
                    observation=observation,
                    timeout_seconds=_RUNNER_AGENT_TIMEOUT_SECONDS,
                ),
            )
        )[0]
        if not result.success or result.action_submission is None:
            raise RuntimeError(
                "shared_shard_external_agent_failure:"
                f"{actor_id}:{result.error_type or 'runner_error'}:"
                f"{result.error_message or 'unknown_runner_failure'}"
            )
        return result.action_submission

    def close_external_agent_participants(self) -> None:
        self.external_agent_session_manager.close(tuple(self.external_agent_runners_by_actor.values()))

    def advance_tick(
        self,
        submitted_actions: Mapping[str, str] | None = None,
    ) -> SharedShardLoopStepResult:
        if self.controller.run_state.status is not RunStatus.RUNNING:
            raise ValueError("shared_shard_loop_not_running")
        normalized_actions = {} if submitted_actions is None else dict(submitted_actions)
        if not isinstance(normalized_actions, dict):
            raise ValueError("submitted_actions must be a mapping when provided")

        accepted_action_requests = []
        accepted_actions: list[tuple[str, str]] = []
        step_index = self.current_tick
        active_actor_ids = self.active_actor_ids()

        for actor_id in sorted(normalized_actions):
            if actor_id not in {binding.actor_id for binding in self.participant_bindings}:
                raise ValueError(f"unknown_shared_shard_actor:{actor_id}")
            if actor_id not in active_actor_ids:
                raise ValueError(f"inactive_shared_shard_session:{actor_id}")

        for actor_id in active_actor_ids:
            observation = self.get_observation(actor_id)
            selected_action = normalized_actions.get(actor_id, "wait")
            pipeline_result = run_action_pipeline(
                actor_id=actor_id,
                submission=ActionSubmission(action=selected_action),
                observation=observation,
            )
            if not pipeline_result.accepted or pipeline_result.action_request is None:
                raise ValueError(
                    f"shared_shard_action_rejected:{actor_id}:{pipeline_result.reason or 'action_rejected'}"
                )
            accepted_action_requests.append(pipeline_result.action_request)
            accepted_actions.append((actor_id, selected_action))

        step_outcome = self.controller.step(tuple(accepted_action_requests))
        self.shard_state = self.shard_state.advance_world_tick()
        return SharedShardLoopStepResult(
            step_index=step_index,
            accepted_actions=tuple(accepted_actions),
            emitted_event_types=tuple(event.event_type for event in step_outcome.emitted_events),
            active_actor_ids=active_actor_ids,
            shard_mutation_generation=self.shard_state.journal.last_committed_mutation_generation,
            world_tick_count=self.shard_state.metadata.world_tick_count,
            world_tick_heartbeat=self.shard_state.metadata.last_world_tick_heartbeat,
            world_npc_stance_phase=self.shard_state.metadata.npc_stance_phase,
        )


def _build_shared_shard_participant_binding(actor_id: str) -> SharedShardParticipantBinding:
    if not isinstance(actor_id, str) or not actor_id:
        raise ValueError("actor_id must be a non-empty string")
    return SharedShardParticipantBinding(
        actor_id=actor_id,
        account_id=f"acct-{actor_id}",
        character_id=f"char-{actor_id}",
        session_id=f"sess-{actor_id}",
    )


def build_playable_benchmark_session(
    *,
    scenario: Mapping[str, Any] | str,
    actor_ids: Sequence[str],
    run_id: str,
    run_seed: int | None = None,
    max_steps_override: int | None = None,
) -> BenchmarkPlayableSession:
    """Build an initialized deterministic runtime session without scorecard/replay orchestration."""
    scenario_load = load_scenario_definition(scenario)
    if not scenario_load.accepted or scenario_load.scenario is None:
        raise ValueError(f"scenario load rejected: {scenario_load.reason}")

    initialization = build_scenario_initialization(
        scenario_load.scenario,
        seed_override=run_seed,
    )
    max_steps = initialization.max_steps
    if max_steps_override is not None:
        max_steps = max_steps_override

    scenario_vars = _scenario_vars_to_dict(initialization.scenario_vars)
    world_config = _extract_world_config(initialization.scenario_vars)
    world_config = _apply_seed_variation_to_world_config(
        world_config=world_config,
        scenario_vars=scenario_vars,
        seed=initialization.run_seed,
    )
    world_state = _build_runner_world_state(
        start_room_id=initialization.start_room_id,
        actor_ids=actor_ids,
        seed=initialization.run_seed,
        world_config=world_config,
        scenario_vars=scenario_vars,
    )
    controller = SimulationController(
        world_state_manager=world_state,
        action_processor=BasicDeterministicActionProcessor(),
        event_logger=_InMemoryEventLogger(),
        seed=initialization.run_seed,
        max_steps=max_steps,
        run_id=run_id,
    )
    controller.initialize()
    return BenchmarkPlayableSession(
        scenario_initialization=initialization,
        scenario_vars=scenario_vars,
        world_state=world_state,
        controller=controller,
    )


def build_shared_shard_loop_session(
    *,
    scenario: Mapping[str, Any] | str,
    actor_ids: Sequence[str],
    run_id: str,
    shard_id: str = "shared-shard-local",
    run_seed: int | None = None,
    max_steps_override: int | None = None,
    enforce_reconciliation: bool = False,
    agent_actor_ids: Sequence[str] = (),
    external_agent_commands_by_actor: Mapping[str, Sequence[str]] | None = None,
    persistent_agent_session: bool = False,
) -> SharedShardLoopSession:
    """Build the first minimal in-process shared shard loop over one evolving world state."""
    if isinstance(actor_ids, (str, bytes)) or not isinstance(actor_ids, Sequence):
        raise ValueError("actor_ids must be a sequence of strings")
    normalized_actor_ids = tuple(str(actor_id) for actor_id in actor_ids)
    if len(normalized_actor_ids) < 2:
        raise ValueError("shared_shard_loop_requires_at_least_two_actors")
    if len(set(normalized_actor_ids)) != len(normalized_actor_ids):
        raise ValueError("shared_shard_loop_actor_ids_must_be_unique")
    if isinstance(agent_actor_ids, (str, bytes)) or not isinstance(agent_actor_ids, Sequence):
        raise ValueError("agent_actor_ids must be a sequence of strings")
    normalized_agent_actor_ids = tuple(str(actor_id) for actor_id in agent_actor_ids)
    if len(set(normalized_agent_actor_ids)) != len(normalized_agent_actor_ids):
        raise ValueError("shared_shard_loop_agent_actor_ids_must_be_unique")
    if any(actor_id not in normalized_actor_ids for actor_id in normalized_agent_actor_ids):
        raise ValueError("shared_shard_loop_agent_actor_ids_must_be_subset_of_actor_ids")
    if not isinstance(persistent_agent_session, bool):
        raise ValueError("persistent_agent_session must be a boolean")
    external_agent_runners_by_actor: dict[str, LocalProcessRunner] = {}
    if external_agent_commands_by_actor is not None:
        normalized_external_commands = _resolve_external_local_agent_commands_by_actor(
            external_agent_commands_by_actor
        )
        if any(actor_id not in normalized_actor_ids for actor_id in normalized_external_commands):
            raise ValueError("shared_shard_loop_external_agent_actor_ids_must_be_subset_of_actor_ids")
        if any(actor_id in normalized_agent_actor_ids for actor_id in normalized_external_commands):
            raise ValueError("shared_shard_loop_actor_id_conflicts_between_mock_and_external_agents")
        external_agent_runners_by_actor = {
            actor_id: LocalProcessRunner(command, persistent_session=persistent_agent_session)
            for actor_id, command in normalized_external_commands.items()
        }
    if not isinstance(shard_id, str) or not shard_id:
        raise ValueError("shard_id must be a non-empty string")

    playable_session = build_playable_benchmark_session(
        scenario=scenario,
        actor_ids=normalized_actor_ids,
        run_id=run_id,
        run_seed=run_seed,
        max_steps_override=max_steps_override,
    )
    shard_state = ShardState.create_empty(
        shard_id,
        world_ruleset_version="shared_shard_loop_v1",
        benchmark_engine_version=_BENCHMARK_VERSION,
        scheduler_policy_version="deterministic_local_shared_loop_v1",
    )
    if enforce_reconciliation:
        shard_state = shard_state.with_session_principal_reconciliation_enforced(True)

    participant_bindings = tuple(
        _build_shared_shard_participant_binding(actor_id)
        for actor_id in normalized_actor_ids
    )
    for binding in participant_bindings:
        shard_state = shard_state.register_account(account_id=binding.account_id)
        shard_state = shard_state.register_character(
            character_id=binding.character_id,
            identity_class=(
                "external_agent"
                if binding.actor_id in normalized_agent_actor_ids
                or binding.actor_id in external_agent_runners_by_actor
                else "human_player"
            ),
            owner_account_id=binding.account_id,
        )
        shard_state = shard_state.open_character_session(
            session_id=binding.session_id,
            character_id=binding.character_id,
            account_id=binding.account_id,
        )

    return SharedShardLoopSession(
        shard_state=shard_state,
        scenario_initialization=playable_session.scenario_initialization,
        scenario_vars=playable_session.scenario_vars,
        world_state=playable_session.world_state,
        controller=playable_session.controller,
        run_id=run_id,
        participant_bindings=participant_bindings,
        agent_actor_ids=normalized_agent_actor_ids,
        external_agent_runners_by_actor=external_agent_runners_by_actor,
    )


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
    runtime_telemetry_records: list[dict[str, Any]] = []
    with tempfile.TemporaryDirectory(prefix="mudbench-llm-runtime-telemetry-") as runtime_telemetry_dir:
        gateway_agent_configs = _build_gateway_agent_configs(
            run_config.actor_ids,
            script_policy=_resolve_agent_script_policy(scenario_vars),
            external_agent_command=resolved_config.external_agent_command,
            external_agent_commands_by_actor=resolved_config.external_agent_commands_by_actor,
            external_agent_actor_id=resolved_config.external_agent_actor_id,
            persistent_agent_session=resolved_config.persistent_agent_session,
            external_agent_timeout_seconds=resolved_config.external_agent_timeout_seconds,
            runtime_telemetry_dir=runtime_telemetry_dir,
        )
        session_manager = DeterministicLocalRunnerSessionManager()
        runtime_events: list[EventRecord] = []
        try:
            while lifecycle.state.status is BenchmarkLifecycleStatus.RUNNING:
                step = lifecycle.state.step_index
                gateway_step = drive_gateway_step(
                    snapshot=world_state.get_snapshot(),
                    run_id=run_config.run_id,
                    step=step,
                    max_steps=max_steps,
                    agent_configs=gateway_agent_configs,
                    session_manager=session_manager,
                )
                _raise_external_agent_runner_failure_if_present(
                    gateway_failures=gateway_step.failures,
                    external_agent_command=resolved_config.external_agent_command,
                    external_agent_commands_by_actor=resolved_config.external_agent_commands_by_actor,
                    external_agent_actor_id=resolved_config.external_agent_actor_id,
                )
                step_runtime_telemetry = _read_runtime_telemetry_records(
                    telemetry_dir=Path(runtime_telemetry_dir),
                    run_id=run_config.run_id,
                    step=step,
                    actor_ids=run_config.actor_ids,
                )
                runtime_telemetry_records.extend(step_runtime_telemetry)
                step_outcome = controller.step(gateway_step.accepted_action_requests)
                tracker.apply_signals(
                    _build_runtime_step_signals(
                        run_id=run_config.run_id,
                        step=step,
                        actor_ids=run_config.actor_ids,
                        accepted_action_count=step_outcome.processed_actions,
                        gateway_failures=gateway_step.failures,
                        emitted_events=step_outcome.emitted_events,
                        runtime_telemetry_records=step_runtime_telemetry,
                    )
                )
                runtime_events.extend(step_outcome.emitted_events)
                runtime_events.extend(_build_runtime_telemetry_events(step_runtime_telemetry))
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
                                    runtime_telemetry_summary=_build_runtime_telemetry_summary(
                                        runtime_telemetry_records
                                    ),
                                ),
                            }
                        ),
                    )
                )
                lifecycle.advance_step()
        finally:
            session_manager.close(tuple(config.runner for config in gateway_agent_configs))

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
        runtime_telemetry_records=tuple(runtime_telemetry_records),
    )


def _coerce_runner_config(
    config: BenchmarkRunnerConfig | BenchmarkRunConfig | Mapping[str, Any],
) -> _ResolvedRunnerConfig:
    if isinstance(config, BenchmarkRunConfig):
        return _ResolvedRunnerConfig(run_config=config)

    if isinstance(config, BenchmarkRunnerConfig):
        return _ResolvedRunnerConfig(
            run_config=config.to_run_config(),
            external_agent_command=config.external_agent_command,
            external_agent_commands_by_actor=config.external_agent_commands_by_actor,
            external_agent_actor_id=config.external_agent_actor_id,
            persistent_agent_session=config.persistent_agent_session,
            external_agent_timeout_seconds=config.external_agent_timeout_seconds,
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
    runtime_telemetry_records: Sequence[Mapping[str, Any]] = (),
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
    for record in runtime_telemetry_records:
        actor_id = record.get("actor_id")
        if not isinstance(actor_id, str) or not actor_id:
            continue
        signals.append(
            MetricSignal(
                run_id=run_id,
                step=step,
                actor_id=actor_id,
                metric_name="llm.runtime.turns",
                value=1.0,
            )
        )
        signals.append(
            MetricSignal(
                run_id=run_id,
                step=step,
                actor_id=actor_id,
                metric_name="llm.runtime.repair_used",
                value=1.0 if bool(record.get("repair_used")) else 0.0,
            )
        )
        signals.append(
            MetricSignal(
                run_id=run_id,
                step=step,
                actor_id=actor_id,
                metric_name="llm.runtime.fail_closed_used",
                value=1.0 if bool(record.get("fail_closed_used")) else 0.0,
            )
        )
        final_parse_status = record.get("final_parse_status")
        if isinstance(final_parse_status, str) and final_parse_status:
            signals.append(
                MetricSignal(
                    run_id=run_id,
                    step=step,
                    actor_id=actor_id,
                    metric_name=f"llm.runtime.final_parse_status.{final_parse_status}",
                    value=1.0,
                )
            )
        provider_request_count = record.get("provider_request_count")
        if isinstance(provider_request_count, int):
            signals.append(
                MetricSignal(
                    run_id=run_id,
                    step=step,
                    actor_id=actor_id,
                    metric_name="llm.runtime.provider_request_count",
                    value=float(provider_request_count),
                )
            )
        provider_latency_ms = record.get("provider_latency_ms")
        if isinstance(provider_latency_ms, (int, float)):
            signals.append(
                MetricSignal(
                    run_id=run_id,
                    step=step,
                    actor_id=actor_id,
                    metric_name="llm.runtime.provider_latency_ms",
                    value=float(provider_latency_ms),
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
    external_agent_command: Sequence[str] | None = None,
    external_agent_commands_by_actor: Mapping[str, Sequence[str]] | None = None,
    external_agent_actor_id: str | None = None,
    persistent_agent_session: bool = False,
    external_agent_timeout_seconds: float | None = None,
    runtime_telemetry_dir: str | None = None,
) -> tuple[StepDriverAgentConfig, ...]:
    external_command = _resolve_external_local_agent_command(external_agent_command)
    external_commands_by_actor = _resolve_external_local_agent_commands_by_actor(external_agent_commands_by_actor)
    script_map = _resolve_runner_agent_scripts(script_policy)
    timeout_seconds = (
        _RUNNER_AGENT_TIMEOUT_SECONDS
        if external_agent_timeout_seconds is None
        else float(external_agent_timeout_seconds)
    )
    configs: list[StepDriverAgentConfig] = []
    for idx, actor_id in enumerate(actor_ids):
        actor_external_command = external_commands_by_actor.get(actor_id)
        use_external_command = actor_external_command is not None or external_command is not None and (
            external_agent_actor_id is None or actor_id == external_agent_actor_id
        )
        if not use_external_command:
            script = script_map.get(idx % len(script_map), _RUNNER_AGENT_SCRIPT_EXPLORER)
            command = (sys.executable, "-c", script)
        else:
            command = actor_external_command or external_command
            command = _augment_runtime_telemetry_command(
                command=command,
                actor_id=actor_id,
                runtime_telemetry_dir=runtime_telemetry_dir,
            )
        configs.append(
            StepDriverAgentConfig(
                actor_id=actor_id,
                runner=LocalProcessRunner(
                    command,
                    persistent_session=use_external_command and persistent_agent_session,
                ),
                timeout_seconds=timeout_seconds,
            )
        )
    return tuple(configs)


def _resolve_external_local_agent_command(
    external_agent_command: Sequence[str] | None,
) -> tuple[str, ...] | None:
    if external_agent_command is None:
        return None

    normalized_command = tuple(external_agent_command)
    if len(normalized_command) == 0:
        raise ValueError("external_agent_command_empty")

    executable = normalized_command[0]
    resolved_path = shutil.which(executable)
    if resolved_path is None:
        executable_path = Path(executable)
        if not executable_path.exists():
            raise ValueError(f"external_agent_command_not_found:{executable}")
    return normalized_command


def _resolve_external_local_agent_commands_by_actor(
    external_agent_commands_by_actor: Mapping[str, Sequence[str]] | None,
) -> dict[str, tuple[str, ...]]:
    if external_agent_commands_by_actor is None:
        return {}
    normalized_commands_by_actor: dict[str, tuple[str, ...]] = {}
    for actor_id, command in external_agent_commands_by_actor.items():
        normalized_command = _resolve_external_local_agent_command(command)
        if normalized_command is not None:
            normalized_commands_by_actor[str(actor_id)] = normalized_command
    return normalized_commands_by_actor


def _augment_runtime_telemetry_command(
    *,
    command: Sequence[str],
    actor_id: str,
    runtime_telemetry_dir: str | None,
) -> tuple[str, ...]:
    normalized_command = tuple(command)
    if runtime_telemetry_dir is None or not _is_direct_provider_runner_command(normalized_command):
        return normalized_command
    return normalized_command + (
        "--telemetry-dir",
        runtime_telemetry_dir,
        "--telemetry-actor-id",
        actor_id,
    )


def _is_direct_provider_runner_command(command: Sequence[str]) -> bool:
    if len(command) < 2:
        return False
    return Path(command[1]).name == "direct_provider_runner.py"


def _read_runtime_telemetry_records(
    *,
    telemetry_dir: Path,
    run_id: str,
    step: int,
    actor_ids: Sequence[str],
) -> tuple[dict[str, Any], ...]:
    if not telemetry_dir.exists():
        return ()

    records: list[dict[str, Any]] = []
    for actor_id in sorted(actor_ids):
        record_path = telemetry_dir / (
            f"{run_id}__step_{step:06d}__{actor_id}.llm_runtime_telemetry.json"
        )
        if not record_path.exists():
            continue
        payload = json.loads(record_path.read_text(encoding="utf-8"))
        if not isinstance(payload, dict):
            raise ValueError(f"runtime_telemetry_invalid_payload:{record_path}")
        records.append(_normalize_runtime_telemetry_record(payload))
        record_path.unlink()
    return tuple(records)


def _normalize_runtime_telemetry_record(payload: Mapping[str, Any]) -> dict[str, Any]:
    actor_id = payload.get("actor_id")
    run_id = payload.get("run_id")
    step = payload.get("step")
    final_parse_status = payload.get("final_parse_status")
    if not isinstance(actor_id, str) or not actor_id:
        raise ValueError("runtime_telemetry_missing_actor_id")
    if not isinstance(run_id, str) or not run_id:
        raise ValueError("runtime_telemetry_missing_run_id")
    if not isinstance(step, int) or step < 0:
        raise ValueError("runtime_telemetry_invalid_step")
    if not isinstance(final_parse_status, str) or not final_parse_status:
        raise ValueError("runtime_telemetry_missing_final_parse_status")

    record: dict[str, Any] = {
        "telemetry_schema": str(payload.get("telemetry_schema", _LLM_RUNTIME_TELEMETRY_SCHEMA)),
        "actor_id": actor_id,
        "run_id": run_id,
        "step": step,
        "repair_used": bool(payload.get("repair_used")),
        "fail_closed_used": bool(payload.get("fail_closed_used")),
        "final_parse_status": final_parse_status,
    }
    failure_reason = payload.get("failure_reason")
    if isinstance(failure_reason, str) and failure_reason:
        record["failure_reason"] = failure_reason
    provider_name = payload.get("provider_name")
    if isinstance(provider_name, str) and provider_name:
        record["provider_name"] = provider_name
    provider_request_count = payload.get("provider_request_count")
    if isinstance(provider_request_count, int):
        record["provider_request_count"] = provider_request_count
    provider_latency_ms = payload.get("provider_latency_ms")
    if isinstance(provider_latency_ms, (int, float)):
        record["provider_latency_ms"] = round(float(provider_latency_ms), 3)
    return record


def _build_runtime_telemetry_events(
    runtime_telemetry_records: Sequence[Mapping[str, Any]],
) -> tuple[EventRecord, ...]:
    events: list[EventRecord] = []
    for record in runtime_telemetry_records:
        step = record.get("step")
        actor_id = record.get("actor_id")
        if not isinstance(step, int) or step < 0:
            continue
        if not isinstance(actor_id, str) or not actor_id:
            continue
        events.append(
            EventRecord(
                step_index=step,
                event_type=_LLM_RUNTIME_TELEMETRY_EVENT_TYPE,
                actor_id=actor_id,
                payload=normalize_payload(dict(record)),
            )
        )
    return tuple(events)


def _build_runtime_telemetry_summary(
    runtime_telemetry_records: Sequence[Mapping[str, Any]],
) -> list[dict[str, Any]]:
    summary_by_actor: dict[str, dict[str, Any]] = {}
    for record in runtime_telemetry_records:
        actor_id = record.get("actor_id")
        if not isinstance(actor_id, str) or not actor_id:
            continue
        entry = summary_by_actor.setdefault(
            actor_id,
            {
                "actor_id": actor_id,
                "turn_count": 0,
                "repair_used_count": 0,
                "fail_closed_used_count": 0,
                "provider_request_count_total": 0,
                "provider_latency_ms_total": 0.0,
                "final_parse_status_counts": {},
                "failure_reasons": [],
            },
        )
        entry["turn_count"] += 1
        if bool(record.get("repair_used")):
            entry["repair_used_count"] += 1
        if bool(record.get("fail_closed_used")):
            entry["fail_closed_used_count"] += 1
        final_parse_status = record.get("final_parse_status")
        if isinstance(final_parse_status, str) and final_parse_status:
            counts = entry["final_parse_status_counts"]
            counts[final_parse_status] = counts.get(final_parse_status, 0) + 1
        failure_reason = record.get("failure_reason")
        if isinstance(failure_reason, str) and failure_reason and failure_reason not in entry["failure_reasons"]:
            entry["failure_reasons"].append(failure_reason)
        provider_request_count = record.get("provider_request_count")
        if isinstance(provider_request_count, int):
            entry["provider_request_count_total"] += provider_request_count
        provider_latency_ms = record.get("provider_latency_ms")
        if isinstance(provider_latency_ms, (int, float)):
            entry["provider_latency_ms_total"] += float(provider_latency_ms)

    ordered_summary: list[dict[str, Any]] = []
    for actor_id in sorted(summary_by_actor):
        entry = dict(summary_by_actor[actor_id])
        entry["provider_latency_ms_total"] = round(float(entry["provider_latency_ms_total"]), 3)
        entry["failure_reasons"] = sorted(entry["failure_reasons"])
        entry["final_parse_status_counts"] = {
            key: entry["final_parse_status_counts"][key]
            for key in sorted(entry["final_parse_status_counts"])
        }
        ordered_summary.append(entry)
    return ordered_summary


def _raise_external_agent_runner_failure_if_present(
    *,
    gateway_failures: Sequence[Any],
    external_agent_command: Sequence[str] | None,
    external_agent_commands_by_actor: Mapping[str, Sequence[str]] | None,
    external_agent_actor_id: str | None,
) -> None:
    if external_agent_command is None and external_agent_commands_by_actor is None:
        return
    for failure in gateway_failures:
        if getattr(failure, "stage", None) != "runner":
            continue
        actor_id = getattr(failure, "actor_id", "unknown-actor")
        if external_agent_commands_by_actor is not None and actor_id in external_agent_commands_by_actor:
            reason = getattr(failure, "reason", "runner_error")
            detail = getattr(failure, "detail", None) or "unknown_runner_failure"
            raise RuntimeError(f"external_local_agent_runner_failure:{actor_id}:{reason}:{detail}")
        if external_agent_actor_id is not None and actor_id != external_agent_actor_id:
            continue
        reason = getattr(failure, "reason", "runner_error")
        detail = getattr(failure, "detail", None) or "unknown_runner_failure"
        raise RuntimeError(f"external_local_agent_runner_failure:{actor_id}:{reason}:{detail}")




def _resolve_runner_agent_scripts(script_policy: str | None) -> Mapping[int, str]:
    if script_policy == "memory_delayed_retrieval_v1":
        return _RUNNER_MEMORY_AGENT_SCRIPTS
    if script_policy == "partial_observability_v1":
        return _RUNNER_OBS_AGENT_SCRIPTS
    if script_policy == "planning-dependency":
        return _RUNNER_PLANNING_AGENT_SCRIPTS
    if script_policy == "social-trade-dependency":
        return _RUNNER_SOCIAL_TRADE_AGENT_SCRIPTS
    if script_policy == "guarded-relic-v1":
        return _RUNNER_GUARDED_RELIC_AGENT_SCRIPTS
    if script_policy == "hazard-tradeoff-v1":
        return _RUNNER_HAZARD_TRADEOFF_AGENT_SCRIPTS
    if script_policy == "delayed-cost-v1":
        return _RUNNER_DELAYED_COST_AGENT_SCRIPTS
    if script_policy == "context-pressure-v1":
        return _RUNNER_CONTEXT_PRESSURE_AGENT_SCRIPTS
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
    runtime_telemetry_summary: Sequence[Mapping[str, Any]] = (),
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
        "runtime_telemetry_summary": [dict(entry) for entry in runtime_telemetry_summary],
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


def _find_replay_ref(
    refs: Sequence[tuple[str, str]],
    *,
    ref_name: str,
) -> str:
    for name, ref in refs:
        if name == ref_name:
            return ref
    raise RuntimeError(f"missing replay ref: {ref_name}")
