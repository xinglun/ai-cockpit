import unittest

from development_cycle_cost import build_report


class DevelopmentCycleCostTests(unittest.TestCase):
    def test_stages_keep_elapsed_operations_and_rejections_separate(self):
        report = build_report(
            {
                "environment": {"machine": "test", "toolchain": "test"},
                "captures": [
                    {
                        "stage": "contract_to_reviewable",
                        "elapsedMs": 120,
                        "agentOperations": 4,
                        "preflightRejects": 1,
                    },
                    {
                        "stage": "contract_to_reviewable",
                        "elapsedMs": 80,
                        "agentOperations": 3,
                        "preflightRejects": 0,
                    },
                    {
                        "stage": "verification_to_finish",
                        "elapsedMs": 240,
                        "agentOperations": 2,
                        "preflightRejects": 0,
                    },
                ],
            }
        )
        contract = report["stages"][0]
        self.assertEqual(contract["rawSamplesMs"], [120.0, 80.0])
        self.assertEqual(contract["p50Ms"], 80.0)
        self.assertEqual(contract["p95Ms"], 120.0)
        self.assertEqual(contract["agentOperations"], [4, 3])
        self.assertEqual(contract["preflightRejects"], [1, 0])
        self.assertEqual(contract["agentOperationsBinding"], {"available": True, "values": [4, 3]})
        self.assertEqual(contract["preflightRejectsBinding"], {"available": True, "values": [1, 0]})
        self.assertIsNone(report["stages"][2]["p95Ms"])
        self.assertEqual(report["stages"][2]["reason"], "stage_not_captured")

    def test_invalid_samples_are_not_replaced_with_zero(self):
        report = build_report(
            {
                "environment": {"machine": "test"},
                "captures": [
                    {
                        "stage": "post_merge_cleanup",
                        "elapsedMs": 12,
                        "valid": False,
                    }
                ],
            }
        )
        cleanup = report["stages"][2]
        self.assertFalse(cleanup["available"])
        self.assertIsNone(cleanup["p50Ms"])
        self.assertEqual(cleanup["reason"], "no_valid_stage_samples")


if __name__ == "__main__":
    unittest.main()
