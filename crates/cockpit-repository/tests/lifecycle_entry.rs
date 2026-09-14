use cockpit_core::{DecisionState, Digest};
use cockpit_git::GitRepository;
use cockpit_protocol::{HumanDecision, RuntimeContext};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    amend_work_item_contract, archive_work_item, attach, checkpoint_work_item,
    close_work_item_with_structured_decision, finish_work_item, preflight_work_item,
    preflight_work_item_with_runtime, record_verification, record_verification_with_runtime,
    record_work_item_governance_controls, require_verification_preconditions,
    run_repository_verification, scaffold_work_item, start_work_item_with_options, status,
};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(args)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn output(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(args)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("git output")
}

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("repository");
    run(directory.path(), &["init", "-q"]);
    attach(directory.path()).expect("attach");
    directory
}

fn start_options() -> WorkItemStartOptions {
    WorkItemStartOptions {
        authority: "authorized".into(),
        acceptance_criteria: vec!["entry remains bounded".into()],
        ..Default::default()
    }
}

fn enable_tri_language_projection_convention(root: &Path) {
    fs::create_dir_all(root.join("docs/work-items")).expect("work-item docs");
    fs::create_dir_all(root.join("docs/reference")).expect("reference docs");
    for suffix in ["", ".zh-CN", ".ja"] {
        fs::write(
            root.join(format!("docs/reference/reference-parity{suffix}.md")),
            "# Reference parity\n",
        )
        .expect("parity ledger");
    }
}

fn write_prearchive_projection(root: &Path, work_item_id: &str) {
    for suffix in ["", ".zh-CN", ".ja"] {
        fs::write(
            root.join(format!(
                "docs/work-items/{work_item_id}{suffix}.md"
            )),
            format!(
                "---\nstatus: in_progress\nworkItemId: {work_item_id}\nlastVerifiedBy: {work_item_id}\n---\n\n# {work_item_id}\n"
            ),
        )
        .expect("prearchive page");
    }
    for suffix in ["", ".zh-CN", ".ja"] {
        let path = root.join(format!("docs/reference/reference-parity{suffix}.md"));
        let mut contents = fs::read_to_string(&path).expect("parity ledger");
        contents.push_str(&format!(
            "| {work_item_id} | In progress | [Work Item](../work-items/{work_item_id}.md) | planned terminal lifecycle: archive `.ai/work-items/archive/{work_item_id}.contract.json`; verification `.ai/evidence/{work_item_id}.verification.json`; close `.ai/decisions/{work_item_id}.close.json` |\n"
        ));
        fs::write(path, contents).expect("prearchive parity row");
    }
}

fn write_unclosed_archive(root: &Path, id: &str) -> (PathBuf, Vec<u8>) {
    let archive = root.join(".ai/work-items/archive");
    fs::create_dir_all(&archive).expect("archive directory");
    let path = archive.join(format!("{id}.archive.json"));
    let bytes =
        br#"{"schemaVersion":1,"workItemId":"WI-OLD","state":"archived","closeRequired":true}
"#
        .to_vec();
    fs::write(&path, &bytes).expect("archive marker");
    (path, bytes)
}

#[test]
fn new_and_start_reject_archived_item_without_close() {
    let directory = repository();
    let (archive_path, archive_bytes) = write_unclosed_archive(directory.path(), "WI-OLD");
    let readiness = status(directory.path()).expect("status").readiness;
    assert!(!readiness.ready_on_base);
    assert_eq!(readiness.state, "blocked");
    assert_eq!(readiness.unclosed_archived_work_items, vec!["WI-OLD"]);

    let scaffold = scaffold_work_item(directory.path(), "WI-NEW", "code")
        .expect_err("new scaffold must stop behind an unclosed archive");
    assert!(scaffold.to_string().contains("archived Work Items"));
    assert!(
        !directory
            .path()
            .join(".ai/work-items/active/WI-NEW.contract.json")
            .exists()
    );

    let start = start_work_item_with_options(
        directory.path(),
        "WI-START",
        "entry gate",
        "stop before unsafe start",
        &["src/**".into()],
        &start_options(),
    )
    .expect_err("start must stop behind an unclosed archive");
    assert!(start.to_string().contains("archived Work Items"));
    assert!(
        !directory
            .path()
            .join(".ai/work-items/active/WI-START.contract.json")
            .exists()
    );
    assert_eq!(
        fs::read(archive_path).expect("archive bytes"),
        archive_bytes
    );
}

#[test]
fn start_rejects_user_changes_that_precede_the_contract() {
    let directory = repository();
    fs::create_dir_all(directory.path().join("src")).expect("src");
    fs::write(directory.path().join("src/main.rs"), "fn main() {}\n").expect("user change");

    let error = start_work_item_with_options(
        directory.path(),
        "WI-DIRTY-START",
        "entry gate",
        "stop before dirty start",
        &["src/**".into()],
        &start_options(),
    )
    .expect_err("dirty pre-start repository must fail closed");
    assert!(error.to_string().contains("before start"));
    assert!(error.to_string().contains("src/main.rs"));
    assert!(
        !directory
            .path()
            .join(".ai/work-items/active/WI-DIRTY-START.contract.json")
            .exists()
    );
}

#[test]
fn scenario_coverage_can_be_declared_before_the_first_checkpoint() {
    let directory = repository();
    start_work_item_with_options(
        directory.path(),
        "WI-SCENARIO-DECLARATION",
        "declare high-risk scenario coverage",
        "make the preflight boundary explicit",
        &["src/**".into()],
        &WorkItemStartOptions {
            risk: "high".into(),
            ..start_options()
        },
    )
    .expect("start");

    amend_work_item_contract(
        directory.path(),
        "WI-SCENARIO-DECLARATION",
        &json!({
            "scenarioCoverageAppend": [{
                "scenario": "preflight",
                "required": true,
                "status": "unverified",
                "evidence": [],
                "expected": "preflight stops before expensive verification",
                "verificationPlan": "run the preflight regression"
            }]
        }),
        "declare the required high-risk scenario before checkpoint",
    )
    .expect("scenario declaration");

    let contract: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-SCENARIO-DECLARATION.contract.json"),
        )
        .expect("contract"),
    )
    .expect("contract JSON");
    assert_eq!(contract["scenarioCoverage"][0]["scenario"], "preflight");
}

#[test]
fn verification_preconditions_reject_missing_governance_controls_before_execution() {
    let directory = repository();
    start_work_item_with_options(
        directory.path(),
        "WI-VERIFY-PRECONDITIONS",
        "check cheap verification gates first",
        "reject missing governance controls before the project command",
        &["src/**".into()],
        &WorkItemStartOptions {
            acceptance_criteria: vec!["A: bounded review remains explicit".into()],
            ..start_options()
        },
    )
    .expect("start");
    let contract = directory
        .path()
        .join(".ai/work-items/active/WI-VERIFY-PRECONDITIONS.contract.json");
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), "WI-VERIFY-PRECONDITIONS").expect("checkpoint");
    let snapshot = GitRepository::discover(directory.path())
        .expect("git repository")
        .snapshot()
        .expect("snapshot");
    let runtime = RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    };
    let error = require_verification_preconditions(
        directory.path(),
        "WI-VERIFY-PRECONDITIONS",
        &runtime,
        &snapshot,
    )
    .expect_err("missing governance controls must stop before execution");
    assert!(
        error
            .to_string()
            .contains("verification preconditions are blocked")
    );
    assert!(error.to_string().contains("acceptance_evidence_missing"));
}

#[test]
fn verification_preconditions_accept_complete_repository_bound_custom_evidence() {
    let directory = repository();
    let work_item_id = "WI-CUSTOM-EVIDENCE-PRECONDITION";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "accept a complete custom evidence projection before verification",
        "do not block a valid repository-bound custom evidence class at the execution boundary",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            risk: "high".into(),
            required_evidence_classes: vec!["performance".into()],
            ..start_options()
        },
    )
    .expect("start");

    amend_work_item_contract(
        directory.path(),
        work_item_id,
        &json!({
            "scenarioCoverageAppend": [{
                "scenario": "valid repository-bound custom evidence permits verification preconditions",
                "required": true,
                "status": "unverified",
                "evidence": [],
                "expected": "a complete custom evidence projection is accepted before project verification",
                "verificationPlan": "run the focused precondition regression"
            }]
        }),
        "declare the high-risk custom evidence precondition scenario before checkpoint",
    )
    .expect("declare scenario coverage");

    let contract_path = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    let evidence_path = directory.path().join("performance-measurement.txt");
    fs::write(&evidence_path, b"p50=1ms\np95=2ms\n").expect("evidence file");
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract")).expect("contract");
    let contract_digest = cockpit_protocol::digest_json(&contract).expect("contract digest");
    let evidence_digest = Digest::sha256_bytes(&fs::read(&evidence_path).expect("evidence"));
    record_work_item_governance_controls(
        directory.path(),
        work_item_id,
        &json!({
            "scenarioCoverage": [{
                "scenario": "valid repository-bound custom evidence permits verification preconditions",
                "required": true,
                "status": "unverified",
                "evidence": [],
                "expected": "a complete custom evidence projection is accepted before project verification",
                "verificationPlan": "run the focused precondition regression"
            }],
            "intentAlignment": {
                "state": "resolved",
                "evidence": ["crates/cockpit-repository/src/lib.rs"]
            },
            "evidenceClasses": {
                "schemaVersion": 1,
                "contractDigest": contract_digest,
                "items": [{
                    "class": "performance",
                    "evidence": [{
                        "type": "measurement",
                        "path": "performance-measurement.txt",
                        "locator": "p50,p95",
                        "verification": "passed",
                        "digest": evidence_digest
                    }]
                }]
            }
        }),
    )
    .expect("record complete custom evidence projection");

    let preflight = preflight_work_item(directory.path(), &contract_path).expect("preflight");
    assert_ne!(
        preflight.state,
        DecisionState::Red,
        "custom evidence setup preflight: {preflight:#?}"
    );
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    let snapshot = GitRepository::discover(directory.path())
        .expect("git repository")
        .snapshot()
        .expect("snapshot");
    let runtime = RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    };

    require_verification_preconditions(directory.path(), work_item_id, &runtime, &snapshot)
        .expect("complete custom evidence must not block project verification");

    fs::remove_file(&evidence_path).expect("remove custom evidence");
    let preflight = preflight_work_item(directory.path(), &contract_path).expect("re-preflight");
    assert_ne!(
        preflight.state,
        DecisionState::Red,
        "missing custom evidence should remain a precondition diagnostic: {preflight:#?}"
    );
    let changed_snapshot = GitRepository::discover(directory.path())
        .expect("git repository")
        .snapshot()
        .expect("changed snapshot");
    let error = require_verification_preconditions(
        directory.path(),
        work_item_id,
        &runtime,
        &changed_snapshot,
    )
    .expect_err("missing custom evidence must block before project verification");
    assert!(error.to_string().contains("evidence_classes_missing"));
}

#[test]
fn preflight_rejects_missing_own_projection_before_verification() {
    let directory = repository();
    let work_item_id = "WI-PROJECTION-MISSING";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "require a reader projection before expensive verification",
        "reject missing projection at the cheap preflight boundary",
        &["docs/**".into()],
        &start_options(),
    )
    .expect("start");
    enable_tri_language_projection_convention(directory.path());

    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    let decision = preflight_work_item(directory.path(), &contract).expect("preflight decision");
    assert_eq!(decision.state, DecisionState::Red);
    assert!(decision.blockers.iter().any(|blocker| {
        blocker.contains("documentation_projection_missing")
            && blocker.contains("docs/work-items/WI-PROJECTION-MISSING.md")
    }));
}

#[test]
fn preflight_accepts_a_complete_prearchive_projection() {
    let directory = repository();
    let work_item_id = "WI-PROJECTION-VALID";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "require a valid reader projection before expensive verification",
        "accept a complete prearchive projection at the cheap preflight boundary",
        &["docs/**".into()],
        &start_options(),
    )
    .expect("start");
    enable_tri_language_projection_convention(directory.path());
    write_prearchive_projection(directory.path(), work_item_id);

    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    let decision = preflight_work_item(directory.path(), &contract).expect("preflight decision");
    assert!(
        !decision
            .blockers
            .iter()
            .any(|blocker| blocker.starts_with("documentation_projection_"))
    );
}

#[test]
fn preflight_rejects_a_malformed_projection_with_its_exact_path() {
    let directory = repository();
    let work_item_id = "WI-PROJECTION-MALFORMED";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "reject malformed reader projection before verification",
        "report the exact malformed projection path",
        &["docs/**".into()],
        &start_options(),
    )
    .expect("start");
    enable_tri_language_projection_convention(directory.path());
    write_prearchive_projection(directory.path(), work_item_id);
    let malformed = directory
        .path()
        .join(format!("docs/work-items/{work_item_id}.zh-CN.md"));
    fs::remove_file(&malformed).expect("remove projection page");
    fs::create_dir(&malformed).expect("malformed projection directory");

    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    let decision = preflight_work_item(directory.path(), &contract).expect("preflight decision");
    assert_eq!(decision.state, DecisionState::Red);
    assert!(decision.blockers.iter().any(|blocker| {
        blocker.contains("documentation_projection_invalid")
            && blocker.contains(&format!("docs/work-items/{work_item_id}.zh-CN.md"))
            && blocker.contains("regular non-symlink file")
    }));
}

#[test]
fn verification_preconditions_reject_projection_before_project_execution() {
    let directory = repository();
    let work_item_id = "WI-PROJECTION-VERIFY";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "reject a projection that becomes invalid before verification",
        "keep the project process behind the cheap projection gate",
        &["docs/**".into()],
        &start_options(),
    )
    .expect("start");
    enable_tri_language_projection_convention(directory.path());
    write_prearchive_projection(directory.path(), work_item_id);
    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    fs::remove_file(
        directory
            .path()
            .join(format!("docs/work-items/{work_item_id}.ja.md")),
    )
    .expect("remove projection page");
    let snapshot = GitRepository::discover(directory.path())
        .expect("git repository")
        .snapshot()
        .expect("snapshot");
    let runtime = RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    };
    let error =
        require_verification_preconditions(directory.path(), work_item_id, &runtime, &snapshot)
            .expect_err("invalid projection must stop before the project process");
    assert!(
        error
            .to_string()
            .contains("verification preconditions are blocked")
    );
    assert!(
        error
            .to_string()
            .contains(&format!("docs/work-items/{work_item_id}.ja.md"))
    );
}

#[test]
fn close_rejects_missing_projection_before_writing_a_close_decision() {
    let directory = repository();
    let work_item_id = "WI-PROJECTION-CLOSE";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "reject close without the own reader projection",
        "prevent a partial terminal state after projection loss",
        &["docs/**".into()],
        &start_options(),
    )
    .expect("start");
    enable_tri_language_projection_convention(directory.path());
    write_prearchive_projection(directory.path(), work_item_id);
    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    record_verification(
        directory.path(),
        work_item_id,
        &json!({"passed": true, "nodesPlanned": 1}),
        "test-runtime",
        &Digest::sha256_bytes(b"test-runtime"),
    )
    .expect("verification");
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");
    fs::remove_file(
        directory
            .path()
            .join(format!("docs/work-items/{work_item_id}.md")),
    )
    .expect("remove projection page");

    let error = close_work_item_with_structured_decision(
        directory.path(),
        work_item_id,
        &HumanDecision {
            decision: "approved".into(),
            actor: "test-human".into(),
            authority_source: "test".into(),
            reason: "projection close regression".into(),
            evidence_refs: Vec::new(),
            policy_refs: Vec::new(),
            decided_at: "2026-09-14T00:00:00Z".into(),
            resume_condition: None,
        },
    )
    .expect_err("close must stop before writing terminal state");
    assert!(
        error
            .to_string()
            .contains("close documentation projection preconditions are blocked")
    );
    assert!(
        error
            .to_string()
            .contains(&format!("docs/work-items/{work_item_id}.md"))
    );
    assert!(
        !directory
            .path()
            .join(format!(".ai/decisions/{work_item_id}.close.json"))
            .exists()
    );
}

#[test]
fn object_without_projection_convention_keeps_generic_preflight_behavior() {
    let directory = repository();
    let work_item_id = "WI-PROJECTION-GENERIC";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "keep an object repository generic",
        "do not require an undeclared documentation convention",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start");
    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    let decision = preflight_work_item(directory.path(), &contract).expect("preflight decision");
    assert!(
        !decision
            .blockers
            .iter()
            .any(|blocker| blocker.starts_with("documentation_projection_"))
    );
}

#[test]
fn empty_amendment_invalidation_does_not_block_fresh_verification_preconditions() {
    let directory = repository();
    let work_item_id = "WI-VERIFY-EMPTY-INVALIDATION";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "allow a no-gate amendment to recover",
        "let fresh verification clear an empty invalidation marker",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: Vec::new(),
            ..Default::default()
        },
    )
    .expect("start");
    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    let runtime = RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    };
    preflight_work_item_with_runtime(directory.path(), &contract, &runtime)
        .expect("initial preflight");
    let summary = directory
        .path()
        .join(format!(".ai/work-items/active/{work_item_id}.summary.json"));
    let mut summary_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary).expect("summary")).expect("summary JSON");
    summary_value["intentAlignment"] = json!({
        "state": "resolved",
        "evidence": ["test-intent"]
    });
    fs::write(
        &summary,
        serde_json::to_vec_pretty(&summary_value).expect("summary JSON"),
    )
    .expect("intent alignment");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "typed-receipt-regression".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["src/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("typed verification");
    let mut receipt = serde_json::to_value(&run.receipt).expect("receipt JSON");
    receipt["runtimeVersion"] = runtime.runtime_version.clone().into();
    receipt["runtimeDigest"] = runtime.runtime_digest.to_string().into();
    record_verification_with_runtime(
        directory.path(),
        work_item_id,
        &receipt,
        &runtime,
        &run.final_snapshot,
    )
    .expect("initial verification");

    amend_work_item_contract(
        directory.path(),
        work_item_id,
        &json!({"scopeAppend": ["docs/**"]}),
        "add an authorized scope without any required verification gates",
    )
    .expect("amend Contract");
    preflight_work_item_with_runtime(directory.path(), &contract, &runtime)
        .expect("amended preflight");
    let snapshot = GitRepository::discover(directory.path())
        .expect("git repository")
        .snapshot()
        .expect("amended snapshot");

    require_verification_preconditions(directory.path(), work_item_id, &runtime, &snapshot)
        .expect("empty invalidation marker must allow fresh verification to run");
}

#[test]
fn start_rejects_clean_branch_ahead_of_discoverable_default_base() {
    let directory = repository();
    fs::write(directory.path().join("README.md"), "base\n").expect("base file");
    run(directory.path(), &["add", "-A"]);
    run(
        directory.path(),
        &[
            "-c",
            "user.email=test@example.com",
            "-c",
            "user.name=test",
            "commit",
            "-qm",
            "base",
        ],
    );
    let base = output(directory.path(), &["rev-parse", "HEAD"])
        .trim()
        .to_owned();
    run(directory.path(), &["branch", "-M", "main"]);
    run(
        directory.path(),
        &[
            "remote",
            "add",
            "origin",
            "https://example.invalid/origin.git",
        ],
    );
    run(
        directory.path(),
        &["update-ref", "refs/remotes/origin/main", &base],
    );
    run(
        directory.path(),
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    let readiness = status(directory.path()).expect("status").readiness;
    assert!(readiness.ready_on_base);
    assert_eq!(readiness.state, "ready_on_base");
    assert_eq!(readiness.default_branch.as_deref(), Some("main"));
    fs::write(directory.path().join("README.md"), "ahead\n").expect("ahead change");
    run(directory.path(), &["add", "README.md"]);
    run(
        directory.path(),
        &[
            "-c",
            "user.email=test@example.com",
            "-c",
            "user.name=test",
            "commit",
            "-qm",
            "ahead",
        ],
    );

    let error = start_work_item_with_options(
        directory.path(),
        "WI-AHEAD-START",
        "entry gate",
        "stop ahead branch",
        &["README.md".into()],
        &start_options(),
    )
    .expect_err("branch ahead of default must fail closed");
    assert!(error.to_string().contains("base"));
    assert!(error.to_string().contains("origin/main"));
}

#[test]
fn recovery_scaffold_may_activate_on_its_existing_ahead_branch() {
    let directory = repository();
    fs::write(directory.path().join("README.md"), "base\n").expect("base file");
    run(directory.path(), &["add", "-A"]);
    run(
        directory.path(),
        &[
            "-c",
            "user.email=test@example.com",
            "-c",
            "user.name=test",
            "commit",
            "-qm",
            "base",
        ],
    );
    run(directory.path(), &["branch", "-M", "main"]);
    let base = output(directory.path(), &["rev-parse", "HEAD"])
        .trim()
        .to_owned();
    run(
        directory.path(),
        &[
            "remote",
            "add",
            "origin",
            "https://example.invalid/origin.git",
        ],
    );
    run(
        directory.path(),
        &["update-ref", "refs/remotes/origin/main", &base],
    );
    run(
        directory.path(),
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    run(directory.path(), &["checkout", "-qb", "recovery"]);
    scaffold_work_item(directory.path(), "WI-RECOVERY", "implementation")
        .expect("recovery scaffold");
    let contract_path = directory
        .path()
        .join(".ai/work-items/active/WI-RECOVERY.contract.json");
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract")).expect("json");
    contract["predecessorWorkItemId"] = serde_json::json!("WI-PREDECESSOR");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).expect("serialize contract"),
    )
    .expect("write recovery binding");
    run(directory.path(), &["add", "-A"]);
    run(
        directory.path(),
        &[
            "-c",
            "user.email=test@example.com",
            "-c",
            "user.name=test",
            "commit",
            "-qm",
            "reserve recovery continuation",
        ],
    );

    start_work_item_with_options(
        directory.path(),
        "WI-RECOVERY",
        "continue the recovery",
        "activate a bounded recovery continuation",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["recovery remains explicitly bounded".into()],
            ..start_options()
        },
    )
    .expect("recovery continuation should bypass only the ordinary base check");
}

#[test]
fn status_reports_unknown_readiness_without_remote_metadata() {
    let directory = repository();
    let readiness = status(directory.path()).expect("status").readiness;
    assert!(!readiness.ready_on_base);
    assert_eq!(readiness.state, "unknown");
    assert!(
        readiness
            .unknowns
            .iter()
            .any(|value| value == "default_base_unknown")
    );
}
