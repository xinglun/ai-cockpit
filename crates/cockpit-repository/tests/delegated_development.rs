use cockpit_core::Digest;
use cockpit_protocol::{ContractAmendmentRequest, PROTOCOL_VERSION, RuntimeContext, digest_json};
use cockpit_repository::*;
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const WI: &str = "WI-DELEGATED-FIXTURE";

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "1.0.1-test".into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"isolated-delegated-fixture-runtime"),
    }
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("git");
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn document(root: &Path, suffix: &str) -> Value {
    serde_json::from_slice(
        &fs::read(root.join(format!(".ai/work-items/active/{WI}.{suffix}.json")))
            .expect("document"),
    )
    .expect("JSON")
}

fn profile() -> Value {
    json!({"schemaVersion":1, "reviewerActor":"agent:fixture", "authoritySource":"user:isolated-fixture",
        "allowedChanges":["append_scenarios", "fill_verification_plan", "append_verification"],
        "preapprovedCommands":["git status --porcelain", "git diff --check"]})
}

fn fixture(delegation: bool) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temp repository");
    git(root.path(), &["init", "-q", "--initial-branch=main"]);
    git(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(root.path(), &["config", "user.name", "Isolated fixture"]);
    fs::write(root.path().join("README.md"), "initial\n").expect("source");
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "baseline"]);
    attach(root.path()).expect("attach");
    start_work_item_with_options(
        root.path(),
        WI,
        "synthetic delegated development",
        "bounded flow",
        &["README.md".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["original acceptance remains".into()],
            verification_commands: vec!["git diff --check".into()],
            ..Default::default()
        },
    )
    .expect("start");
    if delegation {
        // Fixture installation is explicitly synthetic, not a real user or B source.
        let path = root
            .path()
            .join(format!(".ai/work-items/active/{WI}.contract.json"));
        let mut c = document(root.path(), "contract");
        c["governanceProfile"] = json!({"developmentReview":profile()});
        c["requiredRuntimeCapabilities"] = json!(["delegated-development-review"]);
        fs::write(path, serde_json::to_vec_pretty(&c).expect("JSON"))
            .expect("fixture configuration");
    }
    root
}

fn preflight(root: &Path) -> cockpit_repository::PreflightResult {
    preflight_work_item_with_runtime_report(
        root,
        &root.join(format!(".ai/work-items/active/{WI}.contract.json")),
        &runtime(),
    )
    .expect("preflight")
}

fn owner_review(root: &Path) {
    preflight(root);
    let c = document(root, "contract");
    let s = document(root, "summary");
    record_work_item_governance_controls_with_runtime(root,WI,&json!({"decisionEvidence":{
        "schemaVersion":1,"decisionId":"contract-preflight-review","decision":"confirm_review",
        "workItemId":WI,"repositoryId":repository_id(root),"contractDigest":digest_json(&c).expect("digest"),
        "preflightDecisionDigest":s["preflightDecisionDigest"],"repositorySnapshotDigest":s["preflightRepositorySnapshotDigest"],
        "recordedAt":"2026-10-23T00:00:00Z", "recordedBy":"human:synthetic-fixture",
        "reason":"isolated local provenance fixture; no external identity authenticated"
    }}),&runtime()).expect("synthetic formal owner review");
    preflight(root);
    checkpoint_work_item(root, WI).expect("checkpoint");
}

fn request(root: &Path, id: &str, changes: Value) -> ContractAmendmentRequest {
    serde_json::from_value(json!({"schemaVersion":1,"changeId":id,"reason":"bounded technical adjustment",
        "expectedContractDigest":digest_json(&document(root,"contract")).expect("digest"),"changes":changes,
        "developmentReview":{"actor":"agent:fixture","authoritySource":"user:isolated-fixture",
            "assurance":"self_declared","operations":["review_development_amendment"],"policyRefs":[],"evidenceRefs":[]}
    })).expect("typed request")
}

#[test]
fn formally_reviewed_opt_in_allows_exact_preapproved_append_without_a_new_human_decision() {
    let root = fixture(true);
    owner_review(root.path());
    let owner_bytes = fs::read(
        root.path()
            .join(format!(".ai/decisions/{WI}.preflight-review.json")),
    )
    .expect("owner bytes");
    let r = request(
        root.path(),
        "append-check",
        json!([{"path":"/verification","operation":"add","value":"git status --porcelain"}]),
    );
    let receipt = amend_work_item_contract_with_runtime(
        root.path(),
        WI,
        &serde_json::to_value(&r).expect("request JSON"),
        "",
        &runtime(),
    )
    .expect("permitted technical amendment");
    assert!(
        receipt["policyReviewRequired"] == false,
        "approved finite change must not demand a second human review"
    );
    assert!(serde_json::to_value(&receipt).expect("receipt")["delegatedReview"].is_object());
    let result = preflight(root.path());
    assert!(
        result.decision.human_decision_request.is_none(),
        "{:?}",
        result.decision
    );
    assert_eq!(
        fs::read(
            root.path()
                .join(format!(".ai/decisions/{WI}.preflight-review.json"))
        )
        .expect("owner bytes"),
        owner_bytes
    );
    assert_eq!(
        document(root.path(), "contract")["verification"],
        json!(["git diff --check", "git status --porcelain"])
    );
}

fn amend(root: &Path, request: &ContractAmendmentRequest) -> Result<Value, ObserverError> {
    amend_work_item_contract_with_runtime(
        root,
        WI,
        &serde_json::to_value(request).expect("typed request"),
        "",
        &runtime(),
    )
}

fn state_bytes(root: &Path) -> (Vec<u8>, Vec<u8>, usize) {
    (
        fs::read(root.join(format!(".ai/work-items/active/{WI}.contract.json"))).expect("Contract"),
        fs::read(root.join(format!(".ai/work-items/active/{WI}.summary.json"))).expect("Summary"),
        read_work_item_contract_amendments(root, WI)
            .expect("history")
            .len(),
    )
}

#[test]
fn strict_mode_and_absent_owner_review_never_allow_a_claimed_agent() {
    for opted_in in [false, true] {
        let root = fixture(opted_in);
        preflight(root.path());
        let before = state_bytes(root.path());
        let r = request(
            root.path(),
            "unapproved",
            json!([{"path":"/verification","operation":"add","value":"git status --porcelain"}]),
        );
        assert!(amend(root.path(), &r).is_err());
        assert_eq!(state_bytes(root.path()), before);
    }
    let root = fixture(false);
    owner_review(root.path());
    let mut r = request(
        root.path(),
        "ordinary-strict",
        json!([{"path":"/verification","operation":"add","value":"git status --porcelain"}]),
    );
    r.development_review = None;
    assert_eq!(
        amend(root.path(), &r).expect("ordinary native amendment")["policyReviewRequired"],
        true
    );
    assert!(
        preflight(root.path())
            .decision
            .human_decision_request
            .is_some()
    );
}

#[test]
fn each_protected_change_or_unapproved_command_has_zero_native_side_effects() {
    let cases = [
        json!({"path":"/scope","operation":"add","value":"outside/**"}),
        json!({"path":"/authority","operation":"set","value":"missing"}),
        json!({"path":"/risk","operation":"set","value":"low"}),
        json!({"path":"/acceptanceCriteria","operation":"add","value":"different standard"}),
        json!({"path":"/verification","operation":"add","value":"git status --short"}),
        json!({"path":"/verification","operation":"replace","value":["git status --porcelain"]}),
        json!({"path":"/governanceProfile/developmentReview/reviewerActor","operation":"set","value":"agent:other"}),
        json!({"path":"/goal","operation":"set","value":"a different purpose"}),
        json!({"path":"/scenarioCoverage","operation":"add","value":{"scenario":"weaker","required":false,"status":"unverified","evidence":[],"expected":"x","verificationPlan":"git diff --check"}}),
    ];
    let root = fixture(true);
    owner_review(root.path());
    let before = state_bytes(root.path());
    for (i, change) in cases.into_iter().enumerate() {
        let r = request(root.path(), &format!("denied-{i}"), json!([change]));
        assert!(
            amend(root.path(), &r).is_err(),
            "case{i} must not receive delegated approval"
        );
        assert_eq!(
            state_bytes(root.path()),
            before,
            "case{i} changed native state"
        );
    }
}

#[test]
fn wrong_reviewer_source_operation_and_assurance_cannot_borrow_the_baseline() {
    let root = fixture(true);
    owner_review(root.path());
    let before = state_bytes(root.path());
    for (field, value) in [
        ("actor", json!("human:owner")),
        ("actor", json!("agent:other")),
        ("authoritySource", json!("arbitrary-file.json")),
        ("operations", json!(["merge"])),
        ("assurance", json!("provider_verified")),
    ] {
        let r = request(
            root.path(),
            "wrong-source",
            json!([{"path":"/verification","operation":"add","value":"git status --porcelain"}]),
        );
        let mut value_request = serde_json::to_value(r).expect("request");
        value_request["developmentReview"][field] = value;
        let r: ContractAmendmentRequest =
            serde_json::from_value(value_request).expect("input shape");
        assert!(amend(root.path(), &r).is_err(), "{field}");
        assert_eq!(state_bytes(root.path()), before);
    }
}

#[test]
fn a_finite_scenario_append_can_be_followed_by_an_exact_preapproved_command() {
    let root = fixture(true);
    owner_review(root.path());
    let r = request(
        root.path(),
        "scenario",
        json!([{"path":"/scenarioCoverage","operation":"add","value":{
        "scenario":"added-regression","required":true,"status":"unverified","evidence":[],
        "expected":"the new regression executes","verificationPlan":"git diff --check"}}]),
    );
    assert_eq!(
        amend(root.path(), &r).expect("planned scenario")["policyReviewRequired"],
        false
    );
    assert!(
        preflight(root.path())
            .decision
            .human_decision_request
            .is_none()
    );
    let r = request(
        root.path(),
        "next-command",
        json!([{"path":"/verification","operation":"add","value":"git status --porcelain"}]),
    );
    assert_eq!(
        amend(root.path(), &r).expect("second delegated amendment")["policyReviewRequired"],
        false
    );
    assert_eq!(
        read_work_item_contract_amendments(root.path(), WI)
            .expect("validated chain")
            .len(),
        2
    );
    assert_eq!(
        document(root.path(), "contract")["scenarioCoverage"][0]["status"],
        "unverified"
    );
}

#[test]
fn empty_existing_plan_can_be_filled_but_a_nonempty_plan_cannot_be_rewritten() {
    let root = fixture(true);
    let path = root
        .path()
        .join(format!(".ai/work-items/active/{WI}.contract.json"));
    let mut c = document(root.path(), "contract");
    c["scenarioCoverage"] = json!([{"scenario":"pending-plan","required":true,"status":"unverified","evidence":[],"expected":"bounded regression","verificationPlan":null}]);
    fs::write(path, serde_json::to_vec_pretty(&c).expect("fixture JSON"))
        .expect("synthetic initial plan");
    owner_review(root.path());
    let r = request(
        root.path(),
        "fill-plan",
        json!([{"path":"/scenarioCoverage/0/verificationPlan","operation":"set","value":"git diff --check"}]),
    );
    assert_eq!(
        amend(root.path(), &r).expect("empty plan fill")["policyReviewRequired"],
        false
    );
    preflight(root.path());
    let before = state_bytes(root.path());
    let r = request(
        root.path(),
        "rewrite-plan",
        json!([{"path":"/scenarioCoverage/0/verificationPlan","operation":"set","value":"git status --porcelain"}]),
    );
    assert!(amend(root.path(), &r).is_err());
    assert_eq!(state_bytes(root.path()), before);
}

#[test]
fn renamed_deleted_or_tampered_owner_receipt_invalidates_native_delegation() {
    for change in ["rename", "delete", "tamper"] {
        let root = fixture(true);
        owner_review(root.path());
        let r = request(
            root.path(),
            "first",
            json!([{"path":"/verification","operation":"add","value":"git status --porcelain"}]),
        );
        amend(root.path(), &r).expect("first delegated change");
        let path = root
            .path()
            .join(format!(".ai/decisions/{WI}.preflight-review.json"));
        match change {
            "rename" => {
                fs::rename(&path, path.with_extension("renamed.json")).expect("rename fixture")
            }
            "delete" => fs::remove_file(&path).expect("delete fixture"),
            _ => fs::write(&path, b"{}").expect("tamper fixture"),
        }
        assert!(
            read_work_item_contract_amendments(root.path(), WI).is_err(),
            "{change}"
        );
    }
}

#[test]
fn changed_source_requires_fresh_preflight_and_stale_cas_remains_rejected() {
    let root = fixture(true);
    owner_review(root.path());
    let r = request(
        root.path(),
        "append",
        json!([{"path":"/verification","operation":"add","value":"git status --porcelain"}]),
    );
    fs::write(
        root.path().join("README.md"),
        "changed within approved scope\n",
    )
    .expect("change source");
    assert!(amend(root.path(), &r).is_err(), "stale source preflight");
    let result = preflight(root.path());
    assert!(
        result.decision.human_decision_request.is_none(),
        "same Contract review remains durable"
    );
    let receipt = amend(root.path(), &r).expect("fresh preflight admits unchanged authorized plan");
    assert_eq!(amend(root.path(), &r).expect("idempotent retry"), receipt);
    let mut stale = r.clone();
    stale.change_id = "stale-other-request".into();
    assert!(amend(root.path(), &stale).is_err());
}

#[test]
fn explicit_human_policy_cannot_be_discharged_by_agent_review() {
    let root = fixture(true);
    owner_review(root.path());
    fs::write(root.path().join(".ai/policy.json"),serde_json::to_vec(&json!({"schemaVersion":1,"organization":{
        "policyId":"explicit-owner","layer":"organization","rules":[{"operation":"modify_source","approvalMode":"single_authorized_human","requiredEvidence":[]}]}
    })).expect("policy JSON")).expect("synthetic stronger policy");
    let r = request(
        root.path(),
        "stronger-policy",
        json!([{"path":"/verification","operation":"add","value":"git status --porcelain"}]),
    );
    assert!(amend(root.path(), &r).is_err());
}

#[test]
fn a_rebuilt_runtime_cannot_reuse_agent_review_as_current_admission() {
    let root = fixture(true);
    owner_review(root.path());
    let r = request(
        root.path(),
        "runtime-bound",
        json!([{"path":"/verification","operation":"add","value":"git status --porcelain"}]),
    );
    amend(root.path(), &r).expect("native technical review");
    let mut rebuilt = runtime();
    rebuilt.runtime_digest = Digest::sha256_bytes(b"different-rebuilt-runtime");
    let path = root
        .path()
        .join(format!(".ai/work-items/active/{WI}.contract.json"));
    let result = preflight_work_item_with_runtime_report(root.path(), &path, &rebuilt);
    assert!(
        result.is_err(),
        "a different Runtime must not project the old agent authorization as current"
    );
}

#[test]
fn delegated_amendment_rechecks_owner_after_waiting_for_work_item_lock() {
    const CHILD: &str = "AI_COCKPIT_DELEGATED_LOCK_CHILD";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = std::path::PathBuf::from(root);
        fs::write(root.join(".ai/locks/delegated-child-ready"), b"ready").expect("ready");
        let r = request(
            &root,
            "locked-request",
            json!([{"path":"/verification","operation":"add","value":"git status --porcelain"}]),
        );
        assert!(
            amend(&root, &r).is_err(),
            "owner changed while waiting; no cached approval may survive"
        );
        return;
    }
    let root = fixture(true);
    owner_review(root.path());
    let before = state_bytes(root.path());
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(root.path().join(format!(".ai/locks/{WI}.lifecycle.lock")))
        .expect("actual lock");
    lock.lock().expect("parent holds Work Item lock");
    let mut child = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "delegated_amendment_rechecks_owner_after_waiting_for_work_item_lock",
            "--test-threads=1",
        ])
        .env(CHILD, root.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("own bounded child");
    let start = std::time::Instant::now();
    let ready = root.path().join(".ai/locks/delegated-child-ready");
    while !ready.exists() && start.elapsed() < std::time::Duration::from_secs(10) {
        if child.try_wait().expect("child state").is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let observed = ready.exists();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let waiting = child.try_wait().expect("waiting child").is_none();
    fs::write(
        root.path()
            .join(format!(".ai/decisions/{WI}.preflight-review.json")),
        b"{}",
    )
    .expect("controlled source replacement");
    lock.unlock().expect("release lock");
    while child.try_wait().expect("child state").is_none()
        && start.elapsed() < std::time::Duration::from_secs(15)
    {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    if child.try_wait().expect("bounded child").is_none() {
        child.kill().expect("stop own timed-out test child");
    }
    let output = child.wait_with_output().expect("child output");
    assert!(
        observed && waiting && output.status.success(),
        "actual lock/source race: {output:?}"
    );
    assert_eq!(state_bytes(root.path()), before);
}

#[test]
fn missing_prepared_record_never_becomes_a_valid_technical_review() {
    let root = fixture(true);
    owner_review(root.path());
    let r = request(
        root.path(),
        "first",
        json!([{"path":"/verification","operation":"add","value":"git status --porcelain"}]),
    );
    amend(root.path(), &r).expect("native amendment");
    let directory = root
        .path()
        .join(format!(".ai/evidence/{WI}.contract-amendments"));
    let prepared = fs::read_dir(directory)
        .expect("journal")
        .map(|e| e.expect("entry").path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains("prepared"))
        })
        .expect("prepared record");
    fs::remove_file(prepared).expect("controlled missing record");
    assert!(read_work_item_contract_amendments(root.path(), WI).is_err());
}

#[cfg(unix)]
#[test]
fn a_symlinked_owner_parent_cannot_supply_a_delegated_review() {
    let root = fixture(true);
    owner_review(root.path());
    let before = state_bytes(root.path());
    let real = root.path().join(".ai/decisions-original");
    fs::rename(root.path().join(".ai/decisions"), &real).expect("fixture parent move");
    std::os::unix::fs::symlink(&real, root.path().join(".ai/decisions"))
        .expect("controlled parent substitution");
    let r = request(
        root.path(),
        "symlink-parent",
        json!([{"path":"/verification","operation":"add","value":"git status --porcelain"}]),
    );
    assert!(amend(root.path(), &r).is_err());
    assert_eq!(state_bytes(root.path()), before);
}

fn explicit_repository_policy(root: &Path, operation: &str, approval_mode: &str) {
    let policy = json!({"schemaVersion":1,"organization":{
        "policyId":"explicit-current-boundary","layer":"organization","rules":[{
            "operation":operation,"approvalMode":approval_mode,"requiredEvidence":[]
        }]
    }});
    fs::write(
        root.join(".ai/policy.json"),
        serde_json::to_vec(&policy).expect("policy JSON"),
    )
    .expect("synthetic current policy");
}

#[test]
fn actual_review_operation_cannot_borrow_agent_review_for_explicit_human_policy() {
    let root = fixture(true);
    let path = root
        .path()
        .join(format!(".ai/work-items/active/{WI}.contract.json"));
    let mut contract = document(root.path(), "contract");
    contract["operation"] = json!("review");
    fs::write(
        path,
        serde_json::to_vec(&contract).expect("initial fixture Contract"),
    )
    .expect("synthetic initial operation");
    explicit_repository_policy(root.path(), "review", "single_authorized_human");
    owner_review(root.path());
    let before = state_bytes(root.path());
    let r = request(
        root.path(),
        "actual-operation-human",
        json!([
            {"path":"/verification","operation":"add","value":"git status --porcelain"}
        ]),
    );
    assert!(
        amend(root.path(), &r).is_err(),
        "actual review operation requires human review"
    );
    assert_eq!(state_bytes(root.path()), before);
}

#[test]
fn stronger_current_policy_preserves_history_and_normal_fresh_human_recovery() {
    let root = fixture(true);
    owner_review(root.path());
    let r = request(
        root.path(),
        "before-policy-strengthening",
        json!([
            {"path":"/verification","operation":"add","value":"git status --porcelain"}
        ]),
    );
    amend(root.path(), &r).expect("originally permitted delegated change");
    let journal = read_work_item_contract_amendments(root.path(), WI).expect("original history");
    let original_owner = fs::read(
        root.path()
            .join(format!(".ai/decisions/{WI}.preflight-review.json")),
    )
    .expect("original owner bytes");
    explicit_repository_policy(root.path(), "modify_source", "single_authorized_human");
    assert_eq!(
        read_work_item_contract_amendments(root.path(), WI)
            .expect("policy strengthening cannot invalidate legitimate historical bytes"),
        journal
    );
    let result = preflight(root.path());
    assert!(
        result.decision.human_decision_request.is_some(),
        "current stricter policy needs fresh human review"
    );
    let before = state_bytes(root.path());
    let r = request(
        root.path(),
        "after-policy-strengthening",
        json!([
            {"path":"/verification","operation":"add","value":"git diff --check"}
        ]),
    );
    assert!(
        amend(root.path(), &r).is_err(),
        "current stronger policy denies a new agent amendment"
    );
    assert_eq!(state_bytes(root.path()), before);
    owner_review(root.path());
    assert!(
        preflight(root.path())
            .decision
            .human_decision_request
            .is_none(),
        "normal fresh human review can recover"
    );
    let mut ordinary = r;
    ordinary.development_review = None;
    assert_eq!(
        amend(root.path(), &ordinary).expect("ordinary human review path remains available")["policyReviewRequired"],
        true
    );
    assert!(
        preflight(root.path())
            .decision
            .human_decision_request
            .is_some()
    );
    owner_review(root.path());
    assert!(
        preflight(root.path())
            .decision
            .human_decision_request
            .is_none()
    );
    assert_eq!(
        fs::read(
            root.path()
                .join(format!(".ai/decisions/{WI}.preflight-review.json"))
        )
        .expect("retained owner bytes"),
        original_owner
    );
    assert_eq!(
        read_work_item_contract_amendments(root.path(), WI)
            .expect("recovered complete native history")
            .len(),
        2
    );
}
