use cockpit_repository::{
    ContractAmendmentReceipt, WorkItemStartOptions, amend_work_item_contract, archive_work_item,
    attach, checkpoint_work_item, finish_work_item, preflight_work_item,
    read_work_item_contract_amendments, record_verification, record_work_item_governance_controls,
    require_verification_preconditions, start_work_item_with_options,
};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("repository tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(directory.path()).expect("attach repository");
    directory
}

fn read_json(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read JSON file")).expect("valid JSON")
}

fn record_human_preflight_review(root: &Path, work_item_id: &str, contract_path: &Path) {
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let summary = read_json(summary_path);
    let contract = read_json(contract_path);
    let decision_evidence = json!({
        "schemaVersion": 1,
        "decisionId": "contract-preflight-review",
        "decision": "confirm_review",
        "workItemId": work_item_id,
        "repositoryId": cockpit_repository::repository_id(root).to_string(),
        "contractDigest": cockpit_protocol::digest_json(&contract).expect("Contract digest"),
        "preflightDecisionDigest": summary["preflightDecisionDigest"],
        "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
        "recordedAt": "2026-09-30T00:00:00Z",
        "recordedBy": "human:test-fixture",
        "reason": "the reviewed sensitive Contract change is accepted"
    });
    record_work_item_governance_controls(
        root,
        work_item_id,
        &json!({"decisionEvidence": decision_evidence}),
    )
    .expect("record identity-bound review evidence");
}

fn start_uncheckpointed(root: &Path, work_item_id: &str) -> std::path::PathBuf {
    start_work_item_with_options(
        root,
        work_item_id,
        "exercise recoverable Contract amendments",
        "preserve a reasoned audit trail across interrupted amendments",
        &["crates/cockpit-repository/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["amendment history is verifiable".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");
    root.join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"))
}

fn start_checkpointed(root: &Path, work_item_id: &str) -> std::path::PathBuf {
    let contract_path = start_uncheckpointed(root, work_item_id);
    preflight_work_item(root, &contract_path).expect("preflight");
    checkpoint_work_item(root, work_item_id).expect("checkpoint");
    contract_path
}

fn typed_request(
    contract_path: &Path,
    change_id: &str,
    reason: &str,
    path: &str,
    operation: &str,
    value: Value,
) -> Value {
    let contract = read_json(contract_path);
    json!({
        "schemaVersion": 1,
        "changeId": change_id,
        "expectedContractDigest": cockpit_protocol::digest_json(&contract).expect("Contract digest"),
        "reason": reason,
        "changes": [{ "path": path, "operation": operation, "value": value }]
    })
}

#[test]
fn typed_request_replaces_goal_without_rewriting_identity() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-TYPED-CONTRACT-AMENDMENT";
    start_work_item_with_options(
        root,
        work_item_id,
        "exercise a typed Contract amendment",
        "replace a human-owned plan field while preserving Runtime identity",
        &["crates/cockpit-repository/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["the replacement and identity are observed".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");

    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    preflight_work_item(root, &contract_path).expect("initial preflight");
    checkpoint_work_item(root, work_item_id).expect("checkpoint");
    let original = read_json(&contract_path);
    let expected_digest = cockpit_protocol::digest_json(&original).expect("Contract digest");
    let replacement = "Replace brittle legacy command handling with governed typed amendments";
    let reason = "the implementation plan now requires replacing, not only appending, goals";

    amend_work_item_contract(
        root,
        work_item_id,
        &json!({
            "schemaVersion": 1,
            "changeId": "replace-goal-1",
            "expectedContractDigest": expected_digest,
            "reason": reason,
            "changes": [{
                "path": "/goal",
                "operation": "replace",
                "value": replacement
            }]
        }),
        reason,
    )
    .expect("typed reasoned amendment");

    let amended = read_json(&contract_path);
    assert_eq!(amended["goal"], replacement);
    for field in [
        "repositoryId",
        "workItemId",
        "baseRevision",
        "projectProfileDigest",
        "repositorySnapshotDigest",
        "authority",
        "state",
    ] {
        assert_eq!(amended[field], original[field], "protected field {field}");
    }
}

#[test]
fn stale_contract_digest_rejects_without_writing() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-STALE-CONTRACT-AMENDMENT";
    start_work_item_with_options(
        root,
        work_item_id,
        "reject stale amendment input",
        "do not silently rebase an amendment over a changed Contract",
        &["crates/cockpit-repository/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["stale amendment state remains unchanged".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    preflight_work_item(root, &contract_path).expect("initial preflight");
    checkpoint_work_item(root, work_item_id).expect("checkpoint");
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let original_contract = fs::read(&contract_path).expect("Contract bytes");
    let original_summary = fs::read(&summary_path).expect("Summary bytes");
    let request = json!({
        "schemaVersion": 1,
        "changeId": "stale-change-1",
        "expectedContractDigest": cockpit_core::Digest::sha256_bytes(b"stale").to_string(),
        "reason": "this request was prepared against an older Contract",
        "changes": [{
            "path": "/goal",
            "operation": "replace",
            "value": "do not apply"
        }]
    });

    let error = amend_work_item_contract(
        root,
        work_item_id,
        &request,
        "this request was prepared against an older Contract",
    )
    .expect_err("stale digest must conflict");
    assert!(error.to_string().contains("contract_digest_conflict"));
    assert_eq!(
        fs::read(&contract_path).expect("Contract bytes"),
        original_contract
    );
    assert_eq!(
        fs::read(&summary_path).expect("Summary bytes"),
        original_summary
    );
}

#[test]
fn retrying_the_same_change_id_returns_the_original_amendment_receipt() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-IDEMPOTENT-CONTRACT-AMENDMENT";
    start_work_item_with_options(
        root,
        work_item_id,
        "make amendment retries idempotent",
        "an interrupted caller can safely retry one amendment",
        &["crates/cockpit-repository/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["same request returns its original receipt".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");

    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    preflight_work_item(root, &contract_path).expect("initial preflight");
    checkpoint_work_item(root, work_item_id).expect("checkpoint");
    let original = read_json(&contract_path);
    let reason = "retry after an uncertain response without creating a second amendment";
    let request = json!({
        "schemaVersion": 1,
        "changeId": "same-change-retry-1",
        "expectedContractDigest": cockpit_protocol::digest_json(&original).expect("digest"),
        "reason": reason,
        "changes": [{
            "path": "/goal",
            "operation": "replace",
            "value": "Safely retry an already committed amendment"
        }]
    });

    let first = amend_work_item_contract(root, work_item_id, &request, reason)
        .expect("first amendment commits");
    let second = amend_work_item_contract(root, work_item_id, &request, reason)
        .expect("identical retry returns the committed receipt");

    assert_eq!(second, first);
    let mut conflicting_retry = request.clone();
    conflicting_retry["reason"] = json!("same changeId with a different explanation");
    let conflict = amend_work_item_contract(
        root,
        work_item_id,
        &conflicting_retry,
        "same changeId with a different explanation",
    )
    .expect_err("a reused changeId cannot alias a different request");
    assert!(
        conflict
            .to_string()
            .contains("already exists with different request bytes")
    );
    assert_eq!(
        read_work_item_contract_amendments(root, work_item_id)
            .expect("single committed history entry")
            .len(),
        1
    );
    assert_eq!(
        read_json(&contract_path)["goal"],
        "Safely retry an already committed amendment"
    );
}

#[test]
fn competing_amendments_from_one_contract_digest_allow_only_the_first_commit() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-COMPETING-CONTRACT-AMENDMENTS";
    let contract_path = start_uncheckpointed(root, work_item_id);
    let reason = "concurrent writers must not silently rebase a stale amendment";
    let contract_digest =
        cockpit_protocol::digest_json(&read_json(&contract_path)).expect("Contract digest");
    let first = json!({
        "schemaVersion": 1,
        "changeId": "competing-change-first",
        "expectedContractDigest": contract_digest,
        "reason": reason,
        "changes": [{"path":"/goal","operation":"replace","value":"first committed plan"}]
    });
    let second = json!({
        "schemaVersion": 1,
        "changeId": "competing-change-second",
        "expectedContractDigest": contract_digest,
        "reason": reason,
        "changes": [{"path":"/goal","operation":"replace","value":"must not overwrite first"}]
    });

    amend_work_item_contract(root, work_item_id, &first, reason).expect("first amendment commits");
    let conflict = amend_work_item_contract(root, work_item_id, &second, reason)
        .expect_err("second writer's old digest must conflict");

    assert!(conflict.to_string().contains("contract_digest_conflict"));
    assert_eq!(read_json(&contract_path)["goal"], "first committed plan");
    assert_eq!(
        read_work_item_contract_amendments(root, work_item_id)
            .expect("read verified history")
            .len(),
        1
    );
}

#[test]
fn amendment_history_chains_receipts_and_rejects_a_tampered_commit() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-CONTRACT-AMENDMENT-AUDIT-CHAIN";
    let contract_path = start_checkpointed(root, work_item_id);
    let first_reason = "record the first traceable plan change";
    let first_request = typed_request(
        &contract_path,
        "audit-chain-first",
        first_reason,
        "/goal",
        "replace",
        json!("first plan revision"),
    );
    amend_work_item_contract(root, work_item_id, &first_request, first_reason)
        .expect("first amendment");
    let second_reason = "record the next traceable plan change";
    let second_request = typed_request(
        &contract_path,
        "audit-chain-second",
        second_reason,
        "/title",
        "replace",
        json!("second title revision"),
    );
    amend_work_item_contract(root, work_item_id, &second_request, second_reason)
        .expect("second amendment");

    for sequence in [1, 2] {
        let directory = root
            .join(".ai/evidence")
            .join(format!("{work_item_id}.contract-amendments"));
        let prepared = read_json(directory.join(format!("{sequence:08}.prepared.json")));
        let committed = read_json(directory.join(format!("{sequence:08}.committed.json")));
        let receipt = &committed["receipt"];
        assert_eq!(committed["sequence"], sequence);
        assert_eq!(committed["preparedDigest"], prepared["preparedDigest"]);
        assert_eq!(receipt["sequence"], sequence);
        assert_eq!(receipt["workItemId"], work_item_id);
        assert_eq!(
            receipt["requestDigest"],
            prepared["prepared"]["requestDigest"]
        );
        assert_eq!(
            receipt["changeId"],
            prepared["prepared"]["request"]["changeId"]
        );
        assert_eq!(
            receipt["previousJournalDigest"],
            prepared["prepared"]["previousJournalDigest"]
        );
        let mut receipt_core = receipt.clone();
        receipt_core
            .as_object_mut()
            .expect("receipt is an object")
            .remove("journalDigest");
        assert_eq!(
            receipt["journalDigest"],
            cockpit_protocol::digest_json(&json!({
                "previousJournalDigest": receipt["previousJournalDigest"],
                "receipt": receipt_core,
            }))
            .expect("journal digest")
            .to_string()
        );
        let parsed_receipt: ContractAmendmentReceipt =
            serde_json::from_value(receipt.clone()).expect("typed amendment receipt");
        let mut typed_core =
            serde_json::to_value(&parsed_receipt).expect("serialize typed receipt");
        typed_core
            .as_object_mut()
            .expect("typed receipt is an object")
            .remove("journalDigest");
        assert_eq!(
            typed_core, receipt_core,
            "typed and persisted receipt core differ"
        );
        assert_eq!(
            parsed_receipt.journal_digest,
            cockpit_protocol::digest_json(&json!({
                "previousJournalDigest": parsed_receipt.previous_journal_digest,
                "receipt": typed_core,
            }))
            .expect("typed journal digest")
        );
        let expected_previous_digest = if sequence == 1 {
            cockpit_core::Digest::sha256_bytes(b"cockpit-contract-amendment-journal-v1").to_string()
        } else {
            let previous = read_json(directory.join("00000001.committed.json"));
            previous["receipt"]["journalDigest"]
                .as_str()
                .expect("first journal digest")
                .to_owned()
        };
        assert_eq!(
            receipt["previousJournalDigest"], expected_previous_digest,
            "sequence {sequence} links to its actual predecessor"
        );
    }

    let history = read_work_item_contract_amendments(root, work_item_id)
        .expect("history has a valid hash chain");
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].sequence, 1);
    assert_eq!(history[1].sequence, 2);
    assert_eq!(
        history[1].previous_journal_digest,
        history[0].journal_digest
    );

    let committed_path = root.join(".ai/evidence").join(format!(
        "{work_item_id}.contract-amendments/00000002.committed.json"
    ));
    let mut committed = read_json(&committed_path);
    committed["receipt"]["reason"] = json!("tampered after commit");
    fs::write(
        &committed_path,
        serde_json::to_vec_pretty(&committed).expect("serialize tampered record"),
    )
    .expect("write corruption fixture");
    let error = read_work_item_contract_amendments(root, work_item_id)
        .expect_err("tampered committed receipt must fail closed");
    assert!(error.to_string().contains("digest chain"));
}

#[test]
fn amendment_history_rejects_a_resealed_receipt_detached_from_its_request() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-AMENDMENT-RECEIPT-BINDING";
    let contract_path = start_checkpointed(root, work_item_id);
    let reason = "bind the immutable receipt back to this exact request";
    let request = typed_request(
        &contract_path,
        "receipt-binding-change",
        reason,
        "/goal",
        "replace",
        json!("the exact requested plan value"),
    );
    amend_work_item_contract(root, work_item_id, &request, reason).expect("record bound amendment");

    let committed_path = root.join(".ai/evidence").join(format!(
        "{work_item_id}.contract-amendments/00000001.committed.json"
    ));
    let mut committed = read_json(&committed_path);
    committed["receipt"]["reason"] = json!("forged reason");
    committed["receipt"]["changedValues"][0]["newValue"] = json!("forged plan value");
    let mut receipt_core = committed["receipt"].clone();
    receipt_core
        .as_object_mut()
        .expect("receipt is an object")
        .remove("journalDigest");
    committed["receipt"]["journalDigest"] = json!(
        cockpit_protocol::digest_json(&json!({
            "previousJournalDigest": committed["receipt"]["previousJournalDigest"],
            "receipt": receipt_core,
        }))
        .expect("re-seal modified receipt")
        .to_string()
    );
    fs::write(
        &committed_path,
        serde_json::to_vec_pretty(&committed).expect("serialize modified commit"),
    )
    .expect("write resealed receipt");

    let error = read_work_item_contract_amendments(root, work_item_id)
        .expect_err("a self-consistent hash cannot detach receipt content from its request");
    assert!(error.to_string().contains("prepared amendment"));
}

#[test]
fn sensitive_plan_amendment_records_its_policy_review_requirement() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-SENSITIVE-CONTRACT-AMENDMENT";
    let contract_path = start_checkpointed(root, work_item_id);
    let reason = "scope changes require the corresponding policy review";
    let request = typed_request(
        &contract_path,
        "sensitive-scope-change",
        reason,
        "/scope",
        "replace",
        json!(["crates/cockpit-repository/**", "crates/cockpit-protocol/**"]),
    );

    amend_work_item_contract(root, work_item_id, &request, reason)
        .expect("sensitive amendment is recorded for review");
    let history =
        read_work_item_contract_amendments(root, work_item_id).expect("history remains valid");

    assert_eq!(history.len(), 1);
    assert!(history[0].policy_review_required);
    assert_eq!(history[0].policy_review_requirements, vec!["/scope"]);
}

#[test]
fn sensitive_amendment_requires_human_review_before_verification() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-SENSITIVE-AMENDMENT-REVIEW-GATE";
    let contract_path = start_checkpointed(root, work_item_id);
    let reason = "a scope change requires fresh human review before verification";
    let request = typed_request(
        &contract_path,
        "sensitive-scope-review-gate",
        reason,
        "/scope",
        "replace",
        json!(["crates/cockpit-repository/**", "crates/cockpit-protocol/**"]),
    );
    amend_work_item_contract(root, work_item_id, &request, reason)
        .expect("sensitive amendment is recorded");

    let decision = preflight_work_item(root, &contract_path).expect("preflight after amendment");
    assert_eq!(
        decision.review_state.as_deref(),
        Some("needs_human_confirmation")
    );
    assert!(decision.human_decision_request.is_some());

    let runtime = cockpit_protocol::RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: cockpit_core::Digest::sha256_bytes(b"test-runtime"),
    };
    let snapshot = cockpit_git::GitRepository::discover(root)
        .expect("discover repository")
        .snapshot()
        .expect("capture repository snapshot");
    let error = require_verification_preconditions(root, work_item_id, &runtime, &snapshot)
        .expect_err("verification must wait for the required human review");
    assert!(
        error
            .to_string()
            .contains("contract_amendment_policy_review_required")
    );

    record_human_preflight_review(root, work_item_id, &contract_path);
    let reviewed = preflight_work_item(root, &contract_path).expect("preflight after review");
    assert_eq!(
        reviewed.review_state.as_deref(),
        Some("human_decision_recorded")
    );
    require_verification_preconditions(root, work_item_id, &runtime, &snapshot)
        .expect("a valid decision receipt satisfies the current verification preconditions");
}

#[test]
fn uncheckpointed_amendment_persists_runtime_capability_requirements() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-UNCP-CAPABILITY-REQUIREMENT";
    let contract_path = start_uncheckpointed(root, work_item_id);
    let reason = "the amended Contract requires a Runtime that enforces its journal";
    let request = typed_request(
        &contract_path,
        "uncheckpointed-runtime-capability",
        reason,
        "/goal",
        "replace",
        json!("persist the amendment capability boundary"),
    );

    amend_work_item_contract(root, work_item_id, &request, reason)
        .expect("uncheckpointed amendment is recorded");

    let contract = read_json(&contract_path);
    assert_eq!(
        contract["requiredRuntimeCapabilities"],
        json!([
            "work-item-contract-amendment",
            "work-item-environment-drift"
        ])
    );
}

#[test]
fn material_review_profile_amendment_adds_audited_protected_guard() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-MATERIAL-REVIEW-CAPABILITY-GUARD";
    let contract_path = start_uncheckpointed(root, work_item_id);
    let reason = "enable only the strict reviewed-syntax profile in a later amendment";
    let request = typed_request(
        &contract_path,
        "material-review-profile-opt-in",
        reason,
        "/governanceProfile",
        "set",
        json!({
            "unrelatedProfileField": {"preserve": true},
            "materialInspectionReview": {
                "schemaVersion": 1,
                "permittedUnknown": "repository_material_inspection_unavailable",
                "permittedCause": "readable_committed_rust_syntax_unknown",
                "assurance": "self_declared",
                "reviewerActor": "agent:Raydot",
                "authoritySource": "user:Ray 2026-10-07 current-conversation delegation to agent:Raydot (WI-1068 Contract.sources)",
                "acceptResidualRisk": true
            }
        }),
    );

    amend_work_item_contract(root, work_item_id, &request, reason)
        .expect("typed writer installs the protected capability guard");
    let contract = read_json(&contract_path);
    let history = read_work_item_contract_amendments(root, work_item_id)
        .expect("amendment journal validates");
    assert_eq!(
        contract["requiredRuntimeCapabilities"],
        json!([
            "material-inspection-review",
            "work-item-contract-amendment",
            "work-item-environment-drift"
        ])
    );
    assert_eq!(
        contract["governanceProfile"]["unrelatedProfileField"]["preserve"],
        true
    );
    assert_eq!(
        history[0]
            .changed_values
            .iter()
            .find(|change| change.path == "/requiredRuntimeCapabilities")
            .and_then(|change| change.new_value.as_ref()),
        contract.get("requiredRuntimeCapabilities")
    );
    assert_eq!(history.len(), 1);
}

#[test]
fn amendment_audit_records_each_ordered_operation_value() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-AMENDMENT-ORDERED-AUDIT";
    let contract_path = start_uncheckpointed(root, work_item_id);
    let original = read_json(&contract_path);
    let reason = "record each intermediate value in the ordered amendment batch";
    let request = json!({
        "schemaVersion": 1,
        "changeId": "ordered-goal-values",
        "expectedContractDigest": cockpit_protocol::digest_json(&original).expect("Contract digest"),
        "reason": reason,
        "changes": [
            {"path": "/goal", "operation": "set", "value": "intermediate goal"},
            {"path": "/goal", "operation": "replace", "value": "final goal"}
        ]
    });

    amend_work_item_contract(root, work_item_id, &request, reason)
        .expect("ordered batch is recorded");

    let history = read_work_item_contract_amendments(root, work_item_id).expect("history");
    assert_eq!(
        history[0].changed_values[0].old_value,
        original.get("goal").cloned()
    );
    assert_eq!(
        history[0].changed_values[0].new_value,
        Some(json!("intermediate goal"))
    );
    assert_eq!(
        history[0].changed_values[1].old_value,
        Some(json!("intermediate goal"))
    );
    assert_eq!(
        history[0].changed_values[1].new_value,
        Some(json!("final goal"))
    );
}

#[test]
fn post_verification_amendment_receipt_lists_invalidated_required_checks() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-AMENDMENT-INVALIDATED-CHECKS";
    let contract_path = start_uncheckpointed(root, work_item_id);
    let policy_reason = "declare the required check before the initial checkpoint";
    let policy = typed_request(
        &contract_path,
        "declare-required-amendment-check",
        policy_reason,
        "/checkpointPolicy",
        "set",
        json!({
            "schemaVersion": 1,
            "profile": "standard",
            "requiredBeforeFinish": true,
            "requiredStages": ["before_edit", "before_finish"],
            "requiredChecks": ["requiredContractCheck"]
        }),
    );
    amend_work_item_contract(root, work_item_id, &policy, policy_reason)
        .expect("declare the required check");
    preflight_work_item(root, &contract_path).expect("preflight updated Contract");
    record_human_preflight_review(root, work_item_id, &contract_path);
    checkpoint_work_item(root, work_item_id).expect("checkpoint updated Contract");
    record_verification(
        root,
        work_item_id,
        &json!({"passed": true, "nodesPlanned": 1}),
        "test-runtime",
        &cockpit_core::Digest::sha256_bytes(b"test-runtime"),
    )
    .expect("record predecessor verification");

    let reason = "a plan change after verification invalidates its required check";
    let amendment = typed_request(
        &contract_path,
        "invalidate-required-check",
        reason,
        "/goal",
        "replace",
        json!("revised after required verification"),
    );
    amend_work_item_contract(root, work_item_id, &amendment, reason)
        .expect("record amendment after verification");
    let history =
        read_work_item_contract_amendments(root, work_item_id).expect("history remains valid");

    assert_eq!(history.len(), 2);
    assert_eq!(
        history[1].invalidated_required_checks,
        vec!["requiredContractCheck"]
    );
    assert_eq!(
        history[1].invalidated_evidence,
        vec![format!(".ai/evidence/{work_item_id}.verification.json")]
    );
}

#[test]
fn amendment_history_survives_archive_without_rewriting_archived_bytes() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-AMENDMENT-ARCHIVE-PRESERVATION";
    let contract_path = start_uncheckpointed(root, work_item_id);
    let reason = "preserve the amendment journal unchanged through archive";
    let request = typed_request(
        &contract_path,
        "archive-preservation-change",
        reason,
        "/goal",
        "replace",
        json!("archive with the complete amendment audit chain"),
    );
    amend_work_item_contract(root, work_item_id, &request, reason)
        .expect("record amendment before archive");

    preflight_work_item(root, &contract_path).expect("preflight amended Contract");
    checkpoint_work_item(root, work_item_id).expect("checkpoint amended Contract");
    record_verification(
        root,
        work_item_id,
        &json!({"passed": true, "nodesPlanned": 1}),
        "test-runtime",
        &cockpit_core::Digest::sha256_bytes(b"test-runtime"),
    )
    .expect("record verification");
    finish_work_item(root, work_item_id).expect("finish");
    archive_work_item(root, work_item_id).expect("archive");

    let archive = root.join(".ai/work-items/archive");
    let archived_paths = [
        archive.join(format!("{work_item_id}.contract.json")),
        archive.join(format!("{work_item_id}.summary.json")),
        archive.join(format!("{work_item_id}.outcome.json")),
        archive.join(format!("{work_item_id}.archive.json")),
    ];
    let archived_before = archived_paths
        .iter()
        .map(|path| fs::read(path).expect("archived bytes"))
        .collect::<Vec<_>>();
    let journal_directory = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.contract-amendments"));
    let journal_before = [
        journal_directory.join("00000001.prepared.json"),
        journal_directory.join("00000001.committed.json"),
    ]
    .map(|path| fs::read(path).expect("journal bytes"));

    let history = read_work_item_contract_amendments(root, work_item_id)
        .expect("archived amendment history remains readable");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].change_id, "archive-preservation-change");
    for (path, original) in archived_paths.iter().zip(archived_before) {
        assert_eq!(
            fs::read(path).expect("archived bytes after query"),
            original
        );
    }
    let journal_after = [
        journal_directory.join("00000001.prepared.json"),
        journal_directory.join("00000001.committed.json"),
    ]
    .map(|path| fs::read(path).expect("journal bytes after query"));
    assert_eq!(journal_after, journal_before);
}
