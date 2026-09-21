use std::{fs, process::Command};

#[test]
fn cli_emits_the_canonical_release_plan_envelope() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("request.json");
    let output = directory.path().join("plan.json");
    fs::write(
        &input,
        serde_json::json!({
            "schemaVersion": 1,
            "repositoryId": "repo-1",
            "event": "workflow_dispatch",
            "headRevision": "head-sha",
            "baseRevision": "base-sha",
            "sourceRevision": "source-sha",
            "version": "0.2.105",
            "fromTag": "v0.2.104",
            "toTag": "v0.2.105",
            "publishExistingTag": false,
            "postReleaseAcceptance": false,
            "closeOnly": false,
            "workItemId": null,
            "sourceWorkItemId": null,
            "contractPath": null,
            "contractDigest": null,
            "sourceContractPath": null,
            "sourceContractDigest": null,
            "recoveryEvidence": [],
            "reuseRunId": null,
            "reuseAcceptanceRunId": null,
            "handoffRunId": "123",
            "requestedMode": null
        })
        .to_string(),
    )
    .expect("request");

    let result = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["release-plan", "--input"])
        .arg(&input)
        .args(["--output"])
        .arg(&output)
        .output()
        .expect("release-plan");
    assert!(
        result.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let envelope: serde_json::Value =
        serde_json::from_slice(&fs::read(&output).expect("plan output")).expect("plan JSON");
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(
        envelope["plan"]["request"]["mode"],
        "independent_public_acceptance"
    );
    assert!(envelope["planDigest"].as_str().is_some());
}
