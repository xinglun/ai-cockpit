use cockpit_core::Digest;
use cockpit_protocol::{
    CONTRACT_AMENDMENT_SCHEMA_VERSION, ContractAmendmentChange, ContractAmendmentOperation,
    ContractAmendmentRequest, PROTOCOL_VERSION, RuntimeContext,
};
use cockpit_repository::{
    WorkItemStartOptions, attach, record_recovery_decision,
    record_work_item_governance_controls_with_runtime, repository_id, start_work_item_with_options,
};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const WORK_ITEM_ID: &str = "WI-CLI-AMEND-PARITY";

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary repository");
    git(root.path(), &["init", "-q", "--initial-branch=main"]);
    git(
        root.path(),
        &["config", "user.email", "acceptance@example.invalid"],
    );
    git(root.path(), &["config", "user.name", "Acceptance"]);
    fs::write(root.path().join("README.md"), "amendment acceptance\n").expect("README");
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "initial"]);
    attach(root.path()).expect("attach repository");
    start_work_item_with_options(
        root.path(),
        WORK_ITEM_ID,
        "exercise typed Contract amendment adapters",
        "record a plan change identically through CLI and MCP",
        &["README.md".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["both adapters return one immutable receipt".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");
    root
}

fn candidate_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ai-cockpit"))
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(candidate_binary())
        .args(args)
        .output()
        .expect("candidate CLI process")
}

fn assert_success(output: &Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn candidate_runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(
            &fs::read(candidate_binary()).expect("read candidate executable"),
        ),
    }
}

fn typed_request(root: &Path) -> ContractAmendmentRequest {
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.contract.json"));
    let contract: Value =
        serde_json::from_slice(&fs::read(contract_path).expect("Contract")).expect("Contract JSON");
    ContractAmendmentRequest {
        schema_version: CONTRACT_AMENDMENT_SCHEMA_VERSION,
        change_id: "plan-change-001".into(),
        expected_contract_digest: cockpit_protocol::digest_json(&contract)
            .expect("Contract digest"),
        reason: "The implementation surfaced a more accurate acceptance plan.".into(),
        changes: vec![ContractAmendmentChange {
            path: "/goal".into(),
            operation: ContractAmendmentOperation::Set,
            value: Some(json!(
                "record and verify the implementation-plan adjustment"
            )),
        }],
    }
}

fn confirm_current_fixture_preflight(root: &Path) {
    let active = root.join(".ai/work-items/active");
    let contract: Value = serde_json::from_slice(
        &fs::read(active.join(format!("{WORK_ITEM_ID}.contract.json"))).expect("Contract"),
    )
    .expect("Contract JSON");
    let summary: Value = serde_json::from_slice(
        &fs::read(active.join(format!("{WORK_ITEM_ID}.summary.json"))).expect("Summary"),
    )
    .expect("Summary JSON");
    record_work_item_governance_controls_with_runtime(
        root,
        WORK_ITEM_ID,
        &json!({"decisionEvidence": {
            "schemaVersion": 1,
            "decisionId": "contract-preflight-review",
            "decision": "confirm_review",
            "workItemId": WORK_ITEM_ID,
            "repositoryId": repository_id(root),
            "contractDigest": cockpit_protocol::digest_json(&contract).expect("Contract digest"),
            "preflightDecisionDigest": summary["preflightDecisionDigest"],
            "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
            "recordedAt": "2026-10-19T00:00:00Z",
            "recordedBy": "human:isolated-test-fixture",
            "reason": "synthetic adapter test review for the exact current Contract"
        }}),
        &candidate_runtime(),
    )
    .expect("record exact synthetic Contract review");
}

const RETAINED_FIXTURE_CHECKS: [&str; 4] = [
    "cargo test --locked --offline --test material",
    "cargo fmt --all -- --check",
    "cargo test --locked --offline",
    "cargo clippy --locked --offline --all-targets -- -D warnings",
];

fn retained_cli(root: &Path, args: &[&str]) -> Output {
    retained_command(root, args)
        .output()
        .expect("retained-history fixture CLI")
}

fn retained_command(root: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(candidate_binary());
    command
        .args(args)
        .arg("--repo")
        .arg(root)
        .current_dir(root)
        .env("CARGO_TARGET_DIR", root.join(".ai/fixture-target"))
        .env("CARGO_BUILD_JOBS", "1")
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_NET_OFFLINE", "true");
    command
}

fn retained_document(root: &Path, suffix: &str) -> Value {
    serde_json::from_slice(
        &fs::read(root.join(format!(
            ".ai/work-items/active/{WORK_ITEM_ID}.{suffix}.json"
        )))
        .expect("native fixture document"),
    )
    .expect("native fixture JSON")
}

fn retained_amend(root: &Path, change_id: &str, changes: Value) {
    let contract = retained_document(root, "contract");
    let result = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_amend",
        json!({"workItemId": WORK_ITEM_ID, "request": {
            "schemaVersion": 1, "changeId": change_id,
            "expectedContractDigest": cockpit_protocol::digest_json(&contract).expect("digest"),
            "reason": "Isolated equivalent historical fixture; no real consumer authority.",
            "changes": changes
        }}),
    );
    assert_eq!(
        result.pointer("/result/isError"),
        Some(&json!(false)),
        "{result}"
    );
}

fn retained_material_input(root: &Path) -> Value {
    let output = retained_cli(
        root,
        &["work-item", "material-review", "plan", "--id", WORK_ITEM_ID],
    );
    assert_success(&output, "retained-history material plan");
    let request: Value = serde_json::from_slice(&output.stdout).expect("material request");
    assert_eq!(request["findingCodes"], json!([]), "{request}");
    assert_eq!(request["reviewEnabled"], true);
    json!({
        "schemaVersion": 1, "decision": "accept_permitted_unknowns",
        "requestDigest": request["requestDigest"],
        "reviewerActor": "agent:fixture-reviewer",
        "authoritySource": "test-fixture:bounded-review", "assurance": "self_declared",
        "evidenceRefs": [{"path": "tests/material.rs", "digest":
            Digest::sha256_bytes(&fs::read(root.join("tests/material.rs")).expect("fixture source"))}],
        "rationale": "Read the exact isolated fixture source and bounded syntax Unknown.",
        "residualRisk": "The bounded scanner is incomplete; synthetic fixture scope only."
    })
}

fn retained_record_material(root: &Path, input: &Value) -> Output {
    let file = tempfile::NamedTempFile::new().expect("material decision input");
    fs::write(file.path(), serde_json::to_vec(input).expect("input JSON")).expect("material input");
    retained_cli(
        root,
        &[
            "work-item",
            "material-review",
            "record",
            "--id",
            WORK_ITEM_ID,
            "--input",
            file.path().to_str().expect("input path"),
        ],
    )
}

struct RetainedCanonicalFixture {
    directory: tempfile::TempDir,
    old_evidence: Vec<u8>,
    old_outcome: Vec<u8>,
}

fn retained_canonical_amendment_fixture() -> RetainedCanonicalFixture {
    retained_canonical_amendment_fixture_with_source_change(false)
}

fn retained_canonical_amendment_fixture_with_source_change(
    change_source: bool,
) -> RetainedCanonicalFixture {
    let directory = tempfile::tempdir().expect("retained fixture");
    let root = directory.path();
    git(root, &["init", "-q", "--initial-branch=main"]);
    git(root, &["config", "user.name", "Retained Fixture"]);
    git(root, &["config", "user.email", "fixture@example.invalid"]);
    fs::write(
        root.join("README.md"),
        "Equivalent lifecycle fixture, not Sentinel.\n",
    )
    .expect("README");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"retained_fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("fixture manifest");
    fs::create_dir(root.join("src")).expect("source directory");
    fs::write(
        root.join("src/lib.rs"),
        "pub fn expected_date() -> &'static str {\n    \"2026-10-14\"\n}\n",
    )
    .expect("fixture library");
    let lock = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(root)
        .output()
        .expect("fixture lockfile");
    assert_success(
        &lock,
        "generate actual fixture Cargo.lock before source inventory",
    );
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "fixture source baseline"]);
    assert_success(&retained_cli(root, &["attach"]), "attach retained fixture");
    let mut start = vec![
        "start",
        "--id",
        WORK_ITEM_ID,
        "--intent",
        "Exercise preserved canonical verification recovery",
        "--goal",
        "Run all four checks through isolated fixture finish and archive",
        "--risk",
        "normal",
        "--authority",
        "authorized",
        "--out-of-scope",
        "No real consumer or human authorization",
        "--acceptance",
        "All four checks execute and history remains intact",
        "--required-evidence",
        "verification_receipt",
    ];
    for path in [
        "README.md",
        "Cargo.toml",
        "Cargo.lock",
        "src/lib.rs",
        "tests/material.rs",
    ] {
        start.extend(["--scope", path]);
    }
    for command in RETAINED_FIXTURE_CHECKS {
        start.extend(["--verification", command]);
    }
    assert_success(&retained_cli(root, &start), "start retained fixture");
    let mut scenario = json!({
        "scenario": "old fixture four-check cycle", "required": true,
        "status": "unverified", "evidence": [],
        "expected": "The old cycle executes every declared fixture check.",
        "verificationPlan": "Native verification of all four declared fixture commands."
    });
    retained_amend(
        root,
        "retained-profile-before-checkpoint",
        json!([
            {"path": "/governanceProfile", "operation": "set", "value": {"materialInspectionReview": {
                "schemaVersion": 1, "permittedUnknown": "repository_material_inspection_unavailable",
                "permittedCause": "readable_committed_rust_syntax_unknown", "assurance": "self_declared",
                "reviewerActor": "agent:fixture-reviewer", "authoritySource": "test-fixture:bounded-review",
                "acceptResidualRisk": true
            }}},
            {"path": "/scenarioCoverage", "operation": "add", "value": scenario}
        ]),
    );
    let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    assert_success(
        &retained_cli(root, &["preflight", "--contract", &relative]),
        "initial preflight",
    );
    confirm_current_fixture_preflight(root);
    record_work_item_governance_controls_with_runtime(
        root,
        WORK_ITEM_ID,
        &json!({"scenarioCoverage": [scenario]}),
        &candidate_runtime(),
    )
    .expect("planned fixture scenario");
    assert_success(
        &retained_cli(root, &["checkpoint", "--id", WORK_ITEM_ID]),
        "fixture checkpoint",
    );
    fs::create_dir(root.join("tests")).expect("tests directory");
    fs::write(root.join("tests/material.rs"), "fn material() -> String {\n    let mut label = String::from(\"token\");\n    label.push_str(\"ization\");\n    label\n}\n\n#[test]\nfn composed_label_matches() {\n    assert!(\n        std::env::var_os(\"RETAINED_FIXTURE_FAIL\").is_none(),\n        \"requested isolated fixture failure\"\n    );\n    assert_eq!(material(), \"tokenization\");\n    assert_eq!(retained_fixture::expected_date(), \"2026-10-14\");\n}\n")
        .expect("reviewable committed test source");
    git(root, &["add", "-A"]);
    git(
        root,
        &["commit", "-qm", "commit reviewable fixture material"],
    );
    assert_success(
        &retained_cli(root, &["preflight", "--contract", &relative]),
        "old-cycle preflight",
    );
    confirm_current_fixture_preflight(root);
    assert_success(
        &retained_record_material(root, &retained_material_input(root)),
        "old-cycle material receipt",
    );
    assert_success(
        &retained_cli(
            root,
            &["verify", "--work-item", WORK_ITEM_ID, "--workers", "1"],
        ),
        "old four real fixture checks",
    );
    let old_evidence =
        fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("old canonical exists before blocked Outcome");
    let envelope: Value = serde_json::from_slice(&old_evidence).expect("typed old envelope");
    assert_eq!(envelope["passed"], true);
    assert_eq!(envelope["receipt"]["nodesExecuted"], 4);
    assert_eq!(envelope["receipt"]["nodesReused"], 0);
    assert_eq!(envelope["receipt"]["processesSpawned"], 4);
    let finish = retained_cli(root, &["finish", "--id", WORK_ITEM_ID, "--json"]);
    assert!(
        !finish.status.success(),
        "unverified scenario must create a real failed finish"
    );
    let old_outcome =
        fs::read(root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.outcome.json")))
            .expect("native blocked Outcome");
    assert_eq!(
        serde_json::from_slice::<Value>(&old_outcome).expect("Outcome")["state"],
        "blocked"
    );
    fs::write(
        root.join(".ai/evidence/fixture-preserved-old-cycle.json"),
        &old_evidence,
    )
    .expect("retain actual old receipt bytes outside current pointer");
    fs::write(
        root.join(".ai/evidence/fixture-preserved-blocked-outcome.json"),
        &old_outcome,
    )
    .expect("retain actual blocked Outcome bytes");
    scenario["status"] = json!("verified");
    scenario["evidence"] = json!([".ai/evidence/fixture-preserved-old-cycle.json"]);
    record_work_item_governance_controls_with_runtime(
        root,
        WORK_ITEM_ID,
        &json!({"scenarioCoverage": [scenario]}),
        &candidate_runtime(),
    )
    .expect("record actual old-cycle scenario evidence");
    if change_source {
        fs::write(
            root.join("README.md"),
            "A committed source change requires fresh verification.\n",
        )
        .expect("source change before the current amendment/review");
        git(root, &["add", "README.md"]);
        git(
            root,
            &["commit", "-qm", "current source after the old verification"],
        );
    }
    retained_amend(
        root,
        "sensitive-after-retained-success-and-failed-finish",
        json!([
            {"path": "/acceptanceCriteria", "operation": "add", "value": "Reverify the amended plan before fixture finish and archive"}
        ]),
    );
    assert_success(
        &retained_cli(root, &["preflight", "--contract", &relative]),
        "amended preflight",
    );
    confirm_current_fixture_preflight(root);
    RetainedCanonicalFixture {
        directory,
        old_evidence,
        old_outcome,
    }
}

#[test]
fn retained_canonical_after_amendment_runs_four_checks_and_finishes_then_archives() {
    // A committed source change invalidates otherwise legitimate attempt reuse.
    // The real consumer also changes producer Runtime; this portable equivalent
    // fixture uses source freshness without fabricating a historical producer.
    let fixture = retained_canonical_amendment_fixture_with_source_change(true);
    let root = fixture.directory.path();
    let material = retained_record_material(root, &retained_material_input(root));
    assert_success(
        &material,
        "retained canonical must admit exact current material recovery",
    );
    let plan = retained_cli(
        root,
        &[
            "verify",
            "--work-item",
            WORK_ITEM_ID,
            "--workers",
            "1",
            "--plan-only",
        ],
    );
    assert_success(&plan, "plan four fresh checks");
    let plan: Value = serde_json::from_slice(&plan.stdout).expect("plan JSON");
    assert_eq!(plan["processesSpawned"], 0);
    assert_eq!(plan["nodesPlanned"], 4);
    assert_eq!(plan["nodesToExecute"], 4);
    assert_eq!(plan["nodesReused"], 0);
    assert_eq!(
        fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("old canonical"),
        fixture.old_evidence
    );
    assert_success(
        &retained_cli(
            root,
            &["verify", "--work-item", WORK_ITEM_ID, "--workers", "1"],
        ),
        "all four fresh checks",
    );
    let evidence: Value = serde_json::from_slice(
        &fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("fresh canonical"),
    )
    .expect("fresh envelope");
    assert_eq!(evidence["passed"], true);
    assert_eq!(evidence["receipt"]["nodesExecuted"], 4);
    assert_eq!(evidence["receipt"]["nodesReused"], 0);
    assert_eq!(evidence["receipt"]["processesSpawned"], 4);
    assert_eq!(evidence["receipt"]["maxConcurrentProcesses"], 1);
    assert_eq!(
        evidence["receipt"]["executionRecords"]
            .as_array()
            .expect("execution records")
            .len(),
        4
    );
    assert!(
        evidence["receipt"]["executionRecords"]
            .as_array()
            .expect("records")
            .iter()
            .all(|r| r["spawned"] == true && r["passed"] == true && r["exitCode"] == 0)
    );
    let summary = retained_document(root, "summary");
    assert_eq!(
        summary["preflightState"], "green",
        "fresh verification records current green preflight"
    );
    assert_eq!(
        summary["preflightContractDigest"],
        evidence["contractDigest"]
    );
    assert_eq!(
        summary["preflightRepositorySnapshotDigest"],
        evidence["repositorySnapshotDigest"]
    );
    assert_success(
        &retained_cli(root, &["finish", "--id", WORK_ITEM_ID, "--json"]),
        "actual fixture finish",
    );
    assert_success(
        &retained_cli(root, &["archive", "--id", WORK_ITEM_ID, "--json"]),
        "actual fixture archive",
    );
    assert_eq!(
        fs::read(root.join(".ai/evidence/fixture-preserved-old-cycle.json"))
            .expect("retained original receipt"),
        fixture.old_evidence
    );
    assert_eq!(
        fs::read(root.join(".ai/evidence/fixture-preserved-blocked-outcome.json"))
            .expect("retained original Outcome"),
        fixture.old_outcome
    );
    assert!(
        root.join(format!(
            ".ai/work-items/archive/{WORK_ITEM_ID}.archive.json"
        ))
        .is_file()
    );
}

#[test]
fn retained_history_with_unchanged_execution_bindings_preserves_valid_reuse() {
    let fixture = retained_canonical_amendment_fixture();
    let root = fixture.directory.path();
    let old: Value = serde_json::from_slice(&fixture.old_evidence).expect("old envelope");
    assert_success(
        &retained_record_material(root, &retained_material_input(root)),
        "same-source historical integrity still permits current material review",
    );
    let plan = retained_cli(
        root,
        &[
            "verify",
            "--work-item",
            WORK_ITEM_ID,
            "--workers",
            "1",
            "--plan-only",
        ],
    );
    assert_success(&plan, "valid existing attempt reuse");
    let plan: Value = serde_json::from_slice(&plan.stdout).expect("plan JSON");
    assert_eq!(plan["processesSpawned"], 0);
    assert_eq!(plan["nodesPlanned"], 4);
    assert_eq!(plan["nodesToExecute"], 0);
    assert_eq!(plan["nodesReused"], 4);
    assert_eq!(
        old["repositorySnapshotDigest"],
        retained_document(root, "summary")["preflightRepositorySnapshotDigest"]
    );
    assert_eq!(
        fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("old canonical is still historical"),
        fixture.old_evidence
    );
}

#[test]
fn retained_history_integrity_is_separate_from_current_source_freshness() {
    let fixture = retained_canonical_amendment_fixture_with_source_change(true);
    let root = fixture.directory.path();
    let old: Value = serde_json::from_slice(&fixture.old_evidence).expect("old envelope");
    let current = retained_cli(
        root,
        &["work-item", "status", "--id", WORK_ITEM_ID, "--json"],
    );
    assert_success(&current, "current source status");
    let current: Value = serde_json::from_slice(&current.stdout).expect("status");
    assert_ne!(
        old["repositorySnapshotDigest"],
        current["sourceDigests"]["repositorySnapshot"]
    );
    assert_success(
        &retained_record_material(root, &retained_material_input(root)),
        "a separately reviewed current source does not corrupt an intact historical receipt",
    );
    assert_eq!(
        fs::read(root.join(".ai/evidence/fixture-preserved-old-cycle.json")).expect("original"),
        fixture.old_evidence
    );
}

#[test]
fn retained_history_real_check_failure_cannot_finish_and_exact_retry_runs_fresh() {
    let fixture = retained_canonical_amendment_fixture_with_source_change(true);
    let root = fixture.directory.path();
    assert_success(
        &retained_record_material(root, &retained_material_input(root)),
        "current material decision before any retry",
    );
    let failed = retained_command(
        root,
        &["verify", "--work-item", WORK_ITEM_ID, "--workers", "1"],
    )
    .env("RETAINED_FIXTURE_FAIL", "1")
    .output()
    .expect("actual failing fixture verification");
    assert!(
        !failed.status.success(),
        "a real required-check failure must remain failed"
    );
    let failed_result: Value =
        serde_json::from_slice(&failed.stdout).expect("failed execution result");
    assert!(
        failed_result["receipt"]["processesSpawned"]
            .as_u64()
            .unwrap_or(0)
            > 0,
        "{failed_result}"
    );
    assert_eq!(failed_result["receipt"]["passed"], false);
    assert_eq!(
        fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("old canonical survives failed execution"),
        fixture.old_evidence
    );
    assert!(
        !retained_cli(root, &["finish", "--id", WORK_ITEM_ID, "--json"])
            .status
            .success()
    );
    let contract = retained_document(root, "contract");
    let summary = retained_document(root, "summary");
    let runtime = candidate_runtime();
    record_recovery_decision(root, WORK_ITEM_ID, &json!({
        "schemaVersion": 1, "decisionId": "work-item-recovery", "decision": "retry",
        "workItemId": WORK_ITEM_ID, "repositoryId": repository_id(root),
        "predecessorWorkItemId": WORK_ITEM_ID,
        "predecessorContractDigest": cockpit_protocol::digest_json(&contract).expect("Contract digest"),
        "predecessorSummaryDigest": cockpit_protocol::digest_json(&summary).expect("Summary digest"),
        "runtimeVersion": runtime.runtime_version, "runtimeDigest": runtime.runtime_digest,
        "actor": "human:isolated-fixture", "authoritySource": "test-fixture:actual-failed-check",
        "reason": "Retry the actual failed fixture execution after current material review; not a material bypass.",
        "decidedAt": "2026-10-19T00:00:00Z", "resumeCondition": "Run one fresh replacement fixture cycle"
    }), &runtime).expect("exact native retry after actual failure");
    assert_success(
        &retained_cli(
            root,
            &["verify", "--work-item", WORK_ITEM_ID, "--workers", "1"],
        ),
        "fresh exact retry without the fixture failure",
    );
    let current: Value = serde_json::from_slice(
        &fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("new canonical"),
    )
    .expect("new envelope");
    assert_eq!(current["receipt"]["nodesExecuted"], 4);
    assert_eq!(current["receipt"]["nodesReused"], 0);
    assert_eq!(current["passed"], true);
    assert_ne!(
        retained_document(root, "summary")["recoveryRetryPending"],
        true
    );
}

fn copy_retained_fixture(source: &RetainedCanonicalFixture) -> RetainedCanonicalFixture {
    fn copy_directory(source: &Path, target: &Path, relative: &Path) {
        for entry in fs::read_dir(source).expect("fixture directory") {
            let entry = entry.expect("fixture entry");
            let relative = relative.join(entry.file_name());
            if relative == Path::new(".ai/fixture-target") {
                continue;
            }
            let destination = target.join(entry.file_name());
            let kind = entry.file_type().expect("fixture entry type");
            if kind.is_dir() {
                fs::create_dir(&destination).expect("fixture copy directory");
                copy_directory(&entry.path(), &destination, &relative);
            } else {
                assert!(kind.is_file(), "fixture copies must not follow symlinks");
                fs::copy(entry.path(), destination).expect("exact fixture file copy");
            }
        }
    }
    let directory = tempfile::tempdir().expect("isolated negative copy");
    copy_directory(source.directory.path(), directory.path(), Path::new(""));
    RetainedCanonicalFixture {
        directory,
        old_evidence: source.old_evidence.clone(),
        old_outcome: source.old_outcome.clone(),
    }
}

fn assert_retained_history_rejection(baseline: &RetainedCanonicalFixture, case: &str) {
    let fixture = copy_retained_fixture(baseline);
    let root = fixture.directory.path();
    let initial = retained_cli(
        root,
        &["work-item", "status", "--id", WORK_ITEM_ID, "--json"],
    );
    assert_success(
        &initial,
        "read copied fixture admission before negative mutation",
    );
    let initial: Value = serde_json::from_slice(&initial.stdout).expect("initial copied status");
    assert!(
        initial["safeActions"]
            .as_array()
            .expect("actions")
            .contains(&json!("record_material_review_decision")),
        "copy must be eligible before its single negative mutation: {initial}"
    );
    let mut input = retained_material_input(root);
    let path = root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json"));
    let mut envelope: Value = serde_json::from_slice(&fixture.old_evidence).expect("old envelope");
    match case {
        "foreign_repository" => {
            envelope["repositoryId"] = json!(Digest::sha256_bytes(b"foreign repo"))
        }
        "foreign_work_item" => envelope["workItemId"] = json!("WI-FOREIGN"),
        "non_predecessor" => {
            envelope["contractDigest"] = json!(Digest::sha256_bytes(b"not an ancestor Contract"))
        }
        "receipt_digest" => {
            envelope["receiptDigest"] = json!(Digest::sha256_bytes(b"wrong receipt digest"))
        }
        "nested_failure" => {
            envelope["receipt"]["results"][0]["passed"] = json!(false);
            envelope["receiptDigest"] =
                json!(cockpit_protocol::digest_json(&envelope["receipt"]).expect("digest"));
        }
        "producer_runtime_identity" => {
            envelope["runtimeDigest"] = json!(Digest::sha256_bytes(b"wrong producer"))
        }
        "unsupported_schema" => envelope["evidenceSchemaVersion"] = json!(99),
        "unsupported_protocol" => envelope["protocolVersion"] = json!(99),
        "legacy_untyped" => envelope["captureMode"] = json!("legacy_untyped"),
        "digest_only" => {
            envelope["captureMode"] = json!("digest_only");
            envelope
                .as_object_mut()
                .expect("envelope")
                .remove("receipt");
        }
        "bad_marker" | "bad_transition" | "missing_current_review" => {
            let summary_path =
                root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
            let mut summary = retained_document(root, "summary");
            if case == "bad_marker" {
                summary["verificationInvalidatedByContractAmendment"]["contractHash"] =
                    json!(Digest::sha256_bytes(b"wrong marker"));
            } else if case == "bad_transition" {
                summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoints")
                    .last_mut()
                    .expect("revalidation")["previousContractHash"] =
                    json!(Digest::sha256_bytes(b"wrong transition"));
            } else {
                summary
                    .as_object_mut()
                    .expect("Summary")
                    .remove("decisionEvidence");
            }
            fs::write(
                summary_path,
                serde_json::to_vec_pretty(&summary).expect("Summary JSON"),
            )
            .expect("negative fixture mutation");
        }
        "bad_journal" => {
            let directory = root.join(format!(".ai/evidence/{WORK_ITEM_ID}.contract-amendments"));
            let committed = directory.join("00000002.committed.json");
            let mut value: Value =
                serde_json::from_slice(&fs::read(&committed).expect("committed record"))
                    .expect("record JSON");
            value["receipt"]["journalDigest"] = json!(Digest::sha256_bytes(b"wrong journal"));
            fs::write(
                committed,
                serde_json::to_vec_pretty(&value).expect("record JSON"),
            )
            .expect("negative journal mutation");
        }
        "dirty_source" => {
            fs::write(root.join("tests/material.rs"), "fn changed() {}\n").expect("dirty source")
        }
        "finding" => {
            fs::write(root.join("tests/material.rs"), include_str!("../../../tests/conformance/fixtures/repository-prompt-injection/repository/material.txt"))
                .expect("isolated Finding fixture");
            git(root, &["add", "tests/material.rs"]);
            git(root, &["commit", "-qm", "isolated committed Finding"]);
        }
        "stale_request" => input["requestDigest"] = json!(Digest::sha256_bytes(b"stale request")),
        "invalid_retry" => fs::write(
            root.join(format!(".ai/decisions/{WORK_ITEM_ID}.recovery.json")),
            br#"{"decision":"retry","fixture":"invalid"}"#,
        )
        .expect("invalid retry fixture"),
        "malformed_json" | "duplicate_keys" => {}
        #[cfg(unix)]
        "symlink" => {
            fs::remove_file(&path).expect("isolated negative only");
            std::os::unix::fs::symlink(
                root.join(".ai/evidence/fixture-preserved-old-cycle.json"),
                &path,
            )
            .expect("negative symlink fixture");
        }
        _ => panic!("unknown rejection case {case}"),
    }
    if matches!(case, "malformed_json") {
        fs::write(&path, b"{").expect("malformed envelope");
    } else if case == "duplicate_keys" {
        let text = serde_json::to_string(&envelope).expect("envelope");
        fs::write(&path, format!("{{\"passed\":true,{}", &text[1..]))
            .expect("duplicate-key envelope");
    } else if case != "symlink" {
        fs::write(
            &path,
            serde_json::to_vec_pretty(&envelope).expect("envelope JSON"),
        )
        .expect("negative envelope fixture");
    }
    let summary_before =
        fs::read(root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json")))
            .expect("Summary before rejection");
    let record = retained_record_material(root, &input);
    assert!(
        !record.status.success(),
        "{case} unexpectedly admitted material: {}",
        String::from_utf8_lossy(&record.stdout)
    );
    assert_eq!(
        fs::read(root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json")))
            .expect("Summary after rejection"),
        summary_before,
        "{case}"
    );
    let request_hex = input["requestDigest"]
        .as_str()
        .expect("request digest")
        .strip_prefix("sha256:")
        .expect("digest prefix");
    assert!(
        !root
            .join(format!(
                ".ai/evidence/material-inspection-review/{WORK_ITEM_ID}/{request_hex}.json"
            ))
            .exists(),
        "{case} published a successful current material receipt"
    );
    let plan = retained_cli(
        root,
        &[
            "verify",
            "--work-item",
            WORK_ITEM_ID,
            "--workers",
            "1",
            "--plan-only",
        ],
    );
    assert!(
        !plan.status.success(),
        "{case} unexpectedly admitted verification"
    );
    let output: Value = serde_json::from_slice(&plan.stdout).expect("precondition rejection JSON");
    let processes = output
        .pointer("/attempt/processesSpawned")
        .or_else(|| output.get("processesSpawned"));
    assert_eq!(processes, Some(&json!(0)), "{case}: {output}");
    assert_eq!(
        fs::read(root.join(".ai/evidence/fixture-preserved-old-cycle.json"))
            .expect("retained original"),
        fixture.old_evidence,
        "{case}"
    );
}

#[test]
fn retained_history_recovery_rejects_corruption_foreign_bindings_and_unauthorized_inputs() {
    let fixture = retained_canonical_amendment_fixture();
    for case in [
        "foreign_repository",
        "foreign_work_item",
        "non_predecessor",
        "receipt_digest",
        "nested_failure",
        "producer_runtime_identity",
        "unsupported_schema",
        "unsupported_protocol",
        "legacy_untyped",
        "digest_only",
        "bad_marker",
        "bad_transition",
        "bad_journal",
        "dirty_source",
        "finding",
        "missing_current_review",
        "stale_request",
        "invalid_retry",
        "malformed_json",
        "duplicate_keys",
    ] {
        assert_retained_history_rejection(&fixture, case);
    }
}

#[cfg(unix)]
#[test]
fn retained_history_recovery_rejects_symlinked_canonical_verification() {
    let fixture = retained_canonical_amendment_fixture();
    assert_retained_history_rejection(&fixture, "symlink");
}

fn assert_blocked_material_recovery_adapters(record_via_mcp: bool) {
    let directory = repository();
    let root = directory.path();
    let repo = root.to_str().expect("repository path");
    let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let contract_path = root.join(&relative);
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
        .expect("Contract JSON");
    let opt_in = json!({
        "schemaVersion": 1,
        "changeId": "adapter-material-opt-in",
        "expectedContractDigest": cockpit_protocol::digest_json(&contract).expect("digest"),
        "reason": "synthetic material opt-in before checkpoint",
        "changes": [{"path": "/governanceProfile", "operation": "set", "value": {
            "materialInspectionReview": {
                "schemaVersion": 1,
                "permittedUnknown": "repository_material_inspection_unavailable",
                "permittedCause": "readable_committed_rust_syntax_unknown",
                "assurance": "self_declared",
                "reviewerActor": "agent:fixture-reviewer",
                "authoritySource": "test-fixture:bounded-review",
                "acceptResidualRisk": true
            }
        }}]
    });
    let amended = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_amend",
        json!({"workItemId": WORK_ITEM_ID, "request": opt_in}),
    );
    assert_eq!(
        amended.pointer("/result/isError"),
        Some(&json!(false)),
        "{amended}"
    );
    assert_success(
        &run_cli(&["preflight", "--repo", repo, "--contract", &relative]),
        "initial fixture preflight",
    );
    confirm_current_fixture_preflight(root);
    assert_success(
        &run_cli(&["checkpoint", "--repo", repo, "--id", WORK_ITEM_ID]),
        "fixture checkpoint",
    );
    let failed = run_cli(&["finish", "--repo", repo, "--id", WORK_ITEM_ID]);
    assert!(
        !failed.status.success(),
        "fixture must persist a failed Outcome"
    );
    let blocked_path = root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.outcome.json"));
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&blocked_path).expect("Outcome"))
            .expect("Outcome JSON")["state"],
        "blocked"
    );

    fs::create_dir_all(root.join("src")).expect("source directory");
    fs::write(root.join("src/material.rs"),
        "fn material() -> String { let mut s = String::from(\"token\"); s.push_str(\"ization\"); s }\n")
        .expect("reviewable source");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", "commit reviewable material"]);
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
        .expect("Contract JSON");
    let scope_change = json!({
        "schemaVersion": 1,
        "changeId": "adapter-sensitive-scope",
        "expectedContractDigest": cockpit_protocol::digest_json(&contract).expect("digest"),
        "reason": "review the committed Rust source in the amended scope",
        "changes": [{"path": "/scope", "operation": "replace",
            "value": ["README.md", "src/material.rs"]}]
    });
    let amended = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_amend",
        json!({"workItemId": WORK_ITEM_ID, "request": scope_change}),
    );
    assert_eq!(
        amended.pointer("/result/isError"),
        Some(&json!(false)),
        "{amended}"
    );
    assert_success(
        &run_cli(&["preflight", "--repo", repo, "--contract", &relative]),
        "current amended preflight",
    );
    let before = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&before, "status before current human review");
    let before: Value = serde_json::from_slice(&before.stdout).expect("status JSON");
    assert!(
        !before["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("record_material_review_decision"))
    );
    confirm_current_fixture_preflight(root);
    let status = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&status, "CLI status after exact human review");
    let status: Value = serde_json::from_slice(&status.stdout).expect("CLI status JSON");
    assert_eq!(status["governanceState"], "red");
    assert!(
        status["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("record_material_review_decision")),
        "{status}"
    );
    assert!(
        !status["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("run_verification"))
    );
    let mcp_status = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_status",
        json!({"workItemId": WORK_ITEM_ID}),
    );
    assert_eq!(
        mcp_status.pointer("/result/isError"),
        Some(&json!(false)),
        "{mcp_status}"
    );
    assert_eq!(
        mcp_status["result"]["structuredContent"]["safeActions"],
        status["safeActions"]
    );

    let cli_plan = run_cli(&[
        "work-item",
        "material-review",
        "plan",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&cli_plan, "CLI material plan");
    let cli_plan: Value = serde_json::from_slice(&cli_plan.stdout).expect("CLI plan JSON");
    let mcp_plan = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_material_review_plan",
        json!({"workItemId": WORK_ITEM_ID}),
    );
    assert_eq!(
        mcp_plan.pointer("/result/isError"),
        Some(&json!(false)),
        "{mcp_plan}"
    );
    assert_eq!(
        mcp_plan["result"]["structuredContent"]["requestDigest"],
        cli_plan["requestDigest"]
    );
    let input = json!({
        "schemaVersion": 1,
        "decision": "accept_permitted_unknowns",
        "requestDigest": cli_plan["requestDigest"],
        "reviewerActor": "agent:fixture-reviewer",
        "authoritySource": "test-fixture:bounded-review",
        "assurance": "self_declared",
        "evidenceRefs": [{"path": "src/material.rs", "digest": cli_plan["materialManifestDigest"]}],
        "rationale": "The committed fixture has the exact bounded Unknown.",
        "residualRisk": "The bounded syntax scanner is incomplete."
    });
    let mut stale_input = input.clone();
    stale_input["requestDigest"] = json!(Digest::sha256_bytes(b"foreign material request"));
    let summary_path = root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    let summary_before_stale = fs::read(&summary_path).expect("Summary before stale input");
    let stale_file = tempfile::NamedTempFile::new().expect("stale input file");
    fs::write(
        stale_file.path(),
        serde_json::to_vec(&stale_input).expect("stale input JSON"),
    )
    .expect("stale input bytes");
    let stale_cli = run_cli(&[
        "work-item",
        "material-review",
        "record",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--input",
        stale_file.path().to_str().expect("stale input path"),
    ]);
    assert!(
        !stale_cli.status.success(),
        "foreign material request must fail"
    );
    let stale_mcp = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_material_review_record",
        json!({"workItemId": WORK_ITEM_ID, "decision": stale_input}),
    );
    assert_eq!(
        stale_mcp.pointer("/result/isError"),
        Some(&json!(true)),
        "{stale_mcp}"
    );
    assert_eq!(
        fs::read(&summary_path).expect("Summary after denied stale inputs"),
        summary_before_stale
    );
    let input_file = tempfile::NamedTempFile::new().expect("material input file");
    fs::write(
        input_file.path(),
        serde_json::to_vec(&input).expect("material input JSON"),
    )
    .expect("material input bytes");
    if record_via_mcp {
        let recorded = mcp_call(
            root,
            &candidate_runtime(),
            "work_item_material_review_record",
            json!({"workItemId": WORK_ITEM_ID, "decision": input.clone()}),
        );
        assert_eq!(
            recorded.pointer("/result/isError"),
            Some(&json!(false)),
            "{recorded}"
        );
    } else {
        let recorded = run_cli(&[
            "work-item",
            "material-review",
            "record",
            "--repo",
            repo,
            "--id",
            WORK_ITEM_ID,
            "--input",
            input_file.path().to_str().expect("input path"),
        ]);
        assert_success(&recorded, "CLI typed material receipt");
    }
    let after = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_status",
        json!({"workItemId": WORK_ITEM_ID}),
    );
    assert_eq!(
        after.pointer("/result/isError"),
        Some(&json!(false)),
        "{after}"
    );
    let after = &after["result"]["structuredContent"];
    assert_eq!(after["governanceState"], "red");
    assert!(
        after["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("run_verification")),
        "{after}"
    );
    assert!(
        !after["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("finish"))
    );
    let cli_after = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&cli_after, "CLI status after typed material receipt");
    let cli_after: Value = serde_json::from_slice(&cli_after.stdout).expect("CLI status JSON");
    assert_eq!(cli_after["safeActions"], after["safeActions"]);
    let summary_before_replay = fs::read(&summary_path).expect("Summary after material receipt");
    let replay = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_material_review_record",
        json!({"workItemId": WORK_ITEM_ID, "decision": input}),
    );
    assert_eq!(
        replay.pointer("/result/isError"),
        Some(&json!(true)),
        "{replay}"
    );
    assert_eq!(
        fs::read(&summary_path).expect("Summary after denied replay"),
        summary_before_replay
    );
}

#[test]
fn blocked_material_recovery_cli_and_mcp_share_exact_admission_and_receipt() {
    assert_blocked_material_recovery_adapters(false);
}

#[test]
fn blocked_material_recovery_mcp_record_matches_cli_and_rejects_replay() {
    assert_blocked_material_recovery_adapters(true);
}

fn mcp_call(root: &Path, runtime: &RuntimeContext, name: &str, arguments: Value) -> Value {
    cockpit_mcp::handle_request_for_repo(
        &json!({
            "jsonrpc":"2.0",
            "id":1,
            "method":"tools/call",
            "params":{"name":name,"arguments":arguments}
        }),
        root,
        runtime,
    )
}

fn amendment_history_bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let directory = root
        .join(".ai/evidence")
        .join(format!("{WORK_ITEM_ID}.contract-amendments"));
    if !directory.exists() {
        return Vec::new();
    }
    let mut records = fs::read_dir(directory)
        .expect("read amendment history")
        .map(|entry| {
            let path = entry.expect("history entry").path();
            let bytes = fs::read(&path).expect("history bytes");
            (path, bytes)
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.0.cmp(&right.0));
    records
}

#[test]
fn amend_help_exposes_typed_request_and_preserves_legacy_adapter() {
    let output = run_cli(&["work-item", "amend", "--help"]);
    assert_success(&output, "work-item amend --help");
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("--request"), "typed request missing: {help}");
    assert!(help.contains("--input"), "legacy input missing: {help}");
    assert!(help.contains("--reason"), "legacy reason missing: {help}");
}

#[test]
fn typed_cli_and_mcp_share_receipt_and_read_history_without_writes() {
    let root = repository();
    let request = typed_request(root.path());
    let request_value = serde_json::to_value(&request).expect("request JSON");
    let request_directory = tempfile::tempdir().expect("request directory");
    let request_path = request_directory.path().join("typed-amendment.json");
    fs::write(
        &request_path,
        serde_json::to_vec_pretty(&request_value).expect("serialize request"),
    )
    .expect("write request");

    let cli = run_cli(&[
        "work-item",
        "amend",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
        "--request",
        request_path.to_str().expect("request path"),
    ]);
    assert_success(&cli, "typed CLI amendment");
    let mut cli_receipt: Value = serde_json::from_slice(&cli.stdout).expect("CLI receipt");
    cli_receipt
        .as_object_mut()
        .expect("CLI receipt object")
        .remove("nextAction");

    let runtime = candidate_runtime();
    let mcp = mcp_call(
        root.path(),
        &runtime,
        "work_item_amend",
        json!({"workItemId":WORK_ITEM_ID,"request":request_value}),
    );
    assert_eq!(
        mcp.pointer("/result/isError"),
        Some(&Value::Bool(false)),
        "{mcp}"
    );
    assert_eq!(mcp.pointer("/result/structuredContent"), Some(&cli_receipt));

    let legacy_input_path = request_directory.path().join("legacy-additive.json");
    fs::write(
        &legacy_input_path,
        br#"{"scopeAppend":["src/legacy-adapter.rs"]}"#,
    )
    .expect("legacy additive input");
    let legacy = run_cli(&[
        "work-item",
        "amend",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
        "--input",
        legacy_input_path.to_str().expect("legacy input path"),
        "--reason",
        "Retain compatibility for existing additive callers.",
    ]);
    assert_success(&legacy, "legacy additive CLI amendment");

    let mut invalid_request = request_value.clone();
    invalid_request["unrecognized"] = json!(true);
    let invalid = mcp_call(
        root.path(),
        &runtime,
        "work_item_amend",
        json!({"workItemId":WORK_ITEM_ID,"request":invalid_request}),
    );
    assert_eq!(invalid.pointer("/result/isError"), Some(&Value::Bool(true)));
    assert!(
        invalid["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains("unknown field"),
        "{invalid}"
    );

    let stale = ContractAmendmentRequest {
        change_id: "plan-change-stale".into(),
        ..request.clone()
    };
    let stale_value = serde_json::to_value(&stale).expect("stale request JSON");
    let contract_path = root
        .path()
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.contract.json"));
    let before_conflict = fs::read(&contract_path).expect("Contract before conflict");
    let conflict = mcp_call(
        root.path(),
        &runtime,
        "work_item_amend",
        json!({"workItemId":WORK_ITEM_ID,"request":stale_value}),
    );
    assert_eq!(
        conflict.pointer("/result/isError"),
        Some(&Value::Bool(true))
    );
    assert!(
        conflict["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains("contract_digest_conflict"),
        "{conflict}"
    );
    assert_eq!(
        fs::read(&contract_path).expect("Contract after conflict"),
        before_conflict
    );

    let history_before = amendment_history_bytes(root.path());
    let cli_history = run_cli(&[
        "work-item",
        "amendments",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&cli_history, "read CLI amendment history");
    let cli_history: Value = serde_json::from_slice(&cli_history.stdout).expect("CLI history");
    assert_eq!(cli_history.as_array().map(Vec::len), Some(2));
    let mcp_history = mcp_call(
        root.path(),
        &runtime,
        "work_item_amendments",
        json!({"workItemId":WORK_ITEM_ID}),
    );
    assert_eq!(
        mcp_history.pointer("/result/structuredContent"),
        Some(&cli_history)
    );
    assert_eq!(amendment_history_bytes(root.path()), history_before);

    let legacy = RuntimeContext {
        runtime_version: "0.2.113".into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: "sha256:c85632062eb5ef8f8b39f6154b765c9e2543fe6b7ca3f54e107a59c174843686"
            .parse()
            .expect("installed predecessor digest"),
    };
    let unsupported = mcp_call(
        root.path(),
        &legacy,
        "work_item_environment_drift",
        json!({"action":"check","workItemId":WORK_ITEM_ID,"generation":1}),
    );
    assert_eq!(
        unsupported.pointer("/result/isError"),
        Some(&Value::Bool(true))
    );
    assert!(
        unsupported["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains("unsupported_runtime_capability"),
        "{unsupported}"
    );
}

#[test]
fn inspect_accepts_null_optional_concurrency_boundary_after_amendment() {
    let root = repository();
    let request = serde_json::to_vec_pretty(&typed_request(root.path()))
        .expect("serialize amendment request");
    let request_file = tempfile::NamedTempFile::new().expect("amendment request file");
    fs::write(request_file.path(), request).expect("write amendment request");

    let amended = run_cli(&[
        "work-item",
        "amend",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
        "--request",
        request_file.path().to_str().expect("request path"),
    ]);
    assert_success(&amended, "amend Work Item before inspect");

    let contract_path = root
        .path()
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.contract.json"));
    let contract: Value = serde_json::from_slice(&fs::read(contract_path).expect("read Contract"))
        .expect("Contract JSON");
    assert_eq!(contract["concurrencyBoundary"], Value::Null);

    let inspected = run_cli(&[
        "work-item",
        "inspect",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&inspected, "inspect amended Contract with null boundary");
}

fn assert_sensitive_amendment_blocks_cli_verification_before_command_spawn(retry_pending: bool) {
    let root = repository();
    let repository_path = root.path().to_str().expect("repository path");
    let contract_relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");

    let initial_preflight = run_cli(&[
        "preflight",
        "--repo",
        repository_path,
        "--contract",
        &contract_relative,
    ]);
    assert_success(&initial_preflight, "initial preflight");
    let checkpoint = run_cli(&[
        "checkpoint",
        "--repo",
        repository_path,
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&checkpoint, "checkpoint before amendment");

    let contract_path = root.path().join(&contract_relative);
    if retry_pending {
        let summary_path = root
            .path()
            .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
        let predecessor_contract: Value =
            serde_json::from_slice(&fs::read(&contract_path).expect("read Contract"))
                .expect("Contract JSON");
        let predecessor_summary: Value =
            serde_json::from_slice(&fs::read(&summary_path).expect("read Summary"))
                .expect("Summary JSON");
        let runtime = candidate_runtime();
        let retry = json!({
            "schemaVersion": 1,
            "decisionId": "work-item-recovery",
            "decision": "retry",
            "workItemId": WORK_ITEM_ID,
            "repositoryId": repository_id(root.path()).to_string(),
            "predecessorWorkItemId": WORK_ITEM_ID,
            "predecessorContractDigest": cockpit_protocol::digest_json(&predecessor_contract).expect("Contract digest"),
            "predecessorSummaryDigest": cockpit_protocol::digest_json(&predecessor_summary).expect("Summary digest"),
            "runtimeVersion": runtime.runtime_version,
            "runtimeDigest": runtime.runtime_digest,
            "actor": "human:fixture",
            "authoritySource": "test fixture",
            "reason": "retry a stale verification attempt",
            "decidedAt": "2026-10-03T00:00:00Z",
            "resumeCondition": "run replacement verification"
        });
        record_recovery_decision(root.path(), WORK_ITEM_ID, &retry, &runtime)
            .expect("bind retry decision through Runtime");
    }
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("read Contract"))
        .expect("Contract JSON");
    let request = ContractAmendmentRequest {
        schema_version: CONTRACT_AMENDMENT_SCHEMA_VERSION,
        change_id: "sensitive-scope-change".into(),
        expected_contract_digest: cockpit_protocol::digest_json(&contract)
            .expect("Contract digest"),
        reason: "scope expansion requires human policy review before verification".into(),
        changes: vec![ContractAmendmentChange {
            path: "/scope".into(),
            operation: ContractAmendmentOperation::Replace,
            value: Some(json!(["README.md", "src/**"])),
        }],
    };
    let request_file = tempfile::NamedTempFile::new().expect("amendment request file");
    fs::write(
        request_file.path(),
        serde_json::to_vec_pretty(&request).expect("serialize amendment request"),
    )
    .expect("write amendment request");
    let amended = run_cli(&[
        "work-item",
        "amend",
        "--repo",
        repository_path,
        "--id",
        WORK_ITEM_ID,
        "--request",
        request_file.path().to_str().expect("request path"),
    ]);
    assert_success(&amended, "record sensitive amendment");

    if retry_pending {
        let status = run_cli(&[
            "work-item",
            "status",
            "--repo",
            repository_path,
            "--id",
            WORK_ITEM_ID,
            "--json",
        ]);
        assert_success(&status, "status before retry amendment review preflight");
        let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
        assert!(
            status["safeActions"]
                .as_array()
                .expect("safe actions")
                .iter()
                .any(|action| action == "run_preflight"),
            "a pending retry and sensitive amendment must admit request-generating preflight: {status}"
        );
    }

    let preflight = run_cli(&[
        "preflight",
        "--repo",
        repository_path,
        "--contract",
        &contract_relative,
    ]);
    assert_success(
        &preflight,
        "Runtime-bound preflight should produce a pending amendment review request",
    );
    let preflight: Value = serde_json::from_slice(&preflight.stdout).expect("preflight JSON");
    let current_contract: Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("read amended Contract"))
            .expect("amended Contract JSON");
    let current_contract_digest =
        cockpit_protocol::digest_json(&current_contract).expect("amended Contract digest");
    assert_eq!(preflight["reviewState"], "needs_human_confirmation");
    assert_eq!(
        preflight["humanDecisionRequest"]["decisionId"],
        "contract-preflight-review"
    );
    assert_eq!(
        preflight["humanDecisionRequest"]["status"],
        "needs_human_confirmation"
    );
    assert_eq!(
        preflight["humanDecisionRequest"]["recommendedOption"],
        "confirm_review"
    );
    assert_eq!(preflight["actionAdmission"]["workItemId"], WORK_ITEM_ID);
    assert_eq!(
        preflight["actionAdmission"]["repositoryId"],
        current_contract["repositoryId"]
    );
    assert_eq!(
        preflight["actionAdmission"]["sourceDigests"]["contract"],
        current_contract_digest.as_str()
    );
    assert_eq!(
        preflight["actionAdmission"]["sourceDigests"]["repositorySnapshot"],
        preflight["actionAdmission"]["snapshotDigest"]
    );
    assert!(
        preflight["actionAdmission"]["snapshotDigest"]
            .as_str()
            .is_some_and(|digest| digest.starts_with("sha256:")),
        "request should be returned with a current repository snapshot binding"
    );
    let status = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repository_path,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&status, "status after refreshed preflight");
    let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert_eq!(status["humanDecisionRequired"], true);
    assert_eq!(status["humanDecisions"], json!([]));
    if retry_pending {
        let summary: Value = serde_json::from_slice(
            &fs::read(
                root.path()
                    .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json")),
            )
            .expect("read Summary"),
        )
        .expect("Summary JSON");
        assert_eq!(summary["recoveryRetryPending"], true);

        let current_receipt = json!({
            "schemaVersion": 1,
            "decisionId": "contract-preflight-review",
            "decision": "confirm_review",
            "workItemId": WORK_ITEM_ID,
            "repositoryId": repository_id(root.path()),
            "contractDigest": current_contract_digest,
            "preflightDecisionDigest": summary["preflightDecisionDigest"],
            "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
            "recordedAt": "2026-10-03T00:00:00Z",
            "recordedBy": "human:test-fixture",
            "reason": "fixture only; no human decision is recorded"
        });
        assert_eq!(
            current_receipt["repositorySnapshotDigest"],
            preflight["actionAdmission"]["snapshotDigest"]
        );
        let previous_contract_digest =
            cockpit_protocol::digest_json(&contract).expect("pre-amendment Contract digest");
        for (field, wrong_digest, expected_diagnostic) in [
            (
                "repositoryId",
                Digest::sha256_bytes(b"foreign repository").to_string(),
                "repository identity mismatch",
            ),
            (
                "contractDigest",
                previous_contract_digest.to_string(),
                "Contract digest mismatch",
            ),
            (
                "repositorySnapshotDigest",
                Digest::sha256_bytes(b"stale snapshot").to_string(),
                "snapshot digest mismatch",
            ),
        ] {
            let mut invalid_receipt = current_receipt.clone();
            invalid_receipt[field] = Value::String(wrong_digest);
            let summary_before = fs::read(
                root.path()
                    .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json")),
            )
            .expect("Summary before rejected decision");
            let error = record_work_item_governance_controls_with_runtime(
                root.path(),
                WORK_ITEM_ID,
                &json!({"decisionEvidence": invalid_receipt}),
                &candidate_runtime(),
            )
            .expect_err("foreign or stale review evidence must be rejected");
            assert!(
                error.to_string().contains(expected_diagnostic),
                "{field} should fail for its identity mismatch: {error}"
            );
            assert_eq!(
                fs::read(
                    root.path()
                        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json")),
                )
                .expect("Summary after rejected decision"),
                summary_before,
                "{field} rejection must not record a human decision"
            );
        }
        assert!(
            !root
                .path()
                .join(format!(
                    ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
                ))
                .exists(),
            "invalid review evidence must not create a decision receipt"
        );
    }
    assert!(
        !status["safeActions"]
            .as_array()
            .expect("safe actions")
            .iter()
            .any(|action| action == "run_verification")
    );

    let marker = root.path().join("verification-command-started");
    let mut verify_args = vec![
        "verify".to_owned(),
        "--repo".to_owned(),
        repository_path.to_owned(),
        "--work-item".to_owned(),
        WORK_ITEM_ID.to_owned(),
        "--command".to_owned(),
    ];
    #[cfg(unix)]
    {
        verify_args.push("sh".into());
        verify_args.push("--args=-c".into());
        verify_args.push(format!("--args=printf started > '{}'", marker.display()));
    }
    #[cfg(windows)]
    {
        verify_args.push("cmd".into());
        verify_args.push("--args=/C".into());
        verify_args.push(format!("--args=echo started > \"{}\"", marker.display()));
    }
    let verify_args = verify_args.iter().map(String::as_str).collect::<Vec<_>>();
    let verify = run_cli(&verify_args);
    assert!(
        !verify.status.success(),
        "unreviewed amendment must reject verify"
    );
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&verify.stdout),
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(
        diagnostic.contains("contract_amendment_policy_review_required"),
        "expected the amendment review gate to reject verification, got: {diagnostic}"
    );
    assert!(
        !marker.exists(),
        "verification command must not start before amendment review"
    );
}

#[test]
fn sensitive_amendment_blocks_cli_verification_before_command_spawn() {
    assert_sensitive_amendment_blocks_cli_verification_before_command_spawn(false);
}

#[test]
fn retry_pending_amendment_admits_bound_review_request_without_verification_spawn() {
    assert_sensitive_amendment_blocks_cli_verification_before_command_spawn(true);
}

fn assert_mixed_pre_amendment_review_binding_is_rejected(use_mcp: bool) {
    let root = repository();
    let repository_path = root.path().to_str().expect("repository path");
    let contract_relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let contract_path = root.path().join(&contract_relative);
    let summary_path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    let initial = run_cli(&[
        "preflight",
        "--repo",
        repository_path,
        "--contract",
        &contract_relative,
    ]);
    assert_success(&initial, "initial preflight before mixed-binding fixture");
    let checkpoint = run_cli(&[
        "checkpoint",
        "--repo",
        repository_path,
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&checkpoint, "checkpoint before mixed-binding fixture");
    let old_summary: Value = serde_json::from_slice(&fs::read(&summary_path).expect("old Summary"))
        .expect("old Summary JSON");
    let old_contract: Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("old Contract"))
            .expect("old Contract JSON");
    let runtime = candidate_runtime();
    let request = ContractAmendmentRequest {
        schema_version: CONTRACT_AMENDMENT_SCHEMA_VERSION,
        change_id: "mixed-binding-sensitive-amendment".into(),
        expected_contract_digest: cockpit_protocol::digest_json(&old_contract)
            .expect("old Contract digest"),
        reason: "isolated regression: invalidate old preflight before any new review".into(),
        changes: vec![ContractAmendmentChange {
            path: "/scope".into(),
            operation: ContractAmendmentOperation::Replace,
            value: Some(json!(["README.md", "src/**"])),
        }],
    };
    let amended = mcp_call(
        root.path(),
        &runtime,
        "work_item_amend",
        json!({"workItemId": WORK_ITEM_ID, "request": request}),
    );
    assert_eq!(
        amended.pointer("/result/isError"),
        Some(&json!(false)),
        "{amended}"
    );
    let new_contract_bytes = fs::read(&contract_path).expect("new Contract");
    let new_contract: Value =
        serde_json::from_slice(&new_contract_bytes).expect("new Contract JSON");
    let summary_before = fs::read(&summary_path).expect("invalidated Summary");
    let invalidated: Value =
        serde_json::from_slice(&summary_before).expect("invalidated Summary JSON");
    assert_eq!(invalidated["preflightState"], "not_run");
    assert_eq!(
        invalidated["preflightContractDigest"],
        old_summary["preflightContractDigest"]
    );
    assert_eq!(
        invalidated["preflightDecisionDigest"],
        old_summary["preflightDecisionDigest"]
    );
    let history_before = amendment_history_bytes(root.path());
    // This input exists only inside this temporary regression fixture.
    // A current Contract cannot turn the retained old preflight fields into
    // a new identity-bound human review before Runtime generates that review.
    let controls = json!({"decisionEvidence": {
        "schemaVersion": 1,
        "decisionId": "contract-preflight-review",
        "decision": "confirm_review",
        "workItemId": WORK_ITEM_ID,
        "repositoryId": repository_id(root.path()),
        "contractDigest": cockpit_protocol::digest_json(&new_contract).expect("new Contract digest"),
        "preflightDecisionDigest": old_summary["preflightDecisionDigest"],
        "repositorySnapshotDigest": old_summary["preflightRepositorySnapshotDigest"],
        "recordedAt": "2026-10-16T00:00:00Z",
        "recordedBy": "human:isolated-automated-test-fixture",
        "reason": "negative automated fixture only; not an actual human approval"
    }});
    if use_mcp {
        let response = mcp_call(
            root.path(),
            &runtime,
            "work_item_controls",
            json!({"workItemId": WORK_ITEM_ID, "controls": controls}),
        );
        assert_eq!(
            response.pointer("/result/isError"),
            Some(&json!(true)),
            "MCP must reject a new Contract mixed with pre-amendment preflight fields: {response}"
        );
    } else {
        let input = tempfile::NamedTempFile::new().expect("controls input");
        fs::write(
            input.path(),
            serde_json::to_vec(&controls).expect("controls JSON"),
        )
        .expect("write controls input");
        let response = run_cli(&[
            "work-item",
            "controls",
            "--repo",
            repository_path,
            "--id",
            WORK_ITEM_ID,
            "--input",
            input.path().to_str().expect("controls path"),
        ]);
        assert!(
            !response.status.success(),
            "CLI must reject a new Contract mixed with pre-amendment preflight fields; stdout={} stderr={}",
            String::from_utf8_lossy(&response.stdout),
            String::from_utf8_lossy(&response.stderr)
        );
    }
    assert_eq!(
        fs::read(&summary_path).expect("Summary after rejection"),
        summary_before
    );
    assert_eq!(
        fs::read(&contract_path).expect("Contract after rejection"),
        new_contract_bytes
    );
    assert_eq!(amendment_history_bytes(root.path()), history_before);
    assert!(
        !root
            .path()
            .join(format!(
                ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
            ))
            .exists(),
        "rejected mixed binding must not publish a decision receipt"
    );
}

#[test]
fn mixed_current_contract_and_pre_amendment_preflight_receipt_is_rejected_by_cli() {
    assert_mixed_pre_amendment_review_binding_is_rejected(false);
}

#[test]
fn mixed_current_contract_and_pre_amendment_preflight_receipt_is_rejected_by_mcp() {
    assert_mixed_pre_amendment_review_binding_is_rejected(true);
}

fn assert_recorded_preflight_review_boundary(
    state: Option<&str>,
    current_contract: bool,
    use_mcp: bool,
    accepted: bool,
) {
    let root = repository();
    let repo = root.path().to_str().expect("repository path");
    let contract_relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let contract_path = root.path().join(&contract_relative);
    let summary_path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    assert_success(
        &run_cli(&[
            "preflight",
            "--repo",
            repo,
            "--contract",
            &contract_relative,
        ]),
        "record fixture preflight",
    );
    let contract_bytes = fs::read(&contract_path).expect("Contract bytes");
    let contract: Value = serde_json::from_slice(&contract_bytes).expect("Contract JSON");
    let mut summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    // Isolated boundary fixtures vary the recorded field under test. The
    // native amendment reproduction above independently covers real history.
    if let Some(state) = state {
        summary["preflightState"] = json!(state);
    } else {
        summary
            .as_object_mut()
            .expect("Summary object")
            .remove("preflightState");
    }
    if !current_contract {
        summary["preflightContractDigest"] = json!(Digest::sha256_bytes(b"older Contract"));
    }
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).expect("fixture Summary"),
    )
    .expect("write isolated fixture");
    let before = fs::read(&summary_path).expect("Summary before record");
    let controls = json!({"decisionEvidence": {
        "schemaVersion": 1, "decisionId": "contract-preflight-review", "decision": "confirm_review",
        "workItemId": WORK_ITEM_ID, "repositoryId": repository_id(root.path()),
        "contractDigest": cockpit_protocol::digest_json(&contract).expect("Contract digest"),
        "preflightDecisionDigest": summary["preflightDecisionDigest"],
        "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
        "recordedAt": "2026-10-16T00:00:00Z", "recordedBy": "human:isolated-test-fixture",
        "reason": "automated boundary fixture; no real human approval"
    }});
    if use_mcp {
        let response = mcp_call(
            root.path(),
            &candidate_runtime(),
            "work_item_controls",
            json!({"workItemId":WORK_ITEM_ID,"controls":controls}),
        );
        assert_eq!(
            response.pointer("/result/isError"),
            Some(&json!(!accepted)),
            "state={state:?}, current_contract={current_contract}: {response}"
        );
    } else {
        let input = tempfile::NamedTempFile::new().expect("controls input");
        fs::write(
            input.path(),
            serde_json::to_vec(&controls).expect("controls JSON"),
        )
        .expect("controls input bytes");
        let response = run_cli(&[
            "work-item",
            "controls",
            "--repo",
            repo,
            "--id",
            WORK_ITEM_ID,
            "--input",
            input.path().to_str().expect("input path"),
        ]);
        assert_eq!(
            response.status.success(),
            accepted,
            "state={state:?}, current_contract={current_contract}: {}{}",
            String::from_utf8_lossy(&response.stdout),
            String::from_utf8_lossy(&response.stderr)
        );
    }
    let receipt_path = root.path().join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
    ));
    assert_eq!(
        fs::read(&contract_path).expect("unchanged Contract"),
        contract_bytes
    );
    if !accepted {
        assert_eq!(
            fs::read(&summary_path).expect("Summary after rejection"),
            before
        );
        assert!(
            !receipt_path.exists(),
            "rejection must not publish a receipt"
        );
    } else {
        let receipt = fs::read(&receipt_path).expect("actual fixture receipt");
        fs::write(
            root.path().join("README.md"),
            "changed source requires fresh execution, same Contract review remains valid\n",
        )
        .expect("source change");
        let status = run_cli(&[
            "work-item",
            "status",
            "--repo",
            repo,
            "--id",
            WORK_ITEM_ID,
            "--json",
        ]);
        assert_success(&status, "status after same-Contract source change");
        let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
        assert_eq!(
            status["humanDecisionRequired"], false,
            "same-Contract approval must be retained: {status}"
        );
        assert_eq!(fs::read(&receipt_path).expect("preserved receipt"), receipt);
    }
}

#[test]
fn missing_recorded_preflight_state_rejects_review_without_writes() {
    for mcp in [false, true] {
        assert_recorded_preflight_review_boundary(None, true, mcp, false);
    }
}

#[test]
fn unknown_recorded_preflight_state_rejects_review_without_writes() {
    for mcp in [false, true] {
        assert_recorded_preflight_review_boundary(Some("unknown"), true, mcp, false);
    }
}

#[test]
fn not_run_recorded_preflight_state_rejects_review_without_writes() {
    for mcp in [false, true] {
        assert_recorded_preflight_review_boundary(Some("not_run"), true, mcp, false);
    }
}

#[test]
fn foreign_recorded_preflight_contract_rejects_review_without_writes() {
    for mcp in [false, true] {
        assert_recorded_preflight_review_boundary(Some("yellow"), false, mcp, false);
    }
}

#[test]
fn valid_recorded_preflight_states_preserve_same_contract_approval() {
    for state in ["green", "yellow", "red"] {
        for mcp in [false, true] {
            assert_recorded_preflight_review_boundary(Some(state), true, mcp, true);
        }
    }
}

fn amended_request_fixture(red: bool) -> tempfile::TempDir {
    let root = repository();
    let repo = root.path().to_str().expect("repository path");
    let contract_relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    if !red {
        let project = root.path().join(".ai/project");
        fs::create_dir_all(&project).expect("project directory");
        fs::write(project.join("documentation-policy.json"), serde_json::to_vec(&json!({
            "schemaVersion":1, "repositoryId":repository_id(root.path()), "defaultProjection":"required",
            "requiredModes":["implementation"], "requiredOperations":[], "preserveExistingRegistrations":true
        })).expect("policy JSON")).expect("documentation-first fixture policy");
    }
    assert_success(
        &run_cli(&[
            "preflight",
            "--repo",
            repo,
            "--contract",
            &contract_relative,
        ]),
        "initial request fixture preflight",
    );
    assert_success(
        &run_cli(&["checkpoint", "--repo", repo, "--id", WORK_ITEM_ID]),
        "request fixture checkpoint",
    );
    let contract: Value =
        serde_json::from_slice(&fs::read(root.path().join(&contract_relative)).expect("Contract"))
            .expect("Contract JSON");
    let changes = vec![json!({"path":"/scope","operation":"add","value":"src/**"})];
    let request = json!({"schemaVersion":1,"changeId":"interlocked-request","expectedContractDigest":cockpit_protocol::digest_json(&contract).expect("digest"),"reason":"isolated request generation fixture","changes":changes});
    let result = mcp_call(
        root.path(),
        &candidate_runtime(),
        "work_item_amend",
        json!({"workItemId":WORK_ITEM_ID,"request":request}),
    );
    assert_eq!(
        result.pointer("/result/isError"),
        Some(&json!(false)),
        "{result}"
    );
    if red {
        // An intentionally contradictory retained envelope makes native
        // Outcome red. Request generation must preserve this evidence and
        // its blocker, never reinterpret it as a passed verification.
        fs::write(
            root.path()
                .join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")),
            br#"{"evidenceSchemaVersion":2,"fixture":"contradictory retained verification envelope"}"#,
        )
        .expect("contradictory evidence fixture");
    }
    root
}

fn assert_interlocked_request_generation(red: bool, use_mcp: bool) {
    let root = amended_request_fixture(red);
    let repo = root.path().to_str().expect("repository path");
    let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let summary_path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    let history = amendment_history_bytes(root.path());
    let evidence_path = root
        .path()
        .join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json"));
    let evidence_before = fs::read(&evidence_path).ok();
    let status = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&status, "interlocked status");
    let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert!(
        status["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("run_preflight")),
        "interlocked request generation must be admitted: {status}"
    );
    if red {
        assert_eq!(status["governanceState"], "red");
    }
    let mut previous_snapshot = None;
    for observation in 0..2 {
        if observation == 1 {
            fs::write(
                root.path().join("README.md"),
                "new current snapshot for repeated request\n",
            )
            .expect("source update");
        }
        let decision = if use_mcp {
            let response = mcp_call(
                root.path(),
                &candidate_runtime(),
                "preflight",
                json!({"contract":relative}),
            );
            assert_eq!(
                response.pointer("/result/isError"),
                Some(&json!(false)),
                "{response}"
            );
            response["result"]["structuredContent"].clone()
        } else {
            let response = run_cli(&["preflight", "--repo", repo, "--contract", &relative]);
            assert_success(&response, "generate interlocked request");
            serde_json::from_slice(&response.stdout).expect("decision JSON")
        };
        assert_eq!(decision["reviewState"], "needs_human_confirmation");
        assert_eq!(
            decision["humanDecisionRequest"]["decisionId"],
            "contract-preflight-review"
        );
        if red {
            assert_eq!(decision["state"], "red");
            assert_eq!(decision["outcome_state"], "blocked");
            assert!(
                !decision["blockers"]
                    .as_array()
                    .expect("blockers")
                    .is_empty()
            );
        }
        let summary: Value = serde_json::from_slice(&fs::read(&summary_path).expect("Summary"))
            .expect("Summary JSON");
        let contract: Value =
            serde_json::from_slice(&fs::read(root.path().join(&relative)).expect("Contract"))
                .expect("Contract JSON");
        assert_eq!(
            summary["preflightContractDigest"],
            json!(cockpit_protocol::digest_json(&contract).expect("Contract digest"))
        );
        if let Some(previous) = previous_snapshot {
            assert_ne!(
                summary["preflightRepositorySnapshotDigest"], previous,
                "repeat must bind the changed current source snapshot"
            );
        }
        previous_snapshot = Some(summary["preflightRepositorySnapshotDigest"].clone());
        assert!(
            summary.get("decisionEvidence").is_none(),
            "request generation never records confirmation"
        );
        assert_eq!(summary["checkpointCount"], 1);
        assert_eq!(amendment_history_bytes(root.path()), history);
        assert_eq!(fs::read(&evidence_path).ok(), evidence_before);
    }
    let after = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&after, "pending request status");
    let after: Value = serde_json::from_slice(&after.stdout).expect("status JSON");
    assert_eq!(after["humanDecisionRequired"], true);
    assert!(
        !after["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("run_verification"))
    );
    assert!(
        !root
            .path()
            .join(format!(
                ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
            ))
            .exists()
    );
}

#[test]
fn documentation_first_interlocked_request_is_generated_by_cli() {
    assert_interlocked_request_generation(false, false);
}

#[test]
fn documentation_first_interlocked_request_is_generated_by_mcp() {
    assert_interlocked_request_generation(false, true);
}

#[test]
fn governance_red_interlocked_request_is_generated_by_cli() {
    assert_interlocked_request_generation(true, false);
}

#[test]
fn governance_red_interlocked_request_is_generated_by_mcp() {
    assert_interlocked_request_generation(true, true);
}

#[test]
fn interlocked_request_rejects_invalid_provenance_without_writes() {
    for case in [
        "marker_missing",
        "marker_hash",
        "invalidated_checks",
        "revalidation_snapshot",
        "before_edit_hash",
        "summary_repo",
        "summary_wi",
        "unjournaled_contract",
        "journal_tampered",
        "journal_pending",
        "not_ready",
        "closed",
        "recovered",
        "archived",
    ] {
        let root = amended_request_fixture(true);
        let repo = root.path().to_str().expect("repository path");
        let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
        let contract_path = root.path().join(&relative);
        let summary_path = root
            .path()
            .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
        let mut summary: Value = serde_json::from_slice(&fs::read(&summary_path).expect("Summary"))
            .expect("Summary JSON");
        match case {
            "marker_missing" => {
                summary
                    .as_object_mut()
                    .expect("Summary object")
                    .remove("verificationInvalidatedByContractAmendment");
            }
            "marker_hash" => {
                summary["verificationInvalidatedByContractAmendment"]["contractHash"] =
                    json!(Digest::sha256_bytes(b"foreign Contract"))
            }
            "invalidated_checks" => {
                summary["verificationInvalidatedByContractAmendment"]["invalidatedRequiredChecks"] =
                    json!(["fabricated"]);
                summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoints")
                    .last_mut()
                    .expect("last checkpoint")["invalidatedRequiredChecks"] = json!(["fabricated"]);
            }
            "revalidation_snapshot" => {
                summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoints")
                    .last_mut()
                    .expect("last checkpoint")["repositorySnapshotDigest"] =
                    json!(Digest::sha256_bytes(b"foreign snapshot"))
            }
            "before_edit_hash" => {
                summary["checkpointEvidence"][0]["contractHash"] =
                    json!(Digest::sha256_bytes(b"foreign before-edit"))
            }
            "summary_repo" => {
                summary["repositoryId"] = json!(Digest::sha256_bytes(b"foreign repository"))
            }
            "summary_wi" => summary["workItemId"] = json!("WI-FOREIGN"),
            "unjournaled_contract" => {
                let mut contract: Value =
                    serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
                        .expect("Contract JSON");
                contract["goal"] = json!("unrecorded Contract change");
                fs::write(
                    &contract_path,
                    serde_json::to_vec(&contract).expect("Contract JSON"),
                )
                .expect("isolated Contract tamper");
            }
            "journal_tampered" => {
                let path = root.path().join(format!(
                    ".ai/evidence/{WORK_ITEM_ID}.contract-amendments/00000001.committed.json"
                ));
                let mut value: Value = serde_json::from_slice(&fs::read(&path).expect("journal"))
                    .expect("journal JSON");
                value["receipt"]["reason"] = json!("tampered");
                fs::write(path, serde_json::to_vec(&value).expect("journal JSON"))
                    .expect("isolated journal tamper");
            }
            "journal_pending" => fs::remove_file(root.path().join(format!(
                ".ai/evidence/{WORK_ITEM_ID}.contract-amendments/00000001.committed.json"
            )))
            .expect("isolated pending transaction"),
            phase => summary["state"] = json!(phase),
        }
        fs::write(
            &summary_path,
            serde_json::to_vec(&summary).expect("Summary JSON"),
        )
        .expect("isolated projection");
        let before_summary = fs::read(&summary_path).expect("Summary before rejection");
        let before_contract = fs::read(&contract_path).expect("Contract before rejection");
        let before_history = amendment_history_bytes(root.path());
        let cli = run_cli(&["preflight", "--repo", repo, "--contract", &relative]);
        assert!(
            !cli.status.success(),
            "{case} must not admit request generation"
        );
        let mcp = mcp_call(
            root.path(),
            &candidate_runtime(),
            "preflight",
            json!({"contract":relative}),
        );
        assert_eq!(
            mcp.pointer("/result/isError"),
            Some(&json!(true)),
            "{case}: {mcp}"
        );
        assert_eq!(
            fs::read(&summary_path).expect("Summary after rejection"),
            before_summary,
            "{case}"
        );
        assert_eq!(
            fs::read(&contract_path).expect("Contract after rejection"),
            before_contract,
            "{case}"
        );
        assert_eq!(
            amendment_history_bytes(root.path()),
            before_history,
            "{case}"
        );
    }
}

#[test]
fn interlocked_request_covers_each_eligible_nonterminal_phase() {
    for phase in ["implementation_active", "checkpointed", "finish_ready"] {
        let root = amended_request_fixture(true);
        let repo = root.path().to_str().expect("repository path");
        let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
        let path = root
            .path()
            .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
        let mut summary: Value =
            serde_json::from_slice(&fs::read(&path).expect("Summary")).expect("Summary JSON");
        summary["state"] = json!(phase);
        fs::write(&path, serde_json::to_vec(&summary).expect("Summary JSON"))
            .expect("isolated lifecycle projection");
        assert_success(
            &run_cli(&["preflight", "--repo", repo, "--contract", &relative]),
            phase,
        );
    }
}

fn assert_checkpoint_provenance_rejected(case: &str, use_mcp: bool) {
    let root = amended_request_fixture(true);
    let repo = root.path().to_str().expect("repository path");
    let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let summary_path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    let contract_path = root.path().join(&relative);
    let decision_path = root.path().join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
    ));
    let material_fallback = case.starts_with("count_");
    if material_fallback {
        // A real non-ancestor HEAD makes committed-material projection
        // unavailable. Keep the declared base and the native journal intact.
        git(
            root.path(),
            &["checkout", "--orphan", "material-unavailable"],
        );
        git(root.path(), &["add", "README.md"]);
        git(
            root.path(),
            &["commit", "-qm", "isolated non-ancestor material HEAD"],
        );
    }
    let read_status = || {
        let result = run_cli(&[
            "work-item",
            "status",
            "--repo",
            repo,
            "--id",
            WORK_ITEM_ID,
            "--json",
        ]);
        assert_success(&result, "checkpoint provenance status");
        serde_json::from_slice::<Value>(&result.stdout).expect("status JSON")
    };
    let control = read_status();
    assert!(
        control["safeActions"]
            .as_array()
            .expect("actions")
            .contains(&json!("run_preflight")),
        "untampered native checkpoint must admit review-request generation: {control}"
    );
    if material_fallback {
        assert!(
            control["blockers"]
                .as_array()
                .expect("blockers")
                .contains(&json!("material_review_projection_unavailable")),
            "the real material fallback must be active: {control}"
        );
    }
    let mut summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    match case {
        "missing" | "forged_digest" | "malformed" => {
            let value = match case {
                "forged_digest" => Some(json!(Digest::sha256_bytes(
                    b"same forged checkpoint source"
                ))),
                "malformed" => Some(json!("not-a-contract-digest")),
                _ => None,
            };
            for (stage, field) in [
                ("before_edit", "contractHash"),
                (
                    "contract_amendment_revalidation",
                    "originalBeforeEditContractHash",
                ),
            ] {
                let record = summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoints")
                    .iter_mut()
                    .rfind(|r| r["stage"] == stage)
                    .expect("source checkpoint");
                if let Some(value) = &value {
                    assert_ne!(
                        &record[field], value,
                        "the forged source must differ from native history"
                    );
                    record[field] = value.clone();
                } else {
                    record
                        .as_object_mut()
                        .expect("checkpoint object")
                        .remove(field);
                }
            }
        }
        "count_zero" => summary["checkpointCount"] = json!(0),
        "count_two" => summary["checkpointCount"] = json!(2),
        _ => panic!("unknown checkpoint provenance case"),
    }
    fs::write(
        &summary_path,
        serde_json::to_vec(&summary).expect("Summary JSON"),
    )
    .expect("isolated Summary tamper");
    let before_summary = fs::read(&summary_path).expect("before Summary");
    let before_contract = fs::read(&contract_path).expect("before Contract");
    let before_history = amendment_history_bytes(root.path());
    let before_decision = fs::read(&decision_path).ok();
    let status = read_status();
    let (rejected, response) = if use_mcp {
        let response = mcp_call(
            root.path(),
            &candidate_runtime(),
            "preflight",
            json!({"contract":relative}),
        );
        (
            response.pointer("/result/isError") == Some(&json!(true)),
            response.to_string(),
        )
    } else {
        let response = run_cli(&["preflight", "--repo", repo, "--contract", &relative]);
        (
            !response.status.success(),
            String::from_utf8_lossy(&response.stdout).into_owned(),
        )
    };
    assert!(
        rejected,
        "{case} transport={} must reject request generation; status={status}; response={response}",
        if use_mcp { "MCP" } else { "CLI" }
    );
    assert!(
        !status["safeActions"]
            .as_array()
            .expect("actions")
            .contains(&json!("run_preflight")),
        "{case} must not regain preflight through material fallback: {status}"
    );
    assert_eq!(
        fs::read(&summary_path).expect("after Summary"),
        before_summary,
        "{case}"
    );
    assert_eq!(
        fs::read(&contract_path).expect("after Contract"),
        before_contract,
        "{case}"
    );
    assert_eq!(
        amendment_history_bytes(root.path()),
        before_history,
        "{case}"
    );
    assert_eq!(fs::read(&decision_path).ok(), before_decision, "{case}");
}

#[test]
fn checkpoint_source_pair_missing_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("missing", false);
}
#[test]
fn checkpoint_source_pair_missing_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("missing", true);
}
#[test]
fn checkpoint_source_pair_forged_digest_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("forged_digest", false);
}
#[test]
fn checkpoint_source_pair_forged_digest_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("forged_digest", true);
}
#[test]
fn checkpoint_source_pair_malformed_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("malformed", false);
}
#[test]
fn checkpoint_source_pair_malformed_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("malformed", true);
}
#[test]
fn checkpoint_count_zero_material_fallback_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("count_zero", false);
}
#[test]
fn checkpoint_count_zero_material_fallback_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("count_zero", true);
}
#[test]
fn checkpoint_count_two_material_fallback_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("count_two", false);
}
#[test]
fn checkpoint_count_two_material_fallback_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("count_two", true);
}

fn activation_checkpoint_fixture() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("activation fixture directory");
    let root = directory.path().join("repo");
    fs::create_dir(&root).expect("fixture repository");
    let origin = directory.path().join("origin.git");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    git(&root, &["init", "-q", "--initial-branch=main"]);
    git(&root, &["config", "user.name", "Activation fixture"]);
    git(
        &root,
        &["config", "user.email", "activation@example.invalid"],
    );
    fs::write(root.join("README.md"), "isolated activation fixture\n").expect("README");
    git(&root, &["add", "README.md"]);
    git(&root, &["commit", "-qm", "fixture baseline"]);
    git(
        &root,
        &[
            "init",
            "-q",
            "--bare",
            "--initial-branch=main",
            origin.to_str().expect("origin path"),
        ],
    );
    git(
        &root,
        &[
            "remote",
            "add",
            "origin",
            origin.to_str().expect("origin path"),
        ],
    );
    git(&root, &["push", "-q", "-u", "origin", "main"]);
    git(&root, &["remote", "set-head", "origin", "-a"]);
    git(&root, &["checkout", "-q", "-b", "fixture/activation"]);
    let cli = |args: &[&str]| {
        Command::new(candidate_binary())
            .args(args)
            .arg("--repo")
            .arg(&root)
            .output()
            .expect("native fixture command")
    };
    assert_success(&cli(&["attach"]), "attach activation fixture");
    assert_success(
        &cli(&["work-item", "new", "--id", id, "--mode", "implementation"]),
        "create real not_ready scaffold",
    );
    let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("scaffold"))
        .expect("scaffold JSON");
    assert_eq!(contract["state"], "not_ready");
    let intent = "Exercise a real native activation boundary";
    let goal = "Keep post-checkpoint amendments reviewable";
    let acceptance = "The activation boundary preserves strict native amendment admission";
    let request_path = directory.path().join("before-activation.json");
    fs::write(
        &request_path,
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "changeId": "before-activation",
            "expectedContractDigest": cockpit_protocol::digest_json(&contract).expect("digest"),
            "reason": "Isolated test plan; no real Work Item or human authority.",
            "changes": [
                {"path":"/intent","operation":"set","value":intent},
                {"path":"/goal","operation":"set","value":goal},
                {"path":"/scope","operation":"replace","value":["README.md"]},
                {"path":"/outOfScope","operation":"replace","value":[]},
                {"path":"/risk","operation":"set","value":"normal"},
                {"path":"/authority","operation":"set","value":"authorized"},
                {"path":"/acceptanceCriteria","operation":"replace","value":[acceptance]},
                {"path":"/verification","operation":"replace","value":["git diff --check"]}
            ]
        }))
        .expect("request JSON"),
    )
    .expect("pre-activation request");
    assert_success(
        &cli(&[
            "work-item",
            "amend",
            "--id",
            id,
            "--request",
            request_path.to_str().expect("request path"),
        ]),
        "record pre-activation amendment",
    );
    assert_success(
        &cli(&[
            "start",
            "--id",
            id,
            "--intent",
            intent,
            "--goal",
            goal,
            "--scope",
            "README.md",
            "--risk",
            "normal",
            "--authority",
            "authorized",
            "--acceptance",
            acceptance,
            "--verification",
            "git diff --check",
            "--prepare",
        ]),
        "activate real scaffold and prepare review",
    );
    let active: Value = serde_json::from_slice(&fs::read(&contract_path).expect("active Contract"))
        .expect("active JSON");
    let summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    let review_path = directory.path().join("fixture-review.json");
    fs::write(
        &review_path,
        serde_json::to_vec_pretty(&json!({"decisionEvidence": {
            "schemaVersion": 1,
            "decisionId": "contract-preflight-review",
            "decision": "confirm_review",
            "workItemId": id,
            "repositoryId": active["repositoryId"],
            "contractDigest": cockpit_protocol::digest_json(&active).expect("active digest"),
            "preflightDecisionDigest": summary["preflightDecisionDigest"],
            "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
            "recordedAt": "2026-10-17T00:00:00Z",
            "recordedBy": "fixture:activation-test-only",
            "reason": "Synthetic isolated fixture review; not a real human response or history proof."
        }}))
        .expect("fixture review JSON"),
    )
    .expect("fixture review");
    assert_success(
        &cli(&[
            "work-item",
            "controls",
            "--id",
            id,
            "--input",
            review_path.to_str().expect("review path"),
        ]),
        "record fixture-only review",
    );
    assert_success(
        &cli(&[
            "preflight",
            "--contract",
            contract_path.to_str().expect("Contract path"),
        ]),
        "preflight reviewed fixture",
    );
    assert_success(&cli(&["checkpoint", "--id", id]), "fixture checkpoint");
    let post_path = directory.path().join("after-checkpoint.json");
    fs::write(
        &post_path,
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "changeId": "after-checkpoint",
            "expectedContractDigest": cockpit_protocol::digest_json(&active).expect("digest"),
            "reason": "Keep the amended fixture plan reviewable without granting its review.",
            "changes": [{"path":"/goal","operation":"set","value":"Review the current amended fixture plan"}]
        }))
        .expect("post-checkpoint JSON"),
    )
    .expect("post-checkpoint request");
    let amended = cli(&[
        "work-item",
        "amend",
        "--id",
        id,
        "--request",
        post_path.to_str().expect("post request path"),
    ]);
    let status = cli(&["work-item", "status", "--id", id, "--json"]);
    let history = cli(&["work-item", "amendments", "--id", id]);
    assert_success(&history, "read intact amendment journal");
    let history: Vec<Value> = serde_json::from_slice(&history.stdout).expect("history JSON");
    assert_eq!(history.len(), 2);
    assert_eq!(
        history[1]["previousJournalDigest"],
        history[0]["journalDigest"]
    );
    if !amended.status.success() || !status.status.success() {
        let preserved = directory.keep();
        panic!(
            "real activation boundary must keep amendment admission reviewable; fixture preserved at {}; amendment stderr={}; status stderr={}",
            preserved.display(),
            String::from_utf8_lossy(&amended.stderr),
            String::from_utf8_lossy(&status.stderr)
        );
    }
    let status: Value = serde_json::from_slice(&status.stdout).expect("current status JSON");
    assert_eq!(status["humanDecisionRequired"], true, "{status}");
    assert!(
        !status["safeActions"]
            .as_array()
            .expect("safe actions")
            .iter()
            .any(|action| action == "run_verification"),
        "the new Contract still needs its own human review: {status}"
    );
    directory
}

#[test]
fn native_activation_keeps_post_checkpoint_amendment_reviewable() {
    activation_checkpoint_fixture();
}

#[test]
fn current_contract_review_cannot_hide_a_missing_activation_commit() {
    let directory = activation_checkpoint_fixture();
    let root = directory.path().join("repo");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let cli = |args: &[&str]| {
        Command::new(candidate_binary())
            .args(args)
            .arg("--repo")
            .arg(&root)
            .output()
            .expect("native fixture command")
    };
    assert_success(
        &cli(&[
            "preflight",
            "--contract",
            contract_path.to_str().expect("path"),
        ]),
        "prepare current Contract review",
    );
    let current: Value = serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
        .expect("Contract JSON");
    let summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    let review_path = directory.path().join("current-fixture-review.json");
    fs::write(&review_path, serde_json::to_vec(&json!({"decisionEvidence": {
        "schemaVersion":1, "decisionId":"contract-preflight-review", "decision":"confirm_review",
        "workItemId":id, "repositoryId":current["repositoryId"],
        "contractDigest":cockpit_protocol::digest_json(&current).expect("digest"),
        "preflightDecisionDigest":summary["preflightDecisionDigest"],
        "repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],
        "recordedAt":"2026-10-17T00:00:00Z", "recordedBy":"fixture:current-review-only",
        "reason":"Synthetic current Contract review never authenticates a missing activation phase."
    }})).expect("review JSON")).expect("review source");
    assert_success(
        &cli(&[
            "work-item",
            "controls",
            "--id",
            id,
            "--input",
            review_path.to_str().expect("path"),
        ]),
        "record current fixture review",
    );
    let before_contract = fs::read(&contract_path).expect("Contract bytes");
    let before_summary = fs::read(&summary_path).expect("Summary bytes");
    fs::remove_file(root.join(format!(
        ".ai/evidence/{id}.activation-transition/committed.json"
    )))
    .expect("simulate isolated partial publication");
    let result = cli(&["work-item", "status", "--id", id, "--json"]);
    assert!(
        !result.status.success(),
        "current review must not hide missing activation commit: {}",
        String::from_utf8_lossy(&result.stdout)
    );
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("unresolved activation transition"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let preflight = cli(&[
        "preflight",
        "--contract",
        contract_path.to_str().expect("path"),
    ]);
    assert!(
        !preflight.status.success(),
        "current preflight must reject the same missing activation commit"
    );
    assert!(
        String::from_utf8_lossy(&preflight.stderr).contains("unresolved activation transition"),
        "{}",
        String::from_utf8_lossy(&preflight.stderr)
    );
    assert_eq!(
        fs::read(contract_path).expect("Contract unchanged"),
        before_contract
    );
    assert_eq!(
        fs::read(summary_path).expect("Summary unchanged"),
        before_summary
    );
}

fn legacy_activation_fixture() -> tempfile::TempDir {
    let directory = activation_checkpoint_fixture();
    let root = directory.path().join("repo");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    // This isolated fixture models the old on-disk format. Removing its new
    // proof does not authenticate the historical transition or any real WI.
    fs::remove_dir_all(root.join(format!(".ai/evidence/{id}.activation-transition")))
        .expect("model a legacy fixture without a forward receipt");
    let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let mut legacy_summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    legacy_summary
        .as_object_mut()
        .expect("Summary object")
        .remove("activationContinuity");
    fs::write(
        &summary_path,
        serde_json::to_vec(&legacy_summary).expect("legacy Summary JSON"),
    )
    .expect("model the legacy Summary format without a capability marker");
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
        .expect("Contract JSON");
    let runtime = candidate_runtime();
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .expect("fixture HEAD");
    let branch = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&root)
        .output()
        .expect("fixture branch");
    let input = directory.path().join("registration.json");
    fs::write(&input, serde_json::to_vec(&json!({
        "schemaVersion":1, "repositoryId":contract["repositoryId"], "workItemId":id,
        "contractDigest":cockpit_protocol::digest_json(&contract).expect("digest"),
        "worktreePath":root.canonicalize().expect("root").to_string_lossy(),
        "branch":String::from_utf8(branch.stdout).expect("branch UTF8").trim(),
        "head":String::from_utf8(head.stdout).expect("head UTF8").trim(), "generation":1,
        "runtime":{"schemaVersion":1,"runtimeVersion":runtime.runtime_version,
            "runtimeDigest":runtime.runtime_digest,"capability":"observed_environment_drift_v1"},
        "declaration":{"providedOutcomes":[],"consumedOutcomes":[],"resourceClaims":[],
            "integrationResponsibility":{"responsibleWorkItemId":id,"targetBranch":"main",
                "compositionOrder":[],"rationale":"Isolated recovery fixture; no real authority."},
            "compositionVerification":{"compatibilityConstraints":[],"requiredScenarios":[],"reusableNodes":[]}}
    })).expect("registration JSON")).expect("registration input");
    let cli = |args: &[&str]| {
        Command::new(candidate_binary())
            .args(args)
            .arg("--repo")
            .arg(&root)
            .output()
            .expect("fixture CLI")
    };
    assert_success(
        &cli(&[
            "work-item",
            "coordination",
            "register",
            "--input",
            input.to_str().expect("path"),
        ]),
        "register exact legacy fixture owner",
    );
    directory
}

fn assert_legacy_activation_recovery_flow(record_via_mcp: bool, existing_c_review: bool) {
    let directory = legacy_activation_fixture();
    let root = directory.path().join("repo");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let runtime = candidate_runtime();
    let cli = |args: &[&str]| {
        Command::new(candidate_binary())
            .args(args)
            .arg("--repo")
            .arg(&root)
            .output()
            .expect("fixture CLI")
    };
    if existing_c_review {
        // Model a previously stored self-declared C receipt in this isolated
        // legacy-format fixture. It is not a real human or historical proof.
        let mut review: Value = serde_json::from_slice(
            &fs::read(root.join(format!(".ai/decisions/{id}.preflight-review.json"))).unwrap(),
        )
        .unwrap();
        let contract: Value = serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
        review["contractDigest"] = json!(cockpit_protocol::digest_json(&contract).unwrap());
        review["preflightDecisionDigest"] =
            json!(Digest::sha256_bytes(b"synthetic old C preflight"));
        let mut summary: Value = serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
        summary["decisionEvidence"] = review.clone();
        summary["preflightDecisionDigest"] = review["preflightDecisionDigest"].clone();
        summary["preflightContractDigest"] = review["contractDigest"].clone();
        summary["preflightRepositorySnapshotDigest"] = review["repositorySnapshotDigest"].clone();
        summary["preflightState"] = json!("green");
        fs::write(&summary_path, serde_json::to_vec(&summary).unwrap()).unwrap();
        fs::write(
            root.join(format!(".ai/decisions/{id}.preflight-review.json")),
            serde_json::to_vec(&review).unwrap(),
        )
        .unwrap();
    }
    let before_contract = fs::read(&contract_path).expect("Contract bytes");
    let before_summary = fs::read(&summary_path).expect("Summary bytes");
    let planned = cli(&["work-item", "activation-recovery", "plan", "--id", id]);
    assert_success(&planned, "plan explicit present-baseline recovery");
    let request: Value = serde_json::from_slice(&planned.stdout).expect("recovery request");
    assert_eq!(request["historicalTransitionAuthentication"], "unknown");
    assert_eq!(request["businessAuthorityGranted"], false);
    assert_eq!(request["freshContractReviewRequired"], true);
    assert_eq!(request["requiredResponse"], "accept_present_baseline");
    assert_eq!(request["registrationGeneration"], 1);
    let mcp = mcp_call(
        &root,
        &runtime,
        "work_item_activation_recovery_plan",
        json!({"workItemId":id}),
    );
    assert_ne!(mcp.pointer("/result/isError"), Some(&json!(true)), "{mcp}");
    let mcp_request: Value = serde_json::from_str(
        mcp["result"]["content"][0]["text"]
            .as_str()
            .expect("MCP text"),
    )
    .expect("MCP request");
    assert_eq!(request, mcp_request);
    assert_eq!(
        fs::read(&contract_path).expect("read Contract"),
        before_contract
    );
    assert_eq!(
        fs::read(&summary_path).expect("read Summary"),
        before_summary
    );

    let relative_source = format!(".ai/evidence/{id}/present-response.json");
    let source_path = root.join(&relative_source);
    fs::create_dir_all(source_path.parent().expect("parent")).expect("evidence directory");
    let source = json!({"schemaVersion":1,"kind":"human_activation_recovery_response_source",
        "requestDigest":request["requestDigest"],"response":"accept_present_baseline",
        "humanActor":"human:isolated-test-fixture","respondedAt":"2026-10-17T00:00:00Z",
        "provenance":"Synthetic fixture response only; no real human or historical authentication."});
    let source_bytes = serde_json::to_vec(&source).expect("source JSON");
    fs::write(&source_path, &source_bytes).expect("fixture-only response source");
    let decision = json!({"schemaVersion":1,"decisionId":"activation-present-baseline-review",
        "response":"accept_present_baseline","requestDigest":request["requestDigest"],
        "humanActor":"human:isolated-test-fixture","recorderActor":"agent:test-fixture",
        "assurance":"self_declared","authoritySourcePath":relative_source,
        "authoritySourceDigest":Digest::sha256_bytes(&source_bytes),
        "respondedAt":"2026-10-17T00:00:00Z","reason":"Review only the exact present fixture baseline and retained unknowns."});
    let decision_path = directory.path().join("decision.json");
    let mut old_review = decision.clone();
    old_review["response"] = json!("confirm_review");
    fs::write(
        &decision_path,
        serde_json::to_vec(&old_review).expect("old review JSON"),
    )
    .expect("invalid decision input");
    let rejected = cli(&[
        "work-item",
        "activation-recovery",
        "record",
        "--id",
        id,
        "--input",
        decision_path.to_str().expect("path"),
    ]);
    assert!(
        !rejected.status.success(),
        "old B-style review cannot recover the gap"
    );
    assert_eq!(
        fs::read(&summary_path).expect("Summary unchanged"),
        before_summary
    );
    assert!(
        !root
            .join(format!(".ai/evidence/{id}.activation-recovery"))
            .exists()
    );
    fs::write(
        &decision_path,
        serde_json::to_vec(&decision).expect("decision JSON"),
    )
    .expect("decision input");
    let record: Value = if record_via_mcp {
        let recorded = mcp_call(
            &root,
            &runtime,
            "work_item_activation_recovery_record",
            json!({"workItemId":id,"decision":decision}),
        );
        assert_eq!(
            recorded.pointer("/result/isError"),
            Some(&json!(false)),
            "{recorded}"
        );
        serde_json::from_str(
            recorded["result"]["content"][0]["text"]
                .as_str()
                .expect("record text"),
        )
        .expect("MCP record JSON")
    } else {
        let recorded = cli(&[
            "work-item",
            "activation-recovery",
            "record",
            "--id",
            id,
            "--input",
            decision_path.to_str().expect("path"),
        ]);
        assert_success(&recorded, "record exact present fixture baseline");
        serde_json::from_slice(&recorded.stdout).expect("CLI record JSON")
    };
    assert_eq!(record["request"], request);
    assert_eq!(record["historicalTransitionAuthentication"], "unknown");
    assert_eq!(record["businessAuthorityGranted"], false);
    assert_eq!(record["freshContractReviewRequired"], true);
    let before_duplicate = recovery_metadata_bytes(&root);
    let mut duplicate = decision.clone();
    duplicate["reason"] = json!("A different decision cannot overwrite an accepted baseline");
    let duplicate = mcp_call(
        &root,
        &runtime,
        "work_item_activation_recovery_record",
        json!({"workItemId":id,"decision":duplicate}),
    );
    assert_eq!(
        duplicate.pointer("/result/isError"),
        Some(&json!(true)),
        "{duplicate}"
    );
    assert_eq!(recovery_metadata_bytes(&root), before_duplicate);
    assert_eq!(
        fs::read(&contract_path).expect("Contract remains unchanged"),
        before_contract
    );
    let status = cli(&["work-item", "status", "--id", id, "--json"]);
    assert_success(&status, "read recovered metadata status");
    let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert!(
        status["safeActions"]
            .as_array()
            .expect("actions")
            .contains(&json!("run_preflight")),
        "{status}"
    );
    assert!(
        !status["safeActions"]
            .as_array()
            .expect("actions")
            .contains(&json!("run_verification")),
        "{status}"
    );
    let preflight = cli(&[
        "preflight",
        "--contract",
        contract_path.to_str().expect("path"),
    ]);
    assert_success(&preflight, "generate fresh current Contract review request");
    let preflight: Value = serde_json::from_slice(&preflight.stdout).expect("preflight JSON");
    assert_eq!(
        preflight["reviewState"], "needs_human_confirmation",
        "{preflight}"
    );
    let fresh_summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("fresh Summary"))
            .expect("fresh Summary JSON");
    let contract: Value = serde_json::from_slice(&before_contract).expect("current Contract");
    let fresh_review = directory.path().join("post-recovery-fixture-review.json");
    fs::write(&fresh_review, serde_json::to_vec(&json!({"decisionEvidence": {
        "schemaVersion":1,"decisionId":"contract-preflight-review","decision":"confirm_review",
        "workItemId":id,"repositoryId":contract["repositoryId"],
        "contractDigest":cockpit_protocol::digest_json(&contract).expect("digest"),
        "preflightDecisionDigest":fresh_summary["preflightDecisionDigest"],
        "repositorySnapshotDigest":fresh_summary["preflightRepositorySnapshotDigest"],
        "recordedAt":record["recordedAt"],"recordedBy":"fixture:separate-post-recovery-review",
        "reason":"Separate synthetic current-Contract review after the recovery request; no real human approval."
    }})).expect("fresh review JSON")).expect("fresh review input");
    assert_success(
        &cli(&[
            "work-item",
            "controls",
            "--id",
            id,
            "--input",
            fresh_review.to_str().expect("path"),
        ]),
        "record separate current fixture review",
    );
    assert_success(
        &cli(&["verify", "--work-item", id, "--workers", "1"]),
        "only the separately reviewed fixture may verify",
    );
    let status = cli(&["work-item", "status", "--id", id, "--json"]);
    assert_success(
        &status,
        "read status after native verification projection refresh",
    );
    let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert_eq!(
        status["humanDecisionRequired"], false,
        "a native verification refresh must not create a review loop: {status}"
    );
    assert!(
        !status["blockers"]
            .as_array()
            .expect("blockers")
            .contains(&json!("activation_recovery_fresh_contract_review_required")),
        "{status}"
    );
    assert!(
        status["unknowns"]
            .as_array()
            .expect("unknowns")
            .contains(&json!("historical_transition_authentication_unknown")),
        "separate C review must not erase the missing historical authentication: {status}"
    );
    let outcome = cli(&["work-item", "outcome", "--id", id, "--json"]);
    assert_success(&outcome, "read honest post-recovery Outcome");
    let outcome: Value = serde_json::from_slice(&outcome.stdout).expect("Outcome JSON");
    assert!(
        outcome["unknowns"]
            .as_array()
            .expect("Outcome unknowns")
            .contains(&json!("historical_transition_authentication_unknown")),
        "{outcome}"
    );
    fs::remove_file(root.join(format!(
        ".ai/evidence/{id}.activation-recovery/committed.json"
    )))
    .expect("model a valid typed prepared-only recovery phase");
    let before_partial = recovery_metadata_bytes(&root);
    let partial = cli(&["work-item", "status", "--id", id, "--json"]);
    assert!(
        !partial.status.success(),
        "reviewed and verified C cannot hide a partial recovery phase"
    );
    assert!(
        String::from_utf8_lossy(&partial.stderr)
            .contains("unresolved activation recovery transaction"),
        "{}",
        String::from_utf8_lossy(&partial.stderr)
    );
    assert_eq!(recovery_metadata_bytes(&root), before_partial);
    let outcome = cli(&["work-item", "outcome", "--id", id, "--json"]);
    assert!(
        !outcome.status.success(),
        "a partial recovery cannot render an apparently complete Outcome"
    );
    assert_eq!(recovery_metadata_bytes(&root), before_partial);
}

#[test]
fn legacy_activation_requires_an_exact_present_baseline_decision_and_fresh_review() {
    assert_legacy_activation_recovery_flow(false, false);
}

#[test]
fn legacy_activation_mcp_record_requires_a_separate_current_contract_review() {
    assert_legacy_activation_recovery_flow(true, false);
}

#[test]
fn a_preexisting_c_receipt_cannot_preapprove_present_baseline_recovery() {
    assert_legacy_activation_recovery_flow(false, true);
}

fn recovery_metadata_bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(path: &Path, values: &mut Vec<(PathBuf, Vec<u8>)>) {
        if !path.exists() {
            return;
        }
        for entry in fs::read_dir(path).expect("metadata directory") {
            let entry = entry.expect("metadata entry");
            if entry.file_type().expect("entry type").is_dir() {
                walk(&entry.path(), values);
            } else {
                values.push((
                    entry.path(),
                    fs::read(entry.path()).expect("metadata bytes"),
                ));
            }
        }
    }
    let mut values = Vec::new();
    walk(&root.join(".ai"), &mut values);
    walk(&root.join(".git/.ai-cockpit"), &mut values);
    values.sort_by(|left, right| left.0.cmp(&right.0));
    values
}

#[test]
fn invalid_legacy_recovery_plans_preserve_all_metadata_bytes() {
    for case in [
        "missing_start",
        "missing_checkpoint",
        "forged_checkpoint",
        "pending_amendment",
        "partial_recovery",
        "foreign_owner",
        "foreign_runtime",
        "unsupported_runtime",
        "new_forward_marker",
        "orphan_recovery_marker",
    ] {
        let directory = legacy_activation_fixture();
        let root = directory.path().join("repo");
        let id = "WI-CLI-ACTIVATION-CONTINUITY";
        let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
        let mut summary: Value = serde_json::from_slice(&fs::read(&summary_path).expect("Summary"))
            .expect("Summary JSON");
        match case {
            "missing_start" => {
                summary["lifecycleFacts"]
                    .as_object_mut()
                    .expect("facts")
                    .remove("start");
            }
            "missing_checkpoint" => {
                summary["checkpointEvidence"] = json!([]);
            }
            "forged_checkpoint" => {
                for record in summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoint records")
                {
                    if record["stage"] == "before_edit" {
                        record["contractHash"] = json!(Digest::sha256_bytes(b"forged checkpoint"));
                    }
                    if record["stage"] == "contract_amendment_revalidation" {
                        record["originalBeforeEditContractHash"] =
                            json!(Digest::sha256_bytes(b"forged checkpoint"));
                    }
                }
            }
            "pending_amendment" => {
                fs::remove_file(root.join(format!(
                    ".ai/evidence/{id}.contract-amendments/00000002.committed.json"
                )))
                .expect("model pending fixture");
            }
            "partial_recovery" => {
                let partial = root.join(format!(".ai/evidence/{id}.activation-recovery"));
                fs::create_dir(&partial).expect("partial directory");
                fs::write(partial.join("prepared.json"), b"{}").expect("partial fixture phase");
            }
            "foreign_owner" => {
                git(&root, &["checkout", "-qb", "foreign-fixture-owner"]);
            }
            "new_forward_marker" => {
                summary["activationContinuity"] = json!({"capability":"activation_continuity_recovery_v1","journalRelevant":true});
            }
            "orphan_recovery_marker" => {
                summary["activationRecovery"] = json!({"schemaVersion":1,
                    "requestDigest":Digest::sha256_bytes(b"missing recovery record"),
                    "historicalTransitionAuthentication":"unknown","businessAuthorityGranted":false,
                    "freshContractReviewRequired":true});
            }
            _ => {}
        }
        fs::write(
            &summary_path,
            serde_json::to_vec(&summary).expect("fixture Summary JSON"),
        )
        .expect("fixture mutation");
        let before = recovery_metadata_bytes(&root);
        let mut runtime = candidate_runtime();
        if case == "foreign_runtime" {
            runtime.runtime_digest = Digest::sha256_bytes(b"foreign fixture Runtime");
        }
        if case == "unsupported_runtime" {
            runtime.runtime_version = "0.2.105".into();
        }
        let response = mcp_call(
            &root,
            &runtime,
            "work_item_activation_recovery_plan",
            json!({"workItemId":id}),
        );
        if response.pointer("/result/isError") != Some(&json!(true)) {
            let preserved = directory.keep();
            panic!(
                "{case}: unexpected request; fixture preserved at {}; {response}",
                preserved.display()
            );
        }
        assert_eq!(
            recovery_metadata_bytes(&root),
            before,
            "{case} must not repair or publish metadata implicitly"
        );
    }
}

#[test]
fn recovery_request_binds_native_environment_and_raw_review_bytes() {
    let directory = legacy_activation_fixture();
    let root = directory.path().join("repo");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    let planned = run_cli(&[
        "work-item",
        "activation-recovery",
        "plan",
        "--id",
        id,
        "--repo",
        root.to_str().expect("root"),
    ]);
    assert_success(&planned, "plan bound metadata request");
    let request: Value = serde_json::from_slice(&planned.stdout).expect("request");
    let observed = cockpit_repository::RepositoryExecutionContext::capture(&root)
        .expect("context")
        .observe_environment_identity()
        .expect("actual environment");
    assert_eq!(
        request["environmentDigest"],
        json!(observed.environment_digest)
    );
    let review = fs::read(root.join(format!(".ai/decisions/{id}.preflight-review.json")))
        .expect("historical review bytes");
    assert_eq!(
        request["rawEvidenceDigests"]["preflight-review"],
        json!(Digest::sha256_bytes(&review))
    );
}

fn fixture_present_recovery_decision(root: &Path, request: &Value) -> Value {
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    let relative = format!(".ai/evidence/{id}/actual-fixture-response.json");
    let path = root.join(&relative);
    fs::create_dir_all(path.parent().expect("response parent")).expect("response directory");
    let source = json!({"schemaVersion":1,"kind":"human_activation_recovery_response_source",
        "requestDigest":request["requestDigest"],"response":"accept_present_baseline",
        "humanActor":"human:isolated-test-fixture","respondedAt":"2026-10-17T00:00:00Z",
        "provenance":"Synthetic response in an isolated test only; no real human authentication."});
    let bytes = serde_json::to_vec(&source).expect("response JSON");
    fs::write(path, &bytes).expect("fixture response source");
    json!({"schemaVersion":1,"decisionId":"activation-present-baseline-review",
        "response":"accept_present_baseline","requestDigest":request["requestDigest"],
        "humanActor":"human:isolated-test-fixture","recorderActor":"agent:test-fixture",
        "assurance":"self_declared","authoritySourcePath":relative,
        "authoritySourceDigest":Digest::sha256_bytes(&bytes),"respondedAt":"2026-10-17T00:00:00Z",
        "reason":"Accept only the exact isolated present baseline, retaining all historical unknowns."})
}

#[test]
fn stale_or_substituted_recovery_decisions_never_publish_or_finish_metadata() {
    for case in [
        "request",
        "summary",
        "contract",
        "history",
        "head",
        "registration",
        "environment",
        "source",
        "agent_as_human",
        "blank_human",
        "old_review_source",
        "partial_recovery",
        "pending_amendment",
        "unknown_field",
    ] {
        let directory = legacy_activation_fixture();
        let root = directory.path().join("repo");
        let id = "WI-CLI-ACTIVATION-CONTINUITY";
        let planned = run_cli(&[
            "work-item",
            "activation-recovery",
            "plan",
            "--id",
            id,
            "--repo",
            root.to_str().expect("root"),
        ]);
        assert_success(&planned, "plan current recovery fixture");
        let request: Value = serde_json::from_slice(&planned.stdout).expect("request");
        let mut decision = fixture_present_recovery_decision(&root, &request);
        let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
        let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
        let source_path = root.join(
            decision["authoritySourcePath"]
                .as_str()
                .expect("source path"),
        );
        match case {
            "request" => {
                decision["requestDigest"] = json!(Digest::sha256_bytes(b"another request"));
            }
            "summary" => {
                let mut value: Value =
                    serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
                value["fixtureNote"] = json!("changed after request");
                fs::write(&summary_path, serde_json::to_vec(&value).unwrap()).unwrap();
            }
            "contract" => {
                let mut value: Value =
                    serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
                value["goal"] = json!("a different current Contract");
                fs::write(&contract_path, serde_json::to_vec(&value).unwrap()).unwrap();
            }
            "history" => {
                let path = root.join(format!(
                    ".ai/evidence/{id}.contract-amendments/00000001.prepared.json"
                ));
                let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                value["prepared"]["request"]["reason"] = json!("changed raw history after request");
                fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
            }
            "head" => {
                git(
                    &root,
                    &[
                        "commit",
                        "--allow-empty",
                        "-qm",
                        "advance isolated fixture HEAD",
                    ],
                );
            }
            "registration" => {
                let input = directory.path().join("registration.json");
                let mut value: Value = serde_json::from_slice(&fs::read(&input).unwrap()).unwrap();
                value["generation"] = json!(2);
                fs::write(&input, serde_json::to_vec(&value).unwrap()).unwrap();
                let registered = run_cli(&[
                    "work-item",
                    "coordination",
                    "register",
                    "--repo",
                    root.to_str().unwrap(),
                    "--input",
                    input.to_str().unwrap(),
                ]);
                assert_success(&registered, "advance actual native fixture registration");
            }
            "source" => {
                fs::write(&source_path, b"{}").unwrap();
            }
            "agent_as_human" | "blank_human" => {
                let mut value: Value =
                    serde_json::from_slice(&fs::read(&source_path).unwrap()).unwrap();
                value["humanActor"] = if case == "blank_human" {
                    json!("human:   ")
                } else {
                    json!("agent:cannot-supply-human-decision")
                };
                let bytes = serde_json::to_vec(&value).unwrap();
                fs::write(&source_path, &bytes).unwrap();
                decision["humanActor"] = value["humanActor"].clone();
                decision["authoritySourceDigest"] = json!(Digest::sha256_bytes(&bytes));
            }
            "old_review_source" => {
                let bytes =
                    fs::read(root.join(format!(".ai/decisions/{id}.preflight-review.json")))
                        .unwrap();
                fs::write(&source_path, &bytes).unwrap();
                decision["authoritySourceDigest"] = json!(Digest::sha256_bytes(&bytes));
            }
            "partial_recovery" => {
                let partial = root.join(format!(".ai/evidence/{id}.activation-recovery"));
                fs::create_dir(&partial).unwrap();
                fs::write(partial.join("prepared.json"), b"{}").unwrap();
            }
            "pending_amendment" => {
                fs::remove_file(root.join(format!(
                    ".ai/evidence/{id}.contract-amendments/00000002.committed.json"
                )))
                .unwrap();
            }
            "unknown_field" => {
                decision["unsupported"] = json!(true);
            }
            _ => {}
        }
        let input = directory.path().join("recovery-decision.json");
        fs::write(&input, serde_json::to_vec(&decision).unwrap()).unwrap();
        let before = recovery_metadata_bytes(&root);
        if case == "unknown_field" {
            let response = mcp_call(
                &root,
                &candidate_runtime(),
                "work_item_activation_recovery_record",
                json!({"workItemId":id,"decision":decision}),
            );
            assert_eq!(
                response.pointer("/result/isError"),
                Some(&json!(true)),
                "{case}: {response}"
            );
        } else {
            let mut command = Command::new(candidate_binary());
            command.args([
                "work-item",
                "activation-recovery",
                "record",
                "--repo",
                root.to_str().unwrap(),
                "--id",
                id,
                "--input",
                input.to_str().unwrap(),
            ]);
            if case == "environment" {
                command.env("CARGO_BUILD_JOBS", "2");
            } // Metadata-only rejection: no build is launched.
            let output = command.output().expect("record fixture decision");
            if output.status.success() {
                let preserved = directory.keep();
                panic!(
                    "{case}: unexpected recovery publication; fixture preserved at {}; response bytes={}, raw digest={}",
                    preserved.display(),
                    output.stdout.len(),
                    Digest::sha256_bytes(&output.stdout)
                );
            }
        }

        assert_eq!(
            recovery_metadata_bytes(&root),
            before,
            "{case}: refusal must preserve every existing phase and byte"
        );
    }
}
