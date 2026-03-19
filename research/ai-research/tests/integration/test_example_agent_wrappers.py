from __future__ import annotations

import json
import sys

import pytest

from cli.main import main


def _read_json_output(output: str) -> dict[str, object]:
    return json.loads(output.strip())


def test_example_deterministic_rule_agent_runs_through_real_cli_runtime_path(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/deterministic_rule_agent.py",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-fetch-quest"
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["replay"]["event_count"] > 0
    assert payload["scorecard"]["aggregate_score"] > 0.0
