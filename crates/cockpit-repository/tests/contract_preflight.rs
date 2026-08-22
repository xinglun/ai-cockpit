use cockpit_core::DecisionState;
use cockpit_repository::{attach, checkpoint_work_item, preflight_work_item, scaffold_work_item};
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
    attach(directory.path()).expect("attach");
    directory
}

#[test]
fn scaffold_preflight_is_not_ready_and_records_human_review_requirements() {
    let directory = repository();
    let receipt =
        scaffold_work_item(directory.path(), "WI-CONTRACT-SCAFFOLD", "code").expect("scaffold");
    let decision = preflight_work_item(
        directory.path(),
        &directory.path().join(&receipt.contract_path),
    )
    .expect("preflight scaffold");

    assert_eq!(decision.state, DecisionState::Yellow);
    assert_eq!(
        decision.review_state.as_deref(),
        Some("needs_human_confirmation")
    );
    for unknown in [
        "contract_intent_missing",
        "contract_scope_missing",
        "contract_acceptance_missing",
        "human_authority_missing",
    ] {
        assert!(
            decision.unknowns.iter().any(|item| item == unknown),
            "missing {unknown}"
        );
    }
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-CONTRACT-SCAFFOLD.summary.json"),
        )
        .expect("summary"),
    )
    .expect("summary json");
    assert_eq!(summary["preflightState"], "yellow");
    assert!(summary["preflightDecisionDigest"].is_string());
    assert!(checkpoint_work_item(directory.path(), "WI-CONTRACT-SCAFFOLD").is_err());
}
