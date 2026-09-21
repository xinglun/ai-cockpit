use cockpit_core::Digest;
use cockpit_protocol::{
    ReleaseEvidenceBinding, ReleaseMode, ReleasePlan, ReleasePlanEnvelope, ReleasePlanError,
    ReleaseRequest, ReleaseRequestInput, ReleaseStage,
};

fn normal_request() -> ReleaseRequest {
    ReleaseRequest {
        schema_version: 1,
        repository_id: "repo-1".into(),
        work_item_id: Some("WI-967-release-plan-ssot".into()),
        source_work_item_id: None,
        mode: ReleaseMode::NormalRelease,
        head_revision: "head-sha".into(),
        source_revision: "source-sha".into(),
        base_revision: "base-sha".into(),
        contract_path: Some(".ai/work-items/active/WI-967-release-plan-ssot.contract.json".into()),
        contract_digest: Some(Digest::sha256_bytes(b"contract")),
        from_tag: Some("v0.2.104".into()),
        version: Some("0.2.105".into()),
        tag: Some("v0.2.105".into()),
        handoff_run_id: None,
        reuse_run_id: None,
        reuse_acceptance_run_id: None,
        source_contract_path: Some(
            ".ai/work-items/active/WI-967-release-plan-ssot.contract.json".into(),
        ),
        source_contract_digest: Some(Digest::sha256_bytes(b"contract")),
        recovery_evidence: Vec::new(),
        requested_stage: None,
    }
}

#[test]
fn normal_release_resolves_to_one_canonical_route() {
    let plan = ReleasePlan::resolve(normal_request()).expect("normal plan");
    assert_eq!(plan.mode(), ReleaseMode::NormalRelease);
    assert_eq!(
        plan.allowed_stages(),
        &[
            ReleaseStage::Resolve,
            ReleaseStage::Preflight,
            ReleaseStage::Build,
            ReleaseStage::CandidateAcceptance,
            ReleaseStage::Publish,
            ReleaseStage::PublicAcceptance,
            ReleaseStage::Close,
        ]
    );

    let envelope = ReleasePlanEnvelope::new(plan).expect("envelope");
    envelope.verify().expect("digest is self-consistent");
    let encoded = serde_json::to_vec(&envelope).expect("JSON");
    let decoded: ReleasePlanEnvelope = serde_json::from_slice(&encoded).expect("roundtrip");
    assert_eq!(decoded, envelope);
}

#[test]
fn historical_recovery_requires_immutable_evidence_and_cannot_be_normal() {
    let mut request = normal_request();
    request.mode = ReleaseMode::HistoricalTagRecovery;
    request.recovery_evidence.clear();
    assert!(matches!(
        ReleasePlan::resolve(request),
        Err(ReleasePlanError::MissingRecoveryEvidence)
    ));

    let mut request = normal_request();
    request.mode = ReleaseMode::HistoricalTagRecovery;
    request.recovery_evidence = vec![ReleaseEvidenceBinding {
        class: "archived_recovery".into(),
        path: ".ai/decisions/recovery.json".into(),
        digest: Digest::sha256_bytes(b"recovery"),
    }];
    let plan = ReleasePlan::resolve(request).expect("historical recovery plan");
    assert_eq!(
        plan.allowed_stages(),
        &[
            ReleaseStage::Resolve,
            ReleaseStage::Preflight,
            ReleaseStage::Publish,
            ReleaseStage::PublicAcceptance,
            ReleaseStage::Close,
        ]
    );
}

#[test]
fn mode_specific_fields_are_rejected_instead_of_combined_flags() {
    let mut request = normal_request();
    request.mode = ReleaseMode::CloseOnly;
    request.recovery_evidence = vec![ReleaseEvidenceBinding {
        class: "archived_recovery".into(),
        path: ".ai/decisions/recovery.json".into(),
        digest: Digest::sha256_bytes(b"recovery"),
    }];
    request.reuse_acceptance_run_id = Some("123".into());
    assert!(matches!(
        ReleasePlan::resolve(request),
        Err(ReleasePlanError::ConflictingField(_))
    ));
}

#[test]
fn tampered_plan_digest_is_rejected_and_close_is_a_fixed_point() {
    let plan = ReleasePlan::resolve(normal_request()).expect("normal plan");
    assert_eq!(plan.next_stage(ReleaseStage::Close), None);

    let mut envelope = ReleasePlanEnvelope::new(plan).expect("envelope");
    envelope.plan.request.version = Some("0.2.999".into());
    assert!(matches!(
        envelope.verify(),
        Err(ReleasePlanError::PlanDigestMismatch)
    ));
}

#[test]
fn recovery_modes_bind_their_reusable_run_identity() {
    let mut post = normal_request();
    post.mode = ReleaseMode::PostReleaseAcceptance;
    assert!(matches!(
        ReleasePlan::resolve(post.clone()),
        Err(ReleasePlanError::MissingField("reuse_run_id"))
    ));
    post.reuse_run_id = Some("123".into());
    let post_plan = ReleasePlan::resolve(post).expect("post-release plan");
    assert_eq!(post_plan.mode(), ReleaseMode::PostReleaseAcceptance);

    let mut close = normal_request();
    close.mode = ReleaseMode::CloseOnly;
    assert!(matches!(
        ReleasePlan::resolve(close.clone()),
        Err(ReleasePlanError::MissingField("reuse_acceptance_run_id"))
    ));
    close.reuse_acceptance_run_id = Some("456".into());
    let close_plan = ReleasePlan::resolve(close).expect("close-only plan");
    assert_eq!(
        close_plan.allowed_stages(),
        &[
            ReleaseStage::Resolve,
            ReleaseStage::Preflight,
            ReleaseStage::Close
        ]
    );
}

#[test]
fn plan_generation_is_deterministic_for_the_same_request() {
    let first = ReleasePlanEnvelope::new(ReleasePlan::resolve(normal_request()).unwrap()).unwrap();
    let second = ReleasePlanEnvelope::new(ReleasePlan::resolve(normal_request()).unwrap()).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.plan_digest, second.plan_digest);
}

#[test]
fn raw_dispatch_flags_are_resolved_once_into_disjoint_modes() {
    let input = ReleaseRequestInput {
        schema_version: 1,
        repository_id: "repo-1".into(),
        event: "workflow_dispatch".into(),
        head_revision: "head-sha".into(),
        base_revision: "base-sha".into(),
        source_revision: "source-sha".into(),
        version: Some("0.2.105".into()),
        from_tag: Some("v0.2.104".into()),
        to_tag: Some("v0.2.105".into()),
        publish_existing_tag: true,
        publish_candidate: false,
        post_release_acceptance: false,
        close_only: false,
        work_item_id: Some("WI-967-release-plan-ssot".into()),
        source_work_item_id: None,
        contract_path: None,
        contract_digest: None,
        source_contract_path: None,
        source_contract_digest: None,
        recovery_evidence: vec![ReleaseEvidenceBinding {
            class: "archived_recovery".into(),
            path: ".ai/decisions/recovery.json".into(),
            digest: Digest::sha256_bytes(b"recovery"),
        }],
        reuse_run_id: Some("123".into()),
        reuse_acceptance_run_id: None,
        handoff_run_id: None,
        requested_mode: None,
    };
    let plan = ReleasePlan::resolve_input(input).expect("historical mode");
    assert_eq!(plan.mode(), ReleaseMode::HistoricalTagRecovery);

    let independent = ReleaseRequestInput {
        schema_version: 1,
        repository_id: "repo-1".into(),
        event: "workflow_dispatch".into(),
        head_revision: "head-sha".into(),
        base_revision: "base-sha".into(),
        source_revision: "source-sha".into(),
        version: Some("0.2.105".into()),
        from_tag: Some("v0.2.104".into()),
        to_tag: Some("v0.2.105".into()),
        publish_existing_tag: false,
        publish_candidate: false,
        post_release_acceptance: false,
        close_only: false,
        work_item_id: None,
        source_work_item_id: None,
        contract_path: None,
        contract_digest: None,
        source_contract_path: None,
        source_contract_digest: None,
        recovery_evidence: Vec::new(),
        reuse_run_id: None,
        reuse_acceptance_run_id: None,
        handoff_run_id: Some("456".into()),
        requested_mode: None,
    };
    assert_eq!(
        ReleasePlan::resolve_input(independent.clone())
            .unwrap()
            .mode(),
        ReleaseMode::IndependentPublicAcceptance
    );

    let normal = ReleaseRequestInput {
        publish_existing_tag: false,
        publish_candidate: true,
        post_release_acceptance: false,
        close_only: false,
        work_item_id: Some("WI-967-release-plan-ssot".into()),
        contract_path: Some(".ai/work-items/active/WI-967-release-plan-ssot.contract.json".into()),
        contract_digest: Some(Digest::sha256_bytes(b"contract")),
        source_contract_path: None,
        source_contract_digest: None,
        source_work_item_id: None,
        recovery_evidence: Vec::new(),
        handoff_run_id: None,
        reuse_run_id: None,
        reuse_acceptance_run_id: None,
        requested_mode: None,
        ..independent
    };
    assert_eq!(
        ReleasePlan::resolve_input(normal).unwrap().mode(),
        ReleaseMode::NormalRelease
    );
}
