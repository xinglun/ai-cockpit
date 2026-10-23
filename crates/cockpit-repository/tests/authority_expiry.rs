//! Supplied-fact assessment and test-owned simulated effects only; no live authority.

use cockpit_core::{
    Digest,
    clock::{Clock, ClockError, ClockSample, FiniteWindow, MonotonicBudget},
};
use cockpit_protocol::{
    GoalAuthorityDeadline, GoalAuthorityReference, GoalAuthoritySource, GoalAuthorityTermination,
    GoalAuthorityTerminationKind,
};
use cockpit_repository::authority::*;
use serde_json::json;
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
};

const BASE: i64 = 1_893_456_000_000;

fn source() -> GoalAuthoritySource {
    serde_json::from_value(json!({
        "schemaVersion":1,
        "repositoryId":Digest::sha256_bytes(b"p2a-offline-fixture").to_string(),
        "grantId":"grant-a", "subjectRef":"goal-a", "sourceVersion":1,
        "goal":"test-owned facts only", "deliverableRefs":["part-a"],
        "actors":["agent:fixture"], "operations":["source.modify"],
        "resources":["repo:src/example.rs"], "constraints":[],
        "issuerEvidence":{"assurance":"self_declared","actor":"human:fixture",
            "authoritySource":"supplied fixture only","operations":["source.modify"],
            "policyRefs":[],"evidenceRefs":[]},
        "validity":{"notBefore":"2030-01-01T00:00:00Z","expiresAt":"2030-01-01T00:00:10Z"}
    }))
    .unwrap()
}

fn reference() -> GoalAuthorityReference {
    serde_json::from_value(json!({"grantId":"grant-a","subjectRef":"goal-a",
        "versionResolution":"current","deliverableRefs":["part-a"],
        "deadline":{"mode":"inherit_source"}}))
    .unwrap()
}

fn key() -> GoalAuthoritySubjectKey {
    let source = source();
    GoalAuthoritySubjectKey {
        repository_id: source.repository_id,
        grant_id: source.grant_id,
        subject_ref: source.subject_ref,
    }
}

fn facts() -> ResolvedGoalAuthorityFacts {
    let source = source();
    let digest = normalize_goal_authority_source(&source)
        .unwrap()
        .source_digest;
    ResolvedGoalAuthorityFacts {
        current_head: GoalAuthorityHeadObservation {
            observation: SuppliedFactObservation::Observed,
            source_version: 1,
            source_digest: digest,
        },
        version_history: vec![source.clone()],
        source,
        issuer_observation: SuppliedFactObservation::Observed,
        actor_observation: SuppliedFactObservation::Observed,
        subject_state: SuppliedGoalAuthorityState::Active,
        parent_chain_observation: SuppliedFactObservation::Observed,
        parent_chain: vec![],
    }
}

fn action() -> GoalAuthorityActionFacts {
    GoalAuthorityActionFacts {
        subject: key(),
        actor: "agent:fixture".into(),
        operation: "source.modify".into(),
        resource: "repo:src/example.rs".into(),
        deliverable_refs: vec!["part-a".into()],
        work_item: GoalAuthorityNarrowing {
            work_item_id: "WI-fixture".into(),
            parent_work_item_id: None,
            state: SuppliedGoalAuthorityState::Active,
            actors: vec!["agent:fixture".into()],
            operations: vec!["source.modify".into()],
            resources: vec!["repo:src/example.rs".into()],
            constraints: vec![],
        },
    }
}

fn sample(offset: i64, monotonic: u64) -> ClockSample {
    ClockSample {
        utc_millis: BASE + offset,
        monotonic_millis: monotonic,
    }
}

#[test]
fn offline_exact_goal_and_deliverable_coverage() {
    let facts = facts();
    let reference = reference();
    assert_eq!(
        assess_goal_authority(&facts, &reference, &action(), sample(0, 0)),
        Ok(GoalAuthorityAssessment::EligibleUnderSuppliedFacts {
            source_digest: normalize_goal_authority_source(&facts.source)
                .unwrap()
                .source_digest,
            reference_digest: normalize_goal_authority_reference(&reference)
                .unwrap()
                .reference_digest,
            effective_window: cockpit_core::clock::FiniteWindow::new(BASE, BASE + 10_000).unwrap(),
        })
    );
}

#[test]
fn public_default_resolver_is_unsupported() {
    assert_eq!(
        UnsupportedCurrentGoalAuthorityResolver.resolve_current(&key()),
        Err(GoalAuthorityResolutionError::Unsupported)
    );
}

fn instant(offset: i64) -> String {
    chrono::DateTime::from_timestamp_millis(BASE + offset)
        .unwrap()
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn reseal(facts: &mut ResolvedGoalAuthorityFacts) {
    facts.current_head.source_version = facts.source.source_version;
    facts.current_head.source_digest = normalize_goal_authority_source(&facts.source)
        .unwrap()
        .source_digest;
    if let Some(last) = facts.version_history.last_mut() {
        assert_eq!(last.source_version, facts.source.source_version);
        *last = facts.source.clone();
    }
}

fn extend_version(facts: &mut ResolvedGoalAuthorityFacts, end: i64) {
    let previous = normalize_goal_authority_source(&facts.source)
        .unwrap()
        .source_digest;
    facts.source.source_version += 1;
    facts.source.predecessor_version_digest = Some(previous);
    facts.source.validity.expires_at = instant(end);
    facts.version_history.push(facts.source.clone());
    reseal(facts);
}

fn window(result: GoalAuthorityAssessment) -> FiniteWindow {
    match result {
        GoalAuthorityAssessment::EligibleUnderSuppliedFacts {
            effective_window, ..
        } => effective_window,
    }
}

fn parent(id: &str, parent_id: Option<&str>, start: i64, end: i64) -> GoalAuthorityParentFacts {
    let mut work_item = action().work_item;
    work_item.work_item_id = id.into();
    work_item.parent_work_item_id = parent_id.map(str::to_owned);
    let mut reference = reference();
    reference.not_before = Some(instant(start));
    reference.deadline = GoalAuthorityDeadline::CapAt {
        expires_at: instant(end),
    };
    GoalAuthorityParentFacts {
        subject: key(),
        work_item,
        reference,
    }
}

#[test]
fn exact_subject_and_deliverable_identity_prevents_goal_reuse_or_grant_stitching() {
    for field in 0..3 {
        let mut action = action();
        match field {
            0 => {
                action.subject.repository_id = Digest::sha256_bytes(b"other-repository").to_string()
            }
            1 => action.subject.grant_id = "grant-b".into(),
            _ => action.subject.subject_ref = "goal-b".into(),
        }
        assert!(matches!(
            assess_goal_authority(&facts(), &reference(), &action, sample(0, 0)),
            Err(GoalAuthorityAssessmentError::IdentityMismatch(_))
        ));
    }
    let mut expanded = reference();
    expanded.deliverable_refs.push("part-b".into());
    assert_eq!(
        assess_goal_authority(&facts(), &expanded, &action(), sample(0, 0)),
        Err(GoalAuthorityAssessmentError::ScopeWidening("deliverables"))
    );
    let mut wrong = reference();
    wrong.subject_ref = "goal-b".into();
    assert!(matches!(
        assess_goal_authority(&facts(), &wrong, &action(), sample(0, 0)),
        Err(GoalAuthorityAssessmentError::IdentityMismatch(_))
    ));
    let mut stitched = facts();
    let mut foreign = parent("foreign", None, 0, 9000);
    foreign.subject.grant_id = "grant-b".into();
    foreign.reference.grant_id = "grant-b".into();
    stitched.parent_chain.push(foreign);
    let mut child = action();
    child.work_item.parent_work_item_id = Some("foreign".into());
    assert_eq!(
        assess_goal_authority(&stitched, &reference(), &child, sample(1, 1)),
        Err(GoalAuthorityAssessmentError::IdentityMismatch(
            "parent subject"
        ))
    );
}

#[test]
fn required_observations_reject_unknown_or_conflicting_claims_even_with_plausible_strings() {
    for observation in [
        SuppliedFactObservation::Unknown,
        SuppliedFactObservation::Conflicting,
    ] {
        for field in 0..4 {
            let mut facts = facts();
            let label = match field {
                0 => {
                    facts.issuer_observation = observation;
                    "issuer"
                }
                1 => {
                    facts.actor_observation = observation;
                    "actor"
                }
                2 => {
                    facts.current_head.observation = observation;
                    "current head"
                }
                _ => {
                    facts.parent_chain_observation = observation;
                    "complete parent chain"
                }
            };
            assert_eq!(
                assess_goal_authority(&facts, &reference(), &action(), sample(0, 0)),
                Err(GoalAuthorityAssessmentError::ObservationUnavailable(label))
            );
        }
    }
    for state in [
        SuppliedGoalAuthorityState::Unknown,
        SuppliedGoalAuthorityState::Conflicting,
    ] {
        let mut facts = facts();
        facts.subject_state = state;
        assert_eq!(
            assess_goal_authority(&facts, &reference(), &action(), sample(0, 0)),
            Err(GoalAuthorityAssessmentError::ObservationUnavailable(
                "stable subject"
            ))
        );
    }
    let mut facts = facts();
    facts.source.issuer_evidence.assurance = cockpit_protocol::AssuranceLevel::ProviderVerified;
    facts.source.issuer_evidence.evidence_refs = vec!["only-a-supplied-string".into()];
    reseal(&mut facts);
    facts.issuer_observation = SuppliedFactObservation::Unknown;
    assert_eq!(
        assess_goal_authority(&facts, &reference(), &action(), sample(0, 0)),
        Err(GoalAuthorityAssessmentError::ObservationUnavailable(
            "issuer"
        ))
    );
}

#[test]
fn action_must_fit_each_exact_scope_and_explicit_narrowing_cannot_expand() {
    for field in 0..4 {
        let mut action = action();
        let label = match field {
            0 => {
                action.actor = "agent:other".into();
                "actor"
            }
            1 => {
                action.operation = "source.delete".into();
                "operation"
            }
            2 => {
                action.resource = "repo:src/other.rs".into();
                "resource"
            }
            _ => {
                action.deliverable_refs = vec!["part-b".into()];
                "deliverables"
            }
        };
        assert_eq!(
            assess_goal_authority(&facts(), &reference(), &action, sample(0, 0)),
            Err(GoalAuthorityAssessmentError::ActionOutsideScope(label))
        );
    }
    for field in 0..3 {
        let mut action = action();
        let label = match field {
            0 => {
                action.work_item.actors.push("agent:other".into());
                "actors"
            }
            1 => {
                action.work_item.operations.push("source.delete".into());
                "operations"
            }
            _ => {
                action.work_item.resources.push("repo:src/other.rs".into());
                "resources"
            }
        };
        assert_eq!(
            assess_goal_authority(&facts(), &reference(), &action, sample(0, 0)),
            Err(GoalAuthorityAssessmentError::ScopeWidening(label))
        );
    }
    let mut facts = facts();
    facts.source.operations.push("source.delete".into());
    reseal(&mut facts);
    assert_eq!(
        assess_goal_authority(&facts, &reference(), &action(), sample(0, 0)),
        Err(GoalAuthorityAssessmentError::ScopeWidening(
            "issuer operations"
        ))
    );
}

#[test]
fn complete_parent_chain_intersects_windows_and_preserves_fixed_caps() {
    let mut facts = facts();
    facts.parent_chain = vec![
        parent("root", None, 1000, 9000),
        parent("parent", Some("root"), 2000, 8000),
    ];
    let mut action = action();
    action.work_item.parent_work_item_id = Some("parent".into());
    let mut reference = reference();
    reference.not_before = Some(instant(3000));
    reference.deadline = GoalAuthorityDeadline::CapAt {
        expires_at: instant(7000),
    };
    let assess = |facts: &ResolvedGoalAuthorityFacts, reference: &GoalAuthorityReference| {
        window(assess_goal_authority(facts, reference, &action, sample(3000, 1)).unwrap())
    };
    assert_eq!(
        assess(&facts, &reference),
        FiniteWindow::new(BASE + 3000, BASE + 7000).unwrap()
    );
    extend_version(&mut facts, 20_000);
    assert_eq!(
        assess(&facts, &reference).expires_at_utc_millis(),
        BASE + 7000
    );
    reference.deadline = GoalAuthorityDeadline::InheritSource {};
    reference.not_before = None;
    assert_eq!(
        assess(&facts, &reference),
        FiniteWindow::new(BASE + 2000, BASE + 8000).unwrap()
    );
}

#[test]
fn missing_forked_cyclic_or_foreign_parent_chains_are_rejected() {
    let mut child = action();
    child.work_item.parent_work_item_id = Some("parent".into());
    assert_eq!(
        assess_goal_authority(&facts(), &reference(), &child, sample(1000, 0)),
        Err(GoalAuthorityAssessmentError::InvalidFact(
            "complete parent chain"
        ))
    );
    for case in 0..4 {
        let mut facts = facts();
        facts.parent_chain = vec![parent("parent", None, 0, 9000)];
        match case {
            0 => facts.parent_chain[0].work_item.parent_work_item_id = Some("missing-root".into()),
            1 => facts
                .parent_chain
                .push(parent("parent", Some("parent"), 0, 8000)),
            2 => {
                facts.parent_chain[0].subject.repository_id =
                    Digest::sha256_bytes(b"foreign").to_string()
            }
            _ => facts.parent_chain[0].work_item.state = SuppliedGoalAuthorityState::Revoked,
        }
        assert!(assess_goal_authority(&facts, &reference(), &child, sample(1000, 0)).is_err());
    }
    let mut facts = facts();
    facts.parent_chain = vec![parent("WI-fixture", None, 0, 9000)];
    child.work_item.parent_work_item_id = Some("WI-fixture".into());
    assert_eq!(
        assess_goal_authority(&facts, &reference(), &child, sample(1000, 0)),
        Err(GoalAuthorityAssessmentError::InvalidFact(
            "complete parent chain"
        ))
    );
}

#[test]
fn explicit_time_widening_rejects_instead_of_silently_clamping() {
    let mut facts = facts();
    facts.parent_chain = vec![parent("parent", None, 1000, 9000)];
    let mut action = action();
    action.work_item.parent_work_item_id = Some("parent".into());
    for (start, end) in [(500, 8000), (2000, 9500)] {
        let mut reference = reference();
        reference.not_before = Some(instant(start));
        reference.deadline = GoalAuthorityDeadline::CapAt {
            expires_at: instant(end),
        };
        assert_eq!(
            assess_goal_authority(&facts, &reference, &action, sample(3000, 0)),
            Err(GoalAuthorityAssessmentError::ScopeWidening("validity"))
        );
    }
}

#[test]
fn exact_millisecond_boundaries_have_no_rounding_or_grace() {
    let facts = facts();
    for (offset, expected) in [
        (-1, Err(ClockError::NotYetValid)),
        (0, Ok(())),
        (9999, Ok(())),
        (10_000, Err(ClockError::Expired)),
    ] {
        assert_eq!(
            assess_goal_authority(&facts, &reference(), &action(), sample(offset, 0)).map(|_| ()),
            expected.map_err(GoalAuthorityAssessmentError::Clock)
        );
    }
    let mut reference = reference();
    reference.not_before = Some(instant(500));
    reference.deadline = GoalAuthorityDeadline::CapAt {
        expires_at: instant(501),
    };
    for (offset, allowed) in [(499, false), (500, true), (501, false)] {
        assert_eq!(
            assess_goal_authority(&facts, &reference, &action(), sample(offset, 0)).is_ok(),
            allowed
        );
    }
}

#[test]
fn p1_normalization_rejects_infinite_malformed_and_submillisecond_inputs() {
    for value in [
        "infinite",
        "until_done",
        "2030-01-01T00:00:10.0000000001Z",
        "2030-01-01T00:00:10-00:00",
    ] {
        let mut facts = facts();
        facts.source.validity.expires_at = value.into();
        assert!(matches!(
            assess_goal_authority(&facts, &reference(), &action(), sample(0, 0)),
            Err(GoalAuthorityAssessmentError::Normalization(_))
        ));
    }
    for field in 0..2 {
        let mut reference = reference();
        if field == 0 {
            reference.not_before = Some("2030-01-01T00:00:00.00000000001Z".into());
        } else {
            reference.deadline = GoalAuthorityDeadline::CapAt {
                expires_at: "2030-01-01T00:00:10.00000000001Z".into(),
            };
        }
        assert!(matches!(
            assess_goal_authority(&facts(), &reference, &action(), sample(0, 0)),
            Err(GoalAuthorityAssessmentError::Normalization(_))
        ));
    }
    let mut value = serde_json::to_value(source()).unwrap();
    value["validity"]["expiresAt"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<GoalAuthoritySource>(value.clone()).is_err());
    value["validity"]
        .as_object_mut()
        .unwrap()
        .remove("expiresAt");
    assert!(serde_json::from_value::<GoalAuthoritySource>(value).is_err());
}

#[test]
fn unsupported_constraints_and_resource_aliases_never_create_implicit_coverage() {
    for field in 0..2 {
        let mut facts = facts();
        let mut action = action();
        if field == 0 {
            facts.source.constraints.push("unknown-policy-dsl".into());
            reseal(&mut facts);
        } else {
            action
                .work_item
                .constraints
                .push("unknown-policy-dsl".into());
        }
        assert_eq!(
            assess_goal_authority(&facts, &reference(), &action, sample(0, 0)),
            Err(GoalAuthorityAssessmentError::UnsupportedConstraint)
        );
    }
    for resource in [
        "provider:github",
        "repo:src/../private",
        "repo:src//a",
        "repo:./a",
        "repo:/absolute",
        "repo:src\\a",
        "repo:src/**",
        "repo:",
    ] {
        let mut action = action();
        action.resource = resource.into();
        assert_eq!(
            assess_goal_authority(&facts(), &reference(), &action, sample(0, 0)),
            Err(GoalAuthorityAssessmentError::UnsupportedResource(
                resource.into()
            ))
        );
    }
}

#[test]
fn source_replacement_requires_complete_current_linear_version_chain() {
    let mut facts = facts();
    let old = facts.current_head.clone();
    extend_version(&mut facts, 20_000);
    assert!(assess_goal_authority(&facts, &reference(), &action(), sample(10_001, 1)).is_ok());
    let mut stale = facts.clone();
    stale.current_head = old;
    assert_eq!(
        assess_goal_authority(&stale, &reference(), &action(), sample(0, 0)),
        Err(GoalAuthorityAssessmentError::SourceHeadMismatch)
    );
    let mut missing = facts.clone();
    missing.version_history.remove(0);
    assert_eq!(
        assess_goal_authority(&missing, &reference(), &action(), sample(0, 0)),
        Err(GoalAuthorityAssessmentError::VersionChainIncomplete)
    );
    let mut forked = facts.clone();
    forked.version_history[1].predecessor_version_digest = Some(Digest::sha256_bytes(b"fork"));
    assert_eq!(
        assess_goal_authority(&forked, &reference(), &action(), sample(0, 0)),
        Err(GoalAuthorityAssessmentError::VersionChainForked)
    );
    facts.version_history[1].goal = "different head bytes".into();
    assert_eq!(
        assess_goal_authority(&facts, &reference(), &action(), sample(0, 0)),
        Err(GoalAuthorityAssessmentError::VersionChainForked)
    );
}

#[test]
fn ordinary_version_replacement_cannot_revive_stable_terminal_subjects_or_work_items() {
    for state in [
        SuppliedGoalAuthorityState::Completed,
        SuppliedGoalAuthorityState::Revoked,
        SuppliedGoalAuthorityState::Replaced,
    ] {
        let mut facts = facts();
        facts.subject_state = state;
        extend_version(&mut facts, 20_000);
        assert_eq!(
            assess_goal_authority(&facts, &reference(), &action(), sample(0, 0)),
            Err(GoalAuthorityAssessmentError::Terminated("stable subject"))
        );
        facts.subject_state = SuppliedGoalAuthorityState::Active;
        let mut action = action();
        action.work_item.state = state;
        assert_eq!(
            assess_goal_authority(&facts, &reference(), &action, sample(0, 0)),
            Err(GoalAuthorityAssessmentError::Terminated("Work Item"))
        );
    }
    let mut facts = facts();
    facts.source.termination = Some(GoalAuthorityTermination {
        kind: GoalAuthorityTerminationKind::Revoked,
        evidence_refs: vec!["test-owned-termination".into()],
    });
    reseal(&mut facts);
    extend_version(&mut facts, 20_000);
    facts.source.termination = None;
    reseal(&mut facts);
    assert_eq!(
        assess_goal_authority(&facts, &reference(), &action(), sample(0, 0)),
        Err(GoalAuthorityAssessmentError::Terminated(
            "source version history"
        ))
    );
}

/// This provider and the simulated effect below exist only in this integration test.
struct FixtureResolver {
    current: RefCell<ResolvedGoalAuthorityFacts>,
    error: RefCell<Option<GoalAuthorityResolutionError>>,
    calls: Cell<u64>,
}

impl FixtureResolver {
    fn new() -> Self {
        Self {
            current: RefCell::new(facts()),
            error: RefCell::new(None),
            calls: Cell::new(0),
        }
    }
}

impl CurrentGoalAuthorityResolver for FixtureResolver {
    fn resolve_current(
        &self,
        key: &GoalAuthoritySubjectKey,
    ) -> Result<ResolvedGoalAuthorityFacts, GoalAuthorityResolutionError> {
        self.calls.set(self.calls.get() + 1);
        if key != &crate::key() {
            return Err(GoalAuthorityResolutionError::Missing);
        }
        if let Some(error) = self.error.borrow().clone() {
            return Err(error);
        }
        Ok(self.current.borrow().clone())
    }
}

struct FixtureClock {
    samples: RefCell<VecDeque<Result<ClockSample, ClockError>>>,
    calls: Cell<u64>,
}

impl FixtureClock {
    fn new(samples: impl IntoIterator<Item = Result<ClockSample, ClockError>>) -> Self {
        Self {
            samples: RefCell::new(samples.into_iter().collect()),
            calls: Cell::new(0),
        }
    }
}

impl Clock for FixtureClock {
    fn sample(&self) -> Result<ClockSample, ClockError> {
        self.calls.set(self.calls.get() + 1);
        self.samples
            .borrow_mut()
            .pop_front()
            .expect("every simulated step must acquire its own sample")
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SimulatedStepError {
    Resolution(GoalAuthorityResolutionError),
    Clock(ClockError),
    Assessment(GoalAuthorityAssessmentError),
    Exhausted,
}

fn simulated_step(
    resolver: &impl CurrentGoalAuthorityResolver,
    clock: &impl Clock,
    original_budget: &mut MonotonicBudget,
    effects: &Cell<u64>,
) -> Result<(), SimulatedStepError> {
    let current = resolver
        .resolve_current(&key())
        .map_err(SimulatedStepError::Resolution)?;
    let fresh = clock.sample().map_err(SimulatedStepError::Clock)?;
    if original_budget
        .observe(fresh)
        .map_err(SimulatedStepError::Clock)?
        == 0
    {
        return Err(SimulatedStepError::Exhausted);
    }
    assess_goal_authority(&current, &reference(), &action(), fresh)
        .map_err(SimulatedStepError::Assessment)?;
    effects.set(effects.get() + 1);
    Ok(())
}

fn budget(start: ClockSample, limit: u64) -> MonotonicBudget {
    MonotonicBudget::new(
        FiniteWindow::new(BASE, BASE + 10_000).unwrap(),
        start,
        limit,
    )
    .unwrap()
}

#[test]
fn fixture_sampling_gap_and_expiry_after_wait_never_use_positive_cached_budget() {
    let resolver = FixtureResolver::new();
    let clock = FixtureClock::new([Err(ClockError::SamplingGap), Ok(sample(10_000, 10_000))]);
    let mut budget = budget(sample(0, 0), 20_000);
    let effects = Cell::new(0);
    assert!(budget.remaining_millis() > 0);
    assert_eq!(
        simulated_step(&resolver, &clock, &mut budget, &effects),
        Err(SimulatedStepError::Clock(ClockError::SamplingGap))
    );
    assert!(budget.remaining_millis() > 0); // Cached arithmetic cannot override that failed sample.
    assert_eq!(effects.get(), 0);
    // The injected wait is represented by the next queued sample at exact expiry; no sleeps.
    assert_eq!(
        simulated_step(&resolver, &clock, &mut budget, &effects),
        Err(SimulatedStepError::Exhausted)
    );
    assert_eq!(effects.get(), 0);
    assert_eq!(resolver.calls.get(), 2);
    assert_eq!(clock.calls.get(), 2);
}

#[test]
fn fixture_source_extension_does_not_refill_original_operation_budget() {
    let resolver = FixtureResolver::new();
    let clock = FixtureClock::new([Ok(sample(20, 20)), Ok(sample(40, 40)), Ok(sample(50, 50))]);
    let mut budget = budget(sample(0, 0), 50);
    let effects = Cell::new(0);
    simulated_step(&resolver, &clock, &mut budget, &effects).unwrap();
    assert_eq!(budget.remaining_millis(), 30);
    extend_version(&mut resolver.current.borrow_mut(), 20_000);
    simulated_step(&resolver, &clock, &mut budget, &effects).unwrap();
    assert_eq!(budget.remaining_millis(), 10);
    assert_eq!(
        simulated_step(&resolver, &clock, &mut budget, &effects),
        Err(SimulatedStepError::Exhausted)
    );
    assert_eq!(effects.get(), 2);
    assert_eq!(resolver.calls.get(), 3);
    assert_eq!(clock.calls.get(), 3);
}

#[test]
fn fixture_wall_and_monotonic_rollback_or_stalled_utc_produce_zero_new_effects() {
    for (fresh, expected) in [
        (sample(9, 11), ClockError::WallClockRollback),
        (sample(11, 9), ClockError::MonotonicRollback),
    ] {
        let resolver = FixtureResolver::new();
        let clock = FixtureClock::new([Ok(fresh), Ok(sample(12, 12))]);
        let mut budget = budget(sample(10, 10), 50);
        let effects = Cell::new(0);
        for _ in 0..2 {
            assert_eq!(
                simulated_step(&resolver, &clock, &mut budget, &effects),
                Err(SimulatedStepError::Clock(expected))
            );
            assert_eq!(effects.get(), 0);
        }
    }
    let clock = FixtureClock::new([Ok(sample(10, 50))]);
    let effects = Cell::new(0);
    assert_eq!(
        simulated_step(
            &FixtureResolver::new(),
            &clock,
            &mut budget(sample(10, 0), 50),
            &effects
        ),
        Err(SimulatedStepError::Exhausted)
    );
    assert_eq!(effects.get(), 0);
}

#[test]
fn fixture_resolves_current_facts_each_step_and_has_no_cached_or_default_fallback() {
    let resolver = FixtureResolver::new();
    let clock = FixtureClock::new([Ok(sample(0, 0)), Ok(sample(2, 2))]);
    let mut budget = budget(sample(0, 0), 50);
    let effects = Cell::new(0);
    simulated_step(&resolver, &clock, &mut budget, &effects).unwrap();
    *resolver.error.borrow_mut() = Some(GoalAuthorityResolutionError::Unavailable);
    assert_eq!(
        simulated_step(&resolver, &clock, &mut budget, &effects),
        Err(SimulatedStepError::Resolution(
            GoalAuthorityResolutionError::Unavailable
        ))
    );
    assert_eq!(clock.calls.get(), 1);
    *resolver.error.borrow_mut() = None;
    resolver.current.borrow_mut().subject_state = SuppliedGoalAuthorityState::Revoked;
    assert!(matches!(
        simulated_step(&resolver, &clock, &mut budget, &effects),
        Err(SimulatedStepError::Assessment(
            GoalAuthorityAssessmentError::Terminated(_)
        ))
    ));
    assert_eq!(effects.get(), 1);
    assert_eq!(resolver.calls.get(), 3);
    assert_eq!(clock.calls.get(), 2);
    assert_eq!(
        simulated_step(
            &UnsupportedCurrentGoalAuthorityResolver,
            &clock,
            &mut budget,
            &effects
        ),
        Err(SimulatedStepError::Resolution(
            GoalAuthorityResolutionError::Unsupported
        ))
    );
    assert_eq!(effects.get(), 1);
}
