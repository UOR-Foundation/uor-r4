"""Local process runner skeleton for agent protocol exchange."""

from __future__ import annotations

import subprocess
from dataclasses import dataclass
from time import monotonic
from typing import Sequence

from agents.gateway.timeout_policy import classify_timeout_boundary_window, classify_timeout_expired
from agents.protocol.action import ActionSubmission, parse_action_submission_payload
from agents.protocol.observation import Observation, require_supported_protocol_version
from agents.protocol.serialization import canonical_json_dumps, json_loads_object


class LocalRunnerError(RuntimeError):
    """Base local runner failure."""


class LocalRunnerTimeoutError(LocalRunnerError):
    """Raised when the agent process exceeds the configured timeout."""


class LocalRunnerProtocolError(LocalRunnerError):
    """Raised when the agent process response is invalid."""


@dataclass(frozen=True, slots=True)
class LocalProcessRunner:
    """One-step local process bridge for observation/action exchange.

    This skeleton executes the configured command per request and exchanges
    newline-delimited JSON over stdin/stdout.
    """

    command: tuple[str, ...]

    def __init__(self, command: Sequence[str]) -> None:
        normalized_command = tuple(command)
        if not normalized_command:
            raise ValueError("LocalProcessRunner requires a non-empty command")
        object.__setattr__(self, "command", normalized_command)

    def request_action(self, observation: Observation, *, timeout_seconds: float = 1.0) -> ActionSubmission:
        """Send observation to local process and parse action response."""
        if timeout_seconds <= 0:
            raise ValueError("timeout_seconds must be greater than zero")

        try:
            require_supported_protocol_version(observation.protocol_version)
        except ValueError as exc:
            raise LocalRunnerProtocolError(f"incompatible_protocol_version:{exc}") from exc

        payload = canonical_json_dumps(observation.to_dict()) + "\n"
        started_at = monotonic()
        try:
            completed = subprocess.run(
                self.command,
                input=payload,
                capture_output=True,
                text=True,
                timeout=timeout_seconds,
                check=False,
            )
        except subprocess.TimeoutExpired as exc:
            timeout_decision = classify_timeout_expired(timeout_seconds=timeout_seconds)
            if not timeout_decision.timed_out:
                raise LocalRunnerProtocolError("timeout classification must be timed_out") from exc
            raise LocalRunnerTimeoutError(
                "Agent process timed out: "
                f"{timeout_decision.reason} "
                f"(elapsed_seconds={timeout_decision.elapsed_seconds:.6f}, "
                f"timeout_seconds={timeout_decision.timeout_seconds:.6f})"
            ) from exc
        elapsed_seconds = monotonic() - started_at
        boundary_decision = classify_timeout_boundary_window(
            elapsed_seconds=elapsed_seconds,
            timeout_seconds=timeout_seconds,
        )
        if boundary_decision.reason == "environment_sensitive_timeout_boundary_window":
            raise LocalRunnerProtocolError(
                "environment_sensitive_timeout_boundary:"
                f"{boundary_decision.reason}"
                f"(elapsed_seconds={boundary_decision.elapsed_seconds:.6f},"
                f"timeout_seconds={boundary_decision.timeout_seconds:.6f},"
                f"boundary_window_seconds={boundary_decision.boundary_window_seconds:.6f})"
            )

        if completed.returncode != 0:
            details = completed.stderr.strip()
            raise LocalRunnerProtocolError(
                f"Agent process exited with code {completed.returncode}: {details}"
            )

        response_line = _first_non_empty_line(completed.stdout)
        if response_line is None:
            raise LocalRunnerProtocolError("Agent process returned no action payload")

        try:
            payload_object = json_loads_object(response_line)
        except ValueError as exc:
            raise LocalRunnerProtocolError("Agent process returned invalid JSON payload") from exc

        try:
            return parse_action_submission_payload(payload_object)
        except ValueError as exc:
            raise LocalRunnerProtocolError(
                f"Agent process returned invalid action schema: {exc}"
            ) from exc


def _first_non_empty_line(value: str) -> str | None:
    for raw_line in value.splitlines():
        line = raw_line.strip()
        if line:
            return line
    return None
