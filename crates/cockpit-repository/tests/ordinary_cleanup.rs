use cockpit_core::Digest;
use cockpit_repository::{
    OrdinaryCleanupObservation, OrdinaryCleanupReceipt, OrdinaryCleanupResult,
};

#[test]
fn ordinary_cleanup_receipt_keeps_identity_and_result_shape() {
    let receipt = OrdinaryCleanupReceipt {
        schema_version: 1,
        operation_id: "ordinary-cleanup-test".into(),
        repository_id: "sha256:repository".into(),
        work_item_id: "WI-ORDINARY-CLEANUP-BOUNDARY".into(),
        contract_digest: Digest::sha256_bytes(b"contract"),
        binding_digest: Digest::sha256_bytes(b"binding"),
        branch_ref: "refs/heads/feature/ordinary-cleanup".into(),
        head_revision: "0123456789012345678901234567890123456789".into(),
        worktree_id: Digest::sha256_bytes(b"worktree"),
        sequence: 1,
        predecessor_receipt_digest: None,
        runtime_version: "test-runtime".into(),
        runtime_digest: Digest::sha256_bytes(b"runtime"),
        observed_at: "2026-09-22T00:00:00Z".into(),
        observation: OrdinaryCleanupObservation {
            branch: "removed".into(),
            worktree: "removed".into(),
        },
        result: OrdinaryCleanupResult {
            state: "verified".into(),
            failure_codes: Vec::new(),
        },
    };
    let value = serde_json::to_value(&receipt).expect("receipt JSON");
    assert_eq!(value["workItemId"], "WI-ORDINARY-CLEANUP-BOUNDARY");
    assert_eq!(value["observation"]["branch"], "removed");
    assert_eq!(value["result"]["state"], "verified");
    let decoded: OrdinaryCleanupReceipt = serde_json::from_value(value).expect("receipt roundtrip");
    assert_eq!(decoded, receipt);
}
