from __future__ import annotations

import re

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle

_WORLD_CONFIG_JSON = (
    '{"items":[{"entity_id":"memory-token","entity_type":"item","location":"cache"},'
    '{"entity_id":"note-fragment","entity_type":"item","location":"start"}],'
    '"rooms":{"cache":{"description":"A quiet storage alcove.","entities":[],'
    '"exits":{"west":"junction"},"title":"Cache"},'
    '"junction":{"description":"A crossroads with worn stone marks.","entities":[],'
    '"exits":{"east":"cache","west":"start"},"title":"Junction"},'
    '"start":{"description":"A small starting chamber with etched markings.","entities":[],'
    '"exits":{"east":"junction"},"title":"Start"}}}'
)


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "tiny-delayed-retrieval",
        "title": "Tiny Delayed Retrieval",
        "description": "Tiny memory-focused scenario with delayed target retrieval.",
        "start_room_id": "start",
        "max_steps": 6,
        "seed": 24,
        "version": "1.0",
        "scenario_vars": {
            "mode": "vertical-slice-memory",
            "agent_script_policy": "memory_delayed_retrieval_v1",
            "world_config_json": _WORLD_CONFIG_JSON,
        },
        "objectives": [
            {
                "objective_id": "retrieve-memory-token",
                "objective_type": "collect_item",
                "target_id": "memory-token",
                "required_count": 1,
            }
        ],
    }


def _runner_config() -> BenchmarkRunnerConfig:
    return BenchmarkRunnerConfig(
        run_id="e2e-second-tiny-run",
        benchmark_id="e2e-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-a", "agent-b"),
    )


def test_second_tiny_scenario_executes_end_to_end() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()
    replay_events = payload["replay_artifact"]["events"]
    event_types = [event["event_type"] for event in replay_events]

    assert payload["lifecycle_state"]["status"] == "finalized"
    assert payload["lifecycle_state"]["scenario_id"] == "tiny-delayed-retrieval"
    assert event_types.count("step_completed") == 6
    assert event_types.count("state_snapshot") == 6
    assert "action_move" in event_types
    assert "action_take" in event_types
    assert "action_attack" not in event_types


def test_second_tiny_scenario_exercises_delayed_retrieval_behavior() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    replay_events = result.replay_artifact.to_dict()["events"]
    move_events = [event for event in replay_events if event["event_type"] == "action_move"]
    take_events = [event for event in replay_events if event["event_type"] == "action_take"]

    assert len(move_events) >= 3
    assert len(take_events) >= 1
    first_take = take_events[0]
    assert first_take["payload"]["item_id"] == "memory-token"
    assert first_take["payload"]["room_id"] == "cache"
    assert first_take["step_index"] >= 4


def test_second_tiny_scenario_emits_replay_scorecard_and_parity() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()

    replay = payload["replay_artifact"]
    parity = payload["replay_parity_artifact"]
    refs = payload["replay_artifact_refs"]
    scorecard = payload["scorecard"]

    assert replay["envelope"]["scenario_id"] == "tiny-delayed-retrieval"
    assert scorecard["metadata"]["scenario_id"] == "tiny-delayed-retrieval"
    assert scorecard["aggregate_score"] > 0.0
    assert [entry["name"] for entry in refs] == ["replay_artifact", "replay_checksum"]
    assert refs[0]["ref"] == refs[1]["ref"]
    for entry in refs:
        assert re.fullmatch(r"sha256:[0-9a-f]{64}", entry["ref"]) is not None
    for field_name in ("terminal_state_hash", "applied_steps_hash", "score_summary_hash"):
        assert re.fullmatch(r"[0-9a-f]{64}", parity[field_name]) is not None


def test_second_tiny_scenario_is_deterministic_for_same_seed() -> None:
    first = run_benchmark_lifecycle(_runner_config())
    second = run_benchmark_lifecycle(_runner_config())

    assert first.to_canonical_json() == second.to_canonical_json()
    assert first.replay_parity_artifact.to_canonical_json() == second.replay_parity_artifact.to_canonical_json()
