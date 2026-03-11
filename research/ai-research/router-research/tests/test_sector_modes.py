import unittest

import numpy as np

import hyperbolic_router_so8 as hr


class TestSectorModes(unittest.TestCase):
    def test_all_sector_modes_produce_valid_ids(self):
        np.random.seed(0)
        n, d, k = 64, 8, 9
        hybrid_local_k = 4
        v = np.random.randn(n, d)
        chart = hr.Chart(R=np.eye(d), s_global=None, S_radial=None, scale_mode="global")
        U = hr.normalize_rows(v)
        C = hr.spherical_kmeans(U, K=k, iters=5, seed=0)

        modes = ["kmeans", "phase2", "phase4d", "phase4d_adaptive", "phase4d_hopf", "phase4d_hopf_base", "phase4d_hopf_iso", "phase4d_hopf_ball", "phase4d_hopf_chi", "phase4d_hopf_fib", "phase4d_hopf_fib_rung", "phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound", "phase4d_hopf_blend", "phase4d_complex_local", "complex2"]
        for mode in modes:
            shell, sector, _u, _z = hr.route_addresses(
                v,
                delta_r=2.0,
                C=C,
                chart=chart,
                sector_mode=mode,
                phase_dim_i=0,
                phase_dim_j=1,
                phase4_dim_i=0,
                phase4_dim_j=1,
                phase4_dim_k=2,
                phase4_dim_l=3,
                complex_dim_i=0,
                complex_dim_j=1,
                K=k,
                time_pressure_lambda=0.0,
                tau=1.0,
                adaptive_min_pair_bins=2,
                adaptive_time_growth=1.0,
                adaptive_balance=1.0,
                adaptive_angle_growth=0.35,
                hybrid_local_k=hybrid_local_k,
                hybrid_complex_roots=4,
                hybrid_local_min_k=1,
                hybrid_local_target=0.60,
                hybrid_local_hysteresis=0.05,
                hybrid_local_converge_lambda=1.0,
            )
            self.assertEqual(shell.shape[0], n)
            self.assertEqual(sector.shape[0], n)
            self.assertTrue(np.all(sector >= 0))
            if mode == "phase4d_complex_local":
                sector_upper = k * hybrid_local_k
            else:
                sector_upper = k
            self.assertTrue(np.all(sector < sector_upper))

    def test_hybrid_route_one_matches_batch_route(self):
        np.random.seed(1)
        v = np.random.randn(8, 8)
        chart = hr.Chart(R=np.eye(8), s_global=None, S_radial=None, scale_mode="global")
        shell, sector, _u, z = hr.route_addresses(
            v,
            delta_r=2.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_complex_local",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=1,
            phase4_dim_k=2,
            phase4_dim_l=3,
            complex_dim_i=4,
            complex_dim_j=5,
            K=9,
            time_pressure_lambda=0.0,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.0,
            adaptive_balance=1.0,
            adaptive_angle_growth=0.35,
            adaptive_shell_growth=0.8,
            adaptive_shell_balance=0.3,
            adaptive_converge_lambda=1.2,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ratio",
            shell_mode="linear",
            hybrid_local_k=4,
            hybrid_complex_roots=4,
            hybrid_local_min_k=1,
            hybrid_local_target=0.60,
            hybrid_local_hysteresis=0.05,
            hybrid_local_converge_lambda=1.0,
        )
        for i in range(v.shape[0]):
            key, z1 = hr.route_one(
                v[i],
                delta_r=2.0,
                C=None,
                chart=chart,
                sector_mode="phase4d_complex_local",
                phase_dim_i=0,
                phase_dim_j=1,
                phase4_dim_i=0,
                phase4_dim_j=1,
                phase4_dim_k=2,
                phase4_dim_l=3,
                complex_dim_i=4,
                complex_dim_j=5,
                K=9,
                time_pressure_lambda=0.0,
                tau=1.0,
                adaptive_min_pair_bins=2,
                adaptive_time_growth=1.0,
                adaptive_balance=1.0,
                adaptive_angle_growth=0.35,
                adaptive_shell_growth=0.8,
                adaptive_shell_balance=0.3,
                adaptive_converge_lambda=1.2,
                adaptive_converge_target=0.85,
                adaptive_converge_hysteresis=0.05,
                adaptive_converge_mode="phi_ratio",
                hybrid_local_k=4,
                hybrid_complex_roots=4,
                hybrid_local_min_k=1,
                hybrid_local_target=0.60,
                hybrid_local_hysteresis=0.05,
                hybrid_local_converge_lambda=1.0,
            )
            self.assertEqual(key, (int(shell[i]), int(sector[i])))
            np.testing.assert_allclose(z1, z[i])

    def test_hybrid_local_controller_can_open_bins(self):
        np.random.seed(7)
        z = np.random.randn(32, 8)
        z[:, 1] *= 4.0
        z[:, 3] *= 4.0
        hybrid = hr.phase4d_complex_local_components(
            z=z,
            K=9,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            complex_dim_i=1,
            complex_dim_j=3,
            delta_r=2.0,
            tau=1.0,
            time_pressure_lambda=0.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.0,
            adaptive_balance=1.0,
            adaptive_angle_growth=0.35,
            adaptive_shell_growth=0.8,
            adaptive_shell_balance=0.3,
            adaptive_converge_lambda=1.2,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ratio",
            shell_mode="linear",
            hybrid_local_k=4,
            hybrid_complex_roots=4,
            hybrid_local_min_k=1,
            hybrid_local_target=0.05,
            hybrid_local_hysteresis=0.0,
            hybrid_local_converge_lambda=0.0,
        )
        self.assertGreater(int(np.max(hybrid["local_k_eff"])), 1)

    def test_hopf_base_sector_is_invariant_to_common_fiber_phase(self):
        np.random.seed(9)
        v = np.random.randn(48, 8)
        chart = hr.Chart(R=np.eye(8), s_global=None, S_radial=None, scale_mode="global")
        shell_a, sector_a, _u1, _z1 = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_base",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=1,
            phase4_dim_k=2,
            phase4_dim_l=3,
            complex_dim_i=0,
            complex_dim_j=1,
            K=25,
            time_pressure_lambda=0.0,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.0,
            adaptive_balance=1.0,
            adaptive_angle_growth=0.35,
            shell_mode="phi_log",
        )

        phi = 0.73
        rot = np.array([[np.cos(phi), -np.sin(phi)], [np.sin(phi), np.cos(phi)]], dtype=np.float64)
        v_shift = v.copy()
        v_shift[:, 0:2] = v_shift[:, 0:2] @ rot.T
        v_shift[:, 2:4] = v_shift[:, 2:4] @ rot.T

        shell_b, sector_b, _u2, _z2 = hr.route_addresses(
            v_shift,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_base",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=1,
            phase4_dim_k=2,
            phase4_dim_l=3,
            complex_dim_i=0,
            complex_dim_j=1,
            K=25,
            time_pressure_lambda=0.0,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.0,
            adaptive_balance=1.0,
            adaptive_angle_growth=0.35,
            shell_mode="phi_log",
        )

        np.testing.assert_array_equal(shell_a, shell_b)
        np.testing.assert_array_equal(sector_a, sector_b)

    def test_phase_coupled_shell_mode_routes_valid_shells(self):
        np.random.seed(11)
        v = np.random.randn(32, 8)
        chart = hr.Chart(R=np.eye(8), s_global=None, S_radial=None, scale_mode="global")
        shell_log, sector_log, _u1, _z1 = hr.route_addresses(
            v,
            delta_r=3.6,
            C=None,
            chart=chart,
            sector_mode="phase4d_adaptive",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            time_pressure_lambda=0.0,
            tau=1.0,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        shell_phase, sector_phase, _u2, _z2 = hr.route_addresses(
            v,
            delta_r=3.6,
            C=None,
            chart=chart,
            sector_mode="phase4d_adaptive",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            time_pressure_lambda=0.0,
            tau=1.0,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_phase",
            shell_phase_coupling=0.3,
        )
        self.assertTrue(np.all(shell_phase >= 0))
        self.assertTrue(np.all(sector_phase >= 0))
        self.assertTrue(np.all(sector_phase < 25))
        self.assertGreater(int(np.count_nonzero(shell_phase != shell_log)), 0)
        np.testing.assert_array_equal(sector_phase.shape, sector_log.shape)

    def test_phase4d_hopf_ball_keeps_sectors_but_changes_shells(self):
        np.random.seed(13)
        v = 0.4 * np.random.randn(32, 8)
        chart = hr.Chart(
            R=np.eye(8),
            s_global=np.full((8,), np.log(1.4), dtype=np.float64),
            S_radial=None,
            scale_mode="global",
        )
        shell_hopf, sector_hopf, _u1, _z1 = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        shell_ball, sector_ball, _u2, _z2 = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_ball",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        np.testing.assert_array_equal(sector_hopf, sector_ball)
        self.assertGreater(int(np.count_nonzero(shell_hopf != shell_ball)), 0)

    def test_phase4d_hopf_iso_matches_hopf_when_chart_is_rotation_only(self):
        np.random.seed(17)
        v = 0.35 * np.random.randn(32, 8)
        q, _ = np.linalg.qr(np.random.randn(8, 8))
        chart = hr.Chart(R=q, s_global=None, S_radial=None, scale_mode="global")
        shell_hopf, sector_hopf, _u1, z_hopf = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        shell_iso, sector_iso, _u2, z_iso = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_iso",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        np.testing.assert_array_equal(shell_hopf, shell_iso)
        np.testing.assert_array_equal(sector_hopf, sector_iso)
        np.testing.assert_allclose(z_hopf, z_iso)

    def test_phase4d_hopf_fib_band_iso_matches_band_when_chart_is_rotation_only(self):
        np.random.seed(19)
        v = 0.35 * np.random.randn(32, 8)
        q, _ = np.linalg.qr(np.random.randn(8, 8))
        chart = hr.Chart(R=q, s_global=None, S_radial=None, scale_mode="global")
        shell_band, sector_band, _u1, z_band = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_fib_band",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        shell_iso, sector_iso, _u2, z_iso = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_fib_band_iso",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        np.testing.assert_array_equal(shell_band, shell_iso)
        np.testing.assert_array_equal(sector_band, sector_iso)
        np.testing.assert_allclose(z_band, z_iso)

    def test_phase4d_hopf_fib_band_bound_lambda_zero_matches_iso(self):
        rs = np.random.RandomState(23)
        v = 0.25 * rs.randn(64, 8)
        chart = hr.Chart(
            R=np.eye(8),
            s_global=np.array([np.log(1.6), np.log(0.8), np.log(1.3), np.log(0.9), 0.0, 0.0, 0.0, 0.0], dtype=np.float64),
            S_radial=None,
            scale_mode="global",
        )
        kwargs = dict(
            delta_r=3.6,
            C=None,
            chart=chart,
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        shell_iso, sector_iso, _u_iso, z_iso = hr.route_addresses(
            v,
            sector_mode="phase4d_hopf_fib_band_iso",
            **kwargs,
        )
        shell_bound, sector_bound, _u_bound, z_bound = hr.route_addresses(
            v,
            sector_mode="phase4d_hopf_fib_band_bound",
            route_scale_lambda=0.0,
            **kwargs,
        )
        np.testing.assert_array_equal(shell_iso, shell_bound)
        np.testing.assert_array_equal(sector_iso, sector_bound)
        np.testing.assert_allclose(z_iso, z_bound)


if __name__ == "__main__":
    unittest.main()
