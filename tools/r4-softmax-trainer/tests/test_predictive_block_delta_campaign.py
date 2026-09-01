from __future__ import annotations

import math
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

import torch

from r4_softmax_trainer import predictive_block_delta_campaign as campaign


def _score(
    intervention: str,
    *,
    gain: float,
    wins: int,
    work: tuple[int, ...] = (64, 64, 4096),
    forbidden_reads: int = 0,
) -> campaign.ProbeScore:
    return campaign.ProbeScore(
        intervention=intervention,
        directions=campaign.PROBE_DIRECTIONS,
        targets=campaign.PROBE_TARGETS,
        mean_gain_nats_per_token=gain,
        wins=wins,
        own_nll_nats_per_token=2.0,
        foreign_nll_nats_per_token=2.0 + gain,
        maximum_head_logits=1.0,
        forbidden_reads=forbidden_reads,
        work_signature=work,
        trace_cid="blake3:" + "1" * 64,
    )


class _DisposableModel:
    def __init__(self) -> None:
        self.values = [
            torch.nn.Parameter(torch.ones(campaign.TRAINABLE_PARAMETERS - 3)),
            torch.nn.Parameter(torch.ones(3)),
        ]

    def trainable_parameters(self):
        return tuple(self.values)


class PredictiveBlockDeltaCampaignTests(unittest.TestCase):
    def test_frozen_probe_dose_and_thresholds(self) -> None:
        self.assertEqual(campaign.PROBE_PAIRS, 32)
        self.assertEqual(campaign.PROBE_DIRECTIONS, 64)
        self.assertEqual(campaign.PROBE_TARGETS, 1_024)
        self.assertEqual(campaign.MAXIMUM_UPDATES, 256)
        self.assertEqual(campaign.TRAINABLE_PARAMETERS, 9_228)
        self.assertAlmostEqual(
            campaign.ABSOLUTE_GAIN_THRESHOLD,
            math.log(2.0) / 16,
        )
        self.assertAlmostEqual(
            campaign.INTERVENTION_LOSS_THRESHOLD,
            math.log(1.5) / 16,
        )

    def test_admission_requires_absolute_wins_delta_and_state(self) -> None:
        native_gain = campaign.ABSOLUTE_GAIN_THRESHOLD + 0.01
        control_gain = native_gain - campaign.INTERVENTION_LOSS_THRESHOLD - 0.01
        decision = campaign.admission_decision(
            native=_score("native", gain=native_gain, wins=52),
            additive=_score("no_delta", gain=control_gain, wins=20),
            state_off=_score("state_off", gain=control_gain, wins=20),
            mechanics={"passed": True},
        )
        self.assertTrue(decision["admitted"])
        self.assertEqual(decision["verdict"], campaign.VERDICT_ADMIT)

        no_delta_loss_missing = campaign.admission_decision(
            native=_score("native", gain=native_gain, wins=52),
            additive=_score("no_delta", gain=native_gain, wins=20),
            state_off=_score("state_off", gain=control_gain, wins=20),
            mechanics={"passed": True},
        )
        self.assertFalse(no_delta_loss_missing["admitted"])
        self.assertEqual(no_delta_loss_missing["verdict"], campaign.VERDICT_REJECT)

    def test_integrity_failure_is_invalid_not_scientific_rejection(self) -> None:
        gain = campaign.ABSOLUTE_GAIN_THRESHOLD + 0.1
        decision = campaign.admission_decision(
            native=_score("native", gain=gain, wins=64),
            additive=_score("no_delta", gain=0.0, wins=0, work=(1,)),
            state_off=_score("state_off", gain=0.0, wins=0),
            mechanics={"passed": True},
        )
        self.assertFalse(decision["admitted"])
        self.assertEqual(decision["verdict"], campaign.VERDICT_INVALID)

    def test_disposable_weights_are_zeroed_and_counted(self) -> None:
        model = _DisposableModel()
        self.assertEqual(
            campaign.destroy_disposable_weights(model),
            campaign.TRAINABLE_PARAMETERS,
        )
        self.assertTrue(
            all(torch.count_nonzero(value).item() == 0 for value in model.values)
        )

    def test_wrong_weight_count_fails_closed(self) -> None:
        model = _DisposableModel()
        model.values = [torch.nn.Parameter(torch.ones(1))]
        with self.assertRaisesRegex(RuntimeError, "count differs"):
            campaign.destroy_disposable_weights(model)

    def test_cached_result_is_verified_without_reopening_v4(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result_path = root / campaign.RESULT_RELATIVE_PATH
            result_path.parent.mkdir(parents=True)
            result = campaign._with_self_cid(
                {
                    "schema": campaign.RESULT_SCHEMA,
                    "issue": campaign.ISSUE,
                    "policy": campaign.POLICY,
                    "admitted": False,
                    "verdict": campaign.VERDICT_REJECT,
                },
                "result_cid",
            )
            result_path.write_bytes(campaign.canonical_json_bytes(result))
            loader = Mock(side_effect=AssertionError("V4 must not reopen"))
            with patch.object(campaign, "load_frozen_probe_inputs", loader):
                observed = campaign.run_predictive_block_delta_preflight(
                    root=root,
                    predecessor_root=root / "predecessor",
                    revealed_v4_root=root / "revealed-v4",
                    frame_sidecar_path=root / "h4-spin-frame.json",
                )
            self.assertEqual(observed, result)
            loader.assert_not_called()

    def test_corrupt_cached_result_fails_before_any_model_construction(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result_path = root / campaign.RESULT_RELATIVE_PATH
            result_path.parent.mkdir(parents=True)
            result_path.write_text("{}\n", encoding="utf-8")
            factory = Mock(side_effect=AssertionError("model must not construct"))
            with self.assertRaisesRegex(ValueError, "result_cid"):
                campaign.run_predictive_block_delta_preflight(
                    root=root,
                    predecessor_root=root / "predecessor",
                    revealed_v4_root=root / "revealed-v4",
                    frame_sidecar_path=root / "h4-spin-frame.json",
                    model_factory=factory,
                )
            factory.assert_not_called()


if __name__ == "__main__":
    unittest.main()
