//! Native domain fixtures use an explicit private test port, never a production source.
#![cfg(test)]
use crate as cockpit_repository;
use crate::review_authority::pipeline_tests::TestAuthority;
use cockpit_core::DecisionState;
use cockpit_repository::attach;
use cockpit_repository::checkpoint_work_item;
use cockpit_repository::preflight_work_item;
use cockpit_repository::scaffold_work_item;
use std::fs;
use std::process::Command;
/// Automated check for collaboration-language semantic invariant 9 (see
/// docs/reference/collaboration-language-contract.md): every question that
/// requires a human decision must name the decision subject, its impact,
/// and its recovery/resume condition. A `HumanDecisionRequest` names the
/// subject via `what_happened`/`question`, the impact via `why_it_matters`
/// and each option's `effect`, and the recovery condition via
/// `resume_condition`. This asserts none of those fields, nor any offered
/// option, is silently empty.
fn assert_human_decision_request_is_complete(request: &cockpit_core::HumanDecisionRequest) {
    assert!(!request.what_happened.is_empty(), "what_happened is empty");
    assert!(
        !request.why_it_matters.is_empty(),
        "why_it_matters is empty"
    );
    assert!(!request.question.is_empty(), "question is empty");
    assert!(
        !request.resume_condition.is_empty(),
        "resume_condition is empty"
    );
    assert!(!request.options.is_empty(), "options is empty");
    assert!(
        !request.recommended_option.is_empty(),
        "recommended_option is empty"
    );
    assert!(
        !request.recommendation_reason.is_empty(),
        "recommendation_reason is empty"
    );
    assert!(
        request
            .options
            .iter()
            .any(|option| option.id == request.recommended_option),
        "recommended_option {:?} does not name one of the offered options",
        request.recommended_option
    );
    for option in &request.options {
        assert!(!option.id.is_empty(), "option id is empty");
        assert!(
            !option.label.is_empty(),
            "option {:?} label is empty",
            option.id
        );
        assert!(
            !option.effect.is_empty(),
            "option {:?} effect is empty",
            option.id
        );
    }
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
    directory
}

#[test]
fn scaffold_preflight_is_not_ready_and_records_human_review_requirements() {
    let _authority = TestAuthority::install();
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
    let request = decision
        .human_decision_request
        .as_ref()
        .expect("structured human decision request");
    assert_eq!(request.status, "Missing");
    assert!(!request.question.is_empty());
    assert!(!request.resume_condition.is_empty());
    assert_human_decision_request_is_complete(request);
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
