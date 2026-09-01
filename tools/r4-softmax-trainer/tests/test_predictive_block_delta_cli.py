"""Focused path, parser, and dispatch checks for the predictive delta gate."""

from __future__ import annotations

import sys
import unittest
from contextlib import redirect_stderr
from io import StringIO
from pathlib import Path
from unittest import mock

from r4_softmax_trainer import cli
from r4_softmax_trainer.paths import (
    default_language_path_root,
    default_learned_associative_readout_root,
    default_predictive_block_delta_frame_sidecar,
    default_predictive_block_delta_predecessor,
    default_predictive_block_delta_revealed_v4_root,
    default_predictive_block_delta_root,
)
from r4_softmax_trainer.predictive_block_delta_campaign import MAXIMUM_UPDATES


class PredictiveBlockDeltaCliTests(unittest.TestCase):
    def test_defaults_bind_the_new_root_predecessor_v4_and_geometry(self) -> None:
        root = default_predictive_block_delta_root()
        self.assertEqual(root.name, "issue-973-predictive-block-delta-v1")
        self.assertEqual(
            default_predictive_block_delta_predecessor(), default_language_path_root()
        )
        self.assertEqual(
            default_predictive_block_delta_revealed_v4_root(),
            default_learned_associative_readout_root(),
        )
        self.assertEqual(
            default_predictive_block_delta_frame_sidecar(),
            root / "geometry" / "h4-spin-frames.json",
        )

        arguments = cli.parser().parse_args(["preflight-predictive-block-delta"])
        self.assertEqual(arguments.predecessor_root, default_language_path_root())
        self.assertEqual(
            arguments.revealed_v4_root, default_learned_associative_readout_root()
        )
        self.assertEqual(arguments.frame_sidecar, default_predictive_block_delta_frame_sidecar())
        self.assertEqual(arguments.maximum_updates, MAXIMUM_UPDATES)

    def test_explicit_overrides_dispatch_and_update_bound_fails_closed(self) -> None:
        root = Path("/tmp/predictive-root")
        predecessor = Path("/tmp/predictive-predecessor")
        revealed = Path("/tmp/predictive-v4")
        frames = Path("/tmp/predictive-frames.json")
        result = {"verdict": "TEST_ONLY"}
        arguments = [
            "r4-softmax-trainer",
            "--root",
            str(root),
            "preflight-predictive-block-delta",
            "--predecessor-root",
            str(predecessor),
            "--revealed-v4-root",
            str(revealed),
            "--frame-sidecar",
            str(frames),
            "--maximum-updates",
            "7",
        ]
        with (
            mock.patch.object(sys, "argv", arguments),
            mock.patch.object(
                cli, "run_predictive_block_delta_preflight", return_value=result
            ) as run,
            mock.patch.object(cli, "_print_result") as print_result,
        ):
            cli.main()
        run.assert_called_once_with(
            root=root.resolve(),
            predecessor_root=predecessor.resolve(),
            revealed_v4_root=revealed.resolve(),
            frame_sidecar_path=frames.resolve(),
            maximum_updates=7,
        )
        print_result.assert_called_once_with(result)

        for invalid in (0, MAXIMUM_UPDATES + 1):
            command = cli.parser()
            with redirect_stderr(StringIO()), self.assertRaises(SystemExit):
                command.parse_args(
                    [
                        "preflight-predictive-block-delta",
                        "--maximum-updates",
                        str(invalid),
                    ]
                )


if __name__ == "__main__":
    unittest.main()
