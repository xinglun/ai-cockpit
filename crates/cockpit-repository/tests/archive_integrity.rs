use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::{
    AssuranceLevel, DataClassification, DelegatedEvidence, EvidencePersistence, EvidenceRetention,
    EvidenceValidity, HumanDecision, RuntimeContext,
};
use cockpit_repository::{
    WorkItemStartOptions, archive_work_item, attach, checkpoint_work_item,
    close_work_item_with_decision, close_work_item_with_structured_decision, evidence_purge_plan,
    evidence_state_for_contract, export_audit_events, finish_work_item,
    governance_decision_for_contract, import_delegated_evidence, preflight_work_item,
    record_verification, set_evidence_retention_policy, start_work_item,
    start_work_item_with_options,
};
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

fn repository() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "cockpit-archive-integrity-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&path)
        .status()
        .expect("git init");
    attach(&path).expect("attach");
    path
}

fn prepare_for_verification(path: &std::path::Path, work_item_id: &str) {
    let contract = path
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let decision = preflight_work_item(path, &contract).expect("preflight");
    assert_ne!(decision.state, cockpit_core::DecisionState::Red);
    checkpoint_work_item(path, work_item_id).expect("checkpoint");
}

#[test]
fn close_rejects_tampered_archived_artifacts() {
    let path = repository();
    start_work_item(&path, "WI-INTEGRITY", "integrity", "verify", &["**".into()]).expect("start");
    prepare_for_verification(&path, "WI-INTEGRITY");
    record_verification(
        &path,
        "WI-INTEGRITY",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-INTEGRITY").expect("finish");
    archive_work_item(&path, "WI-INTEGRITY").expect("archive");
    fs::write(
        path.join(".ai/work-items/archive/WI-INTEGRITY.outcome.json"),
        br#"{"tampered":true}"#,
    )
    .expect("tamper");
    let error = close_work_item_with_decision(&path, "WI-INTEGRITY", "approved")
        .expect_err("tampered archive must be rejected");
    assert!(error.to_string().contains("digest does not match manifest"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn archive_rejects_tampered_verification_even_without_declared_requirement() {
    let path = repository();
    start_work_item(
        &path,
        "WI-EVIDENCE-TAMPER",
        "tamper",
        "verify",
        &["**".into()],
    )
    .expect("start");
    prepare_for_verification(&path, "WI-EVIDENCE-TAMPER");
    record_verification(
        &path,
        "WI-EVIDENCE-TAMPER",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-EVIDENCE-TAMPER").expect("finish");
    let evidence_path = path.join(".ai/evidence/WI-EVIDENCE-TAMPER.verification.json");
    let mut evidence: serde_json::Value =
        serde_json::from_slice(&fs::read(&evidence_path).expect("evidence"))
            .expect("evidence JSON");
    evidence["passed"] = serde_json::Value::Bool(false);
    fs::write(
        &evidence_path,
        serde_json::to_vec_pretty(&evidence).expect("tampered evidence"),
    )
    .expect("write tampered evidence");
    let error = archive_work_item(&path, "WI-EVIDENCE-TAMPER")
        .expect_err("archive must fail closed on tampered evidence");
    assert!(error.to_string().contains("valid verification evidence"));
    assert!(
        path.join(".ai/work-items/active/WI-EVIDENCE-TAMPER.contract.json")
            .is_file()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn verification_receipt_cannot_cross_work_items() {
    let path = repository();
    start_work_item(&path, "WI-A", "first", "verify", &["**".into()]).expect("start A");
    start_work_item(&path, "WI-B", "second", "verify", &["**".into()]).expect("start B");
    prepare_for_verification(&path, "WI-A");
    prepare_for_verification(&path, "WI-B");
    let error = record_verification(
        &path,
        "WI-B",
        &serde_json::json!({"passed": true, "workItemId": "WI-A", "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect_err("cross-work-item evidence must be rejected");
    assert!(error.to_string().contains("another work item"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn close_persists_a_structured_human_decision_and_recovery_condition() {
    let path = repository();
    start_work_item(&path, "WI-DECISION", "decision", "verify", &["**".into()]).expect("start");
    prepare_for_verification(&path, "WI-DECISION");
    record_verification(
        &path,
        "WI-DECISION",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    let evidence: serde_json::Value = serde_json::from_slice(
        &fs::read(path.join(".ai/evidence/WI-DECISION.verification.json")).expect("evidence"),
    )
    .expect("evidence JSON");
    assert_eq!(evidence["evidenceSchemaVersion"], 2);
    assert_eq!(
        evidence["repositoryId"],
        cockpit_repository::repository_id(&path).to_string()
    );
    finish_work_item(&path, "WI-DECISION").expect("finish");
    archive_work_item(&path, "WI-DECISION").expect("archive");
    close_work_item_with_structured_decision(
        &path,
        "WI-DECISION",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "team-policy".into(),
            reason: "bounded change and fresh evidence".into(),
            evidence_refs: vec![".ai/evidence/WI-DECISION.verification.json".into()],
            policy_refs: vec!["team-policy-v1".into()],
            decided_at: "2026-08-21T19:00:00Z".into(),
            resume_condition: Some("rerun verification if the base changes".into()),
        },
    )
    .expect("structured close");
    let decision: serde_json::Value = serde_json::from_slice(
        &fs::read(path.join(".ai/decisions/WI-DECISION.close.json")).expect("decision"),
    )
    .expect("decision JSON");
    assert_eq!(
        decision["repositoryId"],
        cockpit_repository::repository_id(&path).to_string()
    );
    assert_eq!(decision["structuredDecision"]["actor"], "human:owner");
    assert_eq!(
        decision["structuredDecision"]["resumeCondition"],
        "rerun verification if the base changes"
    );
    assert_eq!(decision["finalReport"]["format"], "ai-cockpit.task-outcome");
    assert!(
        !decision["finalReport"]["sections"]["humanDecisions"]
            .as_array()
            .expect("human decision projections")
            .is_empty()
    );
    assert!(decision["finalReportDigest"].as_str().is_some());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn close_accepts_immutable_archived_evidence_after_a_post_archive_commit() {
    let path = repository();
    start_work_item(
        &path,
        "WI-ARCHIVE-MERGE",
        "archive",
        "close",
        &["**".into()],
    )
    .expect("start");
    prepare_for_verification(&path, "WI-ARCHIVE-MERGE");
    record_verification(
        &path,
        "WI-ARCHIVE-MERGE",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.9",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-ARCHIVE-MERGE").expect("finish");
    archive_work_item(&path, "WI-ARCHIVE-MERGE").expect("archive");

    fs::write(
        path.join("post-archive-change.txt"),
        b"merged release change\n",
    )
    .expect("post-archive change");
    assert!(
        Command::new("git")
            .args(["add", "post-archive-change.txt"])
            .current_dir(&path)
            .status()
            .expect("git add")
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit-test@example.invalid",
                "commit",
                "-m",
                "post-archive merge",
            ])
            .current_dir(&path)
            .status()
            .expect("git commit")
            .success()
    );

    close_work_item_with_structured_decision(
        &path,
        "WI-ARCHIVE-MERGE",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "team-policy".into(),
            reason: "archive was reviewed before the merge commit".into(),
            evidence_refs: vec![".ai/evidence/WI-ARCHIVE-MERGE.verification.json".into()],
            policy_refs: Vec::new(),
            decided_at: "2026-08-22T06:00:00Z".into(),
            resume_condition: None,
        },
    )
    .expect("close after merge must use immutable archived evidence");
    assert!(
        path.join(".ai/decisions/WI-ARCHIVE-MERGE.close.json")
            .is_file()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn organization_policy_requires_a_bound_structured_decision_at_close() {
    let path = repository();
    fs::write(
        path.join(".ai/policy.json"),
        r#"{
          "schemaVersion": 1,
          "organization": {
            "policyId": "org-release-v1",
            "layer": "organization",
            "rules": [{
              "operation": "modify_source",
              "approvalMode": "single_authorized_human",
              "requiredEvidence": ["delegated:github"]
            }]
          }
        }"#,
    )
    .expect("policy");
    start_work_item_with_options(
        &path,
        "WI-POLICY",
        "policy",
        "verify",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            required_evidence_classes: vec!["verification".into(), "delegated:github".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    prepare_for_verification(&path, "WI-POLICY");
    let raw = br#"{"run":999}"#;
    import_delegated_evidence(
        &path,
        "WI-POLICY",
        &DelegatedEvidence {
            provider: "github".into(),
            subject: "run:999".into(),
            origin: "https://github.com/example/repo/actions/runs/999".into(),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: "2026-08-21T19:00:00Z".into(),
            digest: cockpit_core::Digest::sha256_bytes(raw),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/github-run-999.json".into(),
        },
        raw,
        &RuntimeContext {
            runtime_version: "0.2.2".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"runtime"),
        },
    )
    .expect("delegated evidence");
    record_verification(
        &path,
        "WI-POLICY",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.1",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-POLICY").expect("finish");
    archive_work_item(&path, "WI-POLICY").expect("archive");

    let missing_binding = close_work_item_with_structured_decision(
        &path,
        "WI-POLICY",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "team-policy".into(),
            reason: "fresh evidence".into(),
            evidence_refs: vec![".ai/evidence/WI-POLICY.verification.json".into()],
            policy_refs: Vec::new(),
            decided_at: "2026-08-21T19:00:00Z".into(),
            resume_condition: None,
        },
    )
    .expect_err("policy close must bind the policy reference");
    assert!(missing_binding.to_string().contains("policy"));

    close_work_item_with_structured_decision(
        &path,
        "WI-POLICY",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "team-policy".into(),
            reason: "fresh evidence".into(),
            evidence_refs: vec![".ai/evidence/WI-POLICY.verification.json".into()],
            policy_refs: vec!["org-release-v1".into()],
            decided_at: "2026-08-21T19:00:00Z".into(),
            resume_condition: None,
        },
    )
    .expect("bound policy close");
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn preflight_exposes_policy_authority_and_evidence_gaps() {
    let path = repository();
    fs::write(
        path.join(".ai/policy.json"),
        r#"{
          "schemaVersion": 1,
          "organization": {
            "policyId": "org-production-v1",
            "layer": "organization",
            "rules": [{
              "operation": "modify_source",
              "approvalMode": "single_authorized_human",
              "requiredEvidence": ["hosted_ci"]
            }]
          }
        }"#,
    )
    .expect("policy");
    start_work_item(
        &path,
        "WI-PREFLIGHT-POLICY",
        "policy",
        "verify",
        &["**".into()],
    )
    .expect("start");
    let contract: cockpit_protocol::Contract = serde_json::from_slice(
        &fs::read(path.join(".ai/work-items/active/WI-PREFLIGHT-POLICY.contract.json"))
            .expect("contract"),
    )
    .expect("parse contract");
    let snapshot = GitRepository::discover(&path)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let decision = governance_decision_for_contract(&path, &contract, &snapshot).expect("decision");
    assert_eq!(decision.state, cockpit_core::DecisionState::Yellow);
    assert!(
        decision
            .unknowns
            .contains(&"human_authority_missing".into())
    );
    assert!(
        decision
            .unknowns
            .contains(&"policy_required_evidence_missing".into())
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn digest_only_retention_never_persists_command_output() {
    let path = repository();
    start_work_item(&path, "WI-DIGEST", "digest", "verify", &["**".into()]).expect("start");
    prepare_for_verification(&path, "WI-DIGEST");
    set_evidence_retention_policy(
        &path,
        "WI-DIGEST",
        EvidenceRetention {
            classification: DataClassification::Restricted,
            persistence: EvidencePersistence::DigestOnly,
            retention_days: Some(30),
            expires_at: None,
            disposal_action: "purge_after_expiry".into(),
        },
        &RuntimeContext {
            runtime_version: "0.2.2".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"runtime"),
        },
    )
    .expect("retention policy");
    let evidence = record_verification(
        &path,
        "WI-DIGEST",
        &serde_json::json!({
            "passed": true,
            "nodesPlanned": 1,
            "output": "credential=do-not-persist"
        }),
        "0.2.2",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    assert_eq!(evidence["captureMode"], "digest_only");
    assert!(evidence.get("receipt").is_none());
    let bytes = fs::read_to_string(path.join(".ai/evidence/WI-DIGEST.verification.json"))
        .expect("stored evidence");
    assert!(!bytes.contains("do-not-persist"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn no_persistence_fails_closed_instead_of_claiming_completion() {
    let path = repository();
    start_work_item(
        &path,
        "WI-NOPERSIST",
        "no persist",
        "verify",
        &["**".into()],
    )
    .expect("start");
    prepare_for_verification(&path, "WI-NOPERSIST");
    set_evidence_retention_policy(
        &path,
        "WI-NOPERSIST",
        EvidenceRetention {
            classification: DataClassification::SecretProhibited,
            persistence: EvidencePersistence::NoPersistence,
            retention_days: Some(1),
            expires_at: None,
            disposal_action: "external_owner".into(),
        },
        &RuntimeContext {
            runtime_version: "0.2.2".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"runtime"),
        },
    )
    .expect("retention policy");
    let error = record_verification(
        &path,
        "WI-NOPERSIST",
        &serde_json::json!({"passed": true}),
        "0.2.2",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect_err("completion evidence must not be silently discarded");
    assert!(error.to_string().contains("no_persistence"));
    assert!(
        !path
            .join(".ai/evidence/WI-NOPERSIST.verification.json")
            .exists()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn purge_plan_is_deterministic_and_never_deletes_evidence() {
    let path = repository();
    start_work_item(&path, "WI-EXPIRE", "expiry", "verify", &["**".into()]).expect("start");
    prepare_for_verification(&path, "WI-EXPIRE");
    set_evidence_retention_policy(
        &path,
        "WI-EXPIRE",
        EvidenceRetention {
            classification: DataClassification::Confidential,
            persistence: EvidencePersistence::DigestOnly,
            retention_days: None,
            expires_at: Some("0".into()),
            disposal_action: "purge_after_review".into(),
        },
        &RuntimeContext {
            runtime_version: "0.2.2".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"runtime"),
        },
    )
    .expect("retention policy");
    record_verification(
        &path,
        "WI-EXPIRE",
        &serde_json::json!({"passed": true}),
        "0.2.2",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    let first = evidence_purge_plan(&path, 1_700_000_000).expect("purge plan");
    let second = evidence_purge_plan(&path, 1_700_000_000).expect("purge plan");
    assert_eq!(first, second);
    assert_eq!(
        first[0].disposition,
        cockpit_protocol::EvidenceDisposition::PurgePlanned
    );
    assert!(
        path.join(".ai/evidence/WI-EXPIRE.verification.json")
            .is_file()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn audit_export_is_deterministic_and_marks_external_retention_boundary() {
    let path = repository();
    start_work_item(&path, "WI-AUDIT", "audit", "export", &["**".into()]).expect("start");
    prepare_for_verification(&path, "WI-AUDIT");
    record_verification(
        &path,
        "WI-AUDIT",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.2",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    let runtime = RuntimeContext {
        runtime_version: "0.2.2".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime"),
    };
    let first = export_audit_events(&path, &runtime).expect("audit export");
    let second = export_audit_events(&path, &runtime).expect("audit export");
    assert_eq!(first, second);
    assert!(first.external_retention_required);
    assert_eq!(first.events.len(), 1);
    assert_eq!(first.events[0].runtime_version, "0.2.2");
    assert_eq!(first.events[0].work_item_id.as_deref(), Some("WI-AUDIT"));
    assert!(first.events[0].event_id.starts_with("sha256:"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn delegated_evidence_import_binds_raw_digest_and_work_item() {
    let path = repository();
    start_work_item(
        &path,
        "WI-EXTERNAL",
        "external",
        "bind evidence",
        &["**".into()],
    )
    .expect("start");
    let raw = br#"{"provider":"github","run":123}"#;
    let evidence = DelegatedEvidence {
        provider: "github".into(),
        subject: "run:123".into(),
        origin: "https://github.com/example/repo/actions/runs/123".into(),
        assurance: AssuranceLevel::ProviderVerified,
        collected_at: "2026-08-21T19:00:00Z".into(),
        digest: cockpit_core::Digest::sha256_bytes(raw),
        validity: EvidenceValidity::Valid,
        raw_evidence_ref: ".ai/evidence/external/github-run-123.json".into(),
    };
    let runtime = RuntimeContext {
        runtime_version: "0.2.2".into(),
        protocol_version: 1,
        runtime_digest: cockpit_core::Digest::sha256_bytes(b"runtime"),
    };
    let receipt =
        import_delegated_evidence(&path, "WI-EXTERNAL", &evidence, raw, &runtime).expect("import");
    assert_eq!(
        receipt.repository_id,
        cockpit_repository::repository_id(&path).to_string()
    );
    assert_eq!(receipt.work_item_id, "WI-EXTERNAL");
    assert!(
        path.join(".ai/evidence/external/github-run-123.json")
            .is_file()
    );
    assert_eq!(
        fs::read_dir(path.join(".ai/evidence/external"))
            .expect("external evidence")
            .count(),
        2,
        "raw evidence and its binding receipt are both archived"
    );

    let mismatch = DelegatedEvidence {
        digest: cockpit_core::Digest::sha256_bytes(b"different"),
        ..evidence.clone()
    };
    let error = import_delegated_evidence(&path, "WI-EXTERNAL", &mismatch, raw, &runtime)
        .expect_err("digest mismatch must fail closed");
    assert!(error.to_string().contains("digest"));

    let unsafe_ref = DelegatedEvidence {
        raw_evidence_ref: ".ai/evidence/external/../escape.json".into(),
        ..evidence
    };
    let error = import_delegated_evidence(&path, "WI-EXTERNAL", &unsafe_ref, raw, &runtime)
        .expect_err("path escape must fail closed");
    assert!(error.to_string().contains("external"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn valid_delegated_evidence_satisfies_a_provider_specific_contract_requirement() {
    let path = repository();
    start_work_item_with_options(
        &path,
        "WI-DELEGATED-REQUIRED",
        "external",
        "require provider evidence",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            required_evidence_classes: vec!["delegated:github".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    let raw = br#"{"run":456}"#;
    import_delegated_evidence(
        &path,
        "WI-DELEGATED-REQUIRED",
        &DelegatedEvidence {
            provider: "github".into(),
            subject: "run:456".into(),
            origin: "https://github.com/example/repo/actions/runs/456".into(),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: "2026-08-21T19:00:00Z".into(),
            digest: cockpit_core::Digest::sha256_bytes(raw),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/github-run-456.json".into(),
        },
        raw,
        &RuntimeContext {
            runtime_version: "0.2.2".into(),
            protocol_version: 1,
            runtime_digest: cockpit_core::Digest::sha256_bytes(b"runtime"),
        },
    )
    .expect("import");
    let contract: cockpit_protocol::Contract = serde_json::from_slice(
        &fs::read(path.join(".ai/work-items/active/WI-DELEGATED-REQUIRED.contract.json"))
            .expect("contract"),
    )
    .expect("parse contract");
    let snapshot = GitRepository::discover(&path)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    assert_eq!(
        evidence_state_for_contract(&path, &contract, &snapshot).expect("evidence state"),
        cockpit_core::EvidenceState::Complete
    );
    fs::remove_dir_all(path).expect("cleanup");
}
