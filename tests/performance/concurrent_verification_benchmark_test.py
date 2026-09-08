#!/usr/bin/env python3
"""Focused tests for the concurrent verification measurement harness."""

import importlib.util
import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location(
    "concurrent_verification_benchmark",
    ROOT / "concurrent_verification_benchmark.py",
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class ConcurrentVerificationBenchmarkTests(unittest.TestCase):
    def test_primary_sample_order_is_retained_before_quantiles(self):
        summary = MODULE.summarize_latencies([120, 20, 22, 21])
        self.assertEqual(summary["rawMs"], [120.0, 20.0, 22.0, 21.0])
        self.assertEqual(summary["sampleCount"], 4)
        self.assertEqual(summary["p50Ms"], 21.0)
        self.assertFalse(summary["reliable"]["p95"])
        self.assertIn("insufficient_samples:4<20", summary["unavailableReason"]["p95"])

    def test_unavailable_metrics_are_not_zero(self):
        value = MODULE.unavailable("platform_metric_not_collected")
        self.assertFalse(value["available"])
        self.assertNotIn("value", value)

    def test_request_arguments_keep_command_identity_material(self):
        vector = MODULE.command_vector(
            pathlib.Path("/runtime"),
            pathlib.Path("/repo"),
            "/python",
            ["-c", "pass"],
        )
        self.assertEqual(vector[-1], "--args=-c,pass")
        self.assertIn("/python", vector)


if __name__ == "__main__":
    unittest.main()
