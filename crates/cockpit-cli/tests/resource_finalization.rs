use sha2::{Digest as ShaDigest, Sha256};
use std::{fs, path::Path, process::Command};

fn run(binary: &str, args: &[&str], repo: &Path) -> std::process::Output {
    Command::new(binary)
        .args(args)
        .args(["--repo"])
        .arg(repo)
        .current_dir(repo)
        .output()
        .expect("run ai-cockpit")
}

fn run_json(binary: &str, args: &[&str], repo: &Path) -> serde_json::Value {
    let output = run(binary, args, repo);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("JSON output")
}

fn runtime_digest(binary: &str) -> String {
    let bytes = fs::read(binary).expect("runtime binary");
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn repository() -> tempfile::TempDir {
    let repo = tempfile::tempdir().expect("temp repo");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(repo.path())
            .status()
            .expect("git init")
            .success()
    );
    repo
}

#[test]
fn runtime_close_requires_explicit_resource_finalization_receipt() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = repository();
    let root = repo.path();
    assert!(run(binary, &["attach"], root).status.success());
    assert!(
        run(
            binary,
            &[
                "start",
                "--id",
                "WI-FINALIZATION",
                "--intent",
                "bind resources",
                "--goal",
                "prevent stale branches",
                "--scope",
                "src/**",
                "--authority",
                "authorized",
            ],
            root,
        )
        .status
        .success()
    );
    let context_file = tempfile::NamedTempFile::new().expect("context temp file");
    let context_path = context_file.path().to_owned();
    let context_arg = context_path.to_string_lossy().into_owned();
    fs::write(
        &context_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "branch": "feature/finalization",
            "worktree": "/tmp/removed-feature-finalization",
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": "https://github.com/example/ai-cockpit/pull/159"
        }))
        .expect("context JSON"),
    )
    .expect("write context");
    let plan_output = run_json(
        binary,
        &[
            "work-item",
            "finalize-plan",
            "--id",
            "WI-FINALIZATION",
            "--input",
            context_arg.as_str(),
        ],
        root,
    );
    let contract_digest = plan_output["contractDigest"]
        .as_str()
        .expect("contract digest")
        .to_owned();
    assert_eq!(plan_output["state"], "planned");
    assert!(
        run(
            binary,
            &[
                "preflight",
                "--contract",
                ".ai/work-items/active/WI-FINALIZATION.contract.json",
            ],
            root,
        )
        .status
        .success()
    );
    let checkpoint = run(binary, &["checkpoint", "--id", "WI-FINALIZATION"], root);
    assert!(
        checkpoint.status.success(),
        "checkpoint stderr: {}",
        String::from_utf8_lossy(&checkpoint.stderr)
    );
    assert!(
        run(
            binary,
            &[
                "verify",
                "--work-item",
                "WI-FINALIZATION",
                "--command",
                "true"
            ],
            root,
        )
        .status
        .success()
    );
    assert!(
        run(binary, &["finish", "--id", "WI-FINALIZATION"], root)
            .status
            .success()
    );

    let archived = run(binary, &["archive", "--id", "WI-FINALIZATION"], root);
    assert!(
        archived.status.success(),
        "archive stderr: {}",
        String::from_utf8_lossy(&archived.stderr)
    );
    let missing_receipt = run(
        binary,
        &[
            "close",
            "--id",
            "WI-FINALIZATION",
            "--human-decision",
            "approved",
        ],
        root,
    );
    assert!(
        !missing_receipt.status.success(),
        "close must fail before resource finalization"
    );
    let status = run_json(binary, &["status"], root);
    let repository_id = status["repositoryId"].as_str().expect("repository id");
    let receipt_file = tempfile::NamedTempFile::new().expect("receipt temp file");
    let receipt_path = receipt_file.path().to_owned();
    let receipt_arg = receipt_path.to_string_lossy().into_owned();
    let receipt = serde_json::json!({
        "schemaVersion": 1,
        "receiptId": "receipt-finalization",
        "operationId": "operation-finalization",
        "repositoryId": repository_id,
        "workItemId": "WI-FINALIZATION",
        "runtimeVersion": env!("CARGO_PKG_VERSION"),
        "runtimeDigest": runtime_digest(binary),
        "provider": "github",
        "pullRequest": {
            "number": 159,
            "url": "https://github.com/example/ai-cockpit/pull/159",
            "headRevision": "abcdef1",
            "baseBranch": "main",
            "baseRemote": "origin",
            "baseRevision": "abcdef0",
            "mergeCommit": "1234567"
        },
        "branch": {
            "name": "feature/finalization",
            "remote": "origin",
            "headRevision": "abcdef1"
        },
        "worktree": {
            "worktreeId": "removed-feature-finalization",
            "path": "/tmp/removed-feature-finalization",
            "branch": "feature/finalization",
            "headRevision": "abcdef1"
        },
        "before": {
            "pullRequest": "merged",
            "branch": "present",
            "worktree": "clean"
        },
        "after": {
            "pullRequest": "merged",
            "branch": "deleted",
            "worktree": "removed"
        },
        "result": {
            "disposition": "deleted",
            "failureCodes": [],
            "unknownCodes": []
        },
        "actor": "human:test",
        "authoritySource": "test-policy",
        "reason": "provider cleanup receipt",
        "timestamp": "2026-08-23T00:00:00Z",
        "contractDigest": contract_digest,
        "resourceContext": {
            "branch": "feature/finalization",
            "worktree": "/tmp/removed-feature-finalization",
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": "https://github.com/example/ai-cockpit/pull/159"
        }
    });
    fs::write(
        &receipt_path,
        serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
    )
    .expect("write receipt");
    let finalized = run_json(
        binary,
        &[
            "work-item",
            "finalize",
            "--id",
            "WI-FINALIZATION",
            "--input",
            receipt_arg.as_str(),
        ],
        root,
    );
    assert_eq!(finalized["state"], "recorded");
    assert_eq!(finalized["disposition"], "deleted");
    let verified = run_json(
        binary,
        &["work-item", "finalize-verify", "--id", "WI-FINALIZATION"],
        root,
    );
    assert_eq!(verified["state"], "verified");
    assert_eq!(verified["disposition"], "deleted");

    let closed = run(
        binary,
        &[
            "close",
            "--id",
            "WI-FINALIZATION",
            "--human-decision",
            "approved",
            "--actor",
            "human:test",
            "--authority-source",
            "test-policy",
            "--reason",
            "resource receipt reviewed",
            "--decided-at",
            "2026-08-23T00:00:00Z",
        ],
        root,
    );
    assert!(
        closed.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&closed.stderr)
    );
}
