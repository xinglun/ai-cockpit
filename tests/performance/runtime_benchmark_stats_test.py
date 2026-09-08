#!/usr/bin/env python3
import pathlib
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))

from runtime_benchmark_stats import PERCENTILE_MINIMUMS, QUANTILE_METHOD, summarize  # noqa: E402
from runtime_benchmark_scenarios import scenario_matrix_entry  # noqa: E402


class RuntimeBenchmarkStatsTest(unittest.TestCase):
    def test_first_sample_is_selected_before_percentile_sorting(self):
        summary = summarize("status", [120, 20, 22, 21])

        self.assertEqual(summary["rawSamplesMs"], [120.0, 20.0, 22.0, 21.0])
        self.assertEqual(summary["firstMeasurementMs"], 120.0)
        self.assertEqual(summary["warmSamplesMs"], [20.0, 22.0, 21.0])
        self.assertEqual(summary["warm"]["quantileMethod"], QUANTILE_METHOD)
        self.assertIsNone(summary["warm"]["p95Ms"])
        self.assertEqual(summary["warm"]["unavailableReason"]["p95"], "insufficient_samples:3<20")

    def test_unreliable_percentiles_are_not_replaced_by_elapsed_fallbacks(self):
        summary = summarize("status", list(range(8)))

        self.assertEqual(summary["warm"]["sampleCount"], 7)
        self.assertEqual(summary["warm"]["p50Ms"], 4.0)
        self.assertIsNone(summary["warm"]["p95Ms"])
        self.assertIsNone(summary["warm"]["p99Ms"])
        self.assertTrue(summary["warm"]["reliable"]["p50"])
        self.assertFalse(summary["warm"]["reliable"]["p95"])
        self.assertFalse(summary["warm"]["reliable"]["p99"])
        self.assertEqual(PERCENTILE_MINIMUMS, {"p50": 5, "p95": 20, "p99": 100})

    def test_nearest_rank_percentiles_require_the_declared_minimum(self):
        values = list(range(1, 21))
        summary = summarize("status", [120] + values)

        self.assertEqual(summary["warm"]["sampleCount"], 20)
        self.assertEqual(summary["warm"]["p50Ms"], 10.0)
        self.assertEqual(summary["warm"]["p95Ms"], 19.0)
        self.assertFalse(summary["warm"]["reliable"]["p99"])
        self.assertEqual(summary["warm"]["unavailableReason"]["p99"], "insufficient_samples:20<100")

    def test_empty_samples_are_rejected(self):
        with self.assertRaises(ValueError):
            summarize("status", [])

    def test_clean_scenario_requires_a_clean_repository(self):
        entry = scenario_matrix_entry(
            "small-clean",
            dirty=True,
            tracked_file_count=12,
            changed_path_count=2,
            large_changed_file=False,
            historical_work_item_count=0,
        )

        self.assertEqual(entry["status"], "not_measured")
        self.assertEqual(entry["reason"], "repository_dirty")

    def test_scenario_scale_and_change_facts_are_checked(self):
        small = scenario_matrix_entry(
            "small-clean",
            dirty=False,
            tracked_file_count=12,
            changed_path_count=0,
            large_changed_file=False,
            historical_work_item_count=0,
        )
        many = scenario_matrix_entry(
            "many-files-clean",
            dirty=False,
            tracked_file_count=12,
            changed_path_count=0,
            large_changed_file=False,
            historical_work_item_count=0,
        )
        single = scenario_matrix_entry(
            "single-file-change",
            dirty=True,
            tracked_file_count=12,
            changed_path_count=1,
            large_changed_file=False,
            historical_work_item_count=0,
        )

        self.assertEqual(small["status"], "measured")
        self.assertEqual(many["status"], "not_measured")
        self.assertEqual(many["reason"], "tracked_file_count_below_1000")
        self.assertEqual(single["status"], "measured")

    def test_unsupported_execution_shapes_cannot_be_claimed_as_measured(self):
        entry = scenario_matrix_entry(
            "concurrent-validation-requests",
            dirty=False,
            tracked_file_count=12,
            changed_path_count=0,
            large_changed_file=False,
            historical_work_item_count=0,
        )

        self.assertEqual(entry["status"], "not_measured")
        self.assertEqual(entry["reason"], "harness_does_not_measure_concurrency")


if __name__ == "__main__":
    unittest.main()
