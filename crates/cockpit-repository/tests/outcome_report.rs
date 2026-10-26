use cockpit_core::{DecisionState, Digest};
use cockpit_protocol::{
    HumanBenefitReport, OutcomeClaim, OutcomeReleaseProjection, OutcomeReportBindings,
    OutcomeReportSections, OutcomeState, OutcomeV2, ResourceFinalizationContext, RuntimeContext,
    TaskOutcomeReport, UsageAssurance, UsageCoverage, UsageSourceKind, UsageSubtotal, UsageSummary,
    UsageTokenCounts,
};
use cockpit_repository::{
    OutcomeRenderView, WorkItemStartOptions, archive_work_item, checkpoint_work_item,
    finish_work_item, outcome_render_input_from_outcome, outcome_v2, plan_resource_finalization,
    preflight_work_item, record_recovery_decision, record_verification,
    render_human_outcome_with_view, start_work_item_with_options,
};
use std::{fs, path::Path, process::Command};

fn clone_shared_no_checkout(source: &std::path::Path, destination: &std::path::Path) {
    let output = Command::new("git")
        .args(["clone", "--quiet", "--shared", "--no-checkout", "--", "."])
        .arg(destination)
        .current_dir(source)
        .output()
        .expect("clone fixed repository from its Git working directory");
    assert!(
        output.status.success(),
        "fixed repository clone failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn preserve_ai_bytes_in_fixture(root: &std::path::Path) {
    fs::write(root.join(".git/info/attributes"), b"/.ai/** -text\n")
        .expect("preserve exact historical fixture bytes");
}

#[test]
fn current_checkout_preserves_archived_ai_blob_bytes_with_autocrlf() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("source repository");
    let relative = ".ai/work-items/archive/WI-1111-token-outcome-ci.task-report.md";
    let source_attributes = Command::new("git")
        .args(["check-attr", "text", "--", relative])
        .current_dir(&source)
        .output()
        .expect("check repository attribute for archived report");
    assert!(source_attributes.status.success());
    assert_eq!(
        String::from_utf8_lossy(&source_attributes.stdout).trim(),
        format!("{relative}: text: unset"),
        "repository .gitattributes must disable text conversion for .ai artifacts"
    );
    let checkout = tempfile::tempdir().expect("temporary checkout root");
    let root = checkout.path().join("current checkout with spaces");
    clone_shared_no_checkout(&source, &root);
    fs::copy(source.join(".gitattributes"), root.join(".gitattributes"))
        .expect("install current repository attributes in fixture");
    assert!(
        Command::new("git")
            .args(["config", "core.autocrlf", "true"])
            .current_dir(&root)
            .status()
            .expect("enable fixture-local autocrlf")
            .success()
    );
    assert!(
        Command::new("git")
            .args(["checkout", "--quiet", "HEAD", "--", relative])
            .current_dir(&root)
            .status()
            .expect("checkout archived report using current repository attributes")
            .success()
    );
    let fixture_attributes = Command::new("git")
        .args(["check-attr", "text", "--", relative])
        .current_dir(&root)
        .output()
        .expect("check cloned repository attribute for archived report");
    assert!(fixture_attributes.status.success());
    assert_eq!(
        String::from_utf8_lossy(&fixture_attributes.stdout).trim(),
        format!("{relative}: text: unset"),
        "the cloned repository .gitattributes must disable text conversion"
    );
    let expected = Command::new("git")
        .args(["show", &format!("HEAD:{relative}")])
        .current_dir(&root)
        .output()
        .expect("read archived report blob");
    assert!(expected.status.success(), "git show failed: {expected:?}");
    let actual = fs::read(root.join(relative)).expect("read checked-out archived report");
    assert_eq!(
        actual, expected.stdout,
        "repository attributes must preserve exact .ai bytes with core.autocrlf=true"
    );
}

#[test]
fn historical_checkout_preserves_archived_ai_blob_bytes_with_autocrlf() {
    const REVISION: &str = "d18be4f8624d08f91fe1c961dfed287cfe9543df";
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("source repository");
    let checkout = tempfile::tempdir().expect("temporary historical checkout root");
    let root = checkout.path().join("historical checkout with spaces");
    clone_shared_no_checkout(&source, &root);
    assert!(
        Command::new("git")
            .args(["config", "core.autocrlf", "true"])
            .current_dir(&root)
            .status()
            .expect("enable fixture-local autocrlf")
            .success()
    );
    preserve_ai_bytes_in_fixture(&root);
    assert!(
        Command::new("git")
            .args(["checkout", "--quiet", "--detach", REVISION])
            .current_dir(&root)
            .status()
            .expect("checkout fixed historical revision")
            .success()
    );
    let relative =
        ".ai/work-items/archive/WI-1078-v1-0-1-post-merge-sync-finalization.task-report.md";
    let expected = Command::new("git")
        .args(["show", &format!("{REVISION}:{relative}")])
        .current_dir(&root)
        .output()
        .expect("read historical report blob");
    assert!(expected.status.success(), "git show failed: {expected:?}");
    let actual = fs::read(root.join(relative)).expect("read checked-out historical report");
    assert_eq!(
        actual, expected.stdout,
        "historical checkout must preserve exact .ai bytes with core.autocrlf=true"
    );
}

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
    cockpit_repository::attach(directory.path()).expect("attach");
    directory
}

#[test]
fn fixed_git_anchored_pre_policy_archive_remains_readable() {
    const FIXED_ARCHIVE_REVISION: &str = "d18be4f8624d08f91fe1c961dfed287cfe9543df";
    const WORK_ITEM_ID: &str = "WI-1078-v1-0-1-post-merge-sync-finalization";

    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let checkout = tempfile::tempdir().expect("temporary historical checkout root");
    let root = checkout.path().join("repository");
    clone_shared_no_checkout(&source, &root);
    preserve_ai_bytes_in_fixture(&root);
    let checkout_status = Command::new("git")
        .args(["checkout", "--quiet", "--detach", FIXED_ARCHIVE_REVISION])
        .current_dir(&root)
        .status()
        .expect("checkout fixed historical archive revision");
    assert!(
        checkout_status.success(),
        "fixed historical checkout failed"
    );

    let outcome = outcome_v2(&root, WORK_ITEM_ID)
        .expect("fixed Git-anchored pre-policy archive remains readable");
    let report = outcome
        .task_outcome_report
        .as_ref()
        .expect("historical archive exposes its preserved Task Outcome report");
    assert_eq!(report.work_item_id, WORK_ITEM_ID);
    assert_eq!(report.format, "ai-cockpit.task-outcome");
    assert_eq!(report.schema_version, 1);
}

fn real_wi1111_to_wi1112_material_reference_at(revision: &str) -> (serde_json::Value, String) {
    const CURRENT_ID: &str = "WI-1112-windows-archive-byte-preservation";
    const RECOVERY_RECEIPT: &str = ".ai/decisions/WI-1111-token-outcome-ci.recovery.b4bfbfe8767af2eeb8fa290a73fb810891c073864a8acb9f29e048bf3cf8c04f.json";

    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("source repository");
    let checkout = tempfile::tempdir().expect("temporary WI-1112 fixture root");
    let root = checkout.path().join("repository");
    clone_shared_no_checkout(&source, &root);
    preserve_ai_bytes_in_fixture(&root);
    assert!(
        Command::new("git")
            .args(["checkout", "--quiet", "--detach", revision])
            .current_dir(&root)
            .status()
            .expect("checkout committed source snapshot for lineage fixture")
            .success()
    );

    for relative in [
        format!(".ai/work-items/active/{CURRENT_ID}.contract.json"),
        RECOVERY_RECEIPT.to_owned(),
    ] {
        let from = source.join(&relative);
        let to = root.join(&relative);
        fs::create_dir_all(to.parent().expect("native fixture parent"))
            .expect("create native fixture parent");
        fs::copy(&from, &to).unwrap_or_else(|error| {
            panic!(
                "copy real WI-1112 lineage input {}: {error}",
                from.display()
            )
        });
    }

    let outcome = OutcomeV2 {
        schema_version: 2,
        repository_id: cockpit_repository::repository_id(&root).to_string(),
        work_item_id: CURRENT_ID.into(),
        state: OutcomeState::Unknown,
        decision_state: Some(DecisionState::Yellow),
        summary: "successor fixture for a reference-only predecessor review".into(),
        acceptance_results: vec![],
        unknowns: vec![],
        evidence_refs: vec![],
        human_benefit_report: HumanBenefitReport {
            state: OutcomeState::Unknown,
            user_visible_changes: vec![],
            affected_users: vec![],
            unknowns: vec![],
            evidence_refs: vec![],
        },
        task_outcome_report: None,
        failed_gate: None,
        recovery_condition: None,
        recovery_decision: None,
        historical_status: None,
        governance_reasons: vec![],
        finalization: None,
    };
    let input = outcome_render_input_from_outcome(&root, outcome);
    let delivery = input
        .delivery_continuity
        .as_ref()
        .expect("real WI-1111 predecessor continuity");
    let projection = serde_json::to_value(delivery).expect("structured continuity projection");
    let material = projection
        .get("materialReviewReference")
        .and_then(serde_json::Value::as_object)
        .unwrap_or_else(|| {
            panic!("separate material review reference projection missing: {projection}")
        });

    let rendered = render_human_outcome_with_view(&input, "en", OutcomeRenderView::Full);
    (serde_json::Value::Object(material.clone()), rendered)
}

#[test]
fn real_wi1111_to_wi1112_current_reference_is_not_review_authority() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("source repository");
    let current_head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(source)
        .output()
        .expect("read current committed source revision");
    assert!(current_head.status.success(), "read current HEAD failed");
    let revision = String::from_utf8(current_head.stdout)
        .expect("HEAD is UTF-8")
        .trim()
        .to_owned();
    let (material, rendered) = real_wi1111_to_wi1112_material_reference_at(&revision);

    assert_eq!(
        material["predecessorWorkItemId"],
        "WI-1111-token-outcome-ci"
    );
    assert_eq!(
        material["state"], "context_changed",
        "material reference: {material:?}"
    );
    assert_eq!(material["referenceOnly"], true);
    assert_eq!(material["satisfiesCurrentReview"], false);
    assert_eq!(material["priorHunkIdentity"], "not_retained");
    assert_eq!(material["priorScannerIdentity"], "not_attested_in_receipt");
    let same_content_paths = material["sameContentBlobs"]
        .as_array()
        .expect("same-content references are an array")
        .iter()
        .map(|blob| blob["path"].as_str().expect("blob path is a string"))
        .collect::<Vec<_>>();
    assert_eq!(
        same_content_paths,
        [
            "crates/cockpit-cli/tests/usage.rs",
            "crates/cockpit-repository/tests/usage.rs"
        ],
        "show exact unchanged predecessor bytes without treating them as current authority: {material:?}"
    );
    assert!(
        material["contextDifferences"]
            .as_array()
            .is_some_and(|items| items.iter().any(|value| value == "contract")),
        "the successor Contract must remain visibly distinct: {material:?}"
    );
    assert!(
        material["contextDifferences"]
            .as_array()
            .is_some_and(|items| items.iter().any(|value| value == "scope")),
        "the changed scope must be explicit beside the read-only byte references: {material:?}"
    );

    assert!(
        rendered.contains("Same committed file bytes verified: 2"),
        "{rendered}"
    );
    assert!(
        rendered.contains("does not authorize the current review"),
        "{rendered}"
    );
}

#[test]
fn real_wi1111_to_wi1112_ancestor_reference_reports_shared_bytes_without_authority() {
    const ORIGINAL_WI1112_REVIEW_HEAD: &str = "52851ee8058d99de9af6717829ec18052b1f57ef";
    let (material, rendered) =
        real_wi1111_to_wi1112_material_reference_at(ORIGINAL_WI1112_REVIEW_HEAD);

    assert_eq!(
        material["predecessorWorkItemId"],
        "WI-1111-token-outcome-ci"
    );
    assert_eq!(material["state"], "context_changed");
    assert_eq!(material["referenceOnly"], true);
    assert_eq!(material["satisfiesCurrentReview"], false);
    assert_eq!(material["priorHunkIdentity"], "not_retained");
    assert_eq!(material["priorScannerIdentity"], "not_attested_in_receipt");
    let same_content_paths = material["sameContentBlobs"]
        .as_array()
        .expect("same-content references are an array")
        .iter()
        .map(|blob| blob["path"].as_str().expect("blob path is a string"))
        .collect::<Vec<_>>();
    assert_eq!(
        same_content_paths,
        [
            "crates/cockpit-cli/tests/usage.rs",
            "crates/cockpit-repository/src/outcome_render.rs",
            "crates/cockpit-repository/tests/usage.rs"
        ],
        "all three real predecessor-reviewed blobs match the original source snapshot: {material:?}"
    );
    assert!(
        material["contextDifferences"]
            .as_array()
            .is_some_and(|items| items.iter().any(|value| value == "contract")),
        "the successor Contract must remain visibly distinct: {material:?}"
    );
    assert!(
        material["contextDifferences"]
            .as_array()
            .is_some_and(|items| items.iter().any(|value| value == "scope")),
        "the changed scope must be explicit beside the read-only byte references: {material:?}"
    );
    assert!(
        rendered.contains("Same committed file bytes verified: 3"),
        "{rendered}"
    );
    assert!(
        rendered.contains("does not authorize the current review"),
        "{rendered}"
    );
}

fn commit_fixture_source(path: &std::path::Path, source: &str) {
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
                "fixture source baseline",
            ])
            .current_dir(path)
            .status()
            .expect("git commit fixture source")
            .success()
    );
}

fn record_fixture_retry(directory: &tempfile::TempDir, work_item_id: &str) {
    let active = directory.path().join(".ai/work-items/active");
    let contract: serde_json::Value = serde_json::from_slice(
        &fs::read(active.join(format!("{work_item_id}.contract.json"))).expect("Contract"),
    )
    .expect("Contract JSON");
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(active.join(format!("{work_item_id}.summary.json"))).expect("Summary"),
    )
    .expect("Summary JSON");
    let outcome: serde_json::Value = serde_json::from_slice(
        &fs::read(active.join(format!("{work_item_id}.outcome.json"))).expect("Outcome"),
    )
    .expect("Outcome JSON");
    let events =
        fs::read(active.join(format!("{work_item_id}.events.jsonl"))).expect("Events bytes");
    let runtime_digest = Digest::sha256_bytes(b"wi136-runtime");
    let runtime = cockpit_protocol::RuntimeContext {
        runtime_version: "0.2.10".into(),
        protocol_version: 1,
        runtime_digest: runtime_digest.clone(),
    };
    let retry = serde_json::json!({
        "schemaVersion": 1,
        "decisionId": "work-item-recovery",
        "decision": "retry",
        "workItemId": work_item_id,
        "repositoryId": contract["repositoryId"],
        "predecessorWorkItemId": work_item_id,
        "predecessorContractDigest": cockpit_protocol::digest_json(&contract).expect("Contract digest"),
        "predecessorSummaryDigest": cockpit_protocol::digest_json(&summary).expect("Summary digest"),
        "predecessorOutcomeDigest": cockpit_protocol::digest_json(&outcome).expect("Outcome digest"),
        "predecessorEventsDigest": Digest::sha256_bytes(&events),
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "actor": "human:test-fixture",
        "authoritySource": "WI-1111 lifecycle fixture",
        "reason": "source changed after a completed finish",
        "evidenceRefs": [],
        "policyRefs": [],
        "decidedAt": "2026-10-25T12:00:00Z",
        "resumeCondition": "current preflight, replacement verification, and explicit finish"
    });
    record_recovery_decision(directory.path(), work_item_id, &retry, &runtime)
        .expect("bind retry to exact finish predecessor");
}

fn ready(directory: &tempfile::TempDir, id: &str) {
    start_work_item_with_options(
        directory.path(),
        id,
        "generate a typed report",
        "show evidence-bound outcome sections",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("start");
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
    .expect("finalization plan");
    let contract = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.10",
        &Digest::sha256_bytes(b"wi136-runtime"),
    )
    .expect("verify");
}

fn rewrite_archived_report_digest_chain(
    root: &std::path::Path,
    work_item_id: &str,
    report_bytes: &[u8],
) {
    let archive = root.join(".ai/work-items/archive");
    let digest = Digest::sha256_bytes(report_bytes).to_string();
    let summary_path = archive.join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    summary["lifecycleFacts"]["taskOutcomeProjectionFinishBinding"]["taskReportJsonDigest"] =
        digest.clone().into();
    let summary_bytes = serde_json::to_vec_pretty(&summary).expect("serialize Summary");
    fs::write(&summary_path, &summary_bytes).expect("rebind Summary");

    let events_path = archive.join(format!("{work_item_id}.events.jsonl"));
    let events = fs::read_to_string(&events_path).expect("events");
    let rebound_events = events
        .lines()
        .map(|line| {
            let mut event: serde_json::Value = serde_json::from_str(line).expect("event JSON");
            if event["eventType"] == "completed" {
                event["projectionBinding"]["taskReportJsonDigest"] = digest.clone().into();
            }
            serde_json::to_string(&event).expect("serialize event")
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(&events_path, &rebound_events).expect("rebind completion event");

    let outcome_path = archive.join(format!("{work_item_id}.outcome.json"));
    let mut outcome: serde_json::Value =
        serde_json::from_slice(&fs::read(&outcome_path).expect("Outcome")).expect("Outcome JSON");
    outcome["taskReportDigest"] = digest.clone().into();
    let outcome_bytes = serde_json::to_vec_pretty(&outcome).expect("serialize Outcome");
    fs::write(&outcome_path, &outcome_bytes).expect("rebind Outcome");

    let manifest_path = archive.join(format!("{work_item_id}.archive.json"));
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("manifest"))
            .expect("manifest JSON");
    manifest["files"]["taskReportDigest"] = digest.into();
    manifest["files"]["summaryDigest"] = Digest::sha256_bytes(&summary_bytes).to_string().into();
    manifest["files"]["eventsDigest"] = Digest::sha256_bytes(rebound_events.as_bytes())
        .to_string()
        .into();
    manifest["files"]["outcomeDigest"] = Digest::sha256_bytes(&outcome_bytes).to_string().into();
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("recompute manifest digest chain");
}

fn ready_with_source_scope(directory: &tempfile::TempDir, id: &str) {
    fs::write(directory.path().join("source.rs"), "pub fn current() {}\n").expect("source fixture");
    commit_fixture_source(directory.path(), "source.rs");
    start_work_item_with_options(
        directory.path(),
        id,
        "verify a source-bound report refresh",
        "preserve finish projections across same-source refresh and native re-finish",
        &["source.rs".into(), ".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("start");
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
    .expect("finalization plan");
    let contract = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.10",
        &Digest::sha256_bytes(b"wi136-runtime"),
    )
    .expect("verify");
}

fn ready_with_source_scope_and_standard_checkpoints(directory: &tempfile::TempDir, id: &str) {
    fs::write(directory.path().join("source.rs"), "pub fn current() {}\n").expect("source fixture");
    commit_fixture_source(directory.path(), "source.rs");
    start_work_item_with_options(
        directory.path(),
        id,
        "verify checkpoint-bound report refresh",
        "require explicit re-finish after source changes",
        &["source.rs".into(), ".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("start");
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
    .expect("finalization plan");
    let contract = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract).expect("Contract")).expect("Contract JSON");
    value["checkpointPolicy"] = serde_json::json!({
        "schemaVersion": 1,
        "profile": "standard",
        "requiredBeforeFinish": true,
        "requiredStages": ["before_edit", "before_finish"],
        "requiredChecks": []
    });
    fs::write(
        &contract,
        serde_json::to_vec_pretty(&value).expect("serialize Contract"),
    )
    .expect("standard checkpoint policy");
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("before-edit checkpoint");
    record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.10",
        &Digest::sha256_bytes(b"wi136-runtime"),
    )
    .expect("verify");
}

#[test]
fn task_outcome_projection_binding_has_a_strict_versioned_wire_shape() {
    let policy = cockpit_protocol::TaskOutcomeProjectionPolicy {
        schema_version: 1,
        repository_id: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .into(),
        work_item_id: "WI-PROJECTION".into(),
        contract_digest: Digest::sha256_bytes(b"contract"),
        source_snapshot_digest: Digest::sha256_bytes(b"snapshot"),
        producer_runtime_version: "1.0.1-candidate".into(),
        producer_runtime_digest: Digest::sha256_bytes(b"runtime"),
        json_format: "ai-cockpit.task-outcome".into(),
        json_schema_version: 1,
        markdown_renderer: "ai-cockpit.task-outcome-markdown".into(),
        markdown_renderer_version: 2,
    };
    let binding = cockpit_protocol::TaskOutcomeProjectionFinishBinding {
        schema_version: 1,
        projection_policy_digest: cockpit_protocol::digest_json(&policy).unwrap(),
        task_report_json_digest: Digest::sha256_bytes(b"json report"),
        task_report_markdown_digest: Digest::sha256_bytes(b"markdown report"),
    };
    let encoded = serde_json::to_value(&policy).expect("policy serializes");
    assert_eq!(encoded["schemaVersion"], 1);
    assert_eq!(encoded["jsonSchemaVersion"], 1);
    assert_eq!(encoded["markdownRendererVersion"], 2);
    assert_eq!(
        serde_json::from_value::<cockpit_protocol::TaskOutcomeProjectionPolicy>(encoded.clone())
            .expect("policy parses"),
        policy
    );
    let mut unknown = encoded;
    unknown["legacyRendererAllowed"] = serde_json::json!(true);
    assert!(
        serde_json::from_value::<cockpit_protocol::TaskOutcomeProjectionPolicy>(unknown).is_err()
    );

    let event = cockpit_protocol::TaskOutcomeEvent {
        schema_version: 1,
        event_id: "event-projection".into(),
        repository_id: policy.repository_id.clone(),
        work_item_id: policy.work_item_id.clone(),
        event_type: "completed".into(),
        timestamp: "2026-10-25T12:00:00Z".into(),
        detail: "Task Outcome generated.".into(),
        evidence_refs: vec![],
        related_event_ids: vec![],
        correction_of: None,
        finding_fingerprint: None,
        projection_binding: Some(binding.clone()),
    };
    let value = serde_json::to_value(&event).expect("event serializes");
    assert_eq!(
        value["projectionBinding"]["projectionPolicyDigest"],
        binding.projection_policy_digest.to_string()
    );
    assert_eq!(
        serde_json::from_value::<cockpit_protocol::TaskOutcomeEvent>(value).unwrap(),
        event
    );
}

#[test]
fn same_source_refresh_freezes_bytes_and_source_change_retains_predecessor() {
    let directory = repository();
    let work_item_id = "WI-PROJECTION-REFRESH";
    ready_with_source_scope(&directory, work_item_id);
    finish_work_item(directory.path(), work_item_id).expect("initial finish");
    let active = directory.path().join(".ai/work-items/active");
    let report_path = active.join(format!("{work_item_id}.task-report.json"));
    let markdown_path = active.join(format!("{work_item_id}.task-report.md"));
    let first_json = fs::read(&report_path).expect("first report bytes");
    let first_markdown = fs::read(&markdown_path).expect("first Markdown bytes");

    record_verification(
        directory.path(),
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.10",
        &Digest::sha256_bytes(b"wi136-runtime"),
    )
    .expect("same-source verification refresh");
    assert_eq!(fs::read(&report_path).expect("frozen report"), first_json);
    assert_eq!(
        fs::read(&markdown_path).expect("frozen Markdown"),
        first_markdown
    );

    let prior_outcome = fs::read_to_string(active.join(format!("{work_item_id}.outcome.json")))
        .expect("prior Outcome bytes");
    fs::write(
        directory.path().join("source.rs"),
        "pub fn changed() -> bool { true }\n",
    )
    .expect("source change");
    commit_fixture_source(directory.path(), "source.rs");
    let summary_before_retry: serde_json::Value = serde_json::from_slice(
        &fs::read(active.join(format!("{work_item_id}.summary.json"))).expect("Summary"),
    )
    .expect("Summary JSON");
    let refused = record_verification(
        directory.path(),
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 2}),
        "0.2.10",
        &Digest::sha256_bytes(b"wi136-runtime"),
    )
    .expect_err("source change requires native retry and explicit finish");
    assert!(refused.to_string().contains("native retry"), "{refused}");
    assert_eq!(fs::read(&report_path).expect("frozen report"), first_json);
    assert_eq!(
        fs::read(&markdown_path).expect("frozen Markdown"),
        first_markdown
    );

    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    preflight_work_item(directory.path(), &contract_path).expect("current source preflight");
    record_fixture_retry(&directory, work_item_id);
    let evidence = record_verification(
        directory.path(),
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 2}),
        "0.2.10",
        &Digest::sha256_bytes(b"wi136-runtime"),
    )
    .expect("replacement verification after native retry");
    finish_work_item(directory.path(), work_item_id).expect("explicit re-finish");

    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(active.join(format!("{work_item_id}.summary.json"))).expect("Summary"),
    )
    .expect("Summary JSON");
    let predecessors = summary["lifecycleFacts"]["taskOutcomeProjectionPredecessors"]
        .as_array()
        .expect("preserved predecessor array");
    assert_eq!(predecessors.len(), 1);
    let predecessor: cockpit_protocol::TaskOutcomeProjectionPredecessor =
        serde_json::from_value(predecessors[0].clone()).expect("typed predecessor");
    assert_eq!(predecessor.task_report_json_utf8.as_bytes(), first_json);
    assert_eq!(
        predecessor.task_report_markdown_utf8.as_bytes(),
        first_markdown
    );
    assert_eq!(predecessor.outcome_json_utf8, prior_outcome);

    let new_report: TaskOutcomeReport =
        serde_json::from_slice(&fs::read(&report_path).expect("new report"))
            .expect("new report JSON");
    assert_eq!(
        new_report.bindings.repository_snapshot_digest.as_ref(),
        serde_json::from_value::<Digest>(evidence["repositorySnapshotDigest"].clone())
            .ok()
            .as_ref()
    );
    let current_policy: cockpit_protocol::TaskOutcomeProjectionPolicy =
        serde_json::from_value(summary["taskOutcomeProjectionPolicy"].clone())
            .expect("current policy");
    let current_binding: cockpit_protocol::TaskOutcomeProjectionFinishBinding =
        serde_json::from_value(
            summary["lifecycleFacts"]["taskOutcomeProjectionFinishBinding"].clone(),
        )
        .expect("new finish binding");
    assert_eq!(
        current_binding.projection_policy_digest,
        cockpit_protocol::digest_json(&current_policy).expect("current policy digest")
    );
    let events = fs::read_to_string(active.join(format!("{work_item_id}.events.jsonl")))
        .expect("events")
        .lines()
        .map(|line| serde_json::from_str::<cockpit_protocol::TaskOutcomeEvent>(line).unwrap())
        .filter(|event| event.event_type == "completed")
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 2);
    assert_eq!(
        events[0].projection_binding.as_ref(),
        predecessor.finish_binding.as_ref()
    );
    assert_eq!(
        events[1].projection_binding.as_ref(),
        Some(&current_binding)
    );
    assert_eq!(summary_before_retry["state"], "finish_ready");
    archive_work_item(directory.path(), work_item_id)
        .expect("archive validates current projection and retained predecessor bytes");
}

#[test]
fn source_change_during_finish_refresh_requires_explicit_checkpoint_and_finish() {
    let directory = repository();
    let work_item_id = "WI-PROJECTION-STRICT-REFRESH";
    ready_with_source_scope_and_standard_checkpoints(&directory, work_item_id);
    finish_work_item(directory.path(), work_item_id)
        .expect("initial finish with required checkpoints");
    let active = directory.path().join(".ai/work-items/active");
    let summary_path = active.join(format!("{work_item_id}.summary.json"));
    let before: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary before"))
            .expect("Summary JSON");
    assert_eq!(before["state"], "finish_ready");
    assert!(
        before["checkpointEvidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| { entry["stage"] == "before_finish" })
    );

    fs::write(
        directory.path().join("source.rs"),
        "pub fn changed() -> bool { true }\n",
    )
    .expect("source change");
    commit_fixture_source(directory.path(), "source.rs");
    let report_path = active.join(format!("{work_item_id}.task-report.json"));
    let markdown_path = active.join(format!("{work_item_id}.task-report.md"));
    let original_report = fs::read(&report_path).expect("original report");
    let original_markdown = fs::read(&markdown_path).expect("original Markdown");
    let refused = record_verification(
        directory.path(),
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 2}),
        "0.2.10",
        &Digest::sha256_bytes(b"wi136-runtime"),
    )
    .expect_err("record_verification must not fabricate a replacement finish");
    assert!(refused.to_string().contains("native retry"), "{refused}");

    let after: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary after"))
            .expect("Summary JSON");
    assert_eq!(
        after, before,
        "refused refresh must preserve lifecycle facts"
    );
    assert_eq!(
        fs::read(report_path).expect("report after"),
        original_report
    );
    assert_eq!(
        fs::read(markdown_path).expect("Markdown after"),
        original_markdown
    );
    let archive_error = archive_work_item(directory.path(), work_item_id).expect_err(
        "source-changing refresh has not produced the required before-finish checkpoint",
    );
    assert!(
        archive_error
            .to_string()
            .contains("checkpoint evidence is invalid"),
        "expected the missing before-finish checkpoint to block archive, got {archive_error}"
    );
    assert_eq!(
        after["state"], "finish_ready",
        "a refused refresh must leave the existing finish state and evidence untouched"
    );
}

#[derive(Clone)]
struct RenderFixture {
    state: OutcomeState,
    decision_state: DecisionState,
    historical_status: Option<String>,
    sections: OutcomeReportSections,
    unknowns: Vec<String>,
    evidence_refs: Vec<String>,
}

fn render_fixture(directory: &tempfile::TempDir, id: &str, fixture: RenderFixture) -> String {
    render_fixture_view(directory, id, fixture, OutcomeRenderView::Full)
}

fn render_fixture_view(
    directory: &tempfile::TempDir,
    id: &str,
    fixture: RenderFixture,
    view: OutcomeRenderView,
) -> String {
    render_fixture_view_language(directory, id, fixture, view, "en")
}

fn render_fixture_view_language(
    directory: &tempfile::TempDir,
    id: &str,
    fixture: RenderFixture,
    view: OutcomeRenderView,
    language: &str,
) -> String {
    render_fixture_view_language_with_usage(directory, id, fixture, view, language, None)
}

fn render_fixture_view_language_with_usage(
    directory: &tempfile::TempDir,
    id: &str,
    fixture: RenderFixture,
    view: OutcomeRenderView,
    language: &str,
    usage: Option<UsageSummary>,
) -> String {
    let RenderFixture {
        state,
        decision_state,
        historical_status,
        sections,
        unknowns,
        evidence_refs,
    } = fixture;
    let report = TaskOutcomeReport {
        format: "ai-cockpit.task-outcome".into(),
        schema_version: 1,
        work_item_id: id.into(),
        status: state.clone(),
        human_status_color: decision_state.clone(),
        bindings: OutcomeReportBindings {
            repository_id: cockpit_repository::repository_id(directory.path()).to_string(),
            work_item_id: id.into(),
            evidence_refs: evidence_refs.clone(),
            repository_snapshot_digest: None,
        },
        sections,
        usage,
        release: None,
        failed_gate: None,
        recovery_condition: None,
    };
    let outcome = OutcomeV2 {
        schema_version: 2,
        repository_id: cockpit_repository::repository_id(directory.path()).to_string(),
        work_item_id: id.into(),
        state: state.clone(),
        decision_state: Some(decision_state),
        summary: "fixture summary".into(),
        acceptance_results: vec!["fixture acceptance".into()],
        unknowns: unknowns.clone(),
        evidence_refs: evidence_refs.clone(),
        human_benefit_report: HumanBenefitReport {
            state: OutcomeState::Unknown,
            user_visible_changes: Vec::new(),
            affected_users: Vec::new(),
            unknowns,
            evidence_refs,
        },
        task_outcome_report: Some(report),
        failed_gate: None,
        recovery_condition: None,
        recovery_decision: None,
        historical_status,
        governance_reasons: Vec::new(),
        finalization: None,
    };
    let input = outcome_render_input_from_outcome(directory.path(), outcome);
    render_human_outcome_with_view(&input, language, view)
}

#[test]
fn usage_projection_is_visible_in_summary_and_full_in_three_languages() {
    let directory = repository();
    let id = "WI-USAGE-OUTCOME";
    let fixture = RenderFixture {
        state: OutcomeState::Unknown,
        decision_state: DecisionState::Yellow,
        historical_status: None,
        sections: OutcomeReportSections::default(),
        unknowns: Vec::new(),
        evidence_refs: Vec::new(),
    };
    let cutoff = "2026-10-07T00:00:00Z";
    for (language, label, unknown_count, claim_label) in [
        ("en", "Usage", "input unknown", "caller claim"),
        ("zh", "用量", "输入 未知", "调用方声明"),
        ("ja", "使用量", "入力 不明", "呼び出し元の申告"),
    ] {
        for view in [OutcomeRenderView::Summary, OutcomeRenderView::Full] {
            let unknown = render_fixture_view_language_with_usage(
                &directory,
                id,
                fixture.clone(),
                view,
                language,
                Some(UsageSummary::unknown(
                    id,
                    cutoff.into(),
                    "no_usage_receipts",
                )),
            );
            assert!(unknown.contains(label), "{language} {view:?}: {unknown}");
            assert!(unknown.contains("no_usage_receipts"), "{unknown}");
            assert!(unknown.contains(cutoff), "{unknown}");
            assert!(unknown.contains(unknown_count), "{unknown}");

            let partial = render_fixture_view_language_with_usage(
                &directory,
                id,
                fixture.clone(),
                view,
                language,
                Some(UsageSummary {
                    schema_version: 1,
                    work_item_id: id.into(),
                    cutoff: cutoff.into(),
                    coverage: UsageCoverage::Partial,
                    unknown_reasons: vec![
                        "caller_reported_usage_only".into(),
                        "token_count_unknown".into(),
                    ],
                    receipt_refs: Vec::new(),
                    totals: UsageTokenCounts {
                        input_tokens: Some(10),
                        output_tokens: None,
                        cached_input_tokens: Some(3),
                        reasoning_tokens: None,
                    },
                    subtotals: vec![UsageSubtotal {
                        reported_model: Some("reported-model".into()),
                        configured_models: vec!["configured-model".into()],
                        source_kinds: vec![UsageSourceKind::HostReported],
                        model_assurance: UsageAssurance::CallerClaim,
                        token_assurance: UsageAssurance::CallerClaim,
                        role: "implementer".into(),
                        phase: "implementation".into(),
                        record_count: 1,
                        counts: UsageTokenCounts {
                            input_tokens: Some(10),
                            output_tokens: None,
                            cached_input_tokens: Some(3),
                            reasoning_tokens: None,
                        },
                    }],
                }),
            );
            assert!(partial.contains(label), "{partial}");
            assert!(partial.contains("reported-model"), "{partial}");
            assert!(partial.contains("configured-model"), "{partial}");
            assert!(partial.contains("caller_reported_usage_only"), "{partial}");
            assert!(partial.contains(claim_label), "{partial}");
            assert!(!partial.contains("verified adapter"), "{partial}");
            assert!(partial.contains("10"), "{partial}");
            assert!(partial.contains(cutoff), "{partial}");
        }
    }
    let legacy =
        render_fixture_view_language(&directory, id, fixture, OutcomeRenderView::Summary, "en");
    assert!(legacy.contains("Usage coverage: unknown"), "{legacy}");
    assert!(legacy.contains("historical_usage_not_recorded"), "{legacy}");
    assert!(!legacy.contains("input 0"), "unknown is not zero: {legacy}");
}

#[test]
fn historical_usage_absence_is_explicit_in_every_human_view() {
    let directory = repository();
    let fixture = RenderFixture {
        state: OutcomeState::Unknown,
        decision_state: DecisionState::Yellow,
        historical_status: None,
        sections: OutcomeReportSections::default(),
        unknowns: Vec::new(),
        evidence_refs: Vec::new(),
    };
    for (language, count, cutoff) in [
        ("en", "input unknown", "cutoff: unknown"),
        ("zh", "输入 未知", "截止时间: 未知"),
        ("ja", "入力 不明", "締切時刻: 不明"),
    ] {
        for view in [OutcomeRenderView::Summary, OutcomeRenderView::Full] {
            let rendered = render_fixture_view_language_with_usage(
                &directory,
                "WI-HISTORICAL-USAGE",
                fixture.clone(),
                view,
                language,
                None,
            );
            assert!(
                rendered.contains("historical_usage_not_recorded"),
                "{rendered}"
            );
            assert!(rendered.contains(count), "{rendered}");
            assert!(rendered.contains(cutoff), "{rendered}");
        }
    }
}

#[test]
fn human_renderer_separates_verification_lifecycle_and_human_decision() {
    let directory = repository();
    let id = "WI-OUTCOME-TRUST";
    fs::write(
        directory
            .path()
            .join(format!(".ai/work-items/active/{id}.summary.json")),
        serde_json::to_vec_pretty(&serde_json::json!({"state":"implementation_active"}))
            .expect("summary"),
    )
    .expect("summary write");
    let text = render_fixture(
        &directory,
        id,
        RenderFixture {
            state: OutcomeState::Verified,
            decision_state: DecisionState::Green,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec![],
            evidence_refs: vec![".ai/evidence/WI-OUTCOME-TRUST.verification.json".into()],
        },
    );
    assert!(text.contains("Outcome: 🟢 Declared verification passed — WI-OUTCOME-TRUST"));
    assert!(text.contains("Verification: Declared verification passed"));
    assert!(text.contains("Lifecycle: Implementation active"));
    assert!(text.contains("Human decision: Not recorded"));
    assert!(text.contains("Governance signal: Green; not a human approval"));
    assert!(!text.contains("Outcome: 🟢 Success"));

    let decision_path = directory
        .path()
        .join(format!(".ai/decisions/{id}.close.json"));
    fs::write(
        &decision_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "workItemId": id,
            "repositoryId": cockpit_repository::repository_id(directory.path()).to_string(),
            "state": "closed",
            "decisionState": "confirmed",
            "humanDecision": "approved",
            "structuredDecision": {
                "decision": "approved",
                "actor": "human:owner",
                "authoritySource": "explicit-user-authorization",
                "reason": "reviewed the evidence",
                "evidenceRefs": [".ai/evidence/WI-OUTCOME-TRUST.verification.json"],
                "policyRefs": ["project-policy"],
                "decidedAt": "2026-09-08T00:00:00Z",
                "resumeCondition": "rerun after base changes"
            }
        }))
        .expect("decision"),
    )
    .expect("decision write");
    let text = render_fixture(
        &directory,
        id,
        RenderFixture {
            state: OutcomeState::Verified,
            decision_state: DecisionState::Green,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec![],
            evidence_refs: vec![".ai/evidence/WI-OUTCOME-TRUST.verification.json".into()],
        },
    );
    assert!(text.contains("Human decision: Recorded: approved"));
    assert!(text.contains("Authority source: explicit-user-authorization"));
    assert!(text.contains("Assurance level: Unknown"));
    assert!(!text.contains("Human decision: Not recorded"));

    let mut decision_record: serde_json::Value =
        serde_json::from_slice(&fs::read(&decision_path).expect("decision record"))
            .expect("decision JSON");
    decision_record["assurance"] = serde_json::json!("repository_verified");
    fs::write(
        &decision_path,
        serde_json::to_vec_pretty(&decision_record).expect("decision with assurance"),
    )
    .expect("decision with assurance write");
    let text = render_fixture(
        &directory,
        id,
        RenderFixture {
            state: OutcomeState::Verified,
            decision_state: DecisionState::Green,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec![],
            evidence_refs: vec![".ai/evidence/WI-OUTCOME-TRUST.verification.json".into()],
        },
    );
    assert!(text.contains("Assurance level: repository_verified"));
}

#[test]
fn human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields() {
    let directory = repository();
    let id = "WI-OUTCOME-RISK-BOUNDARY";
    let evidence = ".ai/evidence/WI-OUTCOME-RISK-BOUNDARY.verification.json".to_owned();
    let text = render_fixture(
        &directory,
        id,
        RenderFixture {
            state: OutcomeState::Verified,
            decision_state: DecisionState::Green,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec![],
            evidence_refs: vec![evidence.clone()],
        },
    );
    assert!(text.contains("Risk findings: Not recorded"));
    assert!(!text.contains("No risks"));
    assert!(!text.contains("Tests were not weakened"));

    let mut sections = OutcomeReportSections::default();
    sections.risks.push(OutcomeClaim {
        text: "No test weakening rule was triggered.".into(),
        evidence_refs: vec![evidence],
        inference: false,
    });
    let text = render_fixture(
        &directory,
        id,
        RenderFixture {
            state: OutcomeState::Verified,
            decision_state: DecisionState::Green,
            historical_status: None,
            sections,
            unknowns: vec![],
            evidence_refs: vec![".ai/evidence/WI-OUTCOME-RISK-BOUNDARY.verification.json".into()],
        },
    );
    assert!(
        text.contains(
            "Test-weakening scan: no trigger was recorded for the referenced check scope"
        )
    );
    assert!(text.contains("does not prove that tests were not weakened"));
    assert!(!text.contains("Tests were not weakened."));
}

#[test]
fn default_summary_is_four_part_and_full_view_retains_audit_sections() {
    let directory = repository();
    let evidence = ".ai/evidence/WI-OUTCOME-SUMMARY.verification.json".to_owned();
    let mut sections = OutcomeReportSections::default();
    sections.delivered_changes.push(OutcomeClaim {
        text: "The evidence-bound summary was generated.".into(),
        evidence_refs: vec![evidence.clone()],
        inference: false,
    });
    sections.risks.push(OutcomeClaim {
        text: "No test weakening rule was triggered.".into(),
        evidence_refs: vec![evidence.clone()],
        inference: false,
    });
    let summary = render_fixture_view(
        &directory,
        "WI-OUTCOME-SUMMARY",
        RenderFixture {
            state: OutcomeState::Verified,
            decision_state: DecisionState::Green,
            historical_status: None,
            sections,
            unknowns: vec!["scope_not_revalidated".into()],
            evidence_refs: vec![evidence],
        },
        OutcomeRenderView::Summary,
    );
    assert!(summary.contains("Result"));
    assert!(summary.contains("Key changes"));
    assert!(summary.contains("Remaining uncertainty"));
    assert!(summary.contains("Human next step"));
    assert!(summary.contains("The evidence-bound summary was generated."));
    assert!(summary.contains("Test-weakening scan: no trigger was recorded"));
    assert!(summary.contains("scope_not_revalidated"));
    assert!(summary.contains("Full evidence report:"));
    assert!(!summary.contains("Problems found"));
    assert!(!summary.contains("Risks avoided"));

    let stale = render_fixture_view(
        &directory,
        "WI-OUTCOME-SUMMARY-STALE",
        RenderFixture {
            state: OutcomeState::NotReady,
            decision_state: DecisionState::Yellow,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec!["evidence_stale".into()],
            evidence_refs: vec![".ai/evidence/stale.json".into()],
        },
        OutcomeRenderView::Summary,
    );
    assert!(stale.contains("evidence is stale"));
    assert!(!stale.contains("Required verification evidence is not present"));

    let full = render_fixture(
        &directory,
        "WI-OUTCOME-SUMMARY-FULL",
        RenderFixture {
            state: OutcomeState::Verified,
            decision_state: DecisionState::Green,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec![],
            evidence_refs: vec![".ai/evidence/full.json".into()],
        },
    );
    assert!(full.contains("Problems found"));
    assert!(full.contains("Evidence"));
}

#[test]
fn summary_does_not_present_changed_paths_as_delivered_behavior() {
    let directory = repository();
    let mut sections = OutcomeReportSections::default();
    sections.delivered_changes.push(OutcomeClaim {
        text: "Changed path: crates/example/src/lib.rs".into(),
        evidence_refs: vec![".ai/evidence/WI-OUTCOME-PATHS.verification.json".into()],
        inference: false,
    });
    let fixture = RenderFixture {
        state: OutcomeState::Verified,
        decision_state: DecisionState::Green,
        historical_status: None,
        sections,
        unknowns: vec![],
        evidence_refs: vec![".ai/evidence/WI-OUTCOME-PATHS.verification.json".into()],
    };
    let summary = render_fixture_view(
        &directory,
        "WI-OUTCOME-PATHS",
        fixture.clone(),
        OutcomeRenderView::Summary,
    );
    assert!(!summary.contains("Changed path:"), "summary={summary}");

    let full = render_fixture(&directory, "WI-OUTCOME-PATHS-FULL", fixture);
    assert!(full.contains("Changed path:"), "full={full}");
}

#[test]
fn summary_keeps_missing_benefit_and_localizes_reader_sections() {
    let directory = repository();
    for (language, sections) in [
        ("zh", ["结果", "关键变化", "剩余不确定性", "人的下一步"]),
        (
            "ja",
            ["結果", "主な変更", "残る不確実性", "人間の次のアクション"],
        ),
    ] {
        let text = render_fixture_view_language(
            &directory,
            &format!("WI-OUTCOME-SUMMARY-{language}"),
            RenderFixture {
                state: OutcomeState::Verified,
                decision_state: DecisionState::Green,
                historical_status: None,
                sections: OutcomeReportSections::default(),
                unknowns: vec![],
                evidence_refs: vec![format!(".ai/evidence/{language}.json")],
            },
            OutcomeRenderView::Summary,
            language,
        );
        for heading in sections {
            assert!(text.contains(heading), "missing {heading:?} in {text}");
        }
        assert!(
            text.contains("用户可见收益尚未声明")
                || text.contains("ユーザー向けの効果はまだ宣言されていません")
        );
    }
}

#[test]
fn release_projection_is_visible_in_summary_and_full_views_without_inference() {
    let directory = repository();
    let evidence = ".ai/evidence/WI-OUTCOME-RELEASE.acceptance.json".to_owned();
    let release = OutcomeReleaseProjection {
        version: Some("v0.2.94".into()),
        release_link: Some("https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.94".into()),
        install_acceptance: Some("passed".into()),
        upgrade_acceptance: Some("passed".into()),
        cleanup_status: Some("passed".into()),
        evidence_refs: vec![evidence.clone()],
    };
    for (language, expected) in [
        ("en", "Release result"),
        ("zh", "发布结果"),
        ("ja", "リリース結果"),
    ] {
        let fixture = RenderFixture {
            state: OutcomeState::Verified,
            decision_state: DecisionState::Green,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec![],
            evidence_refs: vec![evidence.clone()],
        };
        let text = render_fixture_view_language(
            &directory,
            &format!("WI-OUTCOME-RELEASE-{language}"),
            fixture.clone(),
            OutcomeRenderView::Summary,
            language,
        );
        assert!(!text.contains(expected));

        // The release projection is part of the structured report, not a
        // renderer-side repository lookup. Build the same fixture with it
        // attached and verify both reader views expose identical facts.
        let report = TaskOutcomeReport {
            format: "ai-cockpit.task-outcome".into(),
            schema_version: 1,
            work_item_id: format!("WI-OUTCOME-RELEASE-{language}"),
            status: OutcomeState::Verified,
            human_status_color: DecisionState::Green,
            bindings: OutcomeReportBindings {
                repository_id: cockpit_repository::repository_id(directory.path()).to_string(),
                work_item_id: format!("WI-OUTCOME-RELEASE-{language}"),
                evidence_refs: vec![evidence.clone()],
                repository_snapshot_digest: None,
            },
            sections: fixture.sections,
            usage: None,
            release: Some(release.clone()),
            failed_gate: None,
            recovery_condition: None,
        };
        let outcome = OutcomeV2 {
            schema_version: 2,
            repository_id: cockpit_repository::repository_id(directory.path()).to_string(),
            work_item_id: format!("WI-OUTCOME-RELEASE-{language}"),
            state: OutcomeState::Verified,
            decision_state: Some(DecisionState::Green),
            summary: "fixture summary".into(),
            acceptance_results: vec![],
            unknowns: vec![],
            evidence_refs: vec![evidence.clone()],
            human_benefit_report: HumanBenefitReport {
                state: OutcomeState::Unknown,
                user_visible_changes: vec![],
                affected_users: vec![],
                unknowns: vec![],
                evidence_refs: vec![evidence.clone()],
            },
            task_outcome_report: Some(report),
            failed_gate: None,
            recovery_condition: None,
            recovery_decision: None,
            historical_status: None,
            governance_reasons: vec![],
            finalization: None,
        };
        let input = outcome_render_input_from_outcome(directory.path(), outcome);
        let summary = render_human_outcome_with_view(&input, language, OutcomeRenderView::Summary);
        let full = render_human_outcome_with_view(&input, language, OutcomeRenderView::Full);
        assert!(summary.contains(expected), "summary={summary}");
        assert!(full.contains(expected), "full={full}");
        for value in ["v0.2.94", "passed", evidence.as_str()] {
            assert!(
                summary.contains(value),
                "summary missing {value}: {summary}"
            );
            assert!(full.contains(value), "full missing {value}: {full}");
        }
    }
}

#[test]
fn human_renderer_preserves_historical_and_superseded_distinctions() {
    let directory = repository();
    let historical = render_fixture(
        &directory,
        "WI-HISTORICAL",
        RenderFixture {
            state: OutcomeState::Unknown,
            decision_state: DecisionState::Yellow,
            historical_status: Some("runtime_historical".into()),
            sections: OutcomeReportSections::default(),
            unknowns: vec!["historical_evidence_not_revalidated".into()],
            evidence_refs: vec![".ai/evidence/historical.json".into()],
        },
    );
    assert!(historical.contains("Historical verification evidence was not revalidated"));
    assert!(!historical.contains("Repair the missing evidence"));
    assert!(historical.contains("not a current failure"));

    let superseded = render_fixture(
        &directory,
        "WI-SUPERSEDED",
        RenderFixture {
            state: OutcomeState::Unknown,
            decision_state: DecisionState::Yellow,
            historical_status: Some("superseded".into()),
            sections: OutcomeReportSections::default(),
            unknowns: vec!["historical_evidence_not_current".into()],
            evidence_refs: vec![".ai/evidence/superseded.json".into()],
        },
    );
    assert!(superseded.contains("Superseded historical item"));
    assert!(superseded.contains("successor owns follow-up work"));
    assert!(!superseded.contains("Repair the missing evidence"));
}

#[test]
fn human_renderer_keeps_partial_missing_and_stale_verification_distinct() {
    let directory = repository();
    let partial = render_fixture(
        &directory,
        "WI-PARTIAL",
        RenderFixture {
            state: OutcomeState::Partial,
            decision_state: DecisionState::Yellow,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec!["verification_scope_incomplete".into()],
            evidence_refs: vec![".ai/evidence/partial.json".into()],
        },
    );
    assert!(partial.contains("Outcome: 🟡 Partial verification — WI-PARTIAL"));
    assert!(partial.contains("verification_scope_incomplete"));

    let missing = render_fixture(
        &directory,
        "WI-MISSING",
        RenderFixture {
            state: OutcomeState::NotReady,
            decision_state: DecisionState::Yellow,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec!["required_evidence_missing".into()],
            evidence_refs: vec![],
        },
    );
    assert!(missing.contains("Outcome: 🟡 Verification not ready — WI-MISSING"));
    assert!(missing.contains("required_evidence_missing"));
    assert!(!missing.contains("No risks"));

    let stale = render_fixture(
        &directory,
        "WI-STALE",
        RenderFixture {
            state: OutcomeState::Unknown,
            decision_state: DecisionState::Red,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec!["evidence_snapshot_stale".into(), "identity_mismatch".into()],
            evidence_refs: vec![".ai/evidence/stale.json".into()],
        },
    );
    assert!(stale.contains("Outcome: 🔴 Verification status unknown — WI-STALE"));
    assert!(stale.contains("evidence_snapshot_stale"));
    assert!(stale.contains("identity_mismatch"));
    assert!(stale.contains("Verification evidence is expired"));
    assert!(stale.contains("Verification evidence identity does not match"));
    assert!(!stale.contains("Verification evidence is invalid or does not match"));
}

#[test]
fn human_renderer_names_acceptance_and_intent_gaps_without_invalidating_verification() {
    let directory = repository();
    let id = "WI-VALID-VERIFICATION-GAPS";
    start_work_item_with_options(
        directory.path(),
        id,
        "show the actual governance gap",
        "verification remains valid while acceptance and intent evidence are incomplete",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec![
                "A1: acceptance evidence is recorded".into(),
                "A2: intent alignment is recorded".into(),
            ],
            ..Default::default()
        },
    )
    .expect("start");
    preflight_work_item(
        directory.path(),
        &directory
            .path()
            .join(format!(".ai/work-items/active/{id}.contract.json")),
    )
    .expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true}),
        "0.2.89",
        &Digest::sha256_bytes(b"valid-verification"),
    )
    .expect("verification");

    let input =
        cockpit_repository::outcome_render_input(directory.path(), id).expect("assembled outcome");
    for language in ["en", "zh", "ja"] {
        for view in [OutcomeRenderView::Summary, OutcomeRenderView::Full] {
            let text = render_human_outcome_with_view(&input, language, view);
            assert!(
                !text.contains("Verification evidence is invalid or does not match"),
                "{language}/{view:?}: {text}"
            );
            assert!(
                text.contains("acceptance") || text.contains("验收") || text.contains("受入れ"),
                "{language}/{view:?} omitted acceptance gap: {text}"
            );
            assert!(
                text.contains("intent") || text.contains("意图") || text.contains("intent"),
                "{language}/{view:?} omitted intent gap: {text}"
            );
        }
    }
    let summary = render_human_outcome_with_view(&input, "en", OutcomeRenderView::Summary);
    assert!(
        summary.contains("Complete the acceptance-evidence mapping"),
        "{summary}"
    );
    assert!(!summary.contains("verify again"), "{summary}");
}

#[cfg(unix)]
#[test]
fn outcome_observation_rejects_symlinked_active_contract() {
    use std::os::unix::fs::symlink;

    let directory = repository();
    let id = "WI-OUTCOME-SYMLINK-CONTRACT";
    start_work_item_with_options(
        directory.path(),
        id,
        "reject a symlinked active Contract during Outcome observation",
        "preserve the original Contract path identity",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("start");

    let contract = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    let target = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.target.json"));
    fs::rename(&contract, &target).expect("move contract target");
    symlink(&target, &contract).expect("contract symlink");

    let error = cockpit_repository::outcome_render_input(directory.path(), id)
        .expect_err("Outcome observation must reject a symlinked active Contract");
    assert!(error.to_string().contains("regular non-symlink"), "{error}");
}

#[test]
fn report_is_typed_evidence_bound_and_serializable() {
    let directory = repository();
    ready(&directory, "WI-136-REPORT");
    let outcome = outcome_v2(directory.path(), "WI-136-REPORT").expect("outcome");
    let report = outcome.task_outcome_report.clone().expect("task report");
    assert_eq!(report.format, "ai-cockpit.task-outcome");
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.bindings.repository_id, outcome.repository_id);
    assert_eq!(report.bindings.work_item_id, outcome.work_item_id);
    assert!(!report.sections.outcome_summary.is_empty());
    assert!(!report.sections.warnings.is_empty());
    assert!(
        report
            .sections
            .outcome_summary
            .iter()
            .all(|claim| !claim.evidence_refs.is_empty() || claim.inference)
    );
    let encoded = serde_json::to_value(&outcome).expect("encode outcome");
    assert!(encoded.get("taskOutcomeReport").is_some());
}

#[test]
fn outcome_reads_only_explicit_evidence_bound_release_projection() {
    let directory = repository();
    let id = "WI-136-RELEASE-PROJECTION";
    ready(&directory, id);
    let evidence_ref = format!(".ai/evidence/{id}.release.json");
    fs::write(
        directory.path().join(&evidence_ref),
        b"{\"releasePublished\":true}",
    )
    .expect("release evidence");
    let summary_path = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.summary.json"));
    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("summary read"))
            .expect("summary json");
    summary["release"] = serde_json::json!({
        "version": "v0.2.94",
        "releaseLink": "https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.94",
        "installAcceptance": "passed",
        "upgradeAcceptance": "passed",
        "cleanupStatus": "passed",
        "evidenceRefs": [evidence_ref],
    });
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).expect("summary encode"),
    )
    .expect("summary write");

    let outcome = outcome_v2(directory.path(), id).expect("outcome");
    let release = outcome
        .task_outcome_report
        .as_ref()
        .and_then(|report| report.release.as_ref())
        .expect("release projection");
    assert_eq!(release.version.as_deref(), Some("v0.2.94"));
    assert_eq!(release.install_acceptance.as_deref(), Some("passed"));
    assert_eq!(release.cleanup_status.as_deref(), Some("passed"));

    // A release-shaped object without a bound evidence reference is not
    // exposed as a claim, even when it contains plausible prose.
    summary["release"]["evidenceRefs"] = serde_json::json!([]);
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).expect("summary encode"),
    )
    .expect("summary rewrite");
    let outcome = outcome_v2(directory.path(), id).expect("outcome without evidence");
    assert!(
        outcome
            .task_outcome_report
            .and_then(|report| report.release)
            .is_none()
    );
}

#[test]
fn finish_writes_event_stream_and_archive_binds_it() {
    let directory = repository();
    ready(&directory, "WI-136-EVENTS");
    finish_work_item(directory.path(), "WI-136-EVENTS").expect("finish");
    let active_report_bytes = fs::read(
        directory
            .path()
            .join(".ai/work-items/active/WI-136-EVENTS.task-report.json"),
    )
    .expect("active report bytes");
    let active_markdown_bytes = fs::read(
        directory
            .path()
            .join(".ai/work-items/active/WI-136-EVENTS.task-report.md"),
    )
    .expect("active Markdown bytes");
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-136-EVENTS.summary.json"),
        )
        .expect("active Summary"),
    )
    .expect("Summary JSON");
    let policy: cockpit_protocol::TaskOutcomeProjectionPolicy =
        serde_json::from_value(summary["taskOutcomeProjectionPolicy"].clone())
            .expect("finish Summary has a strict projection policy");
    let binding: cockpit_protocol::TaskOutcomeProjectionFinishBinding = serde_json::from_value(
        summary["lifecycleFacts"]["taskOutcomeProjectionFinishBinding"].clone(),
    )
    .expect("finish Summary has a typed projection binding");
    assert_eq!(
        binding.projection_policy_digest,
        cockpit_protocol::digest_json(&policy).expect("policy digest")
    );
    assert_eq!(
        binding.task_report_json_digest,
        Digest::sha256_bytes(&active_report_bytes)
    );
    assert_eq!(
        binding.task_report_markdown_digest,
        Digest::sha256_bytes(&active_markdown_bytes)
    );
    let report: TaskOutcomeReport = serde_json::from_slice(&active_report_bytes).expect("report");
    let usage = report
        .usage
        .expect("new finish report has explicit usage state");
    assert_eq!(usage.coverage, cockpit_protocol::UsageCoverage::Unknown);
    assert_eq!(usage.unknown_reasons, ["no_usage_receipts"]);
    assert_eq!(usage.totals.input_tokens, None);
    assert!(!usage.cutoff.is_empty());
    let active_events = directory
        .path()
        .join(".ai/work-items/active/WI-136-EVENTS.events.jsonl");
    assert!(active_events.is_file());
    assert!(
        directory
            .path()
            .join(".ai/work-items/active/WI-136-EVENTS.task-report.json")
            .is_file()
    );
    assert!(
        directory
            .path()
            .join(".ai/work-items/active/WI-136-EVENTS.task-report.md")
            .is_file()
    );
    let text = fs::read_to_string(&active_events).expect("events");
    assert!(text.contains("\"eventType\":\"completed\""));
    let completed_event: cockpit_protocol::TaskOutcomeEvent = text
        .lines()
        .map(|line| serde_json::from_str(line).expect("event JSON"))
        .find(|event: &cockpit_protocol::TaskOutcomeEvent| event.event_type == "completed")
        .expect("completion event");
    assert_eq!(completed_event.projection_binding.as_ref(), Some(&binding));
    archive_work_item(directory.path(), "WI-136-EVENTS").expect("archive");
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-136-EVENTS.task-report.json")
        )
        .expect("archived report bytes"),
        active_report_bytes
    );
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-136-EVENTS.task-report.md")
        )
        .expect("archived report Markdown bytes"),
        active_markdown_bytes
    );
    let archived_events = directory
        .path()
        .join(".ai/work-items/archive/WI-136-EVENTS.events.jsonl");
    assert!(archived_events.is_file());
    assert!(
        directory
            .path()
            .join(".ai/work-items/archive/WI-136-EVENTS.task-report.json")
            .is_file()
    );
    assert!(
        directory
            .path()
            .join(".ai/work-items/archive/WI-136-EVENTS.task-report.md")
            .is_file()
    );
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-136-EVENTS.archive.json"),
        )
        .expect("manifest"),
    )
    .expect("manifest JSON");
    assert!(manifest["files"]["eventsDigest"].is_string());

    let archived_report: TaskOutcomeReport = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-136-EVENTS.task-report.json"),
        )
        .expect("split archived report bytes"),
    )
    .expect("split archived report schema");
    let archived = outcome_v2(directory.path(), "WI-136-EVENTS")
        .expect("archive reader resolves the separate report and Markdown artifacts");
    let projected = archived
        .task_outcome_report
        .expect("archive reader exposes the frozen report");
    let mut expected_resolved = serde_json::to_value(&archived_report).expect("report value");
    fn resolve_fixture_paths(value: &mut serde_json::Value, work_item_id: &str) {
        match value {
            serde_json::Value::String(text) => {
                *text = text.replace(
                    &format!(".ai/work-items/active/{work_item_id}"),
                    &format!(".ai/work-items/archive/{work_item_id}"),
                );
            }
            serde_json::Value::Array(values) => {
                for value in values {
                    resolve_fixture_paths(value, work_item_id);
                }
            }
            serde_json::Value::Object(values) => {
                for value in values.values_mut() {
                    resolve_fixture_paths(value, work_item_id);
                }
            }
            _ => {}
        }
    }
    resolve_fixture_paths(&mut expected_resolved, "WI-136-EVENTS");
    let expected_resolved: TaskOutcomeReport =
        serde_json::from_value(expected_resolved).expect("resolved frozen report schema");
    assert_eq!(projected, expected_resolved);

    let manifest_bytes = fs::read(
        directory
            .path()
            .join(".ai/work-items/archive/WI-136-EVENTS.archive.json"),
    )
    .expect("archive manifest bytes");
    let repeated = archive_work_item(directory.path(), "WI-136-EVENTS")
        .expect_err("an already archived Work Item is not archived twice");
    assert!(repeated.to_string().contains("active") || repeated.to_string().contains("archive"));
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-136-EVENTS.archive.json")
        )
        .expect("manifest remains unchanged"),
        manifest_bytes
    );
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-136-EVENTS.task-report.json")
        )
        .expect("report remains unchanged after repeated archive"),
        active_report_bytes
    );
}

#[test]
fn archived_usage_projection_rejects_locally_recomputed_markdown_digests() {
    let directory = repository();
    let work_item_id = "WI-OUTCOME-USAGE-TAMPER";
    ready(&directory, work_item_id);
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");

    let archive = directory.path().join(".ai/work-items/archive");
    let markdown_path = archive.join(format!("{work_item_id}.task-report.md"));
    let markdown = fs::read_to_string(&markdown_path).expect("Markdown report");
    let tampered = markdown
        .split_once("## Usage\n")
        .map(|(prefix, _)| format!("{prefix}\n"))
        .expect("Usage section exists");
    fs::write(&markdown_path, tampered.as_bytes()).expect("tamper Markdown");

    let outcome_path = archive.join(format!("{work_item_id}.outcome.json"));
    let mut outcome: serde_json::Value =
        serde_json::from_slice(&fs::read(&outcome_path).expect("Outcome")).expect("Outcome JSON");
    let tampered_digest = Digest::sha256_bytes(tampered.as_bytes()).to_string();
    outcome["taskReportMarkdownDigest"] = tampered_digest.clone().into();
    fs::write(
        &outcome_path,
        serde_json::to_vec_pretty(&outcome).expect("serialize Outcome"),
    )
    .expect("write Outcome with recomputed local digest");
    let manifest_path = archive.join(format!("{work_item_id}.archive.json"));
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("manifest"))
            .expect("manifest JSON");
    manifest["files"]["taskReportMarkdownDigest"] = tampered_digest.into();
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("write manifest with recomputed local digest");

    let outcome = outcome_v2(directory.path(), work_item_id).expect("archived Outcome");
    assert_eq!(outcome.state, OutcomeState::Unknown);
    assert_eq!(outcome.decision_state, Some(DecisionState::Red));
}

#[test]
fn archived_projection_policy_removal_is_rejected_even_with_updated_manifest_digest() {
    let directory = repository();
    let work_item_id = "WI-OUTCOME-POLICY-DOWNGRADE";
    ready(&directory, work_item_id);
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");

    let archive = directory.path().join(".ai/work-items/archive");
    let summary_path = archive.join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    summary
        .as_object_mut()
        .expect("Summary object")
        .remove("taskOutcomeProjectionPolicy");
    let summary_bytes = serde_json::to_vec_pretty(&summary).expect("serialize Summary");
    fs::write(&summary_path, &summary_bytes).expect("remove policy");
    let manifest_path = archive.join(format!("{work_item_id}.archive.json"));
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("manifest"))
            .expect("manifest JSON");
    manifest["files"]["summaryDigest"] = Digest::sha256_bytes(&summary_bytes).to_string().into();
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("write manifest with recomputed local digest");

    let outcome = outcome_v2(directory.path(), work_item_id).expect("archived Outcome");
    assert_eq!(outcome.state, OutcomeState::Unknown);
    assert_eq!(outcome.decision_state, Some(DecisionState::Red));
}

#[test]
fn current_archive_cannot_be_relabelled_legacy_by_committing_self_selected_markers() {
    let directory = repository();
    let work_item_id = "WI-OUTCOME-LEGACY-SELECTOR";
    ready(&directory, work_item_id);
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");

    let archive = directory.path().join(".ai/work-items/archive");
    let evidence_path = directory
        .path()
        .join(format!(".ai/evidence/{work_item_id}.verification.json"));
    let original_evidence = fs::read(&evidence_path).expect("original verification receipt");
    let original_archive_paths = [
        format!(".ai/work-items/archive/{work_item_id}.archive.json"),
        format!(".ai/work-items/archive/{work_item_id}.contract.json"),
        format!(".ai/work-items/archive/{work_item_id}.summary.json"),
        format!(".ai/work-items/archive/{work_item_id}.outcome.json"),
        format!(".ai/work-items/archive/{work_item_id}.events.jsonl"),
        format!(".ai/work-items/archive/{work_item_id}.task-report.json"),
        format!(".ai/work-items/archive/{work_item_id}.task-report.md"),
        format!(".ai/evidence/{work_item_id}.verification.json"),
    ];
    let mut git_add = Command::new("git");
    git_add.arg("add").arg("-f").arg("--");
    git_add.args(&original_archive_paths);
    assert!(
        git_add
            .current_dir(directory.path())
            .status()
            .expect("stage original archive and receipt")
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
                "anchor original archive and verification receipt",
            ])
            .current_dir(directory.path())
            .status()
            .expect("commit original archive anchor")
            .success()
    );

    let summary_path = archive.join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    summary
        .as_object_mut()
        .expect("Summary object")
        .remove("taskOutcomeProjectionPolicy");
    summary["lifecycleFacts"]
        .as_object_mut()
        .expect("lifecycle facts")
        .remove("taskOutcomeProjectionFinishBinding");
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).expect("serialize Summary"),
    )
    .expect("remove policy and finish binding in fixture");

    let events_path = archive.join(format!("{work_item_id}.events.jsonl"));
    let events = fs::read_to_string(&events_path).expect("event stream");
    let downgraded_events = events
        .lines()
        .map(|line| {
            let mut event: serde_json::Value = serde_json::from_str(line).expect("event JSON");
            event
                .as_object_mut()
                .expect("event object")
                .remove("projectionBinding");
            serde_json::to_string(&event).expect("serialize event")
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(&events_path, &downgraded_events).expect("remove event bindings in fixture");

    let markdown_path = archive.join(format!("{work_item_id}.task-report.md"));
    let markdown = fs::read_to_string(&markdown_path).expect("Markdown");
    let legacy_markdown = markdown
        .split_once("## Usage\n")
        .map(|(prefix, _)| format!("{prefix}\n"))
        .expect("current report has a Usage section");
    fs::write(&markdown_path, legacy_markdown.as_bytes()).expect("legacy-style Markdown fixture");

    let outcome_path = archive.join(format!("{work_item_id}.outcome.json"));
    let mut outcome: serde_json::Value =
        serde_json::from_slice(&fs::read(&outcome_path).expect("Outcome")).expect("Outcome JSON");
    outcome["taskReportMarkdownDigest"] = Digest::sha256_bytes(legacy_markdown.as_bytes())
        .to_string()
        .into();
    let outcome_bytes = serde_json::to_vec_pretty(&outcome).expect("serialize Outcome");
    fs::write(&outcome_path, &outcome_bytes).expect("update local Outcome digest");

    let manifest_path = archive.join(format!("{work_item_id}.archive.json"));
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("manifest"))
            .expect("manifest JSON");
    manifest["files"]["summaryDigest"] =
        Digest::sha256_bytes(&fs::read(&summary_path).expect("rewritten Summary"))
            .to_string()
            .into();
    manifest["files"]["eventsDigest"] = Digest::sha256_bytes(downgraded_events.as_bytes())
        .to_string()
        .into();
    manifest["files"]["outcomeDigest"] = Digest::sha256_bytes(&outcome_bytes).to_string().into();
    manifest["files"]["taskReportMarkdownDigest"] =
        Digest::sha256_bytes(legacy_markdown.as_bytes())
            .to_string()
            .into();
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("recompute local manifest digests");

    // The fixture models a hostile but ordinary follow-up commit containing
    // only the rewritten archive. The original verification receipt remains
    // present at the earlier fixed revision and is byte-for-byte unchanged.
    let mut git_add = Command::new("git");
    git_add.arg("add").arg("-f").arg("--");
    git_add.args(&original_archive_paths[..7]);
    assert!(
        git_add
            .current_dir(directory.path())
            .status()
            .expect("stage tampered archive fixture")
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
                "tamper archive and claim legacy format",
            ])
            .current_dir(directory.path())
            .status()
            .expect("commit tampered archive fixture")
            .success()
    );

    let observed = outcome_v2(directory.path(), work_item_id).expect("read archived Outcome");
    assert_ne!(
        observed.state,
        OutcomeState::Verified,
        "a commit created after policy removal must not become a fixed historical legacy anchor"
    );
    assert_eq!(
        fs::read(evidence_path).expect("verification receipt unchanged"),
        original_evidence
    );
}

#[test]
fn fixed_wi754_outcome_digest_selects_the_later_bound_receipt() {
    const ARCHIVE_REVISION: &str = "74506e60f96dfcbc1352c186faefb9b8653e5a57";
    const RECEIPT_REVISION: &str = "b8c90f327e82597ecc78d982611426c6eb78b3c7";
    const WORK_ITEM_ID: &str = "WI-754-runtime-recovery-digest-revalidation";

    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let checkout = tempfile::tempdir().expect("temporary historical checkout root");
    let root = checkout.path().join("repository");
    clone_shared_no_checkout(&source, &root);
    preserve_ai_bytes_in_fixture(&root);
    assert!(
        Command::new("git")
            .args(["checkout", "--quiet", "--detach", "HEAD"])
            .current_dir(&root)
            .status()
            .expect("checkout current repository history")
            .success()
    );
    assert!(
        Command::new("git")
            .args(["merge-base", "--is-ancestor", ARCHIVE_REVISION, "HEAD"])
            .current_dir(&root)
            .status()
            .expect("check fixed archive ancestry")
            .success()
    );
    assert!(
        Command::new("git")
            .args(["merge-base", "--is-ancestor", RECEIPT_REVISION, "HEAD"])
            .current_dir(&root)
            .status()
            .expect("check fixed receipt ancestry")
            .success()
    );

    let archived_outcome_path = root.join(format!(
        ".ai/work-items/archive/{WORK_ITEM_ID}.outcome.json"
    ));
    let archived_outcome: serde_json::Value =
        serde_json::from_slice(&fs::read(&archived_outcome_path).expect("fixed Outcome bytes"))
            .expect("fixed Outcome JSON");
    assert_eq!(
        archived_outcome["evidenceDigest"],
        "sha256:3fe7857eeefb34d008d639920d731fb29341a543c89e02abbff50413645fe6d9"
    );
    let receipt_path = root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json"));
    let receipt_bytes = fs::read(&receipt_path).expect("current fixed receipt bytes");
    let receipt: serde_json::Value =
        serde_json::from_slice(&receipt_bytes).expect("fixed receipt JSON");
    assert_eq!(
        cockpit_protocol::digest_json(&receipt)
            .expect("canonical fixed receipt digest")
            .to_string(),
        archived_outcome["evidenceDigest"]
    );
    let archived_summary: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(format!(
            ".ai/work-items/archive/{WORK_ITEM_ID}.summary.json"
        )))
        .expect("fixed Summary bytes"),
    )
    .expect("fixed Summary JSON");
    assert_eq!(
        receipt["contractDigest"],
        archived_summary["preflightContractDigest"]
    );
    assert_eq!(
        receipt["repositorySnapshotDigest"],
        archived_summary["preflightRepositorySnapshotDigest"]
    );
    let fixed_report: TaskOutcomeReport = serde_json::from_slice(
        &fs::read(root.join(format!(
            ".ai/work-items/archive/{WORK_ITEM_ID}.task-report.json"
        )))
        .expect("fixed report bytes"),
    )
    .expect("fixed report schema");

    let runtime = RuntimeContext {
        runtime_version: "1.0.1-independent-reader".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"WI-754 independent reader"),
    };
    let outcome = cockpit_repository::outcome_v2_with_runtime(&root, WORK_ITEM_ID, &runtime)
        .expect("fixed archived report and its bound receipt remain readable");
    assert_eq!(
        outcome.historical_status.as_deref(),
        Some("runtime_historical")
    );
    assert_ne!(outcome.state, OutcomeState::Unknown);
    let observed_report = outcome
        .task_outcome_report
        .as_ref()
        .expect("the fixed receipt keeps the historical report readable");
    assert_eq!(observed_report.format, fixed_report.format);
    assert_eq!(observed_report.schema_version, fixed_report.schema_version);
    assert_eq!(observed_report.work_item_id, fixed_report.work_item_id);
    assert_eq!(observed_report.status, fixed_report.status);
    assert_eq!(observed_report.bindings, fixed_report.bindings);
    assert_eq!(
        observed_report.sections.evidence,
        fixed_report.sections.evidence
    );
}

#[test]
fn archived_task_report_unknown_schema_is_rejected_after_local_digest_rewrite() {
    let directory = repository();
    let work_item_id = "WI-OUTCOME-JSON-SCHEMA-DOWNGRADE";
    ready(&directory, work_item_id);
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");

    let archive = directory.path().join(".ai/work-items/archive");
    let report_path = archive.join(format!("{work_item_id}.task-report.json"));
    let mut report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).expect("report")).expect("report JSON");
    report["schemaVersion"] = serde_json::json!(999);
    let report_bytes = serde_json::to_vec_pretty(&report).expect("serialize report");
    fs::write(&report_path, &report_bytes).expect("set unsupported report schema");

    let report_digest = Digest::sha256_bytes(&report_bytes).to_string();
    let summary_path = archive.join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    summary["lifecycleFacts"]["taskOutcomeProjectionFinishBinding"]["taskReportJsonDigest"] =
        report_digest.clone().into();
    let summary_bytes = serde_json::to_vec_pretty(&summary).expect("serialize Summary");
    fs::write(&summary_path, &summary_bytes).expect("update local finish binding");

    let events_path = archive.join(format!("{work_item_id}.events.jsonl"));
    let events = fs::read_to_string(&events_path).expect("events");
    let rebound_events = events
        .lines()
        .map(|line| {
            let mut event: serde_json::Value = serde_json::from_str(line).expect("event JSON");
            if event["eventType"] == "completed" {
                event["projectionBinding"]["taskReportJsonDigest"] = report_digest.clone().into();
            }
            serde_json::to_string(&event).expect("serialize event")
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(&events_path, &rebound_events).expect("update local event binding");

    let outcome_path = archive.join(format!("{work_item_id}.outcome.json"));
    let mut outcome: serde_json::Value =
        serde_json::from_slice(&fs::read(&outcome_path).expect("Outcome")).expect("Outcome JSON");
    outcome["taskReportDigest"] = Digest::sha256_bytes(&report_bytes).to_string().into();
    let outcome_bytes = serde_json::to_vec_pretty(&outcome).expect("serialize Outcome");
    fs::write(&outcome_path, &outcome_bytes).expect("update local Outcome digest");

    let manifest_path = archive.join(format!("{work_item_id}.archive.json"));
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("manifest"))
            .expect("manifest JSON");
    manifest["files"]["taskReportDigest"] = report_digest.into();
    manifest["files"]["summaryDigest"] = Digest::sha256_bytes(&summary_bytes).to_string().into();
    manifest["files"]["eventsDigest"] = Digest::sha256_bytes(rebound_events.as_bytes())
        .to_string()
        .into();
    manifest["files"]["outcomeDigest"] = Digest::sha256_bytes(&outcome_bytes).to_string().into();
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("recompute local manifest digests");

    let observed = outcome_v2(directory.path(), work_item_id).expect("read archived Outcome");
    assert_ne!(
        observed.state,
        OutcomeState::Verified,
        "canonical JSON formatting does not validate the report schema version"
    );
}

#[test]
fn archived_task_report_truncation_is_rejected_after_local_digest_rewrite() {
    let directory = repository();
    let work_item_id = "WI-OUTCOME-JSON-TRUNCATED";
    ready(&directory, work_item_id);
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");

    let archive = directory.path().join(".ai/work-items/archive");
    let report_path = archive.join(format!("{work_item_id}.task-report.json"));
    let truncated = br#"{"format":"ai-cockpit.task-outcome","schemaVersion":1"#;
    fs::write(&report_path, truncated).expect("truncate report bytes");
    rewrite_archived_report_digest_chain(directory.path(), work_item_id, truncated);

    let observed = outcome_v2(directory.path(), work_item_id).expect("read archived Outcome");
    assert_ne!(
        observed.state,
        OutcomeState::Verified,
        "a truncated report remains invalid after local digest bindings are recomputed"
    );
}

#[test]
fn archived_projection_policy_downgrade_and_foreign_identity_are_rejected_after_rebinding() {
    for attack in [
        "policy_schema",
        "json_schema",
        "markdown_renderer",
        "repository",
        "contract",
        "source_snapshot",
        "producer_runtime",
    ] {
        let directory = repository();
        let work_item_id = format!("WI-PROJECTION-POLICY-{attack}").to_ascii_uppercase();
        ready(&directory, &work_item_id);
        finish_work_item(directory.path(), &work_item_id).expect("finish");
        archive_work_item(directory.path(), &work_item_id).expect("archive");

        let archive = directory.path().join(".ai/work-items/archive");
        let summary_path = archive.join(format!("{work_item_id}.summary.json"));
        let mut summary: serde_json::Value =
            serde_json::from_slice(&fs::read(&summary_path).expect("Summary"))
                .expect("Summary JSON");
        {
            let policy = &mut summary["taskOutcomeProjectionPolicy"];
            match attack {
                "policy_schema" => policy["schemaVersion"] = serde_json::json!(0),
                "json_schema" => policy["jsonSchemaVersion"] = serde_json::json!(2),
                "markdown_renderer" => policy["markdownRendererVersion"] = serde_json::json!(1),
                "repository" => {
                    policy["repositoryId"] = Digest::sha256_bytes(b"foreign repository")
                        .to_string()
                        .into()
                }
                "contract" => {
                    policy["contractDigest"] =
                        Digest::sha256_bytes(b"foreign Contract").to_string().into()
                }
                "source_snapshot" => {
                    policy["sourceSnapshotDigest"] =
                        Digest::sha256_bytes(b"foreign source snapshot")
                            .to_string()
                            .into()
                }
                "producer_runtime" => {
                    policy["producerRuntimeDigest"] =
                        Digest::sha256_bytes(b"foreign Runtime").to_string().into()
                }
                _ => unreachable!("test case is listed above"),
            }
        }
        if attack == "source_snapshot" {
            let foreign = summary["taskOutcomeProjectionPolicy"]["sourceSnapshotDigest"]
                .as_str()
                .expect("source snapshot")
                .to_owned();
            summary["checkpointRepositorySnapshotDigest"] = foreign.clone().into();
            summary["preflightRepositorySnapshotDigest"] = foreign.into();
        }
        let policy_digest = cockpit_protocol::digest_json(&summary["taskOutcomeProjectionPolicy"])
            .expect("policy digest");
        summary["lifecycleFacts"]["taskOutcomeProjectionFinishBinding"]["projectionPolicyDigest"] =
            policy_digest.to_string().into();
        let summary_bytes = serde_json::to_vec_pretty(&summary).expect("serialize Summary");
        fs::write(&summary_path, &summary_bytes).expect("rebind Summary");

        let events_path = archive.join(format!("{work_item_id}.events.jsonl"));
        let events = fs::read_to_string(&events_path).expect("events");
        let rebound_events = events
            .lines()
            .map(|line| {
                let mut event: serde_json::Value = serde_json::from_str(line).expect("event JSON");
                if event["eventType"] == "completed" {
                    event["projectionBinding"]["projectionPolicyDigest"] =
                        policy_digest.to_string().into();
                }
                serde_json::to_string(&event).expect("serialize event")
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        fs::write(&events_path, &rebound_events).expect("rebind completion event");

        let manifest_path = archive.join(format!("{work_item_id}.archive.json"));
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).expect("manifest"))
                .expect("manifest JSON");
        manifest["files"]["summaryDigest"] =
            Digest::sha256_bytes(&summary_bytes).to_string().into();
        manifest["files"]["eventsDigest"] = Digest::sha256_bytes(rebound_events.as_bytes())
            .to_string()
            .into();
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
        )
        .expect("recompute local manifest digests");

        let observed = outcome_v2(directory.path(), &work_item_id).expect("read archived Outcome");
        assert_ne!(
            observed.state,
            OutcomeState::Verified,
            "locally rebound {attack} policy must still fail against fixed Contract, source, Runtime, or supported versions"
        );
    }
}

#[test]
fn task_report_markdown_has_one_terminal_newline_in_active_and_archive() {
    let directory = repository();
    let work_item_id = "WI-136-REPORT-EOF";
    ready(&directory, work_item_id);
    finish_work_item(directory.path(), work_item_id).expect("finish");

    let active = fs::read_to_string(directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.task-report.md"
    )))
    .expect("active task report");
    assert!(active.ends_with('\n'));
    assert!(
        !active.ends_with("\n\n"),
        "active task report must not end with a blank line"
    );

    archive_work_item(directory.path(), work_item_id).expect("archive");
    let archived = fs::read_to_string(directory.path().join(format!(
        ".ai/work-items/archive/{work_item_id}.task-report.md"
    )))
    .expect("archived task report");
    assert!(archived.ends_with('\n'));
    assert!(
        !archived.ends_with("\n\n"),
        "archived task report must not end with a blank line"
    );
}

#[test]
fn malformed_or_foreign_event_stream_fails_archive_closed() {
    let directory = repository();
    ready(&directory, "WI-136-TAMPER");
    finish_work_item(directory.path(), "WI-136-TAMPER").expect("finish");
    let path = directory
        .path()
        .join(".ai/work-items/active/WI-136-TAMPER.events.jsonl");
    let mut line: serde_json::Value = serde_json::from_str(
        fs::read_to_string(&path)
            .expect("events")
            .lines()
            .next()
            .expect("event"),
    )
    .expect("event JSON");
    line["repositoryId"] = serde_json::Value::String("sha256:foreign".into());
    fs::write(&path, serde_json::to_vec(&line).expect("event JSON")).expect("tamper");
    assert!(archive_work_item(directory.path(), "WI-136-TAMPER").is_err());
}

#[test]
fn archived_report_tamper_is_red_and_not_reprojected_as_verified() {
    let directory = repository();
    ready(&directory, "WI-136-REPORT-TAMPER");
    finish_work_item(directory.path(), "WI-136-REPORT-TAMPER").expect("finish");
    archive_work_item(directory.path(), "WI-136-REPORT-TAMPER").expect("archive");
    let report_path = directory
        .path()
        .join(".ai/work-items/archive/WI-136-REPORT-TAMPER.task-report.json");
    let mut report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).expect("task report"))
            .expect("task report JSON");
    report["sections"]["warnings"] = serde_json::json!([
        {"text":"tampered", "evidenceRefs":[], "inference":true}
    ]);
    fs::write(
        &report_path,
        serde_json::to_vec_pretty(&report).expect("report JSON"),
    )
    .expect("tamper report");
    let outcome = outcome_v2(directory.path(), "WI-136-REPORT-TAMPER").expect("outcome");
    assert_eq!(outcome.state, cockpit_protocol::OutcomeState::Unknown);
    assert_eq!(
        outcome.decision_state,
        Some(cockpit_core::DecisionState::Red)
    );
    assert!(
        outcome
            .unknowns
            .iter()
            .any(|unknown| unknown == "outcome_report_invalid")
    );
}

#[test]
fn archived_artifact_single_byte_tamper_is_rejected_without_rewriting_bytes() {
    let directory = repository();
    let work_item_id = "WI-136-SINGLE-BYTE-TAMPER";
    ready(&directory, work_item_id);
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");

    let artifact_path = directory.path().join(format!(
        ".ai/work-items/archive/{work_item_id}.task-report.md"
    ));
    let original = fs::read(&artifact_path).expect("archived Markdown bytes");
    assert!(!original.is_empty(), "fixture report must contain bytes");
    let original_digest = Digest::sha256_bytes(&original);
    let mut tampered = original.clone();
    tampered[0] ^= 1;
    assert_eq!(
        original
            .iter()
            .zip(&tampered)
            .filter(|(left, right)| left != right)
            .count(),
        1,
        "fixture must change exactly one byte"
    );
    let tampered_digest = Digest::sha256_bytes(&tampered);
    assert_ne!(original_digest, tampered_digest);
    fs::write(&artifact_path, &tampered).expect("write exact one-byte mutation");

    let observed = outcome_v2(directory.path(), work_item_id).expect("read tampered archive");
    assert_eq!(observed.state, OutcomeState::Unknown);
    assert!(
        observed
            .unknowns
            .iter()
            .any(|unknown| unknown == "outcome_report_invalid"),
        "a single-byte artifact mutation with its declared digest unchanged must be rejected"
    );
    assert_eq!(
        fs::read(&artifact_path).expect("tampered bytes remain on disk"),
        tampered,
        "read-only projection must not rewrite or normalize the protected artifact"
    );
}
