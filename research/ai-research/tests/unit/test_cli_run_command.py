from __future__ import annotations

import json

import pytest

from cli.main import main


def _read_json_output(output: str) -> dict[str, object]:
    return json.loads(output.strip())


def test_cli_run_default_executes_real_runtime_path_and_emits_structured_output(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["run_id"] == "cli-run"
    assert payload["benchmark_id"] == "mudbench-cli"
    assert payload["scenario_id"] == "cli-minimal-scenario"
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["scorecard"]["metadata"]["scoring_version"] == "phase3-v1"
    assert payload["replay"]["schema_version"] == "1.0"
    assert payload["replay"]["event_count"] >= payload["lifecycle"]["step_count"] * 2


def test_cli_run_surfaces_replay_and_scorecard_references(capsys: pytest.CaptureFixture[str]) -> None:
    exit_code = main(["run"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    refs = payload["replay"]["artifact_refs"]
    assert [entry["name"] for entry in refs] == ["replay_artifact", "replay_checksum"]
    assert refs[0]["ref"] == refs[1]["ref"]
    assert payload["scorecard"]["aggregate_score"] >= 0.0
    assert payload["scorecard"]["aggregate_score"] <= 1.0
    parity = payload["replay"]["parity"]
    for hash_key in ("terminal_state_hash", "applied_steps_hash", "score_summary_hash"):
        assert isinstance(parity[hash_key], str)
        assert len(parity[hash_key]) == 64


def test_cli_run_supports_scenario_selection(capsys: pytest.CaptureFixture[str]) -> None:
    exit_code = main(["run", "--scenario", "phase4-runtime-replay"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["scenario_id"] == "phase4-runtime-replay-scenario"


def test_cli_run_supports_second_tiny_scenario_selection(capsys: pytest.CaptureFixture[str]) -> None:
    exit_code = main(["run", "--scenario", "tiny-delayed-retrieval"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["scenario_id"] == "tiny-delayed-retrieval"


def test_cli_run_supports_third_tiny_scenario_selection(capsys: pytest.CaptureFixture[str]) -> None:
    exit_code = main(["run", "--scenario", "tiny-hidden-key"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["scenario_id"] == "tiny-hidden-key"


def test_cli_run_supports_social_trade_tiny_scenario_selection(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--scenario", "tiny-social-trade"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["scenario_id"] == "tiny-social-trade"


def test_cli_run_is_deterministic_for_identical_invocation(capsys: pytest.CaptureFixture[str]) -> None:
    first_exit = main(["run"])
    first_output = capsys.readouterr().out
    second_exit = main(["run"])
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output


def test_cli_run_rejects_invalid_scenario_selection() -> None:
    with pytest.raises(SystemExit) as exc_info:
        main(["run", "--scenario", "does-not-exist"])
    assert exc_info.value.code == 2
