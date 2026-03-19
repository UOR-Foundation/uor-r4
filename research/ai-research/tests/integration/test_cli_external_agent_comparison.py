from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

from cli.main import main


def _read_json_output(output: str) -> dict[str, object]:
    return json.loads(output.strip())


def _manifest_path_for(output_path: Path) -> Path:
    if output_path.suffix:
        return output_path.with_suffix(output_path.suffix + ".manifest.json")
    return Path(str(output_path) + ".manifest.json")


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
            "--agent-label",
            "deterministic-wrapper",
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
    assert payload["actor_ids"] == ["agent-a", "deterministic-wrapper"]
    report = payload["report"]
    assert report["schema_version"] == "tiny_suite_comparison_report_v1"
    assert report["baseline_agent_id"] == "agent-a"
    assert report["candidate_agent_id"] == "deterministic-wrapper"
    assert report["external_agent_label"] == "deterministic-wrapper"
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
        assert entry["candidate"]["agent_id"] == "deterministic-wrapper"
        assert entry["baseline"]["replay_ref"] == entry["candidate"]["replay_ref"]
        assert entry["baseline"]["parity_ref"] == entry["candidate"]["parity_ref"]
        expected_difference = (
            float(entry["candidate"]["composite_score"]) - float(entry["baseline"]["composite_score"])
        )
        assert entry["composite_score_difference"] == expected_difference
        difference_total += expected_difference

    assert report["summary"]["composite_score_difference_total"] == pytest.approx(difference_total)


def test_cli_tiny_suite_mixed_external_comparison_writes_labeled_manifest_context(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_output_path = tmp_path / "tiny-suite-baseline.json"
    output_path = tmp_path / "tiny-suite-mixed-comparison.json"
    manifest_path = _manifest_path_for(output_path)
    baseline_manifest_path = _manifest_path_for(baseline_output_path)

    baseline_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--output",
            "json",
            "--output-file",
            str(baseline_output_path),
        ]
    )
    capsys.readouterr()

    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/deterministic_rule_agent.py",
            "--agent-label",
            "deterministic-wrapper",
            "--external-agent-actor",
            "agent-b",
            "--output",
            "json",
            "--output-file",
            str(output_path),
        ]
    )
    captured = capsys.readouterr()

    assert baseline_exit == 0
    assert exit_code == 0
    stdout_payload = _read_json_output(captured.out)
    file_payload = _read_json_output(output_path.read_text(encoding="utf-8"))
    manifest_payload = _read_json_output(manifest_path.read_text(encoding="utf-8"))

    assert stdout_payload == file_payload
    assert file_payload["actor_ids"] == ["agent-a", "deterministic-wrapper"]
    assert file_payload["report"]["candidate_agent_id"] == "deterministic-wrapper"
    assert file_payload["report"]["external_agent_label"] == "deterministic-wrapper"
    assert manifest_payload["command_mode"] == "suite_comparison"
    assert manifest_payload["actor_ids"] == ["agent-a", "deterministic-wrapper"]
    assert manifest_payload["external_agent_label"] == "deterministic-wrapper"

    history_exit = main(["reports", "history", "--dir", str(tmp_path), "--output", "json"])
    history_payload = _read_json_output(capsys.readouterr().out)

    assert history_exit == 0
    assert history_payload["accepted"] is True
    assert history_payload["command"] == "reports_history"
    assert history_payload["artifact_count"] == 2
    assert [entry["manifest_path"] for entry in history_payload["history"]] == [
        str(baseline_manifest_path),
        str(manifest_path),
    ]
    assert history_payload["history"][0]["command_mode"] == "suite_baseline"
    assert history_payload["history"][0]["actor_ids"] == ["agent-a", "agent-b"]
    assert "external_agent_label" not in history_payload["history"][0]
    assert history_payload["history"][1]["command_mode"] == "suite_comparison"
    assert history_payload["history"][1]["actor_ids"] == ["agent-a", "deterministic-wrapper"]
    assert history_payload["history"][1]["external_agent_label"] == "deterministic-wrapper"
    assert any(entry["actor_id"] == "deterministic-wrapper" for entry in history_payload["leaderboard"])

    export_exit = main(["reports", "export", "--dir", str(tmp_path), "--output", "json"])
    export_payload = _read_json_output(capsys.readouterr().out)

    assert export_exit == 0
    assert export_payload["accepted"] is True
    assert export_payload["command"] == "reports_export"
    assert export_payload["viewmodel_version"] == "reports_export_viewmodel_v1"
    assert export_payload["artifact_count"] == 2
    assert export_payload["coverage"]["scenario_ids"] == [
        "tiny-delayed-retrieval",
        "tiny-fetch-quest",
        "tiny-hidden-key",
        "tiny-locked-path",
        "tiny-social-trade",
    ]
    assert export_payload["coverage"]["actor_ids"] == ["agent-a", "agent-b", "deterministic-wrapper"]
    assert export_payload["coverage"]["external_agent_labels"] == ["deterministic-wrapper"]
    assert export_payload["artifacts"] == [
        {
            "report_path": str(baseline_output_path),
            "manifest_path": str(baseline_manifest_path),
            "artifact_type": "suite_report_manifest_v1",
            "command_mode": "suite_baseline",
            "suite_id": "tiny",
            "benchmark_id": "mudbench-cli",
        },
        {
            "report_path": str(output_path),
            "manifest_path": str(manifest_path),
            "artifact_type": "suite_report_manifest_v1",
            "command_mode": "suite_comparison",
            "suite_id": "tiny",
            "benchmark_id": "mudbench-cli",
        },
    ]
    assert export_payload["history"] == history_payload["history"]
    assert export_payload["leaderboard"] == history_payload["leaderboard"]
