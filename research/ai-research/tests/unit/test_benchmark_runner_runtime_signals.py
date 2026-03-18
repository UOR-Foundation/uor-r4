from __future__ import annotations

from core.event_logger import EventRecord, normalize_payload
from evaluation.benchmark_runner.runner import _build_runtime_step_signals


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
