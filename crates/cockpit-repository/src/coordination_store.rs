use cockpit_git::GitRepository;
use cockpit_protocol::{
    COLLABORATION_SCHEMA_VERSION, CoordinationEvent, CoordinationRecovery, CoordinationRequest,
    CoordinationRequestState, ResourceClaimMode, ResourceReservation, RuntimeCapabilityBinding,
    WorktreeRegistration,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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

#[derive(Clone, Debug)]
pub struct CoordinationStore {
    root: PathBuf,
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
        Ok(Self { root, runtime })
    }

    pub fn root(&self) -> &Path {
        &self.root
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
        self.runtime.validate_candidate()?;
        validate_registration(&registration)?;
        if !registration.runtime.same_identity(&self.runtime) {
            return Err(CoordinationError::Runtime(
                cockpit_protocol::RuntimeCapabilityError::IdentityMismatch,
            ));
        }
        self.with_lock(|| {
            let path = self.registration_path(&registration.work_item_id);
            if path.exists() {
                let existing: WorktreeRegistration = self.read_json(&path)?;
                if existing == registration {
                    return Ok(existing);
                }
                if existing.generation >= registration.generation {
                    return Err(CoordinationError::StaleGeneration {
                        work_item_id: registration.work_item_id.clone(),
                        expected: existing.generation,
                        actual: registration.generation,
                    });
                }
            }
            self.atomic_write(&path, &registration)?;
            Ok(registration)
        })
    }

    pub fn publish_event(
        &self,
        event: CoordinationEvent,
    ) -> Result<CoordinationEvent, CoordinationError> {
        self.runtime.validate_candidate()?;
        validate_event(&event)?;
        self.with_lock(|| {
            let registration_path = self.registration_path(&event.work_item_id);
            let registration: WorktreeRegistration = self.read_json(&registration_path)?;
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
        })
    }

    pub fn reserve_resources(
        &self,
        reservation: ResourceReservation,
    ) -> Result<ResourceReservation, CoordinationError> {
        self.runtime.validate_candidate()?;
        validate_reservation(&reservation)?;
        self.with_lock(|| {
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
            if reservation.work_item_id != work_item_id || reservation.generation != generation {
                return Err(CoordinationError::StaleGeneration {
                    work_item_id: work_item_id.into(),
                    expected: reservation.generation,
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
        self.with_lock(|| {
            let registration_path = self.registration_path(&request.target_work_item_id);
            let registration: WorktreeRegistration = self.read_json(&registration_path)?;
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
        self.with_lock(|| {
            let path = self
                .root
                .join("requests")
                .join(format!("{request_id}.json"));
            let mut request: CoordinationRequest = self.read_json(&path)?;
            let registration: WorktreeRegistration =
                self.read_json(&self.registration_path(&request.target_work_item_id))?;
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
            if provider_registration.generation != event.generation {
                return Err(CoordinationError::StaleGeneration {
                    work_item_id: event.work_item_id.clone(),
                    expected: provider_registration.generation,
                    actual: event.generation,
                });
            }
            let consumer_registration: WorktreeRegistration =
                self.read_json(&self.registration_path(consumer_work_item_id))?;
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
            if !Path::new(&registration.worktree_path).exists() {
                return Err(CoordinationError::RecoveryRequired(format!(
                    "worktree moved or deleted: {}",
                    registration.worktree_path
                )));
            }
            inspection.registrations.push(registration);
            Ok(())
        })?;
        self.read_directory("events", &mut inspection.unknowns, |path| {
            inspection.events.push(self.read_json(path)?);
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
            inspection.recoveries.push(self.read_json(path)?);
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
        let started = SystemTime::now();
        let lock = loop {
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
            {
                Ok(file) => break file,
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    if started.elapsed().unwrap_or(LOCK_TIMEOUT) >= LOCK_TIMEOUT {
                        return Err(CoordinationError::RecoveryRequired(
                            "coordination lock did not become available".into(),
                        ));
                    }
                    thread::sleep(LOCK_WAIT);
                }
                Err(source) => {
                    return Err(CoordinationError::Io {
                        path: lock_path.clone(),
                        source,
                    });
                }
            }
        };
        let result = operation();
        drop(lock);
        let _ = fs::remove_file(&lock_path);
        result
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
