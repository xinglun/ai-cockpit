use cockpit_core::Digest;
use cockpit_protocol::{
    TaskAuthorizationBatchBudget, TaskAuthorizationBatchDecision, TaskAuthorizationBatchManifest,
    TaskAuthorizationBatchMember, TaskAuthorizationBatchReceipt, TaskAuthorizationBatchReview,
    TaskAuthorizationTarget,
};
use std::collections::BTreeMap;

fn digest(value: &str) -> Digest {
    Digest::sha256_bytes(value.as_bytes())
}

fn member(id: &str, action: &str) -> TaskAuthorizationBatchMember {
    TaskAuthorizationBatchMember {
        work_item_id: id.into(),
        contract_digest: digest(&format!("contract:{id}")),
        snapshot_digest: digest(&format!("snapshot:{id}")),
        scope_digest: digest(&format!("scope:{id}")),
        parent_grant_id: digest(&format!("parent:{id}")),
        actor: format!("agent:{id}"),
        authority_source: format!("conversation:{id}"),
        expires_at: "2026-11-03T12:00:00Z".into(),
        actions: vec![action.into()],
        public_target: (action == "create_pull_request").then(|| TaskAuthorizationTarget {
            provider: "github".into(),
            repository: "example/repo".into(),
            base_branch: "release-candidate".into(),
        }),
        budget: TaskAuthorizationBatchBudget {
            start_quota: 1,
            retry_budget_by_check: BTreeMap::from([("cargo test --locked -p crate".into(), 2)]),
        },
    }
}

fn review(manifest: &TaskAuthorizationBatchManifest) -> TaskAuthorizationBatchReview {
    TaskAuthorizationBatchReview {
        schema_version: 1,
        repository_id: manifest.repository_id.clone(),
        manifest_digest: manifest.manifest_digest.clone(),
        decision: TaskAuthorizationBatchDecision::ConfirmReview,
        reviewer_actor: "human:owner".into(),
        authority_source: "conversation:owner-confirmed-exact-manifest".into(),
        recorded_at: "2026-10-27T12:00:00Z".into(),
        reason: "Review exact explicitly selected members".into(),
    }
}

#[test]
fn exact_member_set_has_a_stable_digest_and_independent_child_grants() {
    let first = TaskAuthorizationBatchManifest::new(
        "sha256:repo",
        vec![
            member("WI-B", "create_pull_request"),
            member("WI-A", "modify_source"),
        ],
    )
    .expect("manifest");
    let reordered = TaskAuthorizationBatchManifest::new(
        "sha256:repo",
        vec![
            member("WI-A", "modify_source"),
            member("WI-B", "create_pull_request"),
        ],
    )
    .expect("same canonical manifest");
    assert_eq!(first, reordered);
    assert_eq!(first.members[0].work_item_id, "WI-A");

    let grants = first
        .derive_child_grant_candidates(&review(&first))
        .expect("exact review claim derives separate candidates");
    assert_eq!(grants.len(), 2);
    assert_ne!(grants[0].grant_id, grants[1].grant_id);
    assert_ne!(
        grants[0].member.parent_grant_id,
        grants[1].member.parent_grant_id
    );
    assert_eq!(grants[0].member.actions, ["modify_source"]);
    assert_eq!(grants[1].member.actions, ["create_pull_request"]);
    assert!(
        grants
            .iter()
            .all(|grant| grant.manifest_digest == first.manifest_digest)
    );
}

#[test]
fn review_for_another_manifest_cannot_issue_child_grants() {
    let manifest =
        TaskAuthorizationBatchManifest::new("sha256:repo", vec![member("WI-A", "modify_source")])
            .expect("manifest");
    let mut wrong = review(&manifest);
    wrong.manifest_digest = digest("different manifest");
    assert!(manifest.derive_child_grant_candidates(&wrong).is_err());

    let mut rejected = review(&manifest);
    rejected.decision = TaskAuthorizationBatchDecision::Reject;
    assert!(manifest.derive_child_grant_candidates(&rejected).is_err());
}

#[test]
fn immutable_batch_receipt_binds_review_runtime_and_all_child_grants() {
    let manifest = TaskAuthorizationBatchManifest::new(
        "sha256:repo",
        vec![
            member("WI-A", "modify_source"),
            member("WI-B", "run_verification"),
        ],
    )
    .expect("manifest");
    let receipt = TaskAuthorizationBatchReceipt::new(
        manifest,
        review(
            &TaskAuthorizationBatchManifest::new(
                "sha256:repo",
                vec![
                    member("WI-B", "run_verification"),
                    member("WI-A", "modify_source"),
                ],
            )
            .expect("same manifest"),
        ),
        "1.0.1",
        digest("fixed runtime"),
        "2026-10-27T00:00:00Z",
    )
    .expect("exact review receipt");
    receipt
        .validate_integrity()
        .expect("valid immutable receipt");
    assert_eq!(receipt.child_grants.len(), 2);
    assert_eq!(receipt.child_grants[0].work_item_id, "WI-A");
    assert_eq!(receipt.child_grants[1].work_item_id, "WI-B");
    assert!(receipt.recorded_by.starts_with("runtime:1.0.1:sha256:"));

    let mut changed = receipt.clone();
    changed.child_grants[0].member.actions = vec!["merge_pull_request".into()];
    assert!(changed.validate_integrity().is_err());
}

#[test]
fn manifest_rejects_duplicates_invalid_actions_and_unbounded_retry_budgets() {
    assert!(
        TaskAuthorizationBatchManifest::new(
            "sha256:repo",
            vec![
                member("WI-A", "modify_source"),
                member("WI-A", "modify_source")
            ],
        )
        .is_err()
    );

    let mut unknown = member("WI-A", "arbitrary_command");
    assert!(TaskAuthorizationBatchManifest::new("sha256:repo", vec![unknown.clone()]).is_err());

    unknown.actions = vec!["modify_source".into()];
    unknown
        .budget
        .retry_budget_by_check
        .insert("cargo test".into(), 3);
    assert!(TaskAuthorizationBatchManifest::new("sha256:repo", vec![unknown.clone()]).is_err());

    unknown.budget.retry_budget_by_check.clear();
    unknown.budget.start_quota = 0;
    assert!(TaskAuthorizationBatchManifest::new("sha256:repo", vec![unknown]).is_err());
}

#[test]
fn strict_manifest_types_reject_unknown_fields() {
    let input = serde_json::json!({
        "startQuota": 1,
        "retryBudgetByCheck": {},
        "sharedAuthority": true
    });
    assert!(serde_json::from_value::<TaskAuthorizationBatchBudget>(input).is_err());
}
