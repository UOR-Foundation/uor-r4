"""End-to-end vertical-slice integration test for tiny-fetch-quest scenario.

Exercises the full CLI -> runner -> gateway -> simulation -> replay -> scorecard
path with a 3-room world containing items and NPCs, using two deterministic
agents with distinct behavior profiles (explorer and cautious).
"""

from __future__ import annotations

import re

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle


_WORLD_CONFIG_JSON = (
    '{"items":[{"entity_id":"golden-key","entity_type":"item","location":"treasury"}],'
    '"npcs":[{"entity_id":"guard-1","entity_type":"npc","health":30,"location":"corridor"}],'
    '"rooms":{"corridor":{"description":"A narrow stone corridor.","entities":[],'
    '"exits":{"east":"treasury","west":"entrance"},"title":"Stone Corridor"},'
    '"entrance":{"description":"A dimly lit entrance hall.","entities":[],'
    '"exits":{"east":"corridor"},"title":"Entrance Hall"},'
    '"treasury":{"description":"A small treasury chamber.","entities":[],'
    '"exits":{"west":"corridor"},"title":"Treasury Chamber"}}}'
)


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "tiny-fetch-quest",
        "title": "Tiny Fetch Quest",
        "description": "Minimal 3-room scenario with item collection and NPC combat.",
        "start_room_id": "entrance",
        "max_steps": 5,
        "seed": 42,
        "version": "1.0",
        "scenario_vars": {
            "mode": "vertical-slice",
            "seed_variation_policy": "tiny_fetch_v1",
            "seed_variation_axis": "key_room",
            "seed_variation_values_json": "[\"treasury\",\"corridor\"]",
            "world_config_json": _WORLD_CONFIG_JSON,
        },
        "objectives": [
            {
                "objective_id": "collect-golden-key",
                "objective_type": "collect_item",
                "target_id": "golden-key",
                "required_count": 1,
            }
        ],
    }


def _runner_config(*, seed: int = 42) -> BenchmarkRunnerConfig:
    scenario = _scenario_payload()
    scenario["seed"] = seed
    return BenchmarkRunnerConfig(
        run_id="e2e-vertical-slice-run",
        benchmark_id="e2e-benchmark",
        scenario=scenario,
        actor_ids=("agent-a", "agent-b"),
    )


def test_e2e_vertical_slice_produces_non_trivial_events() -> None:
    """Verify event stream contains moves, takes, attacks — not just waits/looks."""
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]
    event_types = [e["event_type"] for e in events]

    assert "action_move" in event_types, "Expected at least one move event"
    assert "action_take" in event_types, "Expected at least one take event (golden-key)"
    assert "action_attack" in event_types, "Expected at least one attack event (guard-1)"
    assert "state_snapshot" in event_types
    assert "step_completed" in event_types


def test_e2e_vertical_slice_exercises_real_world_transitions() -> None:
    """Verify agents actually move between rooms and interact with entities."""
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]

    move_events = [e for e in events if e["event_type"] == "action_move"]
    assert len(move_events) >= 2, "Expected multiple room transitions"
    rooms_visited = set()
    for e in move_events:
        rooms_visited.add(e["payload"]["source_room_id"])
        rooms_visited.add(e["payload"]["destination_room_id"])
    assert len(rooms_visited) >= 2, "Expected visits to at least 2 distinct rooms"

    take_events = [e for e in events if e["event_type"] == "action_take"]
    assert len(take_events) >= 1
    assert take_events[0]["payload"]["item_id"] == "golden-key"

    attack_events = [e for e in events if e["event_type"] == "action_attack"]
    assert len(attack_events) >= 1
    assert attack_events[0]["payload"]["target_id"] == "guard-1"
    assert attack_events[0]["payload"]["damage"] == 10


def test_e2e_vertical_slice_produces_scorecard_with_nonzero_scores() -> None:
    """Verify scorecard reflects meaningful capability scores."""
    result = run_benchmark_lifecycle(_runner_config())
    scorecard = result.scorecard.to_dict()

    assert scorecard["aggregate_score"] > 0.0
    assert scorecard["metadata"]["scenario_id"] == "tiny-fetch-quest"
    assert scorecard["metadata"]["seed"] == 42
    assert scorecard["metadata"]["step_count"] == 5

    actor_scores = {a["actor_id"]: a for a in scorecard["actors"]}
    assert "agent-a" in actor_scores
    assert "agent-b" in actor_scores
    assert actor_scores["agent-a"]["composite_score"] > 0.0
    assert actor_scores["agent-b"]["composite_score"] > 0.0


def test_e2e_vertical_slice_emits_complete_artifact_chain() -> None:
    """Verify replay artifact, parity artifact, and refs are all present."""
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()

    assert result.lifecycle_state.status.value == "finalized"
    assert result.lifecycle_state.step_index == 5

    replay = payload["replay_artifact"]
    assert replay["envelope"]["schema_version"] == "1.0"
    assert replay["envelope"]["run_id"] == "e2e-vertical-slice-run"
    assert replay["envelope"]["scenario_id"] == "tiny-fetch-quest"
    assert len(replay["events"]) == 20  # 4 events per step × 5 steps

    refs = payload["replay_artifact_refs"]
    assert len(refs) == 2
    for entry in refs:
        assert re.fullmatch(r"sha256:[0-9a-f]{64}", entry["ref"]) is not None

    parity = payload["replay_parity_artifact"]
    assert parity["step_count"] == 5
    assert parity["terminal_step"] == 4
    for field in ("terminal_state_hash", "applied_steps_hash", "score_summary_hash"):
        assert re.fullmatch(r"[0-9a-f]{64}", parity[field]) is not None


def test_e2e_vertical_slice_deterministic_repeated_runs() -> None:
    """Two identical-seed runs must produce byte-identical artifacts."""
    first = run_benchmark_lifecycle(_runner_config())
    second = run_benchmark_lifecycle(_runner_config())

    assert first.to_canonical_json() == second.to_canonical_json()
    assert (
        first.replay_parity_artifact.to_canonical_json()
        == second.replay_parity_artifact.to_canonical_json()
    )
    assert (
        first.replay_parity_artifact.terminal_state_hash
        == second.replay_parity_artifact.terminal_state_hash
    )
    assert (
        first.replay_parity_artifact.applied_steps_hash
        == second.replay_parity_artifact.applied_steps_hash
    )


def test_e2e_vertical_slice_different_seed_produces_different_hash() -> None:
    """Different seeds should change canonical runtime state/behavior."""
    run_a = run_benchmark_lifecycle(_runner_config(seed=42))
    run_b = run_benchmark_lifecycle(_runner_config(seed=99))

    # Ensure world behavior/state differs, not just provenance metadata.
    events_a = run_a.replay_artifact.to_dict()["events"]
    events_b = run_b.replay_artifact.to_dict()["events"]
    first_take_a = next(event for event in events_a if event["event_type"] == "action_take")
    first_take_b = next(event for event in events_b if event["event_type"] == "action_take")
    assert first_take_a["step_index"] != first_take_b["step_index"]
    assert first_take_a["payload"]["room_id"] != first_take_b["payload"]["room_id"]

    assert (
        run_a.replay_parity_artifact.terminal_state_hash
        != run_b.replay_parity_artifact.terminal_state_hash
    )


def test_e2e_vertical_slice_agent_behavioral_differentiation() -> None:
    """Verify the two agents exhibit distinct behavior patterns."""
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]

    agent_a_types = [
        e["event_type"] for e in events if e.get("actor_id") == "agent-a"
    ]
    agent_b_types = [
        e["event_type"] for e in events if e.get("actor_id") == "agent-b"
    ]

    # agent-a (explorer): should take item and do lots of moving
    assert "action_take" in agent_a_types, "Explorer agent should take items"
    assert "action_move" in agent_a_types, "Explorer agent should move"

    # agent-b (cautious): should attack guard
    assert "action_attack" in agent_b_types, "Cautious agent should attack"
