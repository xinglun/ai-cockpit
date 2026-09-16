use cockpit_core::Digest;
use cockpit_git::{ChangeContentState, ChangeEvidence, ChangeKind, RepositorySnapshot};
use cockpit_protocol::{HumanDecision, ResourceFinalizationContext, RuntimeContext};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    amend_work_item_contract, archive_work_item, archive_work_item_with_runtime, attach,
    checkpoint_work_item, close_work_item_with_structured_decision,
    close_work_item_with_structured_decision_and_runtime, finish_work_item,
    finish_work_item_with_runtime, outcome_render_input_with_runtime, plan_resource_finalization,
    preflight_work_item, record_recovery_decision, record_resource_finalization,
    record_verification, record_verification_with_runtime, render_human_outcome, repository_id,
    run_repository_verification, start_work_item, start_work_item_with_options,
    work_item_status_index_with_runtime, work_item_status_snapshot_with_runtime,
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, process::Command};

fn governance_snapshot(path: &str, added_lines: &[&str]) -> RepositorySnapshot {
    RepositorySnapshot {
        root: "/tmp/repo".into(),
        git_root: "/tmp/repo".into(),
        head: Some("0123456789abcdef0123456789abcdef01234567".into()),
        changed_paths: vec![path.into()],
        change_evidence: vec![ChangeEvidence {
            path: path.into(),
            kind: ChangeKind::Modified,
            added_lines: added_lines.iter().map(|line| (*line).into()).collect(),
            removed_lines: Vec::new(),
            after_text: Some(added_lines.join("\n")),
            content_state: ChangeContentState::Text,
        }],
        git_calls: 0,
        tree_digest: "sha256:tree".into(),
        diff_digest: "sha256:diff".into(),
        dependency_fingerprint: "sha256:dependencies".into(),
        files_read: 1,
        files_hashed: 1,
        bytes_read: 0,
        bytes_hashed: 0,
        source_tree_digest: None,
    }
}

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(directory.path()).expect("attach");
    directory
}

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "0.1.0".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"status-runtime"),
    }
}

fn git(directory: &std::path::Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(directory)
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

fn commit_all(directory: &std::path::Path, message: &str) {
    git(directory, &["add", "-A"]);
    git(
        directory,
        &[
            "-c",
            "user.name=Status Test",
            "-c",
            "user.email=status@example.invalid",
            "commit",
            "-q",
            "-m",
            message,
        ],
    );
}

fn prepare_ordinary_archive(work_item_id: &str, commit_archive: bool) -> tempfile::TempDir {
    let directory = repository();
    git(
        directory.path(),
        &["branch", "-m", "feature/ordinary-close"],
    );
    commit_all(directory.path(), "attach repository");
    complete_ordinary_archive(directory.path(), work_item_id, commit_archive);
    directory
}

fn complete_ordinary_archive(
    directory: &std::path::Path,
    work_item_id: &str,
    commit_archive: bool,
) {
    fs::create_dir_all(directory.join(".ai/evidence")).expect("evidence directory");
    fs::create_dir_all(directory.join(".ai/decisions")).expect("decisions directory");
    let current_runtime = runtime();
    start_work_item_with_options(
        directory,
        work_item_id,
        "ordinary close cleanup binding",
        "bind cleanup without external resources",
        &[".ai/**".into(), "src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["ordinary cleanup is independently projected".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start ordinary work item");
    let contract_path = directory.join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    preflight_work_item(directory, &contract_path).expect("preflight ordinary work item");
    checkpoint_work_item(directory, work_item_id).expect("checkpoint ordinary work item");
    let run = run_repository_verification(
        directory,
        &RepositoryVerificationRequest {
            node_id: "ordinary-close-check".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec![".ai/**".into(), "src/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current_runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("verify ordinary work item");
    record_verification_with_runtime(
        directory,
        work_item_id,
        &serde_json::to_value(&run.receipt).expect("verification receipt"),
        &current_runtime,
        &run.final_snapshot,
    )
    .expect("record ordinary verification");
    finish_work_item_with_runtime(directory, work_item_id, &current_runtime)
        .expect("finish ordinary work item");
    archive_work_item_with_runtime(directory, work_item_id, &current_runtime)
        .expect("archive ordinary work item");
    if commit_archive {
        commit_all(directory, "archive ordinary work item");
    }
}

fn approved_decision(work_item_id: &str) -> HumanDecision {
    HumanDecision {
        decision: "approved".into(),
        actor: "human:owner".into(),
        authority_source: "user-authorized-work-item".into(),
        reason: "focused status projection evidence is complete".into(),
        evidence_refs: vec![format!(".ai/evidence/{work_item_id}.verification.json")],
        policy_refs: vec!["status-projection".into()],
        decided_at: "2026-09-15T00:00:00Z".into(),
        resume_condition: Some("rerun verification if repository identity changes".into()),
    }
}

#[test]
fn ordinary_close_rejects_unprovable_worktree_binding_without_writing_receipts() {
    let work_item_id = "WI-STATUS-ORDINARY-UNBOUND";
    let directory = prepare_ordinary_archive(work_item_id, false);

    let error = close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        work_item_id,
        &approved_decision(work_item_id),
        &runtime(),
    )
    .expect_err("uncommitted archive cannot be bound to the exact current HEAD");

    assert!(error.to_string().contains("exact archived"));
    let decisions = directory.path().join(".ai/decisions");
    assert!(
        !decisions
            .join(format!("{work_item_id}.close.json"))
            .exists()
    );
    assert!(
        fs::read_dir(decisions)
            .expect("decisions")
            .all(|entry| !entry
                .expect("decision entry")
                .file_name()
                .to_string_lossy()
                .contains(&format!("{work_item_id}.cleanup."))),
        "a rejected close must not leave any cleanup receipt"
    );
}

#[test]
fn ordinary_close_keeps_verified_work_result_while_cleanup_is_pending() {
    let work_item_id = "WI-STATUS-ORDINARY-PENDING";
    let directory = prepare_ordinary_archive(work_item_id, true);

    close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        work_item_id,
        &approved_decision(work_item_id),
        &runtime(),
    )
    .expect("close ordinary work item with exact cleanup binding");

    let decision: Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/decisions/{work_item_id}.close.json")),
        )
        .expect("close decision"),
    )
    .expect("close decision JSON");
    let binding = decision
        .get("ordinaryCleanupBinding")
        .expect("ordinary cleanup binding");
    assert_eq!(
        binding["repositoryId"],
        repository_id(directory.path()).to_string()
    );
    assert_eq!(binding["workItemId"], work_item_id);
    assert_eq!(binding["branchRef"], "refs/heads/feature/ordinary-close");
    assert_eq!(
        binding["headRevision"],
        git(directory.path(), &["rev-parse", "HEAD"])
    );
    assert!(
        binding["worktreeId"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
    );
    assert!(
        decision["ordinaryCleanupBindingDigest"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
    );

    let status = work_item_status_snapshot_with_runtime(directory.path(), work_item_id, &runtime())
        .expect("closed ordinary status");
    assert_eq!(status.lifecycle_phase, "closed");
    assert_eq!(status.verification, "verified");
    assert_eq!(status.completion_domains["workResult"], "verified");
    assert_eq!(status.completion_domains["resourceCleanup"], "pending");
    assert_eq!(status.completion_domains["closure"], "closed");
    assert!(!status.blocking);
}

#[test]
fn ordinary_cleanup_receipts_promote_only_cleanup_after_exact_resources_are_removed() {
    let work_item_id = "WI-STATUS-ORDINARY-CLEANUP";
    let primary = repository();
    git(primary.path(), &["branch", "-m", "main"]);
    commit_all(primary.path(), "attach repository");
    let linked_parent = tempfile::tempdir().expect("linked worktree parent");
    let remote_path = linked_parent.path().join("origin.git");
    let remote_path_text = remote_path.display().to_string();
    git(
        linked_parent.path(),
        &["init", "--bare", "-q", &remote_path_text],
    );
    git(&remote_path, &["symbolic-ref", "HEAD", "refs/heads/main"]);
    git(
        primary.path(),
        &["remote", "add", "origin", &remote_path_text],
    );
    git(primary.path(), &["push", "-q", "-u", "origin", "main"]);
    git(
        primary.path(),
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    let linked_path = linked_parent.path().join("ordinary-worktree");
    let linked_path_text = linked_path.display().to_string();
    git(
        primary.path(),
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feature/ordinary-cleanup",
            &linked_path_text,
        ],
    );
    complete_ordinary_archive(&linked_path, work_item_id, true);
    close_work_item_with_structured_decision_and_runtime(
        &linked_path,
        work_item_id,
        &approved_decision(work_item_id),
        &runtime(),
    )
    .expect("close linked ordinary work item");

    let failed = cockpit_repository::record_ordinary_cleanup_with_runtime(
        &linked_path,
        work_item_id,
        &runtime(),
    )
    .expect("record pending exact resources");
    let failed = serde_json::to_value(failed).expect("failed cleanup receipt JSON");
    assert_eq!(failed["result"]["state"], "failed");
    let failed_status =
        work_item_status_snapshot_with_runtime(&linked_path, work_item_id, &runtime())
            .expect("failed cleanup status");
    assert_eq!(failed_status.lifecycle_phase, "closed");
    assert_eq!(failed_status.completion_domains["workResult"], "verified");
    assert_eq!(
        failed_status.completion_domains["resourceCleanup"],
        "failed"
    );
    assert_eq!(failed_status.completion_domains["closure"], "closed");

    commit_all(&linked_path, "record close and failed cleanup observation");
    git(
        primary.path(),
        &["merge", "--ff-only", "feature/ordinary-cleanup"],
    );
    git(primary.path(), &["worktree", "remove", &linked_path_text]);
    git(
        primary.path(),
        &["branch", "-d", "feature/ordinary-cleanup"],
    );

    let verified = cockpit_repository::record_ordinary_cleanup_with_runtime(
        primary.path(),
        work_item_id,
        &runtime(),
    )
    .expect("record exact cleanup postconditions");
    let verified = serde_json::to_value(verified).expect("verified cleanup receipt JSON");
    assert_eq!(verified["result"]["state"], "verified");
    assert_eq!(verified["sequence"], 2);
    let status = work_item_status_snapshot_with_runtime(primary.path(), work_item_id, &runtime())
        .expect("verified cleanup status");
    assert_eq!(status.lifecycle_phase, "closed");
    assert_eq!(status.verification, "verified");
    assert_eq!(status.completion_domains["workResult"], "verified");
    assert_eq!(status.completion_domains["resourceCleanup"], "verified");
    assert_eq!(status.completion_domains["closure"], "closed");
}

#[test]
fn ordinary_cleanup_keeps_unobservable_worktree_unknown_instead_of_verified_removed() {
    let work_item_id = "WI-STATUS-ORDINARY-UNOBSERVABLE";
    let primary = repository();
    git(primary.path(), &["branch", "-m", "main"]);
    commit_all(primary.path(), "attach repository");
    let linked_parent = tempfile::tempdir().expect("linked worktree parent");
    let remote_path = linked_parent.path().join("origin.git");
    let remote_path_text = remote_path.display().to_string();
    git(
        linked_parent.path(),
        &["init", "--bare", "-q", &remote_path_text],
    );
    git(&remote_path, &["symbolic-ref", "HEAD", "refs/heads/main"]);
    git(
        primary.path(),
        &["remote", "add", "origin", &remote_path_text],
    );
    git(primary.path(), &["push", "-q", "-u", "origin", "main"]);
    git(
        primary.path(),
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    let obstructed_parent = linked_parent.path().join("obstructed-parent");
    fs::create_dir(&obstructed_parent).expect("create linked worktree parent");
    let linked_path = obstructed_parent.join("ordinary-worktree");
    let linked_path_text = linked_path.display().to_string();
    git(
        primary.path(),
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feature/ordinary-unobservable",
            &linked_path_text,
        ],
    );
    complete_ordinary_archive(&linked_path, work_item_id, true);
    close_work_item_with_structured_decision_and_runtime(
        &linked_path,
        work_item_id,
        &approved_decision(work_item_id),
        &runtime(),
    )
    .expect("close linked ordinary work item");
    commit_all(&linked_path, "record ordinary close decision");
    git(
        primary.path(),
        &["merge", "--ff-only", "feature/ordinary-unobservable"],
    );

    fs::remove_dir_all(&linked_path).expect("remove linked worktree files");
    fs::remove_dir(&obstructed_parent).expect("remove linked worktree parent");
    fs::write(&obstructed_parent, "blocks the recorded worktree path")
        .expect("obstruct recorded worktree path");
    git(
        primary.path(),
        &[
            "update-ref",
            "-d",
            "refs/heads/feature/ordinary-unobservable",
        ],
    );

    let common_git_dir = PathBuf::from(git(
        primary.path(),
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    ));
    let obstructed_git_dir_parent = common_git_dir.join("worktrees/obstructed-parent");
    fs::write(&obstructed_git_dir_parent, "blocks the bound Git directory")
        .expect("obstruct bound Git directory");
    let obstructed_git_dir = obstructed_git_dir_parent.join("linked-git-dir");
    let close_path = primary
        .path()
        .join(format!(".ai/decisions/{work_item_id}.close.json"));
    let mut decision: Value =
        serde_json::from_slice(&fs::read(&close_path).expect("close decision"))
            .expect("close decision JSON");
    let mut binding = decision["ordinaryCleanupBinding"].clone();
    let repository_id = binding["repositoryId"]
        .as_str()
        .expect("binding repository identity");
    let bound_worktree_path = binding["worktreePath"]
        .as_str()
        .expect("binding worktree path");
    let obstructed_git_dir = obstructed_git_dir.display().to_string();
    let worktree_id = Digest::sha256_bytes(
        format!(
            "ordinary-worktree-v1\0{repository_id}\0{bound_worktree_path}\0{obstructed_git_dir}"
        )
        .as_bytes(),
    )
    .to_string();
    binding["worktreeGitDir"] = obstructed_git_dir.into();
    binding["worktreeId"] = worktree_id.into();
    decision["ordinaryCleanupBindingDigest"] = cockpit_protocol::digest_json(&binding)
        .expect("rebind cleanup identity")
        .to_string()
        .into();
    decision["ordinaryCleanupBinding"] = binding;
    fs::write(
        &close_path,
        serde_json::to_vec_pretty(&decision).expect("updated close decision"),
    )
    .expect("bind inaccessible worktree paths in isolated fixture");

    let receipt = cockpit_repository::record_ordinary_cleanup_with_runtime(
        primary.path(),
        work_item_id,
        &runtime(),
    )
    .expect("persist a failed observation instead of claiming cleanup success");
    assert_eq!(receipt.result.state, "failed");
    assert_eq!(receipt.observation.worktree, "unknown");
    assert!(
        receipt
            .result
            .failure_codes
            .contains(&"worktree_unknown".into())
    );
    let status = work_item_status_snapshot_with_runtime(primary.path(), work_item_id, &runtime())
        .expect("ordinary cleanup remains separate from verified work");
    assert_eq!(status.lifecycle_phase, "closed");
    assert_eq!(status.completion_domains["workResult"], "verified");
    assert_eq!(status.completion_domains["resourceCleanup"], "failed");
    assert_eq!(status.completion_domains["closure"], "closed");
}

#[test]
fn ordinary_cleanup_rejects_tampered_binding_without_writing_a_receipt() {
    let work_item_id = "WI-STATUS-ORDINARY-BAD-BINDING";
    let directory = prepare_ordinary_archive(work_item_id, true);
    close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        work_item_id,
        &approved_decision(work_item_id),
        &runtime(),
    )
    .expect("close ordinary work item");
    let close_path = directory
        .path()
        .join(format!(".ai/decisions/{work_item_id}.close.json"));
    let mut decision: Value =
        serde_json::from_slice(&fs::read(&close_path).expect("close decision"))
            .expect("close decision JSON");
    decision["ordinaryCleanupBinding"]["branchRef"] =
        "refs/heads/feature/not-the-bound-branch".into();
    fs::write(
        &close_path,
        serde_json::to_vec_pretty(&decision).expect("tampered close decision"),
    )
    .expect("tamper close binding in isolated fixture");

    let error = cockpit_repository::record_ordinary_cleanup_with_runtime(
        directory.path(),
        work_item_id,
        &runtime(),
    )
    .expect_err("tampered binding must be rejected before receipt creation");
    assert!(error.to_string().contains("binding digest mismatch"));
    assert!(
        fs::read_dir(directory.path().join(".ai/decisions"))
            .expect("decisions")
            .all(|entry| !entry
                .expect("decision entry")
                .file_name()
                .to_string_lossy()
                .contains(&format!("{work_item_id}.cleanup.")))
    );
}

fn resource_worktree_path(directory: &tempfile::TempDir, work_item_id: &str) -> String {
    directory
        .path()
        .join(format!("removed-worktree-{work_item_id}"))
        .display()
        .to_string()
}

fn plan(directory: &tempfile::TempDir, work_item_id: &str) {
    plan_resource_finalization(
        directory.path(),
        work_item_id,
        &ResourceFinalizationContext {
            branch: format!("feature/{work_item_id}"),
            worktree: resource_worktree_path(directory, work_item_id),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: format!("https://github.com/example/ai-cockpit/pull/{work_item_id}"),
        },
    )
    .expect("finalization plan");
}

fn assert_no_resource_context(directory: &tempfile::TempDir, work_item_id: &str) {
    let path = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(path).expect("contract")).expect("contract JSON");
    assert!(contract.get("resourceContext").is_none());
}

fn record_deleted_finalization(directory: &tempfile::TempDir, work_item_id: &str) {
    let contract_path = directory.path().join(format!(
        ".ai/work-items/archive/{work_item_id}.contract.json"
    ));
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("archived contract"))
            .expect("contract JSON");
    let branch = format!("feature/{work_item_id}");
    let context = ResourceFinalizationContext {
        branch: branch.clone(),
        worktree: resource_worktree_path(directory, work_item_id),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: "github".into(),
        pull_request: format!("https://github.com/example/ai-cockpit/pull/{work_item_id}"),
    };
    let receipt = serde_json::json!({
        "schemaVersion": 1,
        "receiptId": format!("{work_item_id}-finalize"),
        "operationId": format!("{work_item_id}-finalize-operation"),
        "repositoryId": repository_id(directory.path()).to_string(),
        "workItemId": work_item_id,
        "runtimeVersion": runtime().runtime_version,
        "runtimeDigest": runtime().runtime_digest.to_string(),
        "provider": "github",
        "pullRequest": {
            "number": 1,
            "url": context.pull_request,
            "headRevision": "head",
            "baseBranch": "main",
            "baseRemote": "origin",
            "baseRevision": contract["baseRevision"],
            "mergeCommit": "merge"
        },
        "branch": {"name": branch, "remote": "origin", "headRevision": "head"},
        "worktree": {"worktreeId": "removed", "path": context.worktree, "branch": context.branch, "headRevision": "head"},
        "before": {"pullRequest": "merged", "branch": "present", "worktree": "clean"},
        "after": {"pullRequest": "merged", "branch": "deleted", "worktree": "removed"},
        "result": {"disposition": "deleted", "failureCodes": [], "unknownCodes": []},
        "actor": "human:test",
        "authoritySource": "status-projection-test",
        "reason": "clean up the test resource before close",
        "timestamp": "2026-08-22T12:01:00Z",
        "contractDigest": Digest::sha256_bytes(&fs::read(&contract_path).expect("contract bytes")),
        "resourceContext": context
    });
    let input = tempfile::NamedTempFile::new().expect("finalization input");
    fs::write(
        input.path(),
        serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
    )
    .expect("receipt input");
    record_resource_finalization(directory.path(), work_item_id, input.path(), &runtime())
        .expect("record deleted finalization");
}

fn historical_recovery_fixture(
    complete_successor: bool,
) -> (tempfile::TempDir, String, String, Vec<u8>) {
    let directory = repository();
    let predecessor = "WI-STATUS-HISTORICAL-RECOVERY";
    let successor = "WI-STATUS-HISTORICAL-SUCCESSOR";
    let current_runtime = runtime();

    start_work_item_with_options(
        directory.path(),
        predecessor,
        "recover a historical close projection",
        "project a valid successor recovery without rewriting historical bytes",
        &[".ai/**".into(), "src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["the predecessor remains immutable".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start predecessor");
    let predecessor_contract_path = directory
        .path()
        .join(format!(".ai/work-items/active/{predecessor}.contract.json"));
    preflight_work_item(directory.path(), &predecessor_contract_path)
        .expect("preflight predecessor");
    checkpoint_work_item(directory.path(), predecessor).expect("checkpoint predecessor");
    let predecessor_run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "historical-predecessor-check".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec![".ai/**".into(), "src/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current_runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("verify predecessor");
    record_verification_with_runtime(
        directory.path(),
        predecessor,
        &serde_json::to_value(&predecessor_run.receipt).expect("predecessor receipt"),
        &current_runtime,
        &predecessor_run.final_snapshot,
    )
    .expect("record predecessor evidence");
    finish_work_item(directory.path(), predecessor).expect("finish predecessor");
    archive_work_item(directory.path(), predecessor).expect("archive predecessor");

    let predecessor_contract: Value = serde_json::from_slice(
        &fs::read(directory.path().join(format!(
            ".ai/work-items/archive/{predecessor}.contract.json"
        )))
        .expect("predecessor contract"),
    )
    .expect("predecessor contract JSON");
    let predecessor_summary: Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/work-items/archive/{predecessor}.summary.json")),
        )
        .expect("predecessor summary"),
    )
    .expect("predecessor summary JSON");
    let predecessor_outcome: Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/work-items/archive/{predecessor}.outcome.json")),
        )
        .expect("predecessor outcome"),
    )
    .expect("predecessor outcome JSON");
    let predecessor_events = fs::read(
        directory
            .path()
            .join(format!(".ai/work-items/archive/{predecessor}.events.jsonl")),
    )
    .expect("predecessor events");
    let recovery = json!({
        "schemaVersion": 1,
        "decisionId": "work-item-recovery",
        "decision": "successor",
        "workItemId": predecessor,
        "repositoryId": repository_id(directory.path()),
        "predecessorWorkItemId": predecessor,
        "predecessorContractDigest": cockpit_protocol::digest_json(&predecessor_contract).expect("contract digest"),
        "predecessorSummaryDigest": cockpit_protocol::digest_json(&predecessor_summary).expect("summary digest"),
        "predecessorOutcomeDigest": cockpit_protocol::digest_json(&predecessor_outcome).expect("outcome digest"),
        "predecessorEventsDigest": Digest::sha256_bytes(&predecessor_events),
        "successorWorkItemId": successor,
        "runtimeVersion": current_runtime.runtime_version,
        "runtimeDigest": current_runtime.runtime_digest,
        "actor": "human:owner",
        "authoritySource": "repository-owner",
        "reason": "recover through the independently verified successor",
        "evidenceRefs": [format!(".ai/work-items/archive/{predecessor}.outcome.json")],
        "policyRefs": ["docs/reference/repository-workflow.md"],
        "decidedAt": "2026-08-28T00:00:00Z",
        "resumeCondition": "the successor reaches its own terminal boundary"
    });
    record_recovery_decision(directory.path(), predecessor, &recovery, &current_runtime)
        .expect("record successor recovery");
    start_work_item_with_options(
        directory.path(),
        successor,
        "continue the recovered work",
        "complete an independently bound terminal successor",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["successor evidence is independently verified".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("activate successor");
    let successor_contract_path = directory
        .path()
        .join(format!(".ai/work-items/active/{successor}.contract.json"));
    let preflight = preflight_work_item(directory.path(), &successor_contract_path)
        .expect("preflight successor");
    assert_ne!(
        preflight.state,
        cockpit_core::DecisionState::Red,
        "successor preflight unexpectedly blocked: {preflight:#?}"
    );
    if !complete_successor {
        return (directory, predecessor.into(), successor.into(), Vec::new());
    }
    checkpoint_work_item(directory.path(), successor).expect("checkpoint successor");
    let successor_run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "historical-successor-check".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["src/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current_runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("verify successor");
    record_verification_with_runtime(
        directory.path(),
        successor,
        &serde_json::to_value(&successor_run.receipt).expect("successor receipt"),
        &current_runtime,
        &successor_run.final_snapshot,
    )
    .expect("record successor evidence");
    preflight_work_item(directory.path(), &successor_contract_path)
        .expect("green successor preflight");
    finish_work_item(directory.path(), successor).expect("finish successor");
    archive_work_item(directory.path(), successor).expect("archive successor");
    close_work_item_with_structured_decision(
        directory.path(),
        successor,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "repository-owner".into(),
            reason: "successor terminal evidence is complete".into(),
            evidence_refs: vec![format!(".ai/evidence/{successor}.verification.json")],
            policy_refs: vec!["docs/reference/repository-workflow.md".into()],
            decided_at: "2026-08-28T00:01:00Z".into(),
            resume_condition: None,
        },
    )
    .expect("close successor");

    let close_path = directory
        .path()
        .join(format!(".ai/decisions/{predecessor}.close.json"));
    let historical_close = br#"{"workItemId":"WI-STATUS-HISTORICAL-RECOVERY","state":"closed","decisionState":"confirmed","humanDecision":"approved","structuredDecision":{"decision":"approved"}}"#;
    fs::write(&close_path, historical_close).expect("write historical close");
    (
        directory,
        predecessor.into(),
        successor.into(),
        historical_close.to_vec(),
    )
}

#[test]
fn status_projection_is_read_only_and_contains_fact_counts() {
    let directory = repository();
    start_work_item_with_options(
        directory.path(),
        "WI-STATUS-A",
        "status projection",
        "read lifecycle facts",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("start");
    let before = fs::read_dir(directory.path().join(".ai/work-items/active"))
        .expect("before")
        .count();
    let status =
        work_item_status_snapshot_with_runtime(directory.path(), "WI-STATUS-A", &runtime())
            .expect("status");
    let after = fs::read_dir(directory.path().join(".ai/work-items/active"))
        .expect("after")
        .count();
    assert_eq!(before, after, "status must not write repository state");
    assert_eq!(status.work_item_id, "WI-STATUS-A");
    assert_eq!(status.governance_state, "yellow");
    assert_eq!(status.verification, "not_ready");
    assert!(!status.base_commit.is_empty());
    assert!(!status.blocking);
    assert!(!status.human_decision_required);
    assert_eq!(status.evidence_freshness.state, "missing");
    assert!(status.last_verification_at.is_none());
    assert!(status.updated_at.is_some());
    assert!(status.safe_actions.contains(&"run_preflight".into()));
    assert!(status.source_digests.contains_key("summary"));
    assert!(status.status_digest.as_str().starts_with("sha256:"));
    assert!(
        status
            .progress_facts
            .contains_key("acceptanceCriteriaDeclared")
    );
    assert!(
        status
            .unknowns
            .iter()
            .any(|item| item == "verification_evidence_missing")
    );
    assert!(
        status
            .governance_permissions
            .contains(&"read_status".into())
    );
}

#[test]
fn status_projection_isolated_between_repositories() {
    let left = repository();
    let right = repository();
    for (directory, id) in [(&left, "WI-LEFT"), (&right, "WI-RIGHT")] {
        start_work_item_with_options(
            directory.path(),
            id,
            "isolated status",
            "keep contexts separate",
            &[".ai/**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                ..Default::default()
            },
        )
        .expect("start");
    }
    let left_status =
        work_item_status_snapshot_with_runtime(left.path(), "WI-LEFT", &runtime()).expect("left");
    let right_status = work_item_status_snapshot_with_runtime(right.path(), "WI-RIGHT", &runtime())
        .expect("right");
    assert_ne!(left_status.repository_id, right_status.repository_id);
    assert_eq!(left_status.work_item_id, "WI-LEFT");
    assert_eq!(right_status.work_item_id, "WI-RIGHT");
}

#[test]
fn all_work_item_status_is_sorted_counted_and_digest_stable() {
    let directory = repository();
    for id in ["WI-STATUS-Z", "WI-STATUS-A"] {
        start_work_item_with_options(
            directory.path(),
            id,
            "aggregate status",
            "project every work item",
            &["src/**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                ..Default::default()
            },
        )
        .expect("start");
    }

    let first = work_item_status_index_with_runtime(directory.path(), &runtime()).expect("index");
    let second =
        work_item_status_index_with_runtime(directory.path(), &runtime()).expect("repeat index");

    assert_eq!(
        first
            .items
            .iter()
            .map(|item| item.work_item_id.as_str())
            .collect::<Vec<_>>(),
        vec!["WI-STATUS-A", "WI-STATUS-Z"]
    );
    assert_eq!(first.counts.values().sum::<u64>(), 2);
    assert_eq!(first.counts["yellow"], 2);
    assert_eq!(first.counts["unknown"], 0);
    assert_eq!(first.index_digest, second.index_digest);
    assert_eq!(first.snapshot_digest, second.snapshot_digest);
    assert!(
        first
            .diagnostics
            .contains(&"work_items_aggregated:2".into())
    );
    assert_eq!(
        first.items[0].status_digest,
        first.items[0]
            .status
            .as_ref()
            .expect("member status")
            .status_digest
    );
}

#[test]
fn all_work_item_status_keeps_malformed_member_as_visible_unknown() {
    let directory = repository();
    for id in ["WI-STATUS-BAD", "WI-STATUS-GOOD"] {
        start_work_item_with_options(
            directory.path(),
            id,
            "tamper status",
            "keep valid members visible",
            &["src/**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                ..Default::default()
            },
        )
        .expect("start");
    }
    fs::write(
        directory
            .path()
            .join(".ai/work-items/active/WI-STATUS-BAD.contract.json"),
        b"{not-json",
    )
    .expect("tamper contract");

    let index = work_item_status_index_with_runtime(directory.path(), &runtime()).expect("index");
    assert_eq!(index.items.len(), 2);
    assert_eq!(index.counts["unknown"], 1);
    let bad = &index.items[0];
    assert_eq!(bad.work_item_id, "WI-STATUS-BAD");
    assert_eq!(bad.governance_state, "unknown");
    assert!(bad.status.is_none());
    assert!(bad.unknowns.contains(&"status_projection_failed".into()));
    assert!(
        bad.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.starts_with("status_projection_failed:"))
    );
    assert_eq!(index.items[1].work_item_id, "WI-STATUS-GOOD");
    assert!(index.items[1].status.is_some());
}

#[test]
fn all_work_item_status_rejects_foreign_contract_identity() {
    let directory = repository();
    let work_item_id = "WI-STATUS-FOREIGN";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "foreign status",
        "reject foreign identity",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("start");
    let path = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("contract")).expect("contract JSON");
    contract["repositoryId"] = "sha256:foreign".into();
    fs::write(&path, serde_json::to_vec_pretty(&contract).expect("bytes"))
        .expect("tamper identity");

    let index = work_item_status_index_with_runtime(directory.path(), &runtime()).expect("index");
    assert_eq!(index.counts["unknown"], 1);
    assert_eq!(index.items[0].governance_state, "unknown");
    assert!(index.items[0].status.is_none());
    assert!(
        index.items[0]
            .diagnostics
            .iter()
            .any(|value| value.contains("repository identity mismatch"))
    );
}

#[test]
fn concurrent_all_work_item_status_is_repository_isolated() {
    let left = repository();
    let right = repository();
    for (directory, id) in [(&left, "WI-STATUS-LEFT"), (&right, "WI-STATUS-RIGHT")] {
        start_work_item_with_options(
            directory.path(),
            id,
            "parallel status",
            "keep repository identities isolated",
            &["src/**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                ..Default::default()
            },
        )
        .expect("start");
    }

    let (left_index, right_index) = std::thread::scope(|scope| {
        let left_task = scope.spawn(|| {
            work_item_status_index_with_runtime(left.path(), &runtime()).expect("left index")
        });
        let right_task = scope.spawn(|| {
            work_item_status_index_with_runtime(right.path(), &runtime()).expect("right index")
        });
        (
            left_task.join().expect("left task"),
            right_task.join().expect("right task"),
        )
    });

    assert_ne!(left_index.repository_id, right_index.repository_id);
    assert_ne!(left_index.snapshot_digest, right_index.snapshot_digest);
    assert_eq!(left_index.items[0].work_item_id, "WI-STATUS-LEFT");
    assert_eq!(right_index.items[0].work_item_id, "WI-STATUS-RIGHT");
}

#[test]
fn status_projection_distinguishes_archived_from_valid_closed_decision() {
    let directory = repository();
    let work_item_id = "WI-STATUS-CLOSED";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "status close projection",
        "show terminal close state",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    plan(&directory, work_item_id);
    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    let current_runtime = runtime();
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "status-check".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["src/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current_runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("verification run");
    let evidence = serde_json::to_value(&run.receipt).expect("verification receipt");
    record_verification_with_runtime(
        directory.path(),
        work_item_id,
        &evidence,
        &current_runtime,
        &run.final_snapshot,
    )
    .expect("verification");
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");

    let invalid = cockpit_repository::close_work_item_with_structured_decision(
        directory.path(),
        work_item_id,
        &HumanDecision {
            decision: "closed".into(),
            actor: "human:owner".into(),
            authority_source: "user-authorized-work-item".into(),
            reason: "a lifecycle state is not a decision vocabulary value".into(),
            evidence_refs: vec![],
            policy_refs: vec![],
            decided_at: "2026-08-22T12:00:00Z".into(),
            resume_condition: None,
        },
    )
    .expect_err("free-form close decision must be rejected before receipt creation");
    assert!(
        invalid
            .to_string()
            .contains("human decision must be one of")
    );
    assert!(invalid.to_string().contains("approved"));
    assert!(
        !directory
            .path()
            .join(format!(".ai/decisions/{work_item_id}.close.json"))
            .exists()
    );

    let archived =
        work_item_status_snapshot_with_runtime(directory.path(), work_item_id, &runtime())
            .expect("archived status");
    assert_eq!(archived.lifecycle_phase, "archived");
    assert_eq!(archived.completion_domains["closure"], "archived");
    assert!(archived.human_decisions.is_empty());
    assert!(
        archived.blocking,
        "archived items must not appear ready before close"
    );
    assert!(
        archived
            .blockers
            .contains(&"archived_work_item_pending_close".into())
    );
    assert!(archived.unknowns.contains(&"close_decision_pending".into()));
    assert!(archived.safe_actions.contains(&"finalize_resources".into()));
    assert!(
        archived
            .safe_actions
            .contains(&"close_after_cleanup".into())
    );
    let outcome = outcome_render_input_with_runtime(directory.path(), work_item_id, &runtime())
        .expect("archived outcome");
    assert_eq!(
        outcome.outcome.decision_state,
        Some(cockpit_core::DecisionState::Yellow)
    );
    assert!(
        outcome
            .outcome
            .unknowns
            .contains(&"resource_finalization_pending".into())
    );
    let handoff = render_human_outcome(&outcome, "zh");
    assert!(handoff.starts_with("Outcome: 🟡"));
    assert!(handoff.contains("provider finalization"));
    assert!(handoff.contains("work-item finalize"));
    assert!(handoff.contains("close"));

    record_deleted_finalization(&directory, work_item_id);
    close_work_item_with_structured_decision(
        directory.path(),
        work_item_id,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "user-authorized-work-item".into(),
            reason: "status projection has fresh evidence".into(),
            evidence_refs: vec![format!(".ai/evidence/{work_item_id}.verification.json")],
            policy_refs: vec!["status-projection".into()],
            decided_at: "2026-08-22T12:00:00Z".into(),
            resume_condition: Some("rerun verification if the base changes".into()),
        },
    )
    .expect("close");
    let closed = work_item_status_snapshot_with_runtime(directory.path(), work_item_id, &runtime())
        .expect("closed status");
    assert_eq!(closed.lifecycle_phase, "closed");
    assert_eq!(closed.completion_domains["closure"], "closed");
    assert_eq!(closed.completion_domains["workResult"], "verified");
    assert_eq!(closed.completion_domains["resourceCleanup"], "verified");
    assert_eq!(closed.human_decisions, vec!["close_decision_recorded"]);
    assert!(!closed.blocking);
    assert!(
        !closed
            .safe_actions
            .iter()
            .any(|action| action == "close_after_cleanup")
    );

    let decision_path = directory
        .path()
        .join(format!(".ai/decisions/{work_item_id}.close.json"));
    let decision_bytes = fs::read(&decision_path).expect("decision bytes");
    let decision: Value = serde_json::from_slice(&decision_bytes).expect("decision JSON");
    assert!(decision.get("ordinaryCleanupBinding").is_none());
    let archived_summary = fs::read(directory.path().join(format!(
        ".ai/work-items/archive/{work_item_id}.summary.json"
    )))
    .expect("archived summary");
    assert!(!decision_bytes.is_empty());
    assert!(
        archived_summary
            .windows(b"finish_ready".len())
            .any(|window| window == b"finish_ready")
    );
}

#[test]
fn test_weakening_scanner_distinguishes_diagnostic_text_from_bypass_calls() {
    let diagnostic = cockpit_repository::derive_governance_signals(&governance_snapshot(
        "tests/docs/documentation_acceptance.sh",
        &["raise SystemExit('invalid promotion receipt')"],
    ));
    assert!(
        !diagnostic.test_weakening,
        "diagnostic exception text must not be treated as a test bypass"
    );

    let bypass = cockpit_repository::derive_governance_signals(&governance_snapshot(
        "tests/docs/example.test.js",
        &["xit('still skipped')"],
    ));
    assert!(
        bypass.test_weakening,
        "a standalone JavaScript xit call must remain a hard finding"
    );
}

#[test]
fn terminal_successor_recovery_resolves_preserved_historical_close() {
    let (directory, predecessor, _successor, historical_close) = historical_recovery_fixture(true);
    let status = work_item_status_snapshot_with_runtime(directory.path(), &predecessor, &runtime())
        .expect("recovered historical status");

    assert_eq!(status.lifecycle_phase, "recovered");
    assert_eq!(status.completion_domains["closure"], "recovered");
    assert!(
        !status.blocking,
        "a completed successor resolves the predecessor close obligation"
    );
    assert!(
        !status
            .blockers
            .contains(&"archived_work_item_pending_close".into())
    );
    assert!(
        status
            .unknowns
            .contains(&"historical_close_decision_preserved".into())
    );
    assert!(!status.unknowns.contains(&"close_decision_invalid".into()));
    assert!(
        status
            .diagnostics
            .contains(&"historical_close_decision_preserved".into())
    );
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(format!(".ai/decisions/{predecessor}.close.json"))
        )
        .expect("historical close bytes"),
        historical_close
    );
}

#[test]
fn incomplete_successor_does_not_resolve_preserved_historical_close() {
    let (directory, predecessor, successor, _historical_close) = historical_recovery_fixture(true);
    fs::remove_file(
        directory
            .path()
            .join(format!(".ai/decisions/{successor}.close.json")),
    )
    .expect("remove successor close in isolated fixture");

    let status = work_item_status_snapshot_with_runtime(directory.path(), &predecessor, &runtime())
        .expect("blocked historical status");
    assert_eq!(status.lifecycle_phase, "archived");
    assert_eq!(status.completion_domains["closure"], "archived");
    assert!(status.blocking);
    assert!(
        status
            .blockers
            .contains(&"archived_work_item_pending_close".into())
    );
    assert!(status.unknowns.contains(&"close_decision_invalid".into()));
}

#[test]
fn tampered_successor_does_not_resolve_preserved_historical_close() {
    let (directory, predecessor, successor, _historical_close) = historical_recovery_fixture(true);
    let successor_contract_path = directory
        .path()
        .join(format!(".ai/work-items/archive/{successor}.contract.json"));
    let mut successor_contract: Value =
        serde_json::from_slice(&fs::read(&successor_contract_path).expect("successor contract"))
            .expect("successor contract JSON");
    successor_contract["tamperedAfterClose"] = json!(true);
    fs::write(
        &successor_contract_path,
        serde_json::to_vec_pretty(&successor_contract).expect("tampered contract JSON"),
    )
    .expect("tamper successor contract in isolated fixture");

    let status = work_item_status_snapshot_with_runtime(directory.path(), &predecessor, &runtime())
        .expect("blocked historical status");
    assert_eq!(status.lifecycle_phase, "archived");
    assert!(status.blocking);
    assert!(
        status
            .blockers
            .contains(&"archived_work_item_pending_close".into())
    );
    assert!(status.unknowns.contains(&"close_decision_invalid".into()));
}

#[test]
fn only_the_explicitly_named_recovery_successor_may_overlap_archived_scope() {
    let (directory, predecessor, successor, _) = historical_recovery_fixture(false);
    let successor_contract = directory
        .path()
        .join(format!(".ai/work-items/active/{successor}.contract.json"));
    let decision = preflight_work_item(directory.path(), &successor_contract)
        .expect("the explicitly bound successor preflight");
    assert_ne!(
        decision.state,
        cockpit_core::DecisionState::Red,
        "{decision:#?}"
    );
    assert!(
        decision.blockers.iter().all(|blocker| {
            blocker != &format!("archived_work_item_scope_conflict:{predecessor}")
        })
    );

    let unrelated = "WI-STATUS-UNAUTHORIZED-SUCCESSOR";
    start_work_item_with_options(
        directory.path(),
        unrelated,
        "reject a recovery scope exception for another Work Item",
        "only the named successor may overlap the archived predecessor scope",
        &["tests/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["the successor exception is identity-bound".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("a disjoint unrelated Work Item may start");
    let unrelated_contract = directory
        .path()
        .join(format!(".ai/work-items/active/{unrelated}.contract.json"));
    preflight_work_item(directory.path(), &unrelated_contract).expect("initial disjoint preflight");
    checkpoint_work_item(directory.path(), unrelated).expect("checkpoint unrelated Work Item");
    amend_work_item_contract(
        directory.path(),
        unrelated,
        &json!({"scopeAppend": ["src/**"]}),
        "include the newly discovered overlapping source scope",
    )
    .expect("append overlapping scope");

    let decision = preflight_work_item(directory.path(), &unrelated_contract)
        .expect("wrong-target overlap must become a blocking decision");
    assert_eq!(
        decision.state,
        cockpit_core::DecisionState::Red,
        "{decision:#?}"
    );
    assert!(
        decision.blockers.iter().any(|blocker| {
            blocker == &format!("archived_work_item_scope_conflict:{predecessor}")
        })
    );
}

#[test]
fn malformed_recovery_receipt_cannot_authorize_archived_scope_overlap() {
    let (directory, predecessor, successor, _) = historical_recovery_fixture(false);
    let recovery_path = directory
        .path()
        .join(format!(".ai/decisions/{predecessor}.recovery.json"));
    fs::write(&recovery_path, b"{not-json").expect("corrupt recovery receipt");

    let successor_contract = directory
        .path()
        .join(format!(".ai/work-items/active/{successor}.contract.json"));
    let error = preflight_work_item(directory.path(), &successor_contract)
        .expect_err("malformed recovery must not authorize the scope exception");
    assert!(
        error.to_string().contains("candidate_json_invalid"),
        "unexpected fail-closed recovery error: {error}"
    );
}

#[test]
fn invalid_close_decision_never_promotes_archived_status() {
    let directory = repository();
    let work_item_id = "WI-STATUS-INVALID-CLOSE";
    start_work_item(
        directory.path(),
        work_item_id,
        "status invalid close",
        "reject invalid close projection",
        &["src/**".into()],
    )
    .expect("start");
    assert_no_resource_context(&directory, work_item_id);
    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    record_verification(
        directory.path(),
        work_item_id,
        &serde_json::json!({"passed": true}),
        "0.1.0",
        &Digest::sha256_bytes(b"status-runtime"),
    )
    .expect("verification");
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");
    let path = directory
        .path()
        .join(format!(".ai/decisions/{work_item_id}.close.json"));
    fs::write(
        &path,
        serde_json::json!({
            "workItemId": work_item_id,
            "state": "closed",
            "decisionState": "confirmed",
            "humanDecision": "approved",
            "structuredDecision": {"decision": "approved"}
        })
        .to_string(),
    )
    .expect("invalid decision");
    let status = work_item_status_snapshot_with_runtime(directory.path(), work_item_id, &runtime())
        .expect("status");
    assert_eq!(status.lifecycle_phase, "archived");
    assert_eq!(status.completion_domains["closure"], "archived");
    assert!(status.human_decisions.is_empty());
    assert!(status.unknowns.contains(&"close_decision_invalid".into()));
}

#[test]
fn foreign_close_repository_identity_never_promotes_archived_status() {
    let directory = repository();
    let work_item_id = "WI-STATUS-FOREIGN-CLOSE";
    start_work_item(
        directory.path(),
        work_item_id,
        "status foreign close",
        "reject cross-repository close receipt",
        &["src/**".into()],
    )
    .expect("start");
    assert_no_resource_context(&directory, work_item_id);
    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    record_verification(
        directory.path(),
        work_item_id,
        &serde_json::json!({"passed": true}),
        "0.1.0",
        &Digest::sha256_bytes(b"status-runtime"),
    )
    .expect("verification");
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");
    close_work_item_with_structured_decision(
        directory.path(),
        work_item_id,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "status-projection".into(),
            reason: "valid close before tamper".into(),
            evidence_refs: vec![format!(".ai/evidence/{work_item_id}.verification.json")],
            policy_refs: vec!["status-projection".into()],
            decided_at: "2026-08-22T12:00:00Z".into(),
            resume_condition: Some("rerun verification".into()),
        },
    )
    .expect("close");
    let path = directory
        .path()
        .join(format!(".ai/decisions/{work_item_id}.close.json"));
    let mut decision: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("decision")).expect("decision JSON");
    decision["repositoryId"] = "sha256:foreign".into();
    fs::write(
        &path,
        serde_json::to_vec_pretty(&decision).expect("decision bytes"),
    )
    .expect("tamper decision");
    let status = work_item_status_snapshot_with_runtime(directory.path(), work_item_id, &runtime())
        .expect("status");
    assert_eq!(status.lifecycle_phase, "archived");
    assert_eq!(status.completion_domains["closure"], "archived");
    assert!(status.unknowns.contains(&"close_decision_invalid".into()));
}

#[test]
fn legacy_closed_archive_does_not_block_new_work_item_entry() {
    let directory = repository();
    let work_item_id = "WI-LEGACY-CLOSED";
    let repository_id = repository_id(directory.path()).to_string();
    let archive = directory.path().join(".ai/work-items/archive");
    let decisions = directory.path().join(".ai/decisions");
    fs::write(
        archive.join(format!("{work_item_id}.contract.json")),
        serde_json::json!({
            "workItemId": work_item_id,
            "repositoryId": repository_id,
        })
        .to_string(),
    )
    .expect("legacy contract");
    fs::write(
        archive.join(format!("{work_item_id}.archive.json")),
        serde_json::json!({
            "workItemId": work_item_id,
            "state": "archived",
        })
        .to_string(),
    )
    .expect("legacy archive");
    fs::write(
        decisions.join(format!("{work_item_id}.close.json")),
        serde_json::json!({
            "decisionState": "confirmed",
            "humanDecision": "approved",
            "state": "closed",
            "structuredDecision": {
                "actor": "human:owner",
                "authoritySource": "historical-policy",
                "decidedAt": "2026-01-01T00:00:00Z",
                "decision": "approved",
                "evidenceRefs": [],
                "policyRefs": [],
                "reason": "Historical close record predates repository identity binding.",
                "resumeCondition": "none"
            },
            "timestamp": "2026-01-01T00:00:00Z",
            "workItemId": work_item_id,
        })
        .to_string(),
    )
    .expect("legacy close");

    start_work_item_with_options(
        directory.path(),
        "WI-AFTER-LEGACY",
        "start after a historical close",
        "ensure old close records do not deadlock new work",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("legacy closed archive must not block entry");
}

#[test]
fn historical_noncanonical_close_receipt_does_not_block_new_work_item_entry() {
    let directory = repository();
    let work_item_id = "WI-HISTORICAL-NONCANONICAL-CLOSE";
    start_work_item(
        directory.path(),
        work_item_id,
        "historical close compatibility",
        "accept an older complete close receipt without rewriting it",
        &["**".into()],
    )
    .expect("start");
    assert_no_resource_context(&directory, work_item_id);
    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    record_verification(
        directory.path(),
        work_item_id,
        &serde_json::json!({"passed": true}),
        "0.1.0",
        &Digest::sha256_bytes(b"status-runtime"),
    )
    .expect("verification");
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");
    close_work_item_with_structured_decision(
        directory.path(),
        work_item_id,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "historical-compatibility-test".into(),
            reason: "record the complete close receipt before applying the legacy shape".into(),
            evidence_refs: vec![],
            policy_refs: vec![],
            decided_at: "2026-08-22T12:00:00Z".into(),
            resume_condition: None,
        },
    )
    .expect("close");

    let close_path = directory
        .path()
        .join(format!(".ai/decisions/{work_item_id}.close.json"));
    let mut close: serde_json::Value =
        serde_json::from_slice(&fs::read(&close_path).expect("close receipt")).expect("close JSON");
    close["humanDecision"] = "ready".into();
    close["structuredDecision"]["decision"] = "ready".into();
    let valid_legacy_close = close.clone();
    fs::write(
        &close_path,
        serde_json::to_vec_pretty(&close).expect("legacy close bytes"),
    )
    .expect("legacy close");

    let status = work_item_status_snapshot_with_runtime(directory.path(), work_item_id, &runtime())
        .expect("historical close status");
    assert_eq!(status.lifecycle_phase, "closed");
    assert!(!status.blocking);

    close["finalReportDigest"] = Digest::sha256_bytes(b"tampered-final-report")
        .to_string()
        .into();
    fs::write(
        &close_path,
        serde_json::to_vec_pretty(&close).expect("tampered close bytes"),
    )
    .expect("tampered close");
    let invalid_status =
        work_item_status_snapshot_with_runtime(directory.path(), work_item_id, &runtime())
            .expect("invalid historical close status");
    assert_eq!(invalid_status.lifecycle_phase, "archived");
    assert!(invalid_status.blocking);
    assert!(
        invalid_status
            .unknowns
            .contains(&"close_decision_invalid".into())
    );

    fs::write(
        &close_path,
        serde_json::to_vec_pretty(&valid_legacy_close).expect("restored legacy close bytes"),
    )
    .expect("restore legacy close");

    start_work_item_with_options(
        directory.path(),
        "WI-AFTER-HISTORICAL-NONCANONICAL-CLOSE",
        "start after historical noncanonical close",
        "ensure historical close compatibility does not deadlock entry",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("historical noncanonical close must not block entry");
}

#[test]
fn current_archive_without_close_blocks_new_work_item_entry() {
    let directory = repository();
    let work_item_id = "WI-CURRENT-UNCLOSED";
    let repository_id = repository_id(directory.path()).to_string();
    let archive = directory.path().join(".ai/work-items/archive");
    fs::write(
        archive.join(format!("{work_item_id}.contract.json")),
        serde_json::json!({
            "workItemId": work_item_id,
            "repositoryId": repository_id,
            "scope": ["src/**"],
        })
        .to_string(),
    )
    .expect("current contract");
    fs::write(
        archive.join(format!("{work_item_id}.archive.json")),
        serde_json::json!({
            "workItemId": work_item_id,
            "state": "archived",
            "closeRequired": true,
        })
        .to_string(),
    )
    .expect("current archive");

    let error = start_work_item_with_options(
        directory.path(),
        "WI-BLOCKED-BY-CURRENT",
        "start after an unclosed current archive",
        "preserve the close gate for new archives",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect_err("current archive without close must block entry");
    assert!(error.to_string().contains(work_item_id));
}
