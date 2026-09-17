import unittest

from paired_performance_report import build_report


def capture(scenario, runtime_digest, *, samples=100, comparison_key="sha256:env"):
    values = [float(index + 1) for index in range(samples)]
    return {
        "schemaVersion": 2,
        "runtimeVersion": "0.2.test",
        "runtimeDigest": runtime_digest,
        "repositoryId": "sha256:" + "a" * 64,
        "binaryDigest": "sha256:" + "b" * 64,
        "scenario": scenario,
        "environment": {
            "comparisonKey": comparison_key,
            "toolchain": {"rustcVersion": {"available": True, "value": "rustc test"}},
            "repository": {"head": "fixture", "dirty": False},
        },
        "cacheInvalidationReasons": {
            "available": False,
            "reason": "runtime_does_not_expose_cache_invalidation_events",
        },
        "diagnostics": {"mode": "off"},
        "samples": [
            {
                "name": "status",
                "validWarmSamplesMs": values,
            }
        ],
    }


class PairedPerformanceReportTests(unittest.TestCase):
    def test_99_valid_warm_samples_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "insufficient valid warm samples"):
            build_report(
                [capture("small-clean", "sha256:" + "c" * 64, samples=99)],
                [capture("small-clean", "sha256:" + "d" * 64, samples=99)],
                noise_budget_ms=1,
            )

    def test_100_samples_preserve_raw_values_and_compare_percentiles(self):
        baseline = capture("small-clean", "sha256:" + "c" * 64)
        candidate = capture("small-clean", "sha256:" + "d" * 64)
        candidate["samples"][0]["validWarmSamplesMs"] = [max(value - 2, 0) for value in candidate["samples"][0]["validWarmSamplesMs"]]
        report = build_report([baseline], [candidate], noise_budget_ms=1)
        self.assertEqual(report["status"], "compared")
        comparison = report["comparisons"][0]
        self.assertEqual(comparison["validWarmSampleCount"], {"baseline": 100, "candidate": 100})
        self.assertEqual(len(comparison["rawWarmSamplesMs"]["baseline"]), 100)
        self.assertEqual(comparison["p50P95Ms"]["candidate"]["p50Ms"], 48.0)
        self.assertEqual(comparison["p50P95Ms"]["candidate"]["p99Ms"], 97.0)
        self.assertEqual(comparison["decision"], "improved")

    def test_missing_noise_budget_is_unknown_not_improvement(self):
        report = build_report(
            [capture("small-clean", "sha256:" + "c" * 64)],
            [capture("small-clean", "sha256:" + "d" * 64)],
        )
        self.assertEqual(report["status"], "unknown")
        self.assertEqual(report["comparisons"][0]["decision"], "unknown")

    def test_environment_mismatch_is_rejected(self):
        candidate = capture("small-clean", "sha256:" + "d" * 64, comparison_key="sha256:other")
        with self.assertRaisesRegex(ValueError, "identity"):
            build_report([capture("small-clean", "sha256:" + "c" * 64)], [candidate], noise_budget_ms=1)

    def test_diagnostics_overhead_is_a_separate_paired_measurement(self):
        off = capture("small-clean", "sha256:" + "c" * 64)
        on = capture("small-clean", "sha256:" + "d" * 64)
        on["samples"][0]["validWarmSamplesMs"] = [value + 3 for value in on["samples"][0]["validWarmSamplesMs"]]
        report = build_report(
            [off],
            [off],
            diagnostics_off=[off],
            diagnostics_on=[on],
        )
        overhead = report["diagnostics"]["onOffOverhead"]
        self.assertTrue(overhead["available"])
        self.assertEqual(overhead["comparisons"][0]["overheadMs"]["p50Ms"], 3.0)


if __name__ == "__main__":
    unittest.main()
