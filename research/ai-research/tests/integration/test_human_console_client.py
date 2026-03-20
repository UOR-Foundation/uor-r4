from __future__ import annotations

import builtins
import json
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import pytest

from cli.main import main


def _start_direct_provider_test_server(
    response_contents: list[str],
) -> tuple[str, ThreadingHTTPServer]:
    remaining_responses = list(response_contents)

    class _Handler(BaseHTTPRequestHandler):
        def do_POST(self) -> None:  # noqa: N802
            content_length = int(self.headers.get("Content-Length", "0"))
            self.rfile.read(content_length)
            response_content = remaining_responses.pop(0)
            response_payload = {
                "choices": [
                    {
                        "message": {
                            "content": response_content,
                        }
                    }
                ]
            }
            encoded = json.dumps(
                response_payload,
                sort_keys=True,
                separators=(",", ":"),
                ensure_ascii=True,
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(encoded)))
            self.end_headers()
            self.wfile.write(encoded)

        def log_message(self, format: str, *args: object) -> None:  # noqa: A003
            return

    server = ThreadingHTTPServer(("127.0.0.1", 0), _Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return f"http://127.0.0.1:{server.server_address[1]}/v1/chat/completions", server


def test_cli_play_human_console_guarded_relic_flow_works_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    scripted_inputs = iter(
        (
            "east",
            "take key",
            "north",
            "attack sentinel",
            "east",
            "use key",
            "north",
            "take relic",
        )
    )
    monkeypatch.setattr(builtins, "input", lambda _prompt="": next(scripted_inputs))

    exit_code = main(["play", "--scenario", "tiny-guarded-relic", "--actor-id", "human-player"])
    captured = capsys.readouterr()

    assert exit_code == 0
    assert "Location: camp" in captured.out
    assert "Available Actions:" in captured.out
    assert "Allowed Targets:" in captured.out
    assert "Accepted Action: move east" in captured.out
    assert "Accepted Action: take relic" in captured.out
    assert "Objective complete." in captured.out
    assert "Session Summary" in captured.out
    assert "Scenario: tiny-guarded-relic" in captured.out
    assert "Objective Completed: True" in captured.out


def test_cli_play_human_console_hazard_route_direct_hazard_path_works_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    scripted_inputs = iter(
        (
            "north",
            "attack raider",
            "attack raider",
            "north",
            "take core",
        )
    )
    monkeypatch.setattr(builtins, "input", lambda _prompt="": next(scripted_inputs))

    exit_code = main(["play", "--scenario", "tiny-hazard-route", "--actor-id", "human-player"])
    captured = capsys.readouterr()

    assert exit_code == 0
    assert "Location: camp" in captured.out
    assert "Accepted Action: move north" in captured.out
    assert "Accepted Action: attack raider" in captured.out
    assert "Accepted Action: take storm-core" in captured.out
    assert "Objective complete." in captured.out
    assert "Scenario: tiny-hazard-route" in captured.out
    assert "Objective Completed: True" in captured.out


def test_cli_play_human_console_delayed_cost_safe_path_works_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    scripted_inputs = iter(
        (
            "east",
            "take cell",
            "east",
            "north",
            "use cell",
            "north",
            "take ledger",
        )
    )
    monkeypatch.setattr(builtins, "input", lambda _prompt="": next(scripted_inputs))

    exit_code = main(["play", "--scenario", "tiny-delayed-cost", "--actor-id", "human-player"])
    captured = capsys.readouterr()

    assert exit_code == 0
    assert "Location: camp" in captured.out
    assert "Accepted Action: move east" in captured.out
    assert "Accepted Action: use power-cell" in captured.out
    assert "Accepted Action: take archive-ledger" in captured.out
    assert "Objective complete." in captured.out
    assert "Scenario: tiny-delayed-cost" in captured.out
    assert "Objective Completed: True" in captured.out


def test_cli_play_shared_shard_fetch_quest_flow_works_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    scripted_inputs = iter(
        (
            "east",
            "wait",
            "east",
            "east",
            "take key",
            "look",
        )
    )
    monkeypatch.setattr(builtins, "input", lambda _prompt="": next(scripted_inputs))

    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "human-a",
            "--actor-id",
            "human-b",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert "Actor Turn: human-a" in captured.out
    assert "Actor Turn: human-b" in captured.out
    assert (
        "Messages: The shard feels still, as if the watch has not yet begun., "
        "Hint: the route feels open while the watch remains dormant."
    ) in captured.out
    assert (
        "Messages: A distant sentinel grows watchful in the quiet corridors., "
        "Hint: the sharp watch makes a careful look feel safer than rushing., "
        "Consequence: the exposed west passage is pinned under watch; move west is unavailable."
    ) in captured.out
    assert (
        "Messages: You catch the measured rhythm of a distant patrol., "
        "Hint: the moving patrol leaves brief windows for repositioning."
    ) in captured.out
    assert "Actor Result: human-a" in captured.out
    assert "Actor Result: human-b" in captured.out
    assert "Accepted Action: take golden-key" in captured.out
    assert "Objective complete for: human-a" in captured.out
    assert "Shard: shared-shard-local" in captured.out
    assert "Completed Actors: human-a" in captured.out
    assert "World Tick Effect: npc_stance_phase=watchful" in captured.out
    assert "World Tick Effect: npc_stance_phase=patrolling" in captured.out
    assert "World Tick Effect: npc_stance_phase=settling" in captured.out
    assert "World Tick Count: 3" in captured.out
    assert "Last World Tick Heartbeat: shared_shard_world_tick:0003" in captured.out
    assert "World NPC Stance Phase: settling" in captured.out


def test_cli_play_shared_shard_mixed_human_and_agent_flow_works_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    scripted_inputs = iter(
        (
            "east",
            "east",
            "wait",
        )
    )
    monkeypatch.setattr(builtins, "input", lambda _prompt="": next(scripted_inputs))

    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--mock-agent-actor-id",
            "agent-b",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert "Actor Turn: human-a" in captured.out
    assert "Actor Turn: agent-b" in captured.out
    assert "Agent Selected Action: move east" in captured.out
    assert "Agent Selected Action: take golden-key" in captured.out
    assert "Objective complete for: agent-b" in captured.out
    assert "Completed Actors: agent-b" in captured.out
    assert (
        "Messages: You catch the measured rhythm of a distant patrol., "
        "Hint: the moving patrol leaves brief windows for repositioning."
    ) in captured.out


def test_cli_play_shared_shard_phase_outcome_effect_is_experienced_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    scripted_inputs = iter(
        (
            "east",
            "wait",
            "west",
            "wait",
            "wait",
            "west",
            "wait",
            "quit",
        )
    )
    monkeypatch.setattr(builtins, "input", lambda _prompt="": next(scripted_inputs))

    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "human-a",
            "--actor-id",
            "human-b",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert (
        "Consequence: the exposed west passage is pinned under watch; move west is unavailable."
    ) in captured.out
    assert "Rejected Input: direction_not_available" in captured.out
    assert (
        "Consequence: the west passage opens again between patrol sweeps."
    ) in captured.out
    assert "Accepted Action: move west" in captured.out


def test_cli_play_shared_shard_mixed_human_and_external_agent_flow_works_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    scripted_inputs = iter(
        (
            "east",
            "east",
            "wait",
        )
    )
    monkeypatch.setattr(builtins, "input", lambda _prompt="": next(scripted_inputs))

    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--agent-command",
            f"{sys.executable} examples/agents/mock_llm_wrapper.py",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert "Actor Turn: human-a" in captured.out
    assert "Actor Turn: external-agent" in captured.out
    assert "External Agent Selected Action: move east" in captured.out
    assert "External Agent Selected Action: take golden-key" in captured.out
    assert "Objective complete for: external-agent" in captured.out
    assert "Completed Actors: external-agent" in captured.out
    assert (
        "Messages: You catch the measured rhythm of a distant patrol., "
        "Hint: the moving patrol leaves brief windows for repositioning."
    ) in captured.out


def test_cli_play_shared_shard_mixed_human_and_direct_provider_flow_works_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    base_url, server = _start_direct_provider_test_server(
        [
            '{"action":"move east"}',
            "not-json",
            '{"action":"move east"}',
            '{"action":"take golden-key"}',
        ]
    )
    scripted_inputs = iter(
        (
            "east",
            "east",
            "wait",
        )
    )
    monkeypatch.setattr(builtins, "input", lambda _prompt="": next(scripted_inputs))
    monkeypatch.setenv("MUDBENCH_OPENAI_API_KEY", "test-key")
    monkeypatch.setenv("MUDBENCH_OPENAI_BASE_URL", base_url)

    try:
        exit_code = main(
            [
                "play-shared-shard",
                "--scenario",
                "tiny-fetch-quest",
                "--direct-provider",
                "openai-chat-completions",
                "--direct-provider-model",
                "gpt-4.1-mini",
                "--external-agent-actor-id",
                "direct-provider",
            ]
        )
        captured = capsys.readouterr()
    finally:
        server.shutdown()
        server.server_close()

    assert exit_code == 0
    assert "Actor Turn: human-a" in captured.out
    assert "Actor Turn: direct-provider" in captured.out
    assert "External Agent Selected Action: move east" in captured.out
    assert "External Agent Selected Action: take golden-key" in captured.out
    assert "Objective complete for: direct-provider" in captured.out
    assert "Completed Actors: direct-provider" in captured.out
    assert (
        "Consequence: the exposed west passage is pinned under watch; move west is unavailable."
    ) in captured.out


def test_cli_play_shared_shard_mixed_human_and_persistent_external_agent_flow_works_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    scripted_inputs = iter(
        (
            "east",
            "east",
            "wait",
        )
    )
    monkeypatch.setattr(builtins, "input", lambda _prompt="": next(scripted_inputs))

    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--agent-command",
            f"{sys.executable} examples/agents/mock_llm_wrapper.py --persistent-session",
            "--persistent-agent-session",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert "Actor Turn: human-a" in captured.out
    assert "Actor Turn: external-agent" in captured.out
    assert "External Agent Selected Action: move east" in captured.out
    assert "External Agent Selected Action: take golden-key" in captured.out
    assert "Objective complete for: external-agent" in captured.out
    assert "Completed Actors: external-agent" in captured.out
    assert "World Tick Effect: npc_stance_phase=patrolling" in captured.out
    assert (
        "Messages: You catch the measured rhythm of a distant patrol., "
        "Hint: the moving patrol leaves brief windows for repositioning."
    ) in captured.out
