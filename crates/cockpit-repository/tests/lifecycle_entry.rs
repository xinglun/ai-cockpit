use cockpit_core::{DecisionState, Digest};
use cockpit_git::GitRepository;
use cockpit_protocol::{HumanDecision, RuntimeContext};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    amend_work_item_contract, archive_work_item, attach, checkpoint_work_item,
    close_work_item_with_structured_decision, finish_work_item, finish_work_item_with_runtime,
    load_reusable_verification_attempt, persist_verification_attempt, preflight_work_item,
    preflight_work_item_with_runtime, record_verification, record_verification_with_runtime,
    record_work_item_governance_controls, repository_id, require_verification_preconditions,
    run_repository_verification, scaffold_work_item, set_work_item_intelligence,
    start_work_item_with_options, status, work_item_start_advisory,
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

fn write_derived_documentation_policy(root: &Path) {
    fs::create_dir_all(root.join(".ai/project")).expect("project policy directory");
    fs::write(
        root.join(".ai/project/documentation-policy.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 2,
            "repositoryId": repository_id(root).to_string(),
            "defaultProjection": "derived",
            "requiredModes": ["docs", "documentation"],
            "requiredOperations": ["documentation.modify", "release.publish"],
            "preserveExistingRegistrations": true,
            "effectiveFromContractCreatedAt": "2020-01-01T00:00:00Z"
        }))
        .expect("documentation policy JSON"),
    )
    .expect("documentation policy");
}

fn commit_fixture_baseline(root: &Path) {
    run(root, &["add", "-A"]);
    run(
        root,
        &[
            "-c",
            "user.name=AI Cockpit Test",
            "-c",
            "user.email=ai-cockpit-test@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "fixture baseline",
        ],
    );
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

fn write_unclosed_archive(root: &Path, id: &str, scope: &[&str]) -> (PathBuf, Vec<u8>) {
    let archive = root.join(".ai/work-items/archive");
    fs::create_dir_all(&archive).expect("archive directory");
    let contract_path = archive.join(format!("{id}.contract.json"));
    let contract_bytes = serde_json::to_vec_pretty(&json!({
        "schemaVersion": 2,
        "protocolVersion": 1,
        "repositoryId": repository_id(root).to_string(),
        "workItemId": id,
        "scope": scope,
    }))
    .expect("archive contract JSON");
    fs::write(&contract_path, &contract_bytes).expect("archive contract");
    let path = archive.join(format!("{id}.archive.json"));
    let bytes = serde_json::to_vec_pretty(&json!({
        "schemaVersion": 1,
        "workItemId": id,
        "state": "archived",
        "closeRequired": true,
        "files": {
            "contractDigest": Digest::sha256_bytes(&contract_bytes).to_string(),
        },
    }))
    .expect("archive manifest JSON");
    fs::write(&path, &bytes).expect("archive marker");
    (path, bytes)
}

#[test]
fn unrelated_archived_pending_close_remains_visible_without_blocking_entry() {
    let directory = repository();
    let (archive_path, archive_bytes) =
        write_unclosed_archive(directory.path(), "WI-OLD", &["docs/**"]);
    let readiness = status(directory.path()).expect("status").readiness;
    assert_eq!(readiness.unclosed_archived_work_items, vec!["WI-OLD"]);
    assert!(
        readiness
            .blockers
            .iter()
            .all(|blocker| blocker != "archived_work_items_pending_close")
    );
    assert!(
        readiness
            .historical_debt
            .iter()
            .any(|item| item.work_item_id == "WI-OLD")
    );

    scaffold_work_item(directory.path(), "WI-NEW", "code")
        .expect("unrelated historical debt must not block a new scaffold");

    start_work_item_with_options(
        directory.path(),
        "WI-START",
        "entry gate",
        "start with a scope disjoint from the archived item",
        &["src/**".into()],
        &start_options(),
    )
    .expect("unrelated historical debt must not block a disjoint start");
    assert!(
        directory
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
fn start_advisory_reports_residual_work_without_forcing_a_stop() {
    let directory = repository();
    start_work_item_with_options(
        directory.path(),
        "WI-OLD",
        "old work",
        "leave a residual active Work Item for the next start advisory",
        &["src/**".into()],
        &start_options(),
    )
    .expect("fixture Work Item starts");

    let advisory = work_item_start_advisory(directory.path(), "WI-NEW")
        .expect("start advisory should be read-only");
    assert_eq!(advisory.work_item_id, "WI-NEW");
    assert_eq!(advisory.classification, "advisory");
    assert!(
        advisory
            .active_work_items
            .iter()
            .any(|item| item.work_item_id == "WI-OLD")
    );
    assert!(
        advisory
            .warnings
            .iter()
            .any(|warning| warning == "active_work_items_require_lifecycle_or_cleanup:1")
    );
    assert!(
        advisory
            .next_actions
            .iter()
            .any(|action| action == "review_start_cleanup_advisory_and_continue_if_unrelated")
    );
    assert!(advisory.conflicts.is_empty());
}

#[test]
fn scaffold_includes_the_start_advisory_for_the_next_agent() {
    let directory = repository();
    start_work_item_with_options(
        directory.path(),
        "WI-OLD",
        "old work",
        "leave a residual active Work Item for the scaffold advisory",
        &["src/**".into()],
        &start_options(),
    )
    .expect("fixture Work Item starts");

    let receipt = scaffold_work_item(directory.path(), "WI-NEW", "code")
        .expect("unrelated residual work must remain advisory");
    let advisory = receipt
        .start_advisory
        .expect("work-item new must expose the start advisory");
    assert_eq!(advisory.classification, "advisory");
    assert!(
        advisory
            .active_work_items
            .iter()
            .any(|item| item.work_item_id == "WI-OLD")
    );
    assert!(
        advisory
            .next_actions
            .iter()
            .any(|action| action == "review_start_cleanup_advisory_and_continue_if_unrelated")
    );
    assert!(
        directory
            .path()
            .join(".ai/work-items/active/WI-OLD.contract.json")
            .exists()
    );
}

#[test]
fn start_advisory_treats_absent_lifecycle_directories_as_empty() {
    let directory = tempfile::tempdir().expect("repository");
    run(directory.path(), &["init", "-q"]);

    let advisory = work_item_start_advisory(directory.path(), "WI-NEW")
        .expect("an empty repository has no residual Work Items to report");
    assert_eq!(advisory.classification, "clear");
    assert!(advisory.active_work_items.is_empty());
    assert!(advisory.pending_cleanup.is_empty());
    assert!(advisory.warnings.is_empty());
    assert!(advisory.conflicts.is_empty());
}

#[test]
fn start_advisory_reports_non_default_remote_tracking_branches() {
    let directory = repository();
    let remote = tempfile::tempdir().expect("bare remote");
    run(remote.path(), &["init", "--bare", "-q"]);
    run(remote.path(), &["symbolic-ref", "HEAD", "refs/heads/trunk"]);
    run(
        directory.path(),
        &["config", "user.email", "test@example.com"],
    );
    run(directory.path(), &["config", "user.name", "Test"]);
    run(directory.path(), &["branch", "-M", "trunk"]);
    run(directory.path(), &["commit", "--allow-empty", "-m", "base"]);
    run(
        directory.path(),
        &[
            "remote",
            "add",
            "origin",
            remote.path().to_str().expect("remote path"),
        ],
    );
    run(directory.path(), &["push", "-u", "origin", "trunk:trunk"]);
    run(directory.path(), &["remote", "set-head", "origin", "trunk"]);
    run(
        directory.path(),
        &["update-ref", "refs/remotes/origin/codex/stale", "HEAD"],
    );

    let advisory = work_item_start_advisory(directory.path(), "WI-NEW")
        .expect("remote-tracking branch observation should be available");
    assert_eq!(advisory.remote_branches.len(), 1);
    assert_eq!(advisory.remote_branches[0].name, "origin/codex/stale");
    assert_eq!(advisory.remote_branches[0].head.len(), 40);
    assert!(
        advisory
            .warnings
            .iter()
            .any(|warning| warning == "non_default_remote_branches_present:1")
    );
}

#[test]
fn start_blocks_only_an_exact_active_worktree_binding() {
    let directory = repository();
    let existing_id = "WI-EXACT-RESOURCE";
    start_work_item_with_options(
        directory.path(),
        existing_id,
        "bind a resource for the conflict fixture",
        "prove an exact worktree binding blocks a different Work Item",
        &["src/**".into()],
        &start_options(),
    )
    .expect("fixture Work Item starts");
    let contract_path = directory
        .path()
        .join(format!(".ai/work-items/active/{existing_id}.contract.json"));
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
            .expect("contract JSON");
    contract["resourceContext"] = json!({
        "worktree": directory.path().to_string_lossy(),
        "branch": "fixture-owner-branch"
    });
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).expect("contract bytes"),
    )
    .expect("bind exact worktree");

    let error = start_work_item_with_options(
        directory.path(),
        "WI-NEW-EXACT-CONFLICT",
        "reject exact resource collision",
        "do not share an active Work Item worktree",
        &["src/**".into()],
        &start_options(),
    )
    .expect_err("exact active worktree binding must block start");
    assert!(
        error
            .to_string()
            .contains("current_worktree_bound_to_active_work_item:WI-EXACT-RESOURCE")
    );
    assert!(
        !directory
            .path()
            .join(".ai/work-items/active/WI-NEW-EXACT-CONFLICT.contract.json")
            .exists()
    );
}

#[test]
fn preflight_blocks_a_declared_dependency_on_a_disjoint_unclosed_archived_item() {
    let unrelated = repository();
    write_unclosed_archive(unrelated.path(), "WI-OLD", &["docs/**"]);
    start_work_item_with_options(
        unrelated.path(),
        "WI-UNRELATED",
        "entry gate",
        "keep unrelated historical debt visible without blocking preflight",
        &["src/**".into()],
        &start_options(),
    )
    .expect("a disjoint archived item without a declared dependency must not block start");
    let unrelated_contract = unrelated
        .path()
        .join(".ai/work-items/active/WI-UNRELATED.contract.json");
    let unrelated_decision =
        preflight_work_item(unrelated.path(), &unrelated_contract).expect("unrelated preflight");
    assert!(
        unrelated_decision
            .blockers
            .iter()
            .all(|blocker| blocker != "archived_work_item_dependency_pending_close:WI-OLD"),
        "unrelated pending close must not become a dependency blocker: {unrelated_decision:#?}"
    );

    let dependent = repository();
    write_unclosed_archive(dependent.path(), "WI-OLD", &["docs/**"]);
    start_work_item_with_options(
        dependent.path(),
        "WI-DEPENDENT",
        "entry gate",
        "block a declared dependency on unresolved historical work",
        &["src/**".into()],
        &start_options(),
    )
    .expect("disjoint scopes remain startable before a dependency is declared");
    set_work_item_intelligence(
        dependent.path(),
        "WI-DEPENDENT",
        vec!["WI-OLD".into()],
        Vec::new(),
        false,
    )
    .expect("write the supported identity-bound dependency sidecar");
    let dependent_contract = dependent
        .path()
        .join(".ai/work-items/active/WI-DEPENDENT.contract.json");
    let dependent_decision =
        preflight_work_item(dependent.path(), &dependent_contract).expect("dependent preflight");
    assert_eq!(
        dependent_decision.state,
        DecisionState::Red,
        "{dependent_decision:#?}"
    );
    assert!(
        dependent_decision
            .blockers
            .iter()
            .any(|blocker| blocker == "archived_work_item_dependency_pending_close:WI-OLD"),
        "declared dependency on the exact unclosed archive must block: {dependent_decision:#?}"
    );
}

#[test]
fn preflight_rejects_malformed_or_foreign_work_item_intelligence_identity() {
    let mut accepted_invalid_sidecars = Vec::new();
    for invalid_sidecar in [
        "malformed JSON",
        "foreign repository identity",
        "foreign Work Item identity",
    ] {
        let directory = repository();
        write_unclosed_archive(directory.path(), "WI-OLD", &["docs/**"]);
        start_work_item_with_options(
            directory.path(),
            "WI-DEPENDENT",
            "entry gate",
            "reject an invalid dependency identity instead of bypassing it",
            &["src/**".into()],
            &start_options(),
        )
        .expect("disjoint dependency remains startable before preflight");
        set_work_item_intelligence(
            directory.path(),
            "WI-DEPENDENT",
            vec!["WI-OLD".into()],
            Vec::new(),
            false,
        )
        .expect("write the supported dependency sidecar");
        let intelligence_path = directory
            .path()
            .join(".ai/work-items/active/WI-DEPENDENT.intelligence.json");
        match invalid_sidecar {
            "malformed JSON" => fs::write(&intelligence_path, b"{").expect("malformed sidecar"),
            "foreign repository identity" | "foreign Work Item identity" => {
                let mut intelligence: serde_json::Value =
                    serde_json::from_slice(&fs::read(&intelligence_path).expect("sidecar bytes"))
                        .expect("sidecar JSON");
                let field = if invalid_sidecar == "foreign repository identity" {
                    "repositoryId"
                } else {
                    "workItemId"
                };
                intelligence[field] = "foreign-identity".into();
                fs::write(
                    &intelligence_path,
                    serde_json::to_vec_pretty(&intelligence).expect("foreign sidecar JSON"),
                )
                .expect("foreign sidecar");
            }
            _ => unreachable!("all invalid sidecar cases are enumerated"),
        }
        let contract_path = directory
            .path()
            .join(".ai/work-items/active/WI-DEPENDENT.contract.json");
        if preflight_work_item(directory.path(), &contract_path).is_ok() {
            accepted_invalid_sidecars.push(invalid_sidecar);
        }
    }
    assert!(
        accepted_invalid_sidecars.is_empty(),
        "preflight must fail closed rather than trust invalid sidecar identity: {accepted_invalid_sidecars:?}"
    );
}

#[test]
fn start_blocks_scope_overlap_with_an_archived_pending_item() {
    let directory = repository();
    write_unclosed_archive(directory.path(), "WI-OLD", &["src/**"]);

    let error = start_work_item_with_options(
        directory.path(),
        "WI-START",
        "entry gate",
        "do not overlap unresolved archived work",
        &["src/main.rs".into()],
        &start_options(),
    )
    .expect_err("a declared overlapping scope remains blocked");

    assert!(
        error
            .to_string()
            .contains("archived Work Item scope conflict")
    );
    assert!(error.to_string().contains("WI-OLD"));
    assert!(
        !directory
            .path()
            .join(".ai/work-items/active/WI-START.contract.json")
            .exists()
    );
}

#[test]
fn archived_scope_requires_valid_identity_and_skips_only_valid_foreign_repositories() {
    for case in ["missing", "malformed", "foreign"] {
        let directory = repository();
        let (manifest_path, _) = write_unclosed_archive(directory.path(), "WI-OLD", &["src/**"]);
        let contract_path = directory
            .path()
            .join(".ai/work-items/archive/WI-OLD.contract.json");
        let mut contract: serde_json::Value =
            serde_json::from_slice(&fs::read(&contract_path).expect("archived Contract"))
                .expect("archived Contract JSON");
        if case == "missing" {
            contract
                .as_object_mut()
                .expect("Contract object")
                .remove("repositoryId");
        } else if case == "malformed" {
            contract["repositoryId"] = json!("not-a-digest");
        } else {
            contract["repositoryId"] =
                json!(Digest::sha256_bytes(b"different repository").to_string());
        }
        let contract_bytes =
            serde_json::to_vec_pretty(&contract).expect("updated archived Contract JSON");
        fs::write(&contract_path, &contract_bytes).expect("updated archived Contract");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).expect("archive manifest"))
                .expect("archive manifest JSON");
        manifest["files"]["contractDigest"] =
            json!(Digest::sha256_bytes(&contract_bytes).to_string());
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("updated archive manifest JSON"),
        )
        .expect("updated archive manifest");

        let result = start_work_item_with_options(
            directory.path(),
            "WI-CANDIDATE",
            "reject an untrusted archived repository identity",
            "do not let malformed archive identity bypass scope checks",
            &["src/main.rs".into()],
            &start_options(),
        );
        if case == "foreign" {
            result.expect("a valid, explicitly foreign archive is not authority here");
            continue;
        }
        let error = result.expect_err("untrusted archived identity must fail closed");
        assert!(
            error
                .to_string()
                .contains("archived_work_item_scope_untrusted:WI-OLD"),
            "case {case} returned an unexpected error: {error}"
        );
    }
}

#[test]
fn missing_or_malformed_archive_manifest_remains_untrusted_for_scope_checks() {
    for case in ["missing", "malformed"] {
        let directory = repository();
        let (manifest_path, _) = write_unclosed_archive(directory.path(), "WI-OLD", &["src/**"]);
        if case == "missing" {
            fs::remove_file(&manifest_path).expect("remove archive manifest");
        } else {
            fs::write(&manifest_path, b"{not-json").expect("corrupt archive manifest");
        }

        let readiness = status(directory.path())
            .expect("status with untrusted archive")
            .readiness;
        assert!(
            readiness
                .unclosed_archived_work_items
                .contains(&"WI-OLD".into()),
            "case {case} must keep the incomplete archive visible: {readiness:#?}"
        );
        let result = start_work_item_with_options(
            directory.path(),
            "WI-CANDIDATE",
            "reject an untrusted archive manifest",
            "do not let a missing or malformed manifest bypass scope checks",
            &["src/main.rs".into()],
            &start_options(),
        );
        let error = result.expect_err("untrusted archive manifest must fail closed");
        assert!(
            error
                .to_string()
                .contains("archived_work_item_scope_untrusted:WI-OLD"),
            "case {case} returned an unexpected error: {error}"
        );
    }
}

#[test]
fn preflight_rechecks_archived_scope_conflicts_after_contract_amendment() {
    let directory = repository();
    write_unclosed_archive(directory.path(), "WI-OLD", &["src/**"]);

    let work_item_id = "WI-SCOPE-AMENDMENT";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "recheck scope at preflight",
        "an additive scope amendment must not bypass archived pending-close conflicts",
        &["tests/**".into()],
        &start_options(),
    )
    .expect("a disjoint initial scope may start");
    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    assert_ne!(
        preflight_work_item(directory.path(), &contract)
            .expect("initial preflight")
            .state,
        DecisionState::Red
    );
    checkpoint_work_item(directory.path(), work_item_id)
        .expect("checkpoint the initially disjoint Contract before amendment");

    amend_work_item_contract(
        directory.path(),
        work_item_id,
        &json!({"scopeAppend": ["src/**"]}),
        "include the newly discovered source area in the existing Contract",
    )
    .expect("append the in-scope source path");

    let amended = preflight_work_item(directory.path(), &contract).expect("amended preflight");
    assert_eq!(amended.state, DecisionState::Red, "{amended:#?}");
    assert!(
        amended
            .blockers
            .iter()
            .any(|blocker| { blocker == "archived_work_item_scope_conflict:WI-OLD" })
    );
}

#[test]
fn preflight_blocks_when_archived_scope_no_longer_matches_its_manifest() {
    let directory = repository();
    let (archive_path, archive_bytes) =
        write_unclosed_archive(directory.path(), "WI-OLD", &["docs/**"]);
    start_work_item_with_options(
        directory.path(),
        "WI-SCOPE-TAMPER",
        "scope integrity",
        "do not trust a changed archived scope",
        &["src/**".into()],
        &start_options(),
    )
    .expect("valid disjoint archived scope must allow start");
    let contract_path = directory
        .path()
        .join(".ai/work-items/active/WI-SCOPE-TAMPER.contract.json");
    assert_ne!(
        preflight_work_item(directory.path(), &contract_path)
            .expect("initial preflight")
            .state,
        DecisionState::Red
    );

    let archived_contract_path = directory
        .path()
        .join(".ai/work-items/archive/WI-OLD.contract.json");
    let mut archived_contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&archived_contract_path).expect("archived Contract"))
            .expect("archived Contract JSON");
    archived_contract["scope"] = json!(["src/**"]);
    fs::write(
        &archived_contract_path,
        serde_json::to_vec_pretty(&archived_contract).expect("serialize tampered Contract"),
    )
    .expect("tamper archived Contract scope");

    let decision = preflight_work_item(directory.path(), &contract_path)
        .expect("untrusted archive should produce a blocking decision");
    assert_eq!(decision.state, DecisionState::Red, "{decision:#?}");
    assert!(
        decision
            .blockers
            .iter()
            .any(|blocker| { blocker == "archived_work_item_scope_untrusted:WI-OLD" })
    );
    assert_eq!(
        fs::read(archive_path).expect("archive manifest"),
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
fn scope_can_be_appended_before_the_first_checkpoint() {
    let directory = repository();
    let work_item_id = "WI-PRECHECKPOINT-SCOPE";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "add a required projection path before checkpoint",
        "avoid a preflight deadlock without changing immutable Contract facts",
        &["crates/cockpit-repository/src/**".into()],
        &start_options(),
    )
    .expect("start");

    let amendment = amend_work_item_contract(
        directory.path(),
        work_item_id,
        &json!({"scopeAppend": ["docs/work-items/WI-PRECHECKPOINT-SCOPE.md"]}),
        "add the missing required projection path before the first checkpoint",
    )
    .expect("pre-checkpoint scope amendment");

    assert_eq!(amendment["stage"], "pre_checkpoint_contract_declaration");
    let contract: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.path().join(format!(
            ".ai/work-items/active/{work_item_id}.contract.json"
        )))
        .expect("contract"),
    )
    .expect("contract JSON");
    assert_eq!(
        contract["scope"],
        json!([
            "crates/cockpit-repository/src/**",
            "docs/work-items/WI-PRECHECKPOINT-SCOPE.md"
        ])
    );
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/work-items/active/{work_item_id}.summary.json")),
        )
        .expect("summary"),
    )
    .expect("summary JSON");
    assert_eq!(summary["checkpointCount"], 0);
    assert!(summary.get("checkpointEvidence").is_none());
}

#[test]
fn contract_amendment_appends_sources_and_verification_declarations() {
    let directory = repository();
    let work_item_id = "WI-CONTRACT-DECLARATION-AMENDMENT";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "declare Contract sources and verification",
        "allow a governed Work Item to complete missing descriptive Contract declarations",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start");

    let amendment = amend_work_item_contract(
        directory.path(),
        work_item_id,
        &json!({
            "sourcesAppend": [{
                "path": "docs/reference/work-item-lifecycle-closure.md",
                "reason": "The repository workflow defines the lifecycle acceptance boundary."
            }],
            "verificationAppend": [{
                "check": "cargo test --locked -p cockpit-repository --test lifecycle_entry",
                "required": true
            }]
        }),
        "record the implementation source and the focused lifecycle verification command",
    )
    .expect("append Contract declarations");
    assert_eq!(amendment["stage"], "pre_checkpoint_contract_declaration");

    let contract_path = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(contract_path).expect("contract bytes"))
            .expect("contract JSON");
    assert_eq!(
        contract["sources"][0],
        json!({
            "path": "docs/reference/work-item-lifecycle-closure.md",
            "reason": "The repository workflow defines the lifecycle acceptance boundary."
        })
    );
    assert_eq!(
        contract["verification"][0],
        json!({
            "check": "cargo test --locked -p cockpit-repository --test lifecycle_entry",
            "required": true
        })
    );
}

#[test]
fn contract_amendment_rejects_malformed_declarations_without_writing() {
    let directory = repository();
    let work_item_id = "WI-CONTRACT-DECLARATION-ATOMICITY";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "reject malformed Contract declarations",
        "keep rejected amendments from changing governance state",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start");
    let active = directory.path().join(".ai/work-items/active");
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    let summary_path = active.join(format!("{work_item_id}.summary.json"));
    let events_path = active.join(format!("{work_item_id}.events.jsonl"));
    let original_contract = fs::read(&contract_path).expect("contract bytes");
    let original_summary = fs::read(&summary_path).expect("summary bytes");
    let original_events = fs::read(&events_path).ok();

    let invalid_source = amend_work_item_contract(
        directory.path(),
        work_item_id,
        &json!({
            "sourcesAppend": [{"path": " ", "reason": "not a source"}]
        }),
        "reject an empty source path",
    )
    .expect_err("empty source path must be rejected");
    assert!(
        invalid_source
            .to_string()
            .contains("sourcesAppend entries must not be empty")
    );
    assert_eq!(
        fs::read(&contract_path).expect("contract bytes"),
        original_contract
    );
    assert_eq!(
        fs::read(&summary_path).expect("summary bytes"),
        original_summary
    );
    assert_eq!(fs::read(&events_path).ok(), original_events);

    let invalid_verification = amend_work_item_contract(
        directory.path(),
        work_item_id,
        &json!({
            "verificationAppend": [{"check": "cargo test", "unknown": true}]
        }),
        "reject an unsupported verification declaration field",
    )
    .expect_err("unknown verification fields must be rejected");
    assert!(
        invalid_verification
            .to_string()
            .contains("verificationAppend contains an invalid declaration")
    );
    assert_eq!(
        fs::read(&contract_path).expect("contract bytes"),
        original_contract
    );
    assert_eq!(
        fs::read(&summary_path).expect("summary bytes"),
        original_summary
    );
    assert_eq!(fs::read(&events_path).ok(), original_events);
}

#[test]
fn contract_amendment_rejects_empty_reason_without_writing() {
    let directory = repository();
    let work_item_id = "WI-CONTRACT-AMENDMENT-EMPTY-REASON-ATOMICITY";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "reject an amendment with an empty reason",
        "keep a lifecycle-precondition rejection from changing governance state",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start");
    let active = directory.path().join(".ai/work-items/active");
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    let summary_path = active.join(format!("{work_item_id}.summary.json"));
    let events_path = active.join(format!("{work_item_id}.events.jsonl"));
    preflight_work_item(directory.path(), &contract_path).expect("record preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    let original_contract = fs::read(&contract_path).expect("contract bytes");
    let original_summary = fs::read(&summary_path).expect("summary bytes");
    let original_events = fs::read(&events_path).ok();

    let error = amend_work_item_contract(
        directory.path(),
        work_item_id,
        &json!({"scopeAppend": ["tests/**"]}),
        "",
    )
    .expect_err("an amendment reason is required");
    assert!(
        error
            .to_string()
            .contains("contract amendment reason must not be empty")
    );
    assert_eq!(
        fs::read(&contract_path).expect("contract bytes"),
        original_contract
    );
    assert_eq!(
        fs::read(&summary_path).expect("summary bytes"),
        original_summary
    );
    assert_eq!(fs::read(&events_path).ok(), original_events);
}

#[test]
fn post_checkpoint_contract_declarations_invalidate_preflight_for_revalidation() {
    let directory = repository();
    let work_item_id = "WI-CONTRACT-DECLARATION-REVALIDATION";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "amend Contract declarations after checkpoint",
        "revalidate repository governance after additive declaration changes",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start");
    let contract_path = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    preflight_work_item(directory.path(), &contract_path).expect("initial preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");

    let amendment = amend_work_item_contract(
        directory.path(),
        work_item_id,
        &json!({
            "sourcesAppend": ["the repository's current Work Item workflow"],
            "verificationAppend": ["cargo test --locked -p cockpit-repository --test lifecycle_entry"]
        }),
        "add the missing source and verification declarations after checkpoint",
    )
    .expect("revalidate additive Contract declarations");
    assert_eq!(amendment["stage"], "contract_amendment_revalidation");

    let summary_path = directory
        .path()
        .join(format!(".ai/work-items/active/{work_item_id}.summary.json"));
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(summary_path).expect("summary bytes"))
            .expect("summary JSON");
    assert_eq!(summary["checkpointCount"], 1);
    assert_eq!(summary["preflightState"], "not_run");
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
fn first_typed_required_verification_is_allowed_before_summary_has_passed_entries() {
    let directory = repository();
    let work_item_id = "WI-FIRST-TYPED-VERIFICATION";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "run a declared required check for the first time",
        "allow the first Runtime verification without hand-editing Summary",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: Vec::new(),
            ..Default::default()
        },
    )
    .expect("start");
    amend_work_item_contract(
        directory.path(),
        work_item_id,
        &json!({
            "verificationAppend": [{"check": "first-required-check", "required": true}]
        }),
        "declare the first required verification check before preflight",
    )
    .expect("declare required check");

    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    let runtime = RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    };
    record_work_item_governance_controls(
        directory.path(),
        work_item_id,
        &json!({"intentAlignment":{"state":"resolved","evidence":["test-intent"]}}),
    )
    .expect("record intent alignment");
    let preflight = preflight_work_item_with_runtime(directory.path(), &contract, &runtime)
        .expect("yellow preflight may await the first required check");
    assert!(matches!(
        preflight.state,
        DecisionState::Green | DecisionState::Yellow
    ));
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");

    let snapshot = GitRepository::discover(directory.path())
        .expect("git repository")
        .snapshot()
        .expect("snapshot");
    require_verification_preconditions(directory.path(), work_item_id, &runtime, &snapshot)
        .expect("first required verification must pass cheap preconditions");

    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "first-required-check".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["src/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("first required verification");
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
    .expect("record first required verification");

    let summary_path = directory
        .path()
        .join(format!(".ai/work-items/active/{work_item_id}.summary.json"));
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(summary_path).expect("summary bytes"))
            .expect("summary JSON");
    assert_eq!(
        summary["verification"][0],
        json!({"check": "first-required-check", "result": "passed"})
    );
    finish_work_item_with_runtime(directory.path(), work_item_id, &runtime)
        .expect("finish remains available after the formal receipt");
}

#[test]
fn typed_verification_survives_its_governance_projection_at_preflight_and_finish() {
    let directory = repository();
    let work_item_id = "WI-VERIFICATION-GOVERNANCE-SNAPSHOT";
    fs::write(
        directory.path().join("src.rs"),
        "pub fn value() -> u8 { 1 }\n",
    )
    .expect("source");
    commit_fixture_baseline(directory.path());
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "keep typed verification current across Runtime governance writes",
        "a receipt recorded by Runtime stays current when no source path changed",
        &["src.rs".into()],
        &start_options(),
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
    preflight_work_item_with_runtime(directory.path(), &contract, &runtime).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "governance-projection".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["src.rs".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
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
    .expect("record verification");

    assert_eq!(
        preflight_work_item_with_runtime(directory.path(), &contract, &runtime)
            .expect("post-verification preflight")
            .state,
        DecisionState::Green
    );
    finish_work_item_with_runtime(directory.path(), work_item_id, &runtime)
        .expect("governance projection must not stale verification evidence");
}

#[test]
fn source_mutation_after_typed_verification_stales_the_receipt_and_blocks_finish() {
    let directory = repository();
    let work_item_id = "WI-VERIFICATION-SOURCE-MUTATION";
    fs::write(
        directory.path().join("src.rs"),
        "pub fn value() -> u8 { 1 }\n",
    )
    .expect("source");
    commit_fixture_baseline(directory.path());
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "reject stale typed verification after source mutation",
        "source changes remain part of the verification identity",
        &["src.rs".into()],
        &start_options(),
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
    preflight_work_item_with_runtime(directory.path(), &contract, &runtime).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "source-mutation".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["src.rs".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
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
    .expect("record verification");

    fs::write(
        directory.path().join("src.rs"),
        "pub fn value() -> u8 { 2 }\n",
    )
    .expect("source mutation");
    let preflight = preflight_work_item_with_runtime(directory.path(), &contract, &runtime)
        .expect("stale evidence is a yellow preflight result");
    assert_eq!(preflight.state, DecisionState::Yellow);
    assert!(
        preflight
            .unknowns
            .iter()
            .any(|unknown| unknown == "evidence_stale")
    );
    let error = finish_work_item_with_runtime(directory.path(), work_item_id, &runtime)
        .expect_err("source mutation must block finish");
    assert!(error.to_string().contains("current repository snapshot"));
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
        &[
            "crates/cockpit-repository/src/**".into(),
            "performance-measurement.txt".into(),
        ],
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
fn ordinary_no_resource_work_item_completes_without_derived_document_projection() {
    let directory = repository();
    write_derived_documentation_policy(directory.path());
    enable_tri_language_projection_convention(directory.path());
    commit_fixture_baseline(directory.path());

    let work_item_id = "WI-ORDINARY-NO-DOC-PROJECTION";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "complete ordinary code work without a derived documentation successor",
        "prove that documentation pages are derived unless the Contract selects a documentation route",
        &["src/**/*.rs".into()],
        &start_options(),
    )
    .expect("ordinary code start");

    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    let preflight = preflight_work_item(directory.path(), &contract).expect("preflight");
    assert_ne!(preflight.state, DecisionState::Red, "{preflight:#?}");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");

    let current_runtime = RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    };
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "ordinary-code-check".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["src/**/*.rs".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current_runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("verification run");
    let evidence = serde_json::to_value(&run.receipt).expect("verification receipt");
    record_verification_with_runtime(
        directory.path(),
        work_item_id,
        &evidence,
        &current_runtime,
        &run.final_snapshot,
    )
    .expect("verification evidence");
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");
    close_work_item_with_structured_decision(
        directory.path(),
        work_item_id,
        &HumanDecision {
            decision: "approved".into(),
            actor: "test-human".into(),
            authority_source: "test-authorization".into(),
            reason: "the scoped ordinary code work is verified".into(),
            evidence_refs: vec![format!(".ai/evidence/{work_item_id}.verification.json")],
            policy_refs: vec!["documentation-policy-derived-default".into()],
            decided_at: "2026-09-15T00:00:00Z".into(),
            resume_condition: None,
        },
    )
    .expect("close without a documentation successor");

    for suffix in ["", ".zh-CN", ".ja"] {
        assert!(
            !directory
                .path()
                .join(format!("docs/work-items/{work_item_id}{suffix}.md"))
                .exists(),
            "derived page {suffix} must not be required for ordinary code work"
        );
    }
    assert_eq!(
        fs::read_dir(directory.path().join(".ai/work-items/active"))
            .expect("active Work Items")
            .filter_map(Result::ok)
            .filter(|entry| entry
                .file_name()
                .to_string_lossy()
                .ends_with(".contract.json"))
            .count(),
        0,
        "ordinary completion must not create a documentation successor"
    );
}

#[test]
fn explicit_documentation_mode_requires_projection_without_repository_shape() {
    let directory = repository();
    write_derived_documentation_policy(directory.path());
    commit_fixture_baseline(directory.path());

    let work_item_id = "WI-DOCS-MODE-PROJECTION";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "require documentation projection by declared mode",
        "keep the explicit documentation route governed without relying on repository layout",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start");
    let contract_path = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract bytes"))
            .expect("contract JSON");
    contract["mode"] = "docs".into();
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).expect("updated contract JSON"),
    )
    .expect("declared documentation route");

    let decision = preflight_work_item(directory.path(), &contract_path).expect("preflight");
    assert_eq!(decision.state, DecisionState::Red);
    assert!(decision.blockers.iter().any(|blocker| {
        blocker.contains("documentation_projection_missing")
            && blocker.contains(&format!("docs/work-items/{work_item_id}.md"))
    }));
}

#[test]
fn release_publish_operation_requires_projection_without_repository_shape() {
    let directory = repository();
    write_derived_documentation_policy(directory.path());
    commit_fixture_baseline(directory.path());

    let work_item_id = "WI-RELEASE-OPERATION-PROJECTION";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "require release projection by declared operation",
        "keep the release route governed without relying on repository layout",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start");
    let contract_path = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract bytes"))
            .expect("contract JSON");
    contract["operation"] = "release.publish".into();
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).expect("updated contract JSON"),
    )
    .expect("declared release operation");

    let decision = preflight_work_item(directory.path(), &contract_path).expect("preflight");
    assert_eq!(decision.state, DecisionState::Red);
    assert!(decision.blockers.iter().any(|blocker| {
        blocker.contains("documentation_projection_missing")
            && blocker.contains(&format!("docs/work-items/{work_item_id}.md"))
    }));
}

#[test]
fn requested_release_operation_requires_projection_and_conflicts_fail_closed() {
    for (work_item_id, operation, requested_operation, expected_blocker) in [
        (
            "WI-REQUESTED-RELEASE-OPERATION",
            None,
            Some("release.publish"),
            "documentation_projection_missing",
        ),
        (
            "WI-CONFLICTING-RELEASE-OPERATIONS",
            Some("code"),
            Some("release.publish"),
            "operation fields conflict",
        ),
    ] {
        let directory = repository();
        write_derived_documentation_policy(directory.path());
        commit_fixture_baseline(directory.path());
        start_work_item_with_options(
            directory.path(),
            work_item_id,
            "enforce all declared operation aliases",
            "do not let an operation spelling bypass its required projection",
            &["src/**".into()],
            &start_options(),
        )
        .expect("start");

        let contract_path = directory.path().join(format!(
            ".ai/work-items/active/{work_item_id}.contract.json"
        ));
        let mut contract: serde_json::Value =
            serde_json::from_slice(&fs::read(&contract_path).expect("contract bytes"))
                .expect("contract JSON");
        if let Some(operation) = operation {
            contract["operation"] = operation.into();
        }
        contract["requestedOperation"] = requested_operation.into();
        fs::write(
            &contract_path,
            serde_json::to_vec_pretty(&contract).expect("updated contract JSON"),
        )
        .expect("declared operation aliases");

        let result = preflight_work_item(directory.path(), &contract_path);
        if expected_blocker == "operation fields conflict" {
            assert!(
                result
                    .expect_err("conflicting operation declarations must fail closed")
                    .to_string()
                    .contains(expected_blocker)
            );
        } else {
            let decision = result.expect("preflight");
            assert_eq!(decision.state, DecisionState::Red, "{decision:#?}");
            assert!(
                decision
                    .blockers
                    .iter()
                    .any(|blocker| blocker.contains(expected_blocker)),
                "expected {expected_blocker:?} in {:#?}",
                decision.blockers
            );
        }
    }
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
            work_item_id: None,
            timeout_seconds: None,
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
        &json!({"scopeAppend": ["tests/**"]}),
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
fn verification_record_rolls_back_projection_when_outcome_refresh_fails() {
    let directory = repository();
    let work_item_id = "WI-VERIFY-ATOMIC-PROJECTION";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "keep verification projection atomic",
        "do not leave completion evidence behind when a later Outcome refresh rejects",
        &["src/**".into()],
        &start_options(),
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
    preflight_work_item_with_runtime(directory.path(), &contract, &runtime).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");

    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "atomic-projection-initial".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["src/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("initial verification");
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
    .expect("initial evidence");
    finish_work_item_with_runtime(directory.path(), work_item_id, &runtime).expect("finish");

    let evidence_path = directory
        .path()
        .join(format!(".ai/evidence/{work_item_id}.verification.json"));
    let summary_path = directory
        .path()
        .join(format!(".ai/work-items/active/{work_item_id}.summary.json"));
    let outcome_path = directory
        .path()
        .join(format!(".ai/work-items/active/{work_item_id}.outcome.json"));
    let report_path = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.task-report.json"
    ));
    let markdown_path = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.task-report.md"
    ));
    let evidence_before = fs::read(&evidence_path).expect("existing evidence");
    let summary_before = fs::read(&summary_path).expect("existing summary");
    let outcome_before = fs::read(&outcome_path).expect("existing outcome");
    let report_before = fs::read(&report_path).expect("existing task report");
    let markdown_before = fs::read(&markdown_path).expect("existing task report markdown");

    // Simulate a malformed active projection discovered after the evidence
    // write.  The transaction must restore all bytes, including the fixture's
    // pre-existing malformed Outcome, when its refresh rejects the attempt.
    let mut invalid_outcome: serde_json::Value =
        serde_json::from_slice(&outcome_before).expect("outcome JSON");
    invalid_outcome["state"] = json!("invalid-during-projection-refresh");
    fs::write(
        &outcome_path,
        serde_json::to_vec_pretty(&invalid_outcome).expect("invalid outcome JSON"),
    )
    .expect("malformed outcome fixture");
    let malformed_outcome = fs::read(&outcome_path).expect("malformed outcome");

    let retry = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "atomic-projection-retry".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["src/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("retry verification");
    let mut retry_receipt = serde_json::to_value(&retry.receipt).expect("retry receipt JSON");
    retry_receipt["runtimeVersion"] = runtime.runtime_version.clone().into();
    retry_receipt["runtimeDigest"] = runtime.runtime_digest.to_string().into();
    let error = record_verification_with_runtime(
        directory.path(),
        work_item_id,
        &retry_receipt,
        &runtime,
        &retry.final_snapshot,
    )
    .expect_err("malformed Outcome must reject the projection refresh");
    assert!(error.to_string().contains("verified Outcome"));
    assert_eq!(
        fs::read(&evidence_path).expect("rolled back evidence"),
        evidence_before
    );
    assert_eq!(
        fs::read(&summary_path).expect("rolled back summary"),
        summary_before
    );
    assert_eq!(
        fs::read(&outcome_path).expect("preserved malformed outcome"),
        malformed_outcome
    );
    assert_eq!(
        fs::read(&report_path).expect("rolled back task report"),
        report_before
    );
    assert_eq!(
        fs::read(&markdown_path).expect("rolled back task report markdown"),
        markdown_before
    );
}

#[test]
fn tampered_verification_attempt_is_not_reused_even_when_execution_fields_match() {
    let directory = repository();
    let work_item_id = "WI-VERIFY-ATTEMPT-INTEGRITY";
    start_work_item_with_options(
        directory.path(),
        work_item_id,
        "bind reusable attempts",
        "reject modified attempt content before reuse",
        &["src/**".into()],
        &start_options(),
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
    preflight_work_item_with_runtime(directory.path(), &contract, &runtime).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    let request = RepositoryVerificationRequest {
        node_id: "attempt-integrity".into(),
        program: "true".into(),
        args: Vec::new(),
        scope: vec!["src/**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: runtime.runtime_digest.to_string(),
        base_commit: None,
        workers: 1,
        work_item_id: None,
        timeout_seconds: None,
        policy: RepositoryVerificationPolicy::NeverReuse,
    };
    let run = run_repository_verification(directory.path(), &request).expect("verification");
    let stored = persist_verification_attempt(
        directory.path(),
        work_item_id,
        std::slice::from_ref(&request),
        &run.final_snapshot,
        &runtime,
        "execution_completed",
        None,
        Some(&serde_json::to_value(&run.receipt).expect("receipt JSON")),
    )
    .expect("persist attempt");
    let attempt_path = directory
        .path()
        .join(stored["path"].as_str().expect("attempt path"));
    let mut attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("attempt bytes"))
            .expect("attempt JSON");
    attempt["receipt"]["executionRecords"][0]["stdout"] = json!("tampered");
    fs::write(
        &attempt_path,
        serde_json::to_vec_pretty(&attempt).expect("tampered attempt JSON"),
    )
    .expect("tampered attempt");

    assert!(
        load_reusable_verification_attempt(
            directory.path(),
            work_item_id,
            std::slice::from_ref(&request),
            &run.final_snapshot,
            &runtime,
        )
        .expect("load attempt")
        .is_none()
    );
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
    write_unclosed_archive(directory.path(), "WI-OLD", &["docs/**"]);
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
fn recovery_continuation_does_not_bypass_untrusted_archived_scope() {
    let directory = repository();
    let (manifest_path, _) = write_unclosed_archive(directory.path(), "WI-OLD", &["src/**"]);
    let contract_path = directory
        .path()
        .join(".ai/work-items/archive/WI-OLD.contract.json");
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
    let mut predecessor: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("predecessor Contract"))
            .expect("predecessor Contract JSON");
    predecessor["scope"] = serde_json::json!(["other/**"]);
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&predecessor).expect("tampered predecessor Contract"),
    )
    .expect("tamper predecessor Contract");
    // Keep the manifest bytes unchanged so the Runtime must classify the
    // historical scope as untrusted instead of allowing recovery to bypass it.
    assert!(fs::read(&manifest_path).is_ok());
    let recovery_contract = directory
        .path()
        .join(".ai/work-items/active/WI-RECOVERY.contract.json");
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&recovery_contract).expect("recovery contract"))
            .expect("recovery contract JSON");
    contract["predecessorWorkItemId"] = serde_json::json!("WI-OLD");
    fs::write(
        &recovery_contract,
        serde_json::to_vec_pretty(&contract).expect("serialize recovery contract"),
    )
    .expect("bind recovery predecessor");
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
    let error = start_work_item_with_options(
        directory.path(),
        "WI-RECOVERY",
        "continue the recovery",
        "do not bypass untrusted historical scope",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["untrusted history remains blocking".into()],
            ..start_options()
        },
    )
    .expect_err("recovery must remain blocked by untrusted history");
    assert!(
        error
            .to_string()
            .contains("archived_work_item_scope_untrusted:WI-OLD"),
        "unexpected recovery blocker: {error}"
    );
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
