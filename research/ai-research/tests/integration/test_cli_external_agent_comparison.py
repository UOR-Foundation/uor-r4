from __future__ import annotations

import json
import sys

import pytest

from cli.main import main


def _read_json_output(output: str) -> dict[str, object]:
    return json.loads(output.strip())


def test_cli_tiny_suite_builtin_vs_external_comparison_emits_expected_report_shape(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/deterministic_rule_agent.py",
            "--output",
            "json",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)

    assert payload["accepted"] is True
    assert payload["suite_id"] == "tiny"
    assert payload["actor_ids"] == ["agent-a", "external-local-agent"]
    report = payload["report"]
    assert report["schema_version"] == "tiny_suite_comparison_report_v1"
    assert report["baseline_agent_id"] == "agent-a"
    assert report["candidate_agent_id"] == "external-local-agent"
    assert report["scenario_count"] == 5
    assert len(report["comparisons"]) == 5

    scenario_ids = [entry["scenario_id"] for entry in report["comparisons"]]
    assert scenario_ids == [
        "tiny-delayed-retrieval",
        "tiny-fetch-quest",
        "tiny-hidden-key",
        "tiny-locked-path",
        "tiny-social-trade",
    ]

    difference_total = 0.0
    for entry in report["comparisons"]:
        assert entry["baseline"]["agent_id"] == "agent-a"
        assert entry["candidate"]["agent_id"] == "external-local-agent"
        expected_difference = (
            float(entry["candidate"]["composite_score"]) - float(entry["baseline"]["composite_score"])
        )
        assert entry["composite_score_difference"] == expected_difference
        difference_total += expected_difference

    assert report["summary"]["composite_score_difference_total"] == pytest.approx(difference_total)


def test_cli_tiny_suite_mixed_builtin_vs_external_comparison_reflects_shared_run_results(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/deterministic_rule_agent.py",
            "--external-agent-actor",
            "agent-b",
            "--output",
            "json",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)

    assert payload["accepted"] is True
    assert payload["suite_id"] == "tiny"
    assert payload["actor_ids"] == ["agent-a", "external-local-agent"]
    report = payload["report"]
    assert report["schema_version"] == "tiny_suite_comparison_report_v1"
    assert report["baseline_agent_id"] == "agent-a"
    assert report["candidate_agent_id"] == "external-local-agent"
    assert report["scenario_count"] == 5
    assert len(report["comparisons"]) == 5

    scenario_ids = [entry["scenario_id"] for entry in report["comparisons"]]
    assert scenario_ids == [
        "tiny-delayed-retrieval",
        "tiny-fetch-quest",
        "tiny-hidden-key",
        "tiny-locked-path",
        "tiny-social-trade",
    ]

    difference_total = 0.0
    for entry in report["comparisons"]:
        assert entry["baseline"]["agent_id"] == "agent-a"
        assert entry["candidate"]["agent_id"] == "external-local-agent"
        assert entry["baseline"]["replay_ref"] == entry["candidate"]["replay_ref"]
        assert entry["baseline"]["parity_ref"] == entry["candidate"]["parity_ref"]
        expected_difference = (
            float(entry["candidate"]["composite_score"]) - float(entry["baseline"]["composite_score"])
        )
        assert entry["composite_score_difference"] == expected_difference
        difference_total += expected_difference

    assert report["summary"]["composite_score_difference_total"] == pytest.approx(difference_total)
