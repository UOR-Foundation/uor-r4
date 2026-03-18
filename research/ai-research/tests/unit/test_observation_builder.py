from __future__ import annotations

import copy

import pytest

from agents.gateway.observation_builder import build_observation_for_actor
from agents.protocol.serialization import canonical_json_dumps


def _sample_snapshot() -> dict[str, object]:
    return {
        "tick": 7,
        "entities": {
            "agent-1": {
                "entity_type": "agent",
                "location": "room-a",
                "health": 87,
                "inventory": ["item-2", "item-2"],
            },
            "item-1": {"entity_type": "item", "location": "room-a"},
            "item-2": {"entity_type": "item", "location": None},
            "npc-1": {"entity_type": "npc", "location": "room-a"},
            "npc-2": {"entity_type": "npc", "location": "room-b"},
        },
        "rooms": {
            "room-a": {
                "description": "A stone chamber.",
                "exits": {"west": "room-c", "east": "room-b"},
                "entities_present": ["npc-1", "agent-1", "item-1"],
            },
            "room-b": {
                "description": "A narrow tunnel.",
                "exits": {"west": "room-a"},
                "entities_present": ["npc-2"],
            },
            "room-c": {
                "description": "A dark alcove.",
                "exits": {"east": "room-a"},
                "entities_present": [],
            },
        },
        "scenario_vars": {},
    }


def test_observation_builder_maps_snapshot_to_protocol_observation() -> None:
    observation = build_observation_for_actor(
        _sample_snapshot(),
        actor_id="agent-1",
        run_id="run-1",
        step=3,
        max_steps=10,
        messages=("You hear dripping water.",),
    )

    assert observation.run_id == "run-1"
    assert observation.step == 3
    assert observation.location == "room-a"
    assert observation.description == "A stone chamber."
    assert observation.exits == ("east", "west")
    assert tuple((entity.type, entity.name) for entity in observation.entities) == (
        ("item", "item-1"),
        ("npc", "npc-1"),
    )
    assert observation.inventory == ("item-2",)
    assert observation.health == 87
    assert observation.messages == ("You hear dripping water.",)
    assert observation.remaining_steps == 7
    assert observation.action_space == (
        "wait",
        "look",
        "move east",
        "move west",
        "take item-1",
        "drop item-2",
        "use item-2",
        "give item-2 npc-1",
        "attack npc-1",
    )


def test_observation_builder_defaults_health_and_does_not_mutate_snapshot() -> None:
    snapshot = _sample_snapshot()
    entities = snapshot["entities"]
    assert isinstance(entities, dict)
    agent_payload = entities["agent-1"]
    assert isinstance(agent_payload, dict)
    del agent_payload["health"]

    before = canonical_json_dumps(copy.deepcopy(snapshot))
    observation = build_observation_for_actor(
        snapshot,
        actor_id="agent-1",
        run_id="run-2",
        step=1,
        max_steps=4,
    )
    after = canonical_json_dumps(snapshot)

    assert observation.health == 100
    assert before == after


def test_observation_builder_is_deterministic_for_semantically_identical_snapshots() -> None:
    first = _sample_snapshot()
    second = _sample_snapshot()

    first_rooms = first["rooms"]
    second_rooms = second["rooms"]
    first_entities = first["entities"]
    second_entities = second["entities"]
    assert isinstance(first_rooms, dict) and isinstance(second_rooms, dict)
    assert isinstance(first_entities, dict) and isinstance(second_entities, dict)

    first_room_a = first_rooms["room-a"]
    second_room_a = second_rooms["room-a"]
    assert isinstance(first_room_a, dict) and isinstance(second_room_a, dict)
    first_room_a["exits"] = {"west": "room-c", "east": "room-b"}
    second_room_a["exits"] = {"east": "room-b", "west": "room-c"}
    first_room_a["entities_present"] = ["npc-1", "item-1", "agent-1"]
    second_room_a["entities_present"] = ["item-1", "agent-1", "npc-1"]

    first_agent = first_entities["agent-1"]
    second_agent = second_entities["agent-1"]
    assert isinstance(first_agent, dict) and isinstance(second_agent, dict)
    first_agent["inventory"] = ["item-2", "item-2"]
    second_agent["inventory"] = ["item-2"]

    first_observation = build_observation_for_actor(
        first, actor_id="agent-1", run_id="run-3", step=2, max_steps=9
    )
    second_observation = build_observation_for_actor(
        second, actor_id="agent-1", run_id="run-3", step=2, max_steps=9
    )

    assert canonical_json_dumps(first_observation.to_dict()) == canonical_json_dumps(
        second_observation.to_dict()
    )


def test_observation_builder_rejects_missing_actor() -> None:
    with pytest.raises(ValueError, match="actor_not_found"):
        build_observation_for_actor(
            _sample_snapshot(),
            actor_id="missing-agent",
            run_id="run-4",
            step=0,
            max_steps=5,
        )


def test_observation_builder_rejects_room_entity_without_entity_payload() -> None:
    snapshot = _sample_snapshot()
    rooms = snapshot["rooms"]
    assert isinstance(rooms, dict)
    room_a = rooms["room-a"]
    assert isinstance(room_a, dict)
    room_a["entities_present"] = ["agent-1", "missing-entity"]

    with pytest.raises(ValueError, match="room_entity_not_found:missing-entity"):
        build_observation_for_actor(
            snapshot,
            actor_id="agent-1",
            run_id="run-5",
            step=0,
            max_steps=5,
        )


def test_observation_builder_hides_configured_hidden_items_until_revealed() -> None:
    snapshot = _sample_snapshot()
    entities = snapshot["entities"]
    rooms = snapshot["rooms"]
    scenario_vars = snapshot["scenario_vars"]
    assert isinstance(entities, dict) and isinstance(rooms, dict) and isinstance(scenario_vars, dict)
    entities["hidden-key"] = {"entity_type": "item", "location": "room-a"}
    room_a = rooms["room-a"]
    assert isinstance(room_a, dict)
    room_a["entities_present"] = ["agent-1", "hidden-key", "npc-1"]
    scenario_vars["observation_policy"] = "look_reveals_hidden_items_v1"
    scenario_vars["hidden_item_ids_json"] = "[\"hidden-key\"]"

    unrevealed = build_observation_for_actor(
        snapshot,
        actor_id="agent-1",
        run_id="run-hidden-1",
        step=0,
        max_steps=5,
    )
    assert tuple((entity.type, entity.name) for entity in unrevealed.entities) == (("npc", "npc-1"),)
    assert "take hidden-key" not in unrevealed.action_space

    scenario_vars["reveal.agent-1.room-a"] = 1
    revealed = build_observation_for_actor(
        snapshot,
        actor_id="agent-1",
        run_id="run-hidden-1",
        step=1,
        max_steps=5,
    )
    assert ("item", "hidden-key") in tuple((entity.type, entity.name) for entity in revealed.entities)
    assert "take hidden-key" in revealed.action_space
