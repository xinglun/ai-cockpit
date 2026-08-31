use cockpit_core::{GovernanceInput, evaluate};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Expected {
    decision_state: String,
    blockers: Vec<String>,
    unknowns: Vec<String>,
    safe_actions: Vec<String>,
    required_checks: Vec<String>,
    authority: String,
    outcome_state: String,
}

fn normalize(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

#[test]
fn canonical_corpus_has_rust_semantic_parity() {
    let cases = [
        "scope-exceeded",
        "unauthorized-destructive-change",
        "missing-evidence",
        "stale-evidence",
        "contradictory-evidence",
        "unsupported-completion",
        "repository-prompt-injection",
        "malicious-deletion",
        "missing-human-authority",
        "invalid-archive",
        "cross-work-item-evidence",
        "unknown-provider-result",
        "test-weakening",
        "coverage-weakening",
    ];
    let fixture_root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance/fixtures");
    for name in cases {
        let case_root = fixture_root.join(name);
        let input: GovernanceInput =
            serde_json::from_slice(&fs::read(case_root.join("input.json")).expect("fixture input"))
                .expect("fixture input JSON");
        let expected: Expected = serde_json::from_slice(
            &fs::read(case_root.join("expected.json")).expect("fixture expected"),
        )
        .expect("fixture JSON");
        let actual = evaluate(input);
        assert_eq!(
            format!("{:?}", actual.state).to_lowercase(),
            expected.decision_state,
            "case {name}"
        );
        assert_eq!(
            normalize(actual.blockers),
            normalize(expected.blockers),
            "blockers for {name}"
        );
        assert_eq!(
            normalize(actual.unknowns),
            normalize(expected.unknowns),
            "unknowns for {name}"
        );
        assert_eq!(
            normalize(actual.safe_actions),
            normalize(expected.safe_actions),
            "safe actions for {name}"
        );
        assert_eq!(
            normalize(actual.required_checks),
            normalize(expected.required_checks),
            "checks for {name}"
        );
        assert_eq!(actual.authority, expected.authority, "authority for {name}");
        assert_eq!(
            actual.outcome_state, expected.outcome_state,
            "outcome for {name}"
        );
    }
}

#[test]
fn corpus_manifest_and_fixture_layout_are_complete() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).expect("manifest"))
            .expect("manifest JSON");
    assert_eq!(manifest["oracle"]["sourceLock"], "reference-source.lock");
    assert_eq!(manifest["oracle"]["offline"]["runtimeInvoked"], false);
    assert_eq!(manifest["oracle"]["executable"]["runtimeInvoked"], false);
    assert_eq!(manifest["oracle"]["localReference"]["runtimeInvoked"], true);
    assert_eq!(
        manifest["oracle"]["executable"]["comparison"],
        manifest["comparison"]
    );
    assert!(root.join("v1_oracle.py").is_file());
    for case in manifest["cases"].as_array().expect("cases") {
        let name = case.as_str().expect("case name");
        let fixture = root.join("fixtures").join(name);
        for required in [
            "repository/material.txt",
            "contract.json",
            "evidence/receipt.json",
            "input.json",
            "expected.json",
        ] {
            assert!(
                fixture.join(required).is_file(),
                "missing {name}/{required}"
            );
        }
    }
}
