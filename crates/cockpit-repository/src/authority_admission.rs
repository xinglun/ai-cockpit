//! P2b entry/commit wiring with an unavailable production authority source.
//!
//! A declaration can request a stricter rejection; it cannot authenticate an
//! enabled mode, issuer, actor or grant. Independently trusted enrollment and
//! a writer/consumer source fence are not installed by this substage.

use crate::{ObserverError, evidence_store};
use cap_std::fs::Dir;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Private, non-serializable operation context. This factory can currently
/// construct only an unconfigured legacy context, never an authenticated one.
pub(crate) struct AdmissionContext {
    root: PathBuf,
    work_item_id: Option<String>,
}

/// A legacy commit boundary, not a source lease or an authorization receipt.
pub(crate) struct CommitBoundary {
    _private: (),
}

impl AdmissionContext {
    pub(crate) fn before_commit(&self) -> Result<CommitBoundary, ObserverError> {
        check_requested_root(&self.root, self.work_item_id.as_deref())?;
        Ok(CommitBoundary { _private: () })
    }
}

pub(crate) fn enter(
    root: &Path,
    work_item_id: Option<&str>,
) -> Result<AdmissionContext, ObserverError> {
    check_requested_root(root, work_item_id)?;
    Ok(AdmissionContext {
        root: root.to_path_buf(),
        work_item_id: work_item_id.map(str::to_owned),
    })
}

/// Reject a requested authority boundary when its production source is
/// unavailable. Success describes legacy compatibility, not a grant/permit.
pub fn require_goal_authority_access(
    root: &Path,
    work_item_id: Option<&str>,
) -> Result<(), ObserverError> {
    let _context = enter(root, work_item_id)?;
    Ok(())
}

/// Locate request declarations for a standalone input/output path without
/// reading that business file. This remains a denial/legacy check, not IAM.
pub fn require_goal_authority_path_access(path: &Path) -> Result<(), ObserverError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|source| ObserverError::Read {
                path: path.into(),
                source,
            })?
            .join(path)
    };
    let mut paths = vec![absolute.clone()];
    if let Ok(canonical) = absolute.canonicalize()
        && canonical != absolute
    {
        paths.push(canonical);
    }
    for path in paths {
        for candidate in path.ancestors() {
            if candidate.is_dir() && candidate.join(".ai").exists() {
                require_goal_authority_access(candidate, None)?;
                break;
            }
        }
    }
    Ok(())
}

pub(crate) fn context_for_path(path: &Path) -> Result<Option<AdmissionContext>, ObserverError> {
    let root = path.ancestors().find_map(|ancestor| {
        ancestor.file_name().and_then(|name| {
            (name == ".ai" || name == ".git")
                .then(|| ancestor.parent())
                .flatten()
        })
    });
    root.map(|root| enter(root, None)).transpose()
}

fn optional_directory(parent: &Dir, name: &str, path: &Path) -> Result<Option<Dir>, ObserverError> {
    evidence_store::open_optional_cap_directory_nofollow_strict(parent, name, path)
}

fn check_locator(root: &Path, parent: &Dir, name: &str, path: &Path) -> Result<(), ObserverError> {
    let bytes = match evidence_store::read_optional_cap_file_nofollow_bounded(
        parent,
        name,
        path,
        crate::MAX_RECEIPT_INDEX_BYTES,
    ) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(()),
        Err(ObserverError::Read { source, .. })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            return Ok(());
        }
        Err(error) => {
            return Err(ObserverError::State {
                path: path.into(),
                message: format!(
                    "goal_authority_locator_invalid: authority locator must be a readable regular non-symlink file: {error}"
                ),
            });
        }
    };
    // Legacy malformed documents remain the responsibility of their existing
    // validators. With no trusted enrollment installed, this is not a claim
    // that editing or deleting a declaration cannot disable it.
    let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
        return Ok(());
    };
    crate::reject_duplicate_json_keys(&bytes).map_err(|error| ObserverError::State {
        path: path.into(),
        message: format!("goal_authority_declaration_invalid: {error}"),
    })?;
    let Some(declaration) = value.pointer("/governanceProfile/goalAuthority") else {
        return Ok(());
    };
    let invalid = |error: String| ObserverError::State {
        path: path.into(),
        message: format!("goal_authority_declaration_invalid: {error}"),
    };
    let declaration: cockpit_protocol::GoalAuthorityDeclaration =
        serde_json::from_value(declaration.clone()).map_err(|error| invalid(error.to_string()))?;
    declaration
        .validate_shape()
        .map_err(|error| invalid(error.to_string()))?;
    if let Some(reference) = &declaration.reference {
        crate::authority::normalize_goal_authority_reference(reference)
            .map_err(|error| invalid(error.to_string()))?;
    }
    if declaration.required {
        Err(ObserverError::State {
            path: path.into(),
            message: UnsupportedSource.require_enrollment(root).unwrap_err(),
        })
    } else {
        Ok(())
    }
}

// Resolve only declaration locations, without a Git snapshot or subprocess.
// A caller-supplied subdirectory/alias must not hide its enclosing declaration.
// This is locator completeness, not independently trusted enabled enrollment.
fn check_requested_root(root: &Path, work_item_id: Option<&str>) -> Result<(), ObserverError> {
    check_declarations(root, work_item_id)?;
    let canonical = root.canonicalize().map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if canonical.join(".ai").exists() {
        return Ok(());
    }
    for parent in canonical.ancestors().skip(1) {
        if parent.join(".ai").exists() {
            check_declarations(parent, None)?;
            break;
        }
    }
    Ok(())
}

fn check_declarations(root: &Path, work_item_id: Option<&str>) -> Result<(), ObserverError> {
    if let Some(id) = work_item_id {
        crate::validate_work_item_id(id)?;
    }
    let directory =
        Dir::open_ambient_dir(root, cap_std::ambient_authority()).map_err(|source| {
            ObserverError::Read {
                path: root.into(),
                source,
            }
        })?;
    let Some(ai) = optional_directory(&directory, ".ai", &root.join(".ai"))? else {
        return Ok(());
    };
    let path = root.join(".ai/work-items");
    let Some(items) = optional_directory(&ai, "work-items", &path)? else {
        return Ok(());
    };
    for section in ["active", "archive"] {
        // An unbound operation must not silently choose one active goal.
        // Historical inventory is not an enrollment registry: do not scan it
        // wholesale or interpret old descriptive records as current grants.
        if work_item_id.is_none() && section == "archive" {
            continue;
        }
        let section_path = path.join(section);
        let Some(directory) = optional_directory(&items, section, &section_path)? else {
            continue;
        };
        if let Some(id) = work_item_id {
            let name = format!("{id}.contract.json");
            check_locator(root, &directory, &name, &section_path.join(&name))?;
        } else {
            for entry in directory.entries().map_err(|source| ObserverError::Read {
                path: section_path.clone(),
                source,
            })? {
                let entry = entry.map_err(|source| ObserverError::Read {
                    path: section_path.clone(),
                    source,
                })?;
                let name = entry.file_name();
                let Some(name) = name
                    .to_str()
                    .filter(|name| name.ends_with(".contract.json"))
                else {
                    continue;
                };
                check_locator(root, &directory, name, &section_path.join(name))?;
            }
        }
    }
    Ok(())
}

// The source protocol stays private. No public resolver, RuntimeContext,
// serialized value, ambient permit or production configuration selects it.
#[allow(dead_code)]
trait OrderedSource {
    /// Independently bind enabled mode and the actual repository/source store.
    /// A mutable Contract declaration is only a locator and cannot satisfy this.
    fn require_enrollment(&self, root: &Path) -> Result<(), String>;
    /// Acquire the shared writer/consumer fence last, validate pinned storage
    /// identity, and reread authenticated head AND stable termination under it.
    /// The callback is bounded; no operation-lock wait is permitted inside it.
    fn with_current<T>(
        &self,
        key: &crate::authority::GoalAuthoritySubjectKey,
        operation: impl FnOnce(&crate::authority::ResolvedGoalAuthorityFacts) -> Result<T, String>,
    ) -> Result<T, String>;
}

struct UnsupportedSource;
impl OrderedSource for UnsupportedSource {
    fn require_enrollment(&self, _root: &Path) -> Result<(), String> {
        Err("goal_authority_source_unsupported: no authenticated production source, enabled-mode binding or source commit fence is configured".into())
    }
    fn with_current<T>(
        &self,
        _key: &crate::authority::GoalAuthoritySubjectKey,
        _operation: impl FnOnce(&crate::authority::ResolvedGoalAuthorityFacts) -> Result<T, String>,
    ) -> Result<T, String> {
        self.require_enrollment(Path::new("."))?;
        unreachable!("production source never admits a callback")
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum EffectBoundary {
    LocalDurable,
    Dispatch,
    StrongExternalCommit,
}

/// Conformance protocol for a future qualified adapter. The production factory
/// cannot construct this context. Only the isolated test adapter currently can.
/// No serialization or public constructor may turn supplied P2a facts into it.
#[allow(dead_code)]
struct OrderedOperation<'a, S: OrderedSource, C: cockpit_core::clock::Clock> {
    source: &'a S,
    clock: &'a C,
    root: &'a Path,
    reference: cockpit_protocol::GoalAuthorityReference,
    action: crate::authority::GoalAuthorityActionFacts,
    budget: cockpit_core::clock::MonotonicBudget,
    stopped: bool,
}
#[allow(dead_code)]
impl<'a, S: OrderedSource, C: cockpit_core::clock::Clock> OrderedOperation<'a, S, C> {
    fn begin(
        source: &'a S,
        clock: &'a C,
        root: &'a Path,
        reference: Option<cockpit_protocol::GoalAuthorityReference>,
        action: crate::authority::GoalAuthorityActionFacts,
        command_limit_millis: u64,
    ) -> Result<Self, String> {
        source.require_enrollment(root)?;
        let reference=reference.ok_or_else(|| "goal_authority_reference_required: independent enabled-mode enrollment cannot be removed by editing a declaration".to_owned())?;
        let budget = source.with_current(&action.subject, |facts| {
            let sample = clock.sample().map_err(|error| error.to_string())?;
            let crate::authority::GoalAuthorityAssessment::EligibleUnderSuppliedFacts {
                effective_window,
                ..
            } = crate::authority::assess_goal_authority(facts, &reference, &action, sample)
                .map_err(|error| error.to_string())?;
            cockpit_core::clock::MonotonicBudget::new(
                effective_window,
                sample,
                command_limit_millis,
            )
            .map_err(|error| error.to_string())
        })?;
        Ok(Self {
            source,
            clock,
            root,
            reference,
            action,
            budget,
            stopped: false,
        })
    }
    /// Caller holds existing operation locks. Native policy is checked before
    /// the source fence, then facts/time/budget are refreshed inside the fence.
    /// A local fence does not freeze time or provide an external atomic commit.
    fn commit<T>(
        &mut self,
        boundary: EffectBoundary,
        native_gate: impl FnOnce() -> Result<(), String>,
        effect: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        if matches!(boundary, EffectBoundary::StrongExternalCommit) {
            return Err("goal_authority_external_commit_unsupported: a local fence cannot prove external atomic commit or freeze time".into());
        }
        if self.stopped {
            return Err("goal_authority_operation_stopped".into());
        }
        // No Native operation-lock acquisition is allowed while holding the
        // source fence. Any wait here is included in the next fresh sample.
        native_gate()?;
        self.source.require_enrollment(self.root)?;
        let result = self.source.with_current(&self.action.subject, |facts| {
            let sample = self.clock.sample().map_err(|error| error.to_string())?;
            let remaining = self
                .budget
                .observe(sample)
                .map_err(|error| error.to_string())?;
            if remaining == 0 {
                return Err(cockpit_core::clock::ClockError::Expired.to_string());
            }
            crate::authority::assess_goal_authority(facts, &self.reference, &self.action, sample)
                .map_err(|error| error.to_string())?;
            // This callback performs one bounded durable commit or dispatch.
            // It does not promise the completion time of an external effect.
            effect()
        });
        if result.is_err() {
            self.stopped = true;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn b01_internal_atomic_sink_cannot_bypass_the_entry_rejection() {
        let dir = tempfile::tempdir().unwrap();
        let active = dir.path().join(".ai/work-items/active");
        fs::create_dir_all(&active).unwrap();
        fs::write(
            active.join("WI-FIXTURE.contract.json"),
            br#"{"governanceProfile":{"goalAuthority":{"required":true}}}"#,
        )
        .unwrap();
        let path = active.join("WI-FIXTURE.summary.json");
        fs::write(&path, b"preserve original fixture bytes").unwrap();
        let result = crate::atomic_write(&path, b"must not commit");
        assert!(
            result.is_err(),
            "an internal sink must enforce the same boundary"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("goal_authority_source_unsupported")
        );
        assert_eq!(fs::read(&path).unwrap(), b"preserve original fixture bytes");
    }

    use super::{EffectBoundary, OrderedOperation, OrderedSource};
    use crate::authority::*;
    use cockpit_core::{
        Digest,
        clock::{Clock, ClockError, ClockSample},
    };
    use cockpit_protocol::GoalAuthorityReference;
    use serde_json::json;
    use std::{
        cell::Cell,
        fs::File,
        path::{Path, PathBuf},
    };
    const BASE: i64 = 1_893_456_000_000;
    fn sample(t: i64, m: u64) -> ClockSample {
        ClockSample {
            utc_millis: BASE + t,
            monotonic_millis: m,
        }
    }
    struct ControlledClock(Cell<Result<ClockSample, ClockError>>);
    impl Clock for ControlledClock {
        fn sample(&self) -> Result<ClockSample, ClockError> {
            self.0.get()
        }
    }
    fn reference() -> GoalAuthorityReference {
        serde_json::from_value(json!({"grantId":"fixture","subjectRef":"goal-a","versionResolution":"current","deliverableRefs":["part"],"deadline":{"mode":"inherit_source"}})).unwrap()
    }
    fn facts(revoked: bool) -> ResolvedGoalAuthorityFacts {
        let source:cockpit_protocol::GoalAuthoritySource=serde_json::from_value(json!({
            "schemaVersion":1,"repositoryId":Digest::sha256_bytes(b"isolated ordered-source fixture"),
            "grantId":"fixture","subjectRef":"goal-a","sourceVersion":1,"goal":"test only",
            "deliverableRefs":["part"],"actors":["agent:fixture"],"operations":["source.modify"],
            "resources":["repo:src/example.rs"],"constraints":[],
            "issuerEvidence":{"assurance":"self_declared","actor":"human:fixture","authoritySource":"test-owned adapter only","operations":["source.modify"],"policyRefs":[],"evidenceRefs":[]},
            "validity":{"notBefore":"2030-01-01T00:00:00Z","expiresAt":"2030-01-01T00:00:10Z"}
        })).unwrap();
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
            subject_state: if revoked {
                SuppliedGoalAuthorityState::Revoked
            } else {
                SuppliedGoalAuthorityState::Active
            },
            parent_chain_observation: SuppliedFactObservation::Observed,
            parent_chain: vec![],
        }
    }
    fn action(wi: &str) -> GoalAuthorityActionFacts {
        let source = facts(false).source;
        GoalAuthorityActionFacts {
            subject: GoalAuthoritySubjectKey {
                repository_id: source.repository_id,
                grant_id: source.grant_id,
                subject_ref: source.subject_ref,
            },
            actor: "agent:fixture".into(),
            operation: "source.modify".into(),
            resource: "repo:src/example.rs".into(),
            deliverable_refs: vec!["part".into()],
            work_item: GoalAuthorityNarrowing {
                work_item_id: wi.into(),
                parent_work_item_id: None,
                state: SuppliedGoalAuthorityState::Active,
                actors: vec!["agent:fixture".into()],
                operations: vec!["source.modify".into()],
                resources: vec!["repo:src/example.rs".into()],
                constraints: vec![],
            },
        }
    }
    /// Test-owned enrollment and real OS lock. Never compiled into production;
    /// the source/issuer claims below have no production authentication meaning.
    struct FixtureSource {
        directory: PathBuf,
        roots: Vec<PathBuf>,
        identity: fs::Metadata,
        fence_identity: fs::Metadata,
    }
    impl FixtureSource {
        fn new(directory: &Path, roots: Vec<PathBuf>) -> Self {
            Self {
                directory: directory.into(),
                roots,
                identity: fs::metadata(directory.join("source")).unwrap(),
                fence_identity: fs::metadata(directory.join("fence")).unwrap(),
            }
        }
        fn fence(&self) -> File {
            let f = File::options()
                .read(true)
                .write(true)
                .open(self.directory.join("fence"))
                .unwrap();
            f.lock().unwrap();
            f
        }
        fn valid_identity(&self) -> bool {
            fn matches(path: &Path, expected: &fs::Metadata) -> bool {
                let Ok(actual) = fs::symlink_metadata(path) else {
                    return false;
                };
                if actual.file_type().is_symlink() || !actual.is_file() {
                    return false;
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    actual.dev() == expected.dev() && actual.ino() == expected.ino()
                }
                #[cfg(not(unix))]
                {
                    match (actual.created(), expected.created()) {
                        (Ok(actual), Ok(expected)) => actual == expected,
                        _ => false,
                    }
                }
            }
            matches(&self.directory.join("source"), &self.identity)
                && matches(&self.directory.join("fence"), &self.fence_identity)
        }
    }
    impl OrderedSource for FixtureSource {
        fn require_enrollment(&self, root: &Path) -> Result<(), String> {
            if !self.roots.iter().any(|r| r == root) || !self.valid_identity() {
                return Err("independent enrollment/store identity mismatch".into());
            }
            Ok(())
        }
        fn with_current<T>(
            &self,
            key: &GoalAuthoritySubjectKey,
            operation: impl FnOnce(&ResolvedGoalAuthorityFacts) -> Result<T, String>,
        ) -> Result<T, String> {
            let _fence = self.fence();
            if !self.valid_identity() {
                return Err("source store identity mismatch".into());
            }
            let mut current =
                facts(fs::read_to_string(self.directory.join("source")).unwrap() == "revoked");
            let state = fs::read_to_string(self.directory.join("source")).unwrap();
            if state == "extended" {
                let previous = current.current_head.source_digest.clone();
                current.source.source_version = 2;
                current.source.predecessor_version_digest = Some(previous);
                current.source.validity.expires_at = "2030-01-01T00:00:20Z".into();
                current.version_history.push(current.source.clone());
                current.current_head.source_version = 2;
                current.current_head.source_digest =
                    normalize_goal_authority_source(&current.source)
                        .unwrap()
                        .source_digest;
            }
            if key != &action("WI-fixture").subject {
                return Err("authenticated fixture subject mismatch".into());
            }
            operation(&current)
        }
    }
    fn source_fixture() -> (tempfile::TempDir, FixtureSource, ControlledClock) {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("source"), "active").unwrap();
        fs::write(d.path().join("fence"), b"").unwrap();
        let source = FixtureSource::new(d.path(), vec![d.path().to_path_buf()]);
        (d, source, ControlledClock(Cell::new(Ok(sample(0, 0)))))
    }
    #[test]
    fn ordered_native_intersection_and_goal_scope_are_not_bypassed() {
        let (d, s, c) = source_fixture();
        let mut op = OrderedOperation::begin(
            &s,
            &c,
            d.path(),
            Some(reference()),
            action("WI-first"),
            900_000,
        )
        .unwrap();
        let effects = Cell::new(0);
        assert!(
            op.commit(
                EffectBoundary::LocalDurable,
                || Err("native policy denied".into()),
                || {
                    effects.set(1);
                    Ok(())
                }
            )
            .is_err()
        );
        assert_eq!(effects.get(), 0);
        for id in ["WI-first", "WI-derived", "WI-retry"] {
            let mut op =
                OrderedOperation::begin(&s, &c, d.path(), Some(reference()), action(id), 900_000)
                    .unwrap();
            op.commit(
                EffectBoundary::LocalDurable,
                || Ok(()),
                || {
                    effects.set(effects.get() + 1);
                    Ok(())
                },
            )
            .unwrap();
        }
        assert_eq!(effects.get(), 3);
        let mut wrong = action("WI-other");
        wrong.subject.subject_ref = "other-goal".into();
        assert!(
            OrderedOperation::begin(&s, &c, d.path(), Some(reference()), wrong, 900_000).is_err()
        );
        let mut wrong = action("WI-other");
        wrong.deliverable_refs = vec!["other-part".into()];
        assert!(
            OrderedOperation::begin(&s, &c, d.path(), Some(reference()), wrong, 900_000).is_err()
        );
    }
    #[test]
    fn ordered_waits_and_sampling_errors_cannot_spend_a_cached_positive_budget() {
        for next in [
            Ok(sample(10_000, 10_000)),
            Ok(sample(0, 10_000)),
            Ok(sample(-1, 1)),
            Err(ClockError::SamplingGap),
        ] {
            let (d, s, c) = source_fixture();
            let mut op = OrderedOperation::begin(
                &s,
                &c,
                d.path(),
                Some(reference()),
                action("WI-wait"),
                900_000,
            )
            .unwrap();
            let ran = Cell::new(false);
            assert!(
                op.commit(
                    EffectBoundary::Dispatch,
                    || {
                        c.0.set(next);
                        Ok(())
                    },
                    || {
                        ran.set(true);
                        Ok(())
                    }
                )
                .is_err()
            );
            assert!(!ran.get());
            c.0.set(Ok(sample(1, 1)));
            assert!(
                op.commit(
                    EffectBoundary::Dispatch,
                    || Ok(()),
                    || {
                        ran.set(true);
                        Ok(())
                    }
                )
                .is_err()
            );
            assert!(!ran.get());
        }
    }
    #[test]
    fn ordered_extension_never_refills_old_operation_or_cap() {
        let (d, s, c) = source_fixture();
        let mut old = OrderedOperation::begin(
            &s,
            &c,
            d.path(),
            Some(reference()),
            action("WI-old"),
            900_000,
        )
        .unwrap();
        {
            let _fence = s.fence();
            fs::write(d.path().join("source"), "extended").unwrap();
        }
        c.0.set(Ok(sample(10_000, 10_000)));
        let ran = Cell::new(false);
        assert!(
            old.commit(
                EffectBoundary::LocalDurable,
                || Ok(()),
                || {
                    ran.set(true);
                    Ok(())
                }
            )
            .is_err()
        );
        assert!(!ran.get());
        let mut fresh = OrderedOperation::begin(
            &s,
            &c,
            d.path(),
            Some(reference()),
            action("WI-new-action"),
            900_000,
        )
        .unwrap();
        fresh
            .commit(EffectBoundary::Dispatch, || Ok(()), || Ok(()))
            .unwrap();
        let mut capped = reference();
        capped.deadline = cockpit_protocol::GoalAuthorityDeadline::CapAt {
            expires_at: "2030-01-01T00:00:11Z".into(),
        };
        let mut cap =
            OrderedOperation::begin(&s, &c, d.path(), Some(capped), action("WI-cap"), 900_000)
                .unwrap();
        c.0.set(Ok(sample(11_000, 11_000)));
        assert!(
            cap.commit(EffectBoundary::LocalDurable, || Ok(()), || Ok(()))
                .is_err()
        );
    }
    #[test]
    fn ordered_enrollment_is_independent_and_external_strong_commit_is_unsupported() {
        let (d, s, c) = source_fixture();
        assert!(
            OrderedOperation::begin(&s, &c, d.path(), None, action("WI-no-ref"), 900_000).is_err()
        );
        assert!(
            OrderedOperation::begin(
                &s,
                &c,
                Path::new("/wrong-root"),
                Some(reference()),
                action("WI-root"),
                900_000
            )
            .is_err()
        );
        let mut op = OrderedOperation::begin(
            &s,
            &c,
            d.path(),
            Some(reference()),
            action("WI-effect"),
            900_000,
        )
        .unwrap();
        let ran = Cell::new(false);
        assert!(
            op.commit(
                EffectBoundary::StrongExternalCommit,
                || Ok(()),
                || {
                    ran.set(true);
                    Ok(())
                }
            )
            .is_err()
        );
        assert!(!ran.get());
        // Changing/removing mutable declarations cannot change the fixture's
        // independently pinned enrollment. No production anti-downgrade claim.
        fs::write(d.path().join("contract.json"), b"{}").unwrap();
        assert!(
            OrderedOperation::begin(&s, &c, d.path(), None, action("WI-no-ref"), 900_000).is_err()
        );
        fs::rename(d.path().join("source"), d.path().join("old-source")).unwrap();
        fs::write(d.path().join("source"), "active").unwrap();
        assert!(
            op.commit(EffectBoundary::LocalDurable, || Ok(()), || Ok(()))
                .is_err()
        );
    }
    #[test]
    fn ordered_replaced_fence_cannot_split_writer_consumer_ordering() {
        let (d, s, c) = source_fixture();
        let mut op = OrderedOperation::begin(
            &s,
            &c,
            d.path(),
            Some(reference()),
            action("WI-fence"),
            900_000,
        )
        .unwrap();
        fs::rename(d.path().join("fence"), d.path().join("old-fence")).unwrap();
        fs::write(d.path().join("fence"), b"").unwrap();
        let ran = Cell::new(false);
        assert!(
            op.commit(
                EffectBoundary::LocalDurable,
                || Ok(()),
                || {
                    ran.set(true);
                    Ok(())
                }
            )
            .is_err()
        );
        assert!(!ran.get());
    }

    #[cfg(unix)]
    #[test]
    fn ordered_process_fixture_child() {
        use std::io::{BufRead, Write};
        let Some(directory) = std::env::var_os("COCKPIT_TEST_ORDERED_SOURCE_DIR") else {
            return;
        };
        let d = PathBuf::from(directory);
        let root = d.join("checkout-b");
        let s = FixtureSource::new(&d, vec![d.join("checkout-a"), root.clone()]);
        let c = ControlledClock(Cell::new(Ok(sample(0, 0))));
        println!("SOURCE_FIXTURE_READY");
        std::io::stdout().flush().unwrap();
        let op = OrderedOperation::begin(
            &s,
            &c,
            &root,
            Some(reference()),
            action("WI-consumer"),
            900_000,
        );
        if std::env::var("COCKPIT_TEST_ORDERED_SOURCE_ORDER").unwrap() == "revoke-first" {
            assert!(op.is_err());
            assert!(!d.join("effect").exists());
            return;
        }
        let mut op = op.unwrap();
        op.commit(
            EffectBoundary::LocalDurable,
            || Ok(()),
            || {
                fs::write(d.join("effect.tmp"), b"one durable effect")
                    .map_err(|e| e.to_string())?;
                fs::rename(d.join("effect.tmp"), d.join("effect")).map_err(|e| e.to_string())
            },
        )
        .unwrap();
        println!("SOURCE_FIXTURE_COMMITTED");
        std::io::stdout().flush().unwrap();
        let mut next = String::new();
        std::io::stdin().lock().read_line(&mut next).unwrap();
        assert_eq!(next, "next\n");
        assert!(
            op.commit(
                EffectBoundary::LocalDurable,
                || Ok(()),
                || {
                    fs::write(d.join("unexpected-effect"), b"must not happen")
                        .map_err(|e| e.to_string())
                }
            )
            .is_err()
        );
        assert_eq!(fs::read(d.join("effect")).unwrap(), b"one durable effect");
        assert!(!d.join("unexpected-effect").exists());
    }
    #[cfg(unix)]
    #[test]
    fn ordered_cross_process_linked_checkout_commit_and_revocation_both_orders() {
        use std::io::{BufRead, BufReader, Write};
        use std::process::{Command, Stdio};
        fn marker(reader: &mut BufReader<std::process::ChildStdout>, wanted: &str) {
            loop {
                let mut line = String::new();
                assert!(
                    reader.read_line(&mut line).unwrap() > 0,
                    "child exited before {wanted}"
                );
                if line.contains(wanted) {
                    return;
                }
            }
        }
        for order in ["revoke-first", "effect-first"] {
            let (d, _, _) = source_fixture();
            let a = d.path().join("checkout-a");
            let b = d.path().join("checkout-b");
            fs::create_dir(&a).unwrap();
            for args in [
                vec!["init", "-q"],
                vec![
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "commit",
                    "--allow-empty",
                    "-qm",
                    "fixture",
                ],
            ] {
                assert!(
                    Command::new("git")
                        .args(args)
                        .current_dir(&a)
                        .status()
                        .unwrap()
                        .success()
                );
            }
            assert!(
                Command::new("git")
                    .args(["worktree", "add", "-q", "-b", "fixture-other"])
                    .arg(&b)
                    .current_dir(&a)
                    .status()
                    .unwrap()
                    .success()
            );
            let ga = cockpit_git::GitRepository::discover(&a)
                .unwrap()
                .topology()
                .unwrap();
            let gb = cockpit_git::GitRepository::discover(&b)
                .unwrap()
                .topology()
                .unwrap();
            assert_eq!(
                ga.common_dir.canonicalize().unwrap(),
                gb.common_dir.canonicalize().unwrap()
            );
            let s = FixtureSource::new(d.path(), vec![a, b]);
            let held = if order == "revoke-first" {
                Some(s.fence())
            } else {
                None
            };
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "authority_admission::tests::ordered_process_fixture_child",
                    "--nocapture",
                    "--test-threads=1",
                ])
                .env("COCKPIT_TEST_ORDERED_SOURCE_DIR", d.path())
                .env("COCKPIT_TEST_ORDERED_SOURCE_ORDER", order)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap();
            let mut reader = BufReader::new(child.stdout.take().unwrap());
            marker(&mut reader, "SOURCE_FIXTURE_READY");
            if order == "revoke-first" {
                fs::write(d.path().join("source"), "revoked").unwrap();
                drop(held);
            } else {
                marker(&mut reader, "SOURCE_FIXTURE_COMMITTED");
                let _fence = s.fence();
                fs::write(d.path().join("source"), "revoked").unwrap();
                drop(_fence);
                child.stdin.take().unwrap().write_all(b"next\n").unwrap();
            }
            assert!(child.wait().unwrap().success());
            assert_eq!(d.path().join("effect").exists(), order == "effect-first");
            assert!(!d.path().join("unexpected-effect").exists());
        }
    }
}
