use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::{
    COLLABORATION_SCHEMA_VERSION, CoordinationEvent, CoordinationRecovery, CoordinationRequest,
    CoordinationRequestState, ResourceClaimMode, ResourceReservation, RuntimeCapabilityBinding,
    WorktreeRegistration,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;

const LOCK_WAIT: Duration = Duration::from_millis(5);
const LOCK_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Error)]
pub enum CoordinationError {
    #[error(transparent)]
    Runtime(#[from] cockpit_protocol::RuntimeCapabilityError),
    #[error("coordination store I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("coordination record at {path} is malformed: {reason}")]
    InvalidRecord { path: PathBuf, reason: String },
    #[error("coordination record identity already exists with different content: {0}")]
    DuplicateIdentity(String),
    #[error(
        "coordination record is stale for {work_item_id}: expected generation {expected}, got {actual}"
    )]
    StaleGeneration {
        work_item_id: String,
        expected: u64,
        actual: u64,
    },
    #[error("coordination resource is already owned: {0}")]
    ResourceConflict(String),
    #[error("coordination resource identity is invalid: {0}")]
    InvalidResource(String),
    #[error("coordination record requires recovery: {0}")]
    RecoveryRequired(String),
    #[error("coordination request transition is invalid: {from:?} -> {to:?}")]
    InvalidRequestTransition {
        from: CoordinationRequestState,
        to: CoordinationRequestState,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoordinationInspection {
    pub registrations: Vec<WorktreeRegistration>,
    pub events: Vec<CoordinationEvent>,
    pub reservations: Vec<ResourceReservation>,
    pub requests: Vec<CoordinationRequest>,
    pub recoveries: Vec<CoordinationRecovery>,
    pub unknowns: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryReport {
    pub unknowns: Vec<String>,
}

struct ObservedEventEvidence {
    digests: BTreeMap<String, Digest>,
    bytes: BTreeMap<String, Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct CoordinationStore {
    root: PathBuf,
    common_dir: PathBuf,
    runtime: RuntimeCapabilityBinding,
}

impl CoordinationStore {
    pub fn open(
        repository: &GitRepository,
        runtime: RuntimeCapabilityBinding,
    ) -> Result<Self, CoordinationError> {
        let store = Self::open_read_only(repository, runtime)?;
        for directory in [
            "registrations",
            "registration-history",
            "events",
            "reservations",
            "requests",
            "recoveries",
        ] {
            fs::create_dir_all(store.root.join(directory)).map_err(|source| {
                CoordinationError::Io {
                    path: store.root.join(directory),
                    source,
                }
            })?;
        }
        Ok(store)
    }

    pub fn open_read_only(
        repository: &GitRepository,
        runtime: RuntimeCapabilityBinding,
    ) -> Result<Self, CoordinationError> {
        runtime.validate_candidate()?;
        let topology = repository
            .topology()
            .map_err(|source| CoordinationError::Io {
                path: repository.root().to_path_buf(),
                source: std::io::Error::other(source.to_string()),
            })?;
        let root = topology.common_dir.join(".ai-cockpit/coordination/v1");
        Ok(Self {
            root,
            common_dir: topology.common_dir,
            runtime,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn git_common_dir(&self) -> &Path {
        &self.common_dir
    }

    pub fn registration_path(&self, work_item_id: &str) -> PathBuf {
        self.root
            .join("registrations")
            .join(format!("{work_item_id}.json"))
    }

    pub fn register(
        &self,
        registration: WorktreeRegistration,
    ) -> Result<WorktreeRegistration, CoordinationError> {
        self.register_with_contract_reader(registration, |root, reference| {
            crate::collaboration::read_registered_worktree_file(root, reference)
        })
    }

    fn register_with_contract_reader<F>(
        &self,
        registration: WorktreeRegistration,
        mut read_contract: F,
    ) -> Result<WorktreeRegistration, CoordinationError>
    where
        F: FnMut(&Path, &str) -> Result<Vec<u8>, String>,
    {
        self.runtime.validate_candidate()?;
        validate_registration(&registration)?;
        if !registration.runtime.same_identity(&self.runtime) {
            return Err(CoordinationError::Runtime(
                cockpit_protocol::RuntimeCapabilityError::IdentityMismatch,
            ));
        }
        self.validate_registration_facts_with_contract_reader(&registration, &mut read_contract)?;
        self.with_lock(|| {
            let path = self.registration_path(&registration.work_item_id);
            let mut identity_changed = false;
            let mut invalidated_outcome_ids = BTreeSet::new();
            if path.exists() {
                let existing: WorktreeRegistration = self.read_json(&path)?;
                if existing == registration {
                    self.persist_registration_snapshot(&registration)?;
                    return Ok(existing);
                }
                if existing.generation >= registration.generation {
                    return Err(CoordinationError::StaleGeneration {
                        work_item_id: registration.work_item_id.clone(),
                        expected: existing.generation,
                        actual: registration.generation,
                    });
                }
                identity_changed = existing.head != registration.head
                    || existing.branch != registration.branch
                    || existing.contract_digest != registration.contract_digest
                    || existing.declaration != registration.declaration;
                if identity_changed {
                    invalidated_outcome_ids.extend(
                        existing
                            .declaration
                            .provided_outcomes
                            .iter()
                            .map(|outcome| outcome.outcome_id.clone()),
                    );
                    invalidated_outcome_ids.extend(
                        registration
                            .declaration
                            .provided_outcomes
                            .iter()
                            .map(|outcome| outcome.outcome_id.clone()),
                    );
                }
            }
            self.persist_registration_snapshot(&registration)?;
            if identity_changed {
                let event = CoordinationEvent {
                    schema_version: COLLABORATION_SCHEMA_VERSION,
                    event_id: format!(
                        "auto-impact-{}-{}",
                        registration.work_item_id, registration.generation
                    ),
                    repository_id: registration.repository_id.clone(),
                    work_item_id: registration.work_item_id.clone(),
                    generation: registration.generation,
                    kind: cockpit_protocol::CoordinationEventKind::Impact,
                    source: "registration-identity-changed".into(),
                    evidence_refs: Vec::new(),
                    evidence_digests: BTreeMap::new(),
                    outcome_ids: invalidated_outcome_ids.into_iter().collect(),
                };
                let event_path = self
                    .root
                    .join("events")
                    .join(format!("{}.json", event.event_id));
                if event_path.exists() {
                    let existing: CoordinationEvent = self.read_json(&event_path)?;
                    if existing != event {
                        return Err(CoordinationError::DuplicateIdentity(event.event_id));
                    }
                } else {
                    self.atomic_write(&event_path, &event)?;
                }
            }
            // Publish invalidation before making the new registration
            // authoritative. If writing the event fails, the previous
            // registration remains current. If the process stops after the
            // event is persisted, a retry observes the older registration,
            // validates the same deterministic event, and then completes the
            // registration write. This avoids an unrecoverable gap where an
            // identical retry returned early after replacing the facts but
            // before persisting their impact.
            self.atomic_write(&path, &registration)?;
            Ok(registration)
        })
    }

    pub fn publish_event(
        &self,
        event: CoordinationEvent,
    ) -> Result<CoordinationEvent, CoordinationError> {
        if event.kind == cockpit_protocol::CoordinationEventKind::OutcomePublished {
            return Err(CoordinationError::RecoveryRequired(
                "OutcomePublished events must use the typed publish_outcome entry point".into(),
            ));
        }
        self.publish_event_internal(event)
    }

    pub(crate) fn publish_validated_outcome_event<F>(
        &self,
        mut event: CoordinationEvent,
        validate_publication: F,
    ) -> Result<CoordinationEvent, CoordinationError>
    where
        F: FnOnce(
            &CoordinationEvent,
            &CoordinationInspection,
            &WorktreeRegistration,
            &BTreeMap<String, Vec<u8>>,
        ) -> Result<(), CoordinationError>,
    {
        if event.kind != cockpit_protocol::CoordinationEventKind::OutcomePublished {
            return Err(CoordinationError::RecoveryRequired(
                "typed Outcome publication accepts only OutcomePublished events".into(),
            ));
        }
        self.runtime.validate_candidate()?;
        validate_event(&event)?;
        self.with_lock(|| {
            // Registration writers use the same exclusive lock. Re-read the
            // current complete registration projection while holding it, so
            // the verification requirement and event append share one
            // linearization point.
            let inspection = self.inspect()?;
            if !inspection.unknowns.is_empty() {
                return Err(CoordinationError::RecoveryRequired(format!(
                    "cannot publish an outcome while coordination facts are unknown: {}",
                    inspection.unknowns.join(", ")
                )));
            }
            let registration = inspection
                .registrations
                .iter()
                .find(|registration| registration.work_item_id == event.work_item_id)
                .ok_or_else(|| {
                    CoordinationError::RecoveryRequired(format!(
                        "outcome publisher requires a current registration for {}",
                        event.work_item_id
                    ))
                })?;
            if registration.repository_id != event.repository_id {
                return Err(CoordinationError::RecoveryRequired(
                    "event repository identity differs from registration".into(),
                ));
            }
            if registration.generation != event.generation {
                return Err(CoordinationError::StaleGeneration {
                    work_item_id: event.work_item_id.clone(),
                    expected: registration.generation,
                    actual: event.generation,
                });
            }
            let observed = self.observed_event_evidence_bytes(&event, registration)?;
            if event.evidence_digests != observed.digests {
                return Err(CoordinationError::RecoveryRequired(
                    "event evidence digest does not match current file bytes".into(),
                ));
            }
            event.evidence_digests = observed.digests;
            validate_publication(&event, &inspection, registration, &observed.bytes)?;
            self.append_event_under_lock(event)
        })
    }

    fn publish_event_internal(
        &self,
        mut event: CoordinationEvent,
    ) -> Result<CoordinationEvent, CoordinationError> {
        self.runtime.validate_candidate()?;
        validate_event(&event)?;
        self.with_lock(|| {
            let registration_path = self.registration_path(&event.work_item_id);
            let registration: WorktreeRegistration = self.read_json(&registration_path)?;
            self.validate_registration_facts(&registration)?;
            if registration.repository_id != event.repository_id {
                return Err(CoordinationError::RecoveryRequired(
                    "event repository identity differs from registration".into(),
                ));
            }
            if registration.generation != event.generation {
                return Err(CoordinationError::StaleGeneration {
                    work_item_id: event.work_item_id.clone(),
                    expected: registration.generation,
                    actual: event.generation,
                });
            }
            let observed_digests = self.observed_event_evidence_digests(&event, &registration)?;
            if !event.evidence_digests.is_empty() && event.evidence_digests != observed_digests {
                return Err(CoordinationError::RecoveryRequired(
                    "event evidence digest does not match current file bytes".into(),
                ));
            }
            event.evidence_digests = observed_digests;
            self.append_event_under_lock(event)
        })
    }

    fn append_event_under_lock(
        &self,
        event: CoordinationEvent,
    ) -> Result<CoordinationEvent, CoordinationError> {
        let path = self
            .root
            .join("events")
            .join(format!("{}.json", event.event_id));
        if path.exists() {
            let existing: CoordinationEvent = self.read_json(&path)?;
            if existing == event {
                return Ok(existing);
            }
            return Err(CoordinationError::DuplicateIdentity(event.event_id.clone()));
        }
        self.atomic_write(&path, &event)?;
        Ok(event)
    }

    pub fn reserve_resources(
        &self,
        reservation: ResourceReservation,
    ) -> Result<ResourceReservation, CoordinationError> {
        self.runtime.validate_candidate()?;
        validate_reservation(&reservation)?;
        self.with_lock(|| {
            let registration: WorktreeRegistration =
                self.read_json(&self.registration_path(&reservation.work_item_id))?;
            self.validate_registration_facts(&registration)?;
            if registration.repository_id != reservation.repository_id {
                return Err(CoordinationError::RecoveryRequired(
                    "reservation repository identity differs from registration".into(),
                ));
            }
            if registration.generation != reservation.generation {
                return Err(CoordinationError::StaleGeneration {
                    work_item_id: reservation.work_item_id.clone(),
                    expected: registration.generation,
                    actual: reservation.generation,
                });
            }
            let path = self
                .root
                .join("reservations")
                .join(format!("{}.json", reservation.reservation_id));
            if path.exists() {
                let existing: ResourceReservation = self.read_json(&path)?;
                if existing == reservation {
                    return Ok(existing);
                }
                return Err(CoordinationError::DuplicateIdentity(
                    reservation.reservation_id.clone(),
                ));
            }
            let existing = self.read_reservations()?;
            for candidate in &existing {
                for requested in &reservation.resources {
                    for owned in &candidate.resources {
                        if requested.resource_id == owned.resource_id
                            && (requested.mode == ResourceClaimMode::Exclusive
                                || owned.mode == ResourceClaimMode::Exclusive
                                || requested.serial
                                || owned.serial)
                        {
                            return Err(CoordinationError::ResourceConflict(
                                requested.resource_id.clone(),
                            ));
                        }
                    }
                }
            }
            // All resources are checked before the one atomic record is
            // published. A failed multi-resource acquisition therefore never
            // leaves a partial owner behind.
            self.atomic_write(&path, &reservation)?;
            Ok(reservation)
        })
    }

    pub fn release_resources(
        &self,
        reservation_id: &str,
        work_item_id: &str,
        generation: u64,
    ) -> Result<(), CoordinationError> {
        self.runtime.validate_candidate()?;
        self.with_lock(|| {
            let path = self
                .root
                .join("reservations")
                .join(format!("{reservation_id}.json"));
            let reservation: ResourceReservation = self.read_json(&path)?;
            let registration: WorktreeRegistration =
                self.read_json(&self.registration_path(work_item_id))?;
            self.validate_registration_facts(&registration)?;
            if registration.generation != generation {
                return Err(CoordinationError::StaleGeneration {
                    work_item_id: work_item_id.into(),
                    expected: registration.generation,
                    actual: generation,
                });
            }
            if reservation.repository_id != registration.repository_id
                || reservation.work_item_id != work_item_id
                || reservation.generation != generation
            {
                return Err(CoordinationError::StaleGeneration {
                    work_item_id: work_item_id.into(),
                    expected: registration.generation,
                    actual: generation,
                });
            }
            fs::remove_file(&path).map_err(|source| CoordinationError::Io { path, source })
        })
    }

    pub fn request_coordination(
        &self,
        request: CoordinationRequest,
    ) -> Result<CoordinationRequest, CoordinationError> {
        self.runtime.validate_candidate()?;
        if request.schema_version != COLLABORATION_SCHEMA_VERSION {
            return Err(CoordinationError::RecoveryRequired(
                "unsupported coordination request schema".into(),
            ));
        }
        if !valid_component(&request.request_id)
            || !valid_component(&request.target_work_item_id)
            || request.target_generation == 0
        {
            return Err(CoordinationError::RecoveryRequired(
                "invalid coordination request identity".into(),
            ));
        }
        if request.state != CoordinationRequestState::Requested {
            return Err(CoordinationError::RecoveryRequired(
                "coordination requests must start in Requested state".into(),
            ));
        }
        self.with_lock(|| {
            let registration_path = self.registration_path(&request.target_work_item_id);
            let registration: WorktreeRegistration = self.read_json(&registration_path)?;
            if registration.work_item_id != request.target_work_item_id {
                return Err(CoordinationError::RecoveryRequired(
                    "coordination request target differs from registration identity".into(),
                ));
            }
            self.validate_registration_facts(&registration)?;
            if registration.repository_id != request.repository_id {
                return Err(CoordinationError::RecoveryRequired(
                    "coordination request repository identity differs from registration".into(),
                ));
            }
            if registration.generation != request.target_generation {
                return Err(CoordinationError::StaleGeneration {
                    work_item_id: request.target_work_item_id.clone(),
                    expected: registration.generation,
                    actual: request.target_generation,
                });
            }
            let path = self
                .root
                .join("requests")
                .join(format!("{}.json", request.request_id));
            if path.exists() {
                let existing: CoordinationRequest = self.read_json(&path)?;
                if existing == request {
                    return Ok(existing);
                }
                return Err(CoordinationError::DuplicateIdentity(
                    request.request_id.clone(),
                ));
            }
            self.atomic_write(&path, &request)?;
            Ok(request)
        })
    }

    pub fn transition_request(
        &self,
        request_id: &str,
        state: CoordinationRequestState,
    ) -> Result<CoordinationRequest, CoordinationError> {
        self.runtime.validate_candidate()?;
        if !valid_component(request_id) {
            return Err(CoordinationError::RecoveryRequired(
                "invalid coordination request identity".into(),
            ));
        }
        self.with_lock(|| {
            let path = self
                .root
                .join("requests")
                .join(format!("{request_id}.json"));
            let mut request: CoordinationRequest = self.read_json(&path)?;
            if request.request_id != request_id {
                return Err(CoordinationError::RecoveryRequired(
                    "coordination request ID differs from stored record identity".into(),
                ));
            }
            if !valid_component(&request.target_work_item_id) {
                return Err(CoordinationError::RecoveryRequired(
                    "invalid coordination request target identity".into(),
                ));
            }
            let registration: WorktreeRegistration =
                self.read_json(&self.registration_path(&request.target_work_item_id))?;
            if registration.work_item_id != request.target_work_item_id {
                return Err(CoordinationError::RecoveryRequired(
                    "coordination request target differs from registration identity".into(),
                ));
            }
            self.validate_registration_facts(&registration)?;
            if registration.repository_id != request.repository_id {
                return Err(CoordinationError::RecoveryRequired(
                    "coordination request repository identity differs from registration".into(),
                ));
            }
            if registration.generation != request.target_generation {
                return Err(CoordinationError::StaleGeneration {
                    work_item_id: request.target_work_item_id.clone(),
                    expected: registration.generation,
                    actual: request.target_generation,
                });
            }
            if !valid_request_transition(request.state, state) {
                return Err(CoordinationError::InvalidRequestTransition {
                    from: request.state,
                    to: state,
                });
            }
            request.state = state;
            self.atomic_write(&path, &request)?;
            Ok(request)
        })
    }

    pub fn recover_event(
        &self,
        event_id: &str,
        consumer_work_item_id: &str,
        consumer_generation: u64,
    ) -> Result<CoordinationRecovery, CoordinationError> {
        self.runtime.validate_candidate()?;
        if !valid_component(event_id)
            || !valid_component(consumer_work_item_id)
            || consumer_generation == 0
        {
            return Err(CoordinationError::RecoveryRequired(
                "invalid recovery consumption identity".into(),
            ));
        }
        self.with_lock(|| {
            let event_path = self.root.join("events").join(format!("{event_id}.json"));
            let event: CoordinationEvent = self.read_json(&event_path)?;
            let provider_registration: WorktreeRegistration =
                self.read_json(&self.registration_path(&event.work_item_id))?;
            self.validate_registration_facts(&provider_registration)?;
            if provider_registration.generation < event.generation {
                return Err(CoordinationError::StaleGeneration {
                    work_item_id: event.work_item_id.clone(),
                    expected: provider_registration.generation,
                    actual: event.generation,
                });
            }
            if event.kind == cockpit_protocol::CoordinationEventKind::OutcomePublished {
                return Err(CoordinationError::RecoveryRequired(
                    "published outcomes do not require invalidation recovery".into(),
                ));
            }
            let consumer_registration: WorktreeRegistration =
                self.read_json(&self.registration_path(consumer_work_item_id))?;
            self.validate_registration_facts(&consumer_registration)?;
            if consumer_registration.generation != consumer_generation {
                return Err(CoordinationError::StaleGeneration {
                    work_item_id: consumer_work_item_id.into(),
                    expected: consumer_registration.generation,
                    actual: consumer_generation,
                });
            }
            if event.repository_id != consumer_registration.repository_id {
                return Err(CoordinationError::RecoveryRequired(
                    "event repository identity differs from consumer registration".into(),
                ));
            }
            let consumption_id =
                format!("recovery-{event_id}-{consumer_work_item_id}-{consumer_generation}");
            let recovery = CoordinationRecovery {
                schema_version: COLLABORATION_SCHEMA_VERSION,
                consumption_id: consumption_id.clone(),
                repository_id: event.repository_id,
                event_id: event.event_id,
                provider_work_item_id: event.work_item_id,
                provider_generation: event.generation,
                current_provider_generation: Some(provider_registration.generation),
                current_provider_head: Some(provider_registration.head),
                current_provider_contract_digest: Some(provider_registration.contract_digest),
                consumer_work_item_id: consumer_work_item_id.into(),
                consumer_generation,
            };
            let path = self
                .root
                .join("recoveries")
                .join(format!("{consumption_id}.json"));
            if path.exists() {
                let existing: CoordinationRecovery = self.read_json(&path)?;
                if existing == recovery {
                    return Ok(existing);
                }
                return Err(CoordinationError::DuplicateIdentity(consumption_id));
            }
            self.atomic_write(&path, &recovery)?;
            Ok(recovery)
        })
    }

    pub fn inspect(&self) -> Result<CoordinationInspection, CoordinationError> {
        self.runtime.validate_candidate()?;
        let mut inspection = CoordinationInspection {
            registrations: Vec::new(),
            events: Vec::new(),
            reservations: Vec::new(),
            requests: Vec::new(),
            recoveries: Vec::new(),
            unknowns: Vec::new(),
        };
        self.read_directory("registrations", &mut inspection.unknowns, |path| {
            let registration: WorktreeRegistration = self.read_json(path)?;
            self.validate_registration_facts(&registration)?;
            inspection.registrations.push(registration);
            Ok(())
        })?;
        self.read_directory("events", &mut inspection.unknowns, |path| {
            let event: CoordinationEvent = self.read_json(path)?;
            let registration: WorktreeRegistration =
                self.read_json(&self.registration_path(&event.work_item_id))?;
            self.validate_registration_facts(&registration)?;
            self.validate_event_facts(&event, &registration)?;
            inspection.events.push(event);
            Ok(())
        })?;
        self.read_directory("reservations", &mut inspection.unknowns, |path| {
            inspection.reservations.push(self.read_json(path)?);
            Ok(())
        })?;
        self.read_directory("requests", &mut inspection.unknowns, |path| {
            inspection.requests.push(self.read_json(path)?);
            Ok(())
        })?;
        self.read_directory("recoveries", &mut inspection.unknowns, |path| {
            let recovery: CoordinationRecovery = self.read_json(path)?;
            self.validate_recovery_facts(path, &recovery)?;
            inspection.recoveries.push(recovery);
            Ok(())
        })?;
        inspection
            .registrations
            .sort_by(|a, b| a.work_item_id.cmp(&b.work_item_id));
        inspection
            .events
            .sort_by(|a, b| a.event_id.cmp(&b.event_id));
        inspection
            .reservations
            .sort_by(|a, b| a.reservation_id.cmp(&b.reservation_id));
        inspection
            .requests
            .sort_by(|a, b| a.request_id.cmp(&b.request_id));
        inspection
            .recoveries
            .sort_by(|a, b| a.consumption_id.cmp(&b.consumption_id));
        Ok(inspection)
    }

    pub fn recover(&self) -> Result<RecoveryReport, CoordinationError> {
        let inspection = self.inspect()?;
        Ok(RecoveryReport {
            unknowns: inspection.unknowns,
        })
    }

    /// Bind a caller-supplied registration to facts observed from Git and the
    /// active Contract before any coordination record can become authoritative
    /// for a later action.  This is deliberately used by both writes and
    /// inspection: a read reports moved or stale state instead of treating the
    /// declaration as current.
    fn validate_registration_facts(
        &self,
        registration: &WorktreeRegistration,
    ) -> Result<(), CoordinationError> {
        self.validate_registration_facts_with_contract_reader(
            registration,
            &mut |root, reference| {
                crate::collaboration::read_registered_worktree_file(root, reference)
            },
        )
    }

    fn validate_registration_facts_with_contract_reader<F>(
        &self,
        registration: &WorktreeRegistration,
        read_contract: &mut F,
    ) -> Result<(), CoordinationError>
    where
        F: FnMut(&Path, &str) -> Result<Vec<u8>, String>,
    {
        validate_registration(registration)?;
        let worktree = Path::new(&registration.worktree_path);
        let canonical_worktree = fs::canonicalize(worktree).map_err(|source| {
            CoordinationError::RecoveryRequired(format!(
                "worktree moved or deleted: {} ({source})",
                registration.worktree_path
            ))
        })?;
        let git = GitRepository::discover(&canonical_worktree).map_err(|source| {
            CoordinationError::RecoveryRequired(format!(
                "registered worktree is not a readable repository: {} ({source})",
                registration.worktree_path
            ))
        })?;
        let topology = git.topology().map_err(|source| {
            CoordinationError::RecoveryRequired(format!(
                "registered worktree topology is unavailable: {} ({source})",
                registration.worktree_path
            ))
        })?;
        if topology.common_dir != self.common_dir {
            return Err(CoordinationError::RecoveryRequired(format!(
                "registered worktree is not in this Git common directory: {}",
                registration.worktree_path
            )));
        }
        if topology.repository_root != canonical_worktree {
            return Err(CoordinationError::RecoveryRequired(format!(
                "registered worktree path does not resolve to its repository root: {}",
                registration.worktree_path
            )));
        }
        if registration.repository_id != crate::repository_id(&topology.repository_root) {
            return Err(CoordinationError::RecoveryRequired(format!(
                "repository identity mismatch for {}",
                registration.work_item_id
            )));
        }
        if registration.worktree_path != canonical_worktree.to_string_lossy() {
            return Err(CoordinationError::RecoveryRequired(format!(
                "worktree path is not canonical for {}",
                registration.work_item_id
            )));
        }
        if topology.branch.as_deref() != Some(registration.branch.as_str()) {
            return Err(CoordinationError::RecoveryRequired(format!(
                "branch mismatch for {}: declared {}, observed {:?}",
                registration.work_item_id, registration.branch, topology.branch
            )));
        }
        if topology.head.as_deref() != Some(registration.head.as_str()) {
            return Err(CoordinationError::RecoveryRequired(format!(
                "head mismatch for {}: declared {}, observed {:?}",
                registration.work_item_id, registration.head, topology.head
            )));
        }

        let contract_path = topology
            .repository_root
            .join(".ai/work-items/active")
            .join(format!("{}.contract.json", registration.work_item_id));
        let contract_reference = format!(
            ".ai/work-items/active/{}.contract.json",
            registration.work_item_id
        );
        let contract_bytes = read_contract(&topology.repository_root, &contract_reference)
            .map_err(|error| {
                CoordinationError::RecoveryRequired(format!(
                    "active Contract is not safely contained for {}: {error}",
                    registration.work_item_id
                ))
            })?;
        let contract_json: serde_json::Value =
            serde_json::from_slice(&contract_bytes).map_err(|error| {
                CoordinationError::RecoveryRequired(format!(
                    "active Contract digest is unavailable for {}: {error}",
                    registration.work_item_id
                ))
            })?;
        let actual_contract_digest =
            cockpit_protocol::digest_json(&contract_json).map_err(|error| {
                CoordinationError::RecoveryRequired(format!(
                    "active Contract digest is unavailable for {}: {error}",
                    registration.work_item_id
                ))
            })?;
        if actual_contract_digest != registration.contract_digest {
            return Err(CoordinationError::RecoveryRequired(format!(
                "Contract digest mismatch for {}",
                registration.work_item_id
            )));
        }
        let contract =
            crate::parse_contract_bytes(&contract_bytes, &contract_path).map_err(|error| {
                CoordinationError::RecoveryRequired(format!(
                    "active Contract is invalid for {}: {error}",
                    registration.work_item_id
                ))
            })?;
        validate_contract_registration_identity(registration, &contract)?;
        for outcome in &registration.declaration.provided_outcomes {
            if outcome.published_head != registration.head {
                return Err(CoordinationError::RecoveryRequired(format!(
                    "provided outcome {} is not bound to the registered head",
                    outcome.outcome_id
                )));
            }
            for evidence_ref in &outcome.evidence_refs {
                crate::collaboration::open_registered_worktree_file(
                    &topology.repository_root,
                    evidence_ref,
                )
                .map_err(|error| {
                    CoordinationError::RecoveryRequired(format!(
                        "required outcome evidence is not safely readable at {}: {error}",
                        evidence_ref
                    ))
                })?;
            }
        }
        Ok(())
    }

    fn validate_event_facts(
        &self,
        event: &CoordinationEvent,
        registration: &WorktreeRegistration,
    ) -> Result<(), CoordinationError> {
        if event.repository_id != registration.repository_id
            || event.generation > registration.generation
        {
            return Err(CoordinationError::RecoveryRequired(
                "event repository or generation does not match its provider registration".into(),
            ));
        }
        if !event.evidence_digests.is_empty() {
            let references = event.evidence_refs.iter().collect::<BTreeSet<_>>();
            let digested_references = event.evidence_digests.keys().collect::<BTreeSet<_>>();
            if references != digested_references {
                return Err(CoordinationError::RecoveryRequired(
                    "event evidence digest references do not match evidenceRefs".into(),
                ));
            }
        }
        let automatic_identity_invalidation = event.kind
            == cockpit_protocol::CoordinationEventKind::Impact
            && event.source == "registration-identity-changed"
            && event.event_id == format!("auto-impact-{}-{}", event.work_item_id, event.generation);
        if event.generation == registration.generation
            && !automatic_identity_invalidation
            && event.outcome_ids.iter().any(|outcome_id| {
                !registration
                    .declaration
                    .provided_outcomes
                    .iter()
                    .any(|outcome| outcome.outcome_id == *outcome_id)
            })
        {
            return Err(CoordinationError::RecoveryRequired(
                "event outcome identity is not declared by the registered provider".into(),
            ));
        }
        for reference in &event.evidence_refs {
            let relative = Path::new(reference);
            if relative.is_absolute()
                || relative
                    .components()
                    .any(|component| component == std::path::Component::ParentDir)
            {
                return Err(CoordinationError::RecoveryRequired(format!(
                    "event evidence reference escapes registered worktree: {reference}"
                )));
            }
            crate::collaboration::open_registered_worktree_file(
                Path::new(&registration.worktree_path),
                reference,
            )
            .map_err(|error| {
                CoordinationError::RecoveryRequired(format!(
                    "event evidence is not safely readable for {} at {}: {error}",
                    event.event_id, reference
                ))
            })?;
        }
        Ok(())
    }

    fn validate_recovery_facts(
        &self,
        path: &Path,
        recovery: &CoordinationRecovery,
    ) -> Result<(), CoordinationError> {
        validate_recovery_identity(recovery)?;
        if path.file_stem().and_then(|stem| stem.to_str()) != Some(recovery.consumption_id.as_str())
        {
            return Err(CoordinationError::RecoveryRequired(
                "recovery filename does not match its consumption identity".into(),
            ));
        }
        let expected_consumption_id = format!(
            "recovery-{}-{}-{}",
            recovery.event_id, recovery.consumer_work_item_id, recovery.consumer_generation
        );
        if recovery.consumption_id != expected_consumption_id {
            return Err(CoordinationError::RecoveryRequired(
                "recovery consumption identity does not match its event and consumer".into(),
            ));
        }

        let event_path = self
            .root
            .join("events")
            .join(format!("{}.json", recovery.event_id));
        let event: CoordinationEvent = self.read_json(&event_path)?;
        validate_event(&event)?;
        let provider_path = self.registration_path(&recovery.provider_work_item_id);
        let provider: WorktreeRegistration = self.read_json(&provider_path)?;
        self.validate_registration_facts(&provider)?;
        self.validate_event_facts(&event, &provider)?;
        if event.event_id != recovery.event_id
            || event.repository_id != recovery.repository_id
            || provider.repository_id != recovery.repository_id
            || event.work_item_id != recovery.provider_work_item_id
            || event.generation != recovery.provider_generation
            || event.kind == cockpit_protocol::CoordinationEventKind::OutcomePublished
        {
            return Err(CoordinationError::RecoveryRequired(
                "recovery does not match its invalidation event and provider".into(),
            ));
        }

        let consumer_path = self.registration_path(&recovery.consumer_work_item_id);
        let consumer: WorktreeRegistration = self.read_json(&consumer_path)?;
        self.validate_registration_facts(&consumer)?;
        if consumer.repository_id != recovery.repository_id
            || recovery.consumer_generation > consumer.generation
        {
            return Err(CoordinationError::RecoveryRequired(
                "recovery consumer identity or generation is inconsistent".into(),
            ));
        }

        let Some(observed_provider_generation) = recovery.current_provider_generation else {
            return Err(CoordinationError::RecoveryRequired(
                "recovery is missing its observed provider generation".into(),
            ));
        };
        let Some(observed_provider_head) = recovery.current_provider_head.as_deref() else {
            return Err(CoordinationError::RecoveryRequired(
                "recovery is missing its observed provider head".into(),
            ));
        };
        let Some(observed_contract_digest) = recovery.current_provider_contract_digest.as_ref()
        else {
            return Err(CoordinationError::RecoveryRequired(
                "recovery is missing its observed provider Contract digest".into(),
            ));
        };
        if observed_provider_generation < event.generation
            || observed_provider_generation > provider.generation
            || !valid_component(observed_provider_head)
        {
            return Err(CoordinationError::RecoveryRequired(
                "recovery provider snapshot is inconsistent with current registration".into(),
            ));
        }
        let history_path = self.registration_history_path(
            &recovery.provider_work_item_id,
            observed_provider_generation,
        );
        let historical_provider: WorktreeRegistration = self
            .read_json(&history_path)
            .map_err(|error| {
                CoordinationError::RecoveryRequired(format!(
                    "recovery provider snapshot has no verifiable immutable registration history: {error}"
                ))
            })?;
        validate_registration(&historical_provider)?;
        if !historical_provider.runtime.same_identity(&self.runtime)
            || historical_provider.repository_id != recovery.repository_id
            || historical_provider.work_item_id != recovery.provider_work_item_id
            || historical_provider.generation != observed_provider_generation
            || historical_provider.head != observed_provider_head
            || historical_provider.contract_digest != *observed_contract_digest
        {
            return Err(CoordinationError::RecoveryRequired(
                "recovery provider snapshot does not match its immutable registration history"
                    .into(),
            ));
        }
        Ok(())
    }

    fn registration_history_path(&self, work_item_id: &str, generation: u64) -> PathBuf {
        self.root
            .join("registration-history")
            .join(work_item_id)
            .join(format!("{generation}.json"))
    }

    fn persist_registration_snapshot(
        &self,
        registration: &WorktreeRegistration,
    ) -> Result<(), CoordinationError> {
        let path =
            self.registration_history_path(&registration.work_item_id, registration.generation);
        let parent = path.parent().expect("registration history has parent");
        fs::create_dir_all(parent).map_err(|source| CoordinationError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
        match fs::symlink_metadata(&path) {
            Ok(_) => {
                let existing: WorktreeRegistration = self.read_json(&path)?;
                if existing == *registration {
                    Ok(())
                } else {
                    Err(CoordinationError::RecoveryRequired(format!(
                        "immutable registration history already differs for {} generation {}",
                        registration.work_item_id, registration.generation
                    )))
                }
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                self.atomic_write(&path, registration)
            }
            Err(source) => Err(CoordinationError::Io { path, source }),
        }
    }

    fn observed_event_evidence_digests(
        &self,
        event: &CoordinationEvent,
        registration: &WorktreeRegistration,
    ) -> Result<BTreeMap<String, Digest>, CoordinationError> {
        self.observed_event_evidence_bytes(event, registration)
            .map(|observed| observed.digests)
    }

    fn observed_event_evidence_bytes(
        &self,
        event: &CoordinationEvent,
        registration: &WorktreeRegistration,
    ) -> Result<ObservedEventEvidence, CoordinationError> {
        self.validate_event_facts(event, registration)?;
        let root = Path::new(&registration.worktree_path);
        let mut digests = BTreeMap::new();
        let mut evidence = BTreeMap::new();
        for reference in &event.evidence_refs {
            let bytes = crate::collaboration::read_registered_worktree_file(root, reference)
                .map_err(|error| {
                    CoordinationError::RecoveryRequired(format!(
                        "event evidence is not safely readable for {} at {}: {error}",
                        event.event_id, reference
                    ))
                })?;
            digests.insert(reference.clone(), Digest::sha256_bytes(&bytes));
            evidence.insert(reference.clone(), bytes);
        }
        Ok(ObservedEventEvidence {
            digests,
            bytes: evidence,
        })
    }

    fn read_reservations(&self) -> Result<Vec<ResourceReservation>, CoordinationError> {
        let mut values = Vec::new();
        let path = self.root.join("reservations");
        for entry in fs::read_dir(&path).map_err(|source| CoordinationError::Io {
            path: path.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| CoordinationError::Io {
                path: path.clone(),
                source,
            })?;
            let record_path = entry.path();
            if record_path.extension().and_then(|value| value.to_str()) == Some("json") {
                values.push(self.read_json(&record_path)?);
            }
        }
        Ok(values)
    }

    fn read_directory<F>(
        &self,
        directory: &str,
        unknowns: &mut Vec<String>,
        mut consume: F,
    ) -> Result<(), CoordinationError>
    where
        F: FnMut(&Path) -> Result<(), CoordinationError>,
    {
        let path = self.root.join(directory);
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(source) if source.kind() == ErrorKind::NotFound => return Ok(()),
            Err(source) => {
                return Err(CoordinationError::Io {
                    path: path.clone(),
                    source,
                });
            }
        };
        for entry in entries {
            let entry = entry.map_err(|source| CoordinationError::Io {
                path: path.clone(),
                source,
            })?;
            let record_path = entry.path();
            if record_path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            if let Err(error) = consume(&record_path) {
                unknowns.push(format!("{}: {error}", record_path.display()));
            }
        }
        Ok(())
    }

    fn read_json<T: DeserializeOwned>(&self, path: &Path) -> Result<T, CoordinationError> {
        let metadata = fs::symlink_metadata(path).map_err(|source| CoordinationError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(CoordinationError::InvalidRecord {
                path: path.to_path_buf(),
                reason: "record is not a regular file".into(),
            });
        }
        let bytes = fs::read(path).map_err(|source| CoordinationError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        serde_json::from_slice(&bytes).map_err(|source| CoordinationError::InvalidRecord {
            path: path.to_path_buf(),
            reason: source.to_string(),
        })
    }

    fn atomic_write<T: Serialize>(&self, path: &Path, value: &T) -> Result<(), CoordinationError> {
        let bytes = serde_json::to_vec_pretty(value).map_err(|source| {
            CoordinationError::InvalidRecord {
                path: path.to_path_buf(),
                reason: source.to_string(),
            }
        })?;
        let sequence = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let temporary = path.with_extension(format!("tmp-{}-{sequence}", std::process::id()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|source| CoordinationError::Io {
                path: temporary.clone(),
                source,
            })?;
        file.write_all(&bytes)
            .map_err(|source| CoordinationError::Io {
                path: temporary.clone(),
                source,
            })?;
        file.sync_all().map_err(|source| CoordinationError::Io {
            path: temporary.clone(),
            source,
        })?;
        fs::rename(&temporary, path).map_err(|source| CoordinationError::Io {
            path: path.to_path_buf(),
            source,
        })
    }

    fn with_lock<T, F>(&self, operation: F) -> Result<T, CoordinationError>
    where
        F: FnOnce() -> Result<T, CoordinationError>,
    {
        let lock_path = self.root.join(".lock");
        let lock = open_lock_file(&lock_path)?;
        let started = Instant::now();
        loop {
            match try_lock_exclusive(&lock) {
                Ok(true) => break,
                Ok(false) if started.elapsed() < LOCK_TIMEOUT => {
                    thread::sleep(LOCK_WAIT);
                }
                Ok(false) => {
                    return Err(CoordinationError::RecoveryRequired(
                        "coordination lock did not become available".into(),
                    ));
                }
                Err(source) => {
                    return Err(CoordinationError::Io {
                        path: lock_path.clone(),
                        source,
                    });
                }
            }
        }
        let result = operation();
        drop(lock);
        result
    }
}

pub(crate) fn validate_contract_registration_identity(
    registration: &WorktreeRegistration,
    contract: &cockpit_protocol::Contract,
) -> Result<(), CoordinationError> {
    if contract.work_item_id != registration.work_item_id
        || contract.repository_id != registration.repository_id.to_string()
    {
        return Err(CoordinationError::RecoveryRequired(format!(
            "registered Contract identity mismatch for {}",
            registration.work_item_id
        )));
    }
    Ok(())
}

fn open_lock_file(path: &Path) -> Result<File, CoordinationError> {
    // Keep this path and inode persistent. Unlinking a lock while another
    // process holds its open handle would let a contender lock a replacement.
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options.open(path).map_err(|source| CoordinationError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let metadata = file.metadata().map_err(|source| CoordinationError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(CoordinationError::RecoveryRequired(format!(
            "coordination lock path is not a regular file: {}",
            path.display()
        )));
    }
    Ok(file)
}

fn try_lock_exclusive(file: &File) -> std::io::Result<bool> {
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if result == 0 {
            return Ok(true);
        }
        let error = std::io::Error::last_os_error();
        if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted) {
            return Ok(false);
        }
        Err(error)
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::{
            LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY, LockFileEx,
        };
        use windows_sys::Win32::System::IO::OVERLAPPED;
        let mut overlapped = OVERLAPPED::default();
        let locked = unsafe {
            LockFileEx(
                file.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE,
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                1,
                0,
                &mut overlapped,
            )
        };
        if locked != 0 {
            return Ok(true);
        }
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(windows_sys::Win32::Foundation::ERROR_LOCK_VIOLATION as i32)
        {
            return Ok(false);
        }
        Err(error)
    }
}

fn validate_registration(registration: &WorktreeRegistration) -> Result<(), CoordinationError> {
    registration.runtime.validate_candidate()?;
    if registration.schema_version != COLLABORATION_SCHEMA_VERSION
        || !valid_component(&registration.work_item_id)
        || registration.worktree_path.trim().is_empty()
        || registration.generation == 0
    {
        return Err(CoordinationError::RecoveryRequired(
            "invalid worktree registration identity".into(),
        ));
    }
    Ok(())
}

fn validate_event(event: &CoordinationEvent) -> Result<(), CoordinationError> {
    if event.schema_version != COLLABORATION_SCHEMA_VERSION
        || !valid_component(&event.event_id)
        || !valid_component(&event.work_item_id)
        || event.generation == 0
    {
        return Err(CoordinationError::RecoveryRequired(
            "invalid event identity".into(),
        ));
    }
    let mut outcome_ids = std::collections::BTreeSet::new();
    if event
        .outcome_ids
        .iter()
        .any(|outcome_id| !valid_component(outcome_id) || !outcome_ids.insert(outcome_id))
    {
        return Err(CoordinationError::RecoveryRequired(
            "invalid or duplicate event outcome identity".into(),
        ));
    }
    Ok(())
}

fn validate_recovery_identity(recovery: &CoordinationRecovery) -> Result<(), CoordinationError> {
    if recovery.schema_version != COLLABORATION_SCHEMA_VERSION
        || !valid_component(&recovery.consumption_id)
        || !valid_component(&recovery.event_id)
        || !valid_component(&recovery.provider_work_item_id)
        || !valid_component(&recovery.consumer_work_item_id)
        || recovery.provider_generation == 0
        || recovery.consumer_generation == 0
    {
        return Err(CoordinationError::RecoveryRequired(
            "invalid coordination recovery identity".into(),
        ));
    }
    Ok(())
}

fn validate_reservation(reservation: &ResourceReservation) -> Result<(), CoordinationError> {
    if reservation.schema_version != COLLABORATION_SCHEMA_VERSION
        || !valid_component(&reservation.reservation_id)
        || !valid_component(&reservation.work_item_id)
        || reservation.generation == 0
        || reservation.resources.is_empty()
    {
        return Err(CoordinationError::RecoveryRequired(
            "invalid resource reservation identity".into(),
        ));
    }
    for resource in &reservation.resources {
        if resource.resource_id.trim().is_empty()
            || resource.resource_id.starts_with('/')
            || resource.resource_id.split('/').any(|part| part == "..")
        {
            return Err(CoordinationError::InvalidResource(
                resource.resource_id.clone(),
            ));
        }
    }
    Ok(())
}

fn valid_component(value: &str) -> bool {
    !value.trim().is_empty()
        && value != "."
        && value != ".."
        && !value.contains('/')
        && !value.contains('\\')
}

#[cfg(test)]
#[path = "coordination_store_lock_tests.rs"]
mod lock_recovery_tests;

fn valid_request_transition(from: CoordinationRequestState, to: CoordinationRequestState) -> bool {
    matches!(
        (from, to),
        (
            CoordinationRequestState::Requested,
            CoordinationRequestState::Acknowledged
        ) | (
            CoordinationRequestState::Requested,
            CoordinationRequestState::Unavailable
        ) | (
            CoordinationRequestState::Requested,
            CoordinationRequestState::Expired
        ) | (
            CoordinationRequestState::Acknowledged,
            CoordinationRequestState::SafelyPaused
        ) | (
            CoordinationRequestState::Acknowledged,
            CoordinationRequestState::Unavailable
        ) | (
            CoordinationRequestState::Acknowledged,
            CoordinationRequestState::Expired
        ) | (
            CoordinationRequestState::SafelyPaused,
            CoordinationRequestState::Resumed
        )
    )
}

#[cfg(all(test, unix))]
mod registered_worktree_file_tests {
    use super::*;
    use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
    use cockpit_protocol::{COLLABORATION_CAPABILITY, CollaborationDeclaration};
    use std::io::Read;

    fn run(root: &Path, args: &[&str]) {
        assert!(
            std::process::Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .expect("git command")
                .success()
        );
    }

    fn repository() -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("repository root");
        run(root.path(), &["init", "-q"]);
        run(
            root.path(),
            &["config", "user.email", "test@example.invalid"],
        );
        run(root.path(), &["config", "user.name", "Coordination test"]);
        fs::write(root.path().join("README.md"), "initial\n").expect("README");
        run(root.path(), &["add", "."]);
        run(root.path(), &["commit", "-qm", "initial"]);
        run(root.path(), &["branch", "-M", "main"]);
        crate::attach(root.path()).expect("attach repository");
        crate::start_work_item_with_options(
            root.path(),
            "WI-REGISTERED-SWAP-BACK",
            "coordination registration race test",
            "reject a moved Contract before durable publication",
            &[".ai/**".into(), "README.md".into()],
            &crate::WorkItemStartOptions {
                authority: "authorized".into(),
                out_of_scope: vec!["target/**".into()],
                acceptance_criteria: vec!["rejected Contract reads leave storage unchanged".into()],
                ..crate::WorkItemStartOptions::default()
            },
        )
        .expect("start test Work Item");
        root
    }

    fn runtime() -> RuntimeCapabilityBinding {
        RuntimeCapabilityBinding {
            schema_version: 1,
            runtime_version: "0.2.113".into(),
            runtime_digest: Digest::sha256_bytes(b"test-runtime"),
            capability: COLLABORATION_CAPABILITY.into(),
        }
    }

    fn registration(root: &Path, generation: u64) -> WorktreeRegistration {
        let work_item_id = "WI-REGISTERED-SWAP-BACK";
        let contract_path = root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"));
        let contract: serde_json::Value =
            serde_json::from_slice(&fs::read(contract_path).expect("Contract bytes"))
                .expect("Contract JSON");
        let topology = GitRepository::discover(root)
            .expect("discover repository")
            .topology()
            .expect("repository topology");
        WorktreeRegistration {
            schema_version: COLLABORATION_SCHEMA_VERSION,
            repository_id: crate::repository_id(root),
            work_item_id: work_item_id.into(),
            contract_digest: cockpit_protocol::digest_json(&contract).expect("Contract digest"),
            worktree_path: topology.repository_root.to_string_lossy().into_owned(),
            branch: topology.branch.expect("branch"),
            head: topology.head.expect("head"),
            generation,
            declaration: CollaborationDeclaration::default(),
            runtime: runtime(),
        }
    }

    fn file_bytes(directory: &Path) -> BTreeMap<String, Vec<u8>> {
        fs::read_dir(directory)
            .expect("record directory")
            .map(|entry| {
                let entry = entry.expect("record entry");
                (
                    entry.file_name().to_string_lossy().into_owned(),
                    fs::read(entry.path()).expect("record bytes"),
                )
            })
            .collect()
    }

    #[test]
    fn swap_back_during_registration_leaves_registration_and_event_bytes_unchanged() {
        let root = repository();
        let git = GitRepository::discover(root.path()).expect("discover");
        let store = CoordinationStore::open(&git, runtime()).expect("coordination store");
        let original = registration(root.path(), 1);
        store
            .register(original.clone())
            .expect("initial registration");
        let registration_path = store.registration_path(&original.work_item_id);
        let original_registration_bytes = fs::read(&registration_path).expect("registration");
        let events_path = store.root().join("events");
        let original_event_bytes = file_bytes(&events_path);

        let contract_path = root
            .path()
            .join(".ai/work-items/active/WI-REGISTERED-SWAP-BACK.contract.json");
        let mut contract: serde_json::Value =
            serde_json::from_slice(&fs::read(&contract_path).expect("Contract bytes"))
                .expect("Contract JSON");
        contract["verification"] = serde_json::json!([{
            "check": "cargo test --locked -p cockpit-repository",
            "required": true
        }]);
        fs::write(
            &contract_path,
            serde_json::to_vec_pretty(&contract).expect("serialize updated Contract"),
        )
        .expect("update test Contract");
        let replacement_bytes = fs::read(&contract_path).expect("updated Contract bytes");
        let replacement = registration(root.path(), 2);
        assert_ne!(replacement.contract_digest, original.contract_digest);

        let outside = tempfile::tempdir().expect("outside directory");
        let active_path = root.path().join(".ai/work-items/active");
        let moved_active = outside.path().join("active");
        let mut opened_while_outside = Vec::new();
        let result =
            store.register_with_contract_reader(replacement, |repository_root, reference| {
                crate::collaboration::read_registered_worktree_file_with_opener(
                    repository_root,
                    reference,
                    |parent, leaf| {
                        fs::rename(&active_path, &moved_active)
                            .expect("move opened active directory");
                        let mut options = cap_std::fs::OpenOptions::new();
                        options.read(true).follow(FollowSymlinks::No);
                        let mut file = parent
                            .open_with(leaf, &options)
                            .expect("open Contract through moved directory handle")
                            .into_std();
                        file.read_to_end(&mut opened_while_outside)
                            .expect("read Contract while outside");
                        fs::rename(&moved_active, &active_path).expect("restore active directory");
                        Ok(file)
                    },
                )
            });

        assert_eq!(opened_while_outside, replacement_bytes);
        assert!(
            matches!(result, Err(CoordinationError::RecoveryRequired(_))),
            "the Contract read must be rejected before registration publication: {result:?}"
        );
        assert_eq!(
            fs::read(&registration_path).expect("registration after rejection"),
            original_registration_bytes,
            "rejected re-registration must preserve the old registration bytes"
        );
        assert_eq!(
            file_bytes(&events_path),
            original_event_bytes,
            "rejected re-registration must not publish an impact event"
        );
    }

    #[test]
    fn moved_open_contract_rejection_leaves_registration_and_event_bytes_unchanged() {
        let root = repository();
        let git = GitRepository::discover(root.path()).expect("discover");
        let store = CoordinationStore::open(&git, runtime()).expect("coordination store");
        let original = registration(root.path(), 1);
        store
            .register(original.clone())
            .expect("initial registration");
        let registration_path = store.registration_path(&original.work_item_id);
        let original_registration_bytes = fs::read(&registration_path).expect("registration");
        let events_path = store.root().join("events");
        let original_event_bytes = file_bytes(&events_path);

        let contract_path = root
            .path()
            .join(".ai/work-items/active/WI-REGISTERED-SWAP-BACK.contract.json");
        let mut contract: serde_json::Value =
            serde_json::from_slice(&fs::read(&contract_path).expect("Contract bytes"))
                .expect("Contract JSON");
        contract["verification"] = serde_json::json!([{
            "check": "cargo test --locked -p cockpit-repository",
            "required": true
        }]);
        fs::write(
            &contract_path,
            serde_json::to_vec_pretty(&contract).expect("serialize updated Contract"),
        )
        .expect("update test Contract");
        let replacement_bytes = fs::read(&contract_path).expect("updated Contract bytes");
        let replacement = registration(root.path(), 2);
        assert_ne!(replacement.contract_digest, original.contract_digest);

        let outside = tempfile::tempdir().expect("outside directory");
        let active_path = root.path().join(".ai/work-items/active");
        let moved_active = outside.path().join("active");
        let contract_leaf = format!("{}.contract.json", original.work_item_id);
        let result =
            store.register_with_contract_reader(replacement, |repository_root, reference| {
                crate::collaboration::read_registered_worktree_file_with_opener(
                    repository_root,
                    reference,
                    |parent, leaf| {
                        fs::rename(&active_path, &moved_active)
                            .expect("move opened active directory outside repository");
                        fs::create_dir_all(&active_path)
                            .expect("install an in-repository replacement directory");
                        fs::write(active_path.join(&contract_leaf), &replacement_bytes)
                            .expect("write matching replacement Contract");
                        let mut options = cap_std::fs::OpenOptions::new();
                        options.read(true).follow(FollowSymlinks::No);
                        parent
                            .open_with(leaf, &options)
                            .map(cap_std::fs::File::into_std)
                    },
                )
            });

        assert_eq!(
            fs::read(moved_active.join(&contract_leaf)).expect("Contract opened outside root"),
            replacement_bytes,
            "the moved directory contains the exact matching Contract bytes"
        );
        assert!(
            matches!(result, Err(CoordinationError::RecoveryRequired(_))),
            "the moved open directory must be rejected before registration publication: {result:?}"
        );
        assert_eq!(
            fs::read(&registration_path).expect("registration after rejection"),
            original_registration_bytes,
            "rejected registration must preserve the old registration bytes"
        );
        assert_eq!(
            file_bytes(&events_path),
            original_event_bytes,
            "rejected registration must not publish an impact event"
        );
    }
}
