use crate::{CoordinationError, CoordinationStore};
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::{
    ConsumedOutcome, CoordinationEvent, CoordinationIntent, CoordinationRecovery,
    CoordinationRequest, CoordinationRequestState, ProviderOutcomeKey, RuntimeCapabilityBinding,
    RuntimeContext, WorktreeRegistration,
};
use cockpit_verification::{
    CompositionAttempt, CompositionCleanupDisposition, CompositionError,
    CompositionExecutionOutcome, CompositionInput, CompositionPrecondition,
    CompositionSupervisorBackend, CompositionSupervisorControl, CompositionSupervisorLaunch,
    CompositionSupervisorReady, CompositionSupervisorReceipt, CompositionSupervisorReply,
    ProcessAdmissionCheck, ProcessStartGate, composition_linux_boot_id,
    current_composition_process_identity, execution_records_digest, new_composition_run_nonce,
    new_supervised_composition_attempt_id,
};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::io::{self, Read, Write};
#[cfg(not(unix))]
use std::io::{BufRead, BufReader};
use std::path::{Component, Path};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;
use thiserror::Error;

#[cfg(test)]
thread_local! {
    static DIRECTORY_CHANGE_TOKEN_OVERRIDE: std::cell::Cell<Option<DirectoryChangeToken>> = const { std::cell::Cell::new(None) };
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationProjection {
    pub registrations: Vec<WorktreeRegistration>,
    pub events: Vec<CoordinationEvent>,
    pub recoveries: Vec<CoordinationRecovery>,
    pub requests: Vec<CoordinationRequest>,
    pub affected_work_items: Vec<String>,
    pub blockers: BTreeMap<String, Vec<String>>,
    pub cycles: Vec<Vec<String>>,
    pub unknowns: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationAdmission {
    pub work_item_id: String,
    pub generation: u64,
    pub allowed: bool,
    pub affected: bool,
    pub blockers: Vec<String>,
    pub unknowns: Vec<String>,
    pub refreshed_events: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationActionKind {
    Composition,
    Verification,
    ResourceReservation,
    ResourceRelease,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationAction {
    pub kind: CollaborationActionKind,
    pub consumer_work_item_id: String,
    #[serde(default)]
    pub outcomes: Vec<ProviderOutcomeKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationOutcomeProjection {
    pub schema_version: u32,
    pub work_item_id: String,
    pub state: String,
    pub providers: Vec<String>,
    pub consumers: Vec<String>,
    pub waiting_edges: Vec<String>,
    pub invalidated_event_ids: Vec<String>,
    pub unhandled_requests: Vec<CoordinationRequest>,
    pub integration_owner: Option<String>,
    pub composition_order: Vec<String>,
    pub implementation_state: String,
    pub composition_state: String,
    pub execution_outcome: CompositionExecutionOutcome,
    pub execution_evidence_complete: bool,
    pub composition_applicability: String,
    pub target_merge_state: String,
    pub cleanup_state: String,
    pub cleanup_disposition: CompositionCleanupDisposition,
    pub revalidation: String,
    pub reusable_checks: Vec<String>,
    pub blockers: Vec<String>,
    pub unknowns: Vec<String>,
    pub human_decision_required: bool,
    pub next_action: String,
}

#[derive(Debug, Error)]
pub enum CollaborationExecutionError {
    #[error(transparent)]
    Coordination(#[from] CoordinationError),
    #[error("collaboration action is blocked for {work_item_id}: {blockers:?}")]
    Blocked {
        work_item_id: String,
        blockers: Vec<String>,
        unknowns: Vec<String>,
    },
    #[error(transparent)]
    Composition(#[from] CompositionError),
    #[error("composition parent observation publication failed: {message}")]
    ParentObservationPublication {
        message: String,
        observations: Vec<cockpit_verification::CompositionParentObservation>,
    },
}

pub fn collaboration_projection(
    store: &CoordinationStore,
) -> Result<CollaborationProjection, CoordinationError> {
    let inspection = store.inspect()?;
    let registrations = inspection.registrations;
    let events = inspection.events;
    let recoveries = inspection.recoveries;
    let requests = inspection.requests;
    let mut projection = CollaborationProjection {
        registrations,
        events,
        recoveries,
        requests,
        unknowns: inspection.unknowns,
        ..CollaborationProjection::default()
    };

    let invalidated_outcomes = transitive_invalidated_outcomes(&projection);
    let mut affected = BTreeSet::new();
    for registration in &projection.registrations {
        let mut blockers = dependency_blockers_for_registration(
            &projection,
            registration,
            None,
            &invalidated_outcomes,
        );
        if blockers
            .iter()
            .any(|blocker| blocker.starts_with("dependency_impact:"))
        {
            affected.insert(registration.work_item_id.clone());
        }
        if !blockers.is_empty() {
            blockers.sort();
            blockers.dedup();
            projection
                .blockers
                .insert(registration.work_item_id.clone(), blockers);
        }
    }
    projection.affected_work_items = affected.into_iter().collect();
    projection.cycles = find_cycles(&projection.registrations);
    for cycle in &projection.cycles {
        for work_item_id in cycle {
            projection
                .blockers
                .entry(work_item_id.clone())
                .or_default()
                .push(format!("dependency_cycle:{}", cycle.join("->")));
        }
    }
    for blockers in projection.blockers.values_mut() {
        blockers.sort();
        blockers.dedup();
    }
    Ok(projection)
}

fn dependency_blockers_for_registration(
    projection: &CollaborationProjection,
    registration: &WorktreeRegistration,
    selected_outcomes: Option<&BTreeSet<ProviderOutcomeKey>>,
    invalidated_outcomes: &BTreeMap<ProviderOutcomeKey, Vec<CoordinationEvent>>,
) -> Vec<String> {
    let mut blockers = Vec::new();
    for dependency in &registration.declaration.consumed_outcomes {
        let key = provider_outcome_key(dependency);
        if selected_outcomes.is_some_and(|selected| !selected.contains(&key)) {
            continue;
        }
        let Some(provider) = projection
            .registrations
            .iter()
            .find(|provider| provider.work_item_id == dependency.provider_work_item_id)
        else {
            blockers.push(format!(
                "dependency_missing:{}:{}",
                dependency.provider_work_item_id, dependency.outcome_id
            ));
            continue;
        };
        let Some(outcome) = provider
            .declaration
            .provided_outcomes
            .iter()
            .find(|outcome| outcome.outcome_id == dependency.outcome_id)
        else {
            blockers.push(format!(
                "outcome_missing:{}:{}",
                dependency.provider_work_item_id, dependency.outcome_id
            ));
            continue;
        };
        if outcome.published_head != provider.head {
            blockers.push(format!(
                "outcome_head_mismatch:{}:{}",
                dependency.provider_work_item_id, dependency.outcome_id
            ));
        }
        if !outcome.stage.satisfies(dependency.minimum_stage) {
            blockers.push(format!(
                "dependency_stage:{}:{}:{:?}",
                dependency.provider_work_item_id, dependency.outcome_id, outcome.stage
            ));
        }
        if outcome.stage == cockpit_protocol::OutcomeStage::MergedTarget {
            match commit_is_ancestor(
                Path::new(&provider.worktree_path),
                &provider
                    .declaration
                    .integration_responsibility
                    .target_branch,
                &outcome.published_head,
            ) {
                Some(true) => {}
                Some(false) => blockers.push(format!(
                    "outcome_merge_fact_missing:{}:{}",
                    dependency.provider_work_item_id, dependency.outcome_id
                )),
                None => blockers.push(format!(
                    "outcome_merge_fact_unknown:{}:{}",
                    dependency.provider_work_item_id, dependency.outcome_id
                )),
            }
        }
        if invalidated_outcomes.get(&key).is_some_and(|events| {
            events
                .iter()
                .any(|event| !recovery_consumed(projection, event, registration))
        }) {
            blockers.push(format!(
                "dependency_impact:{}:{}",
                dependency.provider_work_item_id, dependency.outcome_id
            ));
        }
        if dependency.verification_required
            && !verification_evidence_is_complete(projection, provider, outcome)
        {
            blockers.push(format!(
                "dependency_evidence_missing:{}:{}",
                dependency.provider_work_item_id, dependency.outcome_id
            ));
        }
    }
    blockers
}

pub(crate) fn provider_outcome_key(dependency: &ConsumedOutcome) -> ProviderOutcomeKey {
    ProviderOutcomeKey {
        provider_work_item_id: dependency.provider_work_item_id.clone(),
        outcome_id: dependency.outcome_id.clone(),
    }
}

fn verification_evidence_is_complete(
    projection: &CollaborationProjection,
    provider: &WorktreeRegistration,
    outcome: &cockpit_protocol::ProvidedOutcome,
) -> bool {
    verification_evidence_is_complete_with_reader(
        projection,
        provider,
        outcome,
        read_registered_worktree_file,
    )
}

fn verification_evidence_is_complete_with_reader<F>(
    projection: &CollaborationProjection,
    provider: &WorktreeRegistration,
    outcome: &cockpit_protocol::ProvidedOutcome,
    mut read_file: F,
) -> bool
where
    F: FnMut(&Path, &str) -> Result<Vec<u8>, String>,
{
    projection.events.iter().any(|publication| {
        verification_evidence_matches_publication(publication, provider, outcome, &mut read_file)
    })
}

fn verification_evidence_matches_publication<F>(
    publication: &CoordinationEvent,
    provider: &WorktreeRegistration,
    outcome: &cockpit_protocol::ProvidedOutcome,
    read_file: &mut F,
) -> bool
where
    F: FnMut(&Path, &str) -> Result<Vec<u8>, String>,
{
    let root = Path::new(&provider.worktree_path);
    let mut observed_evidence = BTreeMap::new();
    for reference in &outcome.evidence_refs {
        let Ok(bytes) = read_file(root, reference) else {
            return false;
        };
        observed_evidence.insert(reference.clone(), bytes);
    }
    verification_evidence_matches_publication_with_bytes(
        publication,
        provider,
        outcome,
        &observed_evidence,
        read_file,
    )
}

fn verification_evidence_matches_publication_with_bytes<F>(
    publication: &CoordinationEvent,
    provider: &WorktreeRegistration,
    outcome: &cockpit_protocol::ProvidedOutcome,
    observed_evidence: &BTreeMap<String, Vec<u8>>,
    read_file: &mut F,
) -> bool
where
    F: FnMut(&Path, &str) -> Result<Vec<u8>, String>,
{
    let root = Path::new(&provider.worktree_path);
    let expected_reference = format!(".ai/evidence/{}.verification.json", provider.work_item_id);
    let declared_references = outcome.evidence_refs.iter().collect::<BTreeSet<_>>();
    if publication.kind != cockpit_protocol::CoordinationEventKind::OutcomePublished
        || publication.repository_id != provider.repository_id
        || publication.work_item_id != provider.work_item_id
        || publication.generation != provider.generation
        || !publication
            .outcome_ids
            .iter()
            .any(|outcome_id| outcome_id == &outcome.outcome_id)
        || outcome.published_head != provider.head
        || !declared_references.contains(&&expected_reference)
        || declared_references.len() != outcome.evidence_refs.len()
        || publication.evidence_refs != outcome.evidence_refs
        || publication.evidence_digests.len() != declared_references.len()
        || observed_evidence.len() != declared_references.len()
        || publication
            .evidence_digests
            .keys()
            .any(|reference| !declared_references.contains(reference))
    {
        return false;
    }
    for reference in &outcome.evidence_refs {
        let Some(bytes) = observed_evidence.get(reference) else {
            return false;
        };
        if publication.evidence_digests.get(reference)
            != Some(&cockpit_core::Digest::sha256_bytes(bytes))
        {
            return false;
        }
    }
    let Some(bytes) = observed_evidence.get(&expected_reference) else {
        return false;
    };
    let Ok(envelope) = serde_json::from_slice::<crate::VerificationEvidenceV2>(bytes) else {
        return false;
    };
    if envelope.work_item_id != provider.work_item_id
        || envelope.repository_id != provider.repository_id.to_string()
        || envelope.contract_digest.as_ref() != Some(&provider.contract_digest)
        || envelope.runtime_version != provider.runtime.runtime_version
        || envelope.runtime_digest != provider.runtime.runtime_digest
        || !envelope.passed
        || !matches!(
            envelope.capture_mode,
            crate::VerificationCaptureMode::FullCapture
                | crate::VerificationCaptureMode::RedactedCapture
        )
        || envelope.receipt.is_none()
        || outcome.published_head != provider.head
    {
        return false;
    }
    let Ok(contract) = read_registered_contract_with_reader(provider, read_file) else {
        return false;
    };
    let Ok(git) = GitRepository::discover(root) else {
        return false;
    };
    let Ok(snapshot) = git.snapshot() else {
        return false;
    };
    if snapshot.head.as_deref() != Some(provider.head.as_str()) {
        return false;
    }
    let runtime = RuntimeContext {
        runtime_version: provider.runtime.runtime_version.clone(),
        protocol_version: 1,
        runtime_digest: provider.runtime.runtime_digest.clone(),
    };
    crate::verification_evidence_state_from_bytes(
        root,
        &contract,
        &snapshot,
        false,
        Some(&runtime),
        bytes,
    )
    .is_ok_and(|state| state == cockpit_core::EvidenceState::Complete)
}

/// Read outcome evidence through directory handles rooted at the registered
/// worktree. Every parent is opened without following symlinks and the leaf is
/// opened with no-follow semantics. Parent containment is checked against the
/// identity of the exact opened directory handle before and after opening the
/// leaf. Native directory-mutation witnesses reject rename/delete interleavings
/// without treating filesystem timestamps as mutation counters.
pub(crate) fn open_registered_worktree_file(
    root: &Path,
    reference: &str,
) -> Result<fs::File, String> {
    Ok(open_registered_worktree_file_with_context(root, reference)?.file)
}

fn open_registered_worktree_file_with_context(
    root: &Path,
    reference: &str,
) -> Result<OpenedRegisteredWorktreeFile, String> {
    open_registered_worktree_file_with_opener(root, reference, open_leaf_nofollow)
}

fn open_registered_worktree_file_with_opener<F>(
    root: &Path,
    reference: &str,
    open_leaf: F,
) -> Result<OpenedRegisteredWorktreeFile, String>
where
    F: FnOnce(&Dir, &str) -> io::Result<fs::File>,
{
    if reference.is_empty()
        || reference.contains('\\')
        || reference
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
    {
        return Err(format!("invalid relative evidence reference: {reference}"));
    }
    let relative = Path::new(reference);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("invalid relative evidence reference: {reference}"));
    }
    let components = relative
        .components()
        .map(|component| component.as_os_str().to_owned())
        .collect::<Vec<_>>();
    let (leaf, parents) = components
        .split_last()
        .ok_or_else(|| format!("empty evidence reference: {reference}"))?;
    let leaf = leaf
        .to_str()
        .ok_or_else(|| format!("evidence reference is not valid UTF-8: {reference}"))?;
    let canonical_root = fs::canonicalize(root).map_err(|error| {
        format!(
            "cannot resolve registered worktree {}: {error}",
            root.display()
        )
    })?;
    let mut parent =
        Dir::open_ambient_dir(&canonical_root, cap_std::ambient_authority()).map_err(|error| {
            format!(
                "cannot open registered worktree {}: {error}",
                canonical_root.display()
            )
        })?;
    let mut display_path = canonical_root.clone();
    let mut directory_chain = vec![OpenedDirectoryGuard::capture(
        &parent,
        &display_path,
        reference,
    )?];
    for component in parents {
        let name = component
            .to_str()
            .ok_or_else(|| format!("evidence path component is not valid UTF-8: {reference}"))?;
        display_path.push(name);
        let child = crate::open_cap_directory_nofollow_strict(&parent, name, &display_path)
            .map_err(|error| {
                format!(
                    "evidence parent is not safely contained at {}: {error}",
                    display_path.display()
                )
            })?;
        directory_chain.push(OpenedDirectoryGuard::capture(
            &child,
            &display_path,
            reference,
        )?);
        parent = child;
    }
    display_path.push(leaf);
    open_registered_worktree_leaf(
        &canonical_root,
        parent,
        directory_chain,
        &display_path,
        leaf,
        reference,
        open_leaf,
    )
}

struct OpenedRegisteredWorktreeFile {
    file: fs::File,
    canonical_root: std::path::PathBuf,
    directory_chain: Vec<OpenedDirectoryGuard>,
    reference: String,
    mutation_observer: DirectoryMutationObserver,
}

struct OpenedDirectoryGuard {
    directory: Dir,
    display_path: std::path::PathBuf,
    identity: (u64, u64),
    change_token: DirectoryChangeToken,
    #[cfg(windows)]
    _rename_guard: fs::File,
}

impl OpenedDirectoryGuard {
    fn capture(directory: &Dir, display_path: &Path, reference: &str) -> Result<Self, String> {
        let handle = directory
            .try_clone()
            .map_err(|error| {
                format!("cannot clone opened evidence directory {reference}: {error}")
            })?
            .into_std_file();
        let metadata = handle.metadata().map_err(|error| {
            format!("cannot inspect opened evidence directory {reference}: {error}")
        })?;
        if !metadata.is_dir() {
            return Err(format!(
                "opened evidence parent is not a directory: {reference}"
            ));
        }
        let identity = file_identity(&handle).map_err(|error| {
            format!("cannot identify opened evidence directory {reference}: {error}")
        })?;
        #[cfg(windows)]
        let rename_guard = {
            let guard = open_directory_rename_guard(display_path).map_err(|error| {
                format!("cannot protect opened evidence directory {reference}: {error}")
            })?;
            if file_identity(&guard).map_err(|error| {
                format!("cannot identify protected evidence directory {reference}: {error}")
            })? != identity
            {
                return Err(format!(
                    "protected evidence directory does not match the opened handle: {reference}"
                ));
            }
            guard
        };
        Ok(Self {
            directory: directory.try_clone().map_err(|error| {
                format!("cannot retain opened evidence directory {reference}: {error}")
            })?,
            display_path: display_path.to_path_buf(),
            identity,
            change_token: directory_change_token(&handle).map_err(|error| {
                format!("cannot observe opened evidence directory {reference}: {error}")
            })?,
            #[cfg(windows)]
            _rename_guard: rename_guard,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DirectoryChangeToken([i64; 4]);

#[cfg(target_os = "linux")]
struct DirectoryMutationObserver {
    instance: std::os::fd::OwnedFd,
    _directories: Vec<fs::File>,
}

#[cfg(target_os = "linux")]
impl DirectoryMutationObserver {
    fn start(directory_chain: &[OpenedDirectoryGuard], reference: &str) -> Result<Self, String> {
        use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

        let raw_instance = unsafe { libc::inotify_init1(libc::IN_CLOEXEC | libc::IN_NONBLOCK) };
        if raw_instance < 0 {
            return Err(format!(
                "cannot create directory mutation observer for {reference}: {}",
                io::Error::last_os_error()
            ));
        }
        let instance = unsafe { OwnedFd::from_raw_fd(raw_instance) };
        let mut directories = Vec::with_capacity(directory_chain.len());
        for guard in directory_chain {
            let directory = guard
                .directory
                .try_clone()
                .map_err(|error| format!("cannot retain observed directory {reference}: {error}"))?
                .into_std_file();
            let descriptor_path =
                std::ffi::CString::new(format!("/proc/self/fd/{}", directory.as_raw_fd()))
                    .map_err(|error| format!("invalid directory descriptor path: {error}"))?;
            let mask = libc::IN_MOVE_SELF | libc::IN_DELETE_SELF | libc::IN_UNMOUNT;
            let watch = unsafe {
                libc::inotify_add_watch(instance.as_raw_fd(), descriptor_path.as_ptr(), mask)
            };
            if watch < 0 {
                return Err(format!(
                    "cannot observe directory rename for {reference}: {}",
                    io::Error::last_os_error()
                ));
            }
            directories.push(directory);
        }
        Ok(Self {
            instance,
            _directories: directories,
        })
    }

    fn verify_unchanged(&self, reference: &str) -> Result<(), String> {
        use std::os::fd::AsRawFd;

        let mut buffer = [0u8; 4096];
        loop {
            let count = unsafe {
                libc::read(
                    self.instance.as_raw_fd(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                )
            };
            if count < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                if error.kind() == io::ErrorKind::WouldBlock {
                    return Ok(());
                }
                return Err(format!(
                    "cannot read directory mutation events for {reference}: {error}"
                ));
            }
            if count == 0 {
                return Err(format!(
                    "directory mutation observer closed before validation: {reference}"
                ));
            }
            let count = count as usize;
            let header_size = std::mem::size_of::<libc::inotify_event>();
            if count < header_size {
                return Err(format!(
                    "malformed directory mutation event for {reference}"
                ));
            }
            let mask = u32::from_ne_bytes(
                buffer[4..8]
                    .try_into()
                    .expect("fixed inotify event mask width"),
            );
            let name_len = u32::from_ne_bytes(
                buffer[12..16]
                    .try_into()
                    .expect("fixed inotify event name width"),
            ) as usize;
            let event_size = header_size.checked_add(name_len).ok_or_else(|| {
                format!("directory mutation event length overflow for {reference}")
            })?;
            if event_size > count {
                return Err(format!(
                    "truncated directory mutation event for {reference}"
                ));
            }
            return Err(format!(
                "registered evidence directory was renamed, deleted, or unmounted while being read: {reference} (mask {mask:#x})"
            ));
        }
    }
}

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
struct DirectoryMutationObserver {
    queue: std::os::fd::OwnedFd,
    _directories: Vec<fs::File>,
}

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
impl DirectoryMutationObserver {
    fn start(directory_chain: &[OpenedDirectoryGuard], reference: &str) -> Result<Self, String> {
        use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

        let raw_queue = unsafe { libc::kqueue() };
        if raw_queue < 0 {
            return Err(format!(
                "cannot create directory mutation observer for {reference}: {}",
                io::Error::last_os_error()
            ));
        }
        let queue = unsafe { OwnedFd::from_raw_fd(raw_queue) };
        let mut directories = Vec::with_capacity(directory_chain.len());
        for guard in directory_chain {
            let directory = guard
                .directory
                .try_clone()
                .map_err(|error| format!("cannot retain observed directory {reference}: {error}"))?
                .into_std_file();
            let mut change: libc::kevent = unsafe { std::mem::zeroed() };
            change.ident = directory.as_raw_fd() as libc::uintptr_t;
            change.filter = libc::EVFILT_VNODE;
            change.flags = libc::EV_ADD | libc::EV_CLEAR;
            change.fflags = libc::NOTE_RENAME | libc::NOTE_DELETE;
            let result = unsafe {
                libc::kevent(
                    queue.as_raw_fd(),
                    &change,
                    1,
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null(),
                )
            };
            if result < 0 {
                return Err(format!(
                    "cannot observe directory rename for {reference}: {}",
                    io::Error::last_os_error()
                ));
            }
            directories.push(directory);
        }
        Ok(Self {
            queue,
            _directories: directories,
        })
    }

    fn verify_unchanged(&self, reference: &str) -> Result<(), String> {
        use std::os::fd::AsRawFd;

        let mut event: libc::kevent = unsafe { std::mem::zeroed() };
        let timeout = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        loop {
            let result = unsafe {
                libc::kevent(
                    self.queue.as_raw_fd(),
                    std::ptr::null(),
                    0,
                    &mut event,
                    1,
                    &timeout,
                )
            };
            if result < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(format!(
                    "cannot read directory mutation events for {reference}: {error}"
                ));
            }
            if result == 0 {
                return Ok(());
            }
            return Err(format!(
                "registered evidence directory was renamed or deleted while being read: {reference} (flags {:#x})",
                event.fflags
            ));
        }
    }
}

#[cfg(windows)]
struct DirectoryMutationObserver;

#[cfg(windows)]
impl DirectoryMutationObserver {
    fn start(_directory_chain: &[OpenedDirectoryGuard], _reference: &str) -> Result<Self, String> {
        // OpenedDirectoryGuard holds handles that deny FILE_SHARE_DELETE, so Windows
        // prevents the rename interleaving instead of observing it after the fact.
        Ok(Self)
    }

    fn verify_unchanged(&self, _reference: &str) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(all(
    unix,
    not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))
))]
struct DirectoryMutationObserver;

#[cfg(all(
    unix,
    not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))
))]
impl DirectoryMutationObserver {
    fn start(_directory_chain: &[OpenedDirectoryGuard], reference: &str) -> Result<Self, String> {
        Err(format!(
            "directory mutation observation is unsupported on this platform: {reference}"
        ))
    }

    fn verify_unchanged(&self, _reference: &str) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(windows)]
fn open_directory_rename_guard(path: &Path) -> io::Result<fs::File> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ, FILE_SHARE_WRITE,
    };

    fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
}

fn open_leaf_nofollow(parent: &Dir, leaf: &str) -> io::Result<fs::File> {
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    parent
        .open_with(leaf, &options)
        .map(cap_std::fs::File::into_std)
}

fn open_registered_worktree_leaf<F>(
    canonical_root: &Path,
    parent: Dir,
    directory_chain: Vec<OpenedDirectoryGuard>,
    display_path: &Path,
    leaf: &str,
    reference: &str,
    open_leaf: F,
) -> Result<OpenedRegisteredWorktreeFile, String>
where
    F: FnOnce(&Dir, &str) -> io::Result<fs::File>,
{
    let mutation_observer = DirectoryMutationObserver::start(&directory_chain, reference)?;
    mutation_observer.verify_unchanged(reference)?;
    verify_opened_directory_chain(canonical_root, &directory_chain, reference)?;

    let canonical_evidence = fs::canonicalize(display_path).map_err(|error| {
        format!(
            "cannot resolve outcome evidence {}: {error}",
            display_path.display()
        )
    })?;
    if !canonical_evidence.starts_with(canonical_root) {
        return Err(format!(
            "outcome evidence escapes registered worktree: {reference}"
        ));
    }
    let file = open_leaf(&parent, leaf)
        .map_err(|error| format!("cannot safely open outcome evidence {reference}: {error}"))?;
    verify_opened_directory_chain(canonical_root, &directory_chain, reference)?;
    mutation_observer.verify_unchanged(reference)?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("cannot inspect outcome evidence {reference}: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "outcome evidence is not a regular file: {reference}"
        ));
    }
    Ok(OpenedRegisteredWorktreeFile {
        file,
        canonical_root: canonical_root.to_path_buf(),
        directory_chain,
        reference: reference.to_owned(),
        mutation_observer,
    })
}

fn verify_opened_directory_chain(
    canonical_root: &Path,
    directory_chain: &[OpenedDirectoryGuard],
    reference: &str,
) -> Result<(), String> {
    for guard in directory_chain {
        let canonical_path = fs::canonicalize(&guard.display_path).map_err(|error| {
            format!(
                "cannot resolve outcome evidence parent {}: {error}",
                guard.display_path.display()
            )
        })?;
        if !canonical_path.starts_with(canonical_root) {
            return Err(format!(
                "outcome evidence parent escapes registered worktree: {reference}"
            ));
        }
        let path_metadata = fs::symlink_metadata(&guard.display_path).map_err(|error| {
            format!(
                "cannot inspect outcome evidence parent {}: {error}",
                guard.display_path.display()
            )
        })?;
        if path_metadata.file_type().is_symlink() || !path_metadata.is_dir() {
            return Err(format!(
                "outcome evidence parent is not a contained directory: {reference}"
            ));
        }
        let opened_file = guard
            .directory
            .try_clone()
            .map_err(|error| {
                format!("cannot clone opened evidence directory {reference}: {error}")
            })?
            .into_std_file();
        let displayed_file = open_directory_for_identity(&guard.display_path).map_err(|error| {
            format!("cannot inspect outcome evidence parent {reference}: {error}")
        })?;
        let opened_metadata = opened_file.metadata().map_err(|error| {
            format!("cannot inspect opened evidence parent {reference}: {error}")
        })?;
        let displayed_metadata = displayed_file.metadata().map_err(|error| {
            format!("cannot inspect displayed evidence parent {reference}: {error}")
        })?;
        if !opened_metadata.is_dir()
            || !displayed_metadata.is_dir()
            || file_identity(&opened_file).map_err(|error| {
                format!("cannot identify opened evidence parent {reference}: {error}")
            })? != guard.identity
            || file_identity(&displayed_file).map_err(|error| {
                format!("cannot identify displayed evidence parent {reference}: {error}")
            })? != guard.identity
            || directory_change_token(&opened_file).map_err(|error| {
                format!("cannot observe opened evidence parent {reference}: {error}")
            })? != guard.change_token
            || directory_change_token(&displayed_file).map_err(|error| {
                format!("cannot observe displayed evidence parent {reference}: {error}")
            })? != guard.change_token
        {
            return Err(format!(
                "opened outcome evidence directory changed or no longer matches its contained path: {reference}"
            ));
        }
    }
    Ok(())
}

#[cfg(unix)]
fn open_directory_for_identity(path: &Path) -> io::Result<fs::File> {
    fs::File::open(path)
}

#[cfg(windows)]
fn open_directory_for_identity(path: &Path) -> io::Result<fs::File> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    };

    fs::OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
}

#[cfg(not(any(unix, windows)))]
fn open_directory_for_identity(_path: &Path) -> io::Result<fs::File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "directory handle identity is not supported on this platform",
    ))
}

#[cfg(unix)]
fn file_identity(file: &fs::File) -> io::Result<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;

    let metadata = file.metadata()?;
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
fn file_identity(file: &fs::File) -> io::Result<(u64, u64)> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle,
    };

    let mut information = BY_HANDLE_FILE_INFORMATION::default();
    let succeeded = unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut information) };
    if succeeded == 0 {
        return Err(io::Error::last_os_error());
    }
    let file_index =
        (u64::from(information.nFileIndexHigh) << 32) | u64::from(information.nFileIndexLow);
    Ok((u64::from(information.dwVolumeSerialNumber), file_index))
}

#[cfg(not(any(unix, windows)))]
fn file_identity(_file: &fs::File) -> io::Result<(u64, u64)> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "file handle identity is not supported on this platform",
    ))
}

#[cfg(unix)]
fn directory_change_token(file: &fs::File) -> io::Result<DirectoryChangeToken> {
    #[cfg(test)]
    if let Some(token) = DIRECTORY_CHANGE_TOKEN_OVERRIDE.with(std::cell::Cell::get) {
        return Ok(token);
    }

    use std::os::unix::fs::MetadataExt;

    let metadata = file.metadata()?;
    Ok(DirectoryChangeToken([
        metadata.ctime(),
        metadata.ctime_nsec(),
        metadata.mtime(),
        metadata.mtime_nsec(),
    ]))
}

#[cfg(windows)]
fn directory_change_token(file: &fs::File) -> io::Result<DirectoryChangeToken> {
    #[cfg(test)]
    if let Some(token) = DIRECTORY_CHANGE_TOKEN_OVERRIDE.with(std::cell::Cell::get) {
        return Ok(token);
    }

    use std::mem::size_of;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_BASIC_INFO, FileBasicInfo, GetFileInformationByHandleEx,
    };

    let mut information = FILE_BASIC_INFO::default();
    let succeeded = unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileBasicInfo,
            (&mut information as *mut FILE_BASIC_INFO).cast(),
            size_of::<FILE_BASIC_INFO>() as u32,
        )
    };
    if succeeded == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(DirectoryChangeToken([
        information.ChangeTime,
        information.LastWriteTime,
        0,
        0,
    ]))
}

#[cfg(not(any(unix, windows)))]
fn directory_change_token(_file: &fs::File) -> io::Result<DirectoryChangeToken> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "directory change observation is not supported on this platform",
    ))
}

pub(crate) fn read_registered_worktree_file(
    root: &Path,
    reference: &str,
) -> Result<Vec<u8>, String> {
    read_registered_worktree_file_with_opener(root, reference, open_leaf_nofollow)
}

/// Bound a repository-relative evidence read while retaining the same pinned
/// directory handles and post-read mutation checks as the ordinary reader.
pub(crate) fn read_registered_worktree_file_bounded(
    root: &Path,
    reference: &str,
    maximum_bytes: usize,
) -> Result<Vec<u8>, String> {
    let mut opened = open_registered_worktree_file_with_context(root, reference)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut opened.file)
        .take(maximum_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read bounded evidence {reference}: {error}"))?;
    if bytes.len() > maximum_bytes {
        return Err(format!("evidence exceeds bounded read limit: {reference}"));
    }
    verify_opened_directory_chain(
        &opened.canonical_root,
        &opened.directory_chain,
        &opened.reference,
    )?;
    opened
        .mutation_observer
        .verify_unchanged(&opened.reference)?;
    Ok(bytes)
}

pub(crate) fn read_registered_worktree_file_with_opener<F>(
    root: &Path,
    reference: &str,
    open_leaf: F,
) -> Result<Vec<u8>, String>
where
    F: FnOnce(&Dir, &str) -> io::Result<fs::File>,
{
    let mut opened = open_registered_worktree_file_with_opener(root, reference, open_leaf)?;
    let mut bytes = Vec::new();
    opened
        .file
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read outcome evidence {reference}: {error}"))?;
    verify_opened_directory_chain(
        &opened.canonical_root,
        &opened.directory_chain,
        &opened.reference,
    )?;
    opened
        .mutation_observer
        .verify_unchanged(&opened.reference)?;
    Ok(bytes)
}

fn transitive_invalidated_outcomes(
    projection: &CollaborationProjection,
) -> BTreeMap<ProviderOutcomeKey, Vec<CoordinationEvent>> {
    let mut pending = VecDeque::new();
    for event in projection
        .events
        .iter()
        .filter(|event| event_invalidates(event))
    {
        let Some(provider) = projection
            .registrations
            .iter()
            .find(|registration| registration.work_item_id == event.work_item_id)
        else {
            continue;
        };
        if event.outcome_ids.is_empty() {
            // Legacy events mean "all outcomes", including names removed
            // from the provider's current registration. Recover the affected
            // keys from consumer declarations so invalidation can still
            // propagate through downstream consumers after that removal.
            for consumer in &projection.registrations {
                for dependency in &consumer.declaration.consumed_outcomes {
                    let key = provider_outcome_key(dependency);
                    if key.provider_work_item_id == provider.work_item_id {
                        pending.push_back((event.clone(), key));
                    }
                }
            }
        } else {
            for outcome_id in &event.outcome_ids {
                pending.push_back((
                    event.clone(),
                    ProviderOutcomeKey {
                        provider_work_item_id: provider.work_item_id.clone(),
                        outcome_id: outcome_id.clone(),
                    },
                ));
            }
        }
    }

    let mut invalidated = BTreeMap::<ProviderOutcomeKey, Vec<CoordinationEvent>>::new();
    let mut visited = BTreeSet::new();
    while let Some((root_event, key)) = pending.pop_front() {
        if !visited.insert((root_event.event_id.clone(), key.clone())) {
            continue;
        }
        let events = invalidated.entry(key.clone()).or_default();
        if !events
            .iter()
            .any(|existing| existing.event_id == root_event.event_id)
        {
            events.push(root_event.clone());
        }
        for consumer in &projection.registrations {
            let consumes_invalidated_outcome = consumer
                .declaration
                .consumed_outcomes
                .iter()
                .any(|dependency| provider_outcome_key(dependency) == key);
            if !consumes_invalidated_outcome || recovery_consumed(projection, &root_event, consumer)
            {
                continue;
            }
            for output in &consumer.declaration.provided_outcomes {
                pending.push_back((
                    root_event.clone(),
                    ProviderOutcomeKey {
                        provider_work_item_id: consumer.work_item_id.clone(),
                        outcome_id: output.outcome_id.clone(),
                    },
                ));
            }
        }
    }
    invalidated
}

fn is_outcome_dependency_blocker(blocker: &str) -> bool {
    [
        "dependency_missing:",
        "outcome_missing:",
        "outcome_head_mismatch:",
        "dependency_stage:",
        "outcome_merge_fact_missing:",
        "outcome_merge_fact_unknown:",
        "dependency_impact:",
        "dependency_evidence_missing:",
    ]
    .iter()
    .any(|prefix| blocker.starts_with(prefix))
}

pub fn collaboration_outcome_projection(
    root: &std::path::Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> CollaborationOutcomeProjection {
    let fallback = |state: &str, reason: String| CollaborationOutcomeProjection {
        schema_version: 2,
        work_item_id: work_item_id.into(),
        state: state.into(),
        providers: Vec::new(),
        consumers: Vec::new(),
        waiting_edges: Vec::new(),
        invalidated_event_ids: Vec::new(),
        unhandled_requests: Vec::new(),
        integration_owner: None,
        composition_order: Vec::new(),
        implementation_state: "separate_lifecycle_outcome".into(),
        composition_state: "not_observed".into(),
        execution_outcome: CompositionExecutionOutcome::Unknown,
        execution_evidence_complete: false,
        composition_applicability: "not_observed".into(),
        target_merge_state: "not_observed".into(),
        cleanup_state: "not_observed".into(),
        cleanup_disposition: CompositionCleanupDisposition::Unknown,
        revalidation: "unknown".into(),
        reusable_checks: Vec::new(),
        blockers: Vec::new(),
        unknowns: vec![reason],
        human_decision_required: false,
        next_action: "resolve collaboration capability or recovery unknowns".into(),
    };
    let binding = RuntimeCapabilityBinding {
        schema_version: cockpit_protocol::COLLABORATION_SCHEMA_VERSION,
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        capability: cockpit_protocol::COLLABORATION_CAPABILITY.into(),
    };
    let git = match GitRepository::discover(root) {
        Ok(git) => git,
        Err(error) => return fallback("unknown", format!("repository_topology:{error}")),
    };
    let store = match CoordinationStore::open_read_only(&git, binding) {
        Ok(store) => store,
        Err(error) => return fallback("unsupported", format!("collaboration_store:{error}")),
    };
    let projection = match collaboration_projection(&store) {
        Ok(projection) => projection,
        Err(error) => return fallback("unknown", format!("collaboration_projection:{error}")),
    };
    let current = projection
        .registrations
        .iter()
        .find(|registration| registration.work_item_id == work_item_id);
    let providers = current
        .map(|registration| {
            registration
                .declaration
                .consumed_outcomes
                .iter()
                .map(|dependency| dependency.provider_work_item_id.clone())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    let consumers = projection
        .registrations
        .iter()
        .filter(|registration| {
            registration
                .declaration
                .consumed_outcomes
                .iter()
                .any(|dependency| dependency.provider_work_item_id == work_item_id)
        })
        .map(|registration| registration.work_item_id.clone())
        .collect::<Vec<_>>();
    let waiting_edges = current
        .map(|registration| {
            registration
                .declaration
                .consumed_outcomes
                .iter()
                .map(|dependency| format!("{}->{}", work_item_id, dependency.provider_work_item_id))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let invalidated_outcomes = transitive_invalidated_outcomes(&projection);
    let mut invalidated_ids = BTreeSet::new();
    if let Some(consumer) = current {
        for dependency in &consumer.declaration.consumed_outcomes {
            if let Some(events) = invalidated_outcomes.get(&provider_outcome_key(dependency)) {
                for event in events {
                    if !recovery_consumed(&projection, event, consumer) {
                        invalidated_ids.insert(event.event_id.clone());
                    }
                }
            }
        }
    }
    let invalidated_event_ids = invalidated_ids.into_iter().collect::<Vec<_>>();
    let unhandled_requests = projection
        .requests
        .iter()
        .filter(|request| {
            request.target_work_item_id == work_item_id
                && current.is_some_and(|registration| {
                    registration.generation == request.target_generation
                })
                && matches!(
                    request.state,
                    CoordinationRequestState::Requested | CoordinationRequestState::Acknowledged
                )
        })
        .cloned()
        .collect::<Vec<_>>();
    let admission = current.map(|registration| {
        let action = CollaborationAction {
            kind: CollaborationActionKind::Composition,
            consumer_work_item_id: work_item_id.into(),
            outcomes: registration
                .declaration
                .consumed_outcomes
                .iter()
                .map(provider_outcome_key)
                .collect(),
        };
        evaluate_collaboration_action_from_projection(
            &projection,
            work_item_id,
            registration.generation,
            action,
            Vec::new(),
        )
    });
    let mut admission_unknowns = Vec::new();
    let blockers = match admission {
        Some(admission) => {
            admission_unknowns = admission.unknowns;
            admission.blockers
        }
        None => Vec::new(),
    };
    let mut unknowns = projection.unknowns.clone();
    unknowns.extend(admission_unknowns);
    let (composition, composition_applicability) =
        match latest_composition_attempt(store.root(), work_item_id) {
            Ok(Some(view)) => {
                match &view.child_attempt {
                    Some(attempt) => {
                        let mut facts = composition_facts(store.root(), attempt);
                        if !view.parent_context_valid {
                            // Preserve the raw execution and real cleanup/ECHILD
                            // facts; only parent qualification/reuse is denied.
                            facts.composition_state = "unknown".into();
                            facts.reusable_checks.clear();
                            unknowns.push(format!(
                                "composition_parent_observation:{}",
                                view.attempt_id
                            ));
                        }
                        (
                            facts,
                            composition_applicability(store.root(), attempt, &projection),
                        )
                    }
                    None => {
                        unknowns.push(format!(
                            "composition_parent_observation_without_child:{}",
                            view.attempt_id
                        ));
                        (CompositionProjectionFacts::unknown(), "unknown".into())
                    }
                }
            }
            Ok(None) => (
                CompositionProjectionFacts::not_observed(),
                "not_observed".into(),
            ),
            Err(error) => {
                unknowns.push(format!("composition_projection:{error}"));
                (CompositionProjectionFacts::unknown(), "unknown".into())
            }
        };
    let state = if !unknowns.is_empty() {
        "unknown"
    } else if !blockers.is_empty() {
        "blocked"
    } else {
        "allowed"
    };
    let next_action = if !unknowns.is_empty() {
        "resolve collaboration recovery unknowns before dependent action"
    } else if !blockers.is_empty() {
        "refresh affected dependency and composition evidence"
    } else if !unhandled_requests.is_empty() {
        "acknowledge or safely pause at the next boundary"
    } else {
        "refresh dependency state before the next dependent action"
    };
    CollaborationOutcomeProjection {
        schema_version: 2,
        work_item_id: work_item_id.into(),
        state: state.into(),
        providers,
        consumers,
        waiting_edges,
        invalidated_event_ids: invalidated_event_ids.clone(),
        unhandled_requests: unhandled_requests.clone(),
        integration_owner: current.map(|registration| {
            registration
                .declaration
                .integration_responsibility
                .responsible_work_item_id
                .clone()
        }),
        composition_order: current
            .map(|registration| {
                registration
                    .declaration
                    .integration_responsibility
                    .composition_order
                    .clone()
            })
            .unwrap_or_default(),
        implementation_state: "separate_lifecycle_outcome".into(),
        composition_state: composition.composition_state,
        execution_outcome: composition.execution_outcome,
        execution_evidence_complete: composition.execution_evidence_complete,
        composition_applicability,
        target_merge_state: composition.target_merge_state,
        cleanup_state: composition.cleanup_state,
        cleanup_disposition: composition.cleanup_disposition,
        revalidation: if invalidated_event_ids.is_empty() {
            "not_required"
        } else {
            "required"
        }
        .into(),
        reusable_checks: composition.reusable_checks,
        blockers,
        unknowns,
        // A coordination request is an agent-to-agent protocol state. It is
        // surfaced in `unhandled_requests` and the next action, but it is not
        // a human decision until an explicit human decision record exists.
        human_decision_required: false,
        next_action: next_action.into(),
    }
}

fn latest_composition_attempt(
    coordination_root: &Path,
    work_item_id: &str,
) -> Result<Option<cockpit_verification::CompositionParentView>, String> {
    Ok(load_native_composition_parent_catalog(coordination_root)?
        .latest_for_work_item(work_item_id))
}

struct CompositionProjectionFacts {
    composition_state: String,
    target_merge_state: String,
    cleanup_state: String,
    execution_outcome: CompositionExecutionOutcome,
    execution_evidence_complete: bool,
    cleanup_disposition: CompositionCleanupDisposition,
    reusable_checks: Vec<String>,
}

impl CompositionProjectionFacts {
    fn not_observed() -> Self {
        Self {
            composition_state: "not_observed".into(),
            target_merge_state: "not_observed".into(),
            cleanup_state: "not_observed".into(),
            execution_outcome: CompositionExecutionOutcome::Unknown,
            execution_evidence_complete: false,
            cleanup_disposition: CompositionCleanupDisposition::Unknown,
            reusable_checks: Vec::new(),
        }
    }

    fn unknown() -> Self {
        Self {
            composition_state: "unknown".into(),
            target_merge_state: "unknown".into(),
            cleanup_state: "unknown".into(),
            execution_outcome: CompositionExecutionOutcome::Unknown,
            execution_evidence_complete: false,
            cleanup_disposition: CompositionCleanupDisposition::Unknown,
            reusable_checks: Vec::new(),
        }
    }
}

fn composition_facts(root: &Path, attempt: &CompositionAttempt) -> CompositionProjectionFacts {
    let coherent_success = attempt.is_coherent_successful_terminal();
    let execution_outcome = if attempt.schema_version >= 3 {
        attempt.execution_outcome
    } else if attempt.passed {
        CompositionExecutionOutcome::Passed
    } else if attempt
        .failure
        .as_deref()
        .is_some_and(|failure| failure.starts_with("command_failed:"))
    {
        CompositionExecutionOutcome::Failed
    } else {
        CompositionExecutionOutcome::Unknown
    };
    let execution_evidence_complete =
        attempt.schema_version >= 3 && attempt.execution_evidence_complete;
    let cleanup_disposition = if attempt.schema_version >= 3 {
        attempt.cleanup_disposition
    } else {
        match &attempt.cleanup {
            Some(cleanup) if cleanup.removed => CompositionCleanupDisposition::Cleaned,
            Some(cleanup) if cleanup.attempted => CompositionCleanupDisposition::Failed,
            _ => CompositionCleanupDisposition::Unknown,
        }
    };
    let composition_state = if coherent_success {
        "passed"
    } else if attempt.failure.as_deref() == Some("in_progress") {
        "in_progress"
    } else {
        match execution_outcome {
            CompositionExecutionOutcome::Failed => "failed",
            CompositionExecutionOutcome::Passed | CompositionExecutionOutcome::Unknown => "unknown",
        }
    };
    let target_merge_state = target_merge_state(root, attempt);
    let cleanup_state = match cleanup_disposition {
        CompositionCleanupDisposition::Cleaned => "cleaned",
        CompositionCleanupDisposition::Deferred => "deferred",
        CompositionCleanupDisposition::Retained => "retained",
        CompositionCleanupDisposition::Failed => "failed",
        CompositionCleanupDisposition::Unknown => "unknown",
    };
    let reusable_checks = if coherent_success {
        attempt
            .execution_records
            .iter()
            .filter(|record| record.reused)
            .map(|record| record.node_id.clone())
            .collect()
    } else {
        Vec::new()
    };
    CompositionProjectionFacts {
        composition_state: composition_state.into(),
        target_merge_state,
        cleanup_state: cleanup_state.into(),
        execution_outcome,
        execution_evidence_complete,
        cleanup_disposition,
        reusable_checks,
    }
}

fn target_merge_state(root: &Path, attempt: &CompositionAttempt) -> String {
    let Some(target) = resolve_local_branch(root, &attempt.binding.target_branch) else {
        return "unknown".into();
    };
    if !attempt.text_conflicts.is_empty() {
        return "not_merged".into();
    }
    for participant_head in &attempt.binding.participant_heads {
        let output = Command::new("git")
            .args(["merge-base", "--is-ancestor", participant_head, &target])
            .current_dir(root)
            .output();
        match output {
            Ok(output) if output.status.success() => {}
            Ok(output) if output.status.code() == Some(1) => return "not_merged".into(),
            _ => return "unknown".into(),
        }
    }
    "merged".into()
}

fn composition_applicability(
    root: &Path,
    attempt: &CompositionAttempt,
    projection: &CollaborationProjection,
) -> String {
    let Some(target_head) = resolve_local_branch(root, &attempt.binding.target_branch) else {
        return "unknown".into();
    };
    if target_head != attempt.binding.target_sha {
        return "stale".into();
    }
    for (index, work_item_id) in attempt.binding.participant_work_items.iter().enumerate() {
        let Some(registration) = projection
            .registrations
            .iter()
            .find(|registration| &registration.work_item_id == work_item_id)
        else {
            return "unknown".into();
        };
        if attempt.binding.participant_heads.get(index) != Some(&registration.head)
            || attempt.binding.contract_digests.get(index) != Some(&registration.contract_digest)
        {
            return "stale".into();
        }
    }
    "current".into()
}

fn recovery_consumed(
    projection: &CollaborationProjection,
    event: &CoordinationEvent,
    consumer: &WorktreeRegistration,
) -> bool {
    projection.recoveries.iter().any(|recovery| {
        recovery.event_id == event.event_id
            && recovery.provider_work_item_id == event.work_item_id
            && recovery.provider_generation == event.generation
            && recovery.consumer_work_item_id == consumer.work_item_id
            && recovery.consumer_generation == consumer.generation
    })
}

pub fn refresh_dependency_state(
    store: &CoordinationStore,
    _work_item_id: &str,
    _generation: u64,
) -> Result<CollaborationProjection, CoordinationError> {
    collaboration_projection(store)
}

pub fn admit_collaboration_action(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    action: CollaborationAction,
) -> Result<CollaborationAdmission, CoordinationError> {
    let projection = refresh_dependency_state(store, work_item_id, generation)?;
    let unknowns =
        environment_refresh_unknowns(store, &projection, work_item_id, generation, &action);
    Ok(evaluate_collaboration_action_from_projection(
        &projection,
        work_item_id,
        generation,
        action,
        unknowns,
    ))
}

/// Persist Runtime-observed environment changes and then evaluate the action
/// under the shared coordination lock. Unlike `admit_collaboration_action`,
/// this is a write-bearing pre-action boundary.
pub fn refresh_and_admit_collaboration_action(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    action: CollaborationAction,
) -> Result<CollaborationAdmission, CoordinationError> {
    store.with_lock(|| {
        refresh_and_admit_collaboration_action_under_lock(store, work_item_id, generation, action)
    })
}

pub(crate) fn refresh_and_admit_collaboration_action_under_lock(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    action: CollaborationAction,
) -> Result<CollaborationAdmission, CoordinationError> {
    let mut projection = refresh_dependency_state(store, work_item_id, generation)?;
    let mut environment_unknowns = Vec::new();
    if let Some(registration) = projection
        .registrations
        .iter()
        .find(|registration| registration.work_item_id == work_item_id)
        .filter(|registration| registration.generation == generation)
    {
        let selected_outcomes = action.outcomes.iter().cloned().collect::<BTreeSet<_>>();
        if !registration.declaration.consumed_outcomes.is_empty()
            && !registration.runtime.supports_environment_drift()
        {
            environment_unknowns.push(format!(
                "environment_drift_capability_unsupported:{work_item_id}"
            ));
        }
        for dependency in &registration.declaration.consumed_outcomes {
            let key = provider_outcome_key(dependency);
            if !selected_outcomes.is_empty() && !selected_outcomes.contains(&key) {
                continue;
            }
            let Some(provider) = projection
                .registrations
                .iter()
                .find(|provider| provider.work_item_id == dependency.provider_work_item_id)
            else {
                continue;
            };
            if !provider.runtime.supports_environment_drift() {
                environment_unknowns.push(format!(
                    "environment_drift_capability_unsupported:{}",
                    provider.work_item_id
                ));
                continue;
            }
            let request = cockpit_protocol::EnvironmentDriftRequest {
                schema_version: cockpit_protocol::COLLABORATION_SCHEMA_VERSION,
                work_item_id: provider.work_item_id.clone(),
                expected_generation: provider.generation,
            };
            if let Err(error) = store.record_environment_drift_under_lock(&request) {
                environment_unknowns.push(format!(
                    "environment_refresh:{}:{error}",
                    provider.work_item_id
                ));
            }
        }
        projection = refresh_dependency_state(store, work_item_id, generation)?;
    }
    environment_unknowns.extend(environment_refresh_unknowns(
        store,
        &projection,
        work_item_id,
        generation,
        &action,
    ));
    Ok(evaluate_collaboration_action_from_projection(
        &projection,
        work_item_id,
        generation,
        action,
        environment_unknowns,
    ))
}

fn evaluate_collaboration_action_from_projection(
    projection: &CollaborationProjection,
    work_item_id: &str,
    generation: u64,
    action: CollaborationAction,
    mut extra_unknowns: Vec<String>,
) -> CollaborationAdmission {
    let registration = projection
        .registrations
        .iter()
        .find(|registration| registration.work_item_id == work_item_id);
    let mut blockers = projection
        .blockers
        .get(work_item_id)
        .cloned()
        .unwrap_or_default();
    if action.kind == CollaborationActionKind::ResourceRelease {
        blockers.retain(|blocker| {
            !is_outcome_dependency_blocker(blocker) && !blocker.starts_with("dependency_cycle:")
        });
    }
    let mut unknowns = projection.unknowns.clone();
    unknowns.append(&mut extra_unknowns);
    if registration.is_none() {
        unknowns.push(format!("registration_missing:{work_item_id}"));
    } else if registration.is_some_and(|registration| registration.generation != generation) {
        blockers.push(format!("generation_mismatch:{work_item_id}"));
    } else if action.consumer_work_item_id != work_item_id {
        blockers.push(format!(
            "action_consumer_mismatch:{}",
            action.consumer_work_item_id
        ));
    } else if let Some(registration) = registration {
        if action.kind == CollaborationActionKind::Composition
            && registration
                .declaration
                .integration_responsibility
                .responsible_work_item_id
                != work_item_id
        {
            blockers.push(format!(
                "composition_integration_owner_mismatch:caller={work_item_id}:owner={}",
                registration
                    .declaration
                    .integration_responsibility
                    .responsible_work_item_id
            ));
        }
        if !action.outcomes.is_empty() {
            for key in &action.outcomes {
                if !registration
                    .declaration
                    .consumed_outcomes
                    .iter()
                    .any(|dependency| provider_outcome_key(dependency) == *key)
                {
                    blockers.push(format!(
                        "action_outcome_not_declared:{}:{}",
                        key.provider_work_item_id, key.outcome_id
                    ));
                }
            }
            let selected = action.outcomes.iter().cloned().collect::<BTreeSet<_>>();
            let invalidated_outcomes = transitive_invalidated_outcomes(projection);
            let outcome_blockers = dependency_blockers_for_registration(
                projection,
                registration,
                Some(&selected),
                &invalidated_outcomes,
            );
            blockers.retain(|blocker| !is_outcome_dependency_blocker(blocker));
            blockers.extend(outcome_blockers);
        }
        for request in &projection.requests {
            if request.target_work_item_id == work_item_id
                && request.target_generation == generation
                && request.state == CoordinationRequestState::SafelyPaused
                && action.kind != CollaborationActionKind::ResourceRelease
            {
                blockers.push(format!("coordination_safely_paused:{}", request.request_id));
            }
        }
    }
    let affected = blockers
        .iter()
        .any(|blocker| blocker.starts_with("dependency_impact:"));
    blockers.sort();
    blockers.dedup();
    unknowns.sort();
    unknowns.dedup();
    CollaborationAdmission {
        work_item_id: work_item_id.into(),
        generation,
        allowed: blockers.is_empty() && unknowns.is_empty(),
        affected,
        blockers,
        unknowns,
        refreshed_events: projection.events.len(),
    }
}

fn with_composition_process_start_admission<T, V, F>(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    action: CollaborationAction,
    validate_identity: V,
    spawn: F,
) -> Result<T, String>
where
    V: FnOnce() -> Result<(), String>,
    F: FnOnce() -> Result<T, String>,
{
    store
        .with_lock(|| {
            let admission = refresh_and_admit_collaboration_action_under_lock(
                store,
                work_item_id,
                generation,
                action,
            )
            .map_err(|error| {
                CoordinationError::RecoveryRequired(format!(
                    "composition process-start admission failed: {error}"
                ))
            })?;
            if !admission.allowed {
                let details = admission
                    .blockers
                    .iter()
                    .chain(admission.unknowns.iter())
                    .cloned()
                    .collect::<Vec<_>>();
                return Err(CoordinationError::RecoveryRequired(format!(
                    "composition process start is not admitted: {}",
                    details.join(", ")
                )));
            }
            validate_identity().map_err(CoordinationError::RecoveryRequired)?;
            spawn().map_err(CoordinationError::RecoveryRequired)
        })
        .map_err(|error| error.to_string())
}

pub fn run_admitted_composition(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    input: CompositionInput,
) -> Result<CompositionAttempt, CollaborationExecutionError> {
    let executable = std::env::current_exe().map_err(|error| {
        CollaborationExecutionError::Coordination(CoordinationError::RecoveryRequired(format!(
            "resolve exact Runtime composition helper: {error}"
        )))
    })?;
    let input = prepare_admitted_composition(store, work_item_id, generation, input)?;
    launch_composition_supervisor(
        store,
        work_item_id,
        generation,
        input,
        &executable,
        &["__composition-supervisor".into()],
        &[],
    )
}

#[doc(hidden)]
pub fn run_admitted_composition_with_supervisor_executable(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    input: CompositionInput,
    executable: &Path,
    arguments: &[String],
    environment: &[(String, String)],
) -> Result<CompositionAttempt, CollaborationExecutionError> {
    let input = prepare_admitted_composition(store, work_item_id, generation, input)?;
    launch_composition_supervisor(
        store,
        work_item_id,
        generation,
        input,
        executable,
        arguments,
        environment,
    )
}

pub fn run_composition_supervisor_stdio() -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::fd::BorrowedFd;
        // SAFETY: the process owns these standard handles; cloned Files own
        // their duplicates. Require pipes before changing shared status flags.
        let input = unsafe { BorrowedFd::borrow_raw(libc::STDIN_FILENO) };
        let output = unsafe { BorrowedFd::borrow_raw(libc::STDOUT_FILENO) };
        let mut reader = cockpit_verification::supervisor_pipe::FrameReader::new(
            duplicate_composition_pipe(input)?,
            MAX_COMPOSITION_LAUNCH_BYTES + MAX_COMPOSITION_CONTROL_BYTES,
            MAX_COMPOSITION_LAUNCH_BYTES,
        )
        .map_err(|error| format!("initialize composition input: {error}"))?;
        let mut writer = duplicate_composition_pipe(output)?;
        run_composition_supervisor_protocol(
            |deadline, limit| {
                reader
                    .set_frame_limit(limit)
                    .map_err(|error| error.to_string())?;
                reader
                    .receive_until(deadline)
                    .map_err(|error| error.to_string())
            },
            |bytes, deadline| {
                cockpit_verification::supervisor_pipe::write_until(&mut writer, bytes, deadline)
                    .map_err(|error| format!("write composition protocol: {error}"))
            },
        )
    }
    #[cfg(not(unix))]
    {
        // Windows handle I/O remains a distinct, unproven bounded-I/O backend.
        // The protocol FSM and finite byte policy are shared; no Unix FD is used.
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = stdin.lock();
        let mut writer = stdout.lock();
        run_composition_supervisor_protocol(
            |_deadline, limit| read_legacy_composition_frame(&mut reader, limit),
            |bytes, _deadline| {
                writer
                    .write_all(bytes)
                    .and_then(|()| writer.flush())
                    .map_err(|error| format!("write composition protocol: {error}"))
            },
        )
    }
}

#[cfg(unix)]
fn duplicate_composition_pipe(fd: std::os::fd::BorrowedFd<'_>) -> Result<fs::File, String> {
    use std::os::fd::AsRawFd;
    // SAFETY: fstat only writes this initialized output and observes a live FD.
    let mut metadata: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstat(fd.as_raw_fd(), &mut metadata) } != 0 {
        return Err(format!(
            "inspect composition pipe: {}",
            io::Error::last_os_error()
        ));
    }
    let kind = metadata.st_mode & libc::S_IFMT;
    if kind != libc::S_IFIFO && kind != libc::S_IFSOCK {
        return Err("composition supervisor requires pipe standard handles".into());
    }
    fd.try_clone_to_owned()
        .map(fs::File::from)
        .map_err(|error| format!("duplicate composition pipe: {error}"))
}

pub fn run_admitted_composition_supervisor(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    input: CompositionInput,
    receipt: CompositionSupervisorReceipt,
) -> Result<CompositionAttempt, CollaborationExecutionError> {
    composition_full_tree_backend_available().map_err(recovery_error)?;
    run_admitted_composition_in_process(store, work_item_id, generation, input, receipt)
}

fn prepare_admitted_composition(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    mut input: CompositionInput,
) -> Result<CompositionInput, CollaborationExecutionError> {
    let action = composition_action(store, work_item_id)?;
    let admission =
        refresh_and_admit_collaboration_action(store, work_item_id, generation, action)?;
    if !admission.allowed {
        return Err(CollaborationExecutionError::Blocked {
            work_item_id: work_item_id.into(),
            blockers: admission.blockers,
            unknowns: admission.unknowns,
        });
    }
    let identity_blockers =
        verify_composition_identity(store, work_item_id, generation, &mut input)?;
    if !identity_blockers.is_empty() {
        return Err(CollaborationExecutionError::Blocked {
            work_item_id: work_item_id.into(),
            blockers: identity_blockers,
            unknowns: Vec::new(),
        });
    }
    input.state_dir = store.root().join("compositions");
    Ok(input)
}

fn composition_action(
    store: &CoordinationStore,
    work_item_id: &str,
) -> Result<CollaborationAction, CoordinationError> {
    let outcomes = store
        .inspect()?
        .registrations
        .into_iter()
        .find(|registration| registration.work_item_id == work_item_id)
        .map(|registration| {
            registration
                .declaration
                .consumed_outcomes
                .into_iter()
                .map(|dependency| ProviderOutcomeKey {
                    provider_work_item_id: dependency.provider_work_item_id,
                    outcome_id: dependency.outcome_id,
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(CollaborationAction {
        kind: CollaborationActionKind::Composition,
        consumer_work_item_id: work_item_id.into(),
        outcomes,
    })
}

fn run_admitted_composition_in_process(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    input: CompositionInput,
    supervisor_receipt: CompositionSupervisorReceipt,
) -> Result<CompositionAttempt, CollaborationExecutionError> {
    let input = prepare_admitted_composition(store, work_item_id, generation, input)?;
    let action = composition_action(store, work_item_id)?;
    let check_store = store.clone();
    let check_work_item_id = work_item_id.to_owned();
    let check_action = action.clone();
    let check_input = input.clone();
    let process_admission_check: ProcessAdmissionCheck = Arc::new(move |_node_id: &str, accept| {
        with_composition_process_start_admission(
            &check_store,
            &check_work_item_id,
            generation,
            check_action.clone(),
            || {
                validate_composition_identity_for_process_start(
                    &check_store,
                    &check_work_item_id,
                    generation,
                    &check_input,
                )
            },
            accept,
        )
    });
    let start_store = store.clone();
    let start_work_item_id = work_item_id.to_owned();
    let start_action = action;
    let start_input = input.clone();
    let process_start_gate: ProcessStartGate = Arc::new(
        move |_node_id: &str, spawn: &mut dyn FnMut() -> Result<Child, String>| {
            with_composition_process_start_admission(
                &start_store,
                &start_work_item_id,
                generation,
                start_action.clone(),
                || {
                    validate_composition_identity_for_process_start(
                        &start_store,
                        &start_work_item_id,
                        generation,
                        &start_input,
                    )
                },
                spawn,
            )
        },
    );
    let _owner_interruption_guard = cockpit_verification::OwnerInterruptionGuard::install()?;
    let catalog_loader = || load_native_composition_parent_catalog(store.root());
    Ok(
        cockpit_verification::run_composition_with_supervisor_receipt_and_parent_catalog(
            input,
            process_admission_check,
            process_start_gate,
            supervisor_receipt,
            &catalog_loader,
        )?,
    )
}

fn launch_composition_supervisor(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    input: CompositionInput,
    executable: &Path,
    arguments: &[String],
    extra_environment: &[(String, String)],
) -> Result<CompositionAttempt, CollaborationExecutionError> {
    let actual_runtime_digest = executable_digest(executable).map_err(recovery_error)?;
    let expected_runtime_digest = &input.binding.verifier.runtime_digest;
    if actual_runtime_digest != *expected_runtime_digest
        || input.binding.verifier.runtime_version != env!("CARGO_PKG_VERSION")
    {
        return Err(recovery_error(
            "composition supervisor executable does not match the exact Runtime identity".into(),
        ));
    }
    let owner = current_composition_process_identity().map_err(recovery_error)?;
    let run_nonce = new_composition_run_nonce();
    let attempt_id = new_supervised_composition_attempt_id(&input, &run_nonce);
    let launch = CompositionSupervisorLaunch {
        repository_root: input.repository_root.clone(),
        work_item_id: work_item_id.into(),
        generation,
        input: input.clone(),
        attempt_id: attempt_id.clone(),
        run_nonce,
        owner: owner.clone(),
        runtime_version: input.binding.verifier.runtime_version.clone(),
        runtime_digest: actual_runtime_digest.clone(),
    };
    let bounds = composition_protocol_bounds(&input).map_err(recovery_error)?;
    let launch_bytes =
        encode_composition_frame(&launch, MAX_COMPOSITION_LAUNCH_BYTES).map_err(recovery_error)?;
    // A group-only backend cannot discharge this admitted full-tree operation.
    // This safety rejection is NOT a completed Mac support implementation.
    if let Err(error) = composition_full_tree_backend_available() {
        persist_supervisor_failure(&input, &attempt_id, None, &error)?;
        return Err(recovery_error(error));
    }
    let startup_deadline =
        composition_deadline(COMPOSITION_HANDSHAKE_BUDGET).map_err(recovery_error)?;
    let action = composition_action(store, work_item_id)?;
    let spawn_store = store.clone();
    let spawn_work_item_id = work_item_id.to_owned();
    let spawn_input = input.clone();
    let spawn_executable = executable.to_path_buf();
    let spawn_arguments = arguments.to_vec();
    let spawn_environment = extra_environment.to_vec();
    let spawn_helper = || {
        Command::new(&spawn_executable)
            .args(&spawn_arguments)
            .envs(spawn_environment.iter().cloned())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| format!("spawn exact Runtime composition supervisor: {error}"))
    };
    let attempt_started_at_unix_nanos = parent_observation_now();
    let mut child = with_composition_parent_capacity(&launch, || {
        with_composition_process_start_admission(
            store,
            work_item_id,
            generation,
            action,
            || {
                validate_composition_identity_for_process_start(
                    &spawn_store,
                    &spawn_work_item_id,
                    generation,
                    &spawn_input,
                )
            },
            spawn_helper,
        )
    })
    .map_err(recovery_error)?;
    let mut parent_identity =
        composition_parent_identity(&launch, child.id(), attempt_started_at_unix_nanos);
    let mut parent_phase = "take_pipe";
    let mut release_may_have_been_delivered = false;
    let mut completed_status = None;
    let result = (|| -> Result<CompositionAttempt, CollaborationExecutionError> {
        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                return Err(recovery_error(
                    "composition supervisor stdout is unavailable".into(),
                ));
            }
        };
        let mut stdin = match child.stdin.take() {
            Some(stdin) => stdin,
            None => {
                return Err(recovery_error(
                    "composition supervisor stdin is unavailable".into(),
                ));
            }
        };
        let messages = match supervisor_output_channel(stdout, bounds.reply_bytes) {
            Ok(messages) => messages,
            Err(error) => {
                return Err(recovery_error(error));
            }
        };
        parent_phase = "launch_write";
        if let Err(error) =
            write_composition_bytes_until(&mut stdin, &launch_bytes, startup_deadline)
        {
            return Err(recovery_error(error));
        }
        parent_phase = "ready_read";
        let ready: CompositionSupervisorReady = match read_composition_message_until(
            &messages,
            startup_deadline,
            MAX_COMPOSITION_READY_BYTES,
        ) {
            Ok(ready) => ready,
            Err(error) => {
                return Err(recovery_error(error));
            }
        };
        parent_phase = "ready_identity";
        let observed_supervisor =
            match cockpit_verification::observe_composition_process_identity(child.id()) {
                Ok(identity) => identity,
                Err(error) => {
                    return Err(recovery_error(error));
                }
            };
        parent_identity.observed_supervisor = Some(observed_supervisor.clone());
        if ready.schema_version != 1
            || ready.supervisor.process_id != child.id()
            || ready.supervisor != observed_supervisor
            || ready.runtime_version != launch.runtime_version
            || ready.runtime_digest != launch.runtime_digest
        {
            return Err(recovery_error(
                "composition supervisor ready identity does not match the spawned Runtime".into(),
            ));
        }
        parent_phase = "receipt_binding";
        let receipt = match build_supervisor_receipt(&input, &launch, &ready, extra_environment) {
            Ok(receipt) => receipt,
            Err(error) => {
                return Err(recovery_error(error));
            }
        };
        parent_identity.registered_receipt = Some(receipt.clone());
        parent_phase = "registration_publication";
        if let Err(error) = persist_supervisor_registration(&input.state_dir, &receipt) {
            return Err(recovery_error(error));
        }
        parent_phase = "admission_revalidation";
        let prepared_again =
            prepare_admitted_composition(store, work_item_id, generation, input.clone());
        if let Err(error) = prepared_again {
            let detail = error.to_string();
            let abort_deadline =
                composition_deadline(COMPOSITION_HANDSHAKE_BUDGET).map_err(recovery_error)?;
            let exchange = (|| {
                parent_phase = "abort_write";
                send_composition_control(
                    &mut stdin,
                    &CompositionSupervisorControl::Abort {
                        reason: detail.clone(),
                        receipt: receipt.clone(),
                    },
                    abort_deadline,
                )?;
                parent_phase = "abort_reply";
                let reply = read_terminal_composition_message::<CompositionSupervisorReply>(
                    &messages,
                    abort_deadline,
                    bounds.reply_bytes,
                )?;
                parent_phase = "abort_exit";
                let status = finish_composition_child_until(&mut child, abort_deadline)?;
                completed_status = Some(status);
                Ok::<_, String>((reply, status))
            })();
            let (reply, status) = match exchange {
                Ok(result) => result,
                Err(exchange_error) => {
                    return Err(recovery_error(format!(
                        "{detail}; abort acknowledgement: {exchange_error}"
                    )));
                }
            };
            parent_phase = "abort_validation";
            if !status.success() || reply.error.as_deref() != Some(detail.as_str()) {
                return Err(recovery_error(format!(
                    "composition supervisor abort reply was inconsistent (status={status})"
                )));
            }
            let Some(attempt) = reply.attempt else {
                return Err(recovery_error(
                    "composition supervisor abort reply omitted its durable attempt".into(),
                ));
            };
            if attempt.attempt_id != attempt_id
                || attempt.supervisor_receipt.as_ref() != Some(&receipt)
                || attempt.execution_outcome != CompositionExecutionOutcome::Unknown
                || attempt.passed
                || attempt.processes_spawned != 0
                || !attempt.execution_records.is_empty()
            {
                return Err(recovery_error("composition supervisor abort attempt did not close the registered receipt without spawning a verifier".into()));
            }
            parent_phase = "aborted_before_release";
            return Err(error);
        }
        let execution_deadline =
            composition_deadline(bounds.execution_budget).map_err(recovery_error)?;
        let control_deadline = composition_deadline(COMPOSITION_HANDSHAKE_BUDGET)
            .map_err(recovery_error)?
            .min(execution_deadline);
        parent_phase = "release_write";
        release_may_have_been_delivered = true;
        if let Err(error) = send_composition_control(
            &mut stdin,
            &CompositionSupervisorControl::Release {
                receipt: receipt.clone(),
            },
            control_deadline,
        ) {
            return Err(recovery_error(error));
        }
        // This is the existing normal single-control-frame lifecycle. EOF after
        // Release is NOT a cancellation or evidence that the parent died.
        drop(stdin);
        parent_phase = "reply_read";
        let reply: CompositionSupervisorReply = match read_terminal_composition_message(
            &messages,
            execution_deadline,
            bounds.reply_bytes,
        ) {
            Ok(reply) => reply,
            Err(error) => {
                return Err(recovery_error(error));
            }
        };
        parent_phase = "worker_exit";
        let status = match finish_composition_child_until(&mut child, execution_deadline) {
            Ok(status) => status,
            Err(error) => {
                return Err(recovery_error(error));
            }
        };
        completed_status = Some(status);
        parent_phase = "reply_validation";
        let Some(attempt) = reply.attempt else {
            let error = reply
                .error
                .unwrap_or_else(|| "composition supervisor returned no attempt".into());
            return Err(recovery_error(error));
        };
        let returned_receipt_matches =
            attempt.supervisor_receipt.as_ref().is_some_and(|returned| {
                let mut expected = receipt.clone();
                expected.execution_records_digest =
                    execution_records_digest(&attempt.execution_records);
                expected.descendants_reaped_to_echild = returned.descendants_reaped_to_echild;
                &expected == returned
            });
        if attempt.attempt_id != attempt_id
            || attempt.binding != input.binding
            || attempt.owner_pid != Some(receipt.supervisor.process_id)
            || !returned_receipt_matches
        {
            return Err(recovery_error(
                "composition supervisor returned an attempt with mismatched ownership evidence"
                    .into(),
            ));
        }
        if !status.success() || reply.error.is_some() {
            return Err(recovery_error(reply.error.unwrap_or_else(|| {
                format!("composition supervisor exited with {status}")
            })));
        }
        Ok(attempt)
    })();
    match result {
        Ok(attempt) => Ok(attempt),
        Err(error) => Err(record_failed_parent_operation(
            store.root(),
            &mut child,
            parent_identity,
            parent_phase,
            release_may_have_been_delivered,
            completed_status,
            error,
        )),
    }
}
fn run_composition_supervisor_protocol<R, W>(
    mut read_frame: R,
    mut write_frame: W,
) -> Result<(), String>
where
    R: FnMut(Instant, usize) -> Result<Option<Vec<u8>>, String>,
    W: FnMut(&[u8], Instant) -> Result<(), String>,
{
    composition_full_tree_backend_available()?;
    let backend = cockpit_verification::initialize_composition_supervisor_backend()?;
    let supervisor = current_composition_process_identity()?;
    let executable = std::env::current_exe()
        .map_err(|error| format!("resolve running Runtime executable: {error}"))?;
    let actual_runtime_digest = executable_digest(&executable)?;
    let startup_deadline = composition_deadline(COMPOSITION_HANDSHAKE_BUDGET)?;
    let line = read_frame(startup_deadline, MAX_COMPOSITION_LAUNCH_BYTES)?
        .ok_or("composition supervisor launch channel closed before request")?;
    let launch: CompositionSupervisorLaunch = serde_json::from_slice(&line)
        .map_err(|error| format!("decode composition supervisor launch: {error}"))?;
    let bounds = composition_protocol_bounds(&launch.input)?;
    let runtime_version = env!("CARGO_PKG_VERSION");
    if launch.runtime_version != runtime_version
        || launch.runtime_digest != actual_runtime_digest
        || launch.input.binding.verifier.runtime_version != runtime_version
        || launch.input.binding.verifier.runtime_digest != actual_runtime_digest
        || launch.repository_root != launch.input.repository_root
        || launch.attempt_id.trim().is_empty()
        || launch.run_nonce.trim().is_empty()
    {
        return Err("composition supervisor launch is not bound to this exact Runtime".into());
    }
    let ready = CompositionSupervisorReady {
        schema_version: 1,
        supervisor: supervisor.clone(),
        runtime_version: runtime_version.into(),
        runtime_digest: actual_runtime_digest.clone(),
    };
    send_composition_frame(
        &mut write_frame,
        &ready,
        MAX_COMPOSITION_READY_BYTES,
        startup_deadline,
    )?;
    let control_deadline = composition_deadline(COMPOSITION_HANDSHAKE_BUDGET)?;

    let control_frame = read_frame(control_deadline, MAX_COMPOSITION_CONTROL_BYTES);
    if !matches!(control_frame, Ok(Some(_))) {
        let failure = match &control_frame {
            Ok(None) => "composition supervisor control channel closed before release".to_owned(),
            Err(error) => format!("composition supervisor control failed before release: {error}"),
            Ok(Some(_)) => unreachable!("checked complete control above"),
        };
        let receipt = read_supervisor_registration(&launch.input.state_dir, &launch.attempt_id)?;
        let attempt = cockpit_verification::record_composition_supervisor_failure(
            launch.input.clone(),
            launch.attempt_id.clone(),
            receipt,
            failure.clone(),
        )
        .map_err(|error| format!("persist closed-channel composition attempt: {error}"))?;
        send_composition_frame(
            &mut write_frame,
            &CompositionSupervisorReply {
                attempt: Some(attempt),
                error: Some(failure.clone()),
            },
            bounds.reply_bytes,
            composition_deadline(COMPOSITION_HANDSHAKE_BUDGET)?,
        )?;
        return Err(failure);
    }
    let line = control_frame
        .expect("complete control checked above")
        .expect("present frame checked above");
    let control: CompositionSupervisorControl = serde_json::from_slice(&line)
        .map_err(|error| format!("decode composition supervisor control: {error}"))?;
    let terminal_deadline = composition_deadline(bounds.execution_budget)?;
    let receipt = match control {
        CompositionSupervisorControl::Abort { reason, receipt } => {
            validate_supervisor_registration(&launch.input.state_dir, &receipt)?;
            let attempt = cockpit_verification::record_composition_supervisor_failure(
                launch.input.clone(),
                launch.attempt_id.clone(),
                Some(receipt),
                format!("supervisor_aborted_before_release:{reason}"),
            )
            .map_err(|error| format!("persist aborted composition attempt: {error}"))?;
            send_composition_frame(
                &mut write_frame,
                &CompositionSupervisorReply {
                    attempt: Some(attempt),
                    error: Some(reason),
                },
                bounds.reply_bytes,
                composition_deadline(COMPOSITION_HANDSHAKE_BUDGET)?,
            )?;
            return Ok(());
        }
        CompositionSupervisorControl::Release { receipt } => receipt,
    };
    if let Err(error) = validate_supervisor_release(&launch, &receipt, &supervisor, backend) {
        let attempt = cockpit_verification::record_composition_supervisor_failure(
            launch.input.clone(),
            launch.attempt_id.clone(),
            Some(receipt),
            format!("supervisor_release_invalid:{error}"),
        )
        .ok();
        send_composition_frame(
            &mut write_frame,
            &CompositionSupervisorReply {
                attempt,
                error: Some(error),
            },
            bounds.reply_bytes,
            composition_deadline(COMPOSITION_HANDSHAKE_BUDGET)?,
        )?;
        return Ok(());
    }

    let repository = GitRepository::discover(&launch.repository_root)
        .map_err(|error| format!("discover supervisor repository: {error}"))?;
    let store = CoordinationStore::open(&repository, launch.input.binding.verifier.clone())
        .map_err(|error| format!("open supervisor coordination store: {error}"))?;
    let result = run_admitted_composition_supervisor(
        &store,
        &launch.work_item_id,
        launch.generation,
        launch.input.clone(),
        receipt.clone(),
    );
    match result {
        Ok(attempt) => send_composition_frame(
            &mut write_frame,
            &CompositionSupervisorReply {
                attempt: Some(attempt),
                error: None,
            },
            bounds.reply_bytes,
            terminal_deadline,
        ),
        Err(error) => {
            let detail = error.to_string();
            let attempt = cockpit_verification::record_composition_supervisor_failure(
                launch.input,
                launch.attempt_id,
                Some(receipt),
                detail.clone(),
            )
            .ok();
            send_composition_frame(
                &mut write_frame,
                &CompositionSupervisorReply {
                    attempt,
                    error: Some(detail),
                },
                bounds.reply_bytes,
                terminal_deadline,
            )
        }
    }
}

fn validate_supervisor_release(
    launch: &CompositionSupervisorLaunch,
    receipt: &CompositionSupervisorReceipt,
    supervisor: &cockpit_verification::CompositionProcessIdentity,
    backend: CompositionSupervisorBackend,
) -> Result<(), String> {
    if receipt.schema_version != 1
        || receipt.backend != backend
        || receipt.attempt_id != launch.attempt_id
        || receipt.run_nonce != launch.run_nonce
        || receipt.generation != launch.generation
        || receipt.owner != launch.owner
        || receipt.supervisor != *supervisor
        || receipt.runtime_version != launch.runtime_version
        || receipt.runtime_digest != launch.runtime_digest
        || receipt.repository_id != launch.input.binding.repository_id
        || receipt.target_sha != launch.input.binding.target_sha
        || receipt.command_plan_digest
            != cockpit_verification::composition_commands_digest(&launch.input.commands)
        || receipt.input_environment_digest != input_environment_digest(&launch.input)
        || receipt.environment_digest != process_environment_digest(&[])
        || receipt.snapshot_digest != target_snapshot_digest(&launch.input)?
        || receipt.linux_boot_id != composition_linux_boot_id()?
        || receipt.execution_records_digest != execution_records_digest(&[])
        || receipt.descendants_reaped_to_echild
    {
        return Err(
            "supervisor receipt does not bind the ready Runtime and exact composition input".into(),
        );
    }
    validate_supervisor_registration(&launch.input.state_dir, receipt)
}

fn build_supervisor_receipt(
    input: &CompositionInput,
    launch: &CompositionSupervisorLaunch,
    ready: &CompositionSupervisorReady,
    extra_environment: &[(String, String)],
) -> Result<CompositionSupervisorReceipt, String> {
    let backend = if cfg!(target_os = "linux") {
        CompositionSupervisorBackend::LinuxSubreaper
    } else if cfg!(windows) {
        CompositionSupervisorBackend::WindowsProcessGroup
    } else {
        CompositionSupervisorBackend::UnixProcessGroup
    };
    if backend == CompositionSupervisorBackend::LinuxSubreaper
        && (launch.owner.start_time_ticks.is_none() || ready.supervisor.start_time_ticks.is_none())
    {
        return Err("Linux supervisor receipt lacks PID start-time identities".into());
    }
    Ok(CompositionSupervisorReceipt {
        schema_version: 1,
        backend,
        attempt_id: launch.attempt_id.clone(),
        run_nonce: launch.run_nonce.clone(),
        generation: launch.generation,
        owner: launch.owner.clone(),
        supervisor: ready.supervisor.clone(),
        linux_boot_id: composition_linux_boot_id()?,
        environment_digest: process_environment_digest(extra_environment),
        runtime_version: launch.runtime_version.clone(),
        runtime_digest: launch.runtime_digest.clone(),
        repository_id: input.binding.repository_id.clone(),
        target_sha: input.binding.target_sha.clone(),
        snapshot_digest: target_snapshot_digest(input)?,
        command_plan_digest: cockpit_verification::composition_commands_digest(&input.commands),
        input_environment_digest: input_environment_digest(input),
        execution_records_digest: execution_records_digest(&[]),
        descendants_reaped_to_echild: false,
    })
}

fn target_snapshot_digest(input: &CompositionInput) -> Result<Digest, String> {
    let tree = format!("{}^{{tree}}", input.binding.target_sha);
    let output = Command::new("git")
        .args(["-C"])
        .arg(&input.repository_root)
        .args(["rev-parse", "--verify", &tree])
        .output()
        .map_err(|error| format!("resolve composition target snapshot: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "resolve composition target tree failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(Digest::sha256_bytes(
        String::from_utf8_lossy(&output.stdout).trim().as_bytes(),
    ))
}

fn input_environment_digest(input: &CompositionInput) -> Digest {
    let values = input
        .commands
        .iter()
        .map(|command| (&command.node_id, &command.environment))
        .collect::<Vec<_>>();
    Digest::sha256_bytes(&serde_json::to_vec(&values).expect("command environments serialize"))
}

fn process_environment_digest(extra: &[(String, String)]) -> Digest {
    let mut environment = std::env::vars_os()
        .map(|(key, value)| {
            (
                key.to_string_lossy().into_owned(),
                value.to_string_lossy().into_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    environment.extend(extra.iter().cloned());
    Digest::sha256_bytes(&serde_json::to_vec(&environment).expect("process environment serializes"))
}

fn executable_digest(executable: &Path) -> Result<Digest, String> {
    let bytes = fs::read(executable).map_err(|error| {
        format!(
            "read exact Runtime executable {}: {error}",
            executable.display()
        )
    })?;
    Ok(Digest::sha256_bytes(&bytes))
}

fn persist_supervisor_registration(
    state_dir: &Path,
    receipt: &CompositionSupervisorReceipt,
) -> Result<(), String> {
    let directory = state_dir.join("supervisors");
    fs::create_dir_all(&directory)
        .map_err(|error| format!("create supervisor registration directory: {error}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
            .map_err(|error| format!("restrict supervisor registration directory: {error}"))?;
    }
    let path = supervisor_registration_path(&directory, &receipt.attempt_id);
    let temporary = path.with_extension(format!("{}.tmp", new_composition_run_nonce()));
    let bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|error| format!("encode supervisor registration: {error}"))?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| format!("create supervisor registration: {error}"))?;
    file.write_all(&bytes)
        .map_err(|error| format!("write supervisor registration: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("sync supervisor registration: {error}"))?;
    fs::rename(&temporary, &path)
        .map_err(|error| format!("publish supervisor registration: {error}"))?;
    #[cfg(unix)]
    fs::File::open(&directory)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("sync supervisor registration directory: {error}"))?;
    Ok(())
}

fn validate_supervisor_registration(
    state_dir: &Path,
    expected: &CompositionSupervisorReceipt,
) -> Result<(), String> {
    let registered = read_supervisor_registration(state_dir, &expected.attempt_id)?
        .ok_or_else(|| "durable supervisor registration is missing".to_owned())?;
    if registered != *expected {
        return Err("durable supervisor registration differs from release receipt".into());
    }
    Ok(())
}

fn read_supervisor_registration(
    state_dir: &Path,
    attempt_id: &str,
) -> Result<Option<CompositionSupervisorReceipt>, String> {
    let path = supervisor_registration_path(&state_dir.join("supervisors"), attempt_id);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "read durable supervisor registration {}: {error}",
                path.display()
            ));
        }
    };
    let registered: CompositionSupervisorReceipt = serde_json::from_slice(&bytes)
        .map_err(|error| format!("decode durable supervisor registration: {error}"))?;
    if registered.attempt_id != attempt_id {
        return Err("durable supervisor registration attempt identity mismatch".into());
    }
    Ok(Some(registered))
}

fn supervisor_registration_path(directory: &Path, attempt_id: &str) -> std::path::PathBuf {
    let stem = Digest::sha256_bytes(attempt_id.as_bytes())
        .to_string()
        .replace(':', "-");
    directory.join(format!("{stem}.json"))
}

fn composition_full_tree_backend_available() -> Result<(), String> {
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        Err(
            "composition full-owned-descendant supervision is unsupported on this Unix platform"
                .into(),
        )
    }
    #[cfg(any(not(unix), target_os = "linux"))]
    {
        Ok(())
    }
}

// The same pinned-root, no-follow bounded reader and immutable publisher used
// by execution checkpoints. This store only appends parent facts; child
// composition JSON is never replaced, promoted, tombstoned or deleted here.
#[cfg(test)]
type ParentPublicationHook = Box<dyn FnOnce(&Dir, &str)>;
#[cfg(test)]
thread_local! {
    static BEFORE_PARENT_OBSERVATION_PUBLISH: std::cell::RefCell<Option<ParentPublicationHook>> = const { std::cell::RefCell::new(None) };
}

const MAX_COMPOSITION_PARENT_RECORD_BYTES: u64 = 128 * 1024;
// Encoded event envelope, including all phase/diagnostic/cleanup/action bytes.
// Identity admission reserves this space before spawning. Neither limit permits
// truncation: an oversized actual event returns publication failure with the
// original typed observation in memory, never a durable-Unknown claim.
const MAX_COMPOSITION_PARENT_EVENT_BYTES: usize = 64 * 1024;
const MAX_COMPOSITION_CHILD_RECORD_BYTES: u64 = 128 * 1024 * 1024;

fn composition_parent_identity(
    launch: &CompositionSupervisorLaunch,
    child_pid: u32,
    attempt_started_at_unix_nanos: u128,
) -> cockpit_verification::CompositionParentIdentity {
    cockpit_verification::CompositionParentIdentity {
        attempt_id: launch.attempt_id.clone(),
        run_nonce: launch.run_nonce.clone(),
        binding: launch.input.binding.clone(),
        launch_input_identity: launch.input.identity.clone(),
        generation: launch.generation,
        owner: launch.owner.clone(),
        child_pid,
        observed_supervisor: None,
        registered_receipt: None,
        command_plan_digest: cockpit_verification::composition_commands_digest(
            &launch.input.commands,
        ),
        input_environment_digest: input_environment_digest(&launch.input),
        attempt_started_at_unix_nanos,
    }
}

fn with_composition_parent_capacity<T>(
    launch: &CompositionSupervisorLaunch,
    spawn: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    use cockpit_verification::{
        CompositionParentCleanup, CompositionParentEvent, CompositionParentObservation,
        CompositionParentObservationFacts, CompositionProcessIdentity,
    };
    // These are serialization size bounds, not fabricated process evidence.
    // Ready may add only the observed numeric process identity; the native
    // receipt adds these bounded fields plus strings already fixed by launch.
    let supervisor = CompositionProcessIdentity {
        process_id: i32::MAX as u32,
        start_time_ticks: Some(u64::MAX),
        process_group_id: Some(u32::MAX),
        session_id: Some(u32::MAX),
    };
    let digest = Digest::sha256_bytes(b"parent-record-capacity-only");
    let mut identity = composition_parent_identity(launch, supervisor.process_id, u128::MAX);
    identity.observed_supervisor = Some(supervisor.clone());
    identity.registered_receipt = Some(CompositionSupervisorReceipt {
        schema_version: u32::MAX,
        // Longest current backend spelling, solely for the encoded bound.
        backend: CompositionSupervisorBackend::WindowsProcessGroup,
        attempt_id: launch.attempt_id.clone(),
        run_nonce: launch.run_nonce.clone(),
        generation: launch.generation,
        owner: launch.owner.clone(),
        supervisor,
        // Native boot identity is at most 64 input bytes; controls have the
        // maximum JSON expansion. This value is never used as an identity.
        linux_boot_id: Some("\0".repeat(64)),
        environment_digest: digest.clone(),
        runtime_version: launch.runtime_version.clone(),
        runtime_digest: launch.runtime_digest.clone(),
        repository_id: launch.input.binding.repository_id.clone(),
        target_sha: launch.input.binding.target_sha.clone(),
        snapshot_digest: digest.clone(),
        command_plan_digest: identity.command_plan_digest.clone(),
        input_environment_digest: identity.input_environment_digest.clone(),
        execution_records_digest: digest.clone(),
        descendants_reaped_to_echild: false,
    });
    for event in [
        CompositionParentEvent::Failure {
            phase: String::new(),
            diagnostic: String::new(),
            release_may_have_been_delivered: false,
        },
        CompositionParentEvent::Cleanup {
            failure_observation_id: digest.clone(),
            facts: CompositionParentCleanup {
                child_reaped: Some(false),
                exit_code: Some(i32::MIN),
                termination_signal: Some(i32::MIN),
                actions: Vec::new(),
                diagnostic: Some(String::new()),
            },
        },
    ] {
        let observation = CompositionParentObservation {
            observation_id: digest.clone(),
            facts: CompositionParentObservationFacts {
                schema_version: 1,
                kind: "composition_parent_observation".into(),
                identity: identity.clone(),
                event,
                observed_at_unix_nanos: u128::MAX,
                passed: false,
                post_execution_identity_validated: false,
            },
        };
        let mut writer = LimitedCompositionBuffer {
            bytes: Vec::new(),
            limit: MAX_COMPOSITION_PARENT_RECORD_BYTES as usize
                - MAX_COMPOSITION_PARENT_EVENT_BYTES,
        };
        serde_json::to_writer_pretty(&mut writer, &observation).map_err(|_| {
            "composition parent observation identity cannot reserve the full failure/cleanup event budget before spawn; not truncated".to_owned()
        })?;
    }
    spawn()
}

fn validate_composition_parent_event_capacity(
    event: &cockpit_verification::CompositionParentEvent,
) -> Result<(), String> {
    // Match the actual observation's facts.event indentation. Counting the
    // enclosing keys too is conservative, and all diagnostic bytes are kept.
    #[derive(Serialize)]
    struct Facts<'a> {
        event: &'a cockpit_verification::CompositionParentEvent,
    }
    #[derive(Serialize)]
    struct Envelope<'a> {
        facts: Facts<'a>,
    }
    let mut writer = LimitedCompositionBuffer {
        bytes: Vec::new(),
        limit: MAX_COMPOSITION_PARENT_EVENT_BYTES,
    };
    serde_json::to_writer_pretty(&mut writer, &Envelope { facts: Facts { event } })
        .map_err(|_| "composition parent event exceeds its encoded byte limit; original observation retained in memory, not truncated or durably published".to_owned())
}
const MAX_COMPOSITION_CATALOG_ENTRIES: usize = 4096;
const MAX_COMPOSITION_CATALOG_BYTES: usize = 256 * 1024 * 1024;

struct NativeCompositionParentStore {
    path: std::path::PathBuf,
    pinned_root: crate::legacy_closeout_source::PinnedRoot,
    compositions: Dir,
    compositions_identity: [u64; 2],
    observations: Option<Dir>,
    observations_identity: Option<[u64; 2]>,
}

impl NativeCompositionParentStore {
    fn open(coordination_root: &Path, create: bool) -> Result<Option<Self>, String> {
        let pinned_root = crate::legacy_closeout_source::PinnedRoot::open(coordination_root)
            .map_err(|error| error.to_string())?;
        let root = pinned_root.directory().map_err(|error| error.to_string())?;
        let path = coordination_root.join("compositions");
        let compositions = if create {
            crate::evidence_store::create_and_open_cap_directory(&root, "compositions", &path)
                .map_err(|error| error.to_string())?
        } else {
            match root.symlink_metadata("compositions") {
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(error.to_string()),
                Ok(_) => crate::evidence_store::open_cap_directory_nofollow_strict(
                    &root,
                    "compositions",
                    &path,
                )
                .map_err(|error| error.to_string())?,
            }
        };
        let compositions_identity =
            crate::legacy_closeout_source::directory_identity(&path, &compositions)
                .map_err(|error| error.to_string())?;
        let observations_path = path.join("parent-observations");
        let observations = if create {
            Some(
                crate::evidence_store::create_and_open_cap_directory(
                    &compositions,
                    "parent-observations",
                    &observations_path,
                )
                .map_err(|error| error.to_string())?,
            )
        } else {
            match compositions.symlink_metadata("parent-observations") {
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(error) => return Err(error.to_string()),
                Ok(_) => Some(
                    crate::evidence_store::open_cap_directory_nofollow_strict(
                        &compositions,
                        "parent-observations",
                        &observations_path,
                    )
                    .map_err(|error| error.to_string())?,
                ),
            }
        };
        let observations_identity = observations
            .as_ref()
            .map(|directory| {
                crate::legacy_closeout_source::directory_identity(&observations_path, directory)
                    .map_err(|error| error.to_string())
            })
            .transpose()?;
        let store = Self {
            path,
            pinned_root,
            compositions,
            compositions_identity,
            observations,
            observations_identity,
        };
        store.require_current()?;
        Ok(Some(store))
    }

    fn require_current(&self) -> Result<(), String> {
        self.pinned_root
            .require_current()
            .map_err(|error| error.to_string())?;
        let root = self
            .pinned_root
            .directory()
            .map_err(|error| error.to_string())?;
        let compositions = crate::evidence_store::open_cap_directory_nofollow_strict(
            &root,
            "compositions",
            &self.path,
        )
        .map_err(|error| error.to_string())?;
        if crate::legacy_closeout_source::directory_identity(&self.path, &compositions)
            .map_err(|error| error.to_string())?
            != self.compositions_identity
        {
            return Err("composition parent store directory changed".into());
        }
        let observations_path = self.path.join("parent-observations");
        match self.observations_identity {
            Some(identity) => {
                let current = crate::evidence_store::open_cap_directory_nofollow_strict(
                    &compositions,
                    "parent-observations",
                    &observations_path,
                )
                .map_err(|error| error.to_string())?;
                if crate::legacy_closeout_source::directory_identity(&observations_path, &current)
                    .map_err(|error| error.to_string())?
                    != identity
                {
                    return Err("composition parent observations directory changed".into());
                }
            }
            None => match compositions.symlink_metadata("parent-observations") {
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.to_string()),
                Ok(_) => return Err("composition parent observations appeared during read".into()),
            },
        }
        Ok(())
    }

    fn publish(
        &self,
        observation: &cockpit_verification::CompositionParentObservation,
    ) -> Result<(), String> {
        observation.validate().map_err(|error| error.to_string())?;
        validate_composition_parent_event_capacity(&observation.facts.event)?;
        self.require_current()?;
        let directory = self
            .observations
            .as_ref()
            .ok_or("parent observation store is read-only")?;
        let bytes = serde_json::to_vec_pretty(observation).map_err(|error| error.to_string())?;
        if bytes.len() as u64 > MAX_COMPOSITION_PARENT_RECORD_BYTES {
            return Err(
                "composition parent observation exceeds its byte limit; not truncated".into(),
            );
        }
        let name = format!(
            "{}.json",
            observation
                .observation_id
                .to_string()
                .strip_prefix("sha256:")
                .ok_or("invalid observation digest")?
        );
        #[cfg(test)]
        BEFORE_PARENT_OBSERVATION_PUBLISH.with_borrow_mut(|pending| {
            if let Some(hook) = pending.take() {
                hook(directory, &name);
            }
        });
        crate::evidence_store::write_cap_immutable(
            directory,
            &name,
            &self.path.join("parent-observations").join(&name),
            &bytes,
        )
        .map_err(|error| error.to_string())?;
        self.require_current()?;
        Ok(())
    }

    fn catalog(&self) -> Result<cockpit_verification::CompositionParentCatalog, String> {
        let handle = self
            .compositions
            .try_clone()
            .map_err(|error| error.to_string())?
            .into_std_file();
        let before = directory_change_token(&handle).map_err(|error| error.to_string())?;
        let mut attempts = Vec::new();
        let mut catalog_bytes = 0_usize;
        for (index, entry) in self
            .compositions
            .entries()
            .map_err(|error| error.to_string())?
            .enumerate()
        {
            if index >= MAX_COMPOSITION_CATALOG_ENTRIES {
                return Err("composition catalog entry limit".into());
            }
            let name = entry.map_err(|error| error.to_string())?.file_name();
            if Path::new(&name)
                .extension()
                .and_then(|extension| extension.to_str())
                != Some("json")
            {
                continue;
            }
            let bytes = crate::legacy_closeout_source::read_regular_cap_bounded(
                &self.compositions,
                name.to_str().ok_or("non-UTF8 child record name")?,
                &self.path.join(&name),
                MAX_COMPOSITION_CHILD_RECORD_BYTES,
            )
            .map_err(|error| error.to_string())?;
            catalog_bytes = catalog_bytes
                .checked_add(bytes.len())
                .ok_or("composition catalog byte overflow")?;
            if catalog_bytes > MAX_COMPOSITION_CATALOG_BYTES {
                return Err("composition catalog aggregate byte limit".into());
            }
            let attempt: CompositionAttempt =
                serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
            if !cockpit_verification::composition_attempt_record_name_matches(
                Path::new(&name),
                &attempt.attempt_id,
            ) {
                return Err("child attempt filename/identity mismatch".into());
            }
            attempts.push(attempt);
        }
        let mut observations = Vec::new();
        let mut parent_read_token = None;
        if let Some(directory) = &self.observations {
            let observation_handle = directory
                .try_clone()
                .map_err(|error| error.to_string())?
                .into_std_file();
            let observation_before =
                directory_change_token(&observation_handle).map_err(|error| error.to_string())?;
            for (index, entry) in directory
                .entries()
                .map_err(|error| error.to_string())?
                .enumerate()
            {
                if index >= MAX_COMPOSITION_CATALOG_ENTRIES {
                    return Err("parent observation catalog entry limit".into());
                }
                let name = entry.map_err(|error| error.to_string())?.file_name();
                if Path::new(&name)
                    .extension()
                    .and_then(|extension| extension.to_str())
                    != Some("json")
                {
                    continue;
                }
                let bytes = crate::legacy_closeout_source::read_regular_cap_bounded(
                    directory,
                    name.to_str().ok_or("non-UTF8 observation name")?,
                    &self.path.join("parent-observations").join(&name),
                    MAX_COMPOSITION_PARENT_RECORD_BYTES,
                )
                .map_err(|error| error.to_string())?;
                catalog_bytes = catalog_bytes
                    .checked_add(bytes.len())
                    .ok_or("composition catalog byte overflow")?;
                if catalog_bytes > MAX_COMPOSITION_CATALOG_BYTES {
                    return Err("composition catalog aggregate byte limit".into());
                }
                let observation: cockpit_verification::CompositionParentObservation =
                    serde_json::from_slice(&bytes)
                        .map_err(|error| format!("malformed parent observation: {error}"))?;
                observation.validate().map_err(|error| error.to_string())?;
                validate_composition_parent_event_capacity(&observation.facts.event)?;
                let expected = format!(
                    "{}.json",
                    observation
                        .observation_id
                        .to_string()
                        .strip_prefix("sha256:")
                        .ok_or("invalid digest")?
                );
                if name.to_str() != Some(expected.as_str()) {
                    return Err("parent observation filename/digest mismatch".into());
                }
                observations.push(observation);
            }
            if directory_change_token(&observation_handle).map_err(|error| error.to_string())?
                != observation_before
            {
                return Err("parent observations changed during read".into());
            }
            parent_read_token = Some((observation_handle, observation_before));
        }
        if directory_change_token(&handle).map_err(|error| error.to_string())? != before {
            return Err("child attempt catalog changed during read".into());
        }
        let mut registrations = Vec::new();
        match self.compositions.symlink_metadata("supervisors") {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
            Ok(_) => {
                let path = self.path.join("supervisors");
                let directory = crate::evidence_store::open_cap_directory_nofollow_strict(
                    &self.compositions,
                    "supervisors",
                    &path,
                )
                .map_err(|error| error.to_string())?;
                let handle = directory
                    .try_clone()
                    .map_err(|error| error.to_string())?
                    .into_std_file();
                let token = directory_change_token(&handle).map_err(|error| error.to_string())?;
                for (index, entry) in directory
                    .entries()
                    .map_err(|error| error.to_string())?
                    .enumerate()
                {
                    if index >= MAX_COMPOSITION_CATALOG_ENTRIES {
                        return Err("supervisor registration catalog entry limit".into());
                    }
                    let name = entry.map_err(|error| error.to_string())?.file_name();
                    if Path::new(&name)
                        .extension()
                        .and_then(|extension| extension.to_str())
                        != Some("json")
                    {
                        continue;
                    }
                    let bytes = crate::legacy_closeout_source::read_regular_cap_bounded(
                        &directory,
                        name.to_str().ok_or("non-UTF8 registration name")?,
                        &path.join(&name),
                        MAX_COMPOSITION_READY_BYTES as u64,
                    )
                    .map_err(|error| error.to_string())?;
                    catalog_bytes = catalog_bytes
                        .checked_add(bytes.len())
                        .ok_or("composition catalog byte overflow")?;
                    if catalog_bytes > MAX_COMPOSITION_CATALOG_BYTES {
                        return Err("composition catalog aggregate byte limit".into());
                    }
                    let registration: CompositionSupervisorReceipt =
                        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
                    if supervisor_registration_path(&path, &registration.attempt_id).file_name()
                        != Some(name.as_os_str())
                    {
                        return Err("registration filename/identity mismatch".into());
                    }
                    registrations.push(registration);
                }
                if directory_change_token(&handle).map_err(|error| error.to_string())? != token {
                    return Err("supervisor registrations changed during read".into());
                }
            }
        }
        if directory_change_token(&handle).map_err(|error| error.to_string())? != before {
            return Err("child catalog changed during registration read".into());
        }
        if let Some((parent_handle, token)) = parent_read_token
            && directory_change_token(&parent_handle).map_err(|error| error.to_string())? != token
        {
            return Err("parent observations changed during registration read".into());
        }
        self.require_current()?;
        cockpit_verification::CompositionParentCatalog::from_native_observations(
            attempts,
            observations,
        )
        .and_then(|catalog| catalog.with_native_registrations(registrations))
        .map_err(|error| error.to_string())
    }
}

fn load_native_composition_parent_catalog(
    coordination_root: &Path,
) -> Result<cockpit_verification::CompositionParentCatalog, String> {
    match NativeCompositionParentStore::open(coordination_root, false)? {
        Some(store) => store.catalog(),
        None => cockpit_verification::CompositionParentCatalog::from_native_observations(
            Vec::new(),
            Vec::new(),
        )
        .map_err(|error| error.to_string()),
    }
}

fn publish_native_composition_parent_observation(
    coordination_root: &Path,
    observation: &cockpit_verification::CompositionParentObservation,
) -> Result<(), String> {
    NativeCompositionParentStore::open(coordination_root, true)?
        .ok_or("parent observation store missing")?
        .publish(observation)
}

// These are transport resource limits, not truncated evidence. An oversized
// input is rejected before spawn; an oversized response is Unknown, never pass.
const MAX_COMPOSITION_LAUNCH_BYTES: usize = 8 * 1024 * 1024;
const MAX_COMPOSITION_CONTROL_BYTES: usize = 64 * 1024;
const MAX_COMPOSITION_READY_BYTES: usize = 64 * 1024;
const MAX_COMPOSITION_REPLY_BYTES: usize = 128 * 1024 * 1024;
const MAX_COMPOSITION_DIAGNOSTIC_BYTES: usize = 8192;
const MAX_COMPOSITION_DIAGNOSTIC_LINES: usize = 64;
const COMPOSITION_HANDSHAKE_BUDGET: Duration = Duration::from_secs(30);
const COMPOSITION_CLEANUP_BUDGET: Duration = Duration::from_secs(12);

struct CompositionProtocolBounds {
    reply_bytes: usize,
    execution_budget: Duration,
}

fn composition_deadline(duration: Duration) -> Result<Instant, String> {
    Instant::now()
        .checked_add(duration)
        .ok_or_else(|| "composition protocol deadline overflow".into())
}

fn composition_protocol_bounds(
    input: &CompositionInput,
) -> Result<CompositionProtocolBounds, String> {
    cockpit_verification::validate_timeout_seconds(input.timeout_seconds, "composition")
        .map_err(|error| error.to_string())?;
    let metadata = encode_composition_frame(input, MAX_COMPOSITION_LAUNCH_BYTES)?.len();
    // A captured stream has at most 64KiB input bytes. JSON's worst expansion
    // is six bytes per control character. Include two streams per real node,
    // input-bound metadata and a separate structural/failure allowance.
    let per_record = cockpit_verification::MAX_CAPTURE_BYTES_PER_STREAM
        .checked_mul(12)
        .and_then(|size| size.checked_add(64 * 1024))
        .ok_or("composition reply byte limit overflow")?;
    let reply_bytes = input
        .commands
        .len()
        .checked_mul(per_record)
        .and_then(|size| {
            metadata
                .checked_mul(2)
                .and_then(|meta| size.checked_add(meta))
        })
        .and_then(|size| size.checked_add(8 * 1024 * 1024))
        .ok_or("composition reply byte limit overflow")?;
    if reply_bytes > MAX_COMPOSITION_REPLY_BYTES {
        return Err("composition plan exceeds the bounded reply byte limit".into());
    }
    // Commands execute serially. Preserve each actual declared timeout and add
    // finite per-node cleanup/observation allowance (the existing executor has
    // a 12s supervisor close tail), then preparation/final observation/close.
    // This is a parent watchdog, not a claim that synchronous Git/FS calls are
    // individually interruptible. Unknown at this boundary cannot be green.
    let nodes = u64::try_from(input.commands.len())
        .map_err(|_| "composition node count overflow")?
        .max(1);
    let seconds = input
        .timeout_seconds
        .checked_add(30)
        .and_then(|per_node| per_node.checked_mul(nodes))
        .and_then(|seconds| seconds.checked_add(30 + 30 + 3 + 5 + 2))
        .ok_or("composition whole-plan deadline overflow")?;
    Ok(CompositionProtocolBounds {
        reply_bytes,
        execution_budget: Duration::from_secs(seconds),
    })
}

struct LimitedCompositionBuffer {
    bytes: Vec<u8>,
    limit: usize,
}

impl Write for LimitedCompositionBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "composition protocol byte limit",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn encode_composition_frame<T: Serialize>(message: &T, limit: usize) -> Result<Vec<u8>, String> {
    let mut writer = LimitedCompositionBuffer {
        bytes: Vec::new(),
        limit,
    };
    serde_json::to_writer(&mut writer, message)
        .map_err(|error| format!("encode composition protocol: {error}"))?;
    writer.write_all(b"\n").map_err(|error| error.to_string())?;
    Ok(writer.bytes)
}

fn send_composition_frame<T: Serialize, W>(
    write_frame: &mut W,
    message: &T,
    limit: usize,
    deadline: Instant,
) -> Result<(), String>
where
    W: FnMut(&[u8], Instant) -> Result<(), String>,
{
    let bytes = encode_composition_frame(message, limit)?;
    if Instant::now() >= deadline {
        return Err("composition protocol encoding exceeded its deadline".into());
    }
    write_frame(&bytes, deadline)
}

fn write_composition_bytes_until(
    writer: &mut std::process::ChildStdin,
    bytes: &[u8],
    deadline: Instant,
) -> Result<(), String> {
    #[cfg(unix)]
    {
        cockpit_verification::supervisor_pipe::write_until(writer, bytes, deadline)
            .map_err(|error| format!("write composition protocol: {error}"))
    }
    #[cfg(not(unix))]
    {
        if Instant::now() >= deadline {
            return Err("composition protocol write deadline".into());
        }
        writer
            .write_all(bytes)
            .and_then(|()| writer.flush())
            .map_err(|error| format!("write composition protocol: {error}"))?;
        if Instant::now() >= deadline {
            return Err("composition protocol write exceeded deadline".into());
        }
        Ok(())
    }
}

fn send_composition_control<T: Serialize>(
    writer: &mut std::process::ChildStdin,
    message: &T,
    deadline: Instant,
) -> Result<(), String> {
    let bytes = encode_composition_frame(message, MAX_COMPOSITION_CONTROL_BYTES)?;
    write_composition_bytes_until(writer, &bytes, deadline)
}

trait CompositionMessageSource {
    fn receive_frame_until(
        &self,
        deadline: Instant,
        limit: usize,
    ) -> Result<Option<Vec<u8>>, String>;
}

// Kept for the unchanged Windows transport and deterministic codec tests.
impl CompositionMessageSource for mpsc::Receiver<Result<String, String>> {
    fn receive_frame_until(
        &self,
        deadline: Instant,
        limit: usize,
    ) -> Result<Option<Vec<u8>>, String> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("composition protocol deadline".into());
        }
        match self.recv_timeout(remaining) {
            Ok(Ok(line)) => {
                let mut bytes = line.into_bytes();
                if bytes.len() > limit {
                    return Err("composition protocol byte limit".into());
                }
                if bytes.pop() != Some(b'\n') {
                    return Err("unterminated composition protocol frame".into());
                }
                Ok(Some(bytes))
            }
            Ok(Err(error)) => Err(format!("read composition output: {error}")),
            Err(mpsc::RecvTimeoutError::Disconnected) => Ok(None),
            Err(mpsc::RecvTimeoutError::Timeout) => Err("composition protocol deadline".into()),
        }
    }
}

#[cfg(unix)]
struct CompositionOutputPipe<R = ChildStdout>(
    std::cell::RefCell<cockpit_verification::supervisor_pipe::FrameReader<R>>,
);

#[cfg(unix)]
impl<R: Read + std::os::fd::AsRawFd> CompositionMessageSource for CompositionOutputPipe<R> {
    fn receive_frame_until(
        &self,
        deadline: Instant,
        limit: usize,
    ) -> Result<Option<Vec<u8>>, String> {
        let mut pipe = self.0.borrow_mut();
        pipe.set_frame_limit(limit)
            .map_err(|error| format!("composition output byte limit: {error}"))?;
        pipe.receive_until(deadline)
            .map_err(|error| format!("read composition output: {error}"))
    }
}

#[cfg(unix)]
fn supervisor_output_channel(
    stdout: ChildStdout,
    reply_limit: usize,
) -> Result<CompositionOutputPipe, String> {
    let total = reply_limit
        .checked_add(MAX_COMPOSITION_READY_BYTES)
        .and_then(|limit| {
            limit.checked_add(
                MAX_COMPOSITION_DIAGNOSTIC_BYTES * MAX_COMPOSITION_DIAGNOSTIC_LINES * 3,
            )
        })
        .ok_or("composition output aggregate byte limit overflow")?;
    cockpit_verification::supervisor_pipe::FrameReader::new(
        stdout,
        total,
        MAX_COMPOSITION_READY_BYTES,
    )
    .map(|pipe| CompositionOutputPipe(std::cell::RefCell::new(pipe)))
    .map_err(|error| format!("initialize composition output pipe: {error}"))
}

#[cfg(not(unix))]
fn supervisor_output_channel(
    stdout: ChildStdout,
    reply_limit: usize,
) -> Result<mpsc::Receiver<Result<String, String>>, String> {
    let (sender, receiver) = mpsc::sync_channel(4);
    let total = reply_limit
        .checked_add(MAX_COMPOSITION_READY_BYTES)
        .and_then(|limit| {
            limit.checked_add(
                MAX_COMPOSITION_DIAGNOSTIC_BYTES * MAX_COMPOSITION_DIAGNOSTIC_LINES * 3,
            )
        })
        .ok_or("composition output aggregate byte limit overflow")?;
    std::thread::Builder::new()
        .name("composition-supervisor-output".into())
        .spawn(move || {
            let mut reader = BufReader::new(stdout).take(total.saturating_add(1) as u64);
            loop {
                match read_legacy_composition_frame(&mut reader, total) {
                    Ok(Some(bytes)) => {
                        let mut line = match String::from_utf8(bytes) {
                            Ok(line) => line,
                            Err(_) => {
                                let _ = sender.send(Err("composition output is not UTF-8".into()));
                                break;
                            }
                        };
                        line.push('\n');
                        if sender.send(Ok(line)).is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(error) => {
                        let _ = sender.send(Err(error));
                        break;
                    }
                }
            }
        })
        .map_err(|error| format!("start composition output reader: {error}"))?;
    Ok(receiver)
}

#[cfg(not(unix))]
fn read_legacy_composition_frame<R: BufRead>(
    reader: &mut R,
    limit: usize,
) -> Result<Option<Vec<u8>>, String> {
    let mut bytes = Vec::new();
    let count = Read::by_ref(reader)
        .take(limit.saturating_add(1) as u64)
        .read_until(b'\n', &mut bytes)
        .map_err(|error| format!("read composition protocol: {error}"))?;
    if count == 0 {
        return Ok(None);
    }
    if bytes.len() > limit {
        return Err("composition protocol byte limit".into());
    }
    if bytes.pop() != Some(b'\n') {
        return Err("unterminated composition protocol frame".into());
    }
    Ok(Some(bytes))
}

fn composition_diagnostic_frame(bytes: &[u8]) -> Result<(), String> {
    let line = std::str::from_utf8(bytes).map_err(|_| "composition output is not UTF-8")?;
    if line.trim_start().starts_with('{') || line.trim_start().starts_with('[') {
        return Err("composition emitted malformed or unexpected protocol JSON".into());
    }
    if bytes.len() > MAX_COMPOSITION_DIAGNOSTIC_BYTES {
        return Err("composition diagnostic byte limit".into());
    }
    Ok(())
}

fn read_composition_message_until<T: DeserializeOwned>(
    receiver: &impl CompositionMessageSource,
    deadline: Instant,
    limit: usize,
) -> Result<T, String> {
    let mut bytes = 0_usize;
    for _ in 0..MAX_COMPOSITION_DIAGNOSTIC_LINES {
        if Instant::now() >= deadline {
            return Err("composition protocol deadline".into());
        }
        let frame = receiver
            .receive_frame_until(deadline, limit)?
            .ok_or("composition protocol closed before a complete message")?;
        bytes = bytes.saturating_add(frame.len().saturating_add(1));
        if bytes > limit {
            return Err("composition protocol byte limit".into());
        }
        match serde_json::from_slice::<T>(&frame) {
            Ok(message) => return Ok(message),
            Err(_) => composition_diagnostic_frame(&frame)?,
        }
    }
    Err("composition supervisor emitted too many non-protocol lines".into())
}

fn read_terminal_composition_message<T: DeserializeOwned>(
    receiver: &impl CompositionMessageSource,
    deadline: Instant,
    limit: usize,
) -> Result<T, String> {
    let message = read_composition_message_until(receiver, deadline, limit)?;
    for _ in 0..MAX_COMPOSITION_DIAGNOSTIC_LINES {
        match receiver.receive_frame_until(deadline, MAX_COMPOSITION_DIAGNOSTIC_BYTES)? {
            None => return Ok(message),
            Some(frame) => composition_diagnostic_frame(&frame)?,
        }
    }
    Err("composition supervisor emitted too many trailing diagnostic lines".into())
}

#[cfg(test)]
fn read_supervisor_message<T: DeserializeOwned>(
    receiver: &impl CompositionMessageSource,
    timeout: Option<Duration>,
) -> Result<T, String> {
    let deadline = composition_deadline(timeout.unwrap_or(COMPOSITION_HANDSHAKE_BUDGET))?;
    read_composition_message_until(receiver, deadline, MAX_COMPOSITION_READY_BYTES)
}

#[cfg(test)]
fn read_composition_terminal_until<T: DeserializeOwned>(
    receiver: &impl CompositionMessageSource,
    deadline: Instant,
) -> Result<T, String> {
    read_terminal_composition_message(receiver, deadline, MAX_COMPOSITION_READY_BYTES)
}

fn finish_composition_child_until(
    child: &mut Child,
    deadline: Instant,
) -> Result<std::process::ExitStatus, String> {
    loop {
        if Instant::now() >= deadline {
            return Err(format!(
                "composition supervisor pid={} exit deadline; reaped=false",
                child.id()
            ));
        }
        #[cfg(unix)]
        let exited = cockpit_verification::supervisor_pipe::observe_owned_child_exit(child);
        #[cfg(not(unix))]
        let exited = child.try_wait().map(|status| status.is_some());
        match exited {
            Ok(true) => match child.try_wait() {
                Ok(Some(status)) => return Ok(status),
                Ok(None) => {
                    return Err(format!(
                        "composition supervisor pid={} exit changed; reaped=false",
                        child.id()
                    ));
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => {
                    return Err(format!(
                        "reap owned composition supervisor pid={}: {error}; reaped=unknown",
                        child.id()
                    ));
                }
            },
            Ok(false) => {}
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                return Err(format!(
                    "composition supervisor pid={} wait ownership unknown: {error}; no signal authorized",
                    child.id()
                ));
            }
        }
        std::thread::sleep(
            Duration::from_millis(10).min(deadline.saturating_duration_since(Instant::now())),
        );
    }
}

fn parent_observation_now() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

fn cleanup_facts_from_exit(
    status: std::process::ExitStatus,
    actions: Vec<String>,
) -> cockpit_verification::CompositionParentCleanup {
    #[cfg(unix)]
    let signal = {
        use std::os::unix::process::ExitStatusExt;
        status.signal()
    };
    #[cfg(not(unix))]
    let signal = None;
    cockpit_verification::CompositionParentCleanup {
        child_reaped: Some(true),
        exit_code: status.code(),
        termination_signal: signal,
        actions,
        diagnostic: None,
    }
}

fn cleanup_owned_composition_child(
    child: &mut Child,
    deadline: Instant,
) -> cockpit_verification::CompositionParentCleanup {
    let pid = child.id();
    let mut actions = Vec::new();
    let mut signalled = false;
    let mut killed = false;
    let kill_at = deadline
        .checked_sub(Duration::from_secs(2))
        .unwrap_or(deadline);
    let failed = |actions, reaped, diagnostic| cockpit_verification::CompositionParentCleanup {
        child_reaped: reaped,
        exit_code: None,
        termination_signal: None,
        actions,
        diagnostic: Some(diagnostic),
    };
    loop {
        if Instant::now() >= deadline {
            return failed(
                actions,
                Some(false),
                format!("cleanup pid={pid} reaped=false: cleanup deadline"),
            );
        }
        #[cfg(unix)]
        let exited = cockpit_verification::supervisor_pipe::observe_owned_child_exit(child);
        #[cfg(not(unix))]
        let exited = child.try_wait().map(|status| status.is_some());
        match exited {
            Ok(true) => match child.try_wait() {
                Ok(Some(status)) => return cleanup_facts_from_exit(status, actions),
                Ok(None) => {
                    return failed(
                        actions,
                        None,
                        format!("cleanup pid={pid}: exit observation changed"),
                    );
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => {
                    return failed(
                        actions,
                        None,
                        format!("cleanup pid={pid}: reap ownership unknown: {error}"),
                    );
                }
            },
            Ok(false) => {
                // Exclusive WNOWAIT observation pins the exact PID before each
                // signal. This spawn has no owned helper process group.
                if !signalled {
                    #[cfg(unix)]
                    {
                        actions.push(format!("SIGINT(pid={pid})"));
                        // SAFETY: exact unreaped direct child, never a PGID.
                        if unsafe { libc::kill(pid as libc::pid_t, libc::SIGINT) } != 0 {
                            return failed(
                                actions,
                                Some(false),
                                format!(
                                    "cleanup pid={pid}: INT failed: {}",
                                    io::Error::last_os_error()
                                ),
                            );
                        }
                    }
                    #[cfg(not(unix))]
                    {
                        actions.push(format!("kill(pid={pid})"));
                        if let Err(error) = child.kill() {
                            return failed(
                                actions,
                                Some(false),
                                format!("cleanup pid={pid}: kill failed: {error}"),
                            );
                        }
                    }
                    signalled = true;
                } else if !killed && Instant::now() >= kill_at {
                    actions.push(format!("KILL(pid={pid})"));
                    if let Err(error) = child.kill() {
                        return failed(
                            actions,
                            Some(false),
                            format!("cleanup pid={pid}: KILL failed: {error}"),
                        );
                    }
                    killed = true;
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                return failed(
                    actions,
                    None,
                    format!("cleanup pid={pid}: wait ownership lost: {error}; no further signal"),
                );
            }
        }
        std::thread::sleep(
            Duration::from_millis(10).min(deadline.saturating_duration_since(Instant::now())),
        );
    }
}

fn render_parent_cleanup(
    pid: u32,
    facts: &cockpit_verification::CompositionParentCleanup,
) -> String {
    format!(
        "cleanup pid={pid} reaped={:?} exit={:?} signal={:?} actions={:?} diagnostic={:?}",
        facts.child_reaped,
        facts.exit_code,
        facts.termination_signal,
        facts.actions,
        facts.diagnostic
    )
}

// Compatibility wrapper for the existing finite owned-child test fixtures.
#[cfg(all(test, unix))]
fn terminate_composition_child_until(
    child: &mut Child,
    deadline: Instant,
) -> Result<String, String> {
    let facts = cleanup_owned_composition_child(child, deadline);
    let report = render_parent_cleanup(child.id(), &facts);
    if facts.child_reaped == Some(true) && facts.diagnostic.is_none() {
        Ok(report)
    } else {
        Err(report)
    }
}

fn record_failed_parent_operation(
    coordination_root: &Path,
    child: &mut Child,
    identity: cockpit_verification::CompositionParentIdentity,
    phase: &str,
    release_may_have_been_delivered: bool,
    completed_status: Option<std::process::ExitStatus>,
    primary: CollaborationExecutionError,
) -> CollaborationExecutionError {
    let diagnostic = primary.to_string();
    let failure = cockpit_verification::CompositionParentObservation::new(
        identity.clone(),
        cockpit_verification::CompositionParentEvent::Failure {
            phase: phase.into(),
            diagnostic: diagnostic.clone(),
            release_may_have_been_delivered,
        },
        parent_observation_now(),
    );
    let mut publications = Vec::new();
    let mut publication_errors = Vec::new();
    let failure_id = match failure {
        Ok(failure) => {
            let id = failure.observation_id.clone();
            // Publish the known failure BEFORE attempting any cleanup. A late
            // helper write to its own JSON cannot erase this independent fact.
            if let Err(error) =
                publish_native_composition_parent_observation(coordination_root, &failure)
            {
                publication_errors.push(format!("failure observation publication failed: {error}"));
            }
            publications.push(failure);
            Some(id)
        }
        Err(error) => {
            publication_errors.push(format!("failure observation identity rejected: {error}"));
            None
        }
    };
    let cleanup = match completed_status {
        Some(status) => cleanup_facts_from_exit(status, Vec::new()),
        None => match composition_deadline(COMPOSITION_CLEANUP_BUDGET) {
            Ok(deadline) => cleanup_owned_composition_child(child, deadline),
            Err(error) => cockpit_verification::CompositionParentCleanup {
                child_reaped: None,
                exit_code: None,
                termination_signal: None,
                actions: Vec::new(),
                diagnostic: Some(error),
            },
        },
    };
    let cleanup_report = render_parent_cleanup(child.id(), &cleanup);
    if let Some(failure_observation_id) = failure_id {
        match cockpit_verification::CompositionParentObservation::new(
            identity,
            cockpit_verification::CompositionParentEvent::Cleanup {
                failure_observation_id,
                facts: cleanup,
            },
            parent_observation_now(),
        ) {
            Ok(observation) => {
                if let Err(error) =
                    publish_native_composition_parent_observation(coordination_root, &observation)
                {
                    publication_errors
                        .push(format!("cleanup observation publication failed: {error}"));
                }
                publications.push(observation);
            }
            Err(error) => {
                publication_errors.push(format!("cleanup observation identity rejected: {error}"))
            }
        }
    }
    let message = format!("{diagnostic}; {cleanup_report}");
    if publication_errors.is_empty() {
        recovery_error(message)
    } else {
        // Same boundary as VerificationCheckpointPublication: retain actual
        // observations in memory without claiming they reached durable storage.
        // Display does not dump argv/environment/body material into the CLI.
        CollaborationExecutionError::ParentObservationPublication {
            message: format!("{message}; {}", publication_errors.join("; ")),
            observations: publications,
        }
    }
}

fn persist_supervisor_failure(
    input: &CompositionInput,
    attempt_id: &str,
    receipt: Option<CompositionSupervisorReceipt>,
    failure: &str,
) -> Result<(), CollaborationExecutionError> {
    cockpit_verification::record_composition_supervisor_failure(
        input.clone(),
        attempt_id.into(),
        receipt,
        failure.into(),
    )?;
    Ok(())
}

fn recovery_error(detail: String) -> CollaborationExecutionError {
    CoordinationError::RecoveryRequired(detail).into()
}

fn validate_composition_identity_for_process_start(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    input: &CompositionInput,
) -> Result<(), String> {
    let mut current_input = input.clone();
    let blockers = verify_composition_identity(store, work_item_id, generation, &mut current_input)
        .map_err(|error| error.to_string())?;
    if blockers.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "composition identity changed before process start: {}",
            blockers.join(", ")
        ))
    }
}

fn read_registered_contract(
    registration: &WorktreeRegistration,
) -> Result<cockpit_protocol::Contract, CoordinationError> {
    read_registered_contract_with_reader(registration, &mut |root, reference| {
        read_registered_worktree_file(root, reference)
    })
}

fn environment_refresh_unknowns(
    store: &CoordinationStore,
    projection: &CollaborationProjection,
    work_item_id: &str,
    generation: u64,
    action: &CollaborationAction,
) -> Vec<String> {
    let Some(consumer) = projection
        .registrations
        .iter()
        .find(|registration| registration.work_item_id == work_item_id)
        .filter(|registration| registration.generation == generation)
    else {
        return Vec::new();
    };
    if consumer.declaration.consumed_outcomes.is_empty() {
        return Vec::new();
    }
    let mut unknowns = Vec::new();
    if !consumer.runtime.supports_environment_drift() {
        unknowns.push(format!(
            "environment_drift_capability_unsupported:{work_item_id}"
        ));
    }
    let selected_outcomes = action.outcomes.iter().cloned().collect::<BTreeSet<_>>();
    for dependency in &consumer.declaration.consumed_outcomes {
        let key = provider_outcome_key(dependency);
        if !selected_outcomes.is_empty() && !selected_outcomes.contains(&key) {
            continue;
        }
        let Some(provider) = projection
            .registrations
            .iter()
            .find(|provider| provider.work_item_id == dependency.provider_work_item_id)
        else {
            continue;
        };
        if !provider.runtime.supports_environment_drift() {
            unknowns.push(format!(
                "environment_drift_capability_unsupported:{}",
                provider.work_item_id
            ));
            continue;
        }
        let request = cockpit_protocol::EnvironmentDriftRequest {
            schema_version: cockpit_protocol::COLLABORATION_SCHEMA_VERSION,
            work_item_id: provider.work_item_id.clone(),
            expected_generation: provider.generation,
        };
        match store.environment_drift_pending(&request) {
            Ok(true) => unknowns.push(format!(
                "environment_drift_unrecorded:{}",
                provider.work_item_id
            )),
            Ok(false) => {}
            Err(error) => unknowns.push(format!(
                "environment_refresh:{}:{error}",
                provider.work_item_id
            )),
        }
    }
    unknowns
}

fn read_registered_contract_with_reader<F>(
    registration: &WorktreeRegistration,
    read_file: &mut F,
) -> Result<cockpit_protocol::Contract, CoordinationError>
where
    F: FnMut(&Path, &str) -> Result<Vec<u8>, String>,
{
    let root = Path::new(&registration.worktree_path);
    let reference = format!(
        ".ai/work-items/active/{}.contract.json",
        registration.work_item_id
    );
    let bytes = read_file(root, &reference).map_err(|error| {
        CoordinationError::RecoveryRequired(format!(
            "registered Contract is not safely readable for {}: {error}",
            registration.work_item_id
        ))
    })?;
    let path = root.join(&reference);
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
        CoordinationError::RecoveryRequired(format!(
            "registered Contract digest is unavailable for {}: {error}",
            registration.work_item_id
        ))
    })?;
    let actual_digest = cockpit_protocol::digest_json(&value).map_err(|error| {
        CoordinationError::RecoveryRequired(format!(
            "registered Contract digest cannot be calculated for {}: {error}",
            registration.work_item_id
        ))
    })?;
    if actual_digest != registration.contract_digest {
        return Err(CoordinationError::RecoveryRequired(format!(
            "registered Contract digest mismatch for {}",
            registration.work_item_id
        )));
    }
    let contract = crate::parse_contract_bytes(&bytes, &path).map_err(|error| {
        CoordinationError::RecoveryRequired(format!(
            "registered Contract is invalid for {}: {error}",
            registration.work_item_id
        ))
    })?;
    crate::coordination_store::validate_contract_registration_identity(registration, &contract)?;
    Ok(contract)
}

fn parse_required_check_identity(check: &str) -> Option<(String, Vec<String>)> {
    if check
        .chars()
        .any(|character| character.is_control() || matches!(character, '\'' | '"' | '\\'))
    {
        return None;
    }
    let mut parts = check.split_whitespace();
    let program = parts.next()?.to_owned();
    let args = parts.map(str::to_owned).collect();
    Some((program, args))
}

fn verify_composition_identity(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    input: &mut CompositionInput,
) -> Result<Vec<String>, CoordinationError> {
    let inspection = store.inspect()?;
    let mut blockers = inspection
        .unknowns
        .into_iter()
        .map(|unknown| format!("coordination_unknown:{unknown}"))
        .collect::<Vec<_>>();
    let registrations = inspection
        .registrations
        .into_iter()
        .map(|registration| (registration.work_item_id.clone(), registration))
        .collect::<BTreeMap<_, _>>();
    let Some(target) = registrations.get(work_item_id) else {
        blockers.push(format!("registration_missing:{work_item_id}"));
        return Ok(blockers);
    };
    if target.generation != generation {
        blockers.push(format!("generation_mismatch:{work_item_id}"));
    }
    let integration_owner = &target
        .declaration
        .integration_responsibility
        .responsible_work_item_id;
    if integration_owner != work_item_id {
        blockers.push(format!(
            "composition_integration_owner_mismatch:caller={work_item_id}:owner={integration_owner}"
        ));
    }
    match registrations.get(integration_owner) {
        None => blockers.push(format!(
            "composition_integration_owner_registration_missing:{integration_owner}"
        )),
        Some(owner_registration)
            if owner_registration
                .declaration
                .integration_responsibility
                .responsible_work_item_id
                != *integration_owner =>
        {
            blockers.push(format!(
                "composition_integration_owner_declaration_mismatch:{integration_owner}"
            ));
        }
        Some(_) => {}
    }
    let execution_topology = GitRepository::discover(&input.repository_root)
        .and_then(|repository| repository.topology())
        .map_err(|error| {
            CoordinationError::RecoveryRequired(format!("composition topology: {error}"))
        })?;
    if execution_topology.common_dir != store.git_common_dir() {
        blockers.push("composition_repository_common_directory_mismatch".into());
    }
    if input.binding.repository_id != target.repository_id {
        blockers.push("composition_repository_identity_mismatch".into());
    }
    if input.binding.target_branch != target.declaration.integration_responsibility.target_branch {
        blockers.push("composition_target_branch_mismatch".into());
    }
    if execution_topology.common_dir == store.git_common_dir() {
        let resolved_target =
            resolve_local_branch(&input.repository_root, &input.binding.target_branch);
        if resolved_target.as_deref() != Some(input.binding.target_sha.as_str()) {
            blockers.push("composition_target_head_mismatch".into());
        }
    }
    if input.binding.participant_work_items.is_empty()
        || input.binding.participant_work_items.len() != input.binding.participant_heads.len()
        || input.binding.participant_work_items.len() != input.binding.contract_digests.len()
    {
        blockers.push("composition_participant_coverage_incomplete".into());
    }
    let mut seen_participants = BTreeSet::new();
    for (index, participant_id) in input.binding.participant_work_items.iter().enumerate() {
        if !seen_participants.insert(participant_id) {
            blockers.push(format!(
                "composition_participant_duplicate:{participant_id}"
            ));
            continue;
        }
        let Some(registration) = registrations.get(participant_id) else {
            blockers.push(format!("composition_participant_missing:{participant_id}"));
            continue;
        };
        if input.binding.participant_heads.get(index) != Some(&registration.head) {
            blockers.push(format!(
                "composition_participant_head_mismatch:{participant_id}"
            ));
        }
        if input.binding.contract_digests.get(index) != Some(&registration.contract_digest) {
            blockers.push(format!(
                "composition_participant_contract_mismatch:{participant_id}"
            ));
        }
    }
    let required_participants = dependency_closure(&registrations, work_item_id);
    let declared_participants = input
        .binding
        .participant_work_items
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if declared_participants != required_participants {
        blockers.push("composition_dependency_closure_incomplete".into());
    }
    let composition_order = &target
        .declaration
        .integration_responsibility
        .composition_order;
    if !composition_order.is_empty() && composition_order != &input.binding.participant_work_items {
        blockers.push("composition_order_mismatch".into());
    }
    let mut required_check_identities = Vec::<(
        String,
        (String, Vec<String>),
        BTreeSet<String>,
        BTreeSet<String>,
    )>::new();
    let mut seen_required_check_identities = BTreeSet::new();
    for participant_id in &input.binding.participant_work_items {
        let Some(participant) = registrations.get(participant_id) else {
            continue;
        };
        let contract = read_registered_contract(participant)?;
        let explicit_required_checks = contract
            .verification
            .iter()
            .filter_map(|declaration| match declaration {
                cockpit_protocol::VerificationDeclaration::Check(check) if check.required => {
                    Some(check)
                }
                cockpit_protocol::VerificationDeclaration::Legacy(_)
                | cockpit_protocol::VerificationDeclaration::Check(_) => None,
            })
            .filter(|check| !check.check.trim().is_empty())
            .collect::<Vec<_>>();
        let complete_required_checks = crate::required_verification_checks(&contract);
        if complete_required_checks.is_empty() {
            blockers.push(format!(
                "required_check_declaration_missing:{participant_id}"
            ));
        }
        let explicit_check_set = explicit_required_checks
            .iter()
            .map(|check| check.check.trim().to_owned())
            .collect::<BTreeSet<_>>();
        for check in complete_required_checks {
            if !explicit_check_set.contains(&check) {
                blockers.push(format!(
                    "required_check_identity_unresolvable:{participant_id}:{check}"
                ));
            }
        }
        for check in explicit_required_checks {
            let check_name = check.check.trim().to_owned();
            let Some(identity) = parse_required_check_identity(&check_name) else {
                blockers.push(format!(
                    "required_check_identity_unresolvable:{participant_id}:{check_name}"
                ));
                continue;
            };
            if !seen_required_check_identities.insert(identity.clone()) {
                blockers.push(format!(
                    "required_check_identity_ambiguous:{participant_id}:{}",
                    check_name
                ));
                continue;
            }
            let scenarios = check
                .covers_scenarios
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            let constraints = check
                .covers_constraints
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            if scenarios.len() != check.covers_scenarios.len()
                || check
                    .covers_scenarios
                    .iter()
                    .any(|scenario| scenario.trim().is_empty())
                || constraints.len() != check.covers_constraints.len()
                || check
                    .covers_constraints
                    .iter()
                    .any(|constraint| constraint.trim().is_empty())
            {
                blockers.push(format!(
                    "required_check_coverage_invalid:{participant_id}:{check_name}"
                ));
                continue;
            }
            required_check_identities.push((
                participant_id.clone(),
                identity,
                scenarios,
                constraints,
            ));
        }
    }
    let mut node_ids = BTreeSet::new();
    let mut prior_node_ids = BTreeSet::new();
    if input.commands.is_empty() {
        blockers.push("required_checks_empty".into());
    }
    for command in &input.commands {
        if command.node_id.trim().is_empty() || !node_ids.insert(command.node_id.clone()) {
            blockers.push(format!(
                "required_check_identity_invalid:{}",
                command.node_id
            ));
        }
        if command.program.trim().is_empty() {
            blockers.push(format!(
                "required_check_program_missing:{}",
                command.node_id
            ));
        }
        let mut dependencies = BTreeSet::new();
        for dependency in &command.depends_on {
            if dependency.trim().is_empty()
                || !dependencies.insert(dependency)
                || !prior_node_ids.contains(dependency)
            {
                blockers.push(format!(
                    "composition_dependency_order_invalid:{}:{}",
                    command.node_id, dependency
                ));
            }
        }
        prior_node_ids.insert(command.node_id.clone());
    }
    let mut seen_command_identities = BTreeSet::new();
    for command in &input.commands {
        let identity = (command.program.clone(), command.args.clone());
        if !seen_command_identities.insert(identity.clone()) {
            blockers.push(format!(
                "composition_command_identity_ambiguous:{}",
                command.node_id
            ));
        }
    }
    if input.commands.len() != required_check_identities.len() {
        blockers.push(format!(
            "required_check_set_incomplete:expected={}:actual={}",
            required_check_identities.len(),
            input.commands.len()
        ));
    }
    let mut covered_scenarios_by_participant = BTreeMap::<String, BTreeSet<String>>::new();
    let mut covered_constraints_by_participant = BTreeMap::<String, BTreeSet<String>>::new();
    for (command, (participant_id, required_identity, declared_scenarios, declared_constraints)) in
        input.commands.iter().zip(&required_check_identities)
    {
        if (command.program.clone(), command.args.clone()) == *required_identity {
            // Required checks have no Contract-owned environment declaration.
            // A composition caller must not be able to change the behavior of
            // an otherwise matching check with an unbound environment overlay.
            if !command.environment.is_empty() {
                blockers.push(format!(
                    "required_check_environment_unbound:{}",
                    command.node_id
                ));
            }
            let scenario_labels = command
                .covered_scenarios
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            let constraint_labels = command
                .covered_constraints
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            if scenario_labels.len() != command.covered_scenarios.len()
                || scenario_labels
                    .iter()
                    .any(|scenario| !declared_scenarios.contains(scenario))
            {
                blockers.push(format!(
                    "composition_scenario_label_unbound:{}",
                    command.node_id
                ));
            }
            if constraint_labels.len() != command.covered_constraints.len()
                || constraint_labels
                    .iter()
                    .any(|constraint| !declared_constraints.contains(constraint))
            {
                blockers.push(format!(
                    "composition_constraint_label_unbound:{}",
                    command.node_id
                ));
            }
            covered_scenarios_by_participant
                .entry(participant_id.clone())
                .or_default()
                .extend(declared_scenarios.iter().cloned());
            covered_constraints_by_participant
                .entry(participant_id.clone())
                .or_default()
                .extend(declared_constraints.iter().cloned());
        } else {
            blockers.push(format!(
                "required_check_identity_mismatch:{}",
                command.node_id
            ));
        }
    }
    for participant_id in &input.binding.participant_work_items {
        let Some(participant) = registrations.get(participant_id) else {
            continue;
        };
        let covered_scenarios = covered_scenarios_by_participant
            .get(participant_id)
            .cloned()
            .unwrap_or_default();
        let covered_constraints = covered_constraints_by_participant
            .get(participant_id)
            .cloned()
            .unwrap_or_default();
        for scenario in &participant
            .declaration
            .composition_verification
            .required_scenarios
        {
            if !covered_scenarios.contains(scenario) {
                blockers.push(format!("required_scenario_uncovered:{scenario}"));
            }
        }
        for constraint in &participant
            .declaration
            .composition_verification
            .compatibility_constraints
        {
            if !covered_constraints.contains(constraint) {
                blockers.push(format!("compatibility_constraint_uncovered:{constraint}"));
            }
        }
    }
    let declared_reusable_nodes = target
        .declaration
        .composition_verification
        .reusable_nodes
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for node_id in &declared_reusable_nodes {
        if !node_ids.contains(node_id) {
            blockers.push(format!("reusable_node_missing:{node_id}"));
        }
    }
    input.reusable_node_ids = input
        .commands
        .iter()
        .filter(|command| declared_reusable_nodes.contains(&command.node_id))
        .map(|command| command.node_id.clone())
        .collect();
    blockers.sort();
    blockers.dedup();
    if blockers.is_empty() {
        // The caller's boolean preconditions are not an authorization source.
        // Reconstruct them only after observing registrations, Git topology,
        // participant Contracts, and complete required-check coverage.
        input.preconditions = vec![
            CompositionPrecondition::satisfied("registration-identity-observed"),
            CompositionPrecondition::satisfied("composition-target-observed"),
            CompositionPrecondition::satisfied("required-checks-covered"),
        ];
    }
    Ok(blockers)
}

fn resolve_local_branch(root: &Path, branch: &str) -> Option<String> {
    let checked = Command::new("git")
        .args(["check-ref-format", "--branch", branch])
        .current_dir(root)
        .output()
        .ok()?;
    if !checked.status.success() {
        return None;
    }
    let reference = format!("refs/heads/{branch}^{{commit}}");
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "--end-of-options", &reference])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn commit_is_ancestor(root: &Path, branch: &str, commit: &str) -> Option<bool> {
    if commit.len() != 40 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Some(false);
    }
    let target = resolve_local_branch(root, branch)?;
    let output = Command::new("git")
        .args(["merge-base", "--is-ancestor", commit, &target])
        .current_dir(root)
        .output()
        .ok()?;
    match output.status.code() {
        Some(0) => Some(true),
        Some(1) => Some(false),
        _ => None,
    }
}

fn dependency_closure(
    registrations: &BTreeMap<String, WorktreeRegistration>,
    starting_work_item_id: &str,
) -> BTreeSet<String> {
    let mut required = BTreeSet::new();
    let mut pending = vec![starting_work_item_id.to_owned()];
    while let Some(work_item_id) = pending.pop() {
        if !required.insert(work_item_id.clone()) {
            continue;
        }
        if let Some(registration) = registrations.get(&work_item_id) {
            pending.extend(
                registration
                    .declaration
                    .consumed_outcomes
                    .iter()
                    .map(|dependency| dependency.provider_work_item_id.clone()),
            );
        }
    }
    required
}

pub fn report_impact(
    store: &CoordinationStore,
    event: CoordinationEvent,
) -> Result<CoordinationEvent, CoordinationError> {
    if event.kind != cockpit_protocol::CoordinationEventKind::Impact {
        return Err(CoordinationError::RecoveryRequired(
            "report_impact accepts only Impact events; use publish_outcome for OutcomePublished events"
                .into(),
        ));
    }
    store.publish_event(event)
}

pub fn publish_outcome(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    outcome_id: &str,
) -> Result<CoordinationEvent, CoordinationError> {
    publish_outcome_with_pre_append(store, work_item_id, generation, outcome_id, || {})
}

fn publish_outcome_with_pre_append<F>(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    outcome_id: &str,
    before_append: F,
) -> Result<CoordinationEvent, CoordinationError>
where
    F: FnOnce(),
{
    let projection = collaboration_projection(store)?;
    if !projection.unknowns.is_empty() {
        return Err(CoordinationError::RecoveryRequired(format!(
            "cannot publish an outcome while coordination facts are unknown: {}",
            projection.unknowns.join(", ")
        )));
    }
    let provider = projection
        .registrations
        .iter()
        .find(|registration| registration.work_item_id == work_item_id)
        .cloned()
        .ok_or_else(|| {
            CoordinationError::RecoveryRequired(format!(
                "outcome publisher requires a current registration for {work_item_id}"
            ))
        })?;
    if provider.generation != generation {
        return Err(CoordinationError::StaleGeneration {
            work_item_id: work_item_id.into(),
            expected: provider.generation,
            actual: generation,
        });
    }
    let outcome = provider
        .declaration
        .provided_outcomes
        .iter()
        .find(|outcome| outcome.outcome_id == outcome_id)
        .ok_or_else(|| {
            CoordinationError::RecoveryRequired(format!(
                "outcome {outcome_id} is not declared by {work_item_id}"
            ))
        })?;
    if outcome.published_head != provider.head {
        return Err(CoordinationError::RecoveryRequired(format!(
            "outcome {outcome_id} head does not match the current registration for {work_item_id}"
        )));
    }

    let root = Path::new(&provider.worktree_path);
    let mut evidence_digests = BTreeMap::new();
    for reference in &outcome.evidence_refs {
        let bytes = read_registered_worktree_file(root, reference).map_err(|error| {
            CoordinationError::RecoveryRequired(format!(
                "outcome evidence is not safely contained: {reference}: {error}"
            ))
        })?;
        if evidence_digests
            .insert(
                reference.clone(),
                cockpit_core::Digest::sha256_bytes(&bytes),
            )
            .is_some()
        {
            return Err(CoordinationError::RecoveryRequired(format!(
                "outcome evidence reference is duplicated: {reference}"
            )));
        }
    }
    let identity_bytes = serde_json::to_vec(&(
        &provider.repository_id,
        work_item_id,
        generation,
        outcome_id,
        &provider.head,
        &provider.contract_digest,
        &evidence_digests,
    ))
    .map_err(|error| CoordinationError::RecoveryRequired(error.to_string()))?;
    let identity_digest = cockpit_core::Digest::sha256_bytes(&identity_bytes).to_string();
    let digest_suffix = identity_digest
        .strip_prefix("sha256:")
        .unwrap_or(&identity_digest);
    let event = CoordinationEvent {
        schema_version: cockpit_protocol::COLLABORATION_SCHEMA_VERSION,
        event_id: format!("outcome-{work_item_id}-{generation}-{outcome_id}-{digest_suffix}"),
        repository_id: provider.repository_id.clone(),
        work_item_id: work_item_id.into(),
        generation,
        kind: cockpit_protocol::CoordinationEventKind::OutcomePublished,
        source: "runtime-publish-outcome".into(),
        evidence_refs: outcome.evidence_refs.clone(),
        evidence_digests,
        outcome_ids: vec![outcome_id.into()],
        environment_change: None,
    };
    let published_key = ProviderOutcomeKey {
        provider_work_item_id: work_item_id.into(),
        outcome_id: outcome_id.into(),
    };
    before_append();
    store.publish_validated_outcome_event(
        event,
        |publication, inspection, current_provider, bound_evidence| {
            if current_provider != &provider {
                return Err(CoordinationError::RecoveryRequired(format!(
                    "provider registration for {work_item_id} changed while preparing publication"
                )));
            }
            if !inspection.unknowns.is_empty() {
                return Err(CoordinationError::RecoveryRequired(format!(
                    "cannot publish an outcome while coordination facts are unknown: {}",
                    inspection.unknowns.join(", ")
                )));
            }
            let verification_required = inspection.registrations.iter().any(|consumer| {
                consumer
                    .declaration
                    .consumed_outcomes
                    .iter()
                    .any(|dependency| {
                        provider_outcome_key(dependency) == published_key
                            && dependency.verification_required
                    })
            });
            if verification_required
                && !verification_evidence_matches_publication_with_bytes(
                    publication,
                    current_provider,
                    outcome,
                    bound_evidence,
                    &mut read_registered_worktree_file,
                )
            {
                return Err(CoordinationError::RecoveryRequired(format!(
                    "outcome {outcome_id} requires a current successful verification receipt before publication"
                )));
            }
            Ok(())
        },
    )
}

pub fn recover_impact(
    store: &CoordinationStore,
    event_id: &str,
    consumer_work_item_id: &str,
    consumer_generation: u64,
) -> Result<CoordinationRecovery, CoordinationError> {
    store.recover_event(event_id, consumer_work_item_id, consumer_generation)
}

pub fn request_safe_pause(
    store: &CoordinationStore,
    request: CoordinationRequest,
) -> Result<CoordinationRequest, CoordinationError> {
    if request.intent != CoordinationIntent::RequestSafePause {
        return Err(CoordinationError::RecoveryRequired(
            "request_safe_pause requires request_safe_pause intent".into(),
        ));
    }
    store.request_coordination(request)
}

pub fn acknowledge_pause(
    store: &CoordinationStore,
    request_id: &str,
    state: CoordinationRequestState,
) -> Result<CoordinationRequest, CoordinationError> {
    if !matches!(
        state,
        CoordinationRequestState::Acknowledged
            | CoordinationRequestState::SafelyPaused
            | CoordinationRequestState::Unavailable
            | CoordinationRequestState::Expired
    ) {
        return Err(CoordinationError::RecoveryRequired(
            "pause acknowledgement must be an explicit safe-boundary state".into(),
        ));
    }
    store.transition_request(request_id, state)
}

pub fn resume_and_re_evaluate(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
) -> Result<CollaborationAdmission, CoordinationError> {
    let projection = refresh_dependency_state(store, work_item_id, generation)?;
    let request_ids = projection
        .requests
        .iter()
        .filter(|request| {
            request.target_work_item_id == work_item_id
                && request.target_generation == generation
                && request.state == CoordinationRequestState::SafelyPaused
        })
        .map(|request| request.request_id.clone())
        .collect::<Vec<_>>();
    for request_id in request_ids {
        store.transition_request(&request_id, CoordinationRequestState::Resumed)?;
    }
    refresh_and_admit_collaboration_action(
        store,
        work_item_id,
        generation,
        CollaborationAction {
            kind: CollaborationActionKind::Verification,
            consumer_work_item_id: work_item_id.into(),
            outcomes: Vec::new(),
        },
    )
}

fn event_invalidates(event: &CoordinationEvent) -> bool {
    !matches!(
        event.kind,
        cockpit_protocol::CoordinationEventKind::OutcomePublished
    )
}

fn find_cycles(registrations: &[WorktreeRegistration]) -> Vec<Vec<String>> {
    let graph = registrations
        .iter()
        .map(|registration| {
            (
                registration.work_item_id.clone(),
                registration
                    .declaration
                    .consumed_outcomes
                    .iter()
                    .map(|dependency| dependency.provider_work_item_id.clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut cycles = Vec::new();
    for node in graph.keys() {
        collect_cycles(node, &graph, &mut Vec::new(), &mut cycles);
    }
    cycles
}

fn collect_cycles(
    node: &str,
    graph: &BTreeMap<String, Vec<String>>,
    stack: &mut Vec<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    if let Some(position) = stack.iter().position(|item| item == node) {
        let mut cycle = stack[position..].to_vec();
        cycle.push(node.into());
        let key = cycle.join("->");
        if !cycles.iter().any(|candidate| candidate.join("->") == key) {
            cycles.push(cycle);
        }
        return;
    }
    stack.push(node.into());
    if let Some(dependencies) = graph.get(node) {
        for dependency in dependencies {
            if graph.contains_key(dependency) {
                collect_cycles(dependency, graph, stack, cycles);
            }
        }
    }
    stack.pop();
}

#[cfg(all(test, unix))]
mod registered_worktree_file_tests {
    use super::*;

    #[test]
    fn opened_active_directory_swap_cannot_redirect_contract_read() {
        let root = tempfile::tempdir().expect("repository root");
        let outside = tempfile::tempdir().expect("outside directory");
        let active_path = root.path().join(".ai/work-items/active");
        fs::create_dir_all(&active_path).expect("active directory");
        let leaf = "WI-test.contract.json";
        fs::write(active_path.join(leaf), b"outside contract bytes").expect("original contract");
        let moved_active = outside.path().join("active");
        let reference = ".ai/work-items/active/WI-test.contract.json";
        let result =
            open_registered_worktree_file_with_opener(root.path(), reference, |parent, leaf| {
                fs::rename(&active_path, &moved_active)
                    .expect("move opened directory outside root");
                fs::create_dir_all(&active_path).expect("install an in-repository replacement");
                fs::write(active_path.join(leaf), b"replacement contract bytes")
                    .expect("replacement contract");
                let mut options = CapOpenOptions::new();
                options.read(true).follow(FollowSymlinks::No);
                parent
                    .open_with(leaf, &options)
                    .map(cap_std::fs::File::into_std)
            });
        assert!(
            result.is_err(),
            "persistent directory swap must not admit the moved directory handle"
        );
    }

    #[test]
    fn opened_active_directory_swap_back_race_is_rejected() {
        let root = tempfile::tempdir().expect("repository root");
        let outside = tempfile::tempdir().expect("outside directory");
        let active_path = root.path().join(".ai/work-items/active");
        fs::create_dir_all(&active_path).expect("active directory");
        let leaf = "WI-test.contract.json";
        fs::write(active_path.join(leaf), b"outside contract bytes").expect("contract");
        let moved_active = outside.path().join("active");
        let reference = ".ai/work-items/active/WI-test.contract.json";
        let mut opened_while_outside = Vec::new();

        let result =
            open_registered_worktree_file_with_opener(root.path(), reference, |parent, leaf| {
                fs::rename(&active_path, &moved_active)
                    .expect("move opened directory outside root");
                let mut options = CapOpenOptions::new();
                options.read(true).follow(FollowSymlinks::No);
                let mut file = parent
                    .open_with(leaf, &options)
                    .expect("open the leaf through the held outside directory handle")
                    .into_std();
                file.read_to_end(&mut opened_while_outside)
                    .expect("read the leaf while the directory is outside");
                fs::rename(&moved_active, &active_path)
                    .expect("restore the same directory before checks");
                Ok(file)
            });

        assert_eq!(opened_while_outside, b"outside contract bytes");
        assert!(
            result.is_err(),
            "move-out/open/move-back must not admit bytes read while outside the repository"
        );
    }

    #[test]
    fn opened_active_directory_swap_back_is_rejected_without_timestamp_resolution() {
        struct CoarseTimestampOverride;
        impl CoarseTimestampOverride {
            fn set() -> Self {
                DIRECTORY_CHANGE_TOKEN_OVERRIDE.with(|token| {
                    token.set(Some(DirectoryChangeToken([0; 4])));
                });
                Self
            }
        }
        impl Drop for CoarseTimestampOverride {
            fn drop(&mut self) {
                DIRECTORY_CHANGE_TOKEN_OVERRIDE.with(|token| token.set(None));
            }
        }

        let _coarse_timestamps = CoarseTimestampOverride::set();
        let root = tempfile::tempdir().expect("repository root");
        let outside = tempfile::tempdir().expect("outside directory");
        let active_path = root.path().join(".ai/work-items/active");
        fs::create_dir_all(&active_path).expect("active directory");
        let leaf = "WI-test.contract.json";
        fs::write(active_path.join(leaf), b"outside contract bytes").expect("contract");
        let moved_active = outside.path().join("active");
        let reference = ".ai/work-items/active/WI-test.contract.json";

        let result =
            open_registered_worktree_file_with_opener(root.path(), reference, |parent, leaf| {
                fs::rename(&active_path, &moved_active)
                    .expect("move opened directory outside root");
                let mut options = CapOpenOptions::new();
                options.read(true).follow(FollowSymlinks::No);
                let mut file = parent
                    .open_with(leaf, &options)
                    .expect("open leaf through held outside directory handle")
                    .into_std();
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)
                    .expect("read leaf while directory is outside");
                fs::rename(&moved_active, &active_path)
                    .expect("restore same directory before post-checks");
                Ok(file)
            });

        assert!(
            result.is_err(),
            "rename detection must not depend on timestamp granularity"
        );
    }

    #[test]
    fn outcome_verification_uses_event_bound_bytes_and_digest_bound_contract() {
        let root = tempfile::tempdir().expect("repository root");
        let run_git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root.path())
                    .status()
                    .expect("git command")
                    .success()
            );
        };
        run_git(&["init", "-q"]);
        run_git(&["config", "user.email", "test@example.invalid"]);
        run_git(&["config", "user.name", "Verification evidence test"]);
        fs::write(root.path().join("README.md"), "initial\n").expect("README");
        run_git(&["add", "."]);
        run_git(&["commit", "-qm", "initial"]);
        run_git(&["branch", "-M", "main"]);
        crate::attach(root.path()).expect("attach repository");
        let work_item_id = "WI-VERIFICATION-CONTRACT-SWAP";
        crate::start_work_item_with_options(
            root.path(),
            work_item_id,
            "verification evidence reader test",
            "require the registered Contract reader during evidence validation",
            &[".ai/**".into(), "README.md".into()],
            &crate::WorkItemStartOptions {
                authority: "authorized".into(),
                out_of_scope: vec!["target/**".into()],
                acceptance_criteria: vec!["Contract identity remains digest bound".into()],
                ..crate::WorkItemStartOptions::default()
            },
        )
        .expect("start provider Work Item");
        let contract_path = root
            .path()
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"));
        crate::preflight_work_item(root.path(), &contract_path).expect("preflight");
        crate::checkpoint_work_item(root.path(), work_item_id).expect("checkpoint");
        let receipt = cockpit_verification::execute_bounded(
            vec![cockpit_verification::VerificationCommand::new(
                "verification-contract-reader",
                "sh",
                vec!["-c".into(), "true".into()],
                cockpit_verification::VerificationReusePolicy::NeverReuse,
            )],
            1,
        )
        .expect("execute verification receipt");
        let runtime_digest = cockpit_core::Digest::sha256_bytes(b"test-runtime");
        crate::record_verification(
            root.path(),
            work_item_id,
            &serde_json::to_value(receipt).expect("serialize verification receipt"),
            "0.2.113",
            &runtime_digest,
        )
        .expect("record typed verification evidence");

        let evidence_reference = format!(".ai/evidence/{work_item_id}.verification.json");
        let evidence_bytes =
            fs::read(root.path().join(&evidence_reference)).expect("verification evidence bytes");
        let git = GitRepository::discover(root.path()).expect("discover repository");
        let topology = git.topology().expect("repository topology");
        let contract_value: serde_json::Value =
            serde_json::from_slice(&fs::read(&contract_path).expect("Contract bytes"))
                .expect("Contract JSON");
        let contract_digest =
            cockpit_protocol::digest_json(&contract_value).expect("registered Contract digest");
        let runtime = RuntimeCapabilityBinding {
            schema_version: 1,
            runtime_version: "0.2.113".into(),
            runtime_digest,
            capability: cockpit_protocol::COLLABORATION_CAPABILITY.into(),
        };
        let outcome = cockpit_protocol::ProvidedOutcome {
            outcome_id: "api".into(),
            interface_contract: "api-v1".into(),
            behavior_contract: "stable behavior".into(),
            published_head: topology.head.clone().expect("head"),
            stage: cockpit_protocol::OutcomeStage::ComposableHead,
            evidence_refs: vec![evidence_reference.clone()],
        };
        let provider = WorktreeRegistration {
            schema_version: cockpit_protocol::COLLABORATION_SCHEMA_VERSION,
            repository_id: crate::repository_id(root.path()),
            work_item_id: work_item_id.into(),
            contract_digest,
            worktree_path: topology.repository_root.to_string_lossy().into_owned(),
            branch: topology.branch.expect("branch"),
            head: outcome.published_head.clone(),
            generation: 1,
            declaration: cockpit_protocol::CollaborationDeclaration {
                provided_outcomes: vec![outcome.clone()],
                ..cockpit_protocol::CollaborationDeclaration::default()
            },
            runtime,
            environment: None,
        };
        let publication = CoordinationEvent {
            schema_version: cockpit_protocol::COLLABORATION_SCHEMA_VERSION,
            event_id: "verified-api-publication".into(),
            repository_id: provider.repository_id.clone(),
            work_item_id: work_item_id.into(),
            generation: 1,
            kind: cockpit_protocol::CoordinationEventKind::OutcomePublished,
            source: "test-verification-publication".into(),
            evidence_refs: vec![evidence_reference.clone()],
            evidence_digests: BTreeMap::from([(
                evidence_reference,
                cockpit_core::Digest::sha256_bytes(&evidence_bytes),
            )]),
            outcome_ids: vec!["api".into()],
            environment_change: None,
        };
        let projection = CollaborationProjection {
            registrations: vec![provider.clone()],
            events: vec![publication],
            ..CollaborationProjection::default()
        };

        assert!(verification_evidence_is_complete(
            &projection,
            &provider,
            &outcome
        ));

        let bound_reference = outcome.evidence_refs[0].clone();
        let mut invalid_bound_value: serde_json::Value =
            serde_json::from_slice(&evidence_bytes).expect("verification evidence JSON");
        invalid_bound_value["receiptDigest"] =
            serde_json::json!(format!("sha256:{}", "0".repeat(64)));
        let invalid_bound_bytes =
            serde_json::to_vec(&invalid_bound_value).expect("invalid bound evidence JSON");
        fs::write(root.path().join(&bound_reference), &invalid_bound_bytes)
            .expect("write invalid event-bound evidence");
        let mut invalid_publication = projection.events[0].clone();
        invalid_publication.event_id = "invalid-bound-verification-publication".into();
        invalid_publication.evidence_digests = BTreeMap::from([(
            bound_reference.clone(),
            cockpit_core::Digest::sha256_bytes(&invalid_bound_bytes),
        )]);
        let invalid_projection = CollaborationProjection {
            registrations: vec![provider.clone()],
            events: vec![invalid_publication.clone()],
            ..CollaborationProjection::default()
        };
        let accepted_different_path_bytes = verification_evidence_is_complete_with_reader(
            &invalid_projection,
            &provider,
            &outcome,
            |repository_root, reference| {
                if reference == bound_reference {
                    let bound_bytes = read_registered_worktree_file(repository_root, reference)?;
                    fs::write(repository_root.join(reference), &evidence_bytes)
                        .map_err(|error| error.to_string())?;
                    Ok(bound_bytes)
                } else {
                    read_registered_worktree_file(repository_root, reference)
                }
            },
        );
        assert!(
            !accepted_different_path_bytes,
            "semantic verification must use the exact digest-bound evidence bytes, not reopen a swapped path"
        );
        let store = CoordinationStore::open(&git, provider.runtime.clone())
            .expect("open typed publication store");
        store
            .register(provider.clone())
            .expect("register typed publication provider");
        fs::write(root.path().join(&bound_reference), &invalid_bound_bytes)
            .expect("restore invalid event-bound evidence before publication");
        let publication_result = store.publish_validated_outcome_event(
            invalid_publication,
            |event, _inspection, current_provider, bound_evidence| {
                fs::write(root.path().join(&bound_reference), &evidence_bytes)
                    .expect("replace path after the store reads digest-bound bytes");
                if verification_evidence_matches_publication_with_bytes(
                    event,
                    current_provider,
                    &outcome,
                    bound_evidence,
                    &mut read_registered_worktree_file,
                ) {
                    Ok(())
                } else {
                    Err(CoordinationError::RecoveryRequired(
                        "bound receipt is not a current successful verification".into(),
                    ))
                }
            },
        );
        assert!(
            publication_result.is_err(),
            "typed publication must reject when only the post-read replacement is semantically valid"
        );
        assert!(
            store
                .inspect()
                .expect("inspect rejected typed publication")
                .events
                .is_empty(),
            "invalid bound evidence must not leave a durable publication"
        );
        fs::write(root.path().join(&bound_reference), &evidence_bytes)
            .expect("restore valid evidence for the Contract race regression");

        let outside = tempfile::tempdir().expect("outside directory");
        let active_path = root.path().join(".ai/work-items/active");
        let moved_active = outside.path().join("active");
        let mut opened_contract_bytes = Vec::new();
        let accepted = verification_evidence_is_complete_with_reader(
            &projection,
            &provider,
            &outcome,
            |repository_root, reference| {
                if reference.ends_with(".contract.json") {
                    read_registered_worktree_file_with_opener(
                        repository_root,
                        reference,
                        |parent, leaf| {
                            fs::rename(&active_path, &moved_active)
                                .expect("move Contract directory outside repository");
                            let mut options = CapOpenOptions::new();
                            options.read(true).follow(FollowSymlinks::No);
                            let mut file = parent
                                .open_with(leaf, &options)
                                .expect("open Contract through moved directory handle")
                                .into_std();
                            file.read_to_end(&mut opened_contract_bytes)
                                .expect("read Contract while outside");
                            fs::rename(&moved_active, &active_path)
                                .expect("restore Contract directory before checks");
                            Ok(file)
                        },
                    )
                } else {
                    read_registered_worktree_file(repository_root, reference)
                }
            },
        );

        assert_eq!(
            opened_contract_bytes,
            fs::read(&contract_path).expect("Contract bytes")
        );
        assert!(
            !accepted,
            "verification evidence must not admit bytes when the registered Contract reader rejects the move-out/open/move-back race"
        );
    }
}

#[cfg(test)]
mod outcome_publication_tests {
    use super::*;
    use cockpit_protocol::{
        CollaborationDeclaration, ConsumedOutcome, IntegrationResponsibility, OutcomeStage,
        ProvidedOutcome, RuntimeCapabilityBinding,
    };

    fn git(root: &Path, args: &[&str]) {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .expect("git command")
                .success()
        );
    }

    fn repository() -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("repository root");
        git(root.path(), &["init", "-q"]);
        git(root.path(), &["config", "core.autocrlf", "false"]);
        git(
            root.path(),
            &["config", "user.email", "test@example.invalid"],
        );
        git(
            root.path(),
            &["config", "user.name", "Outcome publication test"],
        );
        fs::write(root.path().join("README.md"), "initial\n").expect("README");
        fs::create_dir_all(root.path().join("target")).expect("target directory");
        fs::write(root.path().join("target/outcome.json"), "{}\n").expect("outcome evidence");
        git(root.path(), &["add", "."]);
        git(root.path(), &["commit", "-qm", "initial"]);
        git(root.path(), &["branch", "-M", "main"]);
        crate::attach(root.path()).expect("attach repository");
        root
    }

    fn registration(
        root: &Path,
        work_item_id: &str,
        provider: bool,
        consumer: bool,
    ) -> WorktreeRegistration {
        crate::start_work_item_with_options(
            root,
            work_item_id,
            "outcome publication concurrency test",
            "serialize verification-required registration with outcome publication",
            &[".ai/**".into(), "README.md".into(), "target/**".into()],
            &crate::WorkItemStartOptions {
                authority: "authorized".into(),
                acceptance_criteria: vec![
                    "publication is bound to current coordination facts".into(),
                ],
                ..crate::WorkItemStartOptions::default()
            },
        )
        .expect("start test Work Item");
        let contract_path = root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"));
        let contract_value: serde_json::Value =
            serde_json::from_slice(&fs::read(&contract_path).expect("Contract bytes"))
                .expect("Contract JSON");
        let contract_digest =
            cockpit_protocol::digest_json(&contract_value).expect("Contract digest");
        let topology = GitRepository::discover(root)
            .expect("discover repository")
            .topology()
            .expect("repository topology");
        let head = topology.head.expect("head");
        let provided_outcomes = if provider {
            vec![ProvidedOutcome {
                outcome_id: "api".into(),
                interface_contract: "api-v1".into(),
                behavior_contract: "stable behavior".into(),
                published_head: head.clone(),
                stage: OutcomeStage::ComposableHead,
                evidence_refs: vec!["target/outcome.json".into()],
            }]
        } else {
            Vec::new()
        };
        let consumed_outcomes = if consumer {
            vec![ConsumedOutcome {
                provider_work_item_id: "WI-PROVIDER".into(),
                outcome_id: "api".into(),
                minimum_stage: OutcomeStage::ComposableHead,
                verification_required: true,
            }]
        } else {
            Vec::new()
        };
        let declaration = CollaborationDeclaration {
            provided_outcomes,
            consumed_outcomes,
            resource_claims: Vec::new(),
            integration_responsibility: IntegrationResponsibility {
                responsible_work_item_id: work_item_id.into(),
                target_branch: "main".into(),
                composition_order: Vec::new(),
                rationale: "deterministic publication race test".into(),
            },
            composition_verification: Default::default(),
        };
        WorktreeRegistration {
            schema_version: cockpit_protocol::COLLABORATION_SCHEMA_VERSION,
            repository_id: crate::repository_id(root),
            work_item_id: work_item_id.into(),
            contract_digest,
            worktree_path: topology.repository_root.to_string_lossy().into_owned(),
            branch: topology.branch.expect("branch"),
            head,
            generation: 1,
            declaration,
            runtime: RuntimeCapabilityBinding {
                schema_version: 1,
                runtime_version: "0.2.113".into(),
                runtime_digest: cockpit_core::Digest::sha256_bytes(b"test-runtime"),
                capability: cockpit_protocol::COLLABORATION_CAPABILITY.into(),
            },
            environment: None,
        }
    }

    #[test]
    fn publish_outcome_observes_consumer_registered_before_locked_append() {
        let root = repository();
        let git_repository = GitRepository::discover(root.path()).expect("discover repository");
        let store = CoordinationStore::open(
            &git_repository,
            RuntimeCapabilityBinding {
                schema_version: 1,
                runtime_version: "0.2.113".into(),
                runtime_digest: cockpit_core::Digest::sha256_bytes(b"test-runtime"),
                capability: cockpit_protocol::COLLABORATION_CAPABILITY.into(),
            },
        )
        .expect("open coordination store");
        store
            .register(registration(root.path(), "WI-PROVIDER", true, false))
            .expect("register provider");
        let consumer = registration(root.path(), "WI-CONSUMER", false, true);

        let result = publish_outcome_with_pre_append(&store, "WI-PROVIDER", 1, "api", || {
            store
                .register(consumer)
                .expect("register verification-required consumer at the prepared-event boundary");
        });

        assert!(
            matches!(&result, Err(CoordinationError::RecoveryRequired(message)) if message.contains("verification receipt")),
            "typed publication must include a consumer committed immediately before the append transaction: {result:?}"
        );
        assert!(
            store
                .inspect()
                .expect("inspect coordination store")
                .events
                .is_empty(),
            "a publication rejected by a newly registered verification requirement must not append an event"
        );

        // The opposite lock order is also coherent: if publication commits
        // first, a later verification-required registration observes the
        // event but cannot treat its non-verification evidence as sufficient.
        let reverse_root = repository();
        let reverse_repository =
            GitRepository::discover(reverse_root.path()).expect("discover reverse-order repo");
        let reverse_store = CoordinationStore::open(
            &reverse_repository,
            RuntimeCapabilityBinding {
                schema_version: 1,
                runtime_version: "0.2.113".into(),
                runtime_digest: cockpit_core::Digest::sha256_bytes(b"test-runtime"),
                capability: cockpit_protocol::COLLABORATION_CAPABILITY.into(),
            },
        )
        .expect("open reverse-order coordination store");
        reverse_store
            .register(registration(
                reverse_root.path(),
                "WI-PROVIDER",
                true,
                false,
            ))
            .expect("register reverse-order provider");
        publish_outcome(&reverse_store, "WI-PROVIDER", 1, "api")
            .expect("publish before the consumer requires verification");
        reverse_store
            .register(registration(
                reverse_root.path(),
                "WI-CONSUMER",
                false,
                true,
            ))
            .expect("register reverse-order consumer");
        let reverse_projection =
            collaboration_projection(&reverse_store).expect("inspect reverse lock order");
        assert!(
            reverse_projection
                .blockers
                .get("WI-CONSUMER")
                .is_some_and(|blockers| blockers
                    .iter()
                    .any(|blocker| { blocker == "dependency_evidence_missing:WI-PROVIDER:api" })),
            "a consumer registered after publication must still require current verification evidence: {:?}",
            reverse_projection.blockers
        );
    }

    #[test]
    fn pause_committed_after_initial_admission_blocks_the_process_start_boundary() {
        let root = repository();
        let git_repository = GitRepository::discover(root.path()).expect("discover repository");
        let store = CoordinationStore::open(
            &git_repository,
            RuntimeCapabilityBinding {
                schema_version: 1,
                runtime_version: "0.2.113".into(),
                runtime_digest: cockpit_core::Digest::sha256_bytes(b"test-runtime"),
                capability: cockpit_protocol::COLLABORATION_CAPABILITY.into(),
            },
        )
        .expect("open coordination store");
        store
            .register(registration(root.path(), "WI-SOLO", false, false))
            .expect("register composition owner");
        let action = CollaborationAction {
            kind: CollaborationActionKind::Composition,
            consumer_work_item_id: "WI-SOLO".into(),
            outcomes: Vec::new(),
        };
        assert!(
            admit_collaboration_action(&store, "WI-SOLO", 1, action.clone())
                .expect("initial admission")
                .allowed
        );

        let request = CoordinationRequest {
            schema_version: 1,
            request_id: "pause-after-admission".into(),
            repository_id: crate::repository_id(root.path()),
            target_work_item_id: "WI-SOLO".into(),
            target_generation: 1,
            intent: CoordinationIntent::RequestSafePause,
            state: CoordinationRequestState::Requested,
            reason: "pause arrived while composition was preparing".into(),
        };
        request_safe_pause(&store, request).expect("request safe pause");
        acknowledge_pause(
            &store,
            "pause-after-admission",
            CoordinationRequestState::Acknowledged,
        )
        .expect("acknowledge pause");
        acknowledge_pause(
            &store,
            "pause-after-admission",
            CoordinationRequestState::SafelyPaused,
        )
        .expect("confirm safe pause");

        let mut spawn_called = false;
        let result = with_composition_process_start_admission(
            &store,
            "WI-SOLO",
            1,
            action,
            || Ok(()),
            || {
                spawn_called = true;
                Err::<(), _>("test spawn should not be reached".into())
            },
        );
        assert!(
            result
                .expect_err("the fresh process-start admission must observe the pause")
                .contains("coordination_safely_paused")
        );
        assert!(!spawn_called, "the child spawn closure must not run");
    }
}

#[cfg(all(test, windows))]
mod registered_worktree_file_windows_tests {
    use super::*;

    struct CoarseTimestampOverride;

    impl CoarseTimestampOverride {
        fn set() -> Self {
            DIRECTORY_CHANGE_TOKEN_OVERRIDE.with(|token| {
                token.set(Some(DirectoryChangeToken([0; 4])));
            });
            Self
        }
    }

    impl Drop for CoarseTimestampOverride {
        fn drop(&mut self) {
            DIRECTORY_CHANGE_TOKEN_OVERRIDE.with(|token| token.set(None));
        }
    }

    #[test]
    fn opened_directory_handles_prevent_swap_back_even_without_timestamp_resolution() {
        let _coarse_timestamps = CoarseTimestampOverride::set();
        let root = tempfile::tempdir().expect("repository root");
        let outside = tempfile::tempdir().expect("outside directory");
        let active_path = root.path().join(".ai/work-items/active");
        fs::create_dir_all(&active_path).expect("active directory");
        let leaf = "WI-test.contract.json";
        fs::write(active_path.join(leaf), b"registered contract bytes").expect("contract");
        let moved_active = outside.path().join("active");
        let reference = ".ai/work-items/active/WI-test.contract.json";
        let mut rename_was_blocked = false;

        let result =
            open_registered_worktree_file_with_opener(root.path(), reference, |parent, leaf| {
                match fs::rename(&active_path, &moved_active) {
                    Err(error) => {
                        rename_was_blocked = true;
                        Err(error)
                    }
                    Ok(()) => {
                        let mut options = CapOpenOptions::new();
                        options.read(true).follow(FollowSymlinks::No);
                        let file = parent
                            .open_with(leaf, &options)
                            .expect("open Contract through the held directory handle")
                            .into_std();
                        fs::rename(&moved_active, &active_path)
                            .expect("restore moved directory before post-checks");
                        Ok(file)
                    }
                }
            });

        assert!(
            rename_was_blocked,
            "held directory handles must deny rename"
        );
        assert!(
            result.is_err(),
            "a blocked move must not admit the open attempt"
        );
        assert!(active_path.join(leaf).is_file());
        assert!(!moved_active.exists());
    }
}

#[cfg(test)]
mod composition_channel_deadline_tests {
    use super::*;
    use serde::Deserialize;
    use std::time::Instant;

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct TestFrame {
        value: String,
    }

    // Producers are ordinary bounded threads: no child processes, detached
    // workers or rescue cleanup. Every producer is joined before an assertion.
    #[test]
    fn ready_diagnostic_lines_cannot_restart_the_absolute_deadline() {
        let (sender, receiver) = mpsc::channel();
        let producer = std::thread::spawn(move || {
            for _ in 0..5 {
                if sender.send(Ok("diagnostic\n".into())).is_err() {
                    return;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            let _ = sender.send(Ok("{\"value\":\"late\"}\n".into()));
        });
        let started = Instant::now();
        let result =
            read_supervisor_message::<TestFrame>(&receiver, Some(Duration::from_millis(35)));
        let elapsed = started.elapsed();
        producer.join().expect("bounded diagnostic producer");
        assert!(
            result.is_err(),
            "late Ready received a renewed budget: {result:?}"
        );
        assert!(elapsed < Duration::from_millis(90), "read took {elapsed:?}");
    }

    #[test]
    fn oversized_ready_is_rejected_instead_of_deserialized() {
        let (sender, receiver) = mpsc::channel();
        sender
            .send(Ok(format!("{{\"value\":\"{}\"}}\n", "x".repeat(65_536))))
            .expect("queue oversized frame");
        drop(sender);
        let result = read_supervisor_message::<TestFrame>(&receiver, Some(Duration::from_secs(1)));
        assert!(result.is_err(), "oversized Ready was accepted");
        assert!(result.unwrap_err().contains("byte limit"));
    }

    #[test]
    fn malformed_json_protocol_is_not_skipped_as_diagnostic_text() {
        let (sender, receiver) = mpsc::channel();
        sender
            .send(Ok("{\"value\":null}\n".into()))
            .expect("malformed protocol");
        sender
            .send(Ok("{\"value\":\"after-invalid\"}\n".into()))
            .expect("later valid frame");
        drop(sender);
        let result = read_supervisor_message::<TestFrame>(&receiver, Some(Duration::from_secs(1)));
        assert!(
            result.is_err(),
            "malformed expected JSON was skipped: {result:?}"
        );
    }

    #[test]
    fn terminal_reply_without_natural_eof_is_not_success() {
        let (sender, receiver) = mpsc::channel();
        sender
            .send(Ok("{\"value\":\"terminal\"}\n".into()))
            .expect("terminal frame");
        let producer = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            drop(sender);
        });
        let started = Instant::now();
        let result = read_composition_terminal_until::<TestFrame>(
            &receiver,
            started + Duration::from_millis(35),
        );
        let elapsed = started.elapsed();
        producer.join().expect("finite held channel");
        assert!(
            result.is_err(),
            "terminal without natural EOF was accepted: {result:?}"
        );
        assert!(
            elapsed < Duration::from_millis(100),
            "terminal took {elapsed:?}"
        );
    }

    #[test]
    fn duplicate_terminal_reply_is_not_ignored() {
        let (sender, receiver) = mpsc::channel();
        for _ in 0..2 {
            sender
                .send(Ok("{\"value\":\"terminal\"}\n".into()))
                .expect("duplicate terminal");
        }
        drop(sender);
        let result = read_composition_terminal_until::<TestFrame>(
            &receiver,
            Instant::now() + Duration::from_secs(1),
        );
        assert!(
            result.is_err(),
            "a second terminal frame was ignored: {result:?}"
        );
    }

    #[test]
    fn one_terminal_and_natural_eof_preserve_the_original_value() {
        let (sender, receiver) = mpsc::channel();
        sender
            .send(Ok("{\"value\":\"original\"}\n".into()))
            .expect("terminal frame");
        drop(sender);
        let result = read_composition_terminal_until::<TestFrame>(
            &receiver,
            Instant::now() + Duration::from_secs(1),
        )
        .expect("complete channel");
        assert_eq!(result.value, "original");
    }
}

#[cfg(all(test, unix))]
mod composition_unix_channel_tests {
    use super::*;
    use std::os::unix::net::UnixStream;

    fn pipe() -> (UnixStream, CompositionOutputPipe<UnixStream>) {
        let (writer, reader) = UnixStream::pair().expect("private owned socket pair");
        let reader = cockpit_verification::supervisor_pipe::FrameReader::new(
            reader,
            MAX_COMPOSITION_READY_BYTES * 2,
            MAX_COMPOSITION_READY_BYTES,
        )
        .expect("nonblocking frame reader");
        (
            writer,
            CompositionOutputPipe(std::cell::RefCell::new(reader)),
        )
    }

    #[test]
    fn unterminated_oversized_frame_fails_without_waiting_for_eof() {
        let (mut writer, reader) = pipe();
        let start = Instant::now();
        let deadline = start + Duration::from_millis(100);
        let producer = std::thread::spawn(move || {
            let produced = cockpit_verification::supervisor_pipe::write_until(
                &mut writer,
                &vec![b'x'; MAX_COMPOSITION_READY_BYTES + 1],
                deadline,
            );
            // JoinHandle's result owns the endpoint after this thread exits.
            // EOF therefore cannot explain a rejection before the reader decides.
            (writer, produced)
        });
        let result = read_composition_message_until::<serde_json::Value>(
            &reader,
            deadline,
            MAX_COMPOSITION_READY_BYTES,
        );
        // Join only the producer whose every I/O operation uses the same
        // absolute deadline. Never block-write before starting the reader or
        // assume a particular Linux/macOS socket buffer capacity.
        let (_held_writer, produced) = producer.join().expect("bounded oversize producer");
        assert!(
            produced.is_ok(),
            "fault bytes did not complete: {produced:?}"
        );
        assert!(
            result
                .as_ref()
                .is_err_and(|error| error.contains("byte limit")),
            "actual frame result: {result:?}"
        );
        assert!(start.elapsed() < Duration::from_millis(400));
    }

    #[test]
    fn real_pipe_terminal_requires_natural_eof() {
        let (mut writer, reader) = pipe();
        writer
            .write_all(b"{\"original\":true}\n")
            .expect("complete terminal");
        let result = read_terminal_composition_message::<serde_json::Value>(
            &reader,
            Instant::now() + Duration::from_millis(35),
            MAX_COMPOSITION_READY_BYTES,
        );
        assert!(
            result.is_err(),
            "a held writer cannot certify terminal completion"
        );
    }

    #[test]
    fn real_pipe_terminal_then_eof_keeps_exact_json() {
        let (mut writer, reader) = pipe();
        writer
            .write_all(b"{\"original\":true}\n")
            .expect("complete terminal");
        drop(writer);
        let result = read_terminal_composition_message::<serde_json::Value>(
            &reader,
            Instant::now() + Duration::from_secs(1),
            MAX_COMPOSITION_READY_BYTES,
        )
        .expect("natural terminal EOF");
        assert_eq!(result, serde_json::json!({"original":true}));
    }

    #[test]
    fn partial_eof_and_invalid_utf8_are_not_diagnostics_or_empty_success() {
        for bytes in [&b"{\"partial\":"[..], &b"\xff\n"[..]] {
            let (mut writer, reader) = pipe();
            writer.write_all(bytes).expect("bounded fault bytes");
            drop(writer);
            let result = read_composition_message_until::<serde_json::Value>(
                &reader,
                Instant::now() + Duration::from_secs(1),
                MAX_COMPOSITION_READY_BYTES,
            );
            assert!(result.is_err(), "invalid frame was promoted");
        }
    }

    #[test]
    fn unread_request_expires_in_the_shared_writer_without_a_reader_thread() {
        let (mut writer, _nonreading_peer) = UnixStream::pair().expect("private pair");
        let started = Instant::now();
        let result = cockpit_verification::supervisor_pipe::write_until(
            &mut writer,
            &vec![b'x'; 1024 * 1024],
            started + Duration::from_millis(35),
        );
        assert!(result.is_err_and(|error| error.kind() == io::ErrorKind::TimedOut));
        assert!(started.elapsed() < Duration::from_millis(250));
    }

    // A separate real child with no descendants and an unconditional hard TTL.
    // The test's finite fallback owns this exact Child; it never discovers or
    // kills unrelated PIDs, and its cleanup is not Runtime full-tree proof.
    const FIXTURE_MODE: &str = "AI_COCKPIT_COMPOSITION_CHANNEL_OWNED_FIXTURE";
    #[test]
    fn owned_child_fixture() {
        let Some(mode) = std::env::var_os(FIXTURE_MODE) else {
            return;
        };
        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(2));
            // SAFETY: this isolated fixture only, with no cleanup hooks to wait.
            unsafe { libc::_exit(97) };
        });
        let nonce = std::env::var("AI_COCKPIT_COMPOSITION_CHANNEL_WITNESS").expect("fixture nonce");
        let mut stdout = io::stdout().lock();
        stdout.write_all(b"\n").expect("separate libtest prefix");
        serde_json::to_writer(
            &mut stdout,
            &serde_json::json!({"pid":std::process::id(),"nonce":nonce}),
        )
        .expect("witness");
        stdout
            .write_all(b"\n")
            .and_then(|()| stdout.flush())
            .expect("bounded witness line");
        drop(stdout);
        if mode == "delay" {
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    pub(super) struct FixtureChild {
        pub(super) child: Child,
        pub(super) reaped: bool,
        _witness_output: Option<CompositionOutputPipe>,
    }
    impl FixtureChild {
        pub(super) fn new(mode: &str) -> Self {
            #[derive(serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Witness {
                pid: u32,
                nonce: String,
            }
            let nonce = new_composition_run_nonce();
            let mut child =
                Command::new(std::env::current_exe().expect("absolute test executable"))
                    .args([
                        "--exact",
                        "collaboration::composition_unix_channel_tests::owned_child_fixture",
                        "--nocapture",
                    ])
                    .env(FIXTURE_MODE, mode)
                    .env("AI_COCKPIT_COMPOSITION_CHANNEL_WITNESS", &nonce)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("owned TTL fixture");
            let output = match child
                .stdout
                .take()
                .ok_or("missing fixture witness output")
                .and_then(|stdout| {
                    supervisor_output_channel(stdout, MAX_COMPOSITION_READY_BYTES)
                        .map_err(|_| "fixture output setup")
                }) {
                Ok(output) => output,
                Err(error) => {
                    let cleanup = cleanup_owned_composition_child(
                        &mut child,
                        Instant::now() + Duration::from_secs(2),
                    );
                    panic!("{error}; {cleanup:?}");
                }
            };
            let mut fixture = Self {
                child,
                reaped: false,
                _witness_output: Some(output),
            };
            let witness = read_composition_message_until::<Witness>(
                fixture._witness_output.as_ref().expect("output"),
                Instant::now() + Duration::from_secs(1),
                MAX_COMPOSITION_READY_BYTES,
            );
            if !witness
                .is_ok_and(|witness| witness.pid == fixture.child.id() && witness.nonce == nonce)
            {
                let cleanup = fixture.cleanup();
                panic!("exact fixture pid/nonce witness missing; {cleanup:?}");
            }
            fixture
        }
        fn cleanup(&mut self) -> Result<String, String> {
            let result = terminate_composition_child_until(
                &mut self.child,
                Instant::now() + Duration::from_secs(1),
            );
            if result.is_ok() {
                self.reaped = true;
            }
            result
        }
    }
    impl Drop for FixtureChild {
        fn drop(&mut self) {
            if self.reaped {
                return;
            }
            // Failure-only test rescue: still finite and bound to the owned
            // child. A test never labels this fallback Runtime cleanup success.
            if cockpit_verification::supervisor_pipe::observe_owned_child_exit(&self.child).is_ok()
            {
                let _ = self.child.kill();
                let deadline = Instant::now() + Duration::from_secs(2);
                while Instant::now() < deadline {
                    match self.child.try_wait() {
                        Ok(Some(_)) | Err(_) => break,
                        Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                    }
                }
            }
        }
    }

    #[test]
    fn eof_cannot_replace_real_worker_exit() {
        let mut fixture = FixtureChild::new("delay");
        let result = finish_composition_child_until(
            &mut fixture.child,
            Instant::now() + Duration::from_millis(35),
        );
        let cleanup = fixture.cleanup();
        assert!(result.is_err_and(|error| error.contains("reaped=false")));
        assert!(cleanup.is_ok(), "test-owned final cleanup: {cleanup:?}");
    }

    #[test]
    fn natural_worker_exit_is_observed_and_reaped() {
        let mut fixture = FixtureChild::new("normal");
        let result = finish_composition_child_until(
            &mut fixture.child,
            Instant::now() + Duration::from_secs(1),
        );
        fixture.reaped = result.is_ok();
        assert!(result.is_ok_and(|status| status.success()));
    }

    #[test]
    fn reaped_pid_never_authorizes_late_cleanup_signals() {
        let mut fixture = FixtureChild::new("normal");
        let status = finish_composition_child_until(
            &mut fixture.child,
            Instant::now() + Duration::from_secs(1),
        );
        fixture.reaped = status.is_ok();
        assert!(status.is_ok_and(|status| status.success()));
        let cleanup = terminate_composition_child_until(
            &mut fixture.child,
            Instant::now() + Duration::from_millis(50),
        );
        assert!(
            cleanup.is_err_and(|error| error.contains("wait ownership lost")
                && error.contains("no further signal"))
        );
    }

    #[test]
    fn bounded_encoder_keeps_exact_limit_and_never_truncates() {
        let frame =
            encode_composition_frame(&serde_json::json!({"x":"original"}), 128).expect("frame");
        assert_eq!(
            encode_composition_frame(&serde_json::json!({"x":"original"}), frame.len())
                .expect("exact limit"),
            frame
        );
        assert!(
            encode_composition_frame(&serde_json::json!({"x":"original"}), frame.len() - 1)
                .is_err()
        );
    }
}

#[cfg(test)]
mod composition_parent_observation_tests {
    use super::*;
    use cockpit_protocol::CompositionBinding;
    use cockpit_verification::composition::{
        CompositionCleanup, CompositionCommand, CompositionExecutionRecord, CompositionIdentity,
        ReuseDecision, ReuseDecisionKind,
    };
    use cockpit_verification::{
        CompositionParentCatalog, CompositionParentCleanup, CompositionParentEvent,
        CompositionParentIdentity, CompositionParentObservation,
    };

    fn digest(text: &str) -> Digest {
        Digest::sha256_bytes(text.as_bytes())
    }

    // Synthetic store/reducer fixture only. These passed child fields are NOT
    // evidence that any command ran. Real subprocess tests remain separate.
    fn fixture(nonce: &str, timestamp: u128) -> (CompositionParentIdentity, CompositionAttempt) {
        let binding = CompositionBinding {
            schema_version: 1,
            repository_id: digest("fixture-repository"),
            binding_id: "fixture".into(),
            target_branch: "main".into(),
            target_sha: "a".repeat(40),
            participant_work_items: vec!["WI-PARENT-FIXTURE".into()],
            participant_heads: vec!["b".repeat(40)],
            contract_digests: vec![digest("contract")],
            verifier: RuntimeCapabilityBinding {
                schema_version: 1,
                runtime_version: env!("CARGO_PKG_VERSION").into(),
                runtime_digest: digest("synthetic-fixture-runtime"),
                capability: cockpit_protocol::ENVIRONMENT_DRIFT_CAPABILITY.into(),
            },
        };
        let command = CompositionCommand {
            node_id: "node".into(),
            program: "true".into(),
            args: Vec::new(),
            depends_on: Vec::new(),
            environment: Default::default(),
            input_paths: Vec::new(),
            covered_scenarios: Vec::new(),
            covered_constraints: Vec::new(),
        };
        let input = CompositionInput {
            repository_root: "/synthetic/repository".into(),
            state_dir: "/synthetic/state".into(),
            binding: binding.clone(),
            identity: CompositionIdentity::default(),
            commands: vec![command],
            reusable_node_ids: Vec::new(),
            preconditions: Vec::new(),
            timeout_seconds: 1,
        };
        let attempt_id = new_supervised_composition_attempt_id(&input, nonce);
        let owner = cockpit_verification::CompositionProcessIdentity {
            process_id: 100,
            start_time_ticks: Some(10),
            process_group_id: Some(100),
            session_id: Some(100),
        };
        let supervisor = cockpit_verification::CompositionProcessIdentity {
            process_id: 200,
            start_time_ticks: Some(20),
            process_group_id: Some(100),
            session_id: Some(100),
        };
        let mut receipt = CompositionSupervisorReceipt {
            schema_version: 1,
            backend: CompositionSupervisorBackend::LinuxSubreaper,
            attempt_id: attempt_id.clone(),
            run_nonce: nonce.into(),
            generation: 1,
            owner: owner.clone(),
            supervisor: supervisor.clone(),
            linux_boot_id: Some("fixture-boot".into()),
            environment_digest: digest("environment"),
            runtime_version: binding.verifier.runtime_version.clone(),
            runtime_digest: binding.verifier.runtime_digest.clone(),
            repository_id: binding.repository_id.clone(),
            target_sha: binding.target_sha.clone(),
            snapshot_digest: digest("tree"),
            command_plan_digest: cockpit_verification::composition_commands_digest(&input.commands),
            input_environment_digest: input_environment_digest(&input),
            execution_records_digest: execution_records_digest(&[]),
            descendants_reaped_to_echild: false,
        };
        let identity = CompositionParentIdentity {
            attempt_id: attempt_id.clone(),
            run_nonce: nonce.into(),
            binding: binding.clone(),
            launch_input_identity: input.identity.clone(),
            generation: 1,
            owner,
            child_pid: supervisor.process_id,
            observed_supervisor: Some(supervisor),
            registered_receipt: Some(receipt.clone()),
            command_plan_digest: receipt.command_plan_digest.clone(),
            input_environment_digest: receipt.input_environment_digest.clone(),
            attempt_started_at_unix_nanos: timestamp,
        };
        let records = vec![CompositionExecutionRecord {
            node_id: "node".into(),
            program: "true".into(),
            args: Vec::new(),
            identity_digest: digest("node-identity"),
            spawned: true,
            reused: false,
            passed: true,
            exit_code: Some(0),
            termination_signal: None,
            stdout: "saved-child-output".into(),
            stderr: "saved-child-stderr".into(),
            output_digest: digest("output"),
            timed_out: false,
            predecessor_attempt_id: None,
        }];
        receipt.execution_records_digest = execution_records_digest(&records);
        receipt.descendants_reaped_to_echild = true;
        let mut execution_identity = input.identity;
        execution_identity.command_digest = receipt.command_plan_digest.clone();
        let child = CompositionAttempt {
            schema_version: 3,
            attempt_id,
            binding,
            identity: execution_identity,
            preconditions: Vec::new(),
            isolated_worktree: "/synthetic/owned-worktree".into(),
            text_conflicts: Vec::new(),
            execution_records: records,
            processes_spawned: 1,
            reuse_decision: ReuseDecision {
                kind: ReuseDecisionKind::Execute,
                reason: "synthetic fixture".into(),
                predecessor_attempt_id: None,
            },
            passed: true,
            failure: None,
            execution_outcome: CompositionExecutionOutcome::Passed,
            execution_evidence_complete: true,
            supervisor_receipt: Some(receipt),
            cleanup_disposition: CompositionCleanupDisposition::Cleaned,
            owner_termination_signal: None,
            cleanup: Some(CompositionCleanup {
                attempted: true,
                removed: true,
                error: None,
            }),
            recorded_at_unix_nanos: timestamp + 1,
            owner_pid: Some(200),
            process_observation_schema_version: 1,
            active_execution_node: None,
            active_process_group_id: None,
            active_process_group_identity: None,
            owned_tree_termination_unknown: false,
        };
        (identity, child)
    }

    fn failure(identity: &CompositionParentIdentity, phase: &str) -> CompositionParentObservation {
        CompositionParentObservation::new(
            identity.clone(),
            CompositionParentEvent::Failure {
                phase: phase.into(),
                diagnostic: format!("injected-{phase}"),
                release_may_have_been_delivered: true,
            },
            identity.attempt_started_at_unix_nanos + 2,
        )
        .expect("typed synthetic failure")
    }

    fn cleanup(
        identity: &CompositionParentIdentity,
        failure: &CompositionParentObservation,
    ) -> CompositionParentObservation {
        CompositionParentObservation::new(
            identity.clone(),
            CompositionParentEvent::Cleanup {
                failure_observation_id: failure.observation_id.clone(),
                facts: CompositionParentCleanup {
                    child_reaped: Some(true),
                    exit_code: Some(0),
                    termination_signal: None,
                    actions: Vec::new(),
                    diagnostic: None,
                },
            },
            identity.attempt_started_at_unix_nanos + 3,
        )
        .expect("typed synthetic cleanup")
    }

    fn write_child(store: &NativeCompositionParentStore, child: &CompositionAttempt) -> Vec<u8> {
        let stem = format!(
            "composition-{}",
            child
                .attempt_id
                .bytes()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        let bytes = serde_json::to_vec_pretty(child).expect("child bytes");
        store
            .compositions
            .write(format!("{stem}.json"), &bytes)
            .expect("synthetic child record");
        bytes
    }

    fn capacity_launch(binding_id: String) -> CompositionSupervisorLaunch {
        let (identity, _) = fixture("capacity-nonce", 10);
        let mut binding = identity.binding;
        binding.binding_id = binding_id;
        let input = CompositionInput {
            repository_root: "/synthetic/repository".into(),
            state_dir: "/synthetic/state".into(),
            binding,
            identity: identity.launch_input_identity,
            commands: vec![CompositionCommand {
                node_id: "node".into(),
                program: "true".into(),
                args: Vec::new(),
                depends_on: Vec::new(),
                environment: Default::default(),
                input_paths: Vec::new(),
                covered_scenarios: Vec::new(),
                covered_constraints: Vec::new(),
            }],
            reusable_node_ids: Vec::new(),
            preconditions: Vec::new(),
            timeout_seconds: 1,
        };
        CompositionSupervisorLaunch {
            repository_root: input.repository_root.clone(),
            work_item_id: "WI-PARENT-FIXTURE".into(),
            generation: 1,
            attempt_id: new_supervised_composition_attempt_id(&input, "capacity-nonce"),
            run_nonce: "capacity-nonce".into(),
            owner: identity.owner,
            runtime_version: input.binding.verifier.runtime_version.clone(),
            runtime_digest: input.binding.verifier.runtime_digest.clone(),
            input,
        }
    }

    #[test]
    fn oversized_parent_identity_is_rejected_before_the_spawn_boundary() {
        let launch = capacity_launch("b".repeat(150_000));
        let bytes = encode_composition_frame(&launch, MAX_COMPOSITION_LAUNCH_BYTES)
            .expect("counterexample fits the independent 8MiB launch limit");
        assert!(bytes.len() < MAX_COMPOSITION_LAUNCH_BYTES);
        let identity = composition_parent_identity(&launch, 200, 10);
        let event = CompositionParentObservation::new(
            identity,
            CompositionParentEvent::Failure {
                phase: "take_pipe".into(),
                diagnostic: "original".into(),
                release_may_have_been_delivered: false,
            },
            11,
        )
        .expect("the original binding shape does not reject this ID");
        assert!(
            serde_json::to_vec_pretty(&event)
                .expect("untruncated record")
                .len()
                > MAX_COMPOSITION_PARENT_RECORD_BYTES as usize
        );
        let spawned = std::cell::Cell::new(false);
        let error = with_composition_parent_capacity(&launch, || {
            spawned.set(true);
            Ok(())
        })
        .expect_err("must reject before the same guard used by the real spawn");
        assert!(!spawned.get());
        assert!(error.contains("before spawn"), "{error}");
        assert_eq!(launch.input.binding.binding_id, "b".repeat(150_000));
    }

    #[test]
    fn admitted_parent_identity_keeps_the_original_spawn_and_full_event_budget() {
        let launch = capacity_launch("normal-binding".into());
        let before = serde_json::to_vec(&launch).expect("original launch");
        let calls = std::cell::Cell::new(0);
        let value = with_composition_parent_capacity(&launch, || {
            calls.set(calls.get() + 1);
            Ok(71)
        })
        .expect("ordinary identity leaves the reserved failure and cleanup capacity");
        assert_eq!(value, 71);
        assert_eq!(calls.get(), 1);
        assert_eq!(
            serde_json::to_vec(&launch).expect("unchanged launch"),
            before
        );
        let (identity, _) = fixture("event-capacity", 10);
        let event = CompositionParentObservation::new(
            identity,
            CompositionParentEvent::Failure {
                phase: "ready_read".into(),
                diagnostic: "\0".repeat(9_000),
                release_may_have_been_delivered: true,
            },
            11,
        )
        .expect("original failure observation");
        validate_composition_parent_event_capacity(&event.facts.event)
            .expect("JSON-expanded event is within the 64KiB encoded budget");
        assert!(
            serde_json::to_vec_pretty(&event)
                .expect("full record")
                .len()
                <= MAX_COMPOSITION_PARENT_RECORD_BYTES as usize
        );
    }

    #[test]
    fn oversized_diagnostics_are_explicit_publication_failures_without_truncation() {
        let root = tempfile::tempdir().expect("root");
        let store = NativeCompositionParentStore::open(root.path(), true)
            .expect("open")
            .expect("store");
        let (identity, _) = fixture("event-capacity", 10);
        let original = "\0".repeat(12_000);
        for event in [
            CompositionParentEvent::Failure {
                phase: "ready_read".into(),
                diagnostic: original.clone(),
                release_may_have_been_delivered: true,
            },
            CompositionParentEvent::Cleanup {
                failure_observation_id: digest("failure"),
                facts: CompositionParentCleanup {
                    child_reaped: Some(false),
                    exit_code: None,
                    termination_signal: None,
                    actions: vec!["SIGINT(pid=200)".into()],
                    diagnostic: Some(original.clone()),
                },
            },
        ] {
            let observation = CompositionParentObservation::new(identity.clone(), event, 11)
                .expect("full typed observation is still available in memory");
            let error = store
                .publish(&observation)
                .expect_err("encoded event is too large");
            assert!(
                error.contains("event exceeds its encoded byte limit"),
                "{error}"
            );
            match &observation.facts.event {
                CompositionParentEvent::Failure { diagnostic, .. } => {
                    assert_eq!(diagnostic, &original)
                }
                CompositionParentEvent::Cleanup { facts, .. } => {
                    assert_eq!(facts.diagnostic.as_ref(), Some(&original))
                }
            }
            assert!(
                store
                    .observations
                    .as_ref()
                    .expect("directory")
                    .entries()
                    .expect("entries")
                    .next()
                    .is_none(),
                "no truncated observation may be published"
            );
        }
    }

    #[test]
    fn saved_success_then_each_parent_failure_stays_unknown_in_both_consumers() {
        for phase in [
            "reply_eof",
            "reply_partial",
            "reply_trailing",
            "worker_nonzero",
            "abort_mismatch",
            "missing_attempt",
            "ownership_mismatch",
        ] {
            let root = tempfile::tempdir().expect("native store root");
            let store = NativeCompositionParentStore::open(root.path(), true)
                .expect("open")
                .expect("store");
            let (identity, child) = fixture("first-nonce", 10);
            let original = write_child(&store, &child);
            let observation = failure(&identity, phase);
            store.publish(&observation).expect("append known failure");
            store
                .publish(&cleanup(&identity, &observation))
                .expect("append real cleanup separately");
            let catalog =
                load_native_composition_parent_catalog(root.path()).expect("shared cap reader");
            let kernel = catalog
                .latest_for_binding(&child.binding)
                .expect("kernel latest");
            let projection = latest_composition_attempt(root.path(), "WI-PARENT-FIXTURE")
                .expect("projection")
                .expect("latest");
            for view in [kernel, projection] {
                assert!(!view.parent_context_valid, "{phase} became reusable");
                let raw = view.child_attempt.expect("retained raw execution");
                assert!(raw.passed);
                assert_eq!(raw.execution_outcome, CompositionExecutionOutcome::Passed);
                assert_eq!(raw.execution_records, child.execution_records);
                assert_eq!(
                    raw.cleanup_disposition,
                    CompositionCleanupDisposition::Cleaned
                );
                assert!(
                    raw.supervisor_receipt
                        .as_ref()
                        .expect("receipt")
                        .descendants_reaped_to_echild
                );
                assert!(
                    !raw.owned_tree_termination_unknown,
                    "do not falsify known tree facts"
                );
                assert_eq!(serde_json::to_vec_pretty(&raw).expect("same raw"), original);
            }
        }
    }

    #[test]
    fn late_child_write_cannot_erase_parent_failure_or_fall_back_to_older_success() {
        let root = tempfile::tempdir().expect("root");
        let store = NativeCompositionParentStore::open(root.path(), true)
            .expect("open")
            .expect("store");
        let (_, older) = fixture("older", 1);
        write_child(&store, &older);
        let (identity, later) = fixture("later", 10);
        store
            .publish(&failure(&identity, "partial"))
            .expect("failure before child publication");
        assert_eq!(
            store
                .catalog()
                .expect("view")
                .latest_for_work_item("WI-PARENT-FIXTURE")
                .expect("latest")
                .attempt_id,
            later.attempt_id
        );
        let original = write_child(&store, &later); // real writer may finish late
        let view = store
            .catalog()
            .expect("view")
            .latest_for_binding(&later.binding)
            .expect("latest");
        assert_eq!(view.attempt_id, later.attempt_id);
        assert!(!view.parent_context_valid);
        assert_eq!(
            serde_json::to_vec_pretty(&view.child_attempt.expect("raw")).expect("bytes"),
            original
        );
    }

    #[test]
    fn exact_observation_is_idempotent_and_new_nonce_supersedes_without_rewriting_old() {
        let root = tempfile::tempdir().expect("root");
        let store = NativeCompositionParentStore::open(root.path(), true)
            .expect("open")
            .expect("store");
        let (identity, child) = fixture("first", 10);
        let original = write_child(&store, &child);
        let event = failure(&identity, "worker_exit");
        store.publish(&event).expect("first");
        store.publish(&event).expect("idempotent");
        store.publish(&cleanup(&identity, &event)).expect("cleanup");
        let (_, next) = fixture("fresh-new-nonce", 20);
        write_child(&store, &next);
        let catalog = store.catalog().expect("catalog");
        let selected = catalog.latest_for_binding(&next.binding).expect("latest");
        assert_eq!(selected.attempt_id, next.attempt_id);
        assert!(selected.parent_context_valid);
        let old = catalog.view_for_attempt(&child.attempt_id).expect("old");
        assert!(!old.parent_context_valid);
        assert_eq!(
            serde_json::to_vec_pretty(&old.child_attempt.expect("raw")).expect("bytes"),
            original
        );
    }

    #[test]
    fn corrupt_conflicting_or_nonregular_observations_never_fall_back_to_success() {
        let root = tempfile::tempdir().expect("root");
        let store = NativeCompositionParentStore::open(root.path(), true)
            .expect("open")
            .expect("store");
        let (identity, child) = fixture("first", 10);
        write_child(&store, &child);
        let observation = failure(&identity, "eof");
        let name = format!(
            "{}.json",
            observation
                .observation_id
                .to_string()
                .strip_prefix("sha256:")
                .expect("digest")
        );
        let records = store.observations.as_ref().expect("records");
        records
            .write(&name, b"malformed")
            .expect("real conflicting destination");
        assert!(store.publish(&observation).is_err());
        assert!(store.catalog().is_err());
        records
            .remove_file(&name)
            .expect("fixture-owned fault file");
        records.create_dir(&name).expect("nonregular fixture");
        assert!(store.catalog().is_err());
    }

    #[test]
    fn observation_binding_tamper_and_orphan_cleanup_are_rejected() {
        let (identity, child) = fixture("first", 10);
        let event = failure(&identity, "partial");
        let mut tampered = event.clone();
        tampered.facts.identity.run_nonce = "foreign".into();
        assert!(
            CompositionParentCatalog::from_native_observations(vec![child.clone()], vec![tampered])
                .is_err()
        );
        let orphan = cleanup(&identity, &event);
        assert!(
            CompositionParentCatalog::from_native_observations(vec![child.clone()], vec![orphan])
                .is_err()
        );
        let mut foreign = identity.clone();
        foreign.owner.process_id += 1;
        foreign
            .registered_receipt
            .as_mut()
            .expect("bound receipt")
            .owner = foreign.owner.clone();
        // Both observations are valid in isolation; the catalog must still
        // reject incompatible owners for the same exact attempt identity.
        let conflict = failure(&foreign, "exit");
        assert!(
            CompositionParentCatalog::from_native_observations(vec![child], vec![event, conflict])
                .is_err()
        );
    }

    #[test]
    fn failed_parent_ancestor_also_blocks_a_reused_descendant_view() {
        let (identity, first) = fixture("first", 10);
        let (_, mut next) = fixture("next", 20);
        next.execution_records[0].spawned = false;
        next.execution_records[0].reused = true;
        next.execution_records[0].predecessor_attempt_id = Some(first.attempt_id.clone());
        next.processes_spawned = 0;
        next.supervisor_receipt
            .as_mut()
            .expect("receipt")
            .execution_records_digest = execution_records_digest(&next.execution_records);
        let catalog = CompositionParentCatalog::from_native_observations(
            vec![first, next.clone()],
            vec![failure(&identity, "exit")],
        )
        .expect("catalog");
        let view = catalog.latest_for_binding(&next.binding).expect("latest");
        assert!(!view.parent_context_valid);
        assert!(
            view.child_attempt.expect("raw").passed,
            "do not forge old Unknown"
        );
    }

    #[cfg(unix)]
    #[test]
    fn publication_failure_retains_in_memory_facts_and_never_claims_durable_unknown() {
        let root = tempfile::tempdir().expect("root");
        let store = NativeCompositionParentStore::open(root.path(), true)
            .expect("store")
            .expect("present");
        let (mut identity, mut raw) = fixture("publication-fault", 10);
        let mut fixture_child = super::composition_unix_channel_tests::FixtureChild::new("normal");
        let child = &mut fixture_child.child;
        let pid = child.id();
        let status =
            match finish_composition_child_until(child, Instant::now() + Duration::from_secs(1)) {
                Ok(status) => status,
                Err(error) => {
                    let cleanup = cleanup_owned_composition_child(
                        child,
                        Instant::now() + Duration::from_secs(2),
                    );
                    panic!("fixture did not exit: {error}; {cleanup:?}");
                }
            };
        fixture_child.reaped = true;
        identity.child_pid = pid;
        identity
            .observed_supervisor
            .as_mut()
            .expect("synthetic identity")
            .process_id = pid;
        identity
            .registered_receipt
            .as_mut()
            .expect("synthetic receipt")
            .supervisor
            .process_id = pid;
        raw.supervisor_receipt
            .as_mut()
            .expect("receipt")
            .supervisor
            .process_id = pid;
        raw.owner_pid = Some(pid);
        let original = write_child(&store, &raw);
        BEFORE_PARENT_OBSERVATION_PUBLISH.with_borrow_mut(|pending| {
            *pending = Some(Box::new(|directory, name| {
                directory
                    .write(name, b"conflicting fixture-owned bytes")
                    .expect("publication conflict injection");
            }));
        });
        let error = record_failed_parent_operation(
            root.path(),
            &mut fixture_child.child,
            identity,
            "worker_nonzero",
            true,
            Some(status),
            recovery_error("injected parent terminal failure".into()),
        );
        let CollaborationExecutionError::ParentObservationPublication {
            observations,
            message,
        } = error
        else {
            panic!("publication failure was not preserved");
        };
        assert!(message.contains("failure observation publication failed"));
        assert_eq!(observations.len(), 2);
        assert!(
            observations
                .iter()
                .all(|observation| !observation.facts.passed)
        );
        assert!(
            matches!(&observations[1].facts.event, CompositionParentEvent::Cleanup { facts, .. }
            if facts.child_reaped == Some(true) && facts.actions.is_empty())
        );
        assert!(
            store.catalog().is_err(),
            "conflicting/orphan facts cannot expose old success"
        );
        let bytes = store
            .compositions
            .read(format!(
                "composition-{}.json",
                raw.attempt_id
                    .bytes()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            ))
            .expect("old raw remains");
        assert_eq!(bytes, original);
    }

    #[cfg(unix)]
    #[test]
    fn fifo_observation_is_rejected_without_waiting_for_a_peer() {
        use std::os::unix::ffi::OsStrExt;
        let root = tempfile::tempdir().expect("root");
        let store = NativeCompositionParentStore::open(root.path(), true)
            .expect("open")
            .expect("store");
        let path = store.path.join("parent-observations/fifo.json");
        let cpath = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("path");
        // SAFETY: fresh fixture-owned pathname, no peer process.
        assert_eq!(unsafe { libc::mkfifo(cpath.as_ptr(), 0o600) }, 0);
        let started = Instant::now();
        assert!(store.catalog().is_err());
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[cfg(unix)]
    #[test]
    fn observation_directory_replacement_is_not_a_successful_publication() {
        let root = tempfile::tempdir().expect("root");
        let store = NativeCompositionParentStore::open(root.path(), true)
            .expect("open")
            .expect("store");
        let (identity, _) = fixture("first", 10);
        let path = store.path.join("parent-observations");
        fs::rename(&path, store.path.join("retained-original-observations"))
            .expect("retain fixture original");
        fs::create_dir(&path).expect("replacement");
        assert!(store.publish(&failure(&identity, "publication")).is_err());
    }
}
