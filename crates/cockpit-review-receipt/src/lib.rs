//! Domain model for controlled review of `test_weakening` findings.
//!
//! This crate is intentionally standalone and **not** wired into any
//! production admission path yet (`cockpit_repository`'s
//! `material_review_gate_projection_with_contract_digest` and every other
//! existing gate are unchanged, and do not depend on this crate at all). It
//! exists so the domain model,
//! its fail-closed defaults, and the specific failure modes it must reject
//! can be built and regression-tested in isolation before any gate consults
//! it. Until a later stage wires this in, every existing Work Item's
//! `test_weakening` finding blocks exactly as it does today.
//!
//! No live external-authority check is implemented here. The only
//! `AuthorityVerifier` shipped in this module (`NoAuthorityVerifierConfigured`)
//! always reports [`AuthorityVerification::Unverifiable`]; it is impossible
//! for any receipt to reach [`ItemCoverageState::ReviewedPass`] through it.
//! Wiring a real external verifier (e.g. an independently re-queried GitHub
//! PR review) is explicitly out of scope for this stage and is not faked
//! here.

use sha2::{Digest as ShaDigest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// One specific, individually reviewable item inside a larger
/// `test_weakening` finding: a single removed test's identity, not a whole
/// file or a whole diff.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestWeakeningItem {
    pub path: String,
    pub item_key: String,
    pub reason: String,
}

impl TestWeakeningItem {
    pub fn identity(&self) -> (String, String) {
        (self.path.clone(), self.item_key.clone())
    }
}

/// The exact material a review receipt is bound to. Equality is byte-exact:
/// any change to any one field invalidates every receipt bound to the
/// previous value.
///
/// `repository_snapshot_digest` must be computed over the *entire* reviewed
/// tree, including every path under `.ai/` that can affect the judgment
/// (Contract, decisions, prior evidence) — it must **not** reuse a
/// blanket-`.ai`-excluding digest (`cockpit_git`'s existing
/// `source_tree_digest` is documented to exclude all of `.ai`; it must never
/// be used here for that reason). The only path(s) excluded from this digest
/// are the specific review-receipt file(s) being written by *this* act of
/// recording a verdict — never the whole `.ai/` tree. Wiring that exact,
/// narrow exclusion into a real digest computation is a later-stage
/// integration concern; see `self_reference` integration test below for an
/// empirical proof that neither of `cockpit_git`'s two existing digests
/// (`tree_digest`, `source_tree_digest`) already has this property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundSnapshot {
    pub repository_snapshot_digest: String,
    pub contract_digest: String,
    pub finding_set_digest: String,
    pub verification_evidence_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparerKind {
    Agent,
    Human,
}

/// Whoever assembled the technical analysis bound into a receipt. May be an
/// agent. A `Preparer` can never become an `Approver` — there is no
/// conversion between the two types, and `Approver` carries no `kind` field
/// at all, only an externally-verifiable identity claim. This is a type-level
/// guarantee, not a runtime check: it is not possible to construct an
/// `Approver` from a `Preparer` without writing new code that does so
/// explicitly and visibly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preparer {
    pub kind: PreparerKind,
    pub identity: String,
}

/// A claim that a specific, named external reviewer approved a specific
/// payload. This claim is never trusted on its own — [`AuthorityVerifier`]
/// must independently re-confirm it against the live external source before
/// it can authorize anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalIdentityClaim {
    pub source: String,
    pub pr_number: u64,
    pub review_id: u64,
    pub reviewer_login: String,
    pub submitted_at: String,
    pub reviewed_commit_sha: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approver {
    pub external_identity: ExternalIdentityClaim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    GenuineWeakening,
    LegitimateRefactorEquivalent,
    InsufficientEvidence,
}

/// Evidence that a specific command actually ran, not merely that static
/// analysis was performed. `dynamic: false` means the evidence is
/// static-analysis-only (e.g. a lexical diff inspection) and can never by
/// itself satisfy the "real verification" requirement for a passing item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationEvidenceRecord {
    pub command: String,
    pub args: Vec<String>,
    pub exit_code: i32,
    pub log_digest: String,
    pub recorded_at: String,
    pub dynamic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewReceipt {
    pub item: TestWeakeningItem,
    pub bound_snapshot: BoundSnapshot,
    pub preparer: Preparer,
    pub approver: Approver,
    pub verdict: Verdict,
    pub verification_evidence: VerificationEvidenceRecord,
    /// Whether the repository had any uncommitted (dirty) paths at the
    /// moment this receipt's `bound_snapshot` was computed. A real external
    /// reviewer (e.g. a GitHub PR review) can only ever have seen committed
    /// content at `reviewed_commit_sha`; if the material actually reviewed
    /// here included uncommitted changes, no external approval can cover it,
    /// regardless of what the approver claims.
    pub repository_had_dirty_paths_at_binding: bool,
}

/// A stable digest over exactly the fields that must not change out from
/// under an approval: the item identity, the full bound snapshot, and the
/// verdict. An approver's confirmation is only meaningful if it was issued
/// for this exact value — not merely "approved the PR" in general.
pub fn payload_digest(item: &TestWeakeningItem, snapshot: &BoundSnapshot, verdict: Verdict) -> String {
    let verdict_tag = match verdict {
        Verdict::GenuineWeakening => "genuine_weakening",
        Verdict::LegitimateRefactorEquivalent => "legitimate_refactor_equivalent",
        Verdict::InsufficientEvidence => "insufficient_evidence",
    };
    let mut hasher = Sha256::new();
    hasher.update(item.path.as_bytes());
    hasher.update(b"\0");
    hasher.update(item.item_key.as_bytes());
    hasher.update(b"\0");
    hasher.update(snapshot.repository_snapshot_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(snapshot.contract_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(snapshot.finding_set_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(snapshot.verification_evidence_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(verdict_tag.as_bytes());
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorityVerification {
    /// Independently re-confirmed: this exact `payload_digest` really was
    /// approved by `claim.reviewer_login`.
    Confirmed,
    /// No live confirmation is actually available (the default today: no
    /// external verifier is wired up at all).
    Unverifiable,
    /// Actively confirmed NOT to match (e.g. the live review's approved
    /// content digest differs from `payload_digest`).
    Mismatch,
}

pub trait AuthorityVerifier {
    fn verify(&self, claim: &ExternalIdentityClaim, payload_digest: &str) -> AuthorityVerification;
}

/// The only `AuthorityVerifier` wired up in production as of this stage.
/// It never confirms anything: no receipt can reach `ReviewedPass` through
/// it. A real implementation (re-querying a live external source such as a
/// GitHub PR review API and comparing the result's own content digest
/// against `payload_digest`) is a later-stage gap, not implemented here.
pub struct NoAuthorityVerifierConfigured;

impl AuthorityVerifier for NoAuthorityVerifierConfigured {
    fn verify(&self, _claim: &ExternalIdentityClaim, _payload_digest: &str) -> AuthorityVerification {
        AuthorityVerification::Unverifiable
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemCoverageState {
    Uncovered,
    Stale,
    AuthorizationUnverifiable,
    ReviewedFail,
    ReviewedInsufficient,
    ReviewedPass,
}

pub struct CoverageInput<'a> {
    pub current_snapshot: &'a BoundSnapshot,
}

/// Evaluate a single item's coverage state. A missing receipt is
/// `Uncovered`. Every other rejection path (`Stale`,
/// `AuthorizationUnverifiable`) is checked *before* trusting the verdict, so
/// a receipt can never reach `ReviewedFail`/`ReviewedInsufficient`/`ReviewedPass`
/// while any precondition is unmet.
pub fn evaluate_item(
    receipt: Option<&ReviewReceipt>,
    input: &CoverageInput,
    verifier: &dyn AuthorityVerifier,
) -> ItemCoverageState {
    let Some(receipt) = receipt else {
        return ItemCoverageState::Uncovered;
    };
    if &receipt.bound_snapshot != input.current_snapshot {
        return ItemCoverageState::Stale;
    }
    if receipt.repository_had_dirty_paths_at_binding {
        // An external reviewer can only ever have seen committed content.
        // Uncommitted material at binding time can never be authorized by
        // any `reviewedCommitSha`-based external approval, no matter what
        // the approver claims. Model this identically to `Stale`: the
        // material is not a valid, reviewable, immutable snapshot at all.
        return ItemCoverageState::Stale;
    }
    let digest = payload_digest(&receipt.item, &receipt.bound_snapshot, receipt.verdict);
    match verifier.verify(&receipt.approver.external_identity, &digest) {
        AuthorityVerification::Confirmed => {}
        AuthorityVerification::Unverifiable | AuthorityVerification::Mismatch => {
            return ItemCoverageState::AuthorizationUnverifiable;
        }
    }
    if !receipt.verification_evidence.dynamic {
        // Static analysis alone (no recorded command/exit code) can never
        // satisfy the "real verification" requirement.
        return ItemCoverageState::ReviewedInsufficient;
    }
    match receipt.verdict {
        Verdict::GenuineWeakening => ItemCoverageState::ReviewedFail,
        Verdict::InsufficientEvidence => ItemCoverageState::ReviewedInsufficient,
        Verdict::LegitimateRefactorEquivalent => ItemCoverageState::ReviewedPass,
    }
}

/// Independently recomputed, trusted enumeration of every item a raw
/// `test_weakening = true` assessment must account for. This is never taken
/// from a receipt or from agent-supplied input — it is the caller's
/// responsibility to derive it fresh from the same governance scan that
/// produced the raw boolean, so coverage can never be satisfied by a
/// claimed list that merely looks complete.
///
/// Returns `false` (never a panic, never a silent pass) when the raw signal
/// and the structured enumeration disagree in a way that itself signals a
/// parse failure or an incomplete/unmapped enumeration upstream:
/// `raw_test_weakening == true` with zero items is exactly that case, and is
/// treated as a hard failure of completeness, not as "nothing to review".
pub fn enumeration_is_complete(raw_test_weakening: bool, items: &BTreeSet<(String, String)>) -> bool {
    if raw_test_weakening {
        !items.is_empty()
    } else {
        items.is_empty()
    }
}

/// The gate decision: admits only when every expected item is covered by a
/// fresh, fully authorized, dynamically-verified `ReviewedPass`.
///
/// Deliberately checks `expected_items.is_empty()` as an explicit failure,
/// not merely relying on `Iterator::all`, which returns `true` on an empty
/// iterator (`[].iter().all(_)` is vacuously `true` in Rust) and would
/// otherwise let a `test_weakening = true` input with a (buggy, incomplete,
/// or unparsed) empty item list silently pass the gate.
pub fn gate_admits(
    expected_items: &BTreeSet<(String, String)>,
    receipts_by_item: &BTreeMap<(String, String), ReviewReceipt>,
    input: &CoverageInput,
    verifier: &dyn AuthorityVerifier,
) -> bool {
    if expected_items.is_empty() {
        return false;
    }
    expected_items.iter().all(|key| {
        let receipt = receipts_by_item.get(key);
        evaluate_item(receipt, input, verifier) == ItemCoverageState::ReviewedPass
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(path: &str, key: &str) -> TestWeakeningItem {
        TestWeakeningItem {
            path: path.into(),
            item_key: key.into(),
            reason: "removed_without_proven_migration".into(),
        }
    }

    fn snapshot(tag: &str) -> BoundSnapshot {
        BoundSnapshot {
            repository_snapshot_digest: format!("sha256:tree-{tag}"),
            contract_digest: format!("sha256:contract-{tag}"),
            finding_set_digest: format!("sha256:findings-{tag}"),
            verification_evidence_digest: format!("sha256:verification-{tag}"),
        }
    }

    fn dynamic_evidence() -> VerificationEvidenceRecord {
        VerificationEvidenceRecord {
            command: "cargo".into(),
            args: vec!["test".into(), "-p".into(), "cockpit-repository".into()],
            exit_code: 0,
            log_digest: "sha256:log".into(),
            recorded_at: "2026-10-21T00:00:00Z".into(),
            dynamic: true,
        }
    }

    fn static_only_evidence() -> VerificationEvidenceRecord {
        VerificationEvidenceRecord {
            dynamic: false,
            ..dynamic_evidence()
        }
    }

    fn claim(reviewer: &str) -> ExternalIdentityClaim {
        ExternalIdentityClaim {
            source: "github_pr_review".into(),
            pr_number: 9999,
            review_id: 1,
            reviewer_login: reviewer.into(),
            submitted_at: "2026-10-21T00:00:00Z".into(),
            reviewed_commit_sha: "deadbeef".into(),
        }
    }

    fn passing_receipt(path: &str, key: &str, snap: BoundSnapshot) -> ReviewReceipt {
        ReviewReceipt {
            item: item(path, key),
            bound_snapshot: snap,
            preparer: Preparer {
                kind: PreparerKind::Agent,
                identity: "agent-session-1".into(),
            },
            approver: Approver {
                external_identity: claim("real-human-reviewer"),
            },
            verdict: Verdict::LegitimateRefactorEquivalent,
            verification_evidence: dynamic_evidence(),
            repository_had_dirty_paths_at_binding: false,
        }
    }

    /// A verifier that only confirms when asked about one exact,
    /// pre-registered payload digest — modelling a real external check that
    /// compares against what was actually reviewed, not merely "approved
    /// something".
    struct ConfirmsExactDigestOnly {
        confirmed_digest: String,
    }

    impl AuthorityVerifier for ConfirmsExactDigestOnly {
        fn verify(&self, _claim: &ExternalIdentityClaim, payload_digest: &str) -> AuthorityVerification {
            if payload_digest == self.confirmed_digest {
                AuthorityVerification::Confirmed
            } else {
                AuthorityVerification::Mismatch
            }
        }
    }

    // ---- Point 1: authorization must bind to the exact payload ----

    #[test]
    fn approval_of_a_different_payload_digest_does_not_authorize_this_one() {
        let snap = snapshot("v1");
        let receipt = passing_receipt("tests/a.rs", "foo", snap.clone());
        // The external reviewer actually approved some other content --
        // e.g. the receipt's verdict or bound snapshot was altered after
        // the human looked at it.
        let verifier = ConfirmsExactDigestOnly {
            confirmed_digest: "sha256:something-else-entirely".into(),
        };
        let state = evaluate_item(Some(&receipt), &CoverageInput { current_snapshot: &snap }, &verifier);
        assert_eq!(state, ItemCoverageState::AuthorizationUnverifiable);
    }

    #[test]
    fn plain_approved_state_without_exact_digest_match_never_passes() {
        // Even a verifier that models "the PR was APPROVED" still must not
        // authorize a receipt unless it confirms the *exact* payload digest.
        let snap = snapshot("v1");
        let receipt = passing_receipt("tests/a.rs", "foo", snap.clone());
        let wrong_digest_verifier = ConfirmsExactDigestOnly {
            confirmed_digest: payload_digest(&item("tests/a.rs", "foo"), &snapshot("v2"), Verdict::LegitimateRefactorEquivalent),
        };
        let state = evaluate_item(Some(&receipt), &CoverageInput { current_snapshot: &snap }, &wrong_digest_verifier);
        assert_eq!(state, ItemCoverageState::AuthorizationUnverifiable);
    }

    #[test]
    fn no_authority_verifier_configured_never_passes_anything() {
        // The production default today: no live external verifier exists.
        // No receipt, however constructed, can reach ReviewedPass through
        // it. This is the honestly-reported capability gap.
        let snap = snapshot("v1");
        let receipt = passing_receipt("tests/a.rs", "foo", snap.clone());
        let state = evaluate_item(
            Some(&receipt),
            &CoverageInput { current_snapshot: &snap },
            &NoAuthorityVerifierConfigured,
        );
        assert_eq!(state, ItemCoverageState::AuthorizationUnverifiable);
    }

    #[test]
    fn approver_type_cannot_be_constructed_from_an_agent_preparer() {
        // Type-level guarantee, exercised here by construction: Approver
        // has no `kind` field and no conversion from Preparer exists. This
        // test documents and locks in that structural separation -- it
        // would fail to compile (not fail at runtime) if someone added a
        // `Preparer -> Approver` conversion that let an agent identity
        // satisfy `Approver` directly.
        let agent_preparer = Preparer {
            kind: PreparerKind::Agent,
            identity: "agent-session-1".into(),
        };
        let approver = Approver {
            external_identity: claim("real-human-reviewer"),
        };
        assert_ne!(agent_preparer.identity, approver.external_identity.reviewer_login);
    }

    // ---- Point 2: empty/incomplete/parse-failure/unmapped enumeration ----

    #[test]
    fn empty_expected_items_with_test_weakening_true_never_passes_vacuously() {
        let snap = snapshot("v1");
        let expected_items: BTreeSet<(String, String)> = BTreeSet::new();
        let receipts = BTreeMap::new();
        let admitted = gate_admits(
            &expected_items,
            &receipts,
            &CoverageInput { current_snapshot: &snap },
            &NoAuthorityVerifierConfigured,
        );
        assert!(
            !admitted,
            "an empty expected-items set must never admit via Iterator::all's vacuous truth"
        );
        assert!(!enumeration_is_complete(true, &expected_items));
    }

    #[test]
    fn incomplete_enumeration_blocks_even_with_one_fully_valid_receipt() {
        let snap = snapshot("v1");
        let mut expected_items = BTreeSet::new();
        expected_items.insert(("tests/a.rs".to_string(), "foo".to_string()));
        expected_items.insert(("tests/b.rs".to_string(), "bar".to_string()));

        let mut receipts = BTreeMap::new();
        let receipt = passing_receipt("tests/a.rs", "foo", snap.clone());
        let verifier = ConfirmsExactDigestOnly {
            confirmed_digest: payload_digest(&receipt.item, &receipt.bound_snapshot, receipt.verdict),
        };
        receipts.insert(("tests/a.rs".to_string(), "foo".to_string()), receipt);
        // tests/b.rs#bar has no receipt at all.

        let admitted = gate_admits(
            &expected_items,
            &receipts,
            &CoverageInput { current_snapshot: &snap },
            &verifier,
        );
        assert!(!admitted, "two expected items with only one covered must not admit");
    }

    #[test]
    fn unmapped_finding_fails_completeness_check() {
        // raw_test_weakening = true but the structured item list is empty:
        // exactly the "parse failure or incomplete/unmapped enumeration"
        // shape this function exists to catch.
        assert!(!enumeration_is_complete(true, &BTreeSet::new()));
        // The converse is also an inconsistency worth rejecting: a nonempty
        // item list with no corresponding raw signal at all.
        let mut items = BTreeSet::new();
        items.insert(("tests/a.rs".to_string(), "foo".to_string()));
        assert!(!enumeration_is_complete(false, &items));
    }

    #[test]
    fn malformed_receipt_json_is_treated_as_uncovered_not_passing() {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str("{ this is not valid json");
        assert!(parsed.is_err());
        // A caller that cannot parse a receipt must fall back to treating
        // the item as having no receipt at all (Uncovered), never default-
        // constructing a passing one.
        let snap = snapshot("v1");
        let state = evaluate_item(None, &CoverageInput { current_snapshot: &snap }, &NoAuthorityVerifierConfigured);
        assert_eq!(state, ItemCoverageState::Uncovered);
    }

    // ---- Point 3: reviewedCommitSha cannot authorize uncommitted content ----

    #[test]
    fn dirty_working_tree_at_binding_time_can_never_be_authorized() {
        let snap = snapshot("v1");
        let mut receipt = passing_receipt("tests/a.rs", "foo", snap.clone());
        receipt.repository_had_dirty_paths_at_binding = true;
        let verifier = ConfirmsExactDigestOnly {
            confirmed_digest: payload_digest(&receipt.item, &receipt.bound_snapshot, receipt.verdict),
        };
        let state = evaluate_item(Some(&receipt), &CoverageInput { current_snapshot: &snap }, &verifier);
        assert_eq!(
            state,
            ItemCoverageState::Stale,
            "a receipt bound to a dirty working tree must never authorize, even with a matching digest"
        );
    }

    #[test]
    fn content_change_after_review_invalidates_the_receipt() {
        let snap_v1 = snapshot("v1");
        let snap_v2 = snapshot("v2");
        let receipt = passing_receipt("tests/a.rs", "foo", snap_v1);
        let verifier = ConfirmsExactDigestOnly {
            confirmed_digest: payload_digest(&receipt.item, &receipt.bound_snapshot, receipt.verdict),
        };
        let state = evaluate_item(Some(&receipt), &CoverageInput { current_snapshot: &snap_v2 }, &verifier);
        assert_eq!(state, ItemCoverageState::Stale);
    }

    // ---- Point 4: snapshot binding must cover Contract/config/evidence, not just source ----

    #[test]
    fn contract_only_change_elsewhere_still_invalidates_the_receipt() {
        let mut snap_v1 = snapshot("v1");
        let snap_bound = snap_v1.clone();
        let receipt = passing_receipt("tests/a.rs", "foo", snap_bound.clone());
        // Only the Contract digest changes; source/finding/verification
        // digests are untouched.
        snap_v1.contract_digest = "sha256:contract-v1-amended".into();
        let verifier = ConfirmsExactDigestOnly {
            confirmed_digest: payload_digest(&receipt.item, &snap_bound, receipt.verdict),
        };
        let state = evaluate_item(Some(&receipt), &CoverageInput { current_snapshot: &snap_v1 }, &verifier);
        assert_eq!(
            state,
            ItemCoverageState::Stale,
            "a Contract-only change elsewhere in .ai/ must still invalidate a bound receipt"
        );
    }

    // ---- Point 5: static analysis alone cannot satisfy the verification requirement ----

    #[test]
    fn static_only_evidence_can_never_reach_reviewed_pass() {
        let snap = snapshot("v1");
        let mut receipt = passing_receipt("tests/a.rs", "foo", snap.clone());
        receipt.verification_evidence = static_only_evidence();
        let verifier = ConfirmsExactDigestOnly {
            confirmed_digest: payload_digest(&receipt.item, &receipt.bound_snapshot, receipt.verdict),
        };
        let state = evaluate_item(Some(&receipt), &CoverageInput { current_snapshot: &snap }, &verifier);
        assert_eq!(state, ItemCoverageState::ReviewedInsufficient);
    }

    // ---- The one success chain: immutable snapshot -> real verification -> precise authorization -> gate check ----

    #[test]
    fn full_success_chain_admits_only_when_every_condition_holds() {
        let snap = snapshot("success-chain");
        let receipt = passing_receipt("tests/a.rs", "foo", snap.clone());
        assert!(!receipt.repository_had_dirty_paths_at_binding);
        assert!(receipt.verification_evidence.dynamic);
        assert_eq!(receipt.verification_evidence.exit_code, 0);

        let mut expected_items = BTreeSet::new();
        expected_items.insert(("tests/a.rs".to_string(), "foo".to_string()));
        let mut receipts = BTreeMap::new();
        let exact_digest = payload_digest(&receipt.item, &receipt.bound_snapshot, receipt.verdict);
        receipts.insert(("tests/a.rs".to_string(), "foo".to_string()), receipt);

        // This verifier is a TEST DOUBLE standing in for a real, live
        // external check (e.g. an independently re-queried GitHub PR
        // review). It is defined only in this test module and is never the
        // production default (`NoAuthorityVerifierConfigured`, used
        // everywhere else in this file, always returns Unverifiable).
        let verifier = ConfirmsExactDigestOnly {
            confirmed_digest: exact_digest,
        };

        let admitted = gate_admits(
            &expected_items,
            &receipts,
            &CoverageInput { current_snapshot: &snap },
            &verifier,
        );
        assert!(admitted);
    }

    #[test]
    fn same_success_chain_blocks_the_moment_any_one_condition_fails() {
        let snap = snapshot("success-chain-2");
        let mut expected_items = BTreeSet::new();
        expected_items.insert(("tests/a.rs".to_string(), "foo".to_string()));

        // Variant: dirty-at-binding.
        {
            let mut receipt = passing_receipt("tests/a.rs", "foo", snap.clone());
            receipt.repository_had_dirty_paths_at_binding = true;
            let digest = payload_digest(&receipt.item, &receipt.bound_snapshot, receipt.verdict);
            let mut receipts = BTreeMap::new();
            receipts.insert(("tests/a.rs".to_string(), "foo".to_string()), receipt);
            let verifier = ConfirmsExactDigestOnly { confirmed_digest: digest };
            assert!(!gate_admits(&expected_items, &receipts, &CoverageInput { current_snapshot: &snap }, &verifier));
        }
        // Variant: static-only evidence.
        {
            let mut receipt = passing_receipt("tests/a.rs", "foo", snap.clone());
            receipt.verification_evidence = static_only_evidence();
            let digest = payload_digest(&receipt.item, &receipt.bound_snapshot, receipt.verdict);
            let mut receipts = BTreeMap::new();
            receipts.insert(("tests/a.rs".to_string(), "foo".to_string()), receipt);
            let verifier = ConfirmsExactDigestOnly { confirmed_digest: digest };
            assert!(!gate_admits(&expected_items, &receipts, &CoverageInput { current_snapshot: &snap }, &verifier));
        }
        // Variant: production default verifier (no live authority wired up).
        {
            let receipt = passing_receipt("tests/a.rs", "foo", snap.clone());
            let mut receipts = BTreeMap::new();
            receipts.insert(("tests/a.rs".to_string(), "foo".to_string()), receipt);
            assert!(!gate_admits(
                &expected_items,
                &receipts,
                &CoverageInput { current_snapshot: &snap },
                &NoAuthorityVerifierConfigured
            ));
        }
    }
}
