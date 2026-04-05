"""Concrete deterministic action processor for core Phase-1 actions."""

from __future__ import annotations

from typing import Any, Mapping, Sequence

from core.action_processor import ActionProcessingResult, ActionProcessor, ActionRequest
from core.event_logger import EventRecord, normalize_payload
from core.world_state_manager import WorldStateManager
from world.combat.combat_rules import DeterministicCombatRules
from world.items.item_rules import DeterministicItemRules
from world.rooms.movement_rules import DeterministicMovementRules
from world.rooms.room_graph import DeterministicRoomGraph

from .world_state import DeterministicWorldStateManager

_SUPPORTED_ACTION_TYPES = frozenset(
    {"wait", "look", "move", "take", "drop", "use", "give", "attack"}
)


class BasicDeterministicActionProcessor(ActionProcessor):
    """Deterministic processor for a minimal Phase-1 action set."""

    def __init__(self) -> None:
        self._movement_rules = DeterministicMovementRules()
        self._item_rules = DeterministicItemRules()
        self._combat_rules = DeterministicCombatRules(damage=10)

    def validate_action(self, action: ActionRequest) -> bool:
        return self._validation_error(action) is None

    def process_actions(
        self,
        actions: Sequence[ActionRequest],
        world_state: WorldStateManager,
        *,
        step_index: int,
    ) -> tuple[ActionProcessingResult, ...]:
        if not isinstance(world_state, DeterministicWorldStateManager):
            raise ValueError("world_state must be a DeterministicWorldStateManager")
        if not isinstance(step_index, int) or step_index < 0:
            raise ValueError("step_index must be a non-negative integer")

        ordered_actions = tuple(sorted(actions, key=_action_sort_key))
        results: list[ActionProcessingResult] = []
        for action in ordered_actions:
            validation_error = self._validation_error(action)
            if validation_error is not None:
                results.append(
                    _rejected_action_result(
                        step_index=step_index,
                        actor_id=action.actor_id,
                        action_type=action.action_type,
                        reason=validation_error,
                    )
                )
                continue

            if action.action_type == "wait":
                results.append(
                    ActionProcessingResult(
                        accepted=True,
                        events=(
                            EventRecord(
                                step_index=step_index,
                                event_type="action_wait",
                                actor_id=action.actor_id,
                                payload=normalize_payload({"result": "no_state_change"}),
                            ),
                        ),
                    )
                )
                continue

            if action.action_type == "look":
                results.append(self._resolve_look(action, world_state=world_state, step_index=step_index))
                continue

            if action.action_type == "move":
                results.append(self._resolve_move(action, world_state=world_state, step_index=step_index))
                continue

            if action.action_type == "attack":
                results.append(
                    self._resolve_attack(action, world_state=world_state, step_index=step_index)
                )
                continue

            if action.action_type in {"take", "drop", "use", "give"}:
                results.append(
                    self._resolve_item_action(action, world_state=world_state, step_index=step_index)
                )
                continue

            results.append(
                _rejected_action_result(
                    step_index=step_index,
                    actor_id=action.actor_id,
                    action_type=action.action_type,
                    reason="unsupported_action_type",
                )
            )

        return tuple(results)

    def _resolve_look(
        self,
        action: ActionRequest,
        *,
        world_state: DeterministicWorldStateManager,
        step_index: int,
    ) -> ActionProcessingResult:
        snapshot = world_state.get_snapshot()
        entities = _require_mapping(snapshot.get("entities"), field_name="world.entities")
        rooms = _require_mapping(snapshot.get("rooms"), field_name="world.rooms")

        actor_payload = entities.get(action.actor_id)
        if not isinstance(actor_payload, Mapping):
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason="actor_not_found",
            )

        location = actor_payload.get("location")
        if not isinstance(location, str) or not location:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason="actor_has_no_location",
            )

        room_payload = rooms.get(location)
        if not isinstance(room_payload, Mapping):
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason="source_room_missing",
            )

        exits = _require_mapping(room_payload.get("exits", {}), field_name="room.exits")
        visible_exits = tuple(sorted(str(direction) for direction in exits.keys()))
        event = EventRecord(
            step_index=step_index,
            event_type="action_look",
            actor_id=action.actor_id,
            payload=normalize_payload(
                {"location": location, "visible_exits": visible_exits}
            ),
        )
        reveal_key = f"reveal.{action.actor_id}.{location}"
        return ActionProcessingResult(
            accepted=True,
            events=(event,),
            world_delta=_world_delta_to_items({"scenario_vars": {reveal_key: 1}}),
        )

    def _resolve_move(
        self,
        action: ActionRequest,
        *,
        world_state: DeterministicWorldStateManager,
        step_index: int,
    ) -> ActionProcessingResult:
        args = _arguments_to_dict(action.arguments)
        if args is None:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason="invalid_arguments",
            )

        direction = args["direction"]
        room_graph = _room_graph_from_snapshot(world_state.get_snapshot())
        resolution = self._movement_rules.apply_move(
            world_state,
            room_graph,
            actor_id=action.actor_id,
            direction=direction,
        )

        if not resolution.success:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason=resolution.reason or "move_rejected",
            )

        event = EventRecord(
            step_index=step_index,
            event_type="action_move",
            actor_id=action.actor_id,
            payload=normalize_payload(
                {
                    "direction": direction,
                    "source_room_id": resolution.source_room_id,
                    "destination_room_id": resolution.destination_room_id,
                }
            ),
        )
        return ActionProcessingResult(
            accepted=True,
            events=(event,),
            world_delta=_world_delta_to_items(resolution.world_delta),
        )

    def _resolve_item_action(
        self,
        action: ActionRequest,
        *,
        world_state: DeterministicWorldStateManager,
        step_index: int,
    ) -> ActionProcessingResult:
        args = _arguments_to_dict(action.arguments)
        if args is None:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason="invalid_arguments",
            )

        item_id = args["item_id"]
        if action.action_type == "take":
            resolution = self._item_rules.apply_take(
                world_state, actor_id=action.actor_id, item_id=item_id
            )
        elif action.action_type == "drop":
            resolution = self._item_rules.apply_drop(
                world_state, actor_id=action.actor_id, item_id=item_id
            )
        elif action.action_type == "give":
            resolution = self._item_rules.apply_give(
                world_state,
                actor_id=action.actor_id,
                item_id=item_id,
                target_id=str(args["target_id"]),
            )
        else:
            resolution = self._item_rules.apply_use(
                world_state, actor_id=action.actor_id, item_id=item_id
            )

        if not resolution.success:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason=resolution.reason or "item_action_rejected",
            )

        payload: dict[str, Any] = {"item_id": item_id}
        if action.action_type == "give":
            payload["target_id"] = str(args["target_id"])
        if resolution.room_id is not None:
            payload["room_id"] = resolution.room_id
        if action.action_type == "use":
            payload["consumed"] = resolution.consumed

        event = EventRecord(
            step_index=step_index,
            event_type=f"action_{action.action_type}",
            actor_id=action.actor_id,
            payload=normalize_payload(payload),
        )
        events: list[EventRecord] = [event]
        metadata = {key: value for key, value in resolution.metadata}
        effect_id = metadata.get("effect_id")
        if isinstance(effect_id, str) and effect_id:
            unlock_payload = {"effect_id": effect_id, "item_id": item_id}
            if action.action_type == "give":
                unlock_payload["target_id"] = metadata.get("effect_target_id")
                unlock_payload["reward_item_id"] = metadata.get("effect_reward_item_id")
            else:
                unlock_payload["source_room_id"] = metadata.get("effect_source_room_id")
                unlock_payload["direction"] = metadata.get("effect_direction")
                unlock_payload["destination_room_id"] = metadata.get("effect_destination_room_id")
            events.append(
                EventRecord(
                    step_index=step_index,
                    event_type="dependency_unlocked",
                    actor_id=action.actor_id,
                    payload=normalize_payload(unlock_payload),
                )
            )
        return ActionProcessingResult(
            accepted=True,
            events=tuple(events),
            world_delta=_world_delta_to_items(resolution.world_delta),
        )

    def _resolve_attack(
        self,
        action: ActionRequest,
        *,
        world_state: DeterministicWorldStateManager,
        step_index: int,
    ) -> ActionProcessingResult:
        args = _arguments_to_dict(action.arguments)
        if args is None:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason="invalid_arguments",
            )

        target_id = args["target_id"]
        resolution = self._combat_rules.apply_attack(
            world_state,
            attacker_id=action.actor_id,
            target_id=target_id,
        )
        if not resolution.success:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason=resolution.reason or "attack_rejected",
            )

        payload = {
            "target_id": target_id,
            "damage": resolution.damage,
            "resulting_health": resolution.resulting_health,
            "room_id": resolution.room_id,
        }
        event = EventRecord(
            step_index=step_index,
            event_type="action_attack",
            actor_id=action.actor_id,
            payload=normalize_payload(payload),
        )
        return ActionProcessingResult(
            accepted=True,
            events=(event,),
            world_delta=_world_delta_to_items(resolution.world_delta),
        )

    @staticmethod
    def _validation_error(action: ActionRequest) -> str | None:
        if not isinstance(action.actor_id, str) or not action.actor_id:
            return "invalid_actor_id"
        if not isinstance(action.action_type, str) or action.action_type not in _SUPPORTED_ACTION_TYPES:
            return "unsupported_action_type"

        args = _arguments_to_dict(action.arguments)
        if args is None:
            return "invalid_arguments"

        if action.action_type in {"wait", "look"}:
            if args:
                return "unexpected_arguments"
            return None

        if action.action_type == "move":
            if set(args.keys()) != {"direction"}:
                return "move_requires_direction"
            if not isinstance(args["direction"], str) or not args["direction"]:
                return "move_direction_invalid"
            return None

        if action.action_type == "attack":
            if set(args.keys()) != {"target_id"}:
                return "attack_requires_target_id"
            if not isinstance(args["target_id"], str) or not args["target_id"]:
                return "target_id_invalid"
            return None

        if action.action_type == "give":
            if set(args.keys()) != {"item_id", "target_id"}:
                return "give_requires_item_and_target"
            if not isinstance(args["item_id"], str) or not args["item_id"]:
                return "item_id_invalid"
            if not isinstance(args["target_id"], str) or not args["target_id"]:
                return "target_id_invalid"
            return None

        if set(args.keys()) != {"item_id"}:
            return "item_action_requires_item_id"
        if not isinstance(args["item_id"], str) or not args["item_id"]:
            return "item_id_invalid"
        return None


def _action_sort_key(action: ActionRequest) -> tuple[str, str, tuple[tuple[str, str], ...]]:
    return (
        str(action.actor_id),
        str(action.action_type),
        tuple((str(key), repr(value)) for key, value in action.arguments),
    )


def _arguments_to_dict(arguments: Any) -> dict[str, Any] | None:
    if not isinstance(arguments, Sequence):
        return None
    if isinstance(arguments, (str, bytes)):
        return None

    parsed: dict[str, Any] = {}
    for item in arguments:
        if not isinstance(item, tuple) or len(item) != 2:
            return None
        key, value = item
        if not isinstance(key, str) or not key:
            return None
        if key in parsed:
            return None
        parsed[key] = value
    return parsed


def _require_mapping(value: Any, *, field_name: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ValueError(f"{field_name} must be a mapping")
    return value


def _room_graph_from_snapshot(snapshot: Mapping[str, Any]) -> DeterministicRoomGraph:
    rooms = _require_mapping(snapshot.get("rooms"), field_name="world.rooms")
    return DeterministicRoomGraph.from_dict({"rooms": dict(rooms)})


def _world_delta_to_items(delta: Mapping[str, Any] | None) -> tuple[tuple[str, Any], ...]:
    if delta is None:
        return ()
    return tuple(sorted(delta.items(), key=lambda item: item[0]))


def _rejected_action_result(
    *,
    step_index: int,
    actor_id: str,
    action_type: str,
    reason: str,
) -> ActionProcessingResult:
    normalized_actor_id = actor_id if isinstance(actor_id, str) and actor_id else None
    return ActionProcessingResult(
        accepted=False,
        events=(
            EventRecord(
                step_index=step_index,
                event_type="action_rejected",
                actor_id=normalized_actor_id,
                payload=normalize_payload({"action_type": action_type, "reason": reason}),
            ),
        ),
    )
