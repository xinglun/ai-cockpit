//! Native source-observation business regressions migrated from the external
//! lifecycle_order suite. Simulated human decisions use only the private test
//! port; none of these fixtures authenticates a real person or enables a
//! production authority factory.
#![cfg(test)]

use crate as cockpit_repository;
use crate::review_authority::pipeline_tests::TestAuthority;
use crate::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item, attach, checkpoint_work_item, finish_work_item, preflight_work_item,
    preflight_work_item_with_runtime, record_verification, record_verification_with_runtime,
    run_repository_verification, start_work_item_with_options,
    work_item_status_snapshot_with_runtime,
};
use cockpit_core::{DecisionState, Digest};
use cockpit_protocol::RuntimeContext;
use cockpit_protocol::review_authority::HumanReviewOperation;
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
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit-test@example.invalid",
                "commit",
                "--allow-empty",
                "-m",
                "fixture baseline",
            ])
            .current_dir(directory.path())
            .status()
            .expect("git baseline commit")
            .success()
    );
    attach(directory.path()).expect("attach");
    directory
}

fn commit_fixture_source(path: &std::path::Path, source: &str, message: &str) {
    assert!(
        Command::new("git")
            .args(["add", "--", source])
            .current_dir(path)
            .status()
            .expect("git add fixture source")
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
                message,
            ])
            .current_dir(path)
            .status()
            .expect("git commit fixture source")
            .success()
    );
}

fn contract(path: &std::path::Path, id: &str) -> std::path::PathBuf {
    path.join(".ai/work-items/active")
        .join(format!("{id}.contract.json"))
}

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "0.2.91-test".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime-bound-lifecycle"),
    }
}

const OBS_ID: &str = "WI-OBSERVATION-NATIVE";
const OBS_LEDGERS: [&str; 3] = [
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/reference/reference-parity.ja.md",
];

fn source_observation_json(path: impl AsRef<std::path::Path>) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn source_observation_git(root: &std::path::Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn source_observation_fixture(
    authority: &TestAuthority,
    changed: usize,
) -> (tempfile::TempDir, RepositoryVerificationRequest) {
    source_observation_fixture_with_review(authority, changed, false)
}

fn source_observation_fixture_with_review(
    authority: &TestAuthority,
    changed: usize,
    finite_review: bool,
) -> (tempfile::TempDir, RepositoryVerificationRequest) {
    use cockpit_repository::{amend_work_item_contract, start_work_item_with_options_and_runtime};
    let directory = repository();
    let root = directory.path();
    fs::write(root.join("source.rs"), "fn base() {}\n").unwrap();
    fs::create_dir_all(root.join("docs/reference")).unwrap();
    fs::create_dir_all(root.join("docs/work-items")).unwrap();
    for path in OBS_LEDGERS {
        fs::write(root.join(path), format!("| {OBS_ID} | in_progress |\n")).unwrap();
    }
    for suffix in ["", ".zh-CN", ".ja"] {
        fs::write(root.join(format!("docs/work-items/{OBS_ID}{suffix}.md")), format!("---\nworkItemId: {OBS_ID}\nlastVerifiedBy: {OBS_ID}\nstatus: in_progress\n---\nSynthetic native fixture.\n")).unwrap();
    }
    source_observation_git(root, &["add", "source.rs", "docs"]);
    source_observation_git(
        root,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-qm",
            "repository base B",
        ],
    );
    let base = source_observation_git(root, &["rev-parse", "HEAD"]);
    source_observation_git(root, &["branch", "-M", "main"]);
    source_observation_git(
        root,
        &[
            "remote",
            "add",
            "origin",
            "https://example.invalid/synthetic.git",
        ],
    );
    source_observation_git(root, &["update-ref", "refs/remotes/origin/main", &base]);
    source_observation_git(
        root,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    source_observation_git(root, &["checkout", "-qb", "observation-fixture"]);
    // A real synthetic predecessor lifecycle supplies the native recovery
    // exception; no branch-entry check or successor authority is bypassed.
    let predecessor = "WI-OBSERVATION-PREDECESSOR";
    start_work_item_with_options(
        root,
        predecessor,
        "synthetic predecessor",
        "preserve native ancestry",
        &["source.rs".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["bounded predecessor".into()],
            ..Default::default()
        },
    )
    .unwrap();
    preflight_work_item(root, &contract(root, predecessor)).unwrap();
    authority.approve(
        root,
        predecessor,
        HumanReviewOperation::PreflightReview,
        &serde_json::json!({"decision": "confirm_review"}),
    );
    checkpoint_work_item(root, predecessor).unwrap();
    let run = run_repository_verification(
        root,
        &RepositoryVerificationRequest {
            node_id: "predecessor-check".into(),
            program: "true".into(),
            args: vec![],
            scope: vec!["source.rs".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime().runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .unwrap();
    record_verification_with_runtime(
        root,
        predecessor,
        &serde_json::to_value(run.receipt).unwrap(),
        &runtime(),
        &run.final_snapshot,
    )
    .unwrap();
    finish_work_item(root, predecessor).unwrap();
    archive_work_item(root, predecessor).unwrap();
    fs::write(
        root.join("source.rs"),
        "fn base() {}\nfn inherited_predecessor() {}\n",
    )
    .unwrap();
    commit_fixture_source(root, "source.rs", "inherited predecessor S");
    let archive = root.join(".ai/work-items/archive");
    let predecessor_contract =
        source_observation_json(archive.join(format!("{predecessor}.contract.json")));
    let predecessor_summary =
        source_observation_json(archive.join(format!("{predecessor}.summary.json")));
    let predecessor_outcome =
        source_observation_json(archive.join(format!("{predecessor}.outcome.json")));
    let recovery = serde_json::json!({
        "schemaVersion":1,"decisionId":"work-item-recovery","decision":"successor","workItemId":predecessor,"repositoryId":cockpit_repository::repository_id(root),"predecessorWorkItemId":predecessor,
        "predecessorContractDigest":cockpit_protocol::digest_json(&predecessor_contract).unwrap(),"predecessorSummaryDigest":cockpit_protocol::digest_json(&predecessor_summary).unwrap(),"predecessorOutcomeDigest":cockpit_protocol::digest_json(&predecessor_outcome).unwrap(),"predecessorEventsDigest":Digest::sha256_bytes(&fs::read(archive.join(format!("{predecessor}.events.jsonl"))).unwrap()),
        "successorWorkItemId":OBS_ID,"runtimeVersion":runtime().runtime_version,"runtimeDigest":runtime().runtime_digest,"actor":"human:synthetic-fixture","authoritySource":"synthetic test only","reason":"Exercise native successor source responsibility","evidenceRefs":[format!(".ai/work-items/archive/{predecessor}.outcome.json")],"policyRefs":[],"decidedAt":"2026-10-20T00:00:00Z","resumeCondition":"continue bounded synthetic successor"
    });
    authority.approve(
        root,
        predecessor,
        HumanReviewOperation::Recovery,
        &serde_json::json!({"receipt": recovery, "runtime": runtime()}),
    );
    cockpit_repository::record_recovery_decision(root, predecessor, &recovery, &runtime()).unwrap();
    let path = contract(root, OBS_ID);
    let before = source_observation_json(&path);
    let scope = OBS_LEDGERS
        .iter()
        .copied()
        .chain(["source.rs", ".ai/**"])
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut changes = serde_json::json!([
        {"path":"/intent","operation":"set","value":"Observe actual native delta"},
        {"path":"/goal","operation":"set","value":"Checks see committed responsibility"},
        {"path":"/scope","operation":"replace","value":scope},
        {"path":"/risk","operation":"set","value":"normal"},
        {"path":"/authority","operation":"set","value":"authorized"},
        {"path":"/acceptanceCriteria","operation":"replace","value":["A1: native origin remains bounded"]}
    ]);
    if finite_review {
        changes.as_array_mut().unwrap().push(serde_json::json!({"path":"/governanceProfile","operation":"set","value":{"materialInspectionReview":{
        "schemaVersion":1,"permittedUnknown":"repository_material_inspection_unavailable","permittedCause":"readable_committed_rust_syntax_unknown","assurance":"self_declared","reviewerActor":"agent:source-observation-fixture","authoritySource":"synthetic-fixture:finite-risk-only","acceptResidualRisk":true
    }}}));
    }
    amend_work_item_contract(root, OBS_ID, &serde_json::json!({
        "schemaVersion":1,"changeId":"synthetic-native-origin","expectedContractDigest":cockpit_protocol::digest_json(&before).unwrap(),"reason":"Synthetic test fixture only; no real authority.",
        "changes":changes
    }), "").unwrap();
    start_work_item_with_options_and_runtime(
        root,
        OBS_ID,
        "Observe actual native delta",
        "Checks see committed responsibility",
        &scope,
        &WorkItemStartOptions {
            risk: "normal".into(),
            authority: "authorized".into(),
            acceptance_criteria: vec!["A1: native origin remains bounded".into()],
            ..Default::default()
        },
        &runtime(),
    )
    .unwrap();
    for path in OBS_LEDGERS.iter().take(changed) {
        let mut text = fs::read_to_string(root.join(path)).unwrap();
        text.push_str("\nsuccessor hunk\n");
        fs::write(root.join(path), text).unwrap();
    }
    if changed != 0 {
        source_observation_git(root, &["add", "docs"]);
        source_observation_git(
            root,
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "-qm",
                "successor delta H",
            ],
        );
    }
    preflight_work_item_with_runtime(root, &path, &runtime()).unwrap();
    authority.approve(
        root,
        OBS_ID,
        HumanReviewOperation::PreflightReview,
        &serde_json::json!({"decision": "confirm_review"}),
    );
    cockpit_repository::record_work_item_governance_controls_with_runtime(root, OBS_ID, &serde_json::json!({
        "acceptanceEvidence":[{"acceptanceId":"A1","evidence":[{"type":"test","path":"source.rs","locator":"inherited_predecessor","verification":"passed"}]}],
        "intentAlignment":{"state":"resolved","evidence":["source.rs"]}
    }), &runtime()).unwrap();
    checkpoint_work_item(root, OBS_ID).unwrap();
    (
        directory,
        RepositoryVerificationRequest {
            node_id: "project-command-0".into(),
            program: "true".into(),
            args: vec![],
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime().runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: Some(OBS_ID.into()),
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
}

fn source_observation_prepare(
    root: &std::path::Path,
    request: &RepositoryVerificationRequest,
) -> cockpit_repository::PreparedWorkItemVerification {
    let binding =
        cockpit_repository::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
    cockpit_repository::prepare_work_item_verification(binding, std::slice::from_ref(request))
        .unwrap()
}

fn source_observation_receipt(
    root: &std::path::Path,
    request: &RepositoryVerificationRequest,
) -> (serde_json::Value, cockpit_git::RepositorySnapshot) {
    let run = run_repository_verification(root, request).unwrap();
    let mut receipt = serde_json::to_value(&run.receipt).unwrap();
    receipt["runtimeVersion"] = runtime().runtime_version.into();
    receipt["runtimeDigest"] = runtime().runtime_digest.to_string().into();
    (receipt, run.final_snapshot)
}

fn source_observation_retry(authority: &TestAuthority, root: &std::path::Path) {
    let active = root.join(".ai/work-items/active");
    let digest = |suffix| {
        cockpit_protocol::digest_json(&source_observation_json(
            active.join(format!("{OBS_ID}.{suffix}")),
        ))
        .unwrap()
    };
    let contract = source_observation_json(active.join(format!("{OBS_ID}.contract.json")));
    let recovery = serde_json::json!({
        "schemaVersion":1,"decisionId":"work-item-recovery","decision":"retry",
        "workItemId":OBS_ID,"repositoryId":contract["repositoryId"],
        "predecessorWorkItemId":OBS_ID,
        "predecessorContractDigest":digest("contract.json"),
        "predecessorSummaryDigest":digest("summary.json"),
        "predecessorOutcomeDigest":digest("outcome.json"),
        "predecessorEventsDigest":Digest::sha256_bytes(&fs::read(active.join(format!("{OBS_ID}.events.jsonl"))).unwrap()),
        "runtimeVersion":runtime().runtime_version,"runtimeDigest":runtime().runtime_digest,
        "actor":"agent:synthetic-retry-fixture","authoritySource":"synthetic-fixture:one-native-retry",
        "reason":"Exercise the exact current failure's one replacement verification.",
        "evidenceRefs":[],"policyRefs":[],"decidedAt":chrono::Utc::now().to_rfc3339(),
        "resumeCondition":"one replacement verification succeeds"
    });
    authority.approve(
        root,
        OBS_ID,
        HumanReviewOperation::Recovery,
        &serde_json::json!({"receipt": recovery, "runtime": runtime()}),
    );
    cockpit_repository::record_recovery_decision(root, OBS_ID, &recovery, &runtime()).unwrap();
}

#[test]
fn source_change_observation_stale_review_can_refresh_but_cannot_execute() {
    let authority = TestAuthority::install();
    let (directory, request) = source_observation_fixture(&authority, 3);
    let root = directory.path();
    let contract_before = fs::read(contract(root, OBS_ID)).unwrap();
    let old_review = crate::review_authority::current_preflight_review(root, OBS_ID).unwrap();
    let old_receipt = old_review.receipt().clone();
    let old_mirror_path = root.join(format!(
        ".ai/decisions/{OBS_ID}.human-approval.{}.json",
        old_review
            .raw_digest()
            .as_str()
            .trim_start_matches("sha256:")
    ));
    let old_mirror_bytes = fs::read(&old_mirror_path).unwrap();
    assert!(
        !work_item_status_snapshot_with_runtime(root, OBS_ID, &runtime())
            .unwrap()
            .human_decision_required
    );
    let unchanged =
        cockpit_repository::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
    assert!(
        cockpit_repository::prepare_work_item_verification(
            unchanged,
            std::slice::from_ref(&request)
        )
        .is_ok(),
        "the same C/P/R approval remains usable"
    );
    fs::write(root.join("source.rs"), "fn changed_after_review() {}\n").unwrap();
    commit_fixture_source(root, "source.rs", "change source after exact approval");
    assert_eq!(
        fs::read(contract(root, OBS_ID)).unwrap(),
        contract_before,
        "the source change leaves C unchanged"
    );
    assert_ne!(
        cockpit_repository::current_review_authority_state(root, OBS_ID).unwrap(),
        cockpit_protocol::review_authority::CurrentReviewAuthorityState::Active
    );
    let status = work_item_status_snapshot_with_runtime(root, OBS_ID, &runtime()).unwrap();
    assert!(
        status
            .safe_actions
            .iter()
            .any(|action| action == "run_preflight"),
        "{status:#?}"
    );
    preflight_work_item_with_runtime(root, &contract(root, OBS_ID), &runtime()).unwrap();
    assert!(
        work_item_status_snapshot_with_runtime(root, OBS_ID, &runtime())
            .unwrap()
            .human_decision_required,
        "new R requires a new exact approval despite unchanged C"
    );
    assert_eq!(fs::read(&old_mirror_path).unwrap(), old_mirror_bytes);
    assert!(
        crate::review_authority::historical_preflight_reviews(root, OBS_ID)
            .unwrap()
            .iter()
            .any(|proof| proof.receipt() == &old_receipt),
        "stale current permission must not erase the original activated decision fact"
    );
    assert_ne!(
        cockpit_repository::current_review_authority_state(root, OBS_ID).unwrap(),
        cockpit_protocol::review_authority::CurrentReviewAuthorityState::Active
    );
    let snapshot = cockpit_git::GitRepository::discover(root)
        .unwrap()
        .snapshot()
        .unwrap();
    assert!(
        cockpit_repository::require_verification_preconditions(root, OBS_ID, &runtime(), &snapshot)
            .is_err()
    );
    let binding =
        cockpit_repository::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
    assert!(
        cockpit_repository::prepare_work_item_verification(binding, std::slice::from_ref(&request))
            .is_err()
    );
    let fresh_review = authority.approve(
        root,
        OBS_ID,
        HumanReviewOperation::PreflightReview,
        &serde_json::json!({"decision":"confirm_review"}),
    );
    assert_eq!(
        fresh_review.binding.contract_digest,
        old_receipt.request.binding.contract_digest
    );
    assert_ne!(
        fresh_review.binding.repository_snapshot_digest,
        old_receipt.request.binding.repository_snapshot_digest
    );
    let binding =
        cockpit_repository::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
    assert!(cockpit_repository::prepare_work_item_verification(binding, &[request]).is_ok());
    assert_eq!(fs::read(&old_mirror_path).unwrap(), old_mirror_bytes);
    assert!(
        crate::review_authority::historical_preflight_reviews(root, OBS_ID)
            .unwrap()
            .iter()
            .any(|proof| proof.receipt() == &old_receipt)
    );
}

#[test]
fn source_change_observation_refresh_rejects_tampered_checkpoint_binding() {
    for field in [
        "checkpointContractDigest",
        "preflightContractDigest",
        "checkpointAt",
        "checkpointEvidence",
    ] {
        let authority = TestAuthority::install();
        let (directory, _) = source_observation_fixture(&authority, 3);
        let root = directory.path();
        fs::write(root.join("source.rs"), "fn changed_after_review() {}\n").unwrap();
        commit_fixture_source(root, "source.rs", "changed source with tampered checkpoint");
        let path = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
        let mut summary = source_observation_json(&path);
        let foreign = serde_json::json!(Digest::sha256_bytes(b"foreign checkpoint binding"));
        if field == "checkpointEvidence" {
            summary[field] = serde_json::Value::Null;
        } else {
            summary[field] = foreign;
        }
        fs::write(&path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
        let original = fs::read(&path).unwrap();
        assert!(
            preflight_work_item_with_runtime(root, &contract(root, OBS_ID), &runtime()).is_err(),
            "{field}"
        );
        assert_eq!(original, fs::read(path).unwrap(), "{field}");
    }
}

#[cfg(unix)]
#[test]
fn source_change_observation_r1_completion_does_not_extend_r0_approval() {
    r1_completion_preserves_r0_boundary(false);
}

#[cfg(unix)]
#[test]
fn source_change_observation_historical_review_rejects_r1_with_old_r0_summary() {
    r1_completion_preserves_r0_boundary(true);
}

#[cfg(unix)]
fn r1_completion_preserves_r0_boundary(check_history: bool) {
    let authority = TestAuthority::install();
    let (directory, mut request) = source_observation_fixture(&authority, 3);
    let root = directory.path();
    let summary_path = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
    request.program = "sh".into();
    request.args = vec!["-c".into(), "printf 'fn changed_during_execution() {}\\n' > source.rs; git add source.rs; git -c user.name=Fixture -c user.email=fixture@example.invalid commit -qm actual-r1-change".into()];
    let prepared = source_observation_prepare(root, &request);
    let r0_summary = source_observation_json(&summary_path);
    let original = r0_summary["sourceChangeObservation"].clone();
    let (receipt, snapshot) = source_observation_receipt(root, &request);
    cockpit_repository::record_prepared_work_item_verification(&prepared, &receipt, &snapshot)
        .unwrap();
    let summary = source_observation_json(&summary_path);
    assert_eq!(summary["sourceChangeObservation"], original);
    assert_eq!(
        summary["preflightRepositorySnapshotDigest"],
        serde_json::json!(cockpit_repository::snapshot_digest(&snapshot).unwrap())
    );
    assert_eq!(summary["verification"][0]["result"], "passed");
    assert_ne!(
        cockpit_repository::current_review_authority_state(root, OBS_ID).unwrap(),
        cockpit_protocol::review_authority::CurrentReviewAuthorityState::Active
    );
    assert_ne!(summary["preflightState"], "green");
    let status = work_item_status_snapshot_with_runtime(root, OBS_ID, &runtime()).unwrap();
    assert!(status.human_decision_required);
    assert!(!status.safe_actions.iter().any(|a| a == "run_verification"));
    if check_history {
        let mut changed = summary;
        changed["preflightDecisionDigest"] = r0_summary["preflightDecisionDigest"].clone();
        changed["preflightRepositorySnapshotDigest"] =
            r0_summary["preflightRepositorySnapshotDigest"].clone();
        fs::write(&summary_path, serde_json::to_vec_pretty(&changed).unwrap()).unwrap();
        source_observation_git(root, &["add", ".ai"]);
        source_observation_git(
            root,
            &["commit", "-qm", "synthetic stale R0 summary at actual R1"],
        );
        let revision = source_observation_git(root, &["rev-parse", "HEAD"]);
        let error = crate::resource_lifecycle::captured_preflight_review_for_test(
            root,
            OBS_ID,
            &revision,
            &chrono::Utc::now().to_rfc3339(),
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("captured_preflight_review_snapshot_mismatch"),
            "{error}"
        );
    }
}

#[test]
fn source_change_observation_failed_finish_without_retry_does_not_spawn() {
    let authority = TestAuthority::install();
    let (directory, mut request) = source_observation_fixture(&authority, 3);
    let root = directory.path();
    cockpit_repository::finish_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap_err();
    let paths = [
        format!(".ai/work-items/active/{OBS_ID}.summary.json"),
        format!(".ai/work-items/active/{OBS_ID}.outcome.json"),
        format!(".ai/work-items/active/{OBS_ID}.events.jsonl"),
        format!(".ai/evidence/{OBS_ID}.verification.json"),
    ];
    let before = paths.each_ref().map(|p| fs::read(root.join(p)).ok());
    #[cfg(windows)]
    let (program, args) = (
        "cmd",
        vec![
            "/C".into(),
            "echo spawned > .unapproved-verification-spawned".into(),
        ],
    );
    #[cfg(not(windows))]
    let (program, args) = (
        "sh",
        vec![
            "-c".into(),
            "printf spawned > .unapproved-verification-spawned".into(),
        ],
    );
    request.program = program.into();
    request.args = args;
    let snapshot = cockpit_git::GitRepository::discover(root)
        .unwrap()
        .snapshot()
        .unwrap();
    let admission =
        cockpit_repository::require_verification_preconditions(root, OBS_ID, &runtime(), &snapshot);
    if admission.is_ok() {
        run_repository_verification(root, &request).unwrap();
    }
    assert!(
        admission.is_err(),
        "ordinary failed finish needs an explicit retry"
    );
    assert!(!root.join(".unapproved-verification-spawned").exists());
    assert_eq!(
        before,
        paths.each_ref().map(|p| fs::read(root.join(p)).ok())
    );
}

#[test]
fn source_change_observation_native_retry_preserves_latest_failure_basis() {
    let authority = TestAuthority::install();
    let (directory, request) = source_observation_fixture(&authority, 3);
    let root = directory.path();
    // Two real failures ensure the replacement binds the latest one.
    for _ in 0..2 {
        cockpit_repository::finish_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap_err();
    }
    let outcome =
        source_observation_json(root.join(format!(".ai/work-items/active/{OBS_ID}.outcome.json")));
    let events_path = root.join(format!(".ai/work-items/active/{OBS_ID}.events.jsonl"));
    let events: Vec<cockpit_protocol::TaskOutcomeEvent> = fs::read_to_string(&events_path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let last = events
        .iter()
        .rposition(|event| event.event_type == "blocked")
        .unwrap();
    source_observation_retry(&authority, root);
    let prepared = source_observation_prepare(root, &request);
    let (receipt, snapshot) = source_observation_receipt(root, &request);
    cockpit_repository::record_prepared_work_item_verification(&prepared, &receipt, &snapshot)
        .unwrap();
    let summary_path = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
    let summary = source_observation_json(&summary_path);
    let evidence =
        source_observation_json(root.join(format!(".ai/evidence/{OBS_ID}.verification.json")));
    let basis = &summary["verificationRecoveryBasis"];
    assert_eq!(basis["schemaVersion"], 1);
    assert_eq!(
        basis["verificationEvidenceDigest"],
        cockpit_protocol::digest_json(&evidence)
            .unwrap()
            .to_string()
    );
    assert_eq!(
        basis["failure"]["outcomeDigest"],
        cockpit_protocol::digest_json(&outcome).unwrap().to_string()
    );
    assert_eq!(basis["failure"]["blockedEventId"], events[last].event_id);
    assert_eq!(
        basis["failure"]["blockedEventPrefixDigest"],
        cockpit_protocol::digest_json(&serde_json::json!(&events[..=last]))
            .unwrap()
            .to_string()
    );
    assert_eq!(
        cockpit_repository::outcome_v2_with_runtime(root, OBS_ID, &runtime())
            .unwrap()
            .decision_state,
        Some(DecisionState::Green)
    );
    // The successful recorder replaces the blocked Outcome. Restore that
    // exact real failure only in this negative fixture so causal proof is
    // required again; a completed green Outcome needs no recovery bypass.
    let outcome_path = root.join(format!(".ai/work-items/active/{OBS_ID}.outcome.json"));
    let completed_outcome = fs::read(&outcome_path).unwrap();
    fs::write(&outcome_path, serde_json::to_vec_pretty(&outcome).unwrap()).unwrap();
    assert_eq!(
        cockpit_repository::outcome_v2_with_runtime(root, OBS_ID, &runtime())
            .unwrap()
            .decision_state,
        Some(DecisionState::Green)
    );
    for (pointer, replacement) in [
        (
            "/verificationRecoveryBasis/schemaVersion",
            serde_json::json!(2),
        ),
        (
            "/verificationRecoveryBasis/verificationEvidenceDigest",
            serde_json::json!(Digest::sha256_bytes(b"wrong evidence")),
        ),
        (
            "/verificationRecoveryBasis/failure/blockedEventPrefixDigest",
            serde_json::json!(Digest::sha256_bytes(b"wrong prefix")),
        ),
    ] {
        let mut tampered = summary.clone();
        *tampered.pointer_mut(pointer).unwrap() = replacement;
        fs::write(&summary_path, serde_json::to_vec_pretty(&tampered).unwrap()).unwrap();
        assert_eq!(
            cockpit_repository::outcome_v2_with_runtime(root, OBS_ID, &runtime())
                .unwrap()
                .decision_state,
            Some(DecisionState::Red),
            "{pointer}"
        );
    }
    fs::write(&summary_path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
    fs::write(outcome_path, completed_outcome).unwrap();
    assert_ne!(summary["recoveryRetryPending"], true);
    assert!(summary["recoveryRetryConsumed"].is_object());
}

#[test]
fn source_change_observation_validated_origin_and_real_gate_ownership() {
    let authority = TestAuthority::install();
    for changed in [0, 2, 3] {
        let (directory, request) = source_observation_fixture(&authority, changed);
        let root = directory.path();
        source_observation_prepare(root, &request);
        let summary = source_observation_json(
            root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json")),
        );
        assert!(
            !summary["changedPaths"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p == "source.rs"),
            "inherited predecessor hunk is not ours"
        );
        assert_ne!(
            summary["sourceChangeObservation"]["sourceFacts"]["origin"]["repositoryBaseRevision"],
            summary["sourceChangeObservation"]["sourceFacts"]["origin"]["responsibilityRevision"]
        );
        // Invoke the unchanged real gate on the actual native fixture
        // records. No Contract or Summary ownership facts are hand-filled.
        let gate = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/ci/governance_integrity_gate.py");
        let script = "import runpy,sys,pathlib; m=runpy.run_path(sys.argv[1]); ok=m['recovery_successor_owns_parity_projection'](pathlib.Path(sys.argv[2]),'WI-OBSERVATION-PREDECESSOR',{'successorWorkItemId':'WI-OBSERVATION-NATIVE'}); assert ok==(sys.argv[3]=='3'),ok";
        let output = Command::new("python3")
            .args(["-c", script])
            .arg(gate)
            .arg(root)
            .arg(changed.to_string())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "real gate: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn source_change_observation_same_path_inherited_and_successor_hunks() {
    let authority = TestAuthority::install();
    let (directory, request) = source_observation_fixture(&authority, 3);
    let root = directory.path();
    fs::write(
        root.join("source.rs"),
        "fn base() {}\nfn inherited_predecessor() {}\nfn successor() {}\n",
    )
    .unwrap();
    commit_fixture_source(root, "source.rs", "same path successor hunk");
    preflight_work_item_with_runtime(root, &contract(root, OBS_ID), &runtime()).unwrap();
    authority.approve(
        root,
        OBS_ID,
        HumanReviewOperation::PreflightReview,
        &serde_json::json!({"decision": "confirm_review"}),
    );
    source_observation_prepare(root, &request);
    let summary =
        source_observation_json(root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json")));
    assert!(
        summary["changedPaths"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p == "source.rs")
    );
    assert!(
        !summary["sourceChangeObservation"]["sourceFacts"]["origin"]["activationPreparedDigest"]
            .is_null()
    );
}

#[test]
fn source_change_observation_rejects_pending_foreign_tampered_and_missing_origin() {
    let authority = TestAuthority::install();
    for kind in ["pending", "foreign", "tampered", "missing"] {
        let (directory, request) = source_observation_fixture(&authority, 3);
        let root = directory.path();
        let binding =
            cockpit_repository::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
        let dir = root.join(format!(".ai/evidence/{OBS_ID}.activation-transition"));
        if kind == "pending" {
            fs::remove_file(dir.join("committed.json")).unwrap();
        } else if kind == "missing" {
            fs::remove_dir_all(&dir).unwrap();
        } else {
            let p = dir.join("prepared.json");
            let mut value = source_observation_json(&p);
            if kind == "foreign" {
                value["record"]["repositoryId"] = "sha256:foreign".into();
            } else {
                value["recordDigest"] = "sha256:tampered".into();
            }
            fs::write(p, serde_json::to_vec(&value).unwrap()).unwrap();
        }
        let before =
            fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"))).unwrap();
        assert!(
            cockpit_repository::prepare_work_item_verification(
                binding,
                std::slice::from_ref(&request)
            )
            .is_err(),
            "{kind}"
        );
        assert_eq!(
            before,
            fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"))).unwrap()
        );
    }
}

#[test]
fn source_change_observation_lock_wait_rejects_head_contract_and_journal_drift() {
    let authority = TestAuthority::install();
    for kind in ["head", "contract", "journal"] {
        let (directory, request) = source_observation_fixture(&authority, 3);
        let root = directory.path();
        let binding =
            cockpit_repository::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(root.join(format!(".ai/locks/{OBS_ID}.lifecycle.lock")))
            .unwrap();
        file.lock().unwrap();
        let before =
            fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"))).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let shared_authority = authority.share();
        let waiter = std::thread::spawn(move || {
            let _authority = shared_authority.install();
            tx.send(()).unwrap();
            cockpit_repository::prepare_work_item_verification(binding, &[request]).is_err()
        });
        rx.recv().unwrap();
        match kind {
            "head" => {
                source_observation_git(
                    root,
                    &[
                        "-c",
                        "user.name=Test",
                        "-c",
                        "user.email=test@example.invalid",
                        "commit",
                        "--allow-empty",
                        "-qm",
                        "HEAD drift while locked",
                    ],
                );
            }
            "contract" => {
                let p = contract(root, OBS_ID);
                let mut c = source_observation_json(&p);
                c["goal"] = "tamper negative fixture".into();
                fs::write(p, serde_json::to_vec(&c).unwrap()).unwrap();
            }
            _ => {
                fs::write(
                    root.join(format!(
                        ".ai/evidence/{OBS_ID}.contract-amendments/00000002.prepared.json"
                    )),
                    b"{}",
                )
                .unwrap();
            }
        }
        file.unlock().unwrap();
        assert!(waiter.join().unwrap(), "{kind}");
        assert_eq!(
            before,
            fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"))).unwrap()
        );
    }
}

#[test]
fn source_change_observation_concurrent_plans_and_legacy_recorders_preserve_facts() {
    let authority = TestAuthority::install();
    let (directory, request) = source_observation_fixture(&authority, 3);
    let root = directory.path();
    let one = cockpit_repository::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
    let two = cockpit_repository::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let other = barrier.clone();
    let r = request.clone();
    let shared_authority = authority.share();
    let thread = std::thread::spawn(move || {
        let _authority = shared_authority.install();
        other.wait();
        cockpit_repository::prepare_work_item_verification(two, &[r])
    });
    barrier.wait();
    let first =
        cockpit_repository::prepare_work_item_verification(one, std::slice::from_ref(&request));
    let second = thread.join().unwrap();
    assert_ne!(first.is_ok(), second.is_ok(), "one stale plan must reject");
    let prepared = first.ok().or(second.ok()).unwrap();
    let path = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
    let before = fs::read(&path).unwrap();
    assert!(
        record_verification(
            root,
            OBS_ID,
            &serde_json::json!({"passed":true}),
            "legacy",
            &Digest::sha256_bytes(b"legacy")
        )
        .is_err()
    );
    let (receipt, snapshot) = source_observation_receipt(root, &request);
    assert!(
        record_verification_with_runtime(root, OBS_ID, &receipt, &runtime(), &snapshot).is_err()
    );
    assert_eq!(before, fs::read(&path).unwrap());
    cockpit_repository::record_prepared_work_item_verification(&prepared, &receipt, &snapshot)
        .unwrap();
    assert!(
        source_observation_json(&path)["changedPaths"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p == OBS_LEDGERS[2])
    );
}

#[test]
fn source_change_observation_execution_drift_and_completion_rollback_keep_start_facts() {
    let authority = TestAuthority::install();
    let (directory, request) = source_observation_fixture(&authority, 3);
    let root = directory.path();
    let prepared = source_observation_prepare(root, &request);
    let path = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
    let before = fs::read(&path).unwrap();
    let (receipt, snapshot) = source_observation_receipt(root, &request);
    fs::write(root.join("source.rs"), "source drift during execution\n").unwrap();
    assert!(
        cockpit_repository::record_prepared_work_item_verification(&prepared, &receipt, &snapshot)
            .is_err()
    );
    assert_eq!(before, fs::read(&path).unwrap());
    source_observation_git(root, &["checkout", "--", "source.rs"]);
    // Force failure after the evidence write, inside the completion
    // transaction. Its rollback baseline must contain the native start facts.
    let mut malformed = source_observation_json(&path);
    malformed["verification"] = serde_json::json!({});
    fs::write(&path, serde_json::to_vec(&malformed).unwrap()).unwrap();
    let before = fs::read(&path).unwrap();
    let error =
        cockpit_repository::record_prepared_work_item_verification(&prepared, &receipt, &snapshot)
            .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Summary.verification must be an array"),
        "{error}"
    );
    assert_eq!(before, fs::read(&path).unwrap());
    assert!(
        !root
            .join(format!(".ai/evidence/{OBS_ID}.verification.json"))
            .exists()
    );
}

#[test]
fn source_change_observation_failure_success_evidence_commit_finish_archive() {
    let authority = TestAuthority::install();
    let (directory, request) = source_observation_fixture(&authority, 3);
    let root = directory.path();
    let mut failed = request.clone();
    failed.program = "python3".into();
    failed.args = vec!["-c".into(), "raise SystemExit(7)".into()];
    let _prepared = source_observation_prepare(root, &failed);
    let (failed_receipt, snapshot) = source_observation_receipt(root, &failed);
    assert_eq!(failed_receipt["passed"], false);
    let attempt = cockpit_repository::persist_verification_attempt(
        root,
        OBS_ID,
        &[failed],
        &snapshot,
        &runtime(),
        "execution_failed",
        Some(("verification_execution", "synthetic expected first failure")),
        Some(&failed_receipt),
    )
    .unwrap();
    let attempt_path = root.join(attempt["path"].as_str().unwrap());
    let failure_bytes = fs::read(&attempt_path).unwrap();
    let prepared = source_observation_prepare(root, &request);
    let (receipt, snapshot) = source_observation_receipt(root, &request);
    assert_eq!(receipt["passed"], true);
    cockpit_repository::record_prepared_work_item_verification(&prepared, &receipt, &snapshot)
        .unwrap();
    let path = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
    let facts = source_observation_json(&path);
    for ledger in OBS_LEDGERS {
        assert!(
            facts["changedPaths"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p == ledger)
        );
    }
    let source_before = cockpit_repository::snapshot_digest(
        &cockpit_git::GitRepository::discover(root)
            .unwrap()
            .snapshot()
            .unwrap(),
    )
    .unwrap();
    source_observation_git(root, &["add", ".ai"]);
    source_observation_git(
        root,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-qm",
            "normal evidence-only commit",
        ],
    );
    let source_after = cockpit_repository::snapshot_digest(
        &cockpit_git::GitRepository::discover(root)
            .unwrap()
            .snapshot()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        source_before, source_after,
        "HEAD is strict only during execution"
    );
    let status = work_item_status_snapshot_with_runtime(root, OBS_ID, &runtime()).unwrap();
    assert!(!status.human_decision_required, "{status:#?}");
    cockpit_repository::finish_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap();
    cockpit_repository::archive_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap();
    assert_eq!(failure_bytes, fs::read(attempt_path).unwrap());
    let archived =
        source_observation_json(root.join(format!(".ai/work-items/archive/{OBS_ID}.summary.json")));
    assert!(
        archived["changedPaths"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p == OBS_LEDGERS[2])
    );
}

fn source_observation_material_unknown_fixture(
    authority: &TestAuthority,
) -> (tempfile::TempDir, RepositoryVerificationRequest) {
    source_observation_material_unknown_fixture_with_review(authority, false)
}

fn source_observation_material_unknown_fixture_with_review(
    authority: &TestAuthority,
    finite_review: bool,
) -> (tempfile::TempDir, RepositoryVerificationRequest) {
    let (directory, request) = material_unknown_unverified_fixture(authority, finite_review);
    let root = directory.path();
    let prepared = source_observation_prepare(root, &request);
    let (receipt, snapshot) = source_observation_receipt(root, &request);
    cockpit_repository::record_prepared_work_item_verification(&prepared, &receipt, &snapshot)
        .unwrap();
    let material = cockpit_repository::plan_work_item_material_review(root, OBS_ID).unwrap();
    assert!(
        material
            .entries
            .iter()
            .any(|entry| entry.path == "source.rs"
                && matches!(
                    entry.scanner_assessment,
                    crate::material_review::MaterialScannerAssessment::Unknown
                )),
        "this positive case requires actual nonempty source Unknown material"
    );
    assert_eq!(
        material.raw_unknown_codes,
        vec!["repository_material_inspection_unavailable"]
    );
    assert!(material.finding_codes.is_empty());
    (directory, request)
}

fn material_unknown_unverified_fixture(
    authority: &TestAuthority,
    finite_review: bool,
) -> (tempfile::TempDir, RepositoryVerificationRequest) {
    let (directory, request) = source_observation_fixture_with_review(authority, 3, finite_review);
    let root = directory.path();
    fs::write(root.join("source.rs"), "fn base() {}\nfn inherited_predecessor() {}\nfn material() -> String { let mut label = String::from(\"token\"); label.push_str(\"ization\"); label }\n").unwrap();
    commit_fixture_source(
        root,
        "source.rs",
        "readable committed syntax Unknown fixture",
    );
    preflight_work_item_with_runtime(root, &contract(root, OBS_ID), &runtime()).unwrap();
    authority.approve(
        root,
        OBS_ID,
        HumanReviewOperation::PreflightReview,
        &serde_json::json!({"decision": "confirm_review"}),
    );
    assert!(
        !root
            .join(format!(".ai/evidence/{OBS_ID}.verification.json"))
            .exists()
    );
    (directory, request)
}

#[test]
fn source_change_observation_material_review_before_typed_verification_finish_same_commit() {
    fn governance_bytes(
        root: &std::path::Path,
    ) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
        fn walk(
            path: &std::path::Path,
            bytes: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
        ) {
            if !path.exists() {
                return;
            }
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    walk(&entry.path(), bytes);
                } else {
                    bytes.insert(entry.path(), fs::read(entry.path()).unwrap());
                }
            }
        }
        let mut bytes = std::collections::BTreeMap::new();
        walk(&root.join(".ai"), &mut bytes);
        walk(&root.join(".git/.ai-cockpit"), &mut bytes);
        bytes
    }
    let authority = TestAuthority::install();
    let (directory, verification) = material_unknown_unverified_fixture(&authority, true);
    let root = directory.path();
    let material = cockpit_repository::plan_work_item_material_review(root, OBS_ID).unwrap();
    assert_eq!(
        material.raw_unknown_codes,
        vec!["repository_material_inspection_unavailable"]
    );
    assert!(material.finding_codes.is_empty());
    assert!(material.request_artifact.as_ref().unwrap().byte_length > 0);
    let input = serde_json::from_value(serde_json::json!({
        "schemaVersion":2,"decision":"accept_permitted_unknowns","requestDigest":material.request_digest,
        "requestArtifactDigest":material.request_artifact.as_ref().unwrap().digest,
        "requestArtifactByteLength":material.request_artifact.as_ref().unwrap().byte_length,
        "reviewerActor":"agent:source-observation-fixture","authoritySource":"synthetic-fixture:finite-risk-only","assurance":"self_declared",
        "evidenceRefs":[{"path":"synthetic-exact-review","digest":Digest::sha256_bytes(b"synthetic finite review evidence")}],
        "rationale":"Only this actual nonempty request is reviewed.","residualRisk":"Synthetic readable syntax risk only."
    })).unwrap();
    authority.approve(
        root,
        OBS_ID,
        HumanReviewOperation::MaterialReview,
        &serde_json::json!({"input":input,"runtime":runtime()}),
    );
    let prior = crate::human_review_binding(root, OBS_ID).unwrap();
    cockpit_repository::record_work_item_material_review(root, OBS_ID, &input, &runtime()).unwrap();
    let assessment =
        crate::preflight_work_item_with_runtime_report(root, &contract(root, OBS_ID), &runtime())
            .unwrap();
    assert_eq!(
        assessment.decision.review_state.as_deref(),
        Some("needs_human_confirmation"),
        "the returned projection must describe new P, never the old approval"
    );
    assert!(
        !assessment
            .decision
            .safe_actions
            .iter()
            .any(|action| action == "continue_to_checkpoint"),
        "refresh must not advertise authority inherited from old P"
    );
    let refreshed = crate::human_review_binding(root, OBS_ID).unwrap();
    assert_eq!(refreshed.contract_digest, prior.contract_digest);
    assert_eq!(
        refreshed.repository_snapshot_digest,
        prior.repository_snapshot_digest
    );
    assert_ne!(
        refreshed.preflight_decision_digest, prior.preflight_decision_digest,
        "material discharge changes P, without changing the approved source bytes"
    );
    let rejected = crate::review_authority::require_preflight_authority(root, OBS_ID)
        .unwrap_err()
        .to_string();
    assert!(rejected.contains("human_review_not_active"), "{rejected}");
    let before_rejected_prepare = governance_bytes(root);
    let pending = crate::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
    let rejected_prepare =
        crate::prepare_work_item_verification(pending, std::slice::from_ref(&verification))
            .err()
            .expect("the production pre-execution boundary must reject the old approval")
            .to_string();
    assert!(
        rejected_prepare
            .contains("current action admission rejected requested action \"run_verification\""),
        "wrong pre-execution rejection: {rejected_prepare}"
    );
    assert_eq!(
        governance_bytes(root),
        before_rejected_prepare,
        "rejected preparation publishes no execution generation or evidence"
    );
    let snapshot = cockpit_git::GitRepository::discover(root)
        .unwrap()
        .snapshot()
        .unwrap();
    let mut raw = crate::governance_decision_for_pre_execution_boundary(
        root,
        &crate::read_contract(&contract(root, OBS_ID)).unwrap(),
        &snapshot,
        Some(&runtime()),
        None,
    )
    .unwrap();
    crate::material_review::apply_material_review_gate_to_decision(
        root,
        &crate::read_contract(&contract(root, OBS_ID)).unwrap(),
        &root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json")),
        &mut raw,
    );
    assert_eq!(
        refreshed.preflight_decision_digest,
        Some(cockpit_protocol::digest_json(&serde_json::to_value(raw).unwrap()).unwrap()),
        "the refreshed P must equal the independently evaluated raw material-inclusive decision"
    );
    assert!(
        material
            .entries
            .iter()
            .any(|entry| entry.path == "source.rs"
                && matches!(
                    entry.scanner_assessment,
                    crate::material_review::MaterialScannerAssessment::Unknown
                )),
        "the historical fixture contains real source Unknown material"
    );
    authority.approve(
        root,
        OBS_ID,
        HumanReviewOperation::PreflightReview,
        &serde_json::json!({"decision":"confirm_review"}),
    );
    let reviewed = crate::human_review_binding(root, OBS_ID).unwrap();
    assert_eq!(
        crate::current_review_authority_state(root, OBS_ID).unwrap(),
        cockpit_protocol::review_authority::CurrentReviewAuthorityState::Active
    );
    source_observation_git(root, &["add", ".ai"]);
    source_observation_git(
        root,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-qm",
            "actual material review and exact start before verification",
        ],
    );
    let parent = source_observation_git(root, &["rev-parse", "HEAD"]);
    let evidence_path = format!(".ai/evidence/{OBS_ID}.verification.json");
    assert!(
        crate::git_text(root, &["show", &format!("{parent}:{evidence_path}")]).is_none(),
        "the historical recorder branch must supply the new verification"
    );
    let run = run_repository_verification(root, &verification).unwrap();
    record_verification_with_runtime(
        root,
        OBS_ID,
        &serde_json::to_value(&run.receipt).unwrap(),
        &runtime(),
        &run.final_snapshot,
    )
    .unwrap();
    assert_eq!(crate::human_review_binding(root, OBS_ID).unwrap(), reviewed);
    cockpit_repository::finish_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap();
    source_observation_git(root, &["add", ".ai"]);
    source_observation_git(
        root,
        &[
            "commit",
            "-qm",
            "actual typed verification and material finish together",
        ],
    );
    let finish = source_observation_git(root, &["rev-parse", "HEAD"]);
    cockpit_repository::archive_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap();
    source_observation_git(root, &["add", ".ai"]);
    source_observation_git(root, &["commit", "-qm", "actual material archive"]);
    let archive = source_observation_git(root, &["rev-parse", "HEAD"]);
    let before = governance_bytes(root);
    crate::resource_lifecycle::validate_layout_finish_semantics_for_test(
        root, OBS_ID, &parent, &finish, &archive,
    )
    .expect(
        "nonempty reviewed material and the new typed verification reconstruct the approved start",
    );
    assert_eq!(governance_bytes(root), before);
}

#[test]
fn source_change_observation_material_unknown_keeps_collection_but_blocks_finish() {
    let authority = TestAuthority::install();
    let (directory, _) = source_observation_material_unknown_fixture(&authority);
    let outcome =
        cockpit_repository::outcome_v2_with_runtime(directory.path(), OBS_ID, &runtime()).unwrap();
    assert_eq!(
        outcome.decision_state,
        Some(DecisionState::Yellow),
        "human Outcome must retain the canonical material boundary"
    );
    assert!(
        outcome
            .unknowns
            .iter()
            .any(|v| v == "repository_material_inspection_unavailable")
    );
    let status =
        work_item_status_snapshot_with_runtime(directory.path(), OBS_ID, &runtime()).unwrap();
    assert!(
        !status.safe_actions.iter().any(|a| a == "finish"),
        "unresolved canonical material cannot admit finish: {status:#?}"
    );
    assert!(
        status.safe_actions.iter().any(|a| a == "run_verification"),
        "material Unknown must not blanket-block legitimate verification: {status:#?}"
    );
    assert!(
        status
            .safe_actions
            .iter()
            .any(|a| a == "record_governance_controls")
    );
    assert!(
        status
            .unknowns
            .iter()
            .any(|v| v == "repository_material_inspection_unavailable")
    );
}

#[test]
fn source_change_observation_new_failure_cannot_hide_under_old_marker_or_equal_time() {
    let authority = TestAuthority::install();
    let (directory, _) = source_observation_material_unknown_fixture(&authority);
    let root = directory.path();
    let error =
        cockpit_repository::finish_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap_err();
    assert!(
        error.to_string().contains("canonical material Unknowns"),
        "{error}"
    );
    let summary_path = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
    let summary = source_observation_json(&summary_path);
    assert!(
        summary["verificationRecoveryReconciled"].is_string(),
        "fixture requires an older successful marker"
    );
    let original_failure =
        fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.outcome.json"))).unwrap();
    let outcome = cockpit_repository::outcome_v2_with_runtime(root, OBS_ID, &runtime()).unwrap();
    assert_eq!(
        outcome.decision_state,
        Some(DecisionState::Red),
        "later native failure must stay visible"
    );
    assert_eq!(outcome.failed_gate.as_deref(), Some("finish.lifecycle"));
    // Explicit legacy/ambiguous-time negative fixture. Source and real failure
    // stay untouched; assigning equal timestamps cannot create causal proof.
    let evidence =
        source_observation_json(root.join(format!(".ai/evidence/{OBS_ID}.verification.json")));
    let path = root.join(format!(".ai/work-items/active/{OBS_ID}.events.jsonl"));
    let mut events = fs::read_to_string(&path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    for event in &mut events {
        event["timestamp"] = evidence["createdAt"].clone();
    }
    fs::write(
        &path,
        events
            .iter()
            .map(|e| serde_json::to_string(e).unwrap() + "\n")
            .collect::<String>(),
    )
    .unwrap();
    let outcome = cockpit_repository::outcome_v2_with_runtime(root, OBS_ID, &runtime()).unwrap();
    assert_eq!(outcome.decision_state, Some(DecisionState::Red));
    assert_eq!(
        original_failure,
        fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.outcome.json"))).unwrap()
    );
}

#[test]
fn source_change_observation_material_review_evidence_commit_finishes_without_reverification() {
    material_review_evidence_commit_finishes(false);
}

#[test]
fn source_change_observation_material_finish_history_preserves_reviewed_p() {
    material_review_evidence_commit_finishes(true);
}

fn material_review_evidence_commit_finishes(capture_history: bool) {
    let authority = TestAuthority::install();
    let (directory, _) = source_observation_material_unknown_fixture_with_review(&authority, true);
    let root = directory.path();
    let evidence_path = root.join(format!(".ai/evidence/{OBS_ID}.verification.json"));
    let verified_bytes = fs::read(&evidence_path).unwrap();
    cockpit_repository::finish_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap_err();
    let status = work_item_status_snapshot_with_runtime(root, OBS_ID, &runtime()).unwrap();
    assert_eq!(
        status.verification, "verified",
        "software evidence remains current: {status:#?}"
    );
    assert!(
        status
            .safe_actions
            .iter()
            .any(|a| a == "record_material_review_decision"),
        "{status:#?}"
    );
    assert!(
        !status.human_decision_required,
        "fixture already has exact profile and Contract review: {status:#?}"
    );
    let failure_events =
        fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.events.jsonl"))).unwrap();
    let request = cockpit_repository::plan_work_item_material_review(root, OBS_ID).unwrap();
    assert!(request.finding_codes.is_empty());
    let input = serde_json::from_value(serde_json::json!({
        "schemaVersion":2,"decision":"accept_permitted_unknowns","requestDigest":request.request_digest,
        "requestArtifactDigest":request.request_artifact.as_ref().unwrap().digest,
        "requestArtifactByteLength":request.request_artifact.as_ref().unwrap().byte_length,
        "reviewerActor":"agent:source-observation-fixture","authoritySource":"synthetic-fixture:finite-risk-only","assurance":"self_declared",
        "evidenceRefs":[{"path":"synthetic-exact-review","digest":Digest::sha256_bytes(b"synthetic finite review evidence")}],
        "rationale":"Synthetic fixture reviews only its exact native request.","residualRisk":"Synthetic readable syntax risk only; no scanner completeness claim."
    })).unwrap();
    authority.approve(
        root,
        OBS_ID,
        HumanReviewOperation::MaterialReview,
        &serde_json::json!({"input": input, "runtime": runtime()}),
    );
    cockpit_repository::record_work_item_material_review(root, OBS_ID, &input, &runtime()).unwrap();
    source_observation_git(root, &["add", ".ai"]);
    source_observation_git(
        root,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-qm",
            "native review evidence only",
        ],
    );
    let history_parent =
        capture_history.then(|| source_observation_git(root, &["rev-parse", "HEAD"]));
    let reviewed_p =
        source_observation_json(root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json")))
            ["preflightDecisionDigest"]
            .clone();
    assert_eq!(
        verified_bytes,
        fs::read(&evidence_path).unwrap(),
        "no unchanged-source reverify"
    );
    let status = work_item_status_snapshot_with_runtime(root, OBS_ID, &runtime()).unwrap();
    assert!(
        status.safe_actions.iter().any(|a| a == "finish"),
        "{status:#?}"
    );
    cockpit_repository::finish_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap();
    assert_eq!(
        reviewed_p,
        source_observation_json(root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json")))
            ["preflightDecisionDigest"],
        "material discharge does not replace the original reviewed P"
    );
    let history_finish = capture_history.then(|| {
        source_observation_git(root, &["add", ".ai"]);
        source_observation_git(root, &["commit", "-qm", "native material finish"]);
        source_observation_git(root, &["rev-parse", "HEAD"])
    });
    let completed_events =
        fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.events.jsonl"))).unwrap();
    assert!(
        completed_events.starts_with(&failure_events),
        "finish preserves actual failure bytes"
    );
    cockpit_repository::archive_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap();
    assert_eq!(verified_bytes, fs::read(&evidence_path).unwrap());
    // Archive natively relocates active evidence refs; compare all old event
    // values after only that documented reference transformation.
    let archived_events =
        fs::read_to_string(root.join(format!(".ai/work-items/archive/{OBS_ID}.events.jsonl")))
            .unwrap();
    for (before, after) in String::from_utf8(failure_events)
        .unwrap()
        .lines()
        .zip(archived_events.lines())
    {
        let before: serde_json::Value = serde_json::from_str(&before.replace(
            &format!(".ai/work-items/active/{OBS_ID}"),
            &format!(".ai/work-items/archive/{OBS_ID}"),
        ))
        .unwrap();
        let after: serde_json::Value = serde_json::from_str(after).unwrap();
        assert_eq!(
            before, after,
            "archive preserves the failure event and only relocates refs"
        );
    }
    if let (Some(parent), Some(finish)) = (history_parent, history_finish) {
        source_observation_git(root, &["add", ".ai"]);
        source_observation_git(root, &["commit", "-qm", "native material archive"]);
        let archive = source_observation_git(root, &["rev-parse", "HEAD"]);
        crate::resource_lifecycle::validate_layout_finish_semantics_for_test(
            root, OBS_ID, &parent, &finish, &archive,
        )
        .unwrap();
        assert!(
            crate::resource_lifecycle::captured_preflight_review_for_test(
                root,
                OBS_ID,
                &parent,
                &chrono::Utc::now().to_rfc3339(),
            )
            .unwrap(),
            "the authenticated historical event is readable without CURRENT in the clone"
        );
        let error = crate::resource_lifecycle::captured_preflight_review_for_test(
            root,
            OBS_ID,
            &parent,
            "1970-01-01T00:00:00Z",
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("captured_preflight_review_source_unverified"),
            "{error}"
        );
        {
            let _unregistered_source = TestAuthority::install();
            let error = crate::resource_lifecycle::captured_preflight_review_for_test(
                root,
                OBS_ID,
                &parent,
                &chrono::Utc::now().to_rfc3339(),
            )
            .unwrap_err()
            .to_string();
            assert!(
                error.contains("captured_preflight_review_source_unverified"),
                "{error}"
            );
        }

        // The port still authenticates the original receipt, but it cannot
        // justify a different historical P merely because its JSON is present.
        source_observation_git(root, &["checkout", "--detach", "-q", &parent]);
        let summary_path = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
        let mut changed = source_observation_json(&summary_path);
        changed["preflightDecisionDigest"] =
            serde_json::json!(Digest::sha256_bytes(b"different captured P"));
        fs::write(&summary_path, serde_json::to_vec_pretty(&changed).unwrap()).unwrap();
        source_observation_git(root, &["add", ".ai"]);
        source_observation_git(root, &["commit", "-qm", "synthetic changed captured P"]);
        let wrong_parent = source_observation_git(root, &["rev-parse", "HEAD"]);
        let error = crate::resource_lifecycle::validate_layout_finish_semantics_for_test(
            root,
            OBS_ID,
            &wrong_parent,
            &finish,
            &archive,
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("captured_preflight_review_source_unverified"),
            "{error}"
        );

        source_observation_git(root, &["checkout", "--detach", "-q", &archive]);
        drop(authority);
        let error = crate::resource_lifecycle::validate_layout_finish_semantics_for_test(
            root, OBS_ID, &parent, &finish, &archive,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("authority_unavailable"), "{error}");
    }
}

#[test]
fn source_change_observation_new_failure_during_execution_rejects_completion() {
    let authority = TestAuthority::install();
    let (directory, request) = source_observation_material_unknown_fixture(&authority);
    let root = directory.path();
    let prepared = source_observation_prepare(root, &request);
    let (receipt, snapshot) = source_observation_receipt(root, &request);
    cockpit_repository::finish_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap_err();
    let summary = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
    let before = fs::read(&summary).unwrap();
    let evidence = root.join(format!(".ai/evidence/{OBS_ID}.verification.json"));
    let verified = fs::read(&evidence).unwrap();
    cockpit_repository::record_prepared_work_item_verification(&prepared, &receipt, &snapshot)
        .unwrap_err();
    assert_eq!(before, fs::read(summary).unwrap());
    assert_eq!(verified, fs::read(evidence).unwrap());
    let first =
        fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.events.jsonl"))).unwrap();
    cockpit_repository::finish_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap_err();
    let second =
        fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.events.jsonl"))).unwrap();
    assert!(
        second.starts_with(&first) && second.len() > first.len(),
        "identical later failure gets a new ordered event"
    );
}

#[test]
fn source_change_observation_recovered_failure_then_new_event_stays_red_without_material_blocker() {
    let authority = TestAuthority::install();
    let (directory, request) = source_observation_fixture(&authority, 3);
    let root = directory.path();
    cockpit_repository::finish_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap_err();
    source_observation_retry(&authority, root);
    let prepared = source_observation_prepare(root, &request);
    let (receipt, snapshot) = source_observation_receipt(root, &request);
    cockpit_repository::record_prepared_work_item_verification(&prepared, &receipt, &snapshot)
        .unwrap();
    assert_eq!(
        cockpit_repository::outcome_v2_with_runtime(root, OBS_ID, &runtime())
            .unwrap()
            .decision_state,
        Some(DecisionState::Green)
    );
    let summary_path = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
    let old_summary = source_observation_json(&summary_path);
    assert!(old_summary["verificationRecoveryBasis"]["failure"]["blockedEventId"].is_string());
    let evidence_path = root.join(format!(".ai/evidence/{OBS_ID}.verification.json"));
    let verified_bytes = fs::read(&evidence_path).unwrap();
    cockpit_repository::record_work_item_governance_controls_with_runtime(
        root,
        OBS_ID,
        &serde_json::json!({"intentAlignment":{"state":"unknown"}}),
        &runtime(),
    )
    .unwrap();
    cockpit_repository::finish_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap_err();
    cockpit_repository::record_work_item_governance_controls_with_runtime(
        root,
        OBS_ID,
        &serde_json::json!({"intentAlignment":{"state":"resolved","evidence":["source.rs"]}}),
        &runtime(),
    )
    .unwrap();
    let new_summary = source_observation_json(&summary_path);
    assert_eq!(
        old_summary["verificationRecoveryBasis"],
        new_summary["verificationRecoveryBasis"]
    );
    assert_eq!(verified_bytes, fs::read(evidence_path).unwrap());
    // The material gate and controls are now clear. Only a new, specific
    // failure identity invalidates the old causal recovery proof.
    let outcome = cockpit_repository::outcome_v2_with_runtime(root, OBS_ID, &runtime()).unwrap();
    assert_eq!(outcome.decision_state, Some(DecisionState::Red));
    let path = root.join(format!(".ai/work-items/active/{OBS_ID}.events.jsonl"));
    let mut events = fs::read_to_string(&path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    let first_timestamp = events
        .iter()
        .find(|event| event["eventType"] == "blocked")
        .unwrap()["timestamp"]
        .clone();
    events
        .iter_mut()
        .rfind(|event| event["eventType"] == "blocked")
        .unwrap()["timestamp"] = first_timestamp;
    fs::write(
        path,
        events
            .iter()
            .map(|event| serde_json::to_string(event).unwrap() + "\n")
            .collect::<String>(),
    )
    .unwrap();
    assert_eq!(
        cockpit_repository::outcome_v2_with_runtime(root, OBS_ID, &runtime())
            .unwrap()
            .decision_state,
        Some(DecisionState::Red),
        "equal timestamps cannot reverse ordered native failure causality"
    );
}
