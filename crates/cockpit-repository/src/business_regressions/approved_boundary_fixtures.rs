//! Explicit synthetic approvals for domain fixtures. No production factory uses this module.
#![cfg(test)]
use crate::review_authority::pipeline_tests::TestAuthority;
use cockpit_protocol::review_authority::HumanReviewOperation;
use cockpit_protocol::{HumanDecision, RuntimeContext};
use std::path::Path;

pub(super) fn close_legacy(
    authority: &TestAuthority,
    root: &Path,
    id: &str,
    decision: &str,
) -> Result<crate::LifecycleReceipt, crate::ObserverError> {
    let decision = HumanDecision {
        decision: decision.trim().into(),
        actor: "legacy-cli".into(),
        authority_source: "explicit-cli".into(),
        reason: "legacy human-decision input; provide structured fields for enterprise assurance"
            .into(),
        evidence_refs: Vec::new(),
        policy_refs: Vec::new(),
        decided_at: crate::now(),
        resume_condition: None,
    };
    close_structured(authority, root, id, &decision)
}
pub(super) fn close_structured(
    authority: &TestAuthority,
    root: &Path,
    id: &str,
    decision: &HumanDecision,
) -> Result<crate::LifecycleReceipt, crate::ObserverError> {
    authority.approve(
        root,
        id,
        HumanReviewOperation::Close,
        &serde_json::json!({"decision":decision,"runtime":None::<RuntimeContext>}),
    );
    crate::close_work_item_with_structured_decision(root, id, decision)
}
pub(super) fn close_structured_runtime(
    authority: &TestAuthority,
    root: &Path,
    id: &str,
    decision: &HumanDecision,
    runtime: &RuntimeContext,
) -> Result<crate::LifecycleReceipt, crate::ObserverError> {
    authority.approve(
        root,
        id,
        HumanReviewOperation::Close,
        &serde_json::json!({"decision":decision,"runtime":runtime}),
    );
    crate::close_work_item_with_structured_decision_and_runtime(root, id, decision, runtime)
}
pub(super) fn archived_verification(
    authority: &TestAuthority,
    root: &Path,
    id: &str,
    request: &crate::ArchivedVerificationRecoveryRequest,
    runtime: &RuntimeContext,
    snapshot: &cockpit_git::RepositorySnapshot,
) -> Result<serde_json::Value, crate::ObserverError> {
    authority.approve(root, id, HumanReviewOperation::ArchivedVerificationRecovery,
        &serde_json::json!({"request":{"reason":request.reason,"actor":request.actor,"authoritySource":request.authority_source,"evidenceRefs":request.evidence_refs,"policyRefs":request.policy_refs,"verificationReceipt":request.verification_receipt},"runtime":runtime,"snapshot":snapshot}));
    crate::record_archived_verification_recovery_with_runtime(root, id, request, runtime, snapshot)
}
pub(super) fn approve_initial_cross_checkout(
    authority: &TestAuthority,
    destination: &Path,
    source: &Path,
    id: &str,
    runtime: &RuntimeContext,
) {
    let source = std::path::absolute(source).unwrap();
    authority.approve(
        destination,
        id,
        HumanReviewOperation::CrossCheckoutRecovery,
        &serde_json::json!({"sourceRoot":source,"runtime":runtime}),
    );
}

pub(super) fn approve_legacy_registration(
    authority: &TestAuthority,
    root: &Path,
    id: &str,
    plan: &cockpit_protocol::LegacyCloseoutSourcePlan,
    runtime: &RuntimeContext,
) {
    authority.approve(
        root,
        id,
        HumanReviewOperation::LegacySourceRegistration,
        &serde_json::json!({"plan":plan,"runtime":runtime}),
    );
}
