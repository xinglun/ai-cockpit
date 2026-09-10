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

fn assert_success(output: &std::process::Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("git output")
        .trim()
        .to_owned()
}

fn direct_merge_repository_for_cli() -> (tempfile::TempDir, String) {
    let repo = repository();
    let root = repo.path();
    git(root, &["config", "user.email", "tests@example.invalid"]);
    git(root, &["config", "user.name", "AI Cockpit Tests"]);
    git(root, &["branch", "-M", "main"]);
    fs::write(root.join("README.md"), "direct merge CLI fixture\n").expect("write base");
    git(root, &["add", "README.md"]);
    git(root, &["commit", "-q", "-m", "base"]);
    let base = git(root, &["rev-parse", "HEAD"]);
    git(root, &["checkout", "-q", "-b", "feature/direct-merge-cli"]);
    fs::write(root.join("direct-merge.txt"), "historical merge\n").expect("write feature");
    git(root, &["add", "direct-merge.txt"]);
    git(root, &["commit", "-q", "-m", "direct merge feature"]);
    let feature_head = git(root, &["rev-parse", "HEAD"]);
    git(root, &["checkout", "-q", "main"]);
    git(
        root,
        &[
            "merge",
            "--no-ff",
            "-q",
            "feature/direct-merge-cli",
            "-m",
            "direct merge",
        ],
    );
    let merge_commit = git(root, &["rev-parse", "HEAD"]);
    git(root, &["reset", "--hard", "-q", &base]);
    git(root, &["branch", "-D", "feature/direct-merge-cli"]);
    assert!(!feature_head.is_empty());
    (repo, merge_commit)
}

#[test]
fn historical_recovery_commands_are_discoverable_in_work_item_help() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let output = Command::new(binary)
        .args(["work-item", "--help"])
        .output()
        .expect("run work-item help");
    assert_success(&output, "work-item help");
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("finalize-recovery"), "help: {help}");
    assert!(help.contains("finalize-recovery-plan"), "help: {help}");
}

#[test]
fn finalize_recovery_help_describes_first_direct_merge_apply() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let output = Command::new(binary)
        .args(["work-item", "finalize-recovery", "--help"])
        .output()
        .expect("run finalize-recovery help");
    assert_success(&output, "finalize-recovery help");
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("first direct-merge"), "help: {help}");
    assert!(
        help.contains("complete direct-merge receipt"),
        "help: {help}"
    );
}

#[test]
fn cli_direct_merge_plan_round_trips_through_recovery_and_finalize() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let (repo, merge_commit) = direct_merge_repository_for_cli();
    let root = repo.path();
    let id = "WI-DIRECT-MERGE-CLI";
    assert_success(&run(binary, &["attach"], root), "attach");

    let context_file = tempfile::NamedTempFile::new().expect("context file");
    fs::write(
        context_file.path(),
        serde_json::to_vec_pretty(&serde_json::json!({
            "branch": "feature/direct-merge-cli",
            "worktree": root.to_string_lossy(),
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "local",
            "pullRequest": "https://github.com/example/project/pull/999"
        }))
        .expect("context JSON"),
    )
    .expect("write context");
    let context_arg = context_file.path().to_string_lossy().into_owned();
    assert_success(
        &run(
            binary,
            &[
                "start",
                "--id",
                id,
                "--intent",
                "record historical direct merge",
                "--goal",
                "preserve truthful no-PR history",
                "--scope",
                "**",
                "--authority",
                "authorized",
            ],
            root,
        ),
        "start",
    );
    let plan = run_json(
        binary,
        &[
            "work-item",
            "finalize-plan",
            "--id",
            id,
            "--input",
            context_arg.as_str(),
        ],
        root,
    );
    assert_eq!(plan["state"], "planned");
    assert_success(
        &run(
            binary,
            &[
                "preflight",
                "--contract",
                &format!(".ai/work-items/active/{id}.contract.json"),
            ],
            root,
        ),
        "preflight",
    );
    assert_success(
        &run(binary, &["checkpoint", "--id", id], root),
        "checkpoint",
    );
    assert_success(
        &run(
            binary,
            &["verify", "--work-item", id, "--command", "true"],
            root,
        ),
        "verify",
    );
    assert_success(&run(binary, &["finish", "--id", id], root), "finish");
    assert_success(&run(binary, &["archive", "--id", id], root), "archive");

    let recovery_plan = run_json(
        binary,
        &[
            "work-item",
            "finalize-recovery-plan",
            "--id",
            id,
            "--merge-commit",
            merge_commit.as_str(),
        ],
        root,
    );
    assert_eq!(recovery_plan["state"], "direct_merge_candidate");
    assert_eq!(recovery_plan["historicalKind"], "direct_merge_no_pr");
    assert_eq!(
        recovery_plan["suggestedReceipt"]["pullRequest"]["number"],
        0
    );
    assert_eq!(
        recovery_plan["suggestedReceipt"]["historical"]["mergeCommit"],
        merge_commit
    );
    assert_eq!(
        recovery_plan["suggestedReceipt"]["resourceContext"]["provider"],
        "historical"
    );
    let mut receipt = recovery_plan["suggestedReceipt"].clone();
    receipt["actor"] = "human:test".into();
    receipt["authoritySource"] = "historical-test".into();
    receipt["reason"] = "record a direct merge without inventing a pull request".into();
    receipt["timestamp"] = "2026-09-07T00:00:00Z".into();
    let receipt_file = tempfile::NamedTempFile::new().expect("receipt file");
    fs::write(
        receipt_file.path(),
        serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
    )
    .expect("write receipt");
    let receipt_arg = receipt_file.path().to_string_lossy().into_owned();
    let recovered = run_json(
        binary,
        &[
            "work-item",
            "finalize-recovery",
            "--id",
            id,
            "--input",
            receipt_arg.as_str(),
        ],
        root,
    );
    assert_eq!(recovered["state"], "recorded");
    let replay = run_json(
        binary,
        &[
            "work-item",
            "finalize",
            "--id",
            id,
            "--input",
            receipt_arg.as_str(),
        ],
        root,
    );
    assert_eq!(replay["state"], "idempotent");
    let verified = run_json(binary, &["work-item", "finalize-verify", "--id", id], root);
    assert_eq!(verified["state"], "verified");
    assert_eq!(verified["historicalKind"], "direct_merge_no_pr");
    let closed = run(
        binary,
        &[
            "close",
            "--id",
            id,
            "--human-decision",
            "approved",
            "--actor",
            "human:test",
            "--authority-source",
            "historical-test",
            "--reason",
            "close verified historical direct merge",
            "--evidence-ref",
            &format!(".ai/decisions/{id}.finalize.json"),
            "--decided-at",
            "2026-09-07T00:01:00Z",
        ],
        root,
    );
    assert_success(&closed, "close");
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
    let archived_contract: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(".ai/work-items/archive/WI-FINALIZATION.contract.json"))
            .expect("archived contract"),
    )
    .expect("archived contract JSON");
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
            "baseRevision": archived_contract["baseRevision"],
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

#[test]
fn runtime_accepts_explicitly_abandoned_unmerged_cleanup() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = repository();
    let root = repo.path();
    let id = "WI-ABANDONED-FINALIZATION";
    assert_success(&run(binary, &["attach"], root), "attach");
    assert_success(
        &run(
            binary,
            &[
                "start",
                "--id",
                id,
                "--intent",
                "record failed delivery cleanup",
                "--goal",
                "preserve truthful non-merge outcome",
                "--scope",
                "src/**",
                "--authority",
                "authorized",
            ],
            root,
        ),
        "start",
    );
    let context_file = tempfile::NamedTempFile::new().expect("context temp file");
    let context_path = context_file.path().to_owned();
    fs::write(
        &context_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "branch": "feature/abandoned-finalization",
            "worktree": "/tmp/removed-abandoned-finalization",
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": "https://github.com/example/ai-cockpit/pull/160"
        }))
        .expect("context JSON"),
    )
    .expect("write context");
    let plan = run_json(
        binary,
        &[
            "work-item",
            "finalize-plan",
            "--id",
            id,
            "--input",
            context_path.to_string_lossy().as_ref(),
        ],
        root,
    );
    let contract_digest = plan["contractDigest"].as_str().unwrap().to_owned();
    assert_success(
        &run(
            binary,
            &[
                "preflight",
                "--contract",
                &format!(".ai/work-items/active/{id}.contract.json"),
            ],
            root,
        ),
        "preflight",
    );
    assert_success(
        &run(binary, &["checkpoint", "--id", id], root),
        "checkpoint",
    );
    assert_success(
        &run(
            binary,
            &["verify", "--work-item", id, "--command", "true"],
            root,
        ),
        "verify",
    );
    assert_success(&run(binary, &["finish", "--id", id], root), "finish");
    assert_success(&run(binary, &["archive", "--id", id], root), "archive");

    let repository_id = run_json(binary, &["status"], root)["repositoryId"]
        .as_str()
        .unwrap()
        .to_owned();
    let archived_contract: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(format!(".ai/work-items/archive/{id}.contract.json")))
            .expect("archived contract"),
    )
    .expect("archived contract JSON");
    let receipt_file = tempfile::NamedTempFile::new().expect("receipt temp file");
    let receipt = serde_json::json!({
        "schemaVersion": 1,
        "receiptId": "receipt-abandoned-finalization",
        "operationId": "operation-abandoned-finalization",
        "repositoryId": repository_id,
        "workItemId": id,
        "runtimeVersion": env!("CARGO_PKG_VERSION"),
        "runtimeDigest": runtime_digest(binary),
        "provider": "github",
        "pullRequest": {
            "number": 160,
            "url": "https://github.com/example/ai-cockpit/pull/160",
            "headRevision": "abcdef2",
            "baseBranch": "main",
            "baseRemote": "origin",
            "baseRevision": archived_contract["baseRevision"],
            "mergeCommit": null
        },
        "branch": {
            "name": "feature/abandoned-finalization",
            "remote": "origin",
            "headRevision": "abcdef2"
        },
        "worktree": {
            "worktreeId": "removed-abandoned-finalization",
            "path": "/tmp/removed-abandoned-finalization",
            "branch": "feature/abandoned-finalization",
            "headRevision": "abcdef2"
        },
        "before": {
            "pullRequest": "unmerged",
            "branch": "present",
            "worktree": "clean"
        },
        "after": {
            "pullRequest": "unmerged",
            "branch": "deleted",
            "worktree": "removed"
        },
        "result": {
            "disposition": "abandoned",
            "failureCodes": ["unmerged_pull_request"],
            "unknownCodes": []
        },
        "actor": "human:test",
        "authoritySource": "test-policy",
        "reason": "reviewed failed delivery was explicitly closed without merge",
        "timestamp": "2026-08-23T00:00:00Z",
        "contractDigest": contract_digest,
        "resourceContext": {
            "branch": "feature/abandoned-finalization",
            "worktree": "/tmp/removed-abandoned-finalization",
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": "https://github.com/example/ai-cockpit/pull/160"
        }
    });
    fs::write(
        receipt_file.path(),
        serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
    )
    .expect("write receipt");
    let finalized = run_json(
        binary,
        &[
            "work-item",
            "finalize",
            "--id",
            id,
            "--input",
            receipt_file.path().to_string_lossy().as_ref(),
        ],
        root,
    );
    assert_eq!(finalized["state"], "recorded");
    assert_eq!(finalized["disposition"], "abandoned");

    let verified = run_json(binary, &["work-item", "finalize-verify", "--id", id], root);
    assert_eq!(verified["state"], "verified");
    assert_eq!(verified["disposition"], "abandoned");

    let closed = run(
        binary,
        &[
            "close",
            "--id",
            id,
            "--human-decision",
            "superseded_failed_delivery",
            "--actor",
            "human:test",
            "--authority-source",
            "test-policy",
            "--reason",
            "reviewed failed delivery was cleaned without merge",
            "--decided-at",
            "2026-08-23T00:00:00Z",
        ],
        root,
    );
    assert_success(&closed, "close abandoned finalization");
}

#[test]
fn cli_appends_governance_bound_merge_observation_and_cleanup() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = repository();
    let root = repo.path();
    let id = "WI-FINALIZATION-APPEND";
    assert_success(&run(binary, &["attach"], root), "attach");
    assert_success(
        &run(
            binary,
            &[
                "start",
                "--id",
                id,
                "--intent",
                "bind governance receipt append",
                "--goal",
                "preserve finalization history",
                "--scope",
                "**",
                "--authority",
                "authorized",
            ],
            root,
        ),
        "start",
    );
    let context_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(
        context_file.path(),
        serde_json::to_vec_pretty(&serde_json::json!({
            "branch": "feature/finalization-append",
            "worktree": "/tmp/removed-finalization-append",
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": "https://github.com/example/ai-cockpit/pull/191"
        }))
        .unwrap(),
    )
    .unwrap();
    let context_arg = context_file.path().to_string_lossy().into_owned();
    let plan = run_json(
        binary,
        &[
            "work-item",
            "finalize-plan",
            "--id",
            id,
            "--input",
            &context_arg,
        ],
        root,
    );
    let contract_digest = plan["contractDigest"].as_str().unwrap().to_string();
    assert_success(
        &run(
            binary,
            &[
                "preflight",
                "--contract",
                &format!(".ai/work-items/active/{id}.contract.json"),
            ],
            root,
        ),
        "preflight",
    );
    assert_success(
        &run(binary, &["checkpoint", "--id", id], root),
        "checkpoint",
    );
    assert_success(
        &run(
            binary,
            &["verify", "--work-item", id, "--command", "true"],
            root,
        ),
        "verify",
    );
    assert_success(&run(binary, &["finish", "--id", id], root), "finish");
    assert_success(&run(binary, &["archive", "--id", id], root), "archive");

    for args in [
        ["config", "user.email", "tests@example.invalid"],
        ["config", "user.name", "AI Cockpit Tests"],
    ] {
        assert_success(
            &Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .unwrap(),
            "git config",
        );
    }
    assert_success(
        &Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap(),
        "git add archive",
    );
    assert_success(
        &Command::new("git")
            .args(["commit", "-q", "-m", "archive"])
            .current_dir(root)
            .output()
            .unwrap(),
        "git commit archive",
    );
    let archive_head = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    let repository_id = run_json(binary, &["status"], root)["repositoryId"]
        .as_str()
        .unwrap()
        .to_string();
    let archived_contract: serde_json::Value = serde_json::from_slice(
        &fs::read(
            root.join(".ai/work-items/archive")
                .join(format!("{id}.contract.json")),
        )
        .expect("archived contract"),
    )
    .expect("archived contract JSON");
    let mut blocked = serde_json::json!({
        "schemaVersion":1,"receiptId":"blocked-1","operationId":"operation-1",
        "repositoryId":repository_id,"workItemId":id,
        "runtimeVersion":env!("CARGO_PKG_VERSION"),"runtimeDigest":runtime_digest(binary),
        "provider":"github",
        "pullRequest":{"number":191,"url":"https://github.com/example/ai-cockpit/pull/191","headRevision":archive_head,"baseBranch":"main","baseRemote":"origin","baseRevision":archived_contract["baseRevision"]},
        "branch":{"name":"feature/finalization-append","remote":"origin","headRevision":archive_head},
        "worktree":{"worktreeId":"removed-finalization-append","path":"/tmp/removed-finalization-append","branch":"feature/finalization-append","headRevision":archive_head},
        "before":{"pullRequest":"unmerged","branch":"present","worktree":"clean"},
        "after":{"pullRequest":"unmerged","branch":"present","worktree":"clean"},
        "result":{"disposition":"blocked","failureCodes":["unmerged_pull_request"],"unknownCodes":[]},
        "actor":"human:test","authoritySource":"test-policy","reason":"await merge","timestamp":"2026-08-23T00:00:00Z",
        "contractDigest":contract_digest,
        "resourceContext":{"branch":"feature/finalization-append","worktree":"/tmp/removed-finalization-append","baseBranch":"main","baseRemote":"origin","provider":"github","pullRequest":"https://github.com/example/ai-cockpit/pull/191"}
    });
    let blocked_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(
        blocked_file.path(),
        serde_json::to_vec_pretty(&blocked).unwrap(),
    )
    .unwrap();
    let blocked_arg = blocked_file.path().to_string_lossy().into_owned();
    assert_eq!(
        run_json(
            binary,
            &["work-item", "finalize", "--id", id, "--input", &blocked_arg],
            root,
        )["state"],
        "recorded"
    );
    let canonical = format!(".ai/decisions/{id}.finalize.json");
    assert_success(
        &Command::new("git")
            .args(["add", &canonical])
            .current_dir(root)
            .output()
            .unwrap(),
        "git add receipt",
    );
    assert_success(
        &Command::new("git")
            .args(["commit", "-q", "-m", "append receipt"])
            .current_dir(root)
            .output()
            .unwrap(),
        "git commit receipt",
    );
    let append_head = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    let predecessor = cockpit_protocol::digest_json(&blocked).unwrap();
    blocked["receiptId"] = "observed-2".into();
    blocked["operationId"] = "operation-2".into();
    blocked["pullRequest"]["headRevision"] = append_head.clone().into();
    blocked["pullRequest"]["mergeCommit"] = "merge-191".into();
    blocked["branch"]["headRevision"] = append_head.clone().into();
    blocked["worktree"]["headRevision"] = append_head.clone().into();
    blocked["before"] = blocked["after"].clone();
    blocked["after"]["pullRequest"] = "merged".into();
    blocked["result"] =
        serde_json::json!({"disposition":"retained","failureCodes":[],"unknownCodes":[]});
    let observed = serde_json::json!({
        "schemaVersion":1,"transitionId":"transition-1","sequence":1,
        "predecessorReceiptDigest":predecessor,
        "governanceAppendRevision":append_head,
        "receipt":blocked
    });
    let observed_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(
        observed_file.path(),
        serde_json::to_vec_pretty(&observed).unwrap(),
    )
    .unwrap();
    let observed_arg = observed_file.path().to_string_lossy().into_owned();
    let appended = run_json(
        binary,
        &[
            "work-item",
            "finalize",
            "--id",
            id,
            "--input",
            &observed_arg,
        ],
        root,
    );
    assert_eq!(appended["state"], "appended");
    assert_eq!(appended["sequence"], 1);
    let verified = run_json(binary, &["work-item", "finalize-verify", "--id", id], root);
    assert_eq!(verified["sequence"], 1);
    assert_eq!(verified["disposition"], "retained");
}
