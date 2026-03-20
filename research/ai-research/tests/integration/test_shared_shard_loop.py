from __future__ import annotations

import json
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import pytest

from agents.direct_provider_runner import DirectProviderConfig, build_direct_provider_command
from cli.main import _SCENARIO_PRESETS
from evaluation.benchmark_runner.runner import build_shared_shard_loop_session


def _write_external_shared_agent_script(tmp_path: Path) -> Path:
    script_path = tmp_path / "shared_external_agent.py"
    script_path.write_text(
        (
            "import json\n"
            "import sys\n"
            "line = sys.stdin.readline()\n"
            "observation = json.loads(line)\n"
            "messages = tuple(observation.get('messages', ()))\n"
            "action_space = tuple(observation.get('action_space', ()))\n"
            "action = 'wait'\n"
            "if any('dormant' in message for message in messages) and 'move east' in action_space:\n"
            "    action = 'move east'\n"
            "elif any('watchful' in message for message in messages) and 'move east' in action_space:\n"
            "    action = 'move east'\n"
            "elif any('patrol' in message for message in messages):\n"
            "    for candidate in action_space:\n"
            "        if candidate.startswith('take '):\n"
            "            action = candidate\n"
            "            break\n"
            "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
        ),
        encoding="utf-8",
    )
    return script_path


def _write_persistent_external_shared_agent_script(tmp_path: Path) -> tuple[Path, Path]:
    script_path = tmp_path / "persistent_shared_external_agent.py"
    boot_log_path = tmp_path / "persistent_shared_external_agent.boot.log"
    script_path.write_text(
        (
            "import json\n"
            "import sys\n"
            "from pathlib import Path\n"
            f"boot_log_path = Path({str(boot_log_path)!r})\n"
            "boot_log_path.write_text(boot_log_path.read_text(encoding='utf-8') + 'boot\\n' if boot_log_path.exists() else 'boot\\n', encoding='utf-8')\n"
            "turn_count = 0\n"
            "while True:\n"
            "    line = sys.stdin.readline()\n"
            "    if not line:\n"
            "        break\n"
            "    observation = json.loads(line)\n"
            "    action_space = tuple(observation.get('action_space', ()))\n"
            "    action = 'wait'\n"
            "    if turn_count < 2 and 'move east' in action_space:\n"
            "        action = 'move east'\n"
            "    elif turn_count >= 2:\n"
            "        for candidate in action_space:\n"
            "            if candidate.startswith('take '):\n"
            "                action = candidate\n"
            "                break\n"
            "    print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
            "    sys.stdout.flush()\n"
            "    turn_count += 1\n"
        ),
        encoding="utf-8",
    )
    return script_path, boot_log_path


def _start_direct_provider_test_server(
    response_contents: list[str],
) -> tuple[str, list[dict[str, object]], ThreadingHTTPServer]:
    captured_requests: list[dict[str, object]] = []
    remaining_responses = list(response_contents)

    class _Handler(BaseHTTPRequestHandler):
        def do_POST(self) -> None:  # noqa: N802
            content_length = int(self.headers.get("Content-Length", "0"))
            body = self.rfile.read(content_length).decode("utf-8")
            payload = json.loads(body)
            captured_requests.append(payload)
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
    base_url = f"http://127.0.0.1:{server.server_address[1]}/v1/chat/completions"
    return base_url, captured_requests, server


def test_shared_shard_loop_preserves_world_continuity_across_multiple_participants() -> None:
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    first = session.advance_tick({"player-a": "move east", "player-b": "wait"})
    second = session.advance_tick({"player-a": "move east", "player-b": "move east"})
    third = session.advance_tick({"player-a": "take golden-key", "player-b": "move east"})
    player_b_observation = session.get_observation("player-b")

    assert first.to_dict() == {
        "step_index": 0,
        "accepted_actions": [
            {"actor_id": "player-a", "action": "move east"},
            {"actor_id": "player-b", "action": "wait"},
        ],
        "emitted_event_types": ["action_move", "action_wait", "step_completed"],
        "active_actor_ids": ["player-a", "player-b"],
        "shard_mutation_generation": 6,
        "world_tick_count": 1,
        "world_tick_heartbeat": "shared_shard_world_tick:0001",
        "world_npc_stance_phase": "watchful",
    }
    assert second.step_index == 1
    assert third.step_index == 2
    assert player_b_observation.location == "treasury"
    assert player_b_observation.messages == (
        "The far-off watch settles back into guarded stillness.",
        "Hint: the easing watch makes nearby movement feel less exposed.",
    )
    assert [entity.name for entity in player_b_observation.entities] == ["golden-key", "player-a"]
    assert session.world_state.get_snapshot()["entities"]["player-a"]["inventory"] == ["golden-key"]
    assert session.shard_state.get_session("sess-player-a").status == "active"
    assert session.shard_state.get_session("sess-player-b").status == "active"
    assert session.world_tick_count == 3
    assert session.shard_state.metadata.last_world_tick_heartbeat == "shared_shard_world_tick:0003"
    assert session.world_npc_stance_phase == "settling"
    assert session.current_tick == 3


def test_shared_shard_loop_supports_session_close_and_reopen_without_losing_continuity() -> None:
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    session.close_participant_session("player-b")
    assert session.session_is_active("player-b") is False

    first = session.advance_tick({"player-a": "move east"})

    session.open_participant_session("player-b")
    reopened_observation = session.get_observation("player-b")
    second = session.advance_tick({"player-a": "move east", "player-b": "move east"})

    assert first.active_actor_ids == ("player-a",)
    assert reopened_observation.location == "entrance"
    assert session.session_is_active("player-b") is True
    assert session.shard_state.get_session("sess-player-b").status == "active"
    assert second.active_actor_ids == ("player-a", "player-b")
    assert second.world_tick_count == 2
    assert session.shard_state.metadata.last_world_tick_heartbeat == "shared_shard_world_tick:0002"
    assert session.world_npc_stance_phase == "patrolling"
    assert session.current_tick == 2


def test_shared_shard_loop_world_tick_advances_deterministically_even_when_actors_wait() -> None:
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    first = session.advance_tick({"player-a": "wait", "player-b": "wait"})
    second = session.advance_tick({"player-a": "wait", "player-b": "wait"})
    second_observation = session.get_observation("player-a")

    assert first.emitted_event_types == ("action_wait", "action_wait", "step_completed")
    assert first.world_tick_count == 1
    assert first.world_tick_heartbeat == "shared_shard_world_tick:0001"
    assert first.world_npc_stance_phase == "watchful"
    assert second.world_tick_count == 2
    assert second.world_tick_heartbeat == "shared_shard_world_tick:0002"
    assert second.world_npc_stance_phase == "patrolling"
    assert session.shard_state.metadata.world_tick_count == 2
    assert session.shard_state.metadata.last_world_tick_heartbeat == "shared_shard_world_tick:0002"
    assert session.shard_state.metadata.npc_stance_phase == "patrolling"
    assert second_observation.messages == (
        "You catch the measured rhythm of a distant patrol.",
        "Hint: the moving patrol leaves brief windows for repositioning.",
    )
    assert session.current_tick == 2


def test_shared_shard_loop_phase_outcome_effect_blocks_and_reopens_corridor_route_deterministically() -> None:
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    first = session.advance_tick({"player-a": "move east", "player-b": "wait"})
    watchful_observation = session.get_observation("player-a")
    second = session.advance_tick({"player-a": "wait", "player-b": "wait"})
    patrolling_observation = session.get_observation("player-a")

    assert first.world_npc_stance_phase == "watchful"
    assert watchful_observation.location == "corridor"
    assert watchful_observation.action_space == ("wait", "look", "move east", "attack guard-1")
    assert watchful_observation.messages == (
        "A distant sentinel grows watchful in the quiet corridors.",
        "Hint: the sharp watch makes a careful look feel safer than rushing.",
        "Consequence: the exposed west passage is pinned under watch; move west is unavailable.",
    )
    assert second.world_npc_stance_phase == "patrolling"
    assert patrolling_observation.location == "corridor"
    assert "move west" in patrolling_observation.action_space
    assert patrolling_observation.messages == (
        "You catch the measured rhythm of a distant patrol.",
        "Hint: the moving patrol leaves brief windows for repositioning.",
        "Consequence: the west passage opens again between patrol sweeps.",
    )
    assert session.current_tick == 2


def test_shared_shard_loop_supports_mixed_human_and_agent_participation_deterministically() -> None:
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("human-a", "agent-b"),
        agent_actor_ids=("agent-b",),
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    first_agent_turn = session.build_mock_agent_turn("agent-b")
    first = session.advance_tick(
        {
            "human-a": "move east",
            "agent-b": first_agent_turn.action_submission.action,
        }
    )
    second_agent_turn = session.build_mock_agent_turn("agent-b")
    second = session.advance_tick(
        {
            "human-a": "move east",
            "agent-b": second_agent_turn.action_submission.action,
        }
    )
    third_agent_turn = session.build_mock_agent_turn("agent-b")
    third = session.advance_tick(
        {
            "human-a": "wait",
            "agent-b": third_agent_turn.action_submission.action,
        }
    )
    human_observation = session.get_observation("human-a")

    assert session.is_agent_participant("human-a") is False
    assert session.is_agent_participant("agent-b") is True
    assert session.shard_state.get_character("char-agent-b").identity_class == "external_agent"
    assert first_agent_turn.action_submission.action == "move east"
    assert second_agent_turn.action_submission.action == "move east"
    assert third_agent_turn.action_submission.action == "take golden-key"
    assert "Invariant runtime guardrails:" in first_agent_turn.prompt
    assert first_agent_turn.model_facing_observation_payload["observation"]["messages"] == [
        "The shard feels still, as if the watch has not yet begun.",
        "Hint: the route feels open while the watch remains dormant.",
    ]
    assert first.accepted_actions == (("human-a", "move east"), ("agent-b", "move east"))
    assert second.accepted_actions == (("human-a", "move east"), ("agent-b", "move east"))
    assert third.accepted_actions == (("human-a", "wait"), ("agent-b", "take golden-key"))
    assert third.world_tick_count == 3
    assert third.world_npc_stance_phase == "settling"
    assert session.world_state.get_snapshot()["entities"]["agent-b"]["inventory"] == ["golden-key"]
    assert human_observation.messages == (
        "The far-off watch settles back into guarded stillness.",
        "Hint: the easing watch makes nearby movement feel less exposed.",
    )


def test_shared_shard_loop_supports_mixed_human_and_external_agent_participation_deterministically(
    tmp_path: Path,
) -> None:
    script_path = _write_external_shared_agent_script(tmp_path)
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("human-a", "external-b"),
        external_agent_commands_by_actor={
            "external-b": (sys.executable, str(script_path)),
        },
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    try:
        first_action = session.request_external_agent_action("external-b")
        first = session.advance_tick(
            {
                "human-a": "move east",
                "external-b": first_action.action,
            }
        )
        second_action = session.request_external_agent_action("external-b")
        second = session.advance_tick(
            {
                "human-a": "move east",
                "external-b": second_action.action,
            }
        )
        third_action = session.request_external_agent_action("external-b")
        third = session.advance_tick(
            {
                "human-a": "wait",
                "external-b": third_action.action,
            }
        )
        human_observation = session.get_observation("human-a")
    finally:
        session.close_external_agent_participants()

    assert session.is_external_agent_participant("external-b") is True
    assert session.is_agent_participant("external-b") is False
    assert session.shard_state.get_character("char-external-b").identity_class == "external_agent"
    assert first_action.action == "move east"
    assert second_action.action == "move east"
    assert third_action.action == "take golden-key"
    assert first.accepted_actions == (("human-a", "move east"), ("external-b", "move east"))
    assert second.accepted_actions == (("human-a", "move east"), ("external-b", "move east"))
    assert third.accepted_actions == (("human-a", "wait"), ("external-b", "take golden-key"))
    assert session.world_state.get_snapshot()["entities"]["external-b"]["inventory"] == ["golden-key"]
    assert human_observation.messages == (
        "The far-off watch settles back into guarded stillness.",
        "Hint: the easing watch makes nearby movement feel less exposed.",
    )


def test_shared_shard_loop_supports_persistent_external_agent_reuse_deterministically(
    tmp_path: Path,
) -> None:
    script_path, boot_log_path = _write_persistent_external_shared_agent_script(tmp_path)
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("human-a", "external-b"),
        external_agent_commands_by_actor={
            "external-b": (sys.executable, str(script_path)),
        },
        persistent_agent_session=True,
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    try:
        first_action = session.request_external_agent_action("external-b")
        first = session.advance_tick(
            {
                "human-a": "move east",
                "external-b": first_action.action,
            }
        )
        second_action = session.request_external_agent_action("external-b")
        second = session.advance_tick(
            {
                "human-a": "move east",
                "external-b": second_action.action,
            }
        )
        third_action = session.request_external_agent_action("external-b")
        third = session.advance_tick(
            {
                "human-a": "wait",
                "external-b": third_action.action,
            }
        )
        human_observation = session.get_observation("human-a")
    finally:
        session.close_external_agent_participants()

    assert first.accepted_actions == (("human-a", "move east"), ("external-b", "move east"))
    assert second.accepted_actions == (("human-a", "move east"), ("external-b", "move east"))
    assert third.accepted_actions == (("human-a", "wait"), ("external-b", "take golden-key"))
    assert first_action.action == "move east"
    assert second_action.action == "move east"
    assert third_action.action == "take golden-key"
    assert session.world_tick_count == 3
    assert session.world_state.get_snapshot()["entities"]["external-b"]["inventory"] == ["golden-key"]
    assert human_observation.messages == (
        "The far-off watch settles back into guarded stillness.",
        "Hint: the easing watch makes nearby movement feel less exposed.",
    )
    assert boot_log_path.read_text(encoding="utf-8").splitlines() == ["boot"]


def test_shared_shard_loop_supports_mixed_human_and_direct_provider_participation_deterministically(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    base_url, captured_requests, server = _start_direct_provider_test_server(
        [
            '{"action":"move east"}',
            "not-json",
            '{"action":"move east"}',
            '{"action":"take golden-key"}',
        ]
    )
    monkeypatch.setenv("MUDBENCH_OPENAI_API_KEY", "test-key")
    provider_config = DirectProviderConfig(
        provider="openai-chat-completions",
        model="gpt-4.1-mini",
        api_key="test-key",
        base_url=base_url,
    )
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("human-a", "direct-b"),
        external_agent_commands_by_actor={
            "direct-b": build_direct_provider_command(
                provider_config,
                python_executable=sys.executable,
            ),
        },
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    try:
        first_action = session.request_external_agent_action("direct-b")
        first = session.advance_tick(
            {
                "human-a": "move east",
                "direct-b": first_action.action,
            }
        )
        second_action = session.request_external_agent_action("direct-b")
        second = session.advance_tick(
            {
                "human-a": "move east",
                "direct-b": second_action.action,
            }
        )
        third_action = session.request_external_agent_action("direct-b")
        third = session.advance_tick(
            {
                "human-a": "wait",
                "direct-b": third_action.action,
            }
        )
        human_observation = session.get_observation("human-a")
    finally:
        session.close_external_agent_participants()
        server.shutdown()
        server.server_close()

    assert session.is_agent_participant("direct-b") is False
    assert session.shard_state.get_character("char-direct-b").identity_class == "external_agent"
    assert session.is_external_agent_participant("direct-b") is True
    assert first_action.action == "move east"
    assert second_action.action == "move east"
    assert third_action.action == "take golden-key"
    assert first.accepted_actions == (("human-a", "move east"), ("direct-b", "move east"))
    assert second.accepted_actions == (("human-a", "move east"), ("direct-b", "move east"))
    assert third.accepted_actions == (("human-a", "wait"), ("direct-b", "take golden-key"))
    assert session.world_tick_count == 3
    assert session.world_state.get_snapshot()["entities"]["direct-b"]["inventory"] == ["golden-key"]
    assert human_observation.messages == (
        "The far-off watch settles back into guarded stillness.",
        "Hint: the easing watch makes nearby movement feel less exposed.",
    )
    assert len(captured_requests) == 4
    assert "Invariant runtime guardrails:" in str(captured_requests[0]["messages"][0]["content"])
    assert (
        "Consequence: the exposed west passage is pinned under watch; move west is unavailable."
        in str(captured_requests[1]["messages"][0]["content"])
    )
    assert '{"allowed_actions":["wait","look","move east","attack guard-1"]}' in str(
        captured_requests[1]["messages"][0]["content"]
    )
    assert "Your previous response was invalid for MUDBench." in str(
        captured_requests[2]["messages"][0]["content"]
    )
