use cockpit_verification::gate_plan::{GatePlanError, failure_metadata, load_manifest};

#[test]
fn manifest_rejects_an_explicitly_empty_covers_list_like_python() {
    let manifest = br#"{
      "gates": [{
        "category": "ci",
        "command": ["true"],
        "covers": [],
        "id": "ci_light",
        "minimumProfile": "light"
      }],
      "pathProfiles": {
        "light": ["docs/**"],
        "standard": ["src/**"],
        "strict": [".github/**"]
      },
      "profileOrder": ["light", "standard", "strict"],
      "releaseOwnedPatterns": ["release/**"],
      "schemaVersion": 2,
      "stageFloors": {
        "task": "light",
        "pre_ci": "light",
        "pull_request": "light",
        "merge": "strict",
        "release": "strict"
      },
      "unknownProfile": "strict"
    }"#;

    let error = load_manifest(manifest).expect_err("covers: [] must fail closed");
    assert!(matches!(error, GatePlanError::Manifest(detail) if detail.contains("covers")));
}

#[test]
fn failure_metadata_keeps_the_python_route_codes_and_remediations() {
    let cases = [
        (
            "lifecycle_transition_stale: stale",
            "lifecycle_transition_stale",
            "use the Runtime recovery path, refresh evidence, and push only the repaired state",
        ),
        (
            "lifecycle_transition_invalid: invalid",
            "lifecycle_transition_invalid",
            "restore the declared lifecycle order and checkpoint/preflight bindings before pushing",
        ),
        (
            "required_evidence_missing",
            "required_evidence_missing",
            "collect the Contract-required evidence and rerun the declared verification",
        ),
        (
            "route receipt does not match current repository facts",
            "quality_route_failed",
            "inspect the bound route receipt and rerun the declared repository gate locally",
        ),
    ];

    for (detail, code, remediation) in cases {
        let failure = failure_metadata(detail);
        assert_eq!(failure.state, "failed");
        assert_eq!(failure.failure_code, code);
        assert_eq!(failure.remediation, remediation);
    }
}
