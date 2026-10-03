use cockpit_verification::development_cycle_cost::build_report;
use serde_json::json;

#[test]
fn three_stages_keep_time_operations_and_rejections_separate() {
    let report = build_report(&json!({
        "environment": {"machine": "test", "toolchain": "rust"},
        "captures": [
            {"stage": "contract_to_reviewable", "elapsedMs": 120, "agentOperations": 4, "preflightRejects": 1},
            {"stage": "contract_to_reviewable", "elapsedMs": 80, "agentOperations": 3, "preflightRejects": 0},
            {"stage": "verification_to_finish", "elapsedMs": 240, "agentOperations": 2, "preflightRejects": 0}
        ]
    })).expect("report");
    assert_eq!(report["schemaVersion"], 1);
    assert_eq!(report["kind"], "development-cycle-cost");
    assert_eq!(report["stages"][0]["rawSamplesMs"], json!([120.0, 80.0]));
    assert_eq!(report["stages"][0]["p50Ms"], json!(80.0));
    assert_eq!(report["stages"][0]["p95Ms"], json!(120.0));
    assert_eq!(
        report["stages"][0]["agentOperationsBinding"],
        json!({"available": true, "values": [4, 3]})
    );
    assert_eq!(
        report["stages"][0]["preflightRejectsBinding"],
        json!({"available": true, "values": [1, 0]})
    );
    assert_eq!(report["stages"][2]["reason"], "stage_not_captured");
    assert_eq!(report["stages"][2]["p95Ms"], serde_json::Value::Null);
}

#[test]
fn unavailable_stage_and_invalid_capture_never_become_zero() {
    let report = build_report(&json!({
        "environment": {},
        "captures": [
            {"stage": "post_merge_cleanup", "elapsedMs": 12, "valid": false},
            {"stage": "post_merge_cleanup", "elapsedMs": 0, "valid": true}
        ]
    }))
    .expect("report");
    assert_eq!(report["stages"][0]["reason"], "stage_not_captured");
    assert_eq!(report["stages"][2]["rawSamplesMs"], json!([0.0]));
    assert_eq!(report["stages"][2]["p50Ms"], json!(0.0));
    assert_eq!(
        report["stages"][2]["agentOperationsBinding"],
        json!({"available": false, "reason": "agent operation count not persisted for one or more captures"})
    );
    let unavailable = build_report(&json!({"environment": {}, "captures": [
        {"stage": "post_merge_cleanup", "elapsedMs": 12, "valid": false}
    ]}))
    .expect("report");
    assert_eq!(unavailable["stages"][2]["reason"], "no_valid_stage_samples");
    assert_eq!(unavailable["stages"][2]["p50Ms"], serde_json::Value::Null);
}

#[test]
fn samples_round_to_three_decimals_and_use_nearest_rank() {
    let captures = [1.2344, 1.2346, 5.0, 9.0, 20.0]
        .into_iter()
        .map(|elapsed| json!({"stage": "contract_to_reviewable", "elapsedMs": elapsed, "agentOperations": 0, "preflightRejects": 0}))
        .collect::<Vec<_>>();
    let report =
        build_report(&json!({"environment": {"host": "fixture"}, "captures": captures})).unwrap();
    assert_eq!(
        report["stages"][0]["rawSamplesMs"],
        json!([1.234, 1.235, 5.0, 9.0, 20.0])
    );
    assert_eq!(report["stages"][0]["p50Ms"], json!(5.0));
    assert_eq!(report["stages"][0]["p95Ms"], json!(20.0));
}

#[test]
fn decimal_rounding_matches_the_python_oracle_at_binary_half_edges() {
    let captures = [1.2345, 1.2355, 2.6755, 0.0005, 0.0015, 123.4565]
        .into_iter()
        .map(|elapsed| json!({"stage": "contract_to_reviewable", "elapsedMs": elapsed}))
        .collect::<Vec<_>>();
    let report = build_report(&json!({"environment": {}, "captures": captures})).unwrap();
    assert_eq!(
        report["stages"][0]["rawSamplesMs"],
        json!([1.234, 1.236, 2.675, 0.001, 0.002, 123.457])
    );
}

#[test]
fn malformed_environment_stage_and_sample_fail_closed() {
    assert_eq!(
        build_report(&json!({"captures": []})).unwrap_err(),
        "environment identity is required"
    );
    assert_eq!(
        build_report(&json!({"environment": {}, "captures": {}})).unwrap_err(),
        "captures must be an array"
    );
    assert_eq!(
        build_report(&json!({"environment": {}, "captures": [{"stage": "unknown"}]})).unwrap_err(),
        "each capture must name one known stage"
    );
    for elapsed in [json!(-1), json!(true), json!("1"), json!(null)] {
        let input = json!({"environment": {}, "captures": [{"stage": "contract_to_reviewable", "elapsedMs": elapsed}]});
        assert_eq!(
            build_report(&input).unwrap_err(),
            "cycle samples must be finite non-negative numbers"
        );
    }
}

#[test]
fn complete_report_matches_the_python_oracle_document() {
    let capture = json!({
        "stage": "contract_to_reviewable",
        "elapsedMs": 0,
        "agentOperations": 0,
        "preflightRejects": 0,
        "label": "kept as provenance"
    });
    let report = build_report(&json!({
        "environment": {"machine": "fixture"},
        "captures": [capture.clone()]
    }))
    .unwrap();
    assert_eq!(
        report,
        json!({
            "schemaVersion": 1,
            "kind": "development-cycle-cost",
            "environment": {"machine": "fixture"},
            "stages": [
                {
                    "stage": "contract_to_reviewable",
                    "sourceRecords": [capture],
                    "rawSamplesMs": [0.0],
                    "rawSampleCount": 1,
                    "agentOperations": [0],
                    "preflightRejects": [0],
                    "agentOperationsBinding": {"available": true, "values": [0]},
                    "preflightRejectsBinding": {"available": true, "values": [0]},
                    "available": true,
                    "p50Ms": 0.0,
                    "p95Ms": 0.0
                },
                {
                    "stage": "verification_to_finish",
                    "sourceRecords": [],
                    "available": false,
                    "reason": "stage_not_captured",
                    "rawSamplesMs": [],
                    "rawSampleCount": 0,
                    "p50Ms": null,
                    "p95Ms": null,
                    "agentOperations": [],
                    "preflightRejects": [],
                    "agentOperationsBinding": {"available": false, "reason": "stage_not_captured"},
                    "preflightRejectsBinding": {"available": false, "reason": "stage_not_captured"}
                },
                {
                    "stage": "post_merge_cleanup",
                    "sourceRecords": [],
                    "available": false,
                    "reason": "stage_not_captured",
                    "rawSamplesMs": [],
                    "rawSampleCount": 0,
                    "p50Ms": null,
                    "p95Ms": null,
                    "agentOperations": [],
                    "preflightRejects": [],
                    "agentOperationsBinding": {"available": false, "reason": "stage_not_captured"},
                    "preflightRejectsBinding": {"available": false, "reason": "stage_not_captured"}
                }
            ],
            "measurementBoundary": {
                "contractToReviewable": "from Contract creation to the first state where a Draft PR may be created",
                "verificationToFinish": "from verification start to a successful finish boundary",
                "postMergeCleanup": "from reviewed merge observation to exact resource cleanup",
                "overlap": "not inferred; each stage is reported from its own capture"
            }
        })
    );
}
