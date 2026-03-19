"""MUDBench CLI entrypoint with deterministic benchmark run wiring."""

from __future__ import annotations

import argparse
import json
import shlex
from pathlib import Path
from typing import Any, Mapping
from typing import Sequence

from evaluation.benchmark_runner.runner import (
    BenchmarkRunnerConfig,
    build_tiny_suite_baseline_report,
    build_tiny_suite_comparison_report,
    build_tiny_suite_external_comparison_report,
    build_tiny_suite_mixed_external_comparison_report,
    run_benchmark_lifecycle,
)

_DEFAULT_RUN_ID = "cli-run"
_DEFAULT_BENCHMARK_ID = "mudbench-cli"
_DEFAULT_ACTOR_IDS = ("agent-a", "agent-b")
_DEFAULT_SUITE_ID = "tiny"
_BUILTIN_COMPARISON_AGENT_IDS = ("agent-a", "agent-b")
_EXTERNAL_COMPARISON_AGENT_ID = "external-local-agent"
_TINY_SUITE_SCENARIOS = (
    "tiny-delayed-retrieval",
    "tiny-fetch-quest",
    "tiny-hidden-key",
    "tiny-locked-path",
    "tiny-social-trade",
)

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
    run_parser.add_argument(
        "--scenario-file",
        default=None,
        help="Path to a scenario definition JSON file on disk",
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
    run_parser.add_argument(
        "--agent-command",
        default=None,
        help="Single-shot local-process agent command string; MUDBench sends one observation on stdin and expects one action JSON on stdout",
    )
    run_parser.add_argument(
        "--persistent-agent-session",
        action="store_true",
        help="Keep the external local-process agent command alive across turns using the same stdin/stdout JSON contract; requires --agent-command",
    )

    suite_parser = subcommands.add_parser("suite", help="Execute deterministic tiny-suite baseline reporting")
    suite_parser.add_argument(
        "--suite",
        choices=("tiny",),
        default=_DEFAULT_SUITE_ID,
        help="Built-in suite preset to execute",
    )
    suite_parser.add_argument("--benchmark-id", default=_DEFAULT_BENCHMARK_ID, help="Benchmark identifier")
    suite_parser.add_argument(
        "--baseline-agent",
        default=None,
        help="Built-in actor profile to use as the baseline comparison side",
    )
    suite_parser.add_argument(
        "--candidate-agent",
        default=None,
        help="Built-in actor profile to use as the candidate comparison side",
    )
    suite_parser.add_argument(
        "--agent-command",
        default=None,
        help="External local-process agent command string for suite comparison candidate side",
    )
    suite_parser.add_argument(
        "--external-agent-actor",
        default=None,
        help="Built-in actor slot to replace with the external local-process candidate in shared-run suite comparison",
    )
    suite_parser.add_argument(
        "--persistent-agent-session",
        action="store_true",
        help="Keep the external local-process suite candidate alive across turns using the same stdin/stdout JSON contract; requires --agent-command",
    )
    suite_parser.add_argument(
        "--actor-id",
        action="append",
        default=[],
        help="Actor identifier (repeatable). Defaults to built-in deterministic actor set.",
    )
    suite_parser.add_argument(
        "--output",
        choices=("json", "pretty"),
        default="json",
        help="CLI output format",
    )
    suite_parser.add_argument(
        "--output-file",
        default=None,
        help="Optional path to write the emitted suite report JSON",
    )

    reports_parser = subcommands.add_parser("reports", help="Inspect saved suite report artifacts")
    reports_subcommands = reports_parser.add_subparsers(dest="reports_command", required=True)

    reports_list_parser = reports_subcommands.add_parser("list", help="List saved suite report artifacts")
    reports_list_parser.add_argument("--dir", required=True, help="Directory containing saved report manifests")
    reports_list_parser.add_argument(
        "--output",
        choices=("json", "pretty"),
        default="json",
        help="CLI output format",
    )

    reports_show_parser = reports_subcommands.add_parser("show", help="Show a saved suite report artifact")
    reports_show_parser.add_argument("--manifest", required=True, help="Path to a saved suite report manifest")
    reports_show_parser.add_argument(
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
        try:
            scenario_payload = _resolve_run_scenario_payload(
                scenario_name=args.scenario,
                scenario_file=args.scenario_file,
            )
            external_agent_command = _resolve_external_agent_command(args.agent_command)
            if args.persistent_agent_session and external_agent_command is None:
                raise ValueError("persistent_agent_session_requires_agent_command")
        except ValueError as exc:
            error_payload = {
                "accepted": False,
                "error_type": "run_rejected",
                "reason": str(exc),
            }
            print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
            return 1

        config = BenchmarkRunnerConfig(
            run_id=args.run_id,
            benchmark_id=args.benchmark_id,
            scenario=scenario_payload,
            actor_ids=actor_ids,
            run_seed=args.run_seed,
            max_steps_override=args.max_steps,
            external_agent_command=external_agent_command,
            persistent_agent_session=args.persistent_agent_session,
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
    if args.command == "suite":
        actor_ids = tuple(args.actor_id) if len(args.actor_id) > 0 else _DEFAULT_ACTOR_IDS
        try:
            external_agent_command = _resolve_external_agent_command(args.agent_command)
            if args.persistent_agent_session and external_agent_command is None:
                raise ValueError("persistent_agent_session_requires_agent_command")
            _validate_suite_comparison_args(
                baseline_agent=args.baseline_agent,
                candidate_agent=args.candidate_agent,
                actor_ids=actor_ids,
                external_agent_command=external_agent_command,
                external_agent_actor=args.external_agent_actor,
            )

            baseline_result_bundle = tuple(
                run_benchmark_lifecycle(
                    BenchmarkRunnerConfig(
                        run_id=f"cli-suite-{scenario_name}",
                        benchmark_id=args.benchmark_id,
                        scenario=_SCENARIO_PRESETS[scenario_name],
                        actor_ids=actor_ids,
                    )
                )
                for scenario_name in _resolve_suite_scenarios(args.suite)
            )
            if args.baseline_agent is None and args.candidate_agent is None and external_agent_command is None:
                report_payload = build_tiny_suite_baseline_report(baseline_result_bundle)
                response_actor_ids = actor_ids
            elif external_agent_command is None:
                report_payload = build_tiny_suite_comparison_report(
                    baseline_result_bundle,
                    baseline_agent_id=args.baseline_agent,
                    candidate_agent_id=args.candidate_agent,
                )
                response_actor_ids = actor_ids
            elif args.external_agent_actor is not None:
                mixed_result_bundle = tuple(
                    run_benchmark_lifecycle(
                        BenchmarkRunnerConfig(
                            run_id=f"cli-suite-mixed-{scenario_name}",
                            benchmark_id=args.benchmark_id,
                            scenario=_SCENARIO_PRESETS[scenario_name],
                            actor_ids=actor_ids,
                            external_agent_command=external_agent_command,
                            external_agent_actor_id=args.external_agent_actor,
                            persistent_agent_session=args.persistent_agent_session,
                        )
                    )
                    for scenario_name in _resolve_suite_scenarios(args.suite)
                )
                report_payload = build_tiny_suite_mixed_external_comparison_report(
                    mixed_result_bundle,
                    baseline_agent_id=args.baseline_agent,
                    external_actor_id=args.external_agent_actor,
                    external_agent_id=_EXTERNAL_COMPARISON_AGENT_ID,
                )
                response_actor_ids = (args.baseline_agent, _EXTERNAL_COMPARISON_AGENT_ID)
            else:
                external_result_bundle = tuple(
                    run_benchmark_lifecycle(
                        BenchmarkRunnerConfig(
                            run_id=f"cli-suite-external-{scenario_name}",
                            benchmark_id=args.benchmark_id,
                            scenario=_SCENARIO_PRESETS[scenario_name],
                            actor_ids=actor_ids,
                            external_agent_command=external_agent_command,
                            persistent_agent_session=args.persistent_agent_session,
                        )
                    )
                    for scenario_name in _resolve_suite_scenarios(args.suite)
                )
                report_payload = build_tiny_suite_external_comparison_report(
                    baseline_result_bundle,
                    external_result_bundle,
                    compared_actor_id=args.baseline_agent,
                    external_agent_id=_EXTERNAL_COMPARISON_AGENT_ID,
                )
                response_actor_ids = (args.baseline_agent, _EXTERNAL_COMPARISON_AGENT_ID)
        except (ValueError, RuntimeError) as exc:
            error_payload = {
                "accepted": False,
                "error_type": "suite_rejected",
                "reason": str(exc),
            }
            print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
            return 1

        response_payload = _build_suite_response(
            suite_id=args.suite,
            benchmark_id=args.benchmark_id,
            actor_ids=response_actor_ids,
            report_payload=report_payload,
        )
        rendered_output = _render_cli_output(response_payload, output_format=args.output)
        if args.output_file is not None:
            output_path = Path(args.output_file)
            try:
                output_path.write_text(rendered_output + "\n", encoding="utf-8")
            except OSError as exc:
                error_payload = {
                    "accepted": False,
                    "error_type": "suite_rejected",
                    "reason": f"output_file_write_failed:{args.output_file}:{exc.strerror or 'unknown_error'}",
                }
                print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
                return 1
            manifest_payload = _build_suite_output_manifest(
                response_payload=response_payload,
                artifact_path=output_path,
            )
            manifest_path = _resolve_suite_manifest_path(output_path)
            try:
                manifest_path.write_text(
                    json.dumps(manifest_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n",
                    encoding="utf-8",
                )
            except OSError as exc:
                error_payload = {
                    "accepted": False,
                    "error_type": "suite_rejected",
                    "reason": (
                        f"output_manifest_write_failed:{manifest_path}:{exc.strerror or 'unknown_error'}"
                    ),
                }
                print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
                return 1
        print(rendered_output)
        return 0
    if args.command == "reports":
        try:
            if args.reports_command == "list":
                response_payload = _build_reports_list_response(Path(args.dir))
            elif args.reports_command == "show":
                response_payload = _build_reports_show_response(Path(args.manifest))
            else:
                raise ValueError(f"unsupported reports command: {args.reports_command}")
        except ValueError as exc:
            error_payload = {
                "accepted": False,
                "error_type": "reports_rejected",
                "reason": str(exc),
            }
            print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
            return 1

        print(_render_cli_output(response_payload, output_format=args.output))
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


def _build_suite_response(
    *,
    suite_id: str,
    benchmark_id: str,
    actor_ids: Sequence[str],
    report_payload: Mapping[str, Any],
) -> dict[str, Any]:
    return {
        "accepted": True,
        "suite_id": suite_id,
        "benchmark_id": benchmark_id,
        "actor_ids": list(actor_ids),
        "report": report_payload,
    }


def _build_suite_output_manifest(
    *,
    response_payload: Mapping[str, Any],
    artifact_path: Path,
) -> dict[str, Any]:
    report_payload = response_payload["report"]
    report_schema_version = str(report_payload["schema_version"])
    if report_schema_version == "tiny_suite_baseline_report_v1":
        entries = report_payload["entries"]
        scenario_ids = sorted({str(entry["scenario_id"]) for entry in entries})
        actor_ids = sorted({str(entry["agent_id"]) for entry in entries})
        replay_present = all(bool(entry.get("replay_ref")) for entry in entries)
        parity_present = all(bool(entry.get("parity_ref")) for entry in entries)
        command_mode = "suite_baseline"
    elif report_schema_version == "tiny_suite_comparison_report_v1":
        comparisons = report_payload["comparisons"]
        scenario_ids = [str(entry["scenario_id"]) for entry in comparisons]
        actor_ids = [
            str(report_payload["baseline_agent_id"]),
            str(report_payload["candidate_agent_id"]),
        ]
        replay_present = all(
            bool(side.get("replay_ref"))
            for entry in comparisons
            for side in (entry["baseline"], entry["candidate"])
        )
        parity_present = all(
            bool(side.get("parity_ref"))
            for entry in comparisons
            for side in (entry["baseline"], entry["candidate"])
        )
        command_mode = "suite_comparison"
    else:
        raise ValueError(f"unsupported suite report schema_version: {report_schema_version}")

    return {
        "artifact_type": "suite_report_manifest_v1",
        "command_mode": command_mode,
        "artifact_path": str(artifact_path),
        "benchmark_id": response_payload["benchmark_id"],
        "suite_id": response_payload["suite_id"],
        "scenario_ids": scenario_ids,
        "actor_ids": actor_ids,
        "has_replay_refs": replay_present,
        "has_parity_refs": parity_present,
        "report_schema_version": report_schema_version,
    }


def _resolve_suite_manifest_path(output_path: Path) -> Path:
    if output_path.suffix:
        return output_path.with_suffix(output_path.suffix + ".manifest.json")
    return Path(str(output_path) + ".manifest.json")


def _build_reports_list_response(directory_path: Path) -> dict[str, Any]:
    if not directory_path.exists():
        raise ValueError(f"reports_dir_not_found:{directory_path}")
    if not directory_path.is_dir():
        raise ValueError(f"reports_dir_not_directory:{directory_path}")

    artifact_summaries = [
        _load_saved_suite_report_artifact(manifest_path)["artifact"]
        for manifest_path in sorted(directory_path.glob("*.manifest.json"))
    ]
    return {
        "accepted": True,
        "command": "reports_list",
        "directory": str(directory_path),
        "artifact_count": len(artifact_summaries),
        "artifacts": artifact_summaries,
    }


def _build_reports_show_response(manifest_path: Path) -> dict[str, Any]:
    loaded_artifact = _load_saved_suite_report_artifact(manifest_path)
    return {
        "accepted": True,
        "command": "reports_show",
        "artifact": loaded_artifact["artifact"],
        "manifest": loaded_artifact["manifest"],
        "report": loaded_artifact["report"],
    }


def _load_saved_suite_report_artifact(manifest_path: Path) -> dict[str, Any]:
    manifest_payload = _read_json_mapping(manifest_path, missing_prefix="report_manifest_not_found")
    artifact_type = manifest_payload.get("artifact_type")
    if artifact_type != "suite_report_manifest_v1":
        raise ValueError(f"report_manifest_unsupported_artifact_type:{manifest_path}:{artifact_type}")

    artifact_path_value = manifest_payload.get("artifact_path")
    if not isinstance(artifact_path_value, str) or artifact_path_value == "":
        raise ValueError(f"report_manifest_missing_artifact_path:{manifest_path}")
    report_path = Path(artifact_path_value)
    report_payload = _read_json_mapping(report_path, missing_prefix="report_file_not_found")
    if report_payload.get("accepted") is not True:
        raise ValueError(f"report_file_invalid_payload:{report_path}")

    return {
        "artifact": {
            "report_path": str(report_path),
            "manifest_path": str(manifest_path),
            "artifact_type": artifact_type,
            "command_mode": manifest_payload.get("command_mode"),
            "suite_id": manifest_payload.get("suite_id"),
            "benchmark_id": manifest_payload.get("benchmark_id"),
        },
        "manifest": manifest_payload,
        "report": report_payload,
    }


def _read_json_mapping(path: Path, *, missing_prefix: str) -> Mapping[str, Any]:
    try:
        raw_payload = path.read_text(encoding="utf-8")
    except OSError as exc:
        raise ValueError(f"{missing_prefix}:{path}:{exc.strerror or 'unknown_error'}") from exc

    try:
        parsed_payload = json.loads(raw_payload)
    except json.JSONDecodeError as exc:
        raise ValueError(f"json_payload_invalid:{path}:{exc.msg}") from exc

    if not isinstance(parsed_payload, Mapping):
        raise ValueError(f"json_payload_not_mapping:{path}")
    return parsed_payload


def _resolve_suite_scenarios(suite_id: str) -> tuple[str, ...]:
    if suite_id == "tiny":
        return _TINY_SUITE_SCENARIOS
    raise ValueError(f"unsupported suite: {suite_id}")


def _validate_suite_comparison_args(
    *,
    baseline_agent: str | None,
    candidate_agent: str | None,
    actor_ids: Sequence[str],
    external_agent_command: Sequence[str] | None,
    external_agent_actor: str | None,
) -> None:
    if external_agent_command is None and baseline_agent is None and candidate_agent is None:
        return
    if external_agent_command is not None:
        if baseline_agent is None:
            raise ValueError("baseline_agent must be provided for suite external comparison")
        configured_actor_ids = tuple(sorted(actor_ids))
        if baseline_agent not in _BUILTIN_COMPARISON_AGENT_IDS:
            raise ValueError(f"unsupported baseline_agent: {baseline_agent}")
        if baseline_agent not in configured_actor_ids:
            raise ValueError("baseline_agent must be present in configured actor_ids")
        if external_agent_actor is not None:
            if candidate_agent is not None:
                raise ValueError("candidate_agent is not supported with mixed suite external comparison")
            if external_agent_actor not in _BUILTIN_COMPARISON_AGENT_IDS:
                raise ValueError(f"unsupported external_agent_actor: {external_agent_actor}")
            if external_agent_actor not in configured_actor_ids:
                raise ValueError("external_agent_actor must be present in configured actor_ids")
            if external_agent_actor == baseline_agent:
                raise ValueError("external_agent_actor must differ from baseline_agent")
            return
        if candidate_agent is not None:
            raise ValueError("candidate_agent is not supported with suite external comparison")
        return
    if external_agent_actor is not None:
        raise ValueError("external_agent_actor requires agent_command")
    if baseline_agent is None or candidate_agent is None:
        raise ValueError("baseline_agent and candidate_agent must both be provided for suite comparison")
    if baseline_agent not in _BUILTIN_COMPARISON_AGENT_IDS:
        raise ValueError(f"unsupported baseline_agent: {baseline_agent}")
    if candidate_agent not in _BUILTIN_COMPARISON_AGENT_IDS:
        raise ValueError(f"unsupported candidate_agent: {candidate_agent}")
    if baseline_agent == candidate_agent:
        raise ValueError("baseline_agent and candidate_agent must differ")
    configured_actor_ids = tuple(sorted(actor_ids))
    if baseline_agent not in configured_actor_ids or candidate_agent not in configured_actor_ids:
        raise ValueError("baseline_agent and candidate_agent must be present in configured actor_ids")


def _resolve_run_scenario_payload(
    *,
    scenario_name: str,
    scenario_file: str | None,
) -> Mapping[str, Any]:
    if scenario_file is None:
        return _SCENARIO_PRESETS[scenario_name]

    file_path = Path(scenario_file)
    try:
        raw_payload = file_path.read_text(encoding="utf-8")
    except OSError as exc:
        raise ValueError(f"scenario_file_read_failed:{file_path}:{exc.strerror or 'unknown_error'}") from exc

    try:
        parsed_payload = json.loads(raw_payload)
    except json.JSONDecodeError as exc:
        raise ValueError(f"scenario_file_invalid_json:{file_path}:{exc.msg}") from exc

    if not isinstance(parsed_payload, Mapping):
        raise ValueError(f"scenario_file_payload_not_mapping:{file_path}")
    return parsed_payload


def _resolve_external_agent_command(agent_command: str | None) -> tuple[str, ...] | None:
    if agent_command is None:
        return None
    try:
        parsed_command = tuple(shlex.split(agent_command))
    except ValueError as exc:
        raise ValueError(f"external_agent_command_invalid:{exc}") from exc
    if len(parsed_command) == 0:
        raise ValueError("external_agent_command_empty")
    return parsed_command


def _render_cli_output(payload: Mapping[str, Any], *, output_format: str) -> str:
    if output_format == "pretty":
        return json.dumps(payload, sort_keys=True, indent=2, ensure_ascii=True)
    return json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


if __name__ == "__main__":
    raise SystemExit(main())
