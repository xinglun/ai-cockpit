#!/usr/bin/env python3
import pathlib
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))

from knowledge_query_benchmark import measurement_record, parse_resource_metrics, summarize_measurements  # noqa: E402


class KnowledgeQueryBenchmarkTest(unittest.TestCase):
    def test_measurement_keeps_projection_validation_and_candidate_access_separate(self):
        payload = {
            "projection": {"materialization": "reused"},
            "metrics": {
                "knowledgeQuery": {
                    "projectionMs": 4.25,
                    "queryMs": 0.31,
                    "candidateRecordAccessCount": 1,
                }
            },
        }
        sample = measurement_record(
            payload,
            elapsed_ms=5.0,
            resource_metrics={
                "rssBytes": {"available": True, "value": 1024},
                "readBytes": {"available": False, "reason": "not_exposed"},
            },
            phase="warm",
        )

        self.assertEqual(sample["phase"], "warm")
        self.assertEqual(sample["elapsedMs"], 5.0)
        self.assertEqual(sample["cacheValidationMs"], 4.25)
        self.assertEqual(sample["queryMs"], 0.31)
        self.assertEqual(sample["candidateRecordAccessCount"], 1)
        self.assertEqual(sample["resourceUsage"]["readBytes"]["reason"], "not_exposed")

    def test_non_reused_projection_does_not_claim_cache_validation(self):
        payload = {
            "projection": {"materialization": "created"},
            "metrics": {
                "knowledgeQuery": {
                    "projectionMs": 12,
                    "queryMs": 1,
                    "candidateRecordAccessCount": 4,
                }
            },
        }
        sample = measurement_record(
            payload,
            elapsed_ms=13,
            resource_metrics={},
            phase="cold",
        )

        self.assertIsNone(sample["cacheValidationMs"])
        self.assertEqual(
            sample["cacheValidationUnavailableReason"],
            "projection_not_reused",
        )

    def test_resource_parser_reports_host_metrics_and_explicit_unavailability(self):
        metrics = parse_resource_metrics(
            "maximum resident set size: 8192\nbytes read: 4096\n",
            platform_name="darwin",
        )

        self.assertEqual(metrics["rssBytes"], {"available": True, "value": 8192})
        self.assertEqual(metrics["readBytes"], {"available": True, "value": 4096})

        unavailable = parse_resource_metrics("", platform_name="unknown")
        self.assertFalse(unavailable["rssBytes"]["available"])
        self.assertFalse(unavailable["readBytes"]["available"])
        self.assertEqual(
            unavailable["rssBytes"]["reason"],
            "host_does_not_expose_resource_metrics",
        )

    def test_summary_retains_raw_samples_and_declares_percentile_availability(self):
        samples = [
            {
                "phase": "cold",
                "elapsedMs": 10,
                "cacheValidationMs": None,
                "queryMs": 1,
                "candidateRecordAccessCount": 10,
            },
            {
                "phase": "warm",
                "elapsedMs": 2,
                "cacheValidationMs": 1.5,
                "queryMs": 0.2,
                "candidateRecordAccessCount": 1,
            },
        ]
        result = summarize_measurements(samples)

        self.assertEqual(result["rawSampleCount"], 2)
        self.assertEqual(result["rawElapsedMs"], [10.0, 2.0])
        self.assertEqual(result["cold"]["sampleCount"], 1)
        self.assertEqual(result["warm"]["sampleCount"], 1)
        self.assertIsNone(result["warm"]["percentiles"]["p50Ms"])
        self.assertIn("unavailableReason", result["warm"]["percentiles"])


if __name__ == "__main__":
    unittest.main()
