use std::collections::BTreeMap;

use cockpit_release::{
    acceptance::{
        AcceptancePath, AcceptanceScope, ArtifactIdentity, EvidenceReference, IsolationRootPolicy,
        PhaseBoundaries, PhaseResult, PhaseStatus, ReleaseIdentity, ReleasePhase, RuntimeIdentity,
        SourceIdentity,
    },
    recovery::{
        BlockReason, FailureKind, PhaseAction, PhaseFailure, RecoveryPlan, RetryStrategy,
        ReuseDecision, evaluate_reuse,
    },
    resume::{PhaseReceiptStore, plan_for_phase},
};

fn identity(seed: &str) -> ReleaseIdentity {
    let mut assets = BTreeMap::new();
    assets.insert(
        format!("ai-cockpit-{seed}-x86_64.tar.gz"),
        format!("sha256:{seed:0<64}"),
    );
    ReleaseIdentity {
        source: SourceIdentity {
            repository: "sha256:repository".into(),
            commit: format!("{seed:0<40}"),
            cargo_lock_digest: format!("sha256:{seed:0<64}"),
        },
        candidate: ArtifactIdentity {
            version: format!("0.2.{seed}"),
            tag: format!("v0.2.{seed}"),
            manifest_digest: format!("sha256:{seed:0<64}"),
            assets,
        },
        previous: Some(ArtifactIdentity {
            version: "0.2.90".into(),
            tag: "v0.2.90".into(),
            manifest_digest: "sha256:previous".into(),
            assets: BTreeMap::new(),
        }),
        runtime: RuntimeIdentity {
            version: "0.2.87".into(),
            digest: "sha256:runtime".into(),
        },
        target: "x86_64-unknown-linux-gnu".into(),
        isolation: IsolationRootPolicy {
            home: "/tmp/acceptance/home".into(),
            xdg_config_home: "/tmp/acceptance/xdg".into(),
            tmp: "/tmp/acceptance/tmp".into(),
            cargo_home: "/tmp/acceptance/cargo".into(),
        },
    }
}

fn boundaries() -> PhaseBoundaries {
    PhaseBoundaries {
        started_at: "2026-09-11T00:00:00Z".into(),
        finished_at: Some("2026-09-11T00:00:01Z".into()),
        elapsed_ms: Some(1_000),
    }
}

fn evidence(phase: ReleasePhase) -> EvidenceReference {
    EvidenceReference {
        id: format!("receipt-{}", phase.as_str()),
        path: format!(".ai/release/{}.json", phase.as_str()),
        digest: format!("sha256:{}", phase.as_str()),
    }
}

fn succeeded(identity: ReleaseIdentity, phase: ReleasePhase) -> PhaseResult {
    PhaseResult::succeeded(phase, identity, 1, boundaries(), evidence(phase))
}

#[test]
fn all_release_phases_are_explicit_and_ordered() {
    assert_eq!(
        ReleasePhase::all(),
        &[
            ReleasePhase::Prepare,
            ReleasePhase::SourceVerificationBuild,
            ReleasePhase::CandidateAcceptance,
            ReleasePhase::Publish,
            ReleasePhase::PublicAcceptance,
            ReleasePhase::Close,
        ]
    );
    assert_eq!(
        ReleasePhase::Close.prerequisite(),
        Some(ReleasePhase::PublicAcceptance)
    );
    assert_eq!(
        ReleasePhase::CandidateAcceptance.parallel_acceptance_paths(),
        &[
            AcceptancePath::CandidateFreshInstall,
            AcceptancePath::CandidateNMinusOneUpgrade,
        ]
    );
    assert_eq!(
        ReleasePhase::PublicAcceptance.parallel_acceptance_paths(),
        &[
            AcceptancePath::PublicFreshInstall,
            AcceptancePath::PublicNMinusOneUpgrade,
            AcceptancePath::PublicVersionConsistency,
        ]
    );
    assert!(
        AcceptancePath::all()
            .iter()
            .all(|path| path.phase() != ReleasePhase::Close)
    );
}

#[test]
fn identity_digest_is_deterministic_and_changes_with_inputs() {
    let first = identity("91");
    let second = identity("92");
    let mut relocated = first.clone();
    relocated.isolation.home = "/runner/work/relocated/home".into();
    relocated.isolation.xdg_config_home = "/runner/work/relocated/xdg".into();
    relocated.isolation.tmp = "/runner/work/relocated/tmp".into();
    relocated.isolation.cargo_home = "/runner/work/relocated/cargo".into();
    assert_eq!(first.digest(), first.digest());
    assert_eq!(first.digest(), relocated.digest());
    assert_ne!(first.digest(), second.digest());
    assert!(first.digest().starts_with("sha256:"));
}

#[test]
fn an_unchanged_six_phase_run_reuses_every_result() {
    let expected = identity("91");
    let results = ReleasePhase::all()
        .iter()
        .copied()
        .map(|phase| succeeded(expected.clone(), phase))
        .collect::<Vec<_>>();

    let plan = RecoveryPlan::from_results(&expected, &results).unwrap();

    assert!(plan.first_execution().is_none());
    assert!(
        plan.actions
            .iter()
            .all(|action| matches!(action, PhaseAction::Reuse { .. }))
    );
}

#[test]
fn candidate_timeout_retries_only_candidate_and_blocks_dependents() {
    let expected = identity("91");
    let mut results = ReleasePhase::all()
        .iter()
        .copied()
        .take(2)
        .map(|phase| succeeded(expected.clone(), phase))
        .collect::<Vec<_>>();
    results.push(PhaseResult::failed(
        ReleasePhase::CandidateAcceptance,
        expected.clone(),
        1,
        boundaries(),
        PhaseFailure::new(
            FailureKind::Timeout,
            "candidate_timeout",
            "runner timed out",
        ),
    ));

    let plan = RecoveryPlan::from_results(&expected, &results).unwrap();

    assert!(matches!(
        plan.actions[2],
        PhaseAction::Retry {
            phase: ReleasePhase::CandidateAcceptance,
            strategy: RetryStrategy::RetryCurrentPhase,
        }
    ));
    assert!(plan.actions[3..].iter().all(|action| matches!(
        action,
        PhaseAction::Blocked {
            reason: BlockReason::FailedPrerequisite {
                phase: ReleasePhase::CandidateAcceptance
            },
            ..
        }
    )));
}

#[test]
fn input_change_invalidates_from_the_first_phase_and_never_reuses_publish() {
    let old = identity("91");
    let expected = identity("92");
    let old_results = ReleasePhase::all()
        .iter()
        .copied()
        .map(|phase| succeeded(old.clone(), phase))
        .collect::<Vec<_>>();

    let plan = RecoveryPlan::from_results(&expected, &old_results).unwrap();

    assert!(matches!(
        plan.actions[0],
        PhaseAction::Retry {
            phase: ReleasePhase::Prepare,
            strategy: RetryStrategy::RestartFromPhase,
        }
    ));
    assert!(plan.actions[3..].iter().all(|action| !matches!(
        action,
        PhaseAction::Reuse {
            phase: ReleasePhase::Publish,
            ..
        }
    )));
}

#[test]
fn scope_change_requires_a_successor_instead_of_a_technical_retry() {
    let expected = identity("91");
    let result = PhaseResult::failed(
        ReleasePhase::SourceVerificationBuild,
        expected.clone(),
        1,
        boundaries(),
        PhaseFailure::new(
            FailureKind::ScopeChanged,
            "scope_changed",
            "contract changed",
        ),
    );

    assert!(matches!(
        evaluate_reuse(&result, &expected),
        ReuseDecision::Blocked {
            reason: BlockReason::SuccessorRequired,
        }
    ));
}

#[test]
fn cleanup_pending_close_is_retried_without_repeating_publish() {
    let expected = identity("91");
    let mut results = ReleasePhase::all()
        .iter()
        .copied()
        .map(|phase| succeeded(expected.clone(), phase))
        .collect::<Vec<_>>();
    results[5].cleanup = cockpit_release::acceptance::CleanupState::Pending;

    let plan = RecoveryPlan::from_results(&expected, &results).unwrap();

    assert!(matches!(
        plan.actions[3],
        PhaseAction::Reuse {
            phase: ReleasePhase::Publish,
            ..
        }
    ));
    assert!(matches!(
        plan.actions[5],
        PhaseAction::Retry {
            phase: ReleasePhase::Close,
            strategy: RetryStrategy::RetryCleanupOnly,
        }
    ));
}

#[test]
fn cleanup_failure_retries_cleanup_only() {
    let expected = identity("91");
    let result = PhaseResult::failed(
        ReleasePhase::Close,
        expected.clone(),
        1,
        boundaries(),
        PhaseFailure::new(
            FailureKind::Cleanup,
            "cleanup_failed",
            "temporary root remains",
        ),
    );

    assert!(matches!(
        evaluate_reuse(&result, &expected),
        ReuseDecision::Retry {
            strategy: RetryStrategy::RetryCleanupOnly,
            ..
        }
    ));
}

#[test]
fn an_already_published_identity_conflict_blocks_public_acceptance() {
    let expected = identity("91");
    let mut results = ReleasePhase::all()
        .iter()
        .copied()
        .take(3)
        .map(|phase| succeeded(expected.clone(), phase))
        .collect::<Vec<_>>();
    results.push(PhaseResult::failed(
        ReleasePhase::Publish,
        expected.clone(),
        1,
        boundaries(),
        PhaseFailure::new(
            FailureKind::AlreadyPublished,
            "release_identity_conflict",
            "provider release has a different manifest",
        ),
    ));

    let plan = RecoveryPlan::from_results(&expected, &results).unwrap();

    assert!(matches!(
        plan.actions[3],
        PhaseAction::Blocked {
            phase: ReleasePhase::Publish,
            reason: BlockReason::AlreadyPublishedConflict,
        }
    ));
    assert!(matches!(
        plan.actions[4],
        PhaseAction::Blocked {
            phase: ReleasePhase::PublicAcceptance,
            reason: BlockReason::FailedPrerequisite {
                phase: ReleasePhase::Publish
            },
        }
    ));
}

#[test]
fn duplicate_phase_receipts_are_rejected_instead_of_picked_arbitrarily() {
    let expected = identity("91");
    let results = vec![
        succeeded(expected.clone(), ReleasePhase::Prepare),
        succeeded(expected, ReleasePhase::Prepare),
    ];

    assert!(RecoveryPlan::from_results(&identity("91"), &results).is_err());
}

#[test]
fn receipt_store_reuses_prepare_build_and_publish_after_interruption() {
    let expected = identity("91");
    let temp = tempfile::tempdir().unwrap();
    let evidence = temp.path().join("evidence.json");
    std::fs::write(
        &evidence,
        br#"{"identity":"release-91"}
"#,
    )
    .unwrap();
    let receipts = temp.path().join("phase-receipts.json");

    let mut store = PhaseReceiptStore::empty(expected.clone());
    store
        .record_success(ReleasePhase::Prepare, 1, "prepare", &evidence)
        .unwrap();
    store
        .record_success(ReleasePhase::SourceVerificationBuild, 1, "build", &evidence)
        .unwrap();
    store
        .record_success(ReleasePhase::CandidateAcceptance, 1, "candidate", &evidence)
        .unwrap();
    store
        .record_success(ReleasePhase::Publish, 1, "publish", &evidence)
        .unwrap();
    store.write_atomic(&receipts).unwrap();

    let resumed = PhaseReceiptStore::load(&receipts, &expected).unwrap();
    let plan = resumed.plan().unwrap();
    assert!(matches!(
        plan_for_phase(&plan, ReleasePhase::Prepare),
        Some(PhaseAction::Reuse { .. })
    ));
    assert!(matches!(
        plan_for_phase(&plan, ReleasePhase::SourceVerificationBuild),
        Some(PhaseAction::Reuse { .. })
    ));
    assert!(matches!(
        plan_for_phase(&plan, ReleasePhase::Publish),
        Some(PhaseAction::Reuse { .. })
    ));
}

#[test]
fn failed_phase_is_recorded_and_success_replaces_only_the_latest_result() {
    let expected = identity("91");
    let temp = tempfile::tempdir().unwrap();
    let evidence = temp.path().join("evidence.json");
    std::fs::write(&evidence, b"recovered evidence").unwrap();

    let mut store = PhaseReceiptStore::empty(expected.clone());
    store
        .record_success(ReleasePhase::Prepare, 1, "prepare", &evidence)
        .unwrap();
    store
        .record_success(ReleasePhase::SourceVerificationBuild, 1, "build", &evidence)
        .unwrap();
    store
        .record_failure(
            ReleasePhase::CandidateAcceptance,
            1,
            PhaseFailure::new(
                FailureKind::Timeout,
                "candidate_timeout",
                "runner timed out",
            ),
        )
        .unwrap();
    assert!(matches!(
        plan_for_phase(&store.plan().unwrap(), ReleasePhase::CandidateAcceptance),
        Some(PhaseAction::Retry { .. })
    ));

    assert!(matches!(
        store.record_success(
            ReleasePhase::CandidateAcceptance,
            1,
            "candidate-repeated",
            &evidence,
        ),
        Err(cockpit_release::resume::ReceiptError::AttemptNotMonotonic)
    ));

    store
        .record_success(
            ReleasePhase::CandidateAcceptance,
            2,
            "candidate-recovered",
            &evidence,
        )
        .unwrap();
    assert_eq!(store.results.len(), 3);
    assert_eq!(store.history.len(), 1);
    assert_eq!(
        store
            .results
            .iter()
            .find(|result| result.phase == ReleasePhase::CandidateAcceptance)
            .unwrap()
            .status,
        PhaseStatus::Succeeded
    );
    assert_eq!(store.history[0].status, PhaseStatus::TimedOut);
    assert!(matches!(
        plan_for_phase(&store.plan().unwrap(), ReleasePhase::CandidateAcceptance),
        Some(PhaseAction::Reuse { .. })
    ));
}

#[test]
fn receipt_evidence_can_be_revalidated_after_runner_artifact_relocation() {
    let expected = identity("91");
    let old = tempfile::tempdir().unwrap();
    let new = tempfile::tempdir().unwrap();
    let old_evidence = old.path().join("isolation.json");
    let old_receipts = old.path().join("phase-receipts.json");
    std::fs::write(&old_evidence, b"portable evidence").unwrap();

    let mut store = PhaseReceiptStore::empty(expected.clone());
    store
        .record_success(ReleasePhase::Prepare, 1, "prepare", &old_evidence)
        .unwrap();
    store.write_atomic(&old_receipts).unwrap();
    let new_receipts = new.path().join("phase-receipts.json");
    std::fs::copy(&old_receipts, &new_receipts).unwrap();
    std::fs::copy(&old_evidence, new.path().join("isolation.json")).unwrap();
    // The old runner path may still exist after artifact relocation.  It must
    // not win over the restored evidence merely because it is absolute.
    std::fs::write(&old_evidence, b"unrelated old runner evidence").unwrap();

    let mut relocated = PhaseReceiptStore::load(&new_receipts, &expected).unwrap();
    relocated
        .record_success(
            ReleasePhase::Prepare,
            2,
            "prepare",
            &new.path().join("isolation.json"),
        )
        .unwrap();
    assert!(relocated.history.is_empty());
    assert_eq!(relocated.results.len(), 1);

    let next_evidence = new.path().join("source-verification.json");
    std::fs::write(&next_evidence, b"next phase evidence").unwrap();
    relocated
        .record_success(
            ReleasePhase::SourceVerificationBuild,
            2,
            "source-verification-build",
            &next_evidence,
        )
        .unwrap();
    relocated.write_atomic(&new_receipts).unwrap();
    let reloaded = PhaseReceiptStore::load(&new_receipts, &expected).unwrap();
    assert_eq!(reloaded.results.len(), 2);
}

#[test]
fn receipt_store_rejects_changed_artifact_identity_before_planning() {
    let expected = identity("91");
    let changed = identity("92");
    let temp = tempfile::tempdir().unwrap();
    let evidence = temp.path().join("evidence.json");
    std::fs::write(&evidence, b"release evidence").unwrap();
    let receipts = temp.path().join("phase-receipts.json");

    let mut store = PhaseReceiptStore::empty(expected.clone());
    store
        .record_success(ReleasePhase::Publish, 1, "publish", &evidence)
        .unwrap();
    store.write_atomic(&receipts).unwrap();

    assert!(matches!(
        PhaseReceiptStore::load(&receipts, &changed),
        Err(cockpit_release::resume::ReceiptError::IdentityMismatch)
    ));
}

#[test]
fn receipt_store_rejects_changed_evidence_before_planning() {
    let expected = identity("91");
    let temp = tempfile::tempdir().unwrap();
    let evidence = temp.path().join("evidence.json");
    std::fs::write(&evidence, b"release evidence").unwrap();
    let receipts = temp.path().join("phase-receipts.json");

    let mut store = PhaseReceiptStore::empty(expected.clone());
    store
        .record_success(ReleasePhase::CandidateAcceptance, 1, "candidate", &evidence)
        .unwrap();
    store.write_atomic(&receipts).unwrap();
    std::fs::write(&evidence, b"changed evidence").unwrap();

    assert!(matches!(
        PhaseReceiptStore::load(&receipts, &expected),
        Err(cockpit_release::resume::ReceiptError::EvidenceDigestMismatch)
    ));
}

#[test]
fn candidate_scope_reuses_candidate_and_can_continue_to_close_without_publish() {
    let expected = identity("91");
    let results = vec![
        succeeded(expected.clone(), ReleasePhase::Prepare),
        succeeded(expected.clone(), ReleasePhase::SourceVerificationBuild),
        succeeded(expected.clone(), ReleasePhase::CandidateAcceptance),
    ];

    let plan =
        RecoveryPlan::from_results_for_scope(&expected, &results, AcceptanceScope::Candidate)
            .unwrap();

    assert!(
        plan.actions
            .iter()
            .take(3)
            .all(|action| matches!(action, PhaseAction::Reuse { .. }))
    );
    assert!(matches!(
        plan_for_phase(&plan, ReleasePhase::Close),
        Some(PhaseAction::Run {
            phase: ReleasePhase::Close
        })
    ));
}

#[test]
fn public_scope_reuses_publish_and_public_acceptance_before_close() {
    let expected = identity("91");
    let results = vec![
        succeeded(expected.clone(), ReleasePhase::Prepare),
        succeeded(expected.clone(), ReleasePhase::SourceVerificationBuild),
        succeeded(expected.clone(), ReleasePhase::Publish),
        succeeded(expected.clone(), ReleasePhase::PublicAcceptance),
    ];

    let plan =
        RecoveryPlan::from_results_for_scope(&expected, &results, AcceptanceScope::Public).unwrap();

    for phase in [
        ReleasePhase::Prepare,
        ReleasePhase::SourceVerificationBuild,
        ReleasePhase::Publish,
        ReleasePhase::PublicAcceptance,
    ] {
        assert!(matches!(
            plan_for_phase(&plan, phase),
            Some(PhaseAction::Reuse { .. })
        ));
    }
    assert!(matches!(
        plan_for_phase(&plan, ReleasePhase::Close),
        Some(PhaseAction::Run {
            phase: ReleasePhase::Close
        })
    ));
}

#[test]
fn injected_publish_failure_reuses_prior_phases_and_retries_only_publish() {
    let expected = identity("91");
    let mut results = vec![
        succeeded(expected.clone(), ReleasePhase::Prepare),
        succeeded(expected.clone(), ReleasePhase::SourceVerificationBuild),
        succeeded(expected.clone(), ReleasePhase::CandidateAcceptance),
    ];
    results.push(PhaseResult::failed(
        ReleasePhase::Publish,
        expected.clone(),
        1,
        boundaries(),
        PhaseFailure::new(
            FailureKind::Network,
            "injected_publish_failure",
            "failure injection stopped publication after prior receipts were persisted",
        ),
    ));

    let plan = RecoveryPlan::from_results(&expected, &results).unwrap();
    assert!(matches!(
        plan_for_phase(&plan, ReleasePhase::Prepare),
        Some(PhaseAction::Reuse { .. })
    ));
    assert!(matches!(
        plan_for_phase(&plan, ReleasePhase::SourceVerificationBuild),
        Some(PhaseAction::Reuse { .. })
    ));
    assert!(matches!(
        plan_for_phase(&plan, ReleasePhase::Publish),
        Some(PhaseAction::Retry {
            strategy: RetryStrategy::RetryCurrentPhase,
            ..
        })
    ));
    assert!(plan.actions.iter().any(|action| matches!(
        action,
        PhaseAction::Blocked {
            reason: BlockReason::FailedPrerequisite {
                phase: ReleasePhase::Publish
            },
            ..
        }
    )));
}
