use cockpit_core::Digest;
use cockpit_protocol::{HumanDecision, ResourceFinalizationContext, RuntimeContext};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item, attach, checkpoint_work_item, close_work_item_with_structured_decision,
    finish_work_item, outcome_v2_with_runtime, plan_resource_finalization, preflight_work_item,
    record_verification, record_verification_with_runtime, render_human_outcome, repository_id,
    run_repository_verification, start_work_item, start_work_item_with_options,
    work_item_status_index_with_runtime, work_item_status_snapshot_with_runtime,
};
use std::{fs, process::Command};

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

fn plan(directory: &tempfile::TempDir, work_item_id: &str) {
    plan_resource_finalization(
        directory.path(),
        work_item_id,
        &ResourceFinalizationContext {
            branch: format!("feature/{work_item_id}"),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: format!("https://github.com/example/ai-cockpit/pull/{work_item_id}"),
        },
    )
    .expect("finalization plan");
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
            &["**".into()],
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
    start_work_item(
        directory.path(),
        work_item_id,
        "status close projection",
        "show terminal close state",
        &["**".into()],
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
            scope: vec!["**".into()],
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
    let outcome = outcome_v2_with_runtime(directory.path(), work_item_id, &runtime())
        .expect("archived outcome");
    assert_eq!(
        outcome.decision_state,
        Some(cockpit_core::DecisionState::Yellow)
    );
    assert!(
        outcome
            .unknowns
            .contains(&"resource_finalization_pending".into())
    );
    let handoff = render_human_outcome(directory.path(), &outcome, "zh");
    assert!(handoff.starts_with("Outcome: 🟡"));
    assert!(handoff.contains("provider finalization"));
    assert!(handoff.contains("finalize-verify"));
    assert!(handoff.contains("close"));

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
fn invalid_close_decision_never_promotes_archived_status() {
    let directory = repository();
    let work_item_id = "WI-STATUS-INVALID-CLOSE";
    start_work_item(
        directory.path(),
        work_item_id,
        "status invalid close",
        "reject invalid close projection",
        &["**".into()],
    )
    .expect("start");
    plan(&directory, work_item_id);
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
        &["**".into()],
    )
    .expect("start");
    plan(&directory, work_item_id);
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
