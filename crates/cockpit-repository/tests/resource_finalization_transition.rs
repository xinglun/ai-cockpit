use cockpit_core::Digest;
use cockpit_protocol::{
    HumanDecision, ResourceFinalizationContext, ResourceFinalizationTransitionReceipt,
    RuntimeContext,
};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item,
    close_work_item_with_structured_decision_and_runtime, finish_work_item_with_runtime,
    plan_resource_finalization, preflight_work_item_with_runtime, record_resource_finalization,
    record_verification_with_runtime, run_repository_verification, start_work_item_with_options,
    verify_resource_finalization,
};
use serde_json::{Value, json};
use std::{fs, process::Command};

const ID: &str = "WI-FINALIZATION-TRANSITION";

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    }
}

fn repository() -> (tempfile::TempDir, ResourceFinalizationContext, Digest) {
    let directory = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .unwrap()
            .success()
    );
    attach(directory.path()).unwrap();
    start_work_item_with_options(
        directory.path(),
        ID,
        "append finalization transitions",
        "preserve history",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["transition chain is linear".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let context = ResourceFinalizationContext {
        branch: "feature/finalization-transition".into(),
        worktree: "/tmp/removed-finalization-transition".into(),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: "github".into(),
        pull_request: "https://github.com/example/project/pull/191".into(),
    };
    plan_resource_finalization(directory.path(), ID, &context).unwrap();
    let current = runtime();
    let contract = directory
        .path()
        .join(format!(".ai/work-items/active/{ID}.contract.json"));
    preflight_work_item_with_runtime(directory.path(), &contract, &current).unwrap();
    checkpoint_work_item(directory.path(), ID).unwrap();
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "project-command-0".into(),
            program: "true".into(),
            args: vec![],
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .unwrap();
    let mut evidence = serde_json::to_value(&run.receipt).unwrap();
    evidence["runtimeVersion"] = current.runtime_version.clone().into();
    evidence["runtimeDigest"] = current.runtime_digest.to_string().into();
    record_verification_with_runtime(
        directory.path(),
        ID,
        &evidence,
        &current,
        &run.final_snapshot,
    )
    .unwrap();
    finish_work_item_with_runtime(directory.path(), ID, &current).unwrap();
    archive_work_item_with_runtime(directory.path(), ID, &current).unwrap();
    let archived = directory
        .path()
        .join(format!(".ai/work-items/archive/{ID}.contract.json"));
    let digest = Digest::sha256_bytes(&fs::read(archived).unwrap());
    (directory, context, digest)
}

fn blocked(repository_id: &str, context: &ResourceFinalizationContext, contract: &Digest) -> Value {
    json!({
        "schemaVersion":1,"receiptId":"blocked-1","operationId":"operation-1",
        "repositoryId":repository_id,
        "workItemId":ID,"runtimeVersion":"test-runtime","runtimeDigest":runtime().runtime_digest,
        "provider":"github","pullRequest":{"number":191,"url":context.pull_request,"headRevision":"head-191","baseBranch":"main","baseRemote":"origin","baseRevision":"base-191"},
        "branch":{"name":context.branch,"remote":"origin","headRevision":"head-191"},
        "worktree":{"worktreeId":"worktree-191","path":context.worktree,"branch":context.branch,"headRevision":"head-191"},
        "before":{"pullRequest":"unmerged","branch":"present","worktree":"clean"},
        "after":{"pullRequest":"unmerged","branch":"present","worktree":"clean"},
        "result":{"disposition":"blocked","failureCodes":["unmerged_pull_request"],"unknownCodes":[]},
        "actor":"human:test","authoritySource":"test","reason":"await merge","timestamp":"2026-08-23T00:00:00Z",
        "contractDigest":contract,"resourceContext":context
    })
}

fn transition(previous: &Value, sequence: u64, deleted: bool) -> Value {
    let previous_receipt: cockpit_protocol::ResourceFinalizationReceipt =
        serde_json::from_value(previous.clone()).unwrap();
    let mut next = previous_receipt.clone();
    next.receipt_id = format!("receipt-{}", sequence + 1);
    next.operation_id = format!("operation-{}", sequence + 1);
    next.pull_request.merge_commit = Some("merge-191".into());
    next.before = previous_receipt.after.clone();
    next.after.pull_request = cockpit_protocol::ResourceFinalizationPullRequestState::Merged;
    next.result.disposition = cockpit_protocol::ResourceFinalizationDisposition::Retained;
    next.result.failure_codes.clear();
    if deleted {
        next.after.branch = cockpit_protocol::ResourceFinalizationBranchState::Deleted;
        next.after.worktree = cockpit_protocol::ResourceFinalizationWorktreeState::Removed;
        next.result.disposition = cockpit_protocol::ResourceFinalizationDisposition::Deleted;
    }
    serde_json::to_value(ResourceFinalizationTransitionReceipt {
        schema_version: 1,
        transition_id: format!("transition-{sequence}"),
        sequence,
        predecessor_receipt_digest: cockpit_protocol::digest_json(previous).unwrap(),
        governance_append_revision: None,
        receipt: next,
    })
    .unwrap()
}

fn set_receipt_head(value: &mut Value, head: &str) {
    value["pullRequest"]["headRevision"] = head.into();
    value["branch"]["headRevision"] = head.into();
    value["worktree"]["headRevision"] = head.into();
}

fn git(directory: &tempfile::TempDir, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(directory.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn commit_archive(directory: &tempfile::TempDir) -> String {
    git(
        directory,
        &["config", "user.email", "tests@example.invalid"],
    );
    git(directory, &["config", "user.name", "AI Cockpit Tests"]);
    git(directory, &["add", "."]);
    git(directory, &["commit", "-q", "-m", "archive"]);
    git(directory, &["rev-parse", "HEAD"])
}

fn write_input(directory: &tempfile::TempDir, name: &str, value: &Value) -> std::path::PathBuf {
    let path = directory.path().join(name);
    fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    path
}

fn transition_path(decisions: &std::path::Path, value: &Value) -> std::path::PathBuf {
    let digest = cockpit_protocol::digest_json(value).unwrap().to_string();
    decisions.join(format!(
        "{ID}.finalize.{}.json",
        digest.strip_prefix("sha256:").unwrap_or(&digest)
    ))
}

#[test]
fn wi190_topology_appends_two_transitions_and_resolves_deleted_head() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let blocked = blocked(&repository_id, &context, &contract);
    let canonical_input = write_input(&directory, "blocked.json", &blocked);
    record_resource_finalization(directory.path(), ID, &canonical_input, &runtime()).unwrap();
    let canonical = directory
        .path()
        .join(format!(".ai/decisions/{ID}.finalize.json"));
    let original = fs::read(&canonical).unwrap();

    let observed = transition(&blocked, 1, false);
    let observed_input = write_input(&directory, "observed.json", &observed);
    let result =
        record_resource_finalization(directory.path(), ID, &observed_input, &runtime()).unwrap();
    assert_eq!(result["state"], "appended");
    let observed_receipt = observed["receipt"].clone();
    let deleted = transition(&observed_receipt, 2, true);
    let deleted_input = write_input(&directory, "deleted.json", &deleted);
    record_resource_finalization(directory.path(), ID, &deleted_input, &runtime()).unwrap();

    assert_eq!(fs::read(canonical).unwrap(), original);
    let verified = verify_resource_finalization(directory.path(), ID, &runtime()).unwrap();
    assert_eq!(verified["disposition"], "deleted");
    assert_eq!(verified["sequence"], 2);

    fs::remove_file(canonical_input).unwrap();
    fs::remove_file(observed_input).unwrap();
    fs::remove_file(deleted_input).unwrap();
    close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        ID,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:test".into(),
            authority_source: "test".into(),
            reason: "verified transition head".into(),
            evidence_refs: vec![],
            policy_refs: vec![],
            decided_at: "2026-08-23T00:10:00Z".into(),
            resume_condition: None,
        },
        &runtime(),
    )
    .unwrap();
    let close: Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/decisions/{ID}.close.json")),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(close["resourceFinalizationSequence"], 2);
    assert_eq!(
        close["resourceFinalizationHeadDigest"],
        verified["headDigest"]
    );
}

#[test]
fn wi191_governance_receipt_append_allows_bounded_merge_observation() {
    // This models the real WI-191 governance-only 70c17e4 -> 8f5a025 append:
    // the second commit adds only the canonical finalization receipt.
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let archive_head = commit_archive(&directory);
    let mut blocked = blocked(&repository_id, &context, &contract);
    set_receipt_head(&mut blocked, &archive_head);
    let canonical_input = write_input(&directory, "blocked.json", &blocked);
    record_resource_finalization(directory.path(), ID, &canonical_input, &runtime()).unwrap();
    let canonical_relative = format!(".ai/decisions/{ID}.finalize.json");
    git(&directory, &["add", &canonical_relative]);
    git(
        &directory,
        &["commit", "-q", "-m", "append canonical governance receipt"],
    );
    let append_head = git(&directory, &["rev-parse", "HEAD"]);

    let mut observed = transition(&blocked, 1, false);
    set_receipt_head(&mut observed["receipt"], &append_head);
    observed["governanceAppendRevision"] = append_head.clone().into();
    let observed_input = write_input(&directory, "observed.json", &observed);
    record_resource_finalization(directory.path(), ID, &observed_input, &runtime()).unwrap();

    let observed_receipt = observed["receipt"].clone();
    let deleted = transition(&observed_receipt, 2, true);
    let deleted_input = write_input(&directory, "deleted.json", &deleted);
    record_resource_finalization(directory.path(), ID, &deleted_input, &runtime()).unwrap();
    let verified = verify_resource_finalization(directory.path(), ID, &runtime()).unwrap();
    assert_eq!(verified["sequence"], 2);
    assert_eq!(verified["disposition"], "deleted");
}

#[test]
fn unrelated_or_malformed_governance_append_fails_closed() {
    for foreign in [
        "unrelated.txt".to_string(),
        format!(".ai/decisions/{ID}.finalize.bad.json"),
    ] {
        let (directory, context, contract) = repository();
        let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
        let archive_head = commit_archive(&directory);
        let mut blocked = blocked(&repository_id, &context, &contract);
        set_receipt_head(&mut blocked, &archive_head);
        let canonical_input = write_input(&directory, "blocked.json", &blocked);
        record_resource_finalization(directory.path(), ID, &canonical_input, &runtime()).unwrap();
        fs::write(directory.path().join(&foreign), b"foreign").unwrap();
        let canonical_relative = format!(".ai/decisions/{ID}.finalize.json");
        git(&directory, &["add", &canonical_relative, &foreign]);
        git(&directory, &["commit", "-q", "-m", "foreign append"]);
        let append_head = git(&directory, &["rev-parse", "HEAD"]);
        let mut observed = transition(&blocked, 1, false);
        set_receipt_head(&mut observed["receipt"], &append_head);
        observed["governanceAppendRevision"] = append_head.into();
        let observed_input = write_input(&directory, "observed.json", &observed);
        assert!(
            record_resource_finalization(directory.path(), ID, &observed_input, &runtime())
                .is_err()
        );
    }
}

#[cfg(unix)]
#[test]
fn symlinked_governance_append_receipt_fails_closed() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let archive_head = commit_archive(&directory);
    let mut blocked = blocked(&repository_id, &context, &contract);
    set_receipt_head(&mut blocked, &archive_head);
    let canonical_input = write_input(&directory, "blocked.json", &blocked);
    record_resource_finalization(directory.path(), ID, &canonical_input, &runtime()).unwrap();
    let canonical_relative = format!(".ai/decisions/{ID}.finalize.json");
    let symlink_relative = format!(".ai/decisions/{ID}.finalize.{}.json", "0".repeat(64));
    std::os::unix::fs::symlink(
        format!("{ID}.finalize.json"),
        directory.path().join(&symlink_relative),
    )
    .unwrap();
    git(&directory, &["add", &canonical_relative, &symlink_relative]);
    git(&directory, &["commit", "-q", "-m", "symlink append"]);
    let append_head = git(&directory, &["rev-parse", "HEAD"]);
    let mut observed = transition(&blocked, 1, false);
    set_receipt_head(&mut observed["receipt"], &append_head);
    observed["governanceAppendRevision"] = append_head.into();
    let observed_input = write_input(&directory, "observed.json", &observed);
    assert!(
        record_resource_finalization(directory.path(), ID, &observed_input, &runtime()).is_err()
    );
}

#[test]
fn forked_and_symlinked_transition_candidates_fail_closed() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let blocked = blocked(&repository_id, &context, &contract);
    let input = write_input(&directory, "blocked.json", &blocked);
    record_resource_finalization(directory.path(), ID, &input, &runtime()).unwrap();
    let first = transition(&blocked, 1, false);
    let mut second = first.clone();
    second["transitionId"] = "fork".into();
    second["receipt"]["receiptId"] = "fork-receipt".into();
    let decisions = directory.path().join(".ai/decisions");
    let first_path = transition_path(&decisions, &first);
    let second_path = transition_path(&decisions, &second);
    fs::write(&first_path, serde_json::to_vec(&first).unwrap()).unwrap();
    fs::write(&second_path, serde_json::to_vec(&second).unwrap()).unwrap();
    assert!(verify_resource_finalization(directory.path(), ID, &runtime()).is_err());

    fs::remove_file(second_path).unwrap();
    #[cfg(unix)]
    {
        let symlink = decisions.join(format!("{ID}.finalize.symlink.json"));
        std::os::unix::fs::symlink("missing", &symlink).unwrap();
        assert!(verify_resource_finalization(directory.path(), ID, &runtime()).is_err());
        fs::remove_file(symlink).unwrap();
    }
    fs::write(
        decisions.join(format!("{ID}.finalize.malformed.json")),
        b"{\"schemaVersion\":1,\"schemaVersion\":1}",
    )
    .unwrap();
    assert!(verify_resource_finalization(directory.path(), ID, &runtime()).is_err());
}

#[test]
fn stale_missing_foreign_runtime_and_contract_transitions_fail_closed() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let blocked = blocked(&repository_id, &context, &contract);
    let input = write_input(&directory, "blocked.json", &blocked);
    record_resource_finalization(directory.path(), ID, &input, &runtime()).unwrap();

    let mut stale = transition(&blocked, 1, false);
    stale["predecessorReceiptDigest"] = Digest::sha256_bytes(b"missing").to_string().into();
    let stale_input = write_input(&directory, "stale.json", &stale);
    assert!(record_resource_finalization(directory.path(), ID, &stale_input, &runtime()).is_err());

    let mut foreign_runtime = transition(&blocked, 1, false);
    foreign_runtime["receipt"]["runtimeDigest"] =
        Digest::sha256_bytes(b"foreign").to_string().into();
    let foreign_input = write_input(&directory, "foreign-runtime.json", &foreign_runtime);
    assert!(
        record_resource_finalization(directory.path(), ID, &foreign_input, &runtime()).is_err()
    );

    let mut foreign_contract = transition(&blocked, 1, false);
    foreign_contract["receipt"]["contractDigest"] =
        Digest::sha256_bytes(b"foreign-contract").to_string().into();
    let contract_input = write_input(&directory, "foreign-contract.json", &foreign_contract);
    assert!(
        record_resource_finalization(directory.path(), ID, &contract_input, &runtime()).is_err()
    );

    let decisions = directory.path().join(".ai/decisions");
    fs::write(
        transition_path(&decisions, &stale),
        serde_json::to_vec(&stale).unwrap(),
    )
    .unwrap();
    assert!(verify_resource_finalization(directory.path(), ID, &runtime()).is_err());
}
