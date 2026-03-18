from __future__ import annotations

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle

_WORLD_CONFIG_JSON = (
    '{"items":[{"entity_id":"brass-key","entity_type":"item","location":"key-chamber"},'
    '{"entity_id":"artifact","entity_type":"item","location":"treasure"}],'
    '"rooms":{"entry":{"description":"The entry chamber smells of salt and stone. Paths lead east to the locksmiths and south to a sealed gate.","entities":[],'
    '"exits":{"east":"key-chamber","south":"lock-ante"},"title":"Entry Chamber"},'
    '"key-chamber":{"description":"A cramped chamber with a brass key resting on an iron stand.","entities":[],'
    '"exits":{"west":"entry"},"title":"Key Chamber"},'
    '"lock-ante":{"description":"A narrow ante-chamber with a barred northern door and a passage south.","entities":[],'
    '"exits":{"south":"entry"},"title":"Lock Ante-Chamber"},'
    '"treasure":{"description":"A tiny vault lit by phosphor moss; a prized artifact lies within.","entities":[],'
    '"exits":{"south":"lock-ante"},"title":"Treasure Vault"}},"unlock_effects":[{"effect_id":"sealed_gate","item_id":"brass-key","source_room_id":"lock-ante","direction":"north","destination_room_id":"treasure","consume_item":false,"requires_actor_in_place":true}]}'
)


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "tiny-locked-path",
        "title": "Tiny Locked Path",
        "description": "Tiny planning scenario that forces a key → unlock → artifact dependency chain.",
        "start_room_id": "entry",
        "max_steps": 8,
        "seed": 55,
        "version": "1.0",
    "scenario_vars": {
        "mode": "planning-dependency",
        "agent_script_policy": "planning-dependency",
        "world_config_json": _WORLD_CONFIG_JSON,
    },
        "objectives": [
            {
                "objective_id": "collect-artifact",
                "objective_type": "collect_item",
                "target_id": "artifact",
                "required_count": 1,
            }
        ],
    }


def _runner_config() -> BenchmarkRunnerConfig:
    return BenchmarkRunnerConfig(
        run_id="e2e-locked-path-run",
        benchmark_id="e2e-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-a", "agent-b"),
    )


def test_locked_path_executes_and_emits_dependency_event() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]
    event_types = [event["event_type"] for event in events]

    assert result.lifecycle_state.status.value == "finalized"
    assert result.lifecycle_state.scenario_id == "tiny-locked-path"
    assert "action_move" in event_types
    assert "action_use" in event_types
    assert "dependency_unlocked" in event_types
    assert "action_take" in event_types


def test_locked_path_unlock_precedes_treasure_access() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]

    unlock_event = next(event for event in events if event["event_type"] == "dependency_unlocked")
    first_treasure_move = next(
        event
        for event in events
        if event["event_type"] == "action_move"
        and event["payload"].get("destination_room_id") == "treasure"
    )
    artifact_take = next(
        event
        for event in events
        if event["event_type"] == "action_take"
        and event["payload"].get("item_id") == "artifact"
    )

    assert first_treasure_move["step_index"] >= unlock_event["step_index"]
    assert artifact_take["step_index"] >= unlock_event["step_index"]


def test_locked_path_scorecard_parity_and_replay_are_valid() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()
    scorecard = payload["scorecard"]
    parity = payload["replay_parity_artifact"]

    assert scorecard["aggregate_score"] > 0.0
    assert scorecard["metadata"]["scenario_id"] == "tiny-locked-path"
    assert parity["step_count"] > 0
    for field in ("terminal_state_hash", "applied_steps_hash", "score_summary_hash"):
        assert isinstance(parity[field], str)
        assert len(parity[field]) == 64


def test_locked_path_same_seed_is_deterministic() -> None:
    first = run_benchmark_lifecycle(_runner_config())
    second = run_benchmark_lifecycle(_runner_config())

    assert first.to_canonical_json() == second.to_canonical_json()
    assert first.replay_parity_artifact.to_canonical_json() == second.replay_parity_artifact.to_canonical_json()
