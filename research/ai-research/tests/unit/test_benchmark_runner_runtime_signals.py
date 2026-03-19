from __future__ import annotations

import json
from pathlib import Path

from core.event_logger import EventRecord, normalize_payload
from evaluation.benchmark_runner.runner import (
    BenchmarkRunnerConfig,
    _build_runtime_step_signals,
    build_tiny_suite_baseline_report,
    run_benchmark_lifecycle,
)


def _metric_count(signals: tuple[object, ...], *, actor_id: str, metric_name: str) -> int:
    return sum(
        1
        for signal in signals
        if getattr(signal, "actor_id", None) == actor_id
        and getattr(signal, "metric_name", None) == metric_name
    )


def test_runtime_signals_emits_social_give_signal_but_not_trade_success_for_plain_give() -> None:
    signals = _build_runtime_step_signals(
        run_id="run-social-signal",
        step=1,
        actor_ids=("agent-a",),
        accepted_action_count=1,
        gateway_failures=(),
        emitted_events=(
            EventRecord(
                step_index=1,
                event_type="action_give",
                actor_id="agent-a",
                payload=normalize_payload({"item_id": "note", "target_id": "trader"}),
            ),
        ),
    )

    assert _metric_count(signals, actor_id="agent-a", metric_name="social.give.completed") == 1
    assert _metric_count(signals, actor_id="agent-a", metric_name="social.trade.completed") == 0


def test_runtime_signals_emits_social_trade_success_only_for_trade_completion_unlock() -> None:
    signals = _build_runtime_step_signals(
        run_id="run-social-signal",
        step=2,
        actor_ids=("agent-a",),
        accepted_action_count=1,
        gateway_failures=(),
        emitted_events=(
            EventRecord(
                step_index=2,
                event_type="dependency_unlocked",
                actor_id="agent-a",
                payload=normalize_payload(
                    {
                        "effect_id": "market-trade",
                        "item_id": "trade-token",
                        "target_id": "trader",
                        "reward_item_id": "artifact",
                    }
                ),
            ),
        ),
    )
    assert _metric_count(signals, actor_id="agent-a", metric_name="social.trade.completed") == 1

    non_trade_signals = _build_runtime_step_signals(
        run_id="run-social-signal",
        step=2,
        actor_ids=("agent-a",),
        accepted_action_count=1,
        gateway_failures=(),
        emitted_events=(
            EventRecord(
                step_index=2,
                event_type="dependency_unlocked",
                actor_id="agent-a",
                payload=normalize_payload(
                    {
                        "effect_id": "sealed_gate",
                        "item_id": "brass-key",
                        "source_room_id": "lock-ante",
                        "direction": "north",
                        "destination_room_id": "treasure",
                    }
                ),
            ),
        ),
    )
    assert _metric_count(
        non_trade_signals, actor_id="agent-a", metric_name="social.trade.completed"
    ) == 0


def test_tiny_suite_baseline_report_includes_required_fields_for_configured_agents_only() -> None:
    scenario = json.loads(Path("scenarios/canonical/tiny_fetch_quest.json").read_text(encoding="utf-8"))

    result = run_benchmark_lifecycle(
        BenchmarkRunnerConfig(
            run_id="unit-suite-report-run",
            benchmark_id="unit-benchmark",
            scenario=scenario,
            actor_ids=("agent-a", "agent-b"),
        )
    )

    report = build_tiny_suite_baseline_report((result,))

    assert report["schema_version"] == "tiny_suite_baseline_report_v1"
    assert report["benchmark_ids"] == ["unit-benchmark"]
    assert report["scenario_count"] == 1
    assert report["entry_count"] == 2

    entries = report["entries"]
    assert [entry["agent_id"] for entry in entries] == ["agent-a", "agent-b"]
    assert all(entry["scenario_id"] == "tiny-fetch-quest" for entry in entries)
    assert all(isinstance(entry["aggregate_score"], float) for entry in entries)
    assert all(isinstance(entry["composite_score"], float) for entry in entries)
    assert all(entry["replay_ref"].startswith("sha256:") for entry in entries)
    assert all(
        set(entry["parity_ref"].keys()) == {"terminal_state_hash", "applied_steps_hash", "score_summary_hash"}
        for entry in entries
    )
    assert all("quest_completion" in entry["normalized_metrics"] for entry in entries)
    assert all("quest_completion" in entry["contributions"] for entry in entries)
