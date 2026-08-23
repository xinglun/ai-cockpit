use cockpit_core::Digest;
use cockpit_protocol::{
    RESOURCE_FINALIZATION_CODE_AMBIGUOUS_STATE, RESOURCE_FINALIZATION_CODE_DIRTY_WORKTREE,
    RESOURCE_FINALIZATION_CODE_PROTECTED_BRANCH, RESOURCE_FINALIZATION_CODE_UNMERGED_PULL_REQUEST,
    ResourceFinalizationBranchIdentity, ResourceFinalizationBranchState,
    ResourceFinalizationContext, ResourceFinalizationDisposition, ResourceFinalizationError,
    ResourceFinalizationPullRequestIdentity, ResourceFinalizationPullRequestState,
    ResourceFinalizationReceipt, ResourceFinalizationResult, ResourceFinalizationState,
    ResourceFinalizationWorktreeIdentity, ResourceFinalizationWorktreeState,
    validate_resource_finalization_receipt, validate_resource_finalization_receipt_for,
    validate_resource_finalization_replay,
};

const REPOSITORY_ID: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const WORK_ITEM_ID: &str = "WI-158-resource-finalization-protocol";

fn receipt() -> ResourceFinalizationReceipt {
    ResourceFinalizationReceipt {
        schema_version: 1,
        receipt_id: "receipt-1".into(),
        operation_id: "operation-1".into(),
        repository_id: REPOSITORY_ID.into(),
        work_item_id: WORK_ITEM_ID.into(),
        runtime_version: "0.2.17".into(),
        runtime_digest: Digest::sha256_bytes(b"runtime"),
        provider: "github".into(),
        pull_request: ResourceFinalizationPullRequestIdentity {
            number: 158,
            url: "https://github.example/acme/project/pull/158".into(),
            head_revision: "head-158".into(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            base_revision: "base-785112b".into(),
            merge_commit: Some("merge-158".into()),
        },
        branch: ResourceFinalizationBranchIdentity {
            name: "codex/wi-158-resource-finalization-protocol".into(),
            remote: "origin".into(),
            head_revision: "head-158".into(),
        },
        worktree: ResourceFinalizationWorktreeIdentity {
            worktree_id: "worktree-158".into(),
            path: "/private/tmp/ai-cockpit-wi158-resource-finalization-protocol".into(),
            branch: "codex/wi-158-resource-finalization-protocol".into(),
            head_revision: "head-158".into(),
        },
        before: ResourceFinalizationState {
            pull_request: ResourceFinalizationPullRequestState::Merged,
            branch: ResourceFinalizationBranchState::Present,
            worktree: ResourceFinalizationWorktreeState::Clean,
        },
        after: ResourceFinalizationState {
            pull_request: ResourceFinalizationPullRequestState::Merged,
            branch: ResourceFinalizationBranchState::Deleted,
            worktree: ResourceFinalizationWorktreeState::Removed,
        },
        result: ResourceFinalizationResult {
            disposition: ResourceFinalizationDisposition::Deleted,
            failure_codes: vec![],
            unknown_codes: vec![],
        },
        actor: "human:maintainer".into(),
        authority_source: "repository-policy:merge-cleanup".into(),
        reason: "reviewed merge and exact resource cleanup".into(),
        timestamp: "2026-08-23T00:00:00Z".into(),
        contract_digest: Some(Digest::sha256_bytes(b"contract")),
        resource_context: Some(ResourceFinalizationContext {
            branch: "codex/wi-158-resource-finalization-protocol".into(),
            worktree: "/private/tmp/ai-cockpit-wi158-resource-finalization-protocol".into(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: "https://github.example/acme/project/pull/158".into(),
        }),
    }
}

#[test]
fn valid_receipt_round_trips_and_binds_contract_context() {
    let receipt = receipt();
    validate_resource_finalization_receipt(&receipt).unwrap();
    let value = serde_json::to_value(&receipt).unwrap();
    let parsed: ResourceFinalizationReceipt = serde_json::from_value(value).unwrap();
    assert_eq!(parsed, receipt);
    validate_resource_finalization_receipt_for(
        &parsed,
        REPOSITORY_ID,
        WORK_ITEM_ID,
        receipt.contract_digest.as_ref(),
        receipt.resource_context.as_ref(),
    )
    .unwrap();
}

#[test]
fn unknown_top_level_and_nested_fields_fail_closed() {
    let mut value = serde_json::to_value(receipt()).unwrap();
    value["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ResourceFinalizationReceipt>(value).is_err());

    let mut value = serde_json::to_value(receipt()).unwrap();
    value["pullRequest"]["providerToken"] = serde_json::json!("secret");
    assert!(serde_json::from_value::<ResourceFinalizationReceipt>(value).is_err());

    let mut value = serde_json::to_value(receipt()).unwrap();
    value["result"]["providerResult"] = serde_json::json!("must not pass");
    assert!(serde_json::from_value::<ResourceFinalizationReceipt>(value).is_err());
}

#[test]
fn empty_and_invalid_identity_fields_are_rejected() {
    let mut empty = receipt();
    empty.provider.clear();
    assert_eq!(
        validate_resource_finalization_receipt(&empty),
        Err(ResourceFinalizationError::EmptyField("provider"))
    );

    let mut invalid_digest = receipt();
    invalid_digest.runtime_digest = serde_json::from_value(serde_json::json!("not-a-digest"))
        .expect("Digest deserialization accepts data that validation rejects");
    assert_eq!(
        validate_resource_finalization_receipt(&invalid_digest),
        Err(ResourceFinalizationError::InvalidDigest("runtimeDigest"))
    );

    let mut invalid_code = receipt();
    invalid_code.result.disposition = ResourceFinalizationDisposition::Blocked;
    invalid_code.result.failure_codes = vec!["Dirty Worktree".into()];
    assert!(matches!(
        validate_resource_finalization_receipt(&invalid_code),
        Err(ResourceFinalizationError::InvalidCode(_))
    ));
}

#[test]
fn foreign_repository_and_work_item_are_rejected() {
    let receipt = receipt();
    assert_eq!(
        validate_resource_finalization_receipt_for(
            &receipt,
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            WORK_ITEM_ID,
            None,
            None,
        ),
        Err(ResourceFinalizationError::IdentityMismatch("repositoryId"))
    );
    assert_eq!(
        validate_resource_finalization_receipt_for(
            &receipt,
            REPOSITORY_ID,
            "WI-foreign",
            None,
            None,
        ),
        Err(ResourceFinalizationError::IdentityMismatch("workItemId"))
    );

    let mut context = receipt.resource_context.clone().unwrap();
    context.branch = "codex/foreign".into();
    let mut foreign_context = receipt;
    foreign_context.resource_context = Some(context);
    assert_eq!(
        validate_resource_finalization_receipt(&foreign_context),
        Err(ResourceFinalizationError::IdentityMismatch(
            "resource context does not match receipt identity",
        ))
    );
}

#[test]
fn dirty_unmerged_protected_and_ambiguous_states_cannot_be_deleted() {
    let mut dirty = receipt();
    dirty.before.worktree = ResourceFinalizationWorktreeState::Dirty;
    dirty.after.worktree = ResourceFinalizationWorktreeState::Dirty;
    dirty.after.branch = ResourceFinalizationBranchState::Present;
    dirty.result.disposition = ResourceFinalizationDisposition::Blocked;
    dirty.result.failure_codes = vec![RESOURCE_FINALIZATION_CODE_DIRTY_WORKTREE.into()];
    validate_resource_finalization_receipt(&dirty).unwrap();
    dirty.result.disposition = ResourceFinalizationDisposition::Deleted;
    dirty.result.failure_codes.clear();
    assert!(validate_resource_finalization_receipt(&dirty).is_err());

    let mut unmerged = receipt();
    unmerged.before.pull_request = ResourceFinalizationPullRequestState::Unmerged;
    unmerged.after.pull_request = ResourceFinalizationPullRequestState::Unmerged;
    unmerged.after.branch = ResourceFinalizationBranchState::Present;
    unmerged.pull_request.merge_commit = None;
    unmerged.result.disposition = ResourceFinalizationDisposition::Blocked;
    unmerged.result.failure_codes = vec![RESOURCE_FINALIZATION_CODE_UNMERGED_PULL_REQUEST.into()];
    validate_resource_finalization_receipt(&unmerged).unwrap();
    unmerged.result.disposition = ResourceFinalizationDisposition::Deleted;
    unmerged.result.failure_codes.clear();
    assert!(validate_resource_finalization_receipt(&unmerged).is_err());

    let mut protected = receipt();
    protected.before.branch = ResourceFinalizationBranchState::Protected;
    protected.after.branch = ResourceFinalizationBranchState::Protected;
    protected.result.disposition = ResourceFinalizationDisposition::Blocked;
    protected.result.failure_codes = vec![RESOURCE_FINALIZATION_CODE_PROTECTED_BRANCH.into()];
    validate_resource_finalization_receipt(&protected).unwrap();
    protected.result.disposition = ResourceFinalizationDisposition::Deleted;
    protected.result.failure_codes.clear();
    assert!(validate_resource_finalization_receipt(&protected).is_err());

    let mut ambiguous = receipt();
    ambiguous.after.branch = ResourceFinalizationBranchState::Unknown;
    ambiguous.after.worktree = ResourceFinalizationWorktreeState::Unknown;
    ambiguous.result.disposition = ResourceFinalizationDisposition::Unknown;
    ambiguous.result.unknown_codes = vec![RESOURCE_FINALIZATION_CODE_AMBIGUOUS_STATE.into()];
    validate_resource_finalization_receipt(&ambiguous).unwrap();
    ambiguous.result.disposition = ResourceFinalizationDisposition::Deleted;
    ambiguous.result.unknown_codes.clear();
    assert!(validate_resource_finalization_receipt(&ambiguous).is_err());
}

#[test]
fn retained_and_unknown_are_explicit_non_green_results() {
    let mut retained = receipt();
    retained.after.branch = ResourceFinalizationBranchState::Present;
    retained.after.worktree = ResourceFinalizationWorktreeState::Clean;
    retained.result.disposition = ResourceFinalizationDisposition::Retained;
    validate_resource_finalization_receipt(&retained).unwrap();

    let mut unknown = receipt();
    unknown.after.branch = ResourceFinalizationBranchState::Unknown;
    unknown.after.worktree = ResourceFinalizationWorktreeState::Unknown;
    unknown.result.disposition = ResourceFinalizationDisposition::Unknown;
    unknown.result.unknown_codes = vec![RESOURCE_FINALIZATION_CODE_AMBIGUOUS_STATE.into()];
    validate_resource_finalization_receipt(&unknown).unwrap();
}

#[test]
fn duplicate_finalization_replay_is_idempotent_only_for_same_operation_and_result() {
    let original = receipt();
    let mut replay = original.clone();
    replay.receipt_id = "receipt-retry".into();
    replay.timestamp = "2026-08-23T00:01:00Z".into();
    validate_resource_finalization_replay(&original, &replay).unwrap();

    replay.operation_id = "operation-other".into();
    assert_eq!(
        validate_resource_finalization_replay(&original, &replay),
        Err(ResourceFinalizationError::ReplayMismatch("operationId"))
    );
}

#[test]
fn disposition_codes_and_states_are_strict() {
    let mut blocked_without_reason = receipt();
    blocked_without_reason.result.disposition = ResourceFinalizationDisposition::Blocked;
    assert!(validate_resource_finalization_receipt(&blocked_without_reason).is_err());

    let mut unknown_without_code = receipt();
    unknown_without_code.result.disposition = ResourceFinalizationDisposition::Unknown;
    assert!(validate_resource_finalization_receipt(&unknown_without_code).is_err());

    let mut deleted_with_code = receipt();
    deleted_with_code.result.failure_codes =
        vec![RESOURCE_FINALIZATION_CODE_AMBIGUOUS_STATE.into()];
    assert!(validate_resource_finalization_receipt(&deleted_with_code).is_err());
}
