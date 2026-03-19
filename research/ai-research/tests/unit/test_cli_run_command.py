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


def _replay_path_for(output_path: Path) -> Path:
    if output_path.suffix:
        return output_path.with_suffix(output_path.suffix + ".replay.json")
    return Path(str(output_path) + ".replay.json")


def _write_deterministic_agent_script(tmp_path: Path) -> Path:
    script_path = tmp_path / "external_agent.py"
    script_path.write_text(
        (
            "import json\n"
            "import sys\n"
            "line = sys.stdin.readline()\n"
            "observation = json.loads(line)\n"
            "action_space = tuple(observation.get('action_space', ()))\n"
            "action = 'wait'\n"
            "for candidate in action_space:\n"
            "    if candidate.startswith('take '):\n"
            "        action = candidate\n"
            "        break\n"
            "else:\n"
            "    for candidate in action_space:\n"
            "        if candidate.startswith('move '):\n"
            "            action = candidate\n"
            "            break\n"
            "    else:\n"
            "        for candidate in action_space:\n"
            "            if candidate.startswith('attack '):\n"
            "                action = candidate\n"
            "                break\n"
            "        else:\n"
            "            action = 'look' if 'look' in action_space else 'wait'\n"
            "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
        ),
        encoding="utf-8",
    )
    return script_path


def _write_broken_persistent_agent_script(tmp_path: Path) -> Path:
    script_path = tmp_path / "broken_persistent_agent.py"
    script_path.write_text(
        (
            "import sys\n"
            "for line in sys.stdin:\n"
            "    sys.exit(1)\n"
        ),
        encoding="utf-8",
    )
    return script_path


def _write_external_agent_profile(
    tmp_path: Path,
    *,
    agent_id: str = "profile-agent",
    display_name: str = "Profile Wrapper",
    command: str,
    persistent_agent_session: bool = False,
    filename: str = "external-agent-profile.json",
) -> Path:
    profile_path = tmp_path / filename
    profile_path.write_text(
        json.dumps(
            {
                "agent_id": agent_id,
                "display_name": display_name,
                "command": command,
                "persistent_agent_session": persistent_agent_session,
            },
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=True,
        )
        + "\n",
        encoding="utf-8",
    )
    return profile_path


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


def test_cli_run_supports_scenario_file_loading(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--scenario-file", "scenarios/canonical/tiny_fetch_quest.json"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-fetch-quest"


def test_cli_run_rejects_missing_scenario_file_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--scenario-file", "scenarios/canonical/does-not-exist.json"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "run_rejected"
    assert str(payload["reason"]).startswith("scenario_file_read_failed:")


def test_cli_run_rejects_malformed_scenario_file_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    malformed_path = tmp_path / "bad-scenario.json"
    malformed_path.write_text("{not-json", encoding="utf-8")

    exit_code = main(["run", "--scenario-file", str(malformed_path)])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "run_rejected"
    assert str(payload["reason"]).startswith("scenario_file_invalid_json:")


def test_cli_run_external_local_agent_command_executes_deterministically(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_deterministic_agent_script(tmp_path)
    command = f"{sys.executable} {script_path}"

    first_exit = main(
        ["run", "--scenario", "tiny-fetch-quest", "--actor-id", "agent-a", "--agent-command", command]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        ["run", "--scenario", "tiny-fetch-quest", "--actor-id", "agent-a", "--agent-command", command]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-fetch-quest"


def test_cli_run_examples_deterministic_rule_agent_executes_deterministically(
    capsys: pytest.CaptureFixture[str],
) -> None:
    command = f"{sys.executable} examples/agents/deterministic_rule_agent.py"

    first_exit = main(
        ["run", "--scenario", "tiny-fetch-quest", "--actor-id", "agent-a", "--agent-command", command]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        ["run", "--scenario", "tiny-fetch-quest", "--actor-id", "agent-a", "--agent-command", command]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output


def test_cli_run_external_agent_label_surfaces_in_output(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_deterministic_agent_script(tmp_path)
    command = f"{sys.executable} {script_path}"

    first_exit = main(
        [
            "run",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "agent-a",
            "--agent-command",
            command,
            "--agent-label",
            "deterministic-wrapper",
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "run",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "agent-a",
            "--agent-command",
            command,
            "--agent-label",
            "deterministic-wrapper",
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["external_agent_label"] == "deterministic-wrapper"


def test_cli_run_agent_profile_executes_and_surfaces_profile_identity(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_deterministic_agent_script(tmp_path)
    profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="rule-profile",
        display_name="Rule Profile",
        command=f"{sys.executable} {script_path}",
    )

    first_exit = main(
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
    first_output = capsys.readouterr().out
    second_exit = main(
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
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["external_agent_profile_id"] == "rule-profile"
    assert payload["external_agent_label"] == "Rule Profile"


def test_cli_run_rejects_invalid_external_local_agent_command_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--agent-command", "/definitely/missing/mudbench-agent-binary"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "run_rejected"
    assert payload["reason"] == "external_agent_command_not_found:/definitely/missing/mudbench-agent-binary"


def test_cli_run_rejects_missing_agent_profile_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--agent-profile", "profiles/does-not-exist.json"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "run_rejected"
    assert str(payload["reason"]).startswith("agent_profile_read_failed:")


def test_cli_run_rejects_malformed_agent_profile_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    profile_path = tmp_path / "bad-profile.json"
    profile_path.write_text("{not-json", encoding="utf-8")

    exit_code = main(["run", "--agent-profile", str(profile_path)])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "run_rejected"
    assert str(payload["reason"]).startswith("json_payload_invalid:")


def test_cli_run_rejects_broken_persistent_agent_session_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_broken_persistent_agent_script(tmp_path)
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "agent-a",
            "--agent-command",
            f"{sys.executable} {script_path}",
            "--persistent-agent-session",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "run_rejected"
    assert "persistent_session_" in str(payload["reason"])


def test_cli_suite_tiny_emits_deterministic_structured_output(
    capsys: pytest.CaptureFixture[str],
) -> None:
    first_exit = main(["suite", "--suite", "tiny"])
    first_output = capsys.readouterr().out
    second_exit = main(["suite", "--suite", "tiny"])
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["suite_id"] == "tiny"
    assert payload["benchmark_id"] == "mudbench-cli"
    assert payload["actor_ids"] == ["agent-a", "agent-b"]
    assert payload["report"]["schema_version"] == "tiny_suite_baseline_report_v1"
    assert payload["report"]["scenario_count"] == 5
    assert payload["report"]["entry_count"] == 10


def test_cli_suite_tiny_comparison_emits_deterministic_structured_output(
    capsys: pytest.CaptureFixture[str],
) -> None:
    first_exit = main(
        ["suite", "--suite", "tiny", "--baseline-agent", "agent-a", "--candidate-agent", "agent-b"]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        ["suite", "--suite", "tiny", "--baseline-agent", "agent-a", "--candidate-agent", "agent-b"]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["suite_id"] == "tiny"
    assert payload["report"]["schema_version"] == "tiny_suite_comparison_report_v1"
    assert payload["report"]["baseline_agent_id"] == "agent-a"
    assert payload["report"]["candidate_agent_id"] == "agent-b"
    assert payload["report"]["scenario_count"] == 5
    assert len(payload["report"]["comparisons"]) == 5
    assert "composite_score_difference_total" in payload["report"]["summary"]


def test_cli_suite_tiny_comparison_rejects_unsupported_actor_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        ["suite", "--suite", "tiny", "--baseline-agent", "agent-c", "--candidate-agent", "agent-b"]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert payload["reason"] == "unsupported baseline_agent: agent-c"


def test_cli_suite_tiny_external_comparison_emits_deterministic_structured_output(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_deterministic_agent_script(tmp_path)
    command = f"{sys.executable} {script_path}"

    first_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            command,
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            command,
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["suite_id"] == "tiny"
    assert payload["actor_ids"] == ["agent-a", "external-local-agent"]
    assert payload["report"]["schema_version"] == "tiny_suite_comparison_report_v1"
    assert payload["report"]["baseline_agent_id"] == "agent-a"
    assert payload["report"]["candidate_agent_id"] == "external-local-agent"
    assert payload["report"]["scenario_count"] == 5
    assert len(payload["report"]["comparisons"]) == 5
    for entry in payload["report"]["comparisons"]:
        assert entry["baseline"]["agent_id"] == "agent-a"
        assert entry["candidate"]["agent_id"] == "external-local-agent"


def test_cli_suite_tiny_external_profile_comparison_emits_deterministic_structured_output(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_deterministic_agent_script(tmp_path)
    profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="rule-profile",
        display_name="Rule Profile",
        command=f"{sys.executable} {script_path}",
    )

    first_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-profile",
            str(profile_path),
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-profile",
            str(profile_path),
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["actor_ids"] == ["agent-a", "rule-profile"]
    assert payload["report"]["candidate_agent_id"] == "rule-profile"
    assert payload["report"]["external_agent_profile_id"] == "rule-profile"
    assert payload["report"]["external_agent_label"] == "Rule Profile"


def test_cli_suite_tiny_dual_external_profile_comparison_emits_deterministic_structured_output(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="baseline-rule-profile",
        display_name="Baseline Rule Profile",
        command=f"{sys.executable} examples/agents/deterministic_rule_agent.py",
        filename="baseline-agent-profile.json",
    )
    candidate_profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="candidate-mock-profile",
        display_name="Candidate Mock Profile",
        command=f"{sys.executable} examples/agents/mock_llm_wrapper.py",
        filename="candidate-agent-profile.json",
    )

    first_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--baseline-agent-profile",
            str(baseline_profile_path),
            "--candidate-agent-profile",
            str(candidate_profile_path),
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--baseline-agent-profile",
            str(baseline_profile_path),
            "--candidate-agent-profile",
            str(candidate_profile_path),
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["actor_ids"] == ["baseline-rule-profile", "candidate-mock-profile"]
    assert payload["report"]["baseline_agent_id"] == "baseline-rule-profile"
    assert payload["report"]["candidate_agent_id"] == "candidate-mock-profile"
    assert payload["report"]["baseline_external_agent_profile_id"] == "baseline-rule-profile"
    assert payload["report"]["candidate_external_agent_profile_id"] == "candidate-mock-profile"
    assert payload["report"]["baseline_external_agent_label"] == "Baseline Rule Profile"
    assert payload["report"]["candidate_external_agent_label"] == "Candidate Mock Profile"
    assert payload["report"]["scenario_count"] == 5
    assert len(payload["report"]["comparisons"]) == 5
    for entry in payload["report"]["comparisons"]:
        assert entry["baseline"]["agent_id"] == "baseline-rule-profile"
        assert entry["candidate"]["agent_id"] == "candidate-mock-profile"


def test_cli_suite_tiny_shared_dual_external_profile_comparison_emits_deterministic_structured_output(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="baseline-rule-profile",
        display_name="Baseline Rule Profile",
        command=f"{sys.executable} examples/agents/deterministic_rule_agent.py",
        filename="baseline-agent-profile.json",
    )
    candidate_profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="candidate-mock-profile",
        display_name="Candidate Mock Profile",
        command=f"{sys.executable} examples/agents/mock_llm_wrapper.py",
        filename="candidate-agent-profile.json",
    )

    first_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--baseline-agent-profile",
            str(baseline_profile_path),
            "--candidate-agent-profile",
            str(candidate_profile_path),
            "--external-agent-actor",
            "agent-b",
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--baseline-agent-profile",
            str(baseline_profile_path),
            "--candidate-agent-profile",
            str(candidate_profile_path),
            "--external-agent-actor",
            "agent-b",
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["actor_ids"] == ["baseline-rule-profile", "candidate-mock-profile"]
    assert payload["report"]["baseline_agent_id"] == "baseline-rule-profile"
    assert payload["report"]["candidate_agent_id"] == "candidate-mock-profile"
    assert payload["report"]["baseline_external_agent_profile_id"] == "baseline-rule-profile"
    assert payload["report"]["candidate_external_agent_profile_id"] == "candidate-mock-profile"
    for entry in payload["report"]["comparisons"]:
        assert entry["baseline"]["agent_id"] == "baseline-rule-profile"
        assert entry["candidate"]["agent_id"] == "candidate-mock-profile"
        assert entry["baseline"]["replay_ref"] == entry["candidate"]["replay_ref"]
        assert entry["baseline"]["parity_ref"] == entry["candidate"]["parity_ref"]


def test_cli_suite_tiny_external_comparison_rejects_invalid_external_command_machine_readably(
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
            "/definitely/missing/mudbench-agent-binary",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert payload["reason"] == "external_agent_command_not_found:/definitely/missing/mudbench-agent-binary"


def test_cli_suite_tiny_external_profile_rejects_missing_profile_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-profile",
            "profiles/does-not-exist.json",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert str(payload["reason"]).startswith("agent_profile_read_failed:")


def test_cli_suite_tiny_dual_external_profile_rejects_missing_profile_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="baseline-rule-profile",
        display_name="Baseline Rule Profile",
        command=f"{sys.executable} examples/agents/deterministic_rule_agent.py",
        filename="baseline-agent-profile.json",
    )

    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--baseline-agent-profile",
            str(baseline_profile_path),
            "--candidate-agent-profile",
            "profiles/missing-candidate-profile.json",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert str(payload["reason"]).startswith("agent_profile_read_failed:")


def test_cli_suite_tiny_dual_external_profile_rejects_malformed_profile_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="baseline-rule-profile",
        display_name="Baseline Rule Profile",
        command=f"{sys.executable} examples/agents/deterministic_rule_agent.py",
        filename="baseline-agent-profile.json",
    )
    malformed_profile_path = tmp_path / "bad-candidate-profile.json"
    malformed_profile_path.write_text("{not-json", encoding="utf-8")

    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--baseline-agent-profile",
            str(baseline_profile_path),
            "--candidate-agent-profile",
            str(malformed_profile_path),
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert str(payload["reason"]).startswith("json_payload_invalid:")


def test_cli_suite_tiny_mixed_external_comparison_emits_deterministic_structured_output(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_deterministic_agent_script(tmp_path)
    command = f"{sys.executable} {script_path}"

    first_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            command,
            "--agent-label",
            "deterministic-wrapper",
            "--external-agent-actor",
            "agent-b",
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            command,
            "--agent-label",
            "deterministic-wrapper",
            "--external-agent-actor",
            "agent-b",
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["suite_id"] == "tiny"
    assert payload["actor_ids"] == ["agent-a", "deterministic-wrapper"]
    assert payload["report"]["schema_version"] == "tiny_suite_comparison_report_v1"
    assert payload["report"]["baseline_agent_id"] == "agent-a"
    assert payload["report"]["candidate_agent_id"] == "deterministic-wrapper"
    assert payload["report"]["external_agent_label"] == "deterministic-wrapper"
    assert payload["report"]["scenario_count"] == 5
    assert len(payload["report"]["comparisons"]) == 5
    for entry in payload["report"]["comparisons"]:
        assert entry["baseline"]["agent_id"] == "agent-a"
        assert entry["candidate"]["agent_id"] == "deterministic-wrapper"
        assert entry["baseline"]["replay_ref"] == entry["candidate"]["replay_ref"]
        assert entry["baseline"]["parity_ref"] == entry["candidate"]["parity_ref"]


def test_cli_suite_output_file_rejects_unwritable_path_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["suite", "--suite", "tiny", "--output-file", str(tmp_path)])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert str(payload["reason"]).startswith("output_file_write_failed:")


def test_cli_suite_output_manifest_rejects_unwritable_path_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    output_path = tmp_path / "tiny-suite.json"
    manifest_path = _manifest_path_for(output_path)
    manifest_path.mkdir()

    exit_code = main(["suite", "--suite", "tiny", "--output", "json", "--output-file", str(output_path)])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert str(payload["reason"]).startswith("output_manifest_write_failed:")


def test_cli_suite_output_file_is_deterministic_for_identical_invocation(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    output_path = tmp_path / "tiny-suite.json"
    manifest_path = _manifest_path_for(output_path)
    replay_path = _replay_path_for(output_path)

    first_exit = main(["suite", "--suite", "tiny", "--output", "json", "--output-file", str(output_path)])
    first_stdout = capsys.readouterr().out
    first_file = output_path.read_text(encoding="utf-8")
    first_manifest = manifest_path.read_text(encoding="utf-8")
    first_replay = replay_path.read_text(encoding="utf-8")

    second_exit = main(["suite", "--suite", "tiny", "--output", "json", "--output-file", str(output_path)])
    second_stdout = capsys.readouterr().out
    second_file = output_path.read_text(encoding="utf-8")
    second_manifest = manifest_path.read_text(encoding="utf-8")
    second_replay = replay_path.read_text(encoding="utf-8")

    assert first_exit == 0
    assert second_exit == 0
    assert first_stdout == second_stdout
    assert first_file == second_file
    assert first_manifest == second_manifest
    assert first_replay == second_replay


def test_cli_reports_list_and_show_are_deterministic_for_identical_input(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    output_path = tmp_path / "tiny-suite.json"
    manifest_path = _manifest_path_for(output_path)

    write_exit = main(["suite", "--suite", "tiny", "--output", "json", "--output-file", str(output_path)])
    capsys.readouterr()

    first_list_exit = main(["reports", "list", "--dir", str(tmp_path)])
    first_list_output = capsys.readouterr().out
    second_list_exit = main(["reports", "list", "--dir", str(tmp_path)])
    second_list_output = capsys.readouterr().out

    first_show_exit = main(["reports", "show", "--manifest", str(manifest_path)])
    first_show_output = capsys.readouterr().out
    second_show_exit = main(["reports", "show", "--manifest", str(manifest_path)])
    second_show_output = capsys.readouterr().out

    assert write_exit == 0
    assert first_list_exit == 0
    assert second_list_exit == 0
    assert first_list_output == second_list_output
    assert first_show_exit == 0
    assert second_show_exit == 0
    assert first_show_output == second_show_output

    list_payload = _read_json_output(first_list_output)
    assert list_payload["accepted"] is True
    assert list_payload["command"] == "reports_list"
    assert list_payload["artifact_count"] == 1

    show_payload = _read_json_output(first_show_output)
    assert show_payload["accepted"] is True
    assert show_payload["command"] == "reports_show"
    assert show_payload["artifact"]["manifest_path"] == str(manifest_path)
    assert show_payload["artifact"]["report_path"] == str(output_path)


def test_cli_reports_history_is_deterministic_for_identical_input(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_output_path = tmp_path / "tiny-suite.json"
    external_output_path = tmp_path / "tiny-suite-mixed.json"

    write_baseline_exit = main(
        ["suite", "--suite", "tiny", "--output", "json", "--output-file", str(baseline_output_path)]
    )
    capsys.readouterr()
    write_external_exit = main(
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
            str(external_output_path),
        ]
    )
    capsys.readouterr()

    first_exit = main(["reports", "history", "--dir", str(tmp_path)])
    first_output = capsys.readouterr().out
    second_exit = main(["reports", "history", "--dir", str(tmp_path)])
    second_output = capsys.readouterr().out

    assert write_baseline_exit == 0
    assert write_external_exit == 0
    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["command"] == "reports_history"
    assert payload["artifact_count"] == 2
    assert len(payload["history"]) == 2
    assert len(payload["leaderboard"]) >= 2
    assert len(payload["identity_rollups"]) >= 2


def test_cli_reports_export_is_deterministic_for_identical_input(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_output_path = tmp_path / "tiny-suite.json"
    external_output_path = tmp_path / "tiny-suite-mixed.json"

    write_baseline_exit = main(
        ["suite", "--suite", "tiny", "--output", "json", "--output-file", str(baseline_output_path)]
    )
    capsys.readouterr()
    write_external_exit = main(
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
            str(external_output_path),
        ]
    )
    capsys.readouterr()

    first_exit = main(["reports", "export", "--dir", str(tmp_path)])
    first_output = capsys.readouterr().out
    second_exit = main(["reports", "export", "--dir", str(tmp_path)])
    second_output = capsys.readouterr().out

    assert write_baseline_exit == 0
    assert write_external_exit == 0
    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["command"] == "reports_export"
    assert payload["viewmodel_version"] == "reports_export_viewmodel_v1"
    assert payload["artifact_count"] == 2
    assert len(payload["artifacts"]) == 2
    assert len(payload["history"]) == 2
    assert len(payload["leaderboard"]) >= 2
    assert len(payload["identity_rollups"]) >= 2
    assert len(payload["replay_drilldowns"]) == 2
    assert payload["coverage"]["scenario_ids"] == [
        "tiny-delayed-retrieval",
        "tiny-fetch-quest",
        "tiny-hidden-key",
        "tiny-locked-path",
        "tiny-social-trade",
    ]
    assert payload["coverage"]["external_agent_labels"] == ["deterministic-wrapper"]
    assert payload["replay_drilldowns"][0]["replay_run_count"] == 5


def test_cli_reports_history_surfaces_unlabeled_raw_external_agent_gracefully(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    output_path = tmp_path / "tiny-suite-external.json"

    write_exit = main(
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
            "--output-file",
            str(output_path),
        ]
    )
    capsys.readouterr()

    history_exit = main(["reports", "history", "--dir", str(tmp_path)])
    history_payload = _read_json_output(capsys.readouterr().out)
    export_exit = main(["reports", "export", "--dir", str(tmp_path)])
    export_payload = _read_json_output(capsys.readouterr().out)

    assert write_exit == 0
    assert history_exit == 0
    assert export_exit == 0
    assert history_payload["history"][0]["actor_ids"] == ["agent-a", "external-local-agent"]
    assert history_payload["history"][0]["identity_summary"] == [
        {"actor_id": "agent-a", "identity_type": "built_in_actor"},
        {"actor_id": "external-local-agent", "identity_type": "external_agent_command"},
    ]
    assert any(
        entry["actor_id"] == "external-local-agent" and entry["identity_type"] == "external_agent_command"
        for entry in history_payload["leaderboard"]
    )
    assert any(
        entry["identity_value"] == "external-local-agent"
        and entry["identity_type"] == "external_agent_command"
        and entry["comparison_artifact_count"] == 1
        and entry["has_comparison_artifacts"] is True
        for entry in history_payload["identity_rollups"]
    )
    assert export_payload["coverage"]["external_agent_labels"] == []
    assert export_payload["coverage"]["external_agent_profile_ids"] == []
    assert any(
        entry["identity_value"] == "external-local-agent"
        and entry["identity_type"] == "external_agent_command"
        and "has_shared_run_arena_artifacts" not in entry
        for entry in export_payload["identity_rollups"]
    )


def test_cli_reports_rejects_missing_or_malformed_inputs_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    missing_dir_exit = main(["reports", "list", "--dir", str(tmp_path / "missing-dir")])
    missing_dir_payload = _read_json_output(capsys.readouterr().out)
    missing_history_exit = main(["reports", "history", "--dir", str(tmp_path / "missing-dir")])
    missing_history_payload = _read_json_output(capsys.readouterr().out)
    missing_export_exit = main(["reports", "export", "--dir", str(tmp_path / "missing-dir")])
    missing_export_payload = _read_json_output(capsys.readouterr().out)

    malformed_manifest_path = tmp_path / "bad.manifest.json"
    malformed_manifest_path.write_text("{bad-json", encoding="utf-8")

    malformed_manifest_exit = main(["reports", "show", "--manifest", str(malformed_manifest_path)])
    malformed_manifest_payload = _read_json_output(capsys.readouterr().out)
    malformed_history_exit = main(["reports", "history", "--dir", str(tmp_path)])
    malformed_history_payload = _read_json_output(capsys.readouterr().out)
    malformed_export_exit = main(["reports", "export", "--dir", str(tmp_path)])
    malformed_export_payload = _read_json_output(capsys.readouterr().out)

    malformed_manifest_path.unlink()

    valid_output_path = tmp_path / "tiny-suite.json"
    write_exit = main(["suite", "--suite", "tiny", "--output", "json", "--output-file", str(valid_output_path)])
    capsys.readouterr()
    replay_path = _replay_path_for(valid_output_path)
    replay_path.unlink()
    missing_replay_export_exit = main(["reports", "export", "--dir", str(tmp_path)])
    missing_replay_export_payload = _read_json_output(capsys.readouterr().out)

    broken_output_path = tmp_path / "tiny-suite-broken.json"
    write_broken_exit = main(["suite", "--suite", "tiny", "--output", "json", "--output-file", str(broken_output_path)])
    capsys.readouterr()
    broken_replay_path = _replay_path_for(broken_output_path)
    broken_replay_path.write_text("{bad-json", encoding="utf-8")
    broken_replay_export_exit = main(["reports", "export", "--dir", str(tmp_path)])
    broken_replay_export_payload = _read_json_output(capsys.readouterr().out)

    assert missing_dir_exit == 1
    assert missing_dir_payload["accepted"] is False
    assert missing_dir_payload["error_type"] == "reports_rejected"
    assert str(missing_dir_payload["reason"]).startswith("reports_dir_not_found:")

    assert missing_history_exit == 1
    assert missing_history_payload["accepted"] is False
    assert missing_history_payload["error_type"] == "reports_rejected"
    assert str(missing_history_payload["reason"]).startswith("reports_dir_not_found:")

    assert missing_export_exit == 1
    assert missing_export_payload["accepted"] is False
    assert missing_export_payload["error_type"] == "reports_rejected"
    assert str(missing_export_payload["reason"]).startswith("reports_dir_not_found:")

    assert malformed_manifest_exit == 1
    assert malformed_manifest_payload["accepted"] is False
    assert malformed_manifest_payload["error_type"] == "reports_rejected"
    assert str(malformed_manifest_payload["reason"]).startswith("json_payload_invalid:")

    assert malformed_history_exit == 1
    assert malformed_history_payload["accepted"] is False
    assert malformed_history_payload["error_type"] == "reports_rejected"
    assert str(malformed_history_payload["reason"]).startswith("json_payload_invalid:")

    assert malformed_export_exit == 1
    assert malformed_export_payload["accepted"] is False
    assert malformed_export_payload["error_type"] == "reports_rejected"
    assert str(malformed_export_payload["reason"]).startswith("json_payload_invalid:")

    assert write_exit == 0
    assert missing_replay_export_exit == 1
    assert missing_replay_export_payload["accepted"] is False
    assert missing_replay_export_payload["error_type"] == "reports_rejected"
    assert str(missing_replay_export_payload["reason"]).startswith("replay_drilldown_file_not_found:")

    assert write_broken_exit == 0
    assert broken_replay_export_exit == 1
    assert broken_replay_export_payload["accepted"] is False
    assert broken_replay_export_payload["error_type"] == "reports_rejected"
    assert str(broken_replay_export_payload["reason"]).startswith("json_payload_invalid:")


def test_cli_run_with_scenario_file_is_deterministic_for_identical_invocation(
    capsys: pytest.CaptureFixture[str],
) -> None:
    first_exit = main(["run", "--scenario-file", "scenarios/canonical/tiny_hidden_key.json"])
    first_output = capsys.readouterr().out
    second_exit = main(["run", "--scenario-file", "scenarios/canonical/tiny_hidden_key.json"])
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output
