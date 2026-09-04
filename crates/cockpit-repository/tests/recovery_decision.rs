use cockpit_core::Digest;
use cockpit_protocol::{HumanDecision, OutcomeState, ResourceFinalizationContext, RuntimeContext};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item, archive_work_item_with_runtime, attach, checkpoint_work_item,
    close_work_item_with_structured_decision, close_work_item_with_structured_decision_and_runtime,
    finish_work_item, finish_work_item_with_runtime, outcome_v2, outcome_v2_with_runtime,
    plan_resource_finalization, preflight_work_item, preflight_work_item_with_runtime,
    record_recovery_decision, record_verification_with_runtime, render_human_outcome,
    repository_id, revalidate_contract_amendment, run_repository_verification, snapshot_digest,
    start_work_item_with_options, status,
};
use serde_json::json;
use std::fs;
use std::process::Command;

fn commit(path: &std::path::Path, message: &str) {
    assert!(
        Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .status()
            .expect("git add")
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.email=test@example.invalid",
                "-c",
                "user.name=test",
                "commit",
                "-qm",
                message,
            ])
            .current_dir(path)
            .status()
            .expect("git commit")
            .success()
    );
}

fn repository_snapshot_digest(path: &std::path::Path) -> Digest {
    let snapshot = cockpit_git::GitRepository::discover(path)
        .expect("discover")
        .snapshot()
        .expect("snapshot");
    snapshot_digest(&snapshot).expect("snapshot digest")
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
    attach(directory.path()).expect("attach");
    start_work_item_with_options(
        directory.path(),
        "WI-BLOCKED",
        "recover a blocked item",
        "record an explicit recovery decision",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["predecessor remains immutable".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    let contract = directory
        .path()
        .join(".ai/work-items/active/WI-BLOCKED.contract.json");
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), "WI-BLOCKED").expect("checkpoint");
    fs::write(
        directory
            .path()
            .join(".ai/work-items/active/WI-BLOCKED.outcome.json"),
        br#"{"state":"blocked","workItemId":"WI-BLOCKED"}"#,
    )
    .unwrap();
    fs::write(
        directory
            .path()
            .join(".ai/work-items/active/WI-BLOCKED.events.jsonl"),
        br#"{"schemaVersion":1,"eventId":"blocked-1","repositoryId":"REPOSITORY_ID","workItemId":"WI-BLOCKED","eventType":"blocked","timestamp":"2026-08-23T00:00:00Z","detail":"blocked for recovery"}
"#,
    )
    .unwrap();
    let events_path = directory
        .path()
        .join(".ai/work-items/active/WI-BLOCKED.events.jsonl");
    let mut events = fs::read_to_string(&events_path).unwrap();
    events = events.replace(
        "REPOSITORY_ID",
        &repository_id(directory.path()).to_string(),
    );
    fs::write(events_path, events).unwrap();
    directory
}

fn receipt(directory: &tempfile::TempDir, reason: &str) -> serde_json::Value {
    let root = directory.path();
    let contract_path = root.join(".ai/work-items/active/WI-BLOCKED.contract.json");
    let summary_path = root.join(".ai/work-items/active/WI-BLOCKED.summary.json");
    let outcome_path = root.join(".ai/work-items/active/WI-BLOCKED.outcome.json");
    let events_path = root.join(".ai/work-items/active/WI-BLOCKED.events.jsonl");
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
    json!({
        "schemaVersion": 1,
        "decisionId": "work-item-recovery",
        "decision": "successor",
        "workItemId": "WI-BLOCKED",
        "repositoryId": repository_id(root),
        "predecessorWorkItemId": "WI-BLOCKED",
        "predecessorContractDigest": cockpit_protocol::digest_json(&contract).unwrap(),
        "predecessorSummaryDigest": cockpit_protocol::digest_json(&summary).unwrap(),
        "predecessorOutcomeDigest": cockpit_protocol::digest_json(&serde_json::from_slice::<serde_json::Value>(&fs::read(&outcome_path).unwrap()).unwrap()).unwrap(),
        "predecessorEventsDigest": Digest::sha256_bytes(&fs::read(events_path).unwrap()),
        "successorWorkItemId": "WI-SUCCESSOR",
        "runtimeVersion": "0.2.12",
        "runtimeDigest": Digest::sha256_bytes(b"runtime"),
        "actor": "human:owner",
        "authoritySource": "repository-local",
        "reason": reason,
        "evidenceRefs": [".ai/work-items/active/WI-BLOCKED.outcome.json"],
        "policyRefs": [],
        "decidedAt": "2026-08-23T00:00:00Z",
        "resumeCondition": "fresh verification evidence for the successor"
    })
}

fn current_runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "0.2.31".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime-0.2.31"),
    }
}

fn ready_archived_repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(directory.path()).expect("attach");
    let id = "WI-ARCHIVED-RECOVERY";
    start_work_item_with_options(
        directory.path(),
        id,
        "recover an archived item",
        "preserve immutable archive truth",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["archive remains immutable".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    plan_resource_finalization(
        directory.path(),
        id,
        &ResourceFinalizationContext {
            branch: "feature/archived-recovery".into(),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: "https://github.com/example/ai-cockpit/pull/340".into(),
        },
    )
    .expect("finalization plan");
    let contract = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    let runtime = RuntimeContext {
        runtime_version: "0.2.33".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"archived-recovery-runtime"),
    };
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "archived-recovery-check".into(),
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
    .expect("verify");
    let raw = serde_json::to_value(&run.receipt).expect("receipt JSON");
    record_verification_with_runtime(directory.path(), id, &raw, &runtime, &run.final_snapshot)
        .expect("verification");
    finish_work_item(directory.path(), id).expect("finish");
    archive_work_item(directory.path(), id).expect("archive");
    directory
}

fn archived_recovery_receipt(
    directory: &tempfile::TempDir,
    decision: &str,
    successor: Option<&str>,
    runtime: &RuntimeContext,
) -> serde_json::Value {
    let root = directory.path();
    let id = "WI-ARCHIVED-RECOVERY";
    let archive = root.join(".ai/work-items/archive");
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(archive.join(format!("{id}.contract.json"))).unwrap())
            .unwrap();
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(archive.join(format!("{id}.summary.json"))).unwrap())
            .unwrap();
    let outcome: serde_json::Value =
        serde_json::from_slice(&fs::read(archive.join(format!("{id}.outcome.json"))).unwrap())
            .unwrap();
    let events_path = archive.join(format!("{id}.events.jsonl"));
    let mut receipt = json!({
        "schemaVersion": 1,
        "decisionId": "work-item-recovery",
        "decision": decision,
        "workItemId": id,
        "repositoryId": repository_id(root),
        "predecessorWorkItemId": id,
        "predecessorContractDigest": cockpit_protocol::digest_json(&contract).unwrap(),
        "predecessorSummaryDigest": cockpit_protocol::digest_json(&summary).unwrap(),
        "predecessorOutcomeDigest": cockpit_protocol::digest_json(&outcome).unwrap(),
        "predecessorEventsDigest": Digest::sha256_bytes(&fs::read(events_path).unwrap()),
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "actor": "human:owner",
        "authoritySource": "repository-owner",
        "reason": "recover archived predecessor after immutable base mismatch",
        "evidenceRefs": [format!(".ai/work-items/archive/{id}.outcome.json")],
        "policyRefs": ["docs/reference/agent-workflow.md"],
        "decidedAt": "2026-08-28T00:00:00Z",
        "resumeCondition": "continue on the successor Work Item"
    });
    if let Some(successor) = successor {
        receipt["successorWorkItemId"] = json!(successor);
    }
    receipt
}

fn write_forged_supersede(
    directory: &tempfile::TempDir,
    mutate: impl FnOnce(&mut serde_json::Value),
) {
    let runtime = current_runtime();
    let mut forged = receipt(directory, "forged read-side supersession");
    forged["decision"] = json!("supersede");
    forged["runtimeVersion"] = json!(runtime.runtime_version);
    forged["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    mutate(&mut forged);
    fs::write(
        directory
            .path()
            .join(".ai/decisions/WI-BLOCKED.recovery.json"),
        serde_json::to_vec_pretty(&forged).unwrap(),
    )
    .unwrap();
}

fn record_valid_supersede(directory: &tempfile::TempDir, runtime: &RuntimeContext) {
    let mut successor = receipt(directory, "create a bound successor");
    successor["runtimeVersion"] = json!(runtime.runtime_version);
    successor["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &successor, runtime)
        .expect("successor recovery");
    start_work_item_with_options(
        directory.path(),
        "WI-SUCCESSOR",
        "continue on the successor",
        "preserve predecessor recovery bindings",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["successor remains bound".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("activate successor scaffold");

    let mut supersede = receipt(directory, "supersede the bound predecessor");
    supersede["decision"] = json!("supersede");
    supersede["decidedAt"] = json!("2026-08-23T00:01:00Z");
    supersede["runtimeVersion"] = json!(runtime.runtime_version);
    supersede["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &supersede, runtime)
        .expect("supersede recovery");
}

#[test]
fn outcome_rejects_a_foreign_current_recovery_with_a_stable_unknown() {
    let directory = repository();
    write_forged_supersede(&directory, |forged| {
        forged["repositoryId"] = json!(Digest::sha256_bytes(b"foreign-repository"));
    });

    let outcome = outcome_v2_with_runtime(directory.path(), "WI-BLOCKED", &current_runtime())
        .expect("invalid recovery remains a renderable fail-closed Outcome");

    assert_eq!(outcome.state, OutcomeState::Unknown);
    assert_eq!(
        outcome.decision_state,
        Some(cockpit_core::DecisionState::Red)
    );
    assert!(
        outcome
            .unknowns
            .contains(&"recovery_decision_invalid".into()),
        "{:?}",
        outcome.unknowns
    );
    assert!(outcome.recovery_decision.is_none());
    assert_ne!(outcome.historical_status.as_deref(), Some("superseded"));
}

#[test]
fn readiness_accepts_a_closed_terminal_recovery_successor() {
    let directory = ready_archived_repository();
    let runtime = RuntimeContext {
        runtime_version: "0.2.33".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"archived-recovery-runtime"),
    };
    let predecessor = "WI-ARCHIVED-RECOVERY";
    let successor = "WI-SUCCESSOR";
    let recovery = archived_recovery_receipt(&directory, "successor", Some(successor), &runtime);
    record_recovery_decision(directory.path(), predecessor, &recovery, &runtime)
        .expect("record successor recovery");
    start_work_item_with_options(
        directory.path(),
        successor,
        "continue the recovered work",
        "complete a terminal successor with independently bound evidence",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["successor remains independently governed".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("activate recovery successor");
    plan_resource_finalization(
        directory.path(),
        successor,
        &ResourceFinalizationContext {
            branch: "feature/successor".into(),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: "https://github.com/example/ai-cockpit/pull/341".into(),
        },
    )
    .expect("successor finalization plan");
    let contract_path = directory
        .path()
        .join(format!(".ai/work-items/active/{successor}.contract.json"));
    preflight_work_item(directory.path(), &contract_path).expect("preflight before evidence");
    checkpoint_work_item(directory.path(), successor).expect("checkpoint successor");
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "successor-check".into(),
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
    .expect("verify successor");
    let verification = serde_json::to_value(&run.receipt).expect("verification JSON");
    record_verification_with_runtime(
        directory.path(),
        successor,
        &verification,
        &runtime,
        &run.final_snapshot,
    )
    .expect("record successor evidence");
    preflight_work_item(directory.path(), &contract_path).expect("green preflight");
    finish_work_item(directory.path(), successor).expect("finish successor");
    archive_work_item(directory.path(), successor).expect("archive successor");
    let pending_before_close = status(directory.path())
        .expect("repository status before successor close")
        .readiness
        .unclosed_archived_work_items;
    assert!(pending_before_close.contains(&predecessor.to_owned()));
    assert!(pending_before_close.contains(&successor.to_owned()));
    close_work_item_with_structured_decision(
        directory.path(),
        successor,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "repository-owner".into(),
            reason: "successor terminal evidence is complete".into(),
            evidence_refs: vec![format!(".ai/evidence/{successor}.verification.json")],
            policy_refs: vec!["docs/reference/repository-workflow.md".into()],
            decided_at: "2026-08-28T00:01:00Z".into(),
            resume_condition: None,
        },
    )
    .expect("close successor");

    let readiness = status(directory.path())
        .expect("repository status")
        .readiness;
    assert!(readiness.unclosed_archived_work_items.is_empty());
}

#[test]
fn archive_rejects_forged_current_recovery_bindings_without_moving_artifacts() {
    for case in [
        "repository",
        "runtime",
        "contract",
        "summary",
        "outcome",
        "events",
        "successor",
        "timestamp",
    ] {
        let directory = repository();
        write_forged_supersede(&directory, |forged| match case {
            "repository" => {
                forged["repositoryId"] = json!(Digest::sha256_bytes(b"foreign-repository"));
            }
            "runtime" => {
                forged["runtimeDigest"] = json!(Digest::sha256_bytes(b"foreign-runtime"));
            }
            "contract" => {
                forged["predecessorContractDigest"] =
                    json!(Digest::sha256_bytes(b"stale-contract"));
            }
            "summary" => {
                forged["predecessorSummaryDigest"] = json!(Digest::sha256_bytes(b"stale-summary"));
            }
            "outcome" => {
                forged["predecessorOutcomeDigest"] = json!(Digest::sha256_bytes(b"stale-outcome"));
            }
            "events" => {
                forged["predecessorEventsDigest"] = json!(Digest::sha256_bytes(b"stale-events"));
            }
            "successor" => forged["successorWorkItemId"] = json!("WI-MISSING-SUCCESSOR"),
            "timestamp" => forged["decidedAt"] = json!("not-a-timestamp"),
            _ => unreachable!(),
        });

        let error =
            archive_work_item_with_runtime(directory.path(), "WI-BLOCKED", &current_runtime())
                .expect_err("forged current recovery must fail closed");
        assert!(
            error.to_string().contains("recovery_decision_invalid"),
            "{case}: {error}"
        );
        assert!(
            directory
                .path()
                .join(".ai/work-items/active/WI-BLOCKED.contract.json")
                .is_file(),
            "{case} moved the predecessor"
        );
        assert!(
            !directory
                .path()
                .join(".ai/work-items/archive/WI-BLOCKED.archive.json")
                .exists(),
            "{case} created an archive manifest"
        );
    }
}

#[test]
fn recovery_read_side_detects_predecessor_and_successor_tamper_after_recording() {
    for target in ["predecessor-summary", "successor-contract"] {
        let directory = repository();
        let runtime = current_runtime();
        record_valid_supersede(&directory, &runtime);
        let path = match target {
            "predecessor-summary" => directory
                .path()
                .join(".ai/work-items/active/WI-BLOCKED.summary.json"),
            "successor-contract" => directory
                .path()
                .join(".ai/work-items/active/WI-SUCCESSOR.contract.json"),
            _ => unreachable!(),
        };
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        value["tamperedAfterRecovery"] = json!(true);
        fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

        let error = archive_work_item_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
            .expect_err("post-recovery artifact tamper must fail closed");
        assert!(
            error.to_string().contains("recovery_decision_invalid"),
            "{target}: {error}"
        );
        assert!(
            directory
                .path()
                .join(".ai/work-items/active/WI-BLOCKED.contract.json")
                .is_file(),
            "{target} moved the predecessor"
        );
    }
}

#[test]
fn archive_rejects_malformed_duplicate_and_oversized_current_candidates() {
    for case in ["malformed", "duplicate-key", "oversized"] {
        let directory = repository();
        let path = directory
            .path()
            .join(".ai/decisions/WI-BLOCKED.recovery.json");
        let bytes = match case {
            "malformed" => b"{".to_vec(),
            "duplicate-key" => {
                let serialized = serde_json::to_string(&receipt(&directory, "duplicate")).unwrap();
                serialized
                    .replacen('{', "{\"schemaVersion\":1,", 1)
                    .into_bytes()
            }
            "oversized" => vec![b' '; 4 * 1024 * 1024 + 1],
            _ => unreachable!(),
        };
        fs::write(path, bytes).unwrap();

        let error =
            archive_work_item_with_runtime(directory.path(), "WI-BLOCKED", &current_runtime())
                .expect_err("invalid current candidate must fail closed");
        assert!(
            error.to_string().contains("recovery_decision_invalid"),
            "{case}: {error}"
        );
        assert!(
            directory
                .path()
                .join(".ai/work-items/active/WI-BLOCKED.contract.json")
                .is_file(),
            "{case} moved the predecessor"
        );
    }
}

#[cfg(unix)]
#[test]
fn archive_rejects_a_symlink_current_candidate() {
    use std::os::unix::fs::symlink;

    let directory = repository();
    let target = directory.path().join("outside-recovery.json");
    fs::write(
        &target,
        serde_json::to_vec(&receipt(&directory, "symlink")).unwrap(),
    )
    .unwrap();
    symlink(
        &target,
        directory
            .path()
            .join(".ai/decisions/WI-BLOCKED.recovery.json"),
    )
    .unwrap();

    let error = archive_work_item_with_runtime(directory.path(), "WI-BLOCKED", &current_runtime())
        .expect_err("symlink recovery candidate must fail closed");
    assert!(
        error.to_string().contains("recovery_decision_invalid"),
        "{error}"
    );
}

#[test]
fn archive_rejects_a_misnamed_or_invalid_newer_candidate_instead_of_falling_back() {
    for case in ["misnamed", "invalid-newer"] {
        let directory = repository();
        let runtime = current_runtime();
        record_valid_supersede(&directory, &runtime);
        let decisions = directory.path().join(".ai/decisions");
        match case {
            "misnamed" => {
                let versioned = fs::read_dir(&decisions)
                    .unwrap()
                    .flatten()
                    .map(|entry| entry.path())
                    .find(|path| {
                        let name = path.file_name().unwrap().to_string_lossy();
                        name.starts_with("WI-BLOCKED.recovery.") && name.ends_with(".json")
                    })
                    .expect("versioned supersession receipt");
                fs::rename(
                    versioned,
                    decisions.join("WI-BLOCKED.recovery.not-the-content-digest.json"),
                )
                .unwrap();
            }
            "invalid-newer" => {
                fs::write(
                    decisions.join("WI-BLOCKED.recovery.ffffffff.json"),
                    b"{not-json",
                )
                .unwrap();
            }
            _ => unreachable!(),
        }

        let error = archive_work_item_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
            .expect_err("an invalid candidate must not be skipped in favor of an older valid one");
        assert!(
            error.to_string().contains("recovery_decision_invalid"),
            "{case}: {error}"
        );
    }
}

#[test]
fn equally_timed_valid_candidates_are_selected_by_a_deterministic_path_order() {
    let directory = repository();
    let runtime = current_runtime();
    record_valid_supersede(&directory, &runtime);

    let mut tied = receipt(&directory, "second equally timed supersession");
    tied["decision"] = json!("supersede");
    tied["decidedAt"] = json!("2026-08-23T00:01:00Z");
    tied["runtimeVersion"] = json!(runtime.runtime_version);
    tied["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &tied, &runtime)
        .expect("second equally timed supersession");

    let decisions = directory.path().join(".ai/decisions");
    let mut tied_paths = fs::read_dir(&decisions)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            name.starts_with("WI-BLOCKED.recovery.") && name.ends_with(".json")
        })
        .filter(|path| {
            let value: serde_json::Value =
                serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            value["decidedAt"] == "2026-08-23T00:01:00Z"
        })
        .collect::<Vec<_>>();
    tied_paths.sort();
    let expected: serde_json::Value =
        serde_json::from_slice(&fs::read(tied_paths.last().expect("tied receipts")).unwrap())
            .unwrap();

    let outcome = outcome_v2_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
        .expect("valid tied recovery candidates");
    assert_eq!(
        outcome.recovery_decision.unwrap().reason,
        expected["reason"].as_str().unwrap()
    );
}

#[test]
fn recovery_decision_binds_predecessor_and_projects_in_outcome() {
    let directory = repository();
    let runtime = RuntimeContext {
        runtime_version: "0.2.12".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime"),
    };
    let mut first = receipt(&directory, "create an explicit successor");
    first["runtimeVersion"] = json!(runtime.runtime_version);
    first["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &first, &runtime)
        .expect("first recovery receipt");
    let successor_contract: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-SUCCESSOR.contract.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(successor_contract["predecessorWorkItemId"], "WI-BLOCKED");
    assert_eq!(
        successor_contract["predecessorContractDigest"],
        first["predecessorContractDigest"]
    );
    let successor_contract =
        serde_json::from_value::<cockpit_protocol::Contract>(successor_contract).unwrap();
    successor_contract
        .validate()
        .expect("valid successor Contract");
    let mut second = receipt(&directory, "record a retry after the successor decision");
    second["decidedAt"] = json!("2026-08-23T00:01:00Z");
    second["decision"] = json!("retry");
    second
        .as_object_mut()
        .unwrap()
        .remove("successorWorkItemId");
    record_recovery_decision(directory.path(), "WI-BLOCKED", &second, &runtime)
        .expect("second recovery receipt is append-only");

    let decisions = fs::read_dir(directory.path().join(".ai/decisions"))
        .unwrap()
        .flatten()
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("WI-BLOCKED.recovery")
        })
        .count();
    assert_eq!(decisions, 2);
    let outcome = outcome_v2(directory.path(), "WI-BLOCKED").expect("outcome");
    assert_eq!(
        outcome.recovery_decision.as_ref().unwrap().reason,
        "record a retry after the successor decision"
    );
}

#[test]
fn recovery_rejects_competing_successor_for_same_predecessor() {
    let directory = repository();
    let runtime = current_runtime();
    let mut first = receipt(&directory, "select the first successor");
    first["runtimeVersion"] = json!(runtime.runtime_version);
    first["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &first, &runtime)
        .expect("first successor recovery");

    let mut competing = receipt(&directory, "attempt a competing successor");
    competing["successorWorkItemId"] = json!("WI-OTHER-SUCCESSOR");
    competing["runtimeVersion"] = json!(runtime.runtime_version);
    competing["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    let error = record_recovery_decision(directory.path(), "WI-BLOCKED", &competing, &runtime)
        .expect_err("a predecessor must not accumulate competing successors");
    assert!(error.to_string().contains("competing_successor"));
    assert!(
        !directory
            .path()
            .join(".ai/work-items/active/WI-OTHER-SUCCESSOR.contract.json")
            .exists()
    );
}

#[test]
fn recovery_rejects_foreign_runtime_and_predecessor_digest() {
    let directory = repository();
    let runtime = RuntimeContext {
        runtime_version: "0.2.12".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime"),
    };
    let mut foreign = receipt(&directory, "foreign");
    foreign["repositoryId"] = json!(Digest::sha256_bytes(b"foreign"));
    assert!(record_recovery_decision(directory.path(), "WI-BLOCKED", &foreign, &runtime).is_err());
    let mut stale = receipt(&directory, "stale");
    stale["predecessorSummaryDigest"] = json!(Digest::sha256_bytes(b"stale"));
    assert!(record_recovery_decision(directory.path(), "WI-BLOCKED", &stale, &runtime).is_err());
}

#[test]
fn retry_recovery_restores_checkpointed_state_after_failed_finish() {
    let directory = repository();
    let runtime = current_runtime();
    let summary_path = directory
        .path()
        .join(".ai/work-items/active/WI-BLOCKED.summary.json");
    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
    summary["checkpointEvidence"] = json!([
        {"stage": "before_finish", "recorded": true}
    ]);
    summary["state"] = json!("finish_ready");
    summary["failedGate"] = json!("finish.governance");
    summary["recoveryCondition"] = json!("retry after repairing the lifecycle gate");
    fs::write(&summary_path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();

    let mut retry = receipt(&directory, "retry after a failed finish projection");
    retry["decision"] = json!("retry");
    retry.as_object_mut().unwrap().remove("successorWorkItemId");
    retry["runtimeVersion"] = json!(runtime.runtime_version);
    retry["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    retry["decidedAt"] = json!("2026-08-23T00:04:00Z");
    record_recovery_decision(directory.path(), "WI-BLOCKED", &retry, &runtime)
        .expect("retry recovery should reopen the legal checkpointed state");

    let recovered: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
    assert_eq!(recovered["state"], "checkpointed");
    assert_eq!(recovered["checkpointCount"], 1);
    assert_eq!(recovered["preflightState"], "green");
    assert_eq!(
        recovered["checkpointEvidence"]
            .as_array()
            .expect("checkpoint evidence")
            .iter()
            .filter(|entry| entry.get("stage") == Some(&json!("before_finish")))
            .count(),
        0,
        "retry must refresh a stale terminal checkpoint candidate"
    );
    assert!(recovered.get("failedGate").is_none());
    assert!(recovered.get("recoveryCondition").is_none());
    let outcome: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-BLOCKED.outcome.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(outcome["state"], "blocked");
}

#[test]
fn retry_recovery_clears_failed_finish_marker_when_state_is_already_checkpointed() {
    let directory = repository();
    let runtime = current_runtime();
    let summary_path = directory
        .path()
        .join(".ai/work-items/active/WI-BLOCKED.summary.json");
    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
    summary["state"] = json!("checkpointed");
    summary["failedGate"] = json!("finish.governance");
    summary["recoveryCondition"] = json!("retry after restoring the lifecycle state");
    fs::write(&summary_path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();

    let mut retry = receipt(&directory, "retry a checkpointed failed finish");
    retry["decision"] = json!("retry");
    retry.as_object_mut().unwrap().remove("successorWorkItemId");
    retry["runtimeVersion"] = json!(runtime.runtime_version);
    retry["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    retry["decidedAt"] = json!("2026-08-23T00:06:00Z");
    record_recovery_decision(directory.path(), "WI-BLOCKED", &retry, &runtime)
        .expect("retry recovery should clear a stale failed-finish marker");

    let recovered: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
    assert_eq!(recovered["state"], "checkpointed");
    assert_eq!(recovered["recoveryRetryPending"], true);
    assert!(recovered.get("failedGate").is_none());
    assert!(recovered.get("recoveryCondition").is_none());
}

#[test]
fn retry_recovery_accepts_a_lifecycle_state_failure_with_red_preflight() {
    let directory = repository();
    let runtime = current_runtime();
    let summary_path = directory
        .path()
        .join(".ai/work-items/active/WI-BLOCKED.summary.json");
    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
    summary["state"] = json!("finish_ready");
    summary["preflightState"] = json!("red");
    summary["failedGate"] = json!("finish.governance");
    summary["recoveryCondition"] = json!("retry after restoring the lifecycle state");
    fs::write(&summary_path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();

    let mut retry = receipt(
        &directory,
        "retry the governance transition after a red preflight",
    );
    retry["decision"] = json!("retry");
    retry.as_object_mut().unwrap().remove("successorWorkItemId");
    retry["runtimeVersion"] = json!(runtime.runtime_version);
    retry["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    retry["decidedAt"] = json!("2026-08-23T00:05:00Z");
    record_recovery_decision(directory.path(), "WI-BLOCKED", &retry, &runtime)
        .expect("lifecycle state failure remains explicitly retryable");

    let recovered: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
    assert_eq!(recovered["state"], "checkpointed");
    assert_eq!(recovered["preflightState"], "red");
    assert!(recovered.get("failedGate").is_none());
    assert!(recovered.get("recoveryCondition").is_none());
}

#[test]
fn retry_verify_preflight_finish_keeps_recovery_receipt_bound_to_the_attempt() {
    let directory = repository();
    let id = "WI-BLOCKED";
    let runtime = current_runtime();
    plan_resource_finalization(
        directory.path(),
        id,
        &ResourceFinalizationContext {
            branch: "feature/recovery-binding".into(),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: "https://github.com/example/ai-cockpit/pull/recovery-binding".into(),
        },
    )
    .expect("finalization plan");

    let mut retry = receipt(&directory, "retry the failed lifecycle attempt");
    retry["decision"] = json!("retry");
    retry
        .as_object_mut()
        .expect("retry receipt object")
        .remove("successorWorkItemId");
    retry["runtimeVersion"] = json!(runtime.runtime_version);
    retry["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    retry["decidedAt"] = json!("2026-08-28T05:00:00Z");
    record_recovery_decision(directory.path(), id, &retry, &runtime).expect("retry recovery");

    // retry 後に Runtime 自身が追加する blocked projection を模擬する。
    // これらの bytes は外部の偽造ではなく、現在の試行に続く状態である。
    let outcome_path = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.outcome.json"));
    let mut projected_outcome: serde_json::Value =
        serde_json::from_slice(&fs::read(&outcome_path).expect("outcome")).expect("outcome JSON");
    projected_outcome["summary"] = json!("A later Runtime projection kept the item recoverable.");
    fs::write(
        &outcome_path,
        serde_json::to_vec_pretty(&projected_outcome).expect("projected outcome bytes"),
    )
    .expect("projected outcome");
    let events_path = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.events.jsonl"));
    let event = json!({
        "schemaVersion": 1,
        "eventId": "blocked-after-retry",
        "repositoryId": repository_id(directory.path()),
        "workItemId": id,
        "eventType": "blocked",
        "timestamp": "2026-08-28T05:01:00Z",
        "detail": "Runtime projected a recoverable retry state",
        "evidenceRefs": [],
        "relatedEventIds": [],
        "correctionOf": null
    });
    use std::io::Write;
    let mut events = fs::OpenOptions::new()
        .append(true)
        .open(&events_path)
        .expect("events");
    writeln!(
        events,
        "{}",
        serde_json::to_string(&event).expect("event JSON")
    )
    .expect("append event");

    let projected = outcome_v2_with_runtime(directory.path(), id, &runtime)
        .expect("Runtime-owned retry projection remains readable");
    assert_eq!(
        projected
            .recovery_decision
            .as_ref()
            .map(|value| value.decision.as_str()),
        Some("retry")
    );

    let git = cockpit_git::GitRepository::discover(directory.path()).expect("git repository");
    let snapshot = git.snapshot().expect("snapshot");
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "recovery-binding-check".into(),
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
    .expect("verification run");
    record_verification_with_runtime(
        directory.path(),
        id,
        &serde_json::to_value(&run.receipt).expect("receipt JSON"),
        &runtime,
        &snapshot,
    )
    .expect("record verification");

    let recovered_summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/work-items/active/{id}.summary.json")),
        )
        .expect("summary after retry verification"),
    )
    .expect("summary JSON");
    assert!(
        recovered_summary.get("recoveryRetryPending").is_none(),
        "fresh verification must consume the retry marker projection"
    );
    assert!(
        recovered_summary.get("recoveryRetryDecisionPath").is_none(),
        "fresh verification must not leave a stale retry path projection"
    );

    let decision = preflight_work_item_with_runtime(
        directory.path(),
        &directory
            .path()
            .join(format!(".ai/work-items/active/{id}.contract.json")),
        &runtime,
    )
    .expect("fresh preflight after verification");
    assert_eq!(decision.state, cockpit_core::DecisionState::Green);
    finish_work_item_with_runtime(directory.path(), id, &runtime)
        .expect("finish after retry verification and preflight");
}

#[test]
fn contract_amendment_allows_fresh_verification_to_reconcile_stale_evidence() {
    let directory = repository();
    let id = "WI-BLOCKED";
    let runtime = current_runtime();
    let contract_path = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));

    let initial_snapshot = cockpit_git::GitRepository::discover(directory.path())
        .expect("git repository")
        .snapshot()
        .expect("initial snapshot");
    let initial_run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "contract-amendment-initial-check".into(),
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
    .expect("initial verification run");
    record_verification_with_runtime(
        directory.path(),
        id,
        &serde_json::to_value(&initial_run.receipt).expect("initial receipt JSON"),
        &runtime,
        &initial_snapshot,
    )
    .expect("initial verification");

    plan_resource_finalization(
        directory.path(),
        id,
        &ResourceFinalizationContext {
            branch: "feature/contract-amendment-recovery".into(),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: "https://github.com/example/ai-cockpit/pull/355".into(),
        },
    )
    .expect("finalization plan");
    revalidate_contract_amendment(
        directory.path(),
        id,
        "bind the reviewed resource context after the initial verification",
    )
    .expect("contract amendment");

    let stale_preflight =
        preflight_work_item_with_runtime(directory.path(), &contract_path, &runtime)
            .expect("stale predecessor evidence should remain recoverable");
    assert_ne!(stale_preflight.state, cockpit_core::DecisionState::Red);
    assert!(
        !stale_preflight
            .blockers
            .contains(&"evidence_contradictory".into())
    );

    let refreshed_snapshot = cockpit_git::GitRepository::discover(directory.path())
        .expect("git repository")
        .snapshot()
        .expect("refreshed snapshot");
    let refreshed_run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "contract-amendment-fresh-check".into(),
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
    .expect("fresh verification run");
    record_verification_with_runtime(
        directory.path(),
        id,
        &serde_json::to_value(&refreshed_run.receipt).expect("fresh receipt JSON"),
        &runtime,
        &refreshed_snapshot,
    )
    .expect("fresh verification should reconcile the amendment");

    let decision = preflight_work_item_with_runtime(directory.path(), &contract_path, &runtime)
        .expect("fresh verification should restore green preflight");
    assert_eq!(decision.state, cockpit_core::DecisionState::Green);
}

#[test]
fn pending_retry_requires_a_matching_receipt_at_each_lifecycle_boundary() {
    for case in ["missing", "misnamed", "tampered"] {
        let directory = repository();
        let id = "WI-BLOCKED";
        let runtime = current_runtime();
        plan_resource_finalization(
            directory.path(),
            id,
            &ResourceFinalizationContext {
                branch: "feature/recovery-binding-negative".into(),
                worktree: directory.path().display().to_string(),
                base_branch: "main".into(),
                base_remote: "origin".into(),
                provider: "github".into(),
                pull_request:
                    "https://github.com/example/ai-cockpit/pull/recovery-binding-negative".into(),
            },
        )
        .expect("finalization plan");

        let mut retry = receipt(&directory, "retry with a pending current receipt");
        retry["decision"] = json!("retry");
        retry
            .as_object_mut()
            .expect("retry receipt object")
            .remove("successorWorkItemId");
        retry["runtimeVersion"] = json!(runtime.runtime_version);
        retry["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
        retry["decidedAt"] = json!("2026-08-28T05:02:00Z");
        record_recovery_decision(directory.path(), id, &retry, &runtime).expect("retry recovery");

        let canonical = directory
            .path()
            .join(".ai/decisions/WI-BLOCKED.recovery.json");
        match case {
            "missing" => fs::remove_file(&canonical).expect("remove retry receipt"),
            "misnamed" => fs::rename(
                &canonical,
                directory
                    .path()
                    .join(".ai/decisions/WI-BLOCKED.recovery.misnamed.json"),
            )
            .expect("rename retry receipt"),
            "tampered" => {
                let mut value: serde_json::Value =
                    serde_json::from_slice(&fs::read(&canonical).expect("retry receipt"))
                        .expect("retry receipt JSON");
                value["predecessorSummaryDigest"] = json!(Digest::sha256_bytes(b"stale-summary"));
                fs::write(
                    &canonical,
                    serde_json::to_vec_pretty(&value).expect("receipt JSON"),
                )
                .expect("tamper retry receipt");
            }
            _ => unreachable!(),
        }

        let outcome = outcome_v2_with_runtime(directory.path(), id, &runtime)
            .expect("invalid pending retry remains renderable");
        assert_eq!(outcome.state, OutcomeState::Unknown, "{case}");
        assert_eq!(
            outcome.decision_state,
            Some(cockpit_core::DecisionState::Red),
            "{case}"
        );
        assert!(
            outcome
                .unknowns
                .contains(&"recovery_decision_invalid".into()),
            "{case}: {:?}",
            outcome.unknowns
        );

        let contract_path = directory
            .path()
            .join(".ai/work-items/active/WI-BLOCKED.contract.json");
        let preflight =
            preflight_work_item_with_runtime(directory.path(), &contract_path, &runtime)
                .expect_err("preflight must not trust a pending marker alone");
        assert!(
            preflight.to_string().contains("recovery_decision_invalid"),
            "{case}: {preflight}"
        );

        let git = cockpit_git::GitRepository::discover(directory.path()).expect("git repository");
        let snapshot = git.snapshot().expect("snapshot");
        let run = run_repository_verification(
            directory.path(),
            &RepositoryVerificationRequest {
                node_id: "recovery-binding-negative-check".into(),
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
        .expect("verification run");
        let record = record_verification_with_runtime(
            directory.path(),
            id,
            &serde_json::to_value(&run.receipt).expect("receipt JSON"),
            &runtime,
            &snapshot,
        )
        .expect_err("verification must not trust a pending marker alone");
        assert!(
            record.to_string().contains("recovery_decision_invalid"),
            "{case}: {record}"
        );

        let finish = finish_work_item_with_runtime(directory.path(), id, &runtime)
            .expect_err("finish must not trust a pending marker alone");
        assert!(
            finish.to_string().contains("recovery_decision_invalid"),
            "{case}: {finish}"
        );
    }
}

#[test]
fn newer_retry_receipt_supersedes_stale_contract_binding_in_append_only_chain() {
    let directory = repository();
    let runtime = current_runtime();

    let mut first = receipt(&directory, "record the initial retry");
    first["decision"] = json!("retry");
    first.as_object_mut().unwrap().remove("successorWorkItemId");
    first["runtimeVersion"] = json!(runtime.runtime_version);
    first["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    first["decidedAt"] = json!("2026-08-23T00:06:00Z");
    record_recovery_decision(directory.path(), "WI-BLOCKED", &first, &runtime)
        .expect("initial retry receipt");

    let contract_path = directory
        .path()
        .join(".ai/work-items/active/WI-BLOCKED.contract.json");
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
    contract["title"] = json!("amended recovery");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .unwrap();

    let mut second = receipt(&directory, "record the amended retry");
    second["decision"] = json!("retry");
    second
        .as_object_mut()
        .unwrap()
        .remove("successorWorkItemId");
    second["runtimeVersion"] = json!(runtime.runtime_version);
    second["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    second["decidedAt"] = json!("2026-08-23T00:07:00Z");
    record_recovery_decision(directory.path(), "WI-BLOCKED", &second, &runtime)
        .expect("amended retry receipt");

    let outcome = outcome_v2_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
        .expect("newer valid recovery receipt should be selected");
    assert_eq!(
        outcome.recovery_decision.as_ref().unwrap().reason,
        "record the amended retry"
    );
}

#[test]
fn superseded_predecessor_preserves_bytes_and_closes_without_current_verification() {
    let directory = repository();
    let runtime = RuntimeContext {
        runtime_version: "0.2.13".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime-0.2.13"),
    };
    let mut first = receipt(&directory, "create an explicit successor");
    first["runtimeVersion"] = json!(runtime.runtime_version);
    first["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &first, &runtime)
        .expect("successor receipt");
    let successor_contract = directory
        .path()
        .join(".ai/work-items/active/WI-SUCCESSOR.contract.json");
    let mut successor =
        serde_json::from_slice::<serde_json::Value>(&fs::read(&successor_contract).unwrap())
            .unwrap();
    successor["predecessorWorkItemId"] = json!("WI-BLOCKED");
    successor["predecessorContractDigest"] = first["predecessorContractDigest"].clone();
    fs::write(
        &successor_contract,
        serde_json::to_vec_pretty(&successor).unwrap(),
    )
    .unwrap();
    start_work_item_with_options(
        directory.path(),
        "WI-SUCCESSOR",
        "continue on the successor",
        "continue the bounded successor work",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["successor remains independently governed".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("activate recovery successor scaffold");

    let predecessor_contract = fs::read(
        directory
            .path()
            .join(".ai/work-items/active/WI-BLOCKED.contract.json"),
    )
    .unwrap();
    let predecessor_summary = fs::read(
        directory
            .path()
            .join(".ai/work-items/active/WI-BLOCKED.summary.json"),
    )
    .unwrap();
    let predecessor_outcome = fs::read(
        directory
            .path()
            .join(".ai/work-items/active/WI-BLOCKED.outcome.json"),
    )
    .unwrap();
    let mut supersede = receipt(&directory, "supersede the historical predecessor");
    supersede["decision"] = json!("supersede");
    supersede["decidedAt"] = json!("2026-08-23T00:02:00Z");
    supersede["runtimeVersion"] = json!(runtime.runtime_version);
    supersede["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &supersede, &runtime)
        .expect("supersession receipt");

    let historical_variant = "WI-BLOCKED.outcome.finish-recovery.json";
    let historical_bytes = br#"{"state":"blocked","workItemId":"WI-BLOCKED"}"#;
    fs::write(
        directory
            .path()
            .join(".ai/work-items/active")
            .join(historical_variant),
        historical_bytes,
    )
    .expect("historical supersession variant");

    archive_work_item(directory.path(), "WI-BLOCKED").expect("superseded archive");
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(".ai/work-items/archive")
                .join(historical_variant),
        )
        .expect("archived historical supersession variant"),
        historical_bytes
    );
    let superseded_manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-BLOCKED.archive.json"),
        )
        .expect("superseded manifest"),
    )
    .expect("superseded manifest JSON");
    assert_eq!(
        superseded_manifest["historicalArtifacts"][0]["path"],
        format!(".ai/work-items/archive/{historical_variant}")
    );
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-BLOCKED.contract.json")
        )
        .unwrap(),
        predecessor_contract
    );
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-BLOCKED.summary.json")
        )
        .unwrap(),
        predecessor_summary
    );
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-BLOCKED.outcome.json")
        )
        .unwrap(),
        predecessor_outcome
    );

    // Preserve an older, digest-named successor attempt whose target was
    // never bound.  A later valid supersede must make this historical residue
    // non-blocking without rewriting or trusting the invalid receipt.
    let archive_root = directory.path().join(".ai/work-items/archive");
    let archived_contract: serde_json::Value =
        serde_json::from_slice(&fs::read(archive_root.join("WI-BLOCKED.contract.json")).unwrap())
            .unwrap();
    let archived_summary: serde_json::Value =
        serde_json::from_slice(&fs::read(archive_root.join("WI-BLOCKED.summary.json")).unwrap())
            .unwrap();
    let archived_outcome: serde_json::Value =
        serde_json::from_slice(&fs::read(archive_root.join("WI-BLOCKED.outcome.json")).unwrap())
            .unwrap();
    let archived_events = fs::read(archive_root.join("WI-BLOCKED.events.jsonl")).unwrap();
    let mut invalid_historical = json!({
        "schemaVersion": 1,
        "decisionId": "work-item-recovery",
        "decision": "successor",
        "workItemId": "WI-BLOCKED",
        "repositoryId": repository_id(directory.path()),
        "predecessorWorkItemId": "WI-BLOCKED",
        "predecessorContractDigest": cockpit_protocol::digest_json(&archived_contract).unwrap(),
        "predecessorSummaryDigest": cockpit_protocol::digest_json(&archived_summary).unwrap(),
        "predecessorOutcomeDigest": cockpit_protocol::digest_json(&archived_outcome).unwrap(),
        "predecessorEventsDigest": Digest::sha256_bytes(&archived_events),
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "actor": "human:owner",
        "authoritySource": "repository-owner",
        "reason": "historical unbound successor",
        "evidenceRefs": [".ai/work-items/archive/WI-BLOCKED.outcome.json"],
        "policyRefs": ["docs/reference/agent-workflow.md"],
        "decidedAt": "2026-08-23T00:01:00Z",
        "resumeCondition": "continue on the successor Work Item"
    });
    invalid_historical["successorWorkItemId"] = json!("WI-MISSING-HISTORICAL");
    let invalid_digest = cockpit_protocol::digest_json(&invalid_historical).unwrap();
    fs::write(
        directory.path().join(format!(
            ".ai/decisions/WI-BLOCKED.recovery.{}.json",
            invalid_digest.to_string().strip_prefix("sha256:").unwrap()
        )),
        serde_json::to_vec_pretty(&invalid_historical).unwrap(),
    )
    .expect("historical invalid recovery receipt");

    close_work_item_with_structured_decision(
        directory.path(),
        "WI-BLOCKED",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "repository-owner".into(),
            reason: "historical predecessor superseded without rewriting evidence".into(),
            evidence_refs: vec![".ai/decisions/WI-BLOCKED.recovery.json".into()],
            policy_refs: vec!["docs/reference/agent-workflow.md".into()],
            decided_at: "2026-08-23T00:03:00Z".into(),
            resume_condition: Some("continue on the successor Work Item".into()),
        },
    )
    .expect("close superseded predecessor");

    let outcome = outcome_v2(directory.path(), "WI-BLOCKED").expect("historical outcome");
    assert_eq!(outcome.historical_status.as_deref(), Some("superseded"));
    assert_eq!(
        outcome.decision_state,
        Some(cockpit_core::DecisionState::Yellow)
    );
    let handoff = render_human_outcome(directory.path(), &outcome, "zh");
    assert!(handoff.starts_with("Outcome: 🟡"), "{handoff}");
    assert!(!handoff.contains("失败 gate"), "{handoff}");
    assert!(!handoff.contains("修复失败的治理条件"), "{handoff}");

    let current_runtime_outcome =
        outcome_v2_with_runtime(directory.path(), "WI-BLOCKED", &current_runtime())
            .expect("older archived recovery remains historical under the current Runtime");
    assert_eq!(
        current_runtime_outcome.historical_status.as_deref(),
        Some("superseded")
    );
    assert!(
        !current_runtime_outcome
            .unknowns
            .contains(&"recovery_decision_invalid".into())
    );
}

#[test]
fn snapshot_digest_is_stable_when_the_same_source_change_is_committed() {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    fs::write(
        directory.path().join("src.rs"),
        "pub fn value() -> u8 { 1 }\n",
    )
    .expect("source");
    commit(directory.path(), "initial source");

    fs::write(
        directory.path().join("src.rs"),
        "pub fn value() -> u8 { 2 }\n",
    )
    .expect("updated source");
    let dirty = repository_snapshot_digest(directory.path());
    commit(directory.path(), "commit the same source change");
    let committed = repository_snapshot_digest(directory.path());

    assert_eq!(dirty, committed);
}

#[test]
fn archived_pending_finalization_requires_explicit_supersede_recovery_before_close() {
    let directory = ready_archived_repository();
    let id = "WI-ARCHIVED-RECOVERY";
    let runtime = RuntimeContext {
        runtime_version: "0.2.33".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"archived-recovery-runtime"),
    };

    let before = outcome_v2_with_runtime(directory.path(), id, &runtime).expect("outcome");
    assert_ne!(
        before.state,
        OutcomeState::Verified,
        "archived work with a resource context but no finalization receipt must not be green"
    );
    assert_ne!(
        before.decision_state,
        Some(cockpit_core::DecisionState::Green),
        "pending finalization must remain visibly non-green"
    );
    assert!(
        before
            .unknowns
            .contains(&"resource_finalization_pending".into())
    );
    assert!(
        render_human_outcome(directory.path(), &before, "zh").starts_with("Outcome: 🟡"),
        "pending finalization must be visible to a human"
    );

    let predecessor_contract = fs::read(
        directory
            .path()
            .join(format!(".ai/work-items/archive/{id}.contract.json")),
    )
    .unwrap();
    let successor =
        archived_recovery_receipt(&directory, "successor", Some("WI-ARCHIVED-NEXT"), &runtime);
    record_recovery_decision(directory.path(), id, &successor, &runtime)
        .expect("successor recovery decision");
    let mut wrong_binding =
        archived_recovery_receipt(&directory, "supersede", Some("WI-ARCHIVED-NEXT"), &runtime);
    wrong_binding["decidedAt"] = json!("2026-08-28T00:00:30Z");
    wrong_binding["predecessorArchiveManifestDigest"] =
        json!(Digest::sha256_bytes(b"wrong-manifest").to_string());
    let error = record_recovery_decision(directory.path(), id, &wrong_binding, &runtime)
        .expect_err("foreign manifest binding must fail closed");
    assert!(
        error
            .to_string()
            .contains("archive_manifest_digest_mismatch")
    );
    let mut supersede =
        archived_recovery_receipt(&directory, "supersede", Some("WI-ARCHIVED-NEXT"), &runtime);
    supersede["decidedAt"] = json!("2026-08-28T00:01:00Z");
    record_recovery_decision(directory.path(), id, &supersede, &runtime)
        .expect("supersede recovery decision");

    let close = close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        id,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "repository-owner".into(),
            reason: "close the immutable predecessor after explicit supersede recovery".into(),
            evidence_refs: vec![format!(".ai/decisions/{id}.recovery.json")],
            policy_refs: vec!["docs/reference/agent-workflow.md".into()],
            decided_at: "2026-08-28T00:01:00Z".into(),
            resume_condition: Some("continue on WI-ARCHIVED-NEXT".into()),
        },
        &runtime,
    );
    assert!(
        close.is_ok(),
        "valid supersede recovery should permit close: {close:?}"
    );
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(format!(".ai/work-items/archive/{id}.contract.json")),
        )
        .unwrap(),
        predecessor_contract,
        "recovery must not rewrite predecessor archive bytes"
    );
}

#[test]
fn supersede_quarantines_a_bound_optional_archive_report_mismatch() {
    let directory = ready_archived_repository();
    let id = "WI-ARCHIVED-RECOVERY";
    let runtime = RuntimeContext {
        runtime_version: "0.2.33".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"archived-recovery-runtime"),
    };
    let archive = directory.path().join(".ai/work-items/archive");
    let manifest_path = archive.join(format!("{id}.archive.json"));
    let manifest_digest = Digest::sha256_bytes(&fs::read(&manifest_path).unwrap());
    let markdown_path = archive.join(format!("{id}.task-report.md"));
    let mut markdown = fs::read(&markdown_path).expect("archived Task Outcome Markdown");
    markdown.extend_from_slice(b"\nHistorical recovery note.");
    fs::write(&markdown_path, markdown).expect("tamper only optional report bytes");

    let successor =
        archived_recovery_receipt(&directory, "successor", Some("WI-ARCHIVED-NEXT"), &runtime);
    record_recovery_decision(directory.path(), id, &successor, &runtime)
        .expect("successor recovery decision");
    let mut supersede =
        archived_recovery_receipt(&directory, "supersede", Some("WI-ARCHIVED-NEXT"), &runtime);
    supersede["decidedAt"] = json!("2026-08-28T00:01:00Z");
    supersede["predecessorArchiveManifestDigest"] = json!(manifest_digest.to_string());
    record_recovery_decision(directory.path(), id, &supersede, &runtime)
        .expect("manifest-bound supersede recovery decision");

    let close = close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        id,
        &HumanDecision {
            decision: "superseded".into(),
            actor: "human:owner".into(),
            authority_source: "repository-owner".into(),
            reason: "quarantine an immutable historical report mismatch without rewriting it"
                .into(),
            evidence_refs: vec![format!(".ai/decisions/{id}.recovery.json")],
            policy_refs: vec!["docs/reference/agent-workflow.md".into()],
            decided_at: "2026-08-28T00:01:00Z".into(),
            resume_condition: Some("continue on WI-ARCHIVED-NEXT".into()),
        },
        &runtime,
    )
    .expect("bound optional mismatch may be quarantined");
    assert_eq!(close.state, "closed");

    let decision: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/decisions/{id}.close.json")),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        decision["historicalArchiveIntegrity"]["state"],
        json!("quarantined")
    );
    assert_eq!(
        decision["historicalArchiveIntegrity"]["artifact"],
        json!("taskReportMarkdown")
    );

    let outcome = outcome_v2_with_runtime(directory.path(), id, &runtime).expect("outcome");
    assert_eq!(outcome.historical_status.as_deref(), Some("superseded"));
    assert_eq!(
        outcome.decision_state,
        Some(cockpit_core::DecisionState::Yellow)
    );
    assert!(render_human_outcome(directory.path(), &outcome, "en").starts_with("Outcome: 🟡"));
    assert!(!outcome.unknowns.contains(&"outcome_report_invalid".into()));
}

#[test]
fn archived_consumes_a_stale_retry_receipt_as_historical_evidence() {
    let directory = ready_archived_repository();
    let id = "WI-ARCHIVED-RECOVERY";
    let runtime = RuntimeContext {
        runtime_version: "0.2.33".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"archived-recovery-runtime"),
    };
    let mut retry = archived_recovery_receipt(&directory, "retry", None, &runtime);
    retry["predecessorSummaryDigest"] = json!(Digest::sha256_bytes(b"previous-summary"));
    fs::write(
        directory
            .path()
            .join(format!(".ai/decisions/{id}.recovery.json")),
        serde_json::to_vec_pretty(&retry).unwrap(),
    )
    .unwrap();

    let outcome = outcome_v2_with_runtime(directory.path(), id, &runtime)
        .expect("stale archived retry should remain a renderable historical outcome");
    assert!(
        !outcome
            .unknowns
            .contains(&"recovery_decision_invalid".into()),
        "{outcome:?}"
    );
    assert!(outcome.recovery_decision.is_none());
    assert_ne!(outcome.historical_status.as_deref(), Some("superseded"));
}

#[test]
fn invalid_archived_recovery_cannot_bypass_finalization_gate() {
    let directory = ready_archived_repository();
    let id = "WI-ARCHIVED-RECOVERY";
    let runtime = RuntimeContext {
        runtime_version: "0.2.33".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"archived-recovery-runtime"),
    };
    fs::write(
        directory
            .path()
            .join(format!(".ai/decisions/{id}.recovery.json")),
        b"{not-json",
    )
    .unwrap();
    let close = close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        id,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "repository-owner".into(),
            reason: "invalid recovery must not bypass finalization".into(),
            evidence_refs: Vec::new(),
            policy_refs: Vec::new(),
            decided_at: "2026-08-28T00:02:00Z".into(),
            resume_condition: Some("repair recovery evidence".into()),
        },
        &runtime,
    )
    .expect_err("malformed archived recovery must fail closed");
    assert!(close.to_string().contains("recovery_decision_invalid"));
}

#[test]
fn contract_amendment_accepts_legacy_checkpoint_without_typed_evidence() {
    let directory = repository();
    let contract_path = directory
        .path()
        .join(".ai/work-items/active/WI-BLOCKED.contract.json");
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
    contract["title"] = json!("amended after legacy checkpoint");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .unwrap();

    let amendment = revalidate_contract_amendment(
        directory.path(),
        "WI-BLOCKED",
        "bind recovery evidence scope after a legacy checkpoint",
    )
    .expect("legacy checkpoint should be upgraded during amendment");
    assert_eq!(amendment["stage"], "contract_amendment_revalidation");
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-BLOCKED.summary.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        summary["checkpointEvidence"]
            .as_array()
            .is_some_and(|entries| { entries.iter().any(|entry| entry["stage"] == "before_edit") })
    );
}

#[test]
fn supersede_requires_a_matching_existing_successor() {
    let directory = repository();
    let runtime = RuntimeContext {
        runtime_version: "0.2.13".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime-0.2.13"),
    };
    let mut missing = receipt(&directory, "supersede without successor");
    missing["decision"] = json!("supersede");
    missing["runtimeVersion"] = json!(runtime.runtime_version);
    missing["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    assert!(record_recovery_decision(directory.path(), "WI-BLOCKED", &missing, &runtime).is_err());

    let mut successor = receipt(&directory, "create successor");
    successor["runtimeVersion"] = json!(runtime.runtime_version);
    successor["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &successor, &runtime)
        .expect("successor receipt");
    let successor_path = directory
        .path()
        .join(".ai/work-items/active/WI-SUCCESSOR.contract.json");
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&successor_path).unwrap()).unwrap();
    contract["predecessorContractDigest"] = json!(Digest::sha256_bytes(b"foreign"));
    fs::write(
        &successor_path,
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .unwrap();
    let mut mismatch = receipt(&directory, "supersede mismatch");
    mismatch["decision"] = json!("supersede");
    mismatch["runtimeVersion"] = json!(runtime.runtime_version);
    mismatch["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    assert!(record_recovery_decision(directory.path(), "WI-BLOCKED", &mismatch, &runtime).is_err());
}

#[test]
fn strict_successor_rejects_a_forged_legacy_binding_marker() {
    let directory = repository();
    let runtime = current_runtime();
    let mut successor = receipt(&directory, "create successor");
    successor["runtimeVersion"] = json!(runtime.runtime_version);
    successor["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &successor, &runtime)
        .expect("successor receipt");

    let mut supersede = receipt(&directory, "forge legacy marker on strict successor");
    supersede["decision"] = json!("supersede");
    supersede["runtimeVersion"] = json!(runtime.runtime_version);
    supersede["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    supersede["successorBindingMode"] = json!("legacy_terminal_evidence");
    let error = record_recovery_decision(directory.path(), "WI-BLOCKED", &supersede, &runtime)
        .expect_err("strict successor must not claim legacy compatibility");
    assert!(
        error.to_string().contains("successor_binding_mode_invalid"),
        "unexpected error: {error}"
    );
}

#[test]
fn supersede_accepts_a_terminal_legacy_successor_bound_by_recovery_receipt() {
    let directory = ready_archived_repository();
    let id = "WI-ARCHIVED-RECOVERY";
    let runtime = RuntimeContext {
        runtime_version: "0.2.33".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"archived-recovery-runtime"),
    };
    let successor_id = "WI-ARCHIVED-NEXT";
    let successor_receipt =
        archived_recovery_receipt(&directory, "successor", Some(successor_id), &runtime);
    record_recovery_decision(directory.path(), id, &successor_receipt, &runtime)
        .expect("successor recovery receipt");

    start_work_item_with_options(
        directory.path(),
        successor_id,
        "continue the recovered change",
        "complete the recovered change with terminal evidence",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["the recovered successor is auditable".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start successor");

    // This models a successor created by an older Runtime which was validly
    // selected by the predecessor recovery receipt but never received the
    // newer Contract predecessor fields.  The archive manifest below is
    // generated from these exact legacy bytes, so this is not archive tamper.
    let successor_contract_path = directory.path().join(format!(
        ".ai/work-items/active/{successor_id}.contract.json"
    ));
    let mut successor_contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&successor_contract_path).unwrap()).unwrap();
    successor_contract
        .as_object_mut()
        .unwrap()
        .remove("predecessorWorkItemId");
    successor_contract
        .as_object_mut()
        .unwrap()
        .remove("predecessorContractDigest");
    successor_contract
        .as_object_mut()
        .unwrap()
        .remove("recoveryDecisionPath");
    successor_contract
        .as_object_mut()
        .unwrap()
        .remove("resourceContext");
    fs::write(
        &successor_contract_path,
        serde_json::to_vec_pretty(&successor_contract).unwrap(),
    )
    .unwrap();

    let successor_contract_path = directory.path().join(format!(
        ".ai/work-items/active/{successor_id}.contract.json"
    ));
    let contract_bytes = serde_json::to_vec_pretty(&successor_contract).unwrap();
    let summary = json!({
        "protocolVersion": 1,
        "workItemId": successor_id,
        "state": "finish_ready",
        "checkpointCount": 1,
        "preflightState": "green"
    });
    let outcome = json!({
        "protocolVersion": 1,
        "workItemId": successor_id,
        "state": "verified",
        "verification": {"status": "verified"}
    });
    let archive = directory.path().join(".ai/work-items/archive");
    fs::create_dir_all(&archive).unwrap();
    let archived_contract = archive.join(format!("{successor_id}.contract.json"));
    let archived_summary = archive.join(format!("{successor_id}.summary.json"));
    let archived_outcome = archive.join(format!("{successor_id}.outcome.json"));
    fs::write(&archived_contract, &contract_bytes).unwrap();
    let summary_bytes = serde_json::to_vec_pretty(&summary).unwrap();
    let outcome_bytes = serde_json::to_vec_pretty(&outcome).unwrap();
    fs::write(&archived_summary, &summary_bytes).unwrap();
    fs::write(&archived_outcome, &outcome_bytes).unwrap();
    let evidence = json!({
        "protocolVersion": 1,
        "evidenceSchemaVersion": 2,
        "workItemId": successor_id,
        "repositoryId": repository_id(directory.path()),
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "contractDigest": cockpit_protocol::digest_json(&successor_contract).unwrap(),
        "repositorySnapshotDigest": Digest::sha256_bytes(b"legacy-successor-snapshot"),
        "passed": true,
        "captureMode": "digest_only"
    });
    fs::write(
        directory
            .path()
            .join(format!(".ai/evidence/{successor_id}.verification.json")),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
    fs::remove_file(&successor_contract_path).unwrap();
    fs::remove_file(
        directory
            .path()
            .join(format!(".ai/work-items/active/{successor_id}.summary.json")),
    )
    .unwrap();
    let manifest = json!({
        "protocolVersion": 1,
        "workItemId": successor_id,
        "state": "archived",
        "closeRequired": true,
        "files": {
            "contractPath": format!(".ai/work-items/archive/{successor_id}.contract.json"),
            "contractDigest": Digest::sha256_bytes(&contract_bytes),
            "summaryPath": format!(".ai/work-items/archive/{successor_id}.summary.json"),
            "summaryDigest": Digest::sha256_bytes(&summary_bytes),
            "outcomePath": format!(".ai/work-items/archive/{successor_id}.outcome.json"),
            "outcomeDigest": Digest::sha256_bytes(&outcome_bytes)
        },
        "historicalArtifacts": [],
        "createdAt": "2026-08-28T00:04:00Z"
    });
    fs::write(
        archive.join(format!("{successor_id}.archive.json")),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let close = json!({
        "protocolVersion": 1,
        "workItemId": successor_id,
        "repositoryId": repository_id(directory.path()),
        "state": "closed",
        "decisionState": "confirmed",
        "humanDecision": "approved",
        "structuredDecision": {
            "decision": "approved",
            "actor": "human:owner",
            "authoritySource": "repository-owner",
            "reason": "terminal legacy successor evidence is complete",
            "evidenceRefs": [],
            "policyRefs": [],
            "decidedAt": "2026-08-28T00:04:00Z",
            "resumeCondition": "none"
        }
    });
    fs::write(
        directory
            .path()
            .join(format!(".ai/decisions/{successor_id}.close.json")),
        serde_json::to_vec_pretty(&close).unwrap(),
    )
    .unwrap();

    let evidence_path = directory
        .path()
        .join(format!(".ai/evidence/{successor_id}.verification.json"));
    let evidence_bytes = fs::read(&evidence_path).unwrap();
    let mut tampered_evidence: serde_json::Value = serde_json::from_slice(&evidence_bytes).unwrap();
    tampered_evidence["passed"] = json!(false);
    fs::write(
        &evidence_path,
        serde_json::to_vec_pretty(&tampered_evidence).unwrap(),
    )
    .unwrap();
    let mut tampered =
        archived_recovery_receipt(&directory, "supersede", Some(successor_id), &runtime);
    tampered["decidedAt"] = json!("2026-08-28T00:04:30Z");
    let error = record_recovery_decision(directory.path(), id, &tampered, &runtime)
        .expect_err("tampered legacy evidence must be rejected");
    assert!(
        error
            .to_string()
            .contains("legacy_successor_evidence_invalid"),
        "unexpected error: {error}"
    );
    fs::write(&evidence_path, &evidence_bytes).unwrap();

    let close_path = directory
        .path()
        .join(format!(".ai/decisions/{successor_id}.close.json"));
    let close_bytes = fs::read(&close_path).unwrap();
    fs::remove_file(&close_path).unwrap();
    let mut missing_close =
        archived_recovery_receipt(&directory, "supersede", Some(successor_id), &runtime);
    missing_close["decidedAt"] = json!("2026-08-28T00:04:45Z");
    let error = record_recovery_decision(directory.path(), id, &missing_close, &runtime)
        .expect_err("legacy successor without close evidence must be rejected");
    assert!(
        error
            .to_string()
            .contains("legacy_successor_evidence_missing"),
        "unexpected error: {error}"
    );
    fs::write(&close_path, &close_bytes).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let backup = evidence_path.with_extension("json.legacy-backup");
        fs::rename(&evidence_path, &backup).unwrap();
        symlink(&backup, &evidence_path).unwrap();
        let mut symlinked =
            archived_recovery_receipt(&directory, "supersede", Some(successor_id), &runtime);
        symlinked["decidedAt"] = json!("2026-08-28T00:04:55Z");
        let error = record_recovery_decision(directory.path(), id, &symlinked, &runtime)
            .expect_err("symlinked legacy evidence must be rejected");
        assert!(
            error
                .to_string()
                .contains("legacy_successor_evidence_missing"),
            "unexpected error: {error}"
        );
        fs::remove_file(&evidence_path).unwrap();
        fs::rename(backup, &evidence_path).unwrap();
    }

    let mut supersede =
        archived_recovery_receipt(&directory, "supersede", Some(successor_id), &runtime);
    supersede["decidedAt"] = json!("2026-08-28T00:05:00Z");
    let recorded = record_recovery_decision(directory.path(), id, &supersede, &runtime)
        .expect("legacy successor should be accepted only after terminal evidence");
    assert_eq!(
        recorded["successorBindingMode"],
        json!("legacy_terminal_evidence"),
        "legacy compatibility must be explicitly marked in the append-only recovery receipt"
    );

    close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        id,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "repository-owner".into(),
            reason: "close predecessor after its terminal successor".into(),
            evidence_refs: vec![format!(".ai/decisions/{id}.recovery.json")],
            policy_refs: Vec::new(),
            decided_at: "2026-08-28T00:06:00Z".into(),
            resume_condition: Some(format!("continue on {successor_id}")),
        },
        &runtime,
    )
    .expect("superseded predecessor should close");
}
