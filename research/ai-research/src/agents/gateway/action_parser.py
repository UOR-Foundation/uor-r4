"""Deterministic action-command parser for gateway submissions."""

from __future__ import annotations

from dataclasses import dataclass

from core.action_processor import ActionRequest, normalize_arguments

_NO_ARGUMENT_VERBS = frozenset({"wait", "look"})
_VERB_TO_ARGUMENT_KEY = {
    "move": "direction",
    "take": "item_id",
    "drop": "item_id",
    "use": "item_id",
    "attack": "target_id",
    "give": "item_id",
}
_SUPPORTED_VERBS = _NO_ARGUMENT_VERBS | frozenset(_VERB_TO_ARGUMENT_KEY.keys())


@dataclass(frozen=True, slots=True)
class ActionCommandParseResult:
    """Deterministic command-parse outcome."""

    accepted: bool
    action_request: ActionRequest | None = None
    reason: str | None = None


def parse_action_command(*, actor_id: str, action: str) -> ActionCommandParseResult:
    """Parse a strict canonical action command into an ActionRequest."""
    if not isinstance(actor_id, str) or not actor_id:
        raise ValueError("actor_id must be a non-empty string")
    if not isinstance(action, str):
        raise ValueError("action must be a string")

    command = action.strip()
    if not command:
        return _rejected_result("empty_command")

    spacing_reason = _validate_spacing(command)
    if spacing_reason is not None:
        return _rejected_result(spacing_reason)

    if command != command.lower():
        return _rejected_result("non_canonical_case")

    parts = command.split(" ")
    verb = parts[0]
    if verb not in _SUPPORTED_VERBS:
        return _rejected_result("unsupported_action_verb")

    if verb in _NO_ARGUMENT_VERBS:
        if len(parts) != 1:
            return _rejected_result("unexpected_argument")
        return _accepted_result(
            ActionRequest(actor_id=actor_id, action_type=verb, arguments=())
        )

    if verb == "give":
        if len(parts) < 3:
            return _rejected_result("missing_argument")
        if len(parts) > 3:
            return _rejected_result("too_many_arguments")
        return _accepted_result(
            ActionRequest(
                actor_id=actor_id,
                action_type=verb,
                arguments=normalize_arguments({"item_id": parts[1], "target_id": parts[2]}),
            )
        )

    if len(parts) == 1:
        return _rejected_result("missing_argument")
    if len(parts) > 2:
        return _rejected_result("too_many_arguments")

    argument_value = parts[1]
    argument_key = _VERB_TO_ARGUMENT_KEY[verb]
    return _accepted_result(
        ActionRequest(
            actor_id=actor_id,
            action_type=verb,
            arguments=normalize_arguments({argument_key: argument_value}),
        )
    )


def _validate_spacing(command: str) -> str | None:
    if "  " in command:
        return "non_canonical_spacing"

    for char in command:
        if char.isspace() and char != " ":
            return "non_canonical_spacing"
    return None


def _accepted_result(action_request: ActionRequest) -> ActionCommandParseResult:
    return ActionCommandParseResult(accepted=True, action_request=action_request)


def _rejected_result(reason: str) -> ActionCommandParseResult:
    return ActionCommandParseResult(accepted=False, reason=reason)
