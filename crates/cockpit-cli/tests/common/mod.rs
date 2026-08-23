use sha2::{Digest as ShaDigest, Sha256};
use std::{fs, path::Path, process::Command};

pub fn plan(binary: &str, repo: &Path, work_item_id: &str) {
    let context = tempfile::NamedTempFile::new().expect("resource context");
    fs::write(
        context.path(),
        serde_json::to_vec_pretty(&serde_json::json!({
            "branch": format!("feature/{work_item_id}"),
            "worktree": format!("/tmp/removed-{work_item_id}"),
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": format!("https://github.com/example/ai-cockpit/pull/{work_item_id}")
        }))
        .expect("context JSON"),
    )
    .expect("write context");
    let output = Command::new(binary)
        .args([
            "work-item",
            "finalize-plan",
            "--id",
            work_item_id,
            "--input",
        ])
        .arg(context.path())
        .args(["--repo"])
        .arg(repo)
        .current_dir(repo)
        .output()
        .expect("finalize plan");
    assert!(
        output.status.success(),
        "finalize plan stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn record_retained(binary: &str, repo: &Path, work_item_id: &str) {
    let status = Command::new(binary)
        .args(["status", "--repo"])
        .arg(repo)
        .current_dir(repo)
        .output()
        .expect("status");
    assert!(status.status.success());
    let status: serde_json::Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    let repository_id = status["repositoryId"].as_str().expect("repository id");
    let contract_path = repo
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.contract.json"));
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract")).expect("contract");
    let context = contract["resourceContext"].clone();
    let branch = context["branch"].as_str().expect("branch");
    let worktree = context["worktree"].as_str().expect("worktree");
    let pull_request = context["pullRequest"].as_str().expect("pull request");
    let contract_digest = format!(
        "sha256:{}",
        hex::encode(Sha256::digest(
            fs::read(&contract_path).expect("contract bytes")
        ))
    );
    let binary_bytes = fs::read(binary).expect("runtime binary");
    let runtime_digest = format!("sha256:{}", hex::encode(Sha256::digest(binary_bytes)));
    let receipt = serde_json::json!({
        "schemaVersion": 1,
        "receiptId": format!("receipt-{work_item_id}"),
        "operationId": format!("operation-{work_item_id}"),
        "repositoryId": repository_id,
        "workItemId": work_item_id,
        "runtimeVersion": env!("CARGO_PKG_VERSION"),
        "runtimeDigest": runtime_digest,
        "provider": context["provider"],
        "pullRequest": {
            "number": 1,
            "url": pull_request,
            "headRevision": "abcdef1",
            "baseBranch": context["baseBranch"],
            "baseRemote": context["baseRemote"],
            "baseRevision": "abcdef0",
            "mergeCommit": "1234567"
        },
        "branch": {
            "name": branch,
            "remote": context["baseRemote"],
            "headRevision": "abcdef1"
        },
        "worktree": {
            "worktreeId": format!("wt-{work_item_id}"),
            "path": worktree,
            "branch": branch,
            "headRevision": "abcdef1"
        },
        "before": {
            "pullRequest": "merged",
            "branch": "present",
            "worktree": "clean"
        },
        "after": {
            "pullRequest": "merged",
            "branch": "present",
            "worktree": "clean"
        },
        "result": {
            "disposition": "retained",
            "failureCodes": [],
            "unknownCodes": []
        },
        "actor": "human:test",
        "authoritySource": "test-policy",
        "reason": "explicitly retained for test",
        "timestamp": "2026-08-23T00:00:00Z",
        "contractDigest": contract_digest,
        "resourceContext": context
    });
    let receipt_file = tempfile::NamedTempFile::new().expect("resource receipt");
    fs::write(
        receipt_file.path(),
        serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
    )
    .expect("write receipt");
    let output = Command::new(binary)
        .args(["work-item", "finalize", "--id", work_item_id, "--input"])
        .arg(receipt_file.path())
        .args(["--repo"])
        .arg(repo)
        .current_dir(repo)
        .output()
        .expect("record finalization");
    assert!(
        output.status.success(),
        "record finalization stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
