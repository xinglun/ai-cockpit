use cockpit_core::{DecisionState, Digest};
use cockpit_protocol::{
    DataClassification, EvidencePersistence, EvidenceRetention, HumanDecision,
    ResourceFinalizationContext, RuntimeContext,
};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item,
    close_work_item_with_structured_decision_and_runtime, finish_work_item_with_runtime,
    outcome_v2_with_runtime, plan_resource_finalization, preflight_work_item_with_runtime,
    record_resource_finalization, record_verification_with_runtime, render_human_outcome,
    run_repository_verification, set_evidence_retention_policy, start_work_item_with_options,
};
use serde_json::Value;
use std::{fs, process::Command};

type EvidenceMutation = (&'static str, fn(&mut Value));
type RetentionMutation = (&'static str, fn(&mut Value, &mut Value));

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

fn runtime(label: &str) -> RuntimeContext {
    RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(label.as_bytes()),
    }
}

fn start(directory: &tempfile::TempDir, id: &str) {
    start_work_item_with_options(
        directory.path(),
        id,
        "evidence assurance",
        "validate typed verification evidence",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
}

fn plan(directory: &tempfile::TempDir, id: &str) {
    plan_resource_finalization(
        directory.path(),
        id,
        &ResourceFinalizationContext {
            branch: format!("feature/{id}"),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: format!("https://github.com/example/ai-cockpit/pull/{id}"),
        },
    )
    .expect("resource context plan");
}

fn record_typed(directory: &tempfile::TempDir, id: &str, current: &RuntimeContext) -> Value {
    let contract = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    preflight_work_item_with_runtime(directory.path(), &contract, current).expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "project-command-0".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("verify");
    let mut raw = serde_json::to_value(&run.receipt).expect("receipt JSON");
    raw["runtimeVersion"] = current.runtime_version.clone().into();
    raw["runtimeDigest"] = current.runtime_digest.to_string().into();
    record_verification_with_runtime(directory.path(), id, &raw, current, &run.final_snapshot)
        .expect("record typed evidence")
}

#[test]
fn strict_evidence_rejects_unknown_envelope_and_nested_fields() {
    let cases: [EvidenceMutation; 9] = [
        ("unknown-envelope", |evidence: &mut Value| {
            evidence["unexpected"] = true.into();
        }),
        ("unknown-receipt", |evidence: &mut Value| {
            evidence["receipt"]["unexpected"] = true.into();
        }),
        ("unknown-result", |evidence: &mut Value| {
            let result = evidence["receipt"]["results"]
                .as_array_mut()
                .and_then(|results| results.first_mut())
                .expect("verification should produce a result");
            result["unexpected"] = true.into();
        }),
        ("missing-nested-repository", |evidence: &mut Value| {
            evidence["receipt"]
                .as_object_mut()
                .expect("receipt object")
                .remove("repositoryId");
        }),
        ("malformed-receipt", |evidence: &mut Value| {
            evidence["receipt"] = Value::String("not-a-receipt".into());
        }),
        ("invalid-created-at", |evidence: &mut Value| {
            evidence["createdAt"] = "not-an-rfc3339-time".into();
        }),
        ("missing-created-at", |evidence: &mut Value| {
            evidence
                .as_object_mut()
                .expect("evidence object")
                .remove("createdAt");
        }),
        ("invalid-retention-created-at", |evidence: &mut Value| {
            evidence["retention"] = serde_json::json!({
                "schemaVersion": 1,
                "repositoryId": evidence["repositoryId"].clone(),
                "workItemId": evidence["workItemId"].clone(),
                "retention": {
                    "classification": "internal",
                    "persistence": "full_capture",
                    "retentionDays": 30,
                    "expiresAt": null,
                    "disposalAction": "retain"
                },
                "createdAt": "not-an-rfc3339-time"
            });
        }),
        ("invalid-retention-expires-at", |evidence: &mut Value| {
            evidence["retention"] = serde_json::json!({
                "schemaVersion": 1,
                "repositoryId": evidence["repositoryId"].clone(),
                "workItemId": evidence["workItemId"].clone(),
                "retention": {
                    "classification": "internal",
                    "persistence": "full_capture",
                    "retentionDays": null,
                    "expiresAt": "not-an-rfc3339-time",
                    "disposalAction": "retain"
                },
                "createdAt": "2026-08-22T11:00:00Z"
            });
        }),
    ];
    for (name, mutate) in cases {
        let directory = repository();
        let current = runtime(name);
        start(&directory, "WI-110-STRICT");
        record_typed(&directory, "WI-110-STRICT", &current);
        let path = directory
            .path()
            .join(".ai/evidence/WI-110-STRICT.verification.json");
        let mut evidence: Value =
            serde_json::from_slice(&fs::read(&path).expect("evidence")).expect("evidence JSON");
        mutate(&mut evidence);
        fs::write(&path, serde_json::to_vec_pretty(&evidence).expect("JSON"))
            .expect("tamper evidence");
        let outcome =
            outcome_v2_with_runtime(directory.path(), "WI-110-STRICT", &current).expect("outcome");
        assert_eq!(outcome.decision_state, Some(DecisionState::Red), "{name}");
        assert_ne!(
            outcome.state,
            cockpit_protocol::OutcomeState::Verified,
            "{name}"
        );
    }
}

#[test]
fn retention_schema_and_identity_are_bound_to_evidence_and_repository() {
    let mutations: [RetentionMutation; 6] = [
        ("standalone-schema", |standalone, _embedded| {
            standalone["schemaVersion"] = 999.into();
        }),
        ("standalone-repository", |standalone, _embedded| {
            standalone["repositoryId"] = "sha256:foreign".into();
        }),
        ("embedded-repository", |_standalone, embedded| {
            embedded["repositoryId"] = "sha256:foreign".into();
        }),
        ("embedded-work-item", |_standalone, embedded| {
            embedded["workItemId"] = "WI-FOREIGN".into();
        }),
        ("embedded-unknown", |_standalone, embedded| {
            embedded["unexpected"] = true.into();
        }),
        ("standalone-embedded-mismatch", |standalone, _embedded| {
            standalone["retention"]["retentionDays"] = 31.into();
        }),
    ];

    for (name, mutate) in mutations {
        let directory = repository();
        let current = runtime(name);
        start(&directory, "WI-135-RETENTION");
        set_evidence_retention_policy(
            directory.path(),
            "WI-135-RETENTION",
            EvidenceRetention {
                classification: DataClassification::Internal,
                persistence: EvidencePersistence::FullCapture,
                retention_days: Some(30),
                expires_at: None,
                disposal_action: "retain_after_review".into(),
            },
            &current,
        )
        .expect("retention policy");
        record_typed(&directory, "WI-135-RETENTION", &current);

        let standalone_path = directory
            .path()
            .join(".ai/evidence/WI-135-RETENTION.retention.json");
        let evidence_path = directory
            .path()
            .join(".ai/evidence/WI-135-RETENTION.verification.json");
        let mut standalone: Value =
            serde_json::from_slice(&fs::read(&standalone_path).expect("standalone retention"))
                .expect("retention JSON");
        let mut evidence: Value =
            serde_json::from_slice(&fs::read(&evidence_path).expect("evidence")).expect("JSON");
        let mut embedded = evidence["retention"].clone();
        mutate(&mut standalone, &mut embedded);
        if name.starts_with("standalone") {
            fs::write(
                &standalone_path,
                serde_json::to_vec_pretty(&standalone).expect("retention bytes"),
            )
            .expect("tamper standalone retention");
        } else {
            evidence["retention"] = embedded;
            fs::write(
                &evidence_path,
                serde_json::to_vec_pretty(&evidence).expect("evidence bytes"),
            )
            .expect("tamper embedded retention");
        }
        let outcome = outcome_v2_with_runtime(directory.path(), "WI-135-RETENTION", &current)
            .expect("outcome");
        assert_eq!(outcome.decision_state, Some(DecisionState::Red), "{name}");
        assert_ne!(
            outcome.state,
            cockpit_protocol::OutcomeState::Verified,
            "{name}"
        );
        assert!(
            finish_work_item_with_runtime(directory.path(), "WI-135-RETENTION", &current).is_err(),
            "{name} must block finish"
        );
    }
}

#[test]
fn invalid_created_at_blocks_finish_and_archived_close() {
    let directory = repository();
    let current = runtime("timestamp-lifecycle");
    start(&directory, "WI-131-TIMESTAMP-FINISH");
    record_typed(&directory, "WI-131-TIMESTAMP-FINISH", &current);
    let path = directory
        .path()
        .join(".ai/evidence/WI-131-TIMESTAMP-FINISH.verification.json");
    let mut evidence: Value =
        serde_json::from_slice(&fs::read(&path).expect("evidence")).expect("evidence JSON");
    evidence["createdAt"] = "not-an-rfc3339-time".into();
    fs::write(&path, serde_json::to_vec_pretty(&evidence).expect("JSON"))
        .expect("tamper timestamp");
    let outcome = outcome_v2_with_runtime(directory.path(), "WI-131-TIMESTAMP-FINISH", &current)
        .expect("outcome");
    assert_eq!(outcome.decision_state, Some(DecisionState::Red));
    assert!(
        finish_work_item_with_runtime(directory.path(), "WI-131-TIMESTAMP-FINISH", &current)
            .is_err(),
        "invalid timestamp must block finish"
    );

    start(&directory, "WI-131-TIMESTAMP-CLOSE");
    plan(&directory, "WI-131-TIMESTAMP-CLOSE");
    record_typed(&directory, "WI-131-TIMESTAMP-CLOSE", &current);
    finish_work_item_with_runtime(directory.path(), "WI-131-TIMESTAMP-CLOSE", &current)
        .expect("finish");
    archive_work_item_with_runtime(directory.path(), "WI-131-TIMESTAMP-CLOSE", &current)
        .expect("archive");
    let path = directory
        .path()
        .join(".ai/evidence/WI-131-TIMESTAMP-CLOSE.verification.json");
    let mut evidence: Value =
        serde_json::from_slice(&fs::read(&path).expect("evidence")).expect("evidence JSON");
    evidence["createdAt"] = "not-an-rfc3339-time".into();
    fs::write(&path, serde_json::to_vec_pretty(&evidence).expect("JSON"))
        .expect("tamper archived timestamp");
    let close = close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        "WI-131-TIMESTAMP-CLOSE",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "timestamp-regression".into(),
            reason: "invalid evidence must stop close".into(),
            evidence_refs: vec![".ai/evidence/WI-131-TIMESTAMP-CLOSE.verification.json".into()],
            policy_refs: vec!["evidence-assurance".into()],
            decided_at: "2026-08-22T12:00:00Z".into(),
            resume_condition: Some("rerun verification".into()),
        },
        &current,
    );
    assert!(close.is_err(), "invalid timestamp must block close");
}

#[test]
fn current_runtime_lifecycle_rejects_foreign_runtime_evidence() {
    let directory = repository();
    let current = runtime("current");
    let foreign = runtime("foreign");
    start(&directory, "WI-110-RUNTIME");
    plan(&directory, "WI-110-RUNTIME");
    record_typed(&directory, "WI-110-RUNTIME", &current);
    let error = finish_work_item_with_runtime(directory.path(), "WI-110-RUNTIME", &foreign)
        .expect_err("foreign runtime evidence must not finish");
    assert!(error.to_string().contains("valid current receipt"));
    let outcome = outcome_v2_with_runtime(directory.path(), "WI-110-RUNTIME", &foreign)
        .expect("foreign-runtime outcome");
    assert_eq!(outcome.decision_state, Some(DecisionState::Red));
    assert_ne!(outcome.state, cockpit_protocol::OutcomeState::Verified);
}

#[test]
fn archived_foreign_runtime_evidence_is_historical_yellow() {
    let directory = repository();
    let recorded = runtime("recorded");
    let current = runtime("current");
    start(&directory, "WI-110-HISTORICAL-V2");
    plan(&directory, "WI-110-HISTORICAL-V2");
    record_typed(&directory, "WI-110-HISTORICAL-V2", &recorded);
    finish_work_item_with_runtime(directory.path(), "WI-110-HISTORICAL-V2", &recorded)
        .expect("finish");
    archive_work_item_with_runtime(directory.path(), "WI-110-HISTORICAL-V2", &recorded)
        .expect("archive");

    let outcome = outcome_v2_with_runtime(directory.path(), "WI-110-HISTORICAL-V2", &current)
        .expect("historical outcome");
    assert_eq!(outcome.state, cockpit_protocol::OutcomeState::NotReady);
    assert_eq!(outcome.decision_state, Some(DecisionState::Yellow));
    assert_eq!(
        outcome.historical_status.as_deref(),
        Some("runtime_historical")
    );
    assert!(
        outcome
            .unknowns
            .contains(&"historical_evidence_not_revalidated".into())
    );
}

#[test]
fn archived_foreign_runtime_evidence_can_close_without_rewriting_history() {
    let directory = repository();
    let recorded = runtime("recorded-close");
    let current = runtime("upgraded-close");
    start(&directory, "WI-161-HISTORICAL-CLOSE");
    let context = ResourceFinalizationContext {
        branch: "feature/historical-close".into(),
        worktree: "/tmp/removed-historical-close".into(),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: "github".into(),
        pull_request: "https://github.com/example/ai-cockpit/pull/161".into(),
    };
    plan_resource_finalization(directory.path(), "WI-161-HISTORICAL-CLOSE", &context)
        .expect("resource context plan");
    record_typed(&directory, "WI-161-HISTORICAL-CLOSE", &recorded);
    finish_work_item_with_runtime(directory.path(), "WI-161-HISTORICAL-CLOSE", &recorded)
        .expect("finish");
    archive_work_item_with_runtime(directory.path(), "WI-161-HISTORICAL-CLOSE", &recorded)
        .expect("archive");
    // Simulate a later merge on the default branch.  The archived plan
    // snapshot is intentionally older than the repository snapshot observed
    // during close; historical evidence must remain valid and immutable.
    fs::write(
        directory.path().join("post-archive-merge.txt"),
        b"later merge changed the current snapshot\n",
    )
    .expect("post-archive change");

    let contract_path = directory
        .path()
        .join(".ai/work-items/archive/WI-161-HISTORICAL-CLOSE.contract.json");
    let contract_digest = Digest::sha256_bytes(&fs::read(&contract_path).expect("contract"));
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
            .expect("contract JSON");
    let receipt = serde_json::json!({
        "schemaVersion": 1,
        "receiptId": "receipt-historical-close",
        "operationId": "operation-historical-close",
        "repositoryId": cockpit_repository::repository_id(directory.path()).to_string(),
        "workItemId": "WI-161-HISTORICAL-CLOSE",
        "runtimeVersion": current.runtime_version.clone(),
        "runtimeDigest": current.runtime_digest.to_string(),
        "provider": "github",
        "pullRequest": {
            "number": 161,
            "url": context.pull_request,
            "headRevision": "abcdef1",
            "baseBranch": "main",
            "baseRemote": "origin",
            "baseRevision": contract["baseRevision"],
            "mergeCommit": "1234567"
        },
        "branch": {
            "name": context.branch,
            "remote": "origin",
            "headRevision": "abcdef1"
        },
        "worktree": {
            "worktreeId": "removed-historical-close",
            "path": context.worktree,
            "branch": context.branch,
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
        "authoritySource": "historical-runtime-compatibility",
        "reason": "provider cleanup after Runtime upgrade",
        "timestamp": "2026-08-23T01:00:00Z",
        "contractDigest": contract_digest,
        "resourceContext": context
    });
    let receipt_path = directory
        .path()
        .join(".ai/decisions/WI-161-HISTORICAL-CLOSE.receipt.json");
    fs::write(
        &receipt_path,
        serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
    )
    .expect("receipt");
    record_resource_finalization(
        directory.path(),
        "WI-161-HISTORICAL-CLOSE",
        &receipt_path,
        &current,
    )
    .expect("resource finalization");

    let close = close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        "WI-161-HISTORICAL-CLOSE",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "historical-runtime-compatibility".into(),
            reason: "close the already archived item after a Runtime upgrade".into(),
            evidence_refs: vec![".ai/evidence/WI-161-HISTORICAL-CLOSE.verification.json".into()],
            policy_refs: vec!["historical-evidence-policy".into()],
            decided_at: "2026-08-23T01:00:00Z".into(),
            resume_condition: None,
        },
        &current,
    );
    assert!(
        close.is_ok(),
        "historical evidence should not block close: {close:?}"
    );

    let projected = outcome_v2_with_runtime(directory.path(), "WI-161-HISTORICAL-CLOSE", &current)
        .expect("historical outcome");
    assert_eq!(projected.decision_state, Some(DecisionState::Yellow));
    assert_eq!(projected.failed_gate, None);
    assert_eq!(projected.recovery_condition, None);
    let task_report = projected.task_outcome_report.as_ref().expect("task report");
    assert_eq!(task_report.failed_gate, None);
    assert_eq!(task_report.recovery_condition, None);
    assert!(task_report.sections.interventions.is_empty());
    for language in ["zh", "ja", "en"] {
        let handoff = render_human_outcome(directory.path(), &projected, language);
        assert!(
            !handoff.contains("verification_or_human_input"),
            "{language}: {handoff}"
        );
        assert!(
            !handoff.contains("必需的验证证据尚未生成"),
            "{language}: {handoff}"
        );
        assert!(
            !handoff.contains("missing-evidence recovery"),
            "{language}: {handoff}"
        );
        assert!(
            handoff.contains("historical") || handoff.contains("历史") || handoff.contains("履歴"),
            "{language}: {handoff}"
        );
    }

    let bytes = fs::read(
        directory
            .path()
            .join(".ai/evidence/WI-161-HISTORICAL-CLOSE.verification.json"),
    )
    .expect("historical evidence");
    let evidence: Value = serde_json::from_slice(&bytes).expect("evidence JSON");
    assert_eq!(evidence["runtimeVersion"], recorded.runtime_version);
    assert_eq!(
        evidence["runtimeDigest"],
        recorded.runtime_digest.to_string()
    );
}

#[test]
fn legacy_evidence_is_projected_as_historical_yellow_without_rewriting_bytes() {
    let directory = repository();
    let current = runtime("legacy-reader");
    start(&directory, "WI-110-LEGACY");
    let mut evidence = record_typed(&directory, "WI-110-LEGACY", &current);
    evidence
        .as_object_mut()
        .expect("evidence object")
        .remove("evidenceSchemaVersion");
    evidence
        .as_object_mut()
        .expect("evidence object")
        .remove("repositoryId");
    let path = directory
        .path()
        .join(".ai/evidence/WI-110-LEGACY.verification.json");
    let legacy_bytes = serde_json::to_vec_pretty(&evidence).expect("legacy JSON");
    fs::write(&path, &legacy_bytes).expect("write legacy evidence");
    let outcome = outcome_v2_with_runtime(directory.path(), "WI-110-LEGACY", &current)
        .expect("legacy outcome");
    assert_eq!(outcome.state, cockpit_protocol::OutcomeState::NotReady);
    assert_eq!(outcome.decision_state, Some(DecisionState::Yellow));
    assert!(
        outcome
            .unknowns
            .contains(&"legacy_evidence_historical".into())
    );
    assert_eq!(fs::read(&path).expect("legacy bytes"), legacy_bytes);
}
