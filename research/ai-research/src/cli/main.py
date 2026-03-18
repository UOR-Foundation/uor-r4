"""MUDBench CLI entrypoint with deterministic benchmark run wiring."""

from __future__ import annotations

import argparse
import json
from typing import Any, Mapping
from typing import Sequence

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle

_DEFAULT_RUN_ID = "cli-run"
_DEFAULT_BENCHMARK_ID = "mudbench-cli"
_DEFAULT_ACTOR_IDS = ("agent-a", "agent-b")

_SCENARIO_PRESETS: dict[str, dict[str, Any]] = {
    "minimal": {
        "scenario_id": "cli-minimal-scenario",
        "title": "CLI Minimal Scenario",
        "description": "Deterministic minimal scenario for mudbench run.",
        "start_room_id": "room-start",
        "max_steps": 3,
        "seed": 7,
        "version": "1.0",
        "scenario_vars": {"mode": "cli-minimal"},
        "objectives": [
            {
                "objective_id": "obj-a",
                "objective_type": "collect_item",
                "target_id": "item-key",
                "required_count": 1,
            }
        ],
    },
    "phase4-runtime-replay": {
        "scenario_id": "phase4-runtime-replay-scenario",
        "title": "Phase 4 Runtime Replay Wiring Scenario",
        "description": "Scenario preset that matches runtime replay gate expectations.",
        "start_room_id": "room-start",
        "max_steps": 3,
        "seed": 51,
        "version": "1.0",
        "scenario_vars": {"mode": "runtime-replay"},
        "objectives": [
            {
                "objective_id": "obj-a",
                "objective_type": "collect_item",
                "target_id": "item-key",
                "required_count": 1,
            }
        ],
    },
    "tiny-fetch-quest": {
        "scenario_id": "tiny-fetch-quest",
        "title": "Tiny Fetch Quest",
        "description": "Minimal 3-room scenario with item collection and NPC combat for vertical-slice e2e testing.",
        "start_room_id": "entrance",
        "max_steps": 5,
        "seed": 42,
        "version": "1.0",
        "scenario_vars": {
            "mode": "vertical-slice",
            "seed_variation_policy": "tiny_fetch_v1",
            "seed_variation_axis": "key_room",
            "seed_variation_values_json": "[\"treasury\",\"corridor\"]",
            "world_config_json": (
                '{"items":[{"entity_id":"golden-key","entity_type":"item","location":"treasury"}],'
                '"npcs":[{"entity_id":"guard-1","entity_type":"npc","health":30,"location":"corridor"}],'
                '"rooms":{"corridor":{"description":"A narrow stone corridor.","entities":[],'
                '"exits":{"east":"treasury","west":"entrance"},"title":"Stone Corridor"},'
                '"entrance":{"description":"A dimly lit entrance hall.","entities":[],'
                '"exits":{"east":"corridor"},"title":"Entrance Hall"},'
                '"treasury":{"description":"A small treasury chamber.","entities":[],'
                '"exits":{"west":"corridor"},"title":"Treasury Chamber"}}}'
            ),
        },
        "objectives": [
            {
                "objective_id": "collect-golden-key",
                "objective_type": "collect_item",
                "target_id": "golden-key",
                "required_count": 1,
            }
        ],
    },
    "tiny-delayed-retrieval": {
        "scenario_id": "tiny-delayed-retrieval",
        "title": "Tiny Delayed Retrieval",
        "description": "Tiny memory-focused scenario with delayed target retrieval.",
        "start_room_id": "start",
        "max_steps": 6,
        "seed": 24,
        "version": "1.0",
        "scenario_vars": {
            "mode": "vertical-slice-memory",
            "agent_script_policy": "memory_delayed_retrieval_v1",
            "world_config_json": (
                '{"items":[{"entity_id":"memory-token","entity_type":"item","location":"cache"},'
                '{"entity_id":"note-fragment","entity_type":"item","location":"start"}],'
                '"rooms":{"cache":{"description":"A quiet storage alcove.","entities":[],'
                '"exits":{"west":"junction"},"title":"Cache"},'
                '"junction":{"description":"A crossroads with worn stone marks.","entities":[],'
                '"exits":{"east":"cache","west":"start"},"title":"Junction"},'
                '"start":{"description":"A small starting chamber with etched markings.","entities":[],'
                '"exits":{"east":"junction"},"title":"Start"}}}'
            ),
        },
        "objectives": [
            {
                "objective_id": "retrieve-memory-token",
                "objective_type": "collect_item",
                "target_id": "memory-token",
                "required_count": 1,
            }
        ],
    },
    "tiny-hidden-key": {
        "scenario_id": "tiny-hidden-key",
        "title": "Tiny Hidden Key",
        "description": "Tiny partial-observability scenario with look-triggered hidden item reveal.",
        "start_room_id": "start",
        "max_steps": 6,
        "seed": 33,
        "version": "1.0",
        "scenario_vars": {
            "mode": "vertical-slice-observability",
            "agent_script_policy": "partial_observability_v1",
            "observation_policy": "look_reveals_hidden_items_v1",
            "hidden_item_ids_json": "[\"hidden-key\"]",
            "world_config_json": (
                '{"items":[{"entity_id":"hidden-key","entity_type":"item","location":"vault"},'
                '{"entity_id":"decoy-note","entity_type":"item","location":"start"}],'
                '"rooms":{"hall":{"description":"A quiet connecting hall.","entities":[],'
                '"exits":{"east":"vault","west":"start"},"title":"Hall"},'
                '"start":{"description":"A small chamber with worn stone walls.","entities":[],'
                '"exits":{"east":"hall"},"title":"Start"},'
                '"vault":{"description":"A compact vault with dusty shelves.","entities":[],'
                '"exits":{"west":"hall"},"title":"Vault"}}}'
            ),
        },
        "objectives": [
            {
                "objective_id": "collect-hidden-key",
                "objective_type": "collect_item",
                "target_id": "hidden-key",
                "required_count": 1,
            }
        ],
    },
    "tiny-locked-path": {
        "scenario_id": "tiny-locked-path",
        "title": "Tiny Locked Path",
        "description": "Tiny planning scenario that requires a key to open a sealed door before collecting the artifact.",
        "start_room_id": "entry",
        "max_steps": 8,
        "seed": 55,
        "version": "1.0",
        "scenario_vars": {
            "mode": "planning-dependency",
            "world_config_json": (
                '{"items":[{"entity_id":"brass-key","entity_type":"item","location":"key-chamber"},'
                '{"entity_id":"artifact","entity_type":"item","location":"treasure"}],'
                '"rooms":{"entry":{"description":"The entry chamber smells of salt and stone. Paths lead east to the locksmiths and south to a sealed gate.","entities":[],'
                '"exits":{"east":"key-chamber","south":"lock-ante"},"title":"Entry Chamber"},'
                '"key-chamber":{"description":"A cramped chamber with a brass key resting on an iron stand.","entities":[],'
                '"exits":{"west":"entry"},"title":"Key Chamber"},'
                '"lock-ante":{"description":"A narrow ante-chamber with a barred northern door and a passage south.","entities":[],'
                '"exits":{"south":"entry"},"title":"Lock Ante-Chamber"},'
                '"treasure":{"description":"A tiny vault lit by phosphor moss; a prized artifact lies within.","entities":[],'
                '"exits":{"south":"lock-ante"},"title":"Treasure Vault"}},"unlock_effects":[{"effect_id":"sealed_gate",'
                '"item_id":"brass-key","source_room_id":"lock-ante","direction":"north","destination_room_id":"treasure",'
                '"consume_item":false,"requires_actor_in_place":true}]}'
            ),
        },
        "objectives": [
            {
                "objective_id": "collect-artifact",
                "objective_type": "collect_item",
                "target_id": "artifact",
                "required_count": 1,
            }
        ],
    },
    "tiny-social-trade": {
        "scenario_id": "tiny-social-trade",
        "title": "Tiny Social Trade",
        "description": "Tiny social/trade scenario requiring token handoff to an NPC before objective retrieval.",
        "start_room_id": "start",
        "max_steps": 7,
        "seed": 66,
        "version": "1.0",
        "scenario_vars": {
            "mode": "social-trade-dependency",
            "agent_script_policy": "social-trade-dependency",
            "world_config_json": (
                '{"items":[{"entity_id":"trade-token","entity_type":"item","location":"token-room"}],'
                '"npcs":[{"entity_id":"trader","entity_type":"npc","health":30,"location":"market"}],'
                '"rooms":{"market":{"description":"A compact market stall where a trader watches quietly.","entities":[],'
                '"exits":{"west":"start"},"title":"Market"},'
                '"start":{"description":"A small crossroads linking the token room and market.","entities":[],'
                '"exits":{"east":"market","west":"token-room"},"title":"Crossroads"},'
                '"token-room":{"description":"A narrow supply room with a single trade token on a shelf.","entities":[],'
                '"exits":{"east":"start"},"title":"Token Room"}},'
                '"trade_effects":[{"effect_id":"market-trade","item_id":"trade-token","target_id":"trader",'
                '"reward_item_id":"artifact","reward_entity_type":"item"}]}'
            ),
        },
        "objectives": [
            {
                "objective_id": "collect-artifact",
                "objective_type": "collect_item",
                "target_id": "artifact",
                "required_count": 1,
            }
        ],
    },
}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="mudbench")
    subcommands = parser.add_subparsers(dest="command", required=True)

    run_parser = subcommands.add_parser("run", help="Execute deterministic benchmark runtime path")
    run_parser.add_argument(
        "--scenario",
        choices=tuple(sorted(_SCENARIO_PRESETS.keys())),
        default="minimal",
        help="Built-in scenario preset to execute",
    )
    run_parser.add_argument("--run-id", default=_DEFAULT_RUN_ID, help="Run identifier")
    run_parser.add_argument("--benchmark-id", default=_DEFAULT_BENCHMARK_ID, help="Benchmark identifier")
    run_parser.add_argument("--run-seed", type=int, default=None, help="Optional run seed override")
    run_parser.add_argument("--max-steps", type=int, default=None, help="Optional max step override")
    run_parser.add_argument(
        "--actor-id",
        action="append",
        default=[],
        help="Actor identifier (repeatable). Defaults to built-in deterministic actor set.",
    )
    run_parser.add_argument(
        "--output",
        choices=("json", "pretty"),
        default="json",
        help="CLI output format",
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.command == "run":
        actor_ids = tuple(args.actor_id) if len(args.actor_id) > 0 else _DEFAULT_ACTOR_IDS
        scenario_payload = _SCENARIO_PRESETS[args.scenario]
        config = BenchmarkRunnerConfig(
            run_id=args.run_id,
            benchmark_id=args.benchmark_id,
            scenario=scenario_payload,
            actor_ids=actor_ids,
            run_seed=args.run_seed,
            max_steps_override=args.max_steps,
        )

        try:
            result = run_benchmark_lifecycle(config)
        except (ValueError, RuntimeError) as exc:
            error_payload = {
                "accepted": False,
                "error_type": "run_rejected",
                "reason": str(exc),
            }
            print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
            return 1

        response_payload = _build_run_response(result.to_dict())
        if args.output == "pretty":
            print(json.dumps(response_payload, sort_keys=True, indent=2, ensure_ascii=True))
        else:
            print(json.dumps(response_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
        return 0
    parser.error("Unsupported command")
    return 2


def _build_run_response(result_payload: Mapping[str, Any]) -> dict[str, Any]:
    lifecycle_payload = result_payload["lifecycle_state"]
    scorecard_payload = result_payload["scorecard"]
    replay_payload = result_payload["replay_artifact"]
    parity_payload = result_payload["replay_parity_artifact"]
    return {
        "accepted": True,
        "run_id": lifecycle_payload["run_id"],
        "benchmark_id": scorecard_payload["metadata"]["benchmark_id"],
        "scenario_id": lifecycle_payload["scenario_id"],
        "lifecycle": {
            "status": lifecycle_payload["status"],
            "step_count": lifecycle_payload["step_index"],
            "max_steps": lifecycle_payload["max_steps"],
            "seed": lifecycle_payload["seed"],
        },
        "scorecard": {
            "aggregate_score": scorecard_payload["aggregate_score"],
            "metadata": scorecard_payload["metadata"],
        },
        "replay": {
            "artifact_refs": result_payload["replay_artifact_refs"],
            "event_count": len(replay_payload["events"]),
            "schema_version": replay_payload["envelope"]["schema_version"],
            "parity": {
                "terminal_step": parity_payload["terminal_step"],
                "step_count": parity_payload["step_count"],
                "terminal_state_hash": parity_payload["terminal_state_hash"],
                "applied_steps_hash": parity_payload["applied_steps_hash"],
                "score_summary_hash": parity_payload["score_summary_hash"],
            },
        },
    }


if __name__ == "__main__":
    raise SystemExit(main())
