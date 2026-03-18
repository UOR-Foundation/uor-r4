from __future__ import annotations

import re

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle

_WORLD_CONFIG_JSON = (
    '{"items":[{"entity_id":"hidden-key","entity_type":"item","location":"vault"},'
    '{"entity_id":"decoy-note","entity_type":"item","location":"start"}],'
    '"rooms":{"hall":{"description":"A quiet connecting hall.","entities":[],'
    '"exits":{"east":"vault","west":"start"},"title":"Hall"},'
    '"start":{"description":"A small chamber with worn stone walls.","entities":[],'
    '"exits":{"east":"hall"},"title":"Start"},'
    '"vault":{"description":"A compact vault with dusty shelves.","entities":[],'
    '"exits":{"west":"hall"},"title":"Vault"}}}'
)


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "tiny-hidden-key",
        "title": "Tiny Hidden Key",
        "description": "Tiny partial-observability scenario with look-triggered hidden item reveal.",
        "start_room_id": "start",
        "max_steps": 6,
        "seed": 33,
        "version": "1.0",
        "scenario_vars": {
            "mode": "vertical-slice-observability",
            "agent_script_policy": "partial_observability_v1",
            "observation_policy": "look_reveals_hidden_items_v1",
            "hidden_item_ids_json": "[\"hidden-key\"]",
            "world_config_json": _WORLD_CONFIG_JSON,
        },
        "objectives": [
            {
                "objective_id": "collect-hidden-key",
                "objective_type": "collect_item",
                "target_id": "hidden-key",
                "required_count": 1,
            }
        ],
    }


def _runner_config() -> BenchmarkRunnerConfig:
    return BenchmarkRunnerConfig(
        run_id="e2e-third-tiny-run",
        benchmark_id="e2e-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-a", "agent-b"),
    )


def test_third_tiny_scenario_executes_end_to_end() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()
    event_types = [event["event_type"] for event in payload["replay_artifact"]["events"]]
    assert payload["lifecycle_state"]["status"] == "finalized"
    assert payload["lifecycle_state"]["scenario_id"] == "tiny-hidden-key"
    assert "action_move" in event_types
    assert "action_look" in event_types
    assert "action_take" in event_types


def test_third_tiny_scenario_requires_look_before_hidden_key_take() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]
    hidden_take = next(
        event
        for event in events
        if event["event_type"] == "action_take" and event["payload"]["item_id"] == "hidden-key"
    )
    prior_looks = [
        event
        for event in events
        if event["event_type"] == "action_look"
        and event.get("actor_id") == hidden_take.get("actor_id")
        and event["step_index"] < hidden_take["step_index"]
    ]
    assert len(prior_looks) > 0


def test_third_tiny_scenario_replay_scorecard_and_parity_are_valid() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()
    replay = payload["replay_artifact"]
    scorecard = payload["scorecard"]
    parity = payload["replay_parity_artifact"]
    refs = payload["replay_artifact_refs"]

    assert replay["envelope"]["scenario_id"] == "tiny-hidden-key"
    assert scorecard["metadata"]["scenario_id"] == "tiny-hidden-key"
    assert scorecard["aggregate_score"] > 0.0
    assert [entry["name"] for entry in refs] == ["replay_artifact", "replay_checksum"]
    for entry in refs:
        assert re.fullmatch(r"sha256:[0-9a-f]{64}", entry["ref"]) is not None
    for field_name in ("terminal_state_hash", "applied_steps_hash", "score_summary_hash"):
        assert re.fullmatch(r"[0-9a-f]{64}", parity[field_name]) is not None


def test_third_tiny_scenario_is_same_seed_deterministic() -> None:
    first = run_benchmark_lifecycle(_runner_config())
    second = run_benchmark_lifecycle(_runner_config())
    assert first.to_canonical_json() == second.to_canonical_json()
    assert first.replay_parity_artifact.to_canonical_json() == second.replay_parity_artifact.to_canonical_json()
