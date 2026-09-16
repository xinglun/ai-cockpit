use cockpit_core::Digest;
use cockpit_protocol::{ResourceFinalizationContext, RuntimeContext};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item,
    close_work_item_with_structured_decision_and_runtime, finish_work_item_with_runtime,
    plan_resource_finalization, preflight_work_item, record_resource_finalization,
    record_selected_successor_lineage_recovery, record_verification_with_runtime, repository_id,
    run_repository_verification, start_work_item_with_options,
};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "0.1.0".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"selected-lineage-test-runtime"),
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

fn raw_digest(path: &Path) -> Digest {
    Digest::sha256_bytes(&fs::read(path).expect("read immutable fixture"))
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read JSON fixture")).expect("JSON fixture")
}

fn resource_context(directory: &Path, work_item_id: &str) -> ResourceFinalizationContext {
    ResourceFinalizationContext {
        branch: format!("feature/{work_item_id}"),
        worktree: directory
            .join(format!("missing-worktree-{work_item_id}"))
            .display()
            .to_string(),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: "github".into(),
        pull_request: format!("https://github.com/example/ai-cockpit/pull/{work_item_id}"),
    }
}

fn complete_resource_work_item(directory: &Path, work_item_id: &str) {
    let current_runtime = runtime();
    start_work_item_with_options(
        directory,
        work_item_id,
        "complete a selected lineage fixture",
        "preserve a repository-bound terminal successor node",
        &[".ai/**".into(), "src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["all terminal bytes remain immutable".into()],
            ..Default::default()
        },
    )
    .expect("start");
    let context = resource_context(directory, work_item_id);
    plan_resource_finalization(directory, work_item_id, &context).expect("finalization plan");
    let contract_path = directory
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    preflight_work_item(directory, &contract_path).expect("preflight");
    checkpoint_work_item(directory, work_item_id).expect("checkpoint");
    let run = run_repository_verification(
        directory,
        &RepositoryVerificationRequest {
            node_id: format!("{work_item_id}-check"),
            program: "true".into(),
            args: Vec::new(),
            scope: vec![".ai/**".into(), "src/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current_runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("verification");
    record_verification_with_runtime(
        directory,
        work_item_id,
        &serde_json::to_value(&run.receipt).expect("verification JSON"),
        &current_runtime,
        &run.final_snapshot,
    )
    .expect("record verification");
    finish_work_item_with_runtime(directory, work_item_id, &current_runtime).expect("finish");
    archive_work_item_with_runtime(directory, work_item_id, &current_runtime).expect("archive");

    let archive = directory.join(".ai/work-items/archive");
    let archived_contract = read_json(&archive.join(format!("{work_item_id}.contract.json")));
    let finalization = json!({
        "schemaVersion": 1,
        "receiptId": format!("{work_item_id}-finalize"),
        "operationId": format!("{work_item_id}-finalize-operation"),
        "repositoryId": repository_id(directory).to_string(),
        "workItemId": work_item_id,
        "runtimeVersion": current_runtime.runtime_version,
        "runtimeDigest": current_runtime.runtime_digest,
        "provider": "github",
        "pullRequest": {
            "number": 1,
            "url": format!("https://github.com/example/ai-cockpit/pull/{work_item_id}"),
            "headRevision": "head",
            "baseBranch": "main",
            "baseRemote": "origin",
            "baseRevision": archived_contract["baseRevision"],
            "mergeCommit": "merge"
        },
        "branch": {"name": context.branch, "remote": "origin", "headRevision": "head"},
        "worktree": {"worktreeId": "removed", "path": context.worktree, "branch": context.branch, "headRevision": "head"},
        "before": {"pullRequest": "merged", "branch": "present", "worktree": "clean"},
        "after": {"pullRequest": "merged", "branch": "deleted", "worktree": "removed"},
        "result": {"disposition": "deleted", "failureCodes": [], "unknownCodes": []},
        "actor": "human:test",
        "authoritySource": "selected-lineage-test",
        "reason": "the fixture resources are already absent",
        "timestamp": "2026-09-16T00:00:00Z",
        "contractDigest": raw_digest(&archive.join(format!("{work_item_id}.contract.json"))),
        "resourceContext": context
    });
    let finalization_input = tempfile::NamedTempFile::new().expect("finalization input");
    fs::write(
        finalization_input.path(),
        serde_json::to_vec_pretty(&finalization).expect("finalization JSON"),
    )
    .expect("write finalization input");
    record_resource_finalization(
        directory,
        work_item_id,
        finalization_input.path(),
        &current_runtime,
    )
    .expect("record finalization");
    close_work_item_with_structured_decision_and_runtime(
        directory,
        work_item_id,
        &cockpit_protocol::HumanDecision {
            decision: "approved".into(),
            actor: "human:test".into(),
            authority_source: "selected-lineage-test".into(),
            reason: "terminal fixture is complete".into(),
            evidence_refs: vec![format!(".ai/evidence/{work_item_id}.verification.json")],
            policy_refs: vec!["selected-lineage-test".into()],
            decided_at: "2026-09-16T00:01:00Z".into(),
            resume_condition: None,
        },
        &current_runtime,
    )
    .expect("close");
}

fn recovery_edge(directory: &Path, predecessor: &str, successor: &str) -> (Value, Digest) {
    let archive = directory.join(".ai/work-items/archive");
    let contract = read_json(&archive.join(format!("{predecessor}.contract.json")));
    let summary = read_json(&archive.join(format!("{predecessor}.summary.json")));
    let outcome = read_json(&archive.join(format!("{predecessor}.outcome.json")));
    let events_path = archive.join(format!("{predecessor}.events.jsonl"));
    let manifest_path = archive.join(format!("{predecessor}.archive.json"));
    let recovery = json!({
        "schemaVersion": 1,
        "decisionId": "work-item-recovery",
        "decision": "supersede",
        "workItemId": predecessor,
        "repositoryId": repository_id(directory),
        "predecessorWorkItemId": predecessor,
        "predecessorContractDigest": cockpit_protocol::digest_json(&contract).expect("contract digest"),
        "predecessorSummaryDigest": cockpit_protocol::digest_json(&summary).expect("summary digest"),
        "predecessorOutcomeDigest": cockpit_protocol::digest_json(&outcome).expect("outcome digest"),
        "predecessorEventsDigest": raw_digest(&events_path),
        "predecessorArchiveManifestDigest": raw_digest(&manifest_path),
        "successorWorkItemId": successor,
        "runtimeVersion": runtime().runtime_version,
        "runtimeDigest": runtime().runtime_digest,
        "actor": "human:test",
        "authoritySource": "selected-lineage-test",
        "reason": "the selected successor is already terminal",
        "evidenceRefs": [format!(".ai/work-items/archive/{predecessor}.close.json")],
        "policyRefs": ["selected-lineage-test"],
        "decidedAt": "2026-09-16T00:02:00Z",
        "resumeCondition": "the selected lineage aggregate is recorded"
    });
    let bytes = serde_json::to_vec_pretty(&recovery).expect("recovery JSON");
    fs::create_dir_all(directory.join(".ai/decisions")).expect("decisions directory");
    let path = directory
        .join(".ai/decisions")
        .join(format!("{predecessor}.recovery.json"));
    fs::write(&path, bytes).expect("write recovery edge");
    (recovery, raw_digest(&path))
}

fn aggregate(directory: &Path, root: &str, middle: &str, leaf: &str) -> Value {
    let archive = directory.join(".ai/work-items/archive");
    let decisions = directory.join(".ai/decisions");
    let node = |work_item_id: &str| {
        let archived = |kind: &str| archive.join(format!("{work_item_id}.{kind}.json"));
        let contract_path = archived("contract");
        let summary_path = archived("summary");
        let outcome_path = archived("outcome");
        let events_path = archive.join(format!("{work_item_id}.events.jsonl"));
        let verification_path = directory
            .join(".ai/evidence")
            .join(format!("{work_item_id}.verification.json"));
        let manifest_path = archived("archive");
        let close_path = decisions.join(format!("{work_item_id}.close.json"));
        let finalization_path = decisions.join(format!("{work_item_id}.finalize.json"));
        let finalization = read_json(&finalization_path);
        json!({
            "workItemId": work_item_id,
            "contractPath": format!(".ai/work-items/archive/{work_item_id}.contract.json"),
            "contractDigest": raw_digest(&contract_path),
            "summaryPath": format!(".ai/work-items/archive/{work_item_id}.summary.json"),
            "summaryDigest": raw_digest(&summary_path),
            "outcomePath": format!(".ai/work-items/archive/{work_item_id}.outcome.json"),
            "outcomeDigest": raw_digest(&outcome_path),
            "eventsPath": format!(".ai/work-items/archive/{work_item_id}.events.jsonl"),
            "eventsDigest": raw_digest(&events_path),
            "verificationPath": format!(".ai/evidence/{work_item_id}.verification.json"),
            "verificationDigest": raw_digest(&verification_path),
            "archiveManifestPath": format!(".ai/work-items/archive/{work_item_id}.archive.json"),
            "archiveManifestDigest": raw_digest(&manifest_path),
            "closePath": format!(".ai/decisions/{work_item_id}.close.json"),
            "closeDigest": raw_digest(&close_path),
            "finalizationPath": format!(".ai/decisions/{work_item_id}.finalize.json"),
            "finalizationDigest": raw_digest(&finalization_path),
            "finalizationProvider": finalization["provider"],
            "finalizationPullRequest": finalization["pullRequest"],
            "finalizationRuntimeVersion": finalization["runtimeVersion"],
            "finalizationRuntimeDigest": finalization["runtimeDigest"],
            "humanDecision": {
                "actor": "human:test",
                "authoritySource": "selected-lineage-test",
                "reason": "the selected lineage node is terminal",
                "evidenceRefs": [format!(".ai/decisions/{work_item_id}.close.json")],
                "policyRefs": ["selected-lineage-test"],
                "decidedAt": "2026-09-16T00:03:00Z",
                "resumeCondition": "the selected lineage remains immutable"
            }
        })
    };
    let (edge0, digest0) = recovery_edge(directory, root, middle);
    let (edge1, digest1) = recovery_edge(directory, middle, leaf);
    let edge = |sequence: u64, predecessor: &str, successor: &str, digest: Digest| {
        json!({
            "sequence": sequence,
            "predecessorWorkItemId": predecessor,
            "successorWorkItemId": successor,
            "recoveryPath": format!(".ai/decisions/{predecessor}.recovery.json"),
            "recoveryDigest": digest
        })
    };
    let _ = (edge0, edge1);
    json!({
        "schemaVersion": 1,
        "receiptId": "selected-successor-lineage-recovery",
        "rootWorkItemId": root,
        "repositoryId": repository_id(directory),
        "edges": [
            edge(0, root, middle, digest0),
            edge(1, middle, leaf, digest1)
        ],
        "nodes": [node(root), node(middle), node(leaf)],
        "runtimeVersion": runtime().runtime_version,
        "runtimeDigest": runtime().runtime_digest
    })
}

#[test]
fn selected_lineage_recovery_is_append_only_and_rejects_tampering() {
    let directory = repository();
    let root = "WI-SELECTED-LINEAGE-ROOT";
    let middle = "WI-SELECTED-LINEAGE-MIDDLE";
    let leaf = "WI-SELECTED-LINEAGE-LEAF";
    complete_resource_work_item(directory.path(), root);
    complete_resource_work_item(directory.path(), middle);
    complete_resource_work_item(directory.path(), leaf);

    let receipt = aggregate(directory.path(), root, middle, leaf);
    let recorded =
        record_selected_successor_lineage_recovery(directory.path(), root, &receipt, &runtime())
            .expect("record valid selected lineage");
    assert_eq!(recorded, receipt);
    let aggregate_path = directory.path().join(format!(
        ".ai/decisions/{root}.selected-successor-lineage-recovery.json"
    ));
    let original = fs::read(&aggregate_path).expect("aggregate bytes");
    record_selected_successor_lineage_recovery(directory.path(), root, &receipt, &runtime())
        .expect("replay is idempotent");
    assert_eq!(fs::read(&aggregate_path).expect("replayed bytes"), original);

    let tampered_path = directory
        .path()
        .join(format!(".ai/work-items/archive/{middle}.summary.json"));
    let mut tampered = fs::read(&tampered_path).expect("summary bytes");
    tampered.extend_from_slice(b"\n");
    fs::write(&tampered_path, tampered).expect("tamper summary");
    let error =
        record_selected_successor_lineage_recovery(directory.path(), root, &receipt, &runtime())
            .expect_err("tampered immutable node must fail closed");
    assert!(error.to_string().contains("digest"));
    assert_eq!(
        fs::read(&aggregate_path).expect("preserved aggregate"),
        original
    );
}
