import unittest

import numpy as np

from tasks.router_retrieval_eval import (
    augment_route_keys_with_complex,
    compute_amortized_retrieval_metrics,
    dense_retrieval,
    routed_retrieval,
    routed_retrieval_grouped_same_bucket,
)


class RouterRetrievalEvalTest(unittest.TestCase):
    def test_dense_retrieval_uses_full_candidate_set(self):
        train_z = np.array([[1.0, 0.0], [0.9, 0.1], [0.0, 1.0]], dtype=np.float64)
        train_y = np.array([[1.0, 0.0], [1.0, 0.0], [0.0, 1.0]], dtype=np.float64)
        train_tok = np.array([10, 10, 20], dtype=np.int32)
        eval_z = np.array([[1.0, 0.0]], dtype=np.float64)

        yhat, pred_tok, cand_mean, cand_frac = dense_retrieval(
            train_z, train_y, train_tok, eval_z, topk=2, block_size=1
        )

        self.assertEqual(pred_tok.tolist(), [10])
        self.assertAlmostEqual(cand_mean, 3.0)
        self.assertAlmostEqual(cand_frac, 1.0)
        self.assertGreater(yhat[0, 0], yhat[0, 1])

    def test_routed_retrieval_reduces_candidates(self):
        train_keys = [(0, 0), (0, 0), (0, 1), (0, 1)]
        eval_keys = [(0, 0)]
        train_z = np.array([
            [1.0, 0.0],
            [0.9, 0.1],
            [0.0, 1.0],
            [0.1, 0.9],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 10, 20, 20], dtype=np.int32)
        eval_z = np.array([[1.0, 0.0]], dtype=np.float64)

        yhat, pred_tok, cand_mean, cand_frac, probe_mean, fallback = routed_retrieval(
            train_keys, eval_keys, train_z, train_y, train_tok, eval_z, topk=1, probe_buckets=1
        )

        self.assertEqual(pred_tok.tolist(), [10])
        self.assertAlmostEqual(cand_mean, 2.0)
        self.assertAlmostEqual(cand_frac, 0.5)
        self.assertAlmostEqual(probe_mean, 1.0)
        self.assertAlmostEqual(fallback, 0.0)
        self.assertGreater(yhat[0, 0], yhat[0, 1])

    def test_grouped_same_bucket_matches_probe1_path(self):
        train_keys = [(0, 0), (0, 0), (0, 1), (0, 1)]
        eval_keys = [(0, 0), (0, 1)]
        train_z = np.array([
            [1.0, 0.0],
            [0.9, 0.1],
            [0.0, 1.0],
            [0.1, 0.9],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 10, 20, 20], dtype=np.int32)
        eval_z = np.array([[1.0, 0.0], [0.0, 1.0]], dtype=np.float64)

        grouped = routed_retrieval_grouped_same_bucket(
            train_keys, eval_keys, train_z, train_y, train_tok, eval_z, topk=1
        )
        regular = routed_retrieval(
            train_keys, eval_keys, train_z, train_y, train_tok, eval_z, topk=1, probe_buckets=1
        )

        for a, b in zip(grouped[:2], regular[:2]):
            np.testing.assert_allclose(a, b)
        self.assertAlmostEqual(grouped[2], regular[2])
        self.assertAlmostEqual(grouped[3], regular[3])
        self.assertAlmostEqual(grouped[4], regular[4])
        self.assertAlmostEqual(grouped[5], regular[5])

    def test_complex_route_keys_add_secondary_component(self):
        base_keys = [(0, 1), (0, 1), (1, 0)]
        field = np.array([
            [1.0, 0.0],
            [0.0, 1.0],
            [-1.0, 0.0],
        ], dtype=np.float64)
        keys, secondary_count = augment_route_keys_with_complex(
            base_keys=base_keys,
            field=field,
            dim_i=0,
            dim_j=1,
            roots=4,
            radius_bins=1,
        )

        self.assertEqual(len(keys[0]), 3)
        self.assertEqual(keys[0][:2], base_keys[0])
        self.assertGreaterEqual(secondary_count, 2)

    def test_complex_route_keys_reduce_candidates_when_secondary_matches(self):
        train_keys = [(0, 0, 0), (0, 0, 0), (0, 0, 1), (0, 1, 0)]
        eval_keys = [(0, 0, 1)]
        train_z = np.array([
            [1.0, 0.0],
            [0.9, 0.1],
            [0.0, 1.0],
            [0.1, 0.9],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 10, 20, 20], dtype=np.int32)
        eval_z = np.array([[0.0, 1.0]], dtype=np.float64)

        yhat, pred_tok, cand_mean, cand_frac, probe_mean, fallback = routed_retrieval(
            train_keys, eval_keys, train_z, train_y, train_tok, eval_z, topk=1, probe_buckets=1
        )

        self.assertEqual(pred_tok.tolist(), [20])
        self.assertAlmostEqual(cand_mean, 1.0)
        self.assertAlmostEqual(cand_frac, 0.25)
        self.assertAlmostEqual(probe_mean, 1.0)
        self.assertAlmostEqual(fallback, 0.0)
        self.assertGreater(yhat[0, 1], yhat[0, 0])

    def test_amortized_retrieval_metrics_scale_with_repeat_count(self):
        online_per_repeat, amortized = compute_amortized_retrieval_metrics(offline_total=8.0, online_total=4.0, query_repeats=4)
        self.assertAlmostEqual(online_per_repeat, 1.0)
        self.assertAlmostEqual(amortized, 3.0)


if __name__ == "__main__":
    unittest.main()
