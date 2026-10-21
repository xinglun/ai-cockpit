//! Library-domain regressions migrated from CLI fixtures; no CLI transport authority is supplied.
#![cfg(test)]
use super::approved_boundary_fixtures as boundary;
use crate as cockpit_repository;
use crate::review_authority::pipeline_tests::TestAuthority;
use cockpit_core::Digest;
use cockpit_protocol::HumanDecision;
use cockpit_protocol::PROTOCOL_VERSION;
use cockpit_protocol::RuntimeContext;
use cockpit_repository::RepositoryVerificationPolicy;
use cockpit_repository::RepositoryVerificationRequest;
use cockpit_repository::WorkItemStartOptions;
use cockpit_repository::archive_work_item_with_runtime;
use cockpit_repository::attach;
use cockpit_repository::checkpoint_work_item;
use cockpit_repository::finish_work_item_with_runtime;
use cockpit_repository::preflight_work_item;
use cockpit_repository::record_verification_with_runtime;
use cockpit_repository::recover_cross_checkout_closeout;
use cockpit_repository::run_repository_verification;
use cockpit_repository::start_work_item_with_options;
use cockpit_repository::work_item_status_snapshot_with_runtime;
use serde_json::Value;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
const ID: &str = "WI-LEGACY-ROOT";

fn git(root: &Path, args: &[&str]) -> Vec<u8> {
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
    output.stdout
}
fn commit(root: &Path, message: &str) {
    git(root, &["add", "-A"]);
    git(
        root,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            message,
        ],
    );
}
fn clone(source: &Path, destination: &Path) {
    git(
        source,
        &[
            "clone",
            "-q",
            "--no-hardlinks",
            "--config",
            "core.autocrlf=false",
            source.to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
}
fn old_runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "legacy-producer-1.0.0".into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"distinct historical producer fixture"),
    }
}
fn current_runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(
            b"private native legacy-root fixture runtime; not CLI transport",
        ),
    }
}
fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(root: &Path, directory: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                collect(root, &entry.path(), files);
            } else if entry.file_type().unwrap().is_file()
                && !entry.path().to_string_lossy().ends_with(".lock")
            {
                files.insert(
                    entry.path().strip_prefix(root).unwrap().into(),
                    fs::read(entry.path()).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    collect(root, &root.join(".ai"), &mut files);
    files
}
fn legacy_pair(
    authority: &TestAuthority,
) -> (tempfile::TempDir, PathBuf, tempfile::TempDir, PathBuf) {
    let origin = tempfile::tempdir().unwrap();
    let root = origin.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "core.autocrlf", "false"]);
    attach(root).unwrap();
    fs::write(root.join("README.md"), "legacy fixture baseline\n").unwrap();
    commit(root, "baseline");
    let historical = old_runtime();
    start_work_item_with_options(
        root,
        ID,
        "preserve a complete historical closeout",
        "register a present observation without reauthenticating its producer",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["complete immutable historical evidence".into()],
            ..Default::default()
        },
    )
    .unwrap();
    preflight_work_item(
        root,
        &root.join(format!(".ai/work-items/active/{ID}.contract.json")),
    )
    .unwrap();
    checkpoint_work_item(root, ID).unwrap();
    let execution = run_repository_verification(
        root,
        &RepositoryVerificationRequest {
            node_id: "historical-root-fixture".into(),
            program: "true".into(),
            args: vec![],
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: historical.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: Some(ID.into()),
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .unwrap();
    record_verification_with_runtime(
        root,
        ID,
        &serde_json::to_value(execution.receipt).unwrap(),
        &historical,
        &execution.final_snapshot,
    )
    .unwrap();
    finish_work_item_with_runtime(root, ID, &historical).unwrap();
    archive_work_item_with_runtime(root, ID, &historical).unwrap();
    commit(root, "archive old producer evidence");
    let destination = tempfile::tempdir().unwrap();
    let dest = destination.path().join("clone");
    clone(root, &dest);
    boundary::close_structured(
        authority,
        root,
        ID,
        &HumanDecision {
            decision: "approved".into(),
            actor: "test-data:historical-human".into(),
            authority_source: "isolated fixture".into(),
            reason: "historical close test data".into(),
            evidence_refs: vec![format!(".ai/evidence/{ID}.verification.json")],
            policy_refs: vec![],
            decided_at: "2026-10-01T00:00:00Z".into(),
            resume_condition: None,
        },
    )
    .unwrap();
    boundary::approve_initial_cross_checkout(authority, &dest, root, ID, &historical);
    recover_cross_checkout_closeout(&dest, root, ID, &historical).unwrap();
    // TEST FIXTURE ONLY: represent an old-v1 producer without modifying any
    // real repository evidence. Keep every payload and producer field.
    let receipt = dest.join(format!(
        ".ai/decisions/{ID}.cross-checkout-closeout-recovery.json"
    ));
    let mut v: Value = serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
    v["schemaVersion"] = json!(1);
    v.as_object_mut().unwrap().remove("originBinding");
    fs::write(receipt, serde_json::to_vec_pretty(&v).unwrap()).unwrap();
    fs::remove_file(dest.join(format!(
        ".ai/decisions/{ID}.cross-checkout-closeout-origin.json"
    )))
    .unwrap();
    commit(&dest, "complete legacy-v1 fixture");
    let independent = tempfile::tempdir().unwrap();
    let source = independent.path().join("source");
    clone(&dest, &source);
    let status = work_item_status_snapshot_with_runtime(&dest, ID, &current_runtime()).unwrap();
    assert_eq!(status.lifecycle_phase, "closed");
    assert_ne!(status.verification, "verified");
    (destination, dest, independent, source)
}

fn reseal_external_plan(
    plan: &mut cockpit_protocol::LegacyCloseoutSourcePlan,
    registration: &cockpit_protocol::LegacyCloseoutSourceRegistration,
) {
    plan.registration_bytes = serde_json::to_string_pretty(registration).unwrap();
    plan.registration_raw_digest = Digest::sha256_bytes(plan.registration_bytes.as_bytes());
    let mut requirement: cockpit_protocol::LegacyCloseoutSourceRequirement =
        serde_json::from_str(&plan.requirement_bytes).unwrap();
    requirement.registration_bytes = plan.registration_bytes.clone();
    requirement.registration_raw_digest = plan.registration_raw_digest.clone();
    plan.requirement_bytes = serde_json::to_string_pretty(&requirement).unwrap();
    plan.requirement_raw_digest = Digest::sha256_bytes(plan.requirement_bytes.as_bytes());
    let mut value = serde_json::to_value(&plan).unwrap();
    value.as_object_mut().unwrap().remove("planDigest");
    plan.plan_digest = cockpit_protocol::digest_json(&value).unwrap();
}

#[test]
fn direct_record_rejects_self_source_even_when_every_external_digest_is_resealed() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    let mut plan = original_plan(&dest, &source);
    let mut registration: cockpit_protocol::LegacyCloseoutSourceRegistration =
        serde_json::from_str(&plan.registration_bytes).unwrap();
    registration.source.root = dest.to_string_lossy().into_owned();
    registration.source.git_common_directory =
        plan.destination_binding.git_common_directory.clone();
    registration.source.snapshot_digest = cockpit_repository::snapshot_digest(
        &cockpit_git::GitRepository::discover(&dest)
            .unwrap()
            .snapshot()
            .unwrap(),
    )
    .unwrap();
    plan.source_binding = Some(plan.destination_binding.clone());
    reseal_external_plan(&mut plan, &registration);
    boundary::approve_legacy_registration(&authority, &dest, ID, &plan, &current_runtime());
    let before = files(&dest);
    let result =
        cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &current_runtime());
    assert!(
        result.is_err(),
        "external plan is self-consistent but has no independent source: {result:?}"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("independent_source_required")
    );
    assert_eq!(
        files(&dest),
        before,
        "direct record must reject before any canonical publication"
    );
}

#[test]
fn direct_record_rechecks_actual_source_repository_identity() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    let plan = original_plan(&dest, &source);
    let config = source.join(".ai/cockpit.toml");
    let old = cockpit_repository::repository_id(&source).to_string();
    let foreign = Digest::sha256_bytes(b"foreign source identity").to_string();
    let text = fs::read_to_string(&config).unwrap();
    assert!(text.contains(&old));
    fs::write(config, text.replace(&old, &foreign)).unwrap();
    assert_eq!(
        cockpit_repository::repository_id(&source).to_string(),
        foreign
    );
    boundary::approve_legacy_registration(&authority, &dest, ID, &plan, &current_runtime());
    let before = files(&dest);
    let result =
        cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &current_runtime());
    assert!(
        result.is_err(),
        "source identity drift cannot be accepted from an old plan: {result:?}"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("independent_source_required")
    );
    assert_eq!(files(&dest), before);
}

#[test]
fn direct_record_rejects_different_source_path_with_shared_git_common_directory() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    let owner = tempfile::tempdir().unwrap();
    let linked = owner.path().join("shared-source");
    git(
        &dest,
        &[
            "worktree",
            "add",
            "--quiet",
            "--detach",
            linked.to_str().unwrap(),
            "HEAD",
        ],
    );
    let linked = linked.canonicalize().unwrap();
    let linked_plan = original_plan(&linked, &source);
    let mut plan = original_plan(&dest, &source);
    let mut registration: cockpit_protocol::LegacyCloseoutSourceRegistration =
        serde_json::from_str(&plan.registration_bytes).unwrap();
    assert_ne!(linked, dest);
    assert_eq!(
        linked_plan.destination_binding.git_common_directory,
        plan.destination_binding.git_common_directory
    );
    registration.source.root = linked.to_string_lossy().into_owned();
    registration.source.git_common_directory =
        linked_plan.destination_binding.git_common_directory.clone();
    registration.source.snapshot_digest = cockpit_repository::snapshot_digest(
        &cockpit_git::GitRepository::discover(&linked)
            .unwrap()
            .snapshot()
            .unwrap(),
    )
    .unwrap();
    plan.source_binding = Some(linked_plan.destination_binding);
    reseal_external_plan(&mut plan, &registration);
    boundary::approve_legacy_registration(&authority, &dest, ID, &plan, &current_runtime());
    let before = files(&dest);
    let error =
        cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &current_runtime())
            .unwrap_err();
    assert!(
        error.to_string().contains("independent_source_required"),
        "{error}"
    );
    assert_eq!(files(&dest), before);
}

fn original_plan(dest: &Path, source: &Path) -> cockpit_protocol::LegacyCloseoutSourcePlan {
    let revision = String::from_utf8(git(source, &["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    let request:cockpit_protocol::LegacyCloseoutSourceRequest=serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:registrar","authoritySource":"isolated fixture","reason":"accept exact source","sourceAccepted":true})).unwrap();
    cockpit_repository::plan_legacy_closeout_source(
        dest,
        Some(source),
        ID,
        &request,
        &current_runtime(),
    )
    .unwrap()
}

#[test]
fn forged_resume_cannot_create_a_first_root_after_source_disappears() {
    let authority = TestAuthority::install();
    let (_destination, dest, source_owner, source) = legacy_pair(&authority);
    let mut plan = original_plan(&dest, &source);
    drop(source_owner);
    // An input author can recompute self-checking JSON digests. They cannot
    // turn a first registration into resume to skip the full pinned source.
    plan.operation = cockpit_protocol::LegacyCloseoutSourceOperation::Resume;
    let mut value = serde_json::to_value(&plan).unwrap();
    value.as_object_mut().unwrap().remove("planDigest");
    plan.plan_digest = cockpit_protocol::digest_json(&value).unwrap();
    boundary::approve_legacy_registration(&authority, &dest, ID, &plan, &current_runtime());
    let result =
        cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &current_runtime());
    assert!(
        result.is_err(),
        "resume without a surviving obligation must not publish a first root"
    );
    assert!(
        !dest
            .join(format!(
                ".ai/decisions/{ID}.legacy-closeout-source-requirement.json"
            ))
            .exists()
    );
    assert!(
        !dest
            .join(format!(
                ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
            ))
            .exists()
    );
}

#[test]
fn malformed_extension_remains_an_obligation_after_valid_root_registration() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    let plan = original_plan(&dest, &source);
    boundary::approve_legacy_registration(&authority, &dest, ID, &plan, &current_runtime());
    cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &current_runtime())
        .unwrap();
    fs::write(
        dest.join(format!(
            ".ai/decisions/{ID}.legacy-closeout-source-extension.json"
        )),
        b"{broken",
    )
    .unwrap();
    let status = work_item_status_snapshot_with_runtime(&dest, ID, &current_runtime()).unwrap();
    assert_ne!(
        status.lifecycle_phase, "closed",
        "a valid root does not authorize a malformed extension"
    );
    assert!(status.blocking, "surviving malformed extension must block");
}

#[test]
fn lost_archive_keeps_structured_status_and_index_blocking() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    let plan = original_plan(&dest, &source);
    boundary::approve_legacy_registration(&authority, &dest, ID, &plan, &current_runtime());
    cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &current_runtime())
        .unwrap();
    fs::remove_dir_all(dest.join(".ai/work-items/archive")).unwrap();
    let status = work_item_status_snapshot_with_runtime(&dest, ID, &current_runtime())
        .expect("canonical obligations must retain a structured status without the Contract");
    assert!(status.blocking);
    assert_ne!(status.lifecycle_phase, "closed");
    assert!(
        status
            .blockers
            .contains(&"legacy_closeout_source_integrity_invalid".into())
    );
    let index =
        cockpit_repository::work_item_status_index_with_runtime(&dest, &current_runtime()).unwrap();
    let value = serde_json::to_value(index).unwrap();
    assert!(
        value["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["workItemId"] == ID),
        "aggregate index must union canonical proof names: {value}"
    );
}

#[test]
fn delegated_observation_and_raw_dependencies_are_bound_and_loss_revokes_closed() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    let raw = b"provider observation fixture\n";
    let raw_ref = format!(".ai/evidence/external/{ID}.provider.raw");
    let evidence = cockpit_protocol::DelegatedEvidence {
        provider: "fixture".into(),
        subject: "complete legacy dependency".into(),
        origin: "isolated fixture API".into(),
        assurance: cockpit_protocol::EvidenceAssurance::SelfDeclared,
        collected_at: "2026-10-01T00:00:00Z".into(),
        digest: Digest::sha256_bytes(raw),
        validity: cockpit_protocol::EvidenceValidity::Valid,
        raw_evidence_ref: raw_ref.clone(),
    };
    cockpit_repository::import_delegated_evidence(&dest, ID, &evidence, raw, &old_runtime())
        .unwrap();
    let observation = cockpit_repository::import_delegated_evidence_observation(
        &dest,
        ID,
        &evidence,
        raw,
        &old_runtime(),
    )
    .unwrap();
    commit(&dest, "retain full delegated and observation dependencies");
    git(&source, &["pull", "--ff-only", "--quiet"]);
    let plan = original_plan(&dest, &source);
    let registration: Value = serde_json::from_str(&plan.registration_bytes).unwrap();
    let bindings = registration["dependencyBindings"].as_array().unwrap();
    assert!(
        bindings.iter().any(|entry| entry["path"] == raw_ref),
        "raw bytes must be an explicit dependency"
    );
    assert!(
        bindings.iter().any(|entry| entry["path"]
            .as_str()
            .unwrap()
            .contains(&observation.receipt.observation_digest.to_string()[7..])),
        "exact observation selector must be retained"
    );
    boundary::approve_legacy_registration(&authority, &dest, ID, &plan, &current_runtime());
    cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &current_runtime())
        .unwrap();
    fs::remove_file(dest.join(raw_ref)).unwrap();
    let status = work_item_status_snapshot_with_runtime(&dest, ID, &current_runtime()).unwrap();
    assert_ne!(status.lifecycle_phase, "closed");
    assert!(status.blocking);
}

#[test]
fn malformed_delegated_digest_returns_error_without_panicking() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    fs::create_dir_all(dest.join(".ai/evidence/external")).unwrap();
    let bad = json!({"schemaVersion":1,"repositoryId":cockpit_repository::repository_id(&dest).to_string(),"workItemId":ID,"runtimeVersion":"historical","runtimeDigest":Digest::sha256_bytes(b"historical"),"boundAt":"2026-10-01T00:00:00Z","evidence":{"provider":"fixture","subject":"bad digest","origin":"isolated fixture","assurance":"self_declared","collectedAt":"2026-10-01T00:00:00Z","digest":"x","validity":"valid","rawEvidenceRef":format!(".ai/evidence/external/{ID}.raw")}});
    fs::write(
        dest.join(format!(".ai/evidence/external/{ID}.bad.delegated.json")),
        serde_json::to_vec_pretty(&bad).unwrap(),
    )
    .unwrap();
    commit(&dest, "malformed dependency fixture");
    git(&source, &["pull", "--ff-only", "--quiet"]);
    let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    let request:cockpit_protocol::LegacyCloseoutSourceRequest=serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"fixture","authoritySource":"fixture","reason":"malformed input must be an error","sourceAccepted":true})).unwrap();
    let result = std::panic::catch_unwind(|| {
        cockpit_repository::plan_legacy_closeout_source(
            &dest,
            Some(&source),
            ID,
            &request,
            &current_runtime(),
        )
    });
    assert!(result.is_ok(), "malformed untrusted digests must not panic");
    assert!(result.unwrap().is_err());
}

#[test]
fn partial_root_resume_preserves_first_exact_bytes_with_independent_binding() {
    let authority = TestAuthority::install();
    for removed in ["requirement", "registration"] {
        let (_destination, dest, source_owner, source) = legacy_pair(&authority);
        let original = original_plan(&dest, &source);
        boundary::approve_legacy_registration(&authority, &dest, ID, &original, &current_runtime());
        cockpit_repository::record_legacy_closeout_source(&dest, ID, &original, &current_runtime())
            .unwrap();
        let path = dest.join(format!(
            ".ai/decisions/{ID}.legacy-closeout-source-{removed}.json"
        ));
        fs::remove_file(&path).unwrap();
        drop(source_owner);
        let request:cockpit_protocol::LegacyCloseoutSourceRequest=serde_json::from_value(json!({"schemaVersion":1,"operation":"resume","originalRegistrationBytes":original.registration_bytes,"originalRequirementRawDigest":original.requirement_raw_digest})).unwrap();
        let resume = cockpit_repository::plan_legacy_closeout_source(
            &dest,
            None,
            ID,
            &request,
            &current_runtime(),
        )
        .unwrap();
        boundary::approve_legacy_registration(&authority, &dest, ID, &resume, &current_runtime());
        let result = cockpit_repository::record_legacy_closeout_source(
            &dest,
            ID,
            &resume,
            &current_runtime(),
        )
        .unwrap();
        assert_eq!(result["state"], "committed");
        assert_eq!(
            fs::read(dest.join(format!(
                ".ai/decisions/{ID}.legacy-closeout-source-requirement.json"
            )))
            .unwrap(),
            original.requirement_bytes.as_bytes()
        );
        assert_eq!(
            fs::read(dest.join(format!(
                ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
            )))
            .unwrap(),
            original.registration_bytes.as_bytes()
        );
    }
}

#[test]
fn damaged_survivor_cannot_become_original_through_a_fresh_plan() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    let original = original_plan(&dest, &source);
    boundary::approve_legacy_registration(&authority, &dest, ID, &original, &current_runtime());
    cockpit_repository::record_legacy_closeout_source(&dest, ID, &original, &current_runtime())
        .unwrap();
    fs::remove_file(dest.join(format!(
        ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
    )))
    .unwrap();
    let path = dest.join(format!(
        ".ai/decisions/{ID}.legacy-closeout-source-requirement.json"
    ));
    let mut requirement: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let mut changed: Value =
        serde_json::from_str(requirement["registrationBytes"].as_str().unwrap()).unwrap();
    changed["actor"] = json!("changed actor");
    let changed_bytes = serde_json::to_vec_pretty(&changed).unwrap();
    requirement["registrationBytes"] = String::from_utf8(changed_bytes.clone()).unwrap().into();
    requirement["registrationRawDigest"] = Digest::sha256_bytes(&changed_bytes).to_string().into();
    let changed_requirement = serde_json::to_vec_pretty(&requirement).unwrap();
    fs::write(&path, &changed_requirement).unwrap();
    let request:cockpit_protocol::LegacyCloseoutSourceRequest=serde_json::from_value(json!({"schemaVersion":1,"operation":"resume","originalRegistrationBytes":original.registration_bytes,"originalRequirementRawDigest":original.requirement_raw_digest})).unwrap();
    assert!(
        cockpit_repository::plan_legacy_closeout_source(
            &dest,
            None,
            ID,
            &request,
            &current_runtime()
        )
        .is_err()
    );
    let no_original: cockpit_protocol::LegacyCloseoutSourceRequest =
        serde_json::from_value(json!({"schemaVersion":1,"operation":"resume"})).unwrap();
    assert!(
        cockpit_repository::plan_legacy_closeout_source(
            &dest,
            None,
            ID,
            &no_original,
            &current_runtime()
        )
        .is_err()
    );
    assert_eq!(fs::read(path).unwrap(), changed_requirement);
}
#[test]
fn complete_source_rejects_missing_legacy_identity_header_without_rewriting_history() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    let receipt = format!(".ai/decisions/{ID}.cross-checkout-closeout-recovery.json");
    for root in [&dest, &source] {
        let path = root.join(&receipt);
        let mut legacy: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        legacy.as_object_mut().unwrap().remove("runtimeDigest");
        fs::write(path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();
    }
    commit(&source, "preserve incomplete legacy identity fixture");
    // New registration is strict; unregistered historical compatibility is
    // independently preserved rather than silently changing old evidence.
    assert_eq!(
        work_item_status_snapshot_with_runtime(&dest, ID, &current_runtime())
            .unwrap()
            .lifecycle_phase,
        "closed"
    );
    let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    let request=serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:registrar","authoritySource":"isolated fixture","reason":"reject incomplete original identity","sourceAccepted":true})).unwrap();
    let result = cockpit_repository::plan_legacy_closeout_source(
        &dest,
        Some(&source),
        ID,
        &request,
        &current_runtime(),
    );
    assert!(
        result.is_err(),
        "a complete pinned source cannot replace missing legacy Runtime identity"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("legacy_receipt_invalid")
    );
}
#[test]
fn register_request_cannot_silently_ignore_extension_fields() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    let request=serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:registrar","authoritySource":"isolated fixture","reason":"require one unambiguous operation","sourceAccepted":true,"transitionBytes":"unrelated operation content"})).unwrap();
    let result = cockpit_repository::plan_legacy_closeout_source(
        &dest,
        Some(&source),
        ID,
        &request,
        &current_runtime(),
    );
    assert!(
        result.is_err(),
        "typed operation roles must reject ignored fields"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("request_role_invalid")
    );
}
#[test]
fn fresh_plan_rejects_identical_content_destination_root_replacement() {
    let authority = TestAuthority::install();
    assert_replacement_rejected(&authority, "destination_root");
}
#[test]
fn fresh_plan_rejects_identical_content_destination_git_replacement() {
    let authority = TestAuthority::install();
    assert_replacement_rejected(&authority, "destination_git");
}
#[test]
fn fresh_plan_rejects_identical_content_source_root_replacement() {
    let authority = TestAuthority::install();
    assert_replacement_rejected(&authority, "source_root");
}
fn assert_replacement_rejected(authority: &TestAuthority, subject: &str) {
    fn copy_tree(source: &Path, destination: &Path) {
        fs::create_dir_all(destination).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let target = destination.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }
    {
        let (_destination, dest, _source_owner, source) = legacy_pair(authority);
        let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        let request=serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:registrar","authoritySource":"isolated fixture","reason":"bind actual directories as well as bytes","sourceAccepted":true})).unwrap();
        let runtime = current_runtime();
        let plan = cockpit_repository::plan_legacy_closeout_source(
            &dest,
            Some(&source),
            ID,
            &request,
            &runtime,
        )
        .unwrap();
        let path = match subject {
            "destination_root" => dest.clone(),
            "destination_git" => dest.join(".git"),
            _ => source.clone(),
        };
        let retained = if subject == "source_root" {
            source.parent().unwrap()
        } else {
            dest.parent().unwrap()
        }
        .join(format!("retained-{subject}"));
        boundary::approve_legacy_registration(authority, &dest, ID, &plan, &runtime);
        fs::rename(&path, &retained).unwrap();
        copy_tree(&retained, &path);
        let result = cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &runtime);
        assert!(
            result.is_err(),
            "plan must reject physical replacement of {subject}"
        );
        assert!(
            !dest
                .join(format!(
                    ".ai/decisions/{ID}.legacy-closeout-source-requirement.json"
                ))
                .exists()
        );
        assert!(
            !dest
                .join(format!(
                    ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
                ))
                .exists()
        );
    }
}
#[test]
fn concurrent_root_writers_share_one_winner_and_preserve_conflicting_plans() {
    let authority = TestAuthority::install();
    for same_binding in [true, false] {
        let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
        let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        let request: cockpit_protocol::LegacyCloseoutSourceRequest = serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:first-registrar","authoritySource":"isolated fixture","reason":"serialize exact root writers","sourceAccepted":true})).unwrap();
        let runtime = current_runtime();
        let first = cockpit_repository::plan_legacy_closeout_source(
            &dest,
            Some(&source),
            ID,
            &request,
            &runtime,
        )
        .unwrap();
        let second = if same_binding {
            first.clone()
        } else {
            let mut other = request;
            other.actor = Some("test-data:conflicting-registrar".into());
            cockpit_repository::plan_legacy_closeout_source(
                &dest,
                Some(&source),
                ID,
                &other,
                &runtime,
            )
            .unwrap()
        };
        boundary::approve_legacy_registration(&authority, &dest, ID, &first, &runtime);
        if !same_binding {
            boundary::approve_legacy_registration(&authority, &dest, ID, &second, &runtime);
        }
        let shared = authority.share();
        let barrier = std::sync::Barrier::new(2);
        let executions_before = authority.execution_count();
        let mut results = std::thread::scope(|scope| {
            let handles = [&first, &second]
                .into_iter()
                .map(|plan| {
                    let barrier = &barrier;
                    let dest = &dest;
                    let runtime = &runtime;
                    let shared = shared.clone();
                    scope.spawn(move || {
                        let _thread_authority = shared.install();
                        barrier.wait();
                        cockpit_repository::record_legacy_closeout_source(dest, ID, plan, runtime)
                    })
                })
                .collect::<Vec<_>>();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        });
        if same_binding {
            let completed = results
                .iter()
                .find_map(|result| result.as_ref().ok())
                .cloned()
                .expect("one concurrent operation actually completed");
            let after_execution = authority.execution_count();
            assert_eq!(
                after_execution,
                executions_before + 1,
                "same operation may reserve only one execution"
            );
            let completed_files = files(&dest);
            for result in &mut results {
                if let Err(cause) = result {
                    let reason = cause.to_string();
                    assert!(
                        reason.contains("human_execution_recovery_required")
                            || reason.contains("human_review_not_active")
                            || reason.contains("execution_running"),
                        "only the protected in-progress rejection permits this test's explicit terminal lookup: {reason}"
                    );
                    // Both threads have joined and a successful terminal is
                    // known. This is a test's explicit cached-result lookup,
                    // never automatic production retry after an unknown effect.
                    *result = cockpit_repository::record_legacy_closeout_source(
                        &dest, ID, &first, &runtime,
                    );
                }
                assert_eq!(
                    result.as_ref().unwrap(),
                    &completed,
                    "all completed lookups return the same original result"
                );
            }
            assert_eq!(
                authority.execution_count(),
                after_execution,
                "known-terminal replay cannot execute again"
            );
            assert_eq!(
                files(&dest),
                completed_files,
                "known-terminal replay cannot publish again"
            );
        }
        assert_eq!(
            results.iter().filter(|result| result.is_ok()).count(),
            if same_binding { 2 } else { 1 }
        );
        let bytes = fs::read(dest.join(format!(
            ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
        )))
        .unwrap();
        assert!(
            bytes == first.registration_bytes.as_bytes()
                || bytes == second.registration_bytes.as_bytes()
        );
        let winner = if bytes == first.registration_bytes.as_bytes() {
            &first
        } else {
            &second
        };
        assert_eq!(
            fs::read(dest.join(format!(
                ".ai/decisions/{ID}.legacy-closeout-source-requirement.json"
            )))
            .unwrap(),
            winner.requirement_bytes.as_bytes()
        );
        assert_eq!(
            work_item_status_snapshot_with_runtime(&dest, ID, &runtime)
                .unwrap()
                .lifecycle_phase,
            "closed"
        );
    }
}
#[test]
fn read_only_plan_cannot_return_an_unrecordable_oversized_template() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    let request = serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"a".repeat(5 * 1024 * 1024),"authoritySource":"isolated fixture","reason":"bound the complete frozen plan","sourceAccepted":true})).unwrap();
    let before = files(&dest);
    let result = cockpit_repository::plan_legacy_closeout_source(
        &dest,
        Some(&source),
        ID,
        &request,
        &current_runtime(),
    );
    assert!(
        result.is_err(),
        "the whole serialized plan must fit the input budget"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("record_budget_exceeded")
    );
    assert_eq!(files(&dest), before);
}

#[test]
fn every_bound_history_file_and_each_root_obligation_remain_required() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    let plan = original_plan(&dest, &source);
    let runtime = current_runtime();
    boundary::approve_legacy_registration(&authority, &dest, ID, &plan, &runtime);
    cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &runtime).unwrap();
    let registration: cockpit_protocol::LegacyCloseoutSourceRegistration =
        serde_json::from_str(&plan.registration_bytes).unwrap();
    let paths = registration
        .inventory
        .iter()
        .map(|file| file.path.clone())
        .chain([
            format!(".ai/decisions/{ID}.legacy-closeout-source-requirement.json"),
            format!(".ai/decisions/{ID}.legacy-closeout-source-registration.json"),
        ]);
    for relative in paths {
        let path = dest.join(&relative);
        let original = fs::read(&path).unwrap();
        for missing in [true, false] {
            if missing {
                fs::remove_file(&path).unwrap();
            } else {
                let mut changed = original.clone();
                changed.push(b'\n');
                fs::write(&path, changed).unwrap();
            }
            let status = work_item_status_snapshot_with_runtime(&dest, ID, &runtime).unwrap();
            assert_ne!(
                status.lifecycle_phase, "closed",
                "{relative} missing={missing}"
            );
            assert!(status.blocking, "{relative} missing={missing}");
            let observed = cockpit_repository::status(&dest).unwrap();
            assert!(
                observed
                    .readiness
                    .unclosed_archived_work_items
                    .contains(&ID.to_owned()),
                "{relative} missing={missing}"
            );
            fs::write(&path, &original).unwrap();
            assert_eq!(
                work_item_status_snapshot_with_runtime(&dest, ID, &runtime)
                    .unwrap()
                    .lifecycle_phase,
                "closed"
            );
        }
    }
    let manifest_path = dest.join(format!(".ai/work-items/archive/{ID}.archive.json"));
    let original_manifest = fs::read(&manifest_path).unwrap();
    let mut manifest: Value = serde_json::from_slice(&original_manifest).unwrap();
    manifest["closeRequired"] = json!(false);
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    assert_ne!(
        work_item_status_snapshot_with_runtime(&dest, ID, &runtime)
            .unwrap()
            .lifecycle_phase,
        "closed"
    );
    assert!(
        cockpit_repository::status(&dest)
            .unwrap()
            .readiness
            .unclosed_archived_work_items
            .contains(&ID.to_owned())
    );
    let attempt = start_work_item_with_options(
        &dest,
        "WI-OVERLAPPING-PROOF",
        "do not free scope",
        "reject a damaged registered obligation",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["keep the original registered scope".into()],
            ..Default::default()
        },
    );
    let error = attempt
        .expect_err("damaged registration scope must remain blocked")
        .to_string();
    assert!(
        error.contains(&format!("archived_work_item_scope_untrusted:{ID}")),
        "{error}"
    );
    fs::write(manifest_path, original_manifest).unwrap();
}
#[test]
fn registered_scope_cannot_be_changed_and_resealed_before_candidate_preflight() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    let plan = original_plan(&dest, &source);
    boundary::approve_legacy_registration(&authority, &dest, ID, &plan, &current_runtime());
    cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &current_runtime())
        .unwrap();
    let remote_head = String::from_utf8(git(&dest, &["symbolic-ref", "refs/remotes/origin/HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    git(&dest, &["update-ref", &remote_head, "HEAD"]);
    git(
        &dest,
        &["switch", "-q", "-c", "feature/scope-preflight-fixture"],
    );
    let candidate = "WI-SCOPE-PREFLIGHT";
    start_work_item_with_options(
        &dest,
        candidate,
        "independent registered scope",
        "respect a closed registered predecessor",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["do not trust rewritten scope".into()],
            ..Default::default()
        },
    )
    .expect("the intact closed registration actually admits this entry fixture");
    let candidate_path = dest.join(format!(".ai/work-items/active/{candidate}.contract.json"));
    let baseline = preflight_work_item(&dest, &candidate_path).unwrap();
    assert!(
        !baseline
            .blockers
            .iter()
            .any(|blocker| blocker.contains("archived_work_item_scope")),
        "{baseline:?}"
    );
    let contract_path = dest.join(format!(".ai/work-items/archive/{ID}.contract.json"));
    let mut contract: Value = serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
    contract["scope"] = json!(["tests/**"]);
    let bytes = serde_json::to_vec_pretty(&contract).unwrap();
    fs::write(&contract_path, &bytes).unwrap();
    let manifest_path = dest.join(format!(".ai/work-items/archive/{ID}.archive.json"));
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["files"]["contractDigest"] = json!(Digest::sha256_bytes(&bytes));
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    let blocked = preflight_work_item(&dest, &candidate_path).unwrap();
    assert!(
        blocked
            .blockers
            .contains(&format!("archived_work_item_scope_untrusted:{ID}")),
        "{blocked:?}"
    );
}

#[test]
fn review_regression_cli_and_mcp_reject_relative_source_paths() {
    let authority = TestAuthority::install();
    let (_destination, dest, _source_owner, source) = legacy_pair(&authority);
    let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    let request = json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:registrar","authoritySource":"isolated fixture","reason":"use one absolute source path policy","sourceAccepted":true});
    let input = tempfile::NamedTempFile::new().unwrap();
    fs::write(input.path(), serde_json::to_vec(&request).unwrap()).unwrap();
    let cli = Command::new(super::transport_fixture::binary())
        .current_dir(source.parent().unwrap())
        .args([
            "work-item",
            "legacy-closeout-source-plan",
            "--repo",
            dest.to_str().unwrap(),
            "--source-repo",
            source.file_name().unwrap().to_str().unwrap(),
            "--id",
            ID,
            "--input",
            input.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        !cli.status.success(),
        "CLI must use the same absolute-source policy as MCP"
    );
    assert!(String::from_utf8_lossy(&cli.stderr).contains("sourceRepo must be an absolute path"));
    let mcp = super::transport_fixture::mcp_request(
        &dest,
        &json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"work_item_legacy_closeout_source_plan","arguments":{"workItemId":ID,"sourceRepo":"source","request":request}}}),
    );
    assert_eq!(mcp["result"]["isError"], true);
    assert!(
        mcp["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("sourceRepo must be an absolute path")
    );
}
