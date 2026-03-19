from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

from cli.main import main


def _read_json_output(output: str) -> dict[str, object]:
    return json.loads(output.strip())


def _write_external_agent_profile(tmp_path: Path, *, command: str) -> Path:
    profile_path = tmp_path / "example-agent-profile.json"
    profile_path.write_text(
        json.dumps(
            {
                "agent_id": "example-rule-profile",
                "display_name": "Example Rule Profile",
                "command": command,
            },
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=True,
        )
        + "\n",
        encoding="utf-8",
    )
    return profile_path


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


def test_example_deterministic_rule_agent_profile_runs_through_real_cli_runtime_path(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    profile_path = _write_external_agent_profile(
        tmp_path,
        command=f"{sys.executable} examples/agents/deterministic_rule_agent.py",
    )

    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "agent-a",
            "--agent-profile",
            str(profile_path),
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["external_agent_profile_id"] == "example-rule-profile"
    assert payload["external_agent_label"] == "Example Rule Profile"
