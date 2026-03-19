"""Mock LLM-style single-shot local wrapper example for MUDBench."""

from __future__ import annotations

import json
import sys
from typing import Any, Sequence


def _read_observation() -> dict[str, Any]:
    raw_line = sys.stdin.readline()
    if not raw_line:
        raise ValueError("missing_observation_payload")
    payload = json.loads(raw_line)
    if not isinstance(payload, dict):
        raise ValueError("observation_payload_must_be_object")
    return payload


def _build_prompt(observation: dict[str, Any]) -> str:
    action_space = observation.get("action_space", ())
    inventory = observation.get("inventory", ())
    entities = observation.get("entities", ())
    return "\n".join(
        (
            "You are a deterministic MUDBench wrapper.",
            f"location: {observation.get('location', '')}",
            f"description: {observation.get('description', '')}",
            f"inventory: {list(inventory)}",
            f"entities: {list(entities)}",
            f"action_space: {list(action_space)}",
            "Return JSON with one field named action.",
        )
    )


def _mock_model_completion(prompt: str, action_space: Sequence[str]) -> str:
    del prompt
    selected = "wait"
    for candidate in action_space:
        if candidate.startswith("take "):
            selected = candidate
            break
    else:
        for candidate in action_space:
            if candidate.startswith("move "):
                selected = candidate
                break
        else:
            if "look" in action_space:
                selected = "look"
            elif action_space:
                selected = action_space[0]
    return json.dumps({"action": selected}, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _parse_model_output(raw_output: str, action_space: Sequence[str]) -> str:
    payload = json.loads(raw_output)
    if not isinstance(payload, dict):
        raise ValueError("model_output_must_be_object")
    action = payload.get("action")
    if not isinstance(action, str) or not action:
        raise ValueError("model_output_missing_action")
    if action not in action_space:
        raise ValueError(f"model_output_action_not_in_action_space:{action}")
    return action


def main() -> int:
    try:
        observation = _read_observation()
        raw_action_space = observation.get("action_space", ())
        if not isinstance(raw_action_space, list) and not isinstance(raw_action_space, tuple):
            raise ValueError("action_space_must_be_sequence")
        action_space = tuple(str(action) for action in raw_action_space)
        prompt = _build_prompt(observation)
        raw_output = _mock_model_completion(prompt, action_space)
        action = _parse_model_output(raw_output, action_space)
        sys.stdout.write(json.dumps({"action": action}, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
        sys.stdout.write("\n")
        sys.stdout.flush()
        return 0
    except (ValueError, TypeError, json.JSONDecodeError) as exc:
        print(f"mock_llm_wrapper error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
