use crate::{CoordinationError, CoordinationStore};
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use cockpit_git::GitRepository;
use cockpit_protocol::{
    ConsumedOutcome, CoordinationEvent, CoordinationIntent, CoordinationRecovery,
    CoordinationRequest, CoordinationRequestState, ProviderOutcomeKey, RuntimeCapabilityBinding,
    RuntimeContext, WorktreeRegistration,
};
use cockpit_verification::{
    CompositionAttempt, CompositionError, CompositionInput, CompositionPrecondition,
    run_composition,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path};
use std::process::Command;
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
    pub composition_applicability: String,
    pub target_merge_state: String,
    pub cleanup_state: String,
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

fn provider_outcome_key(dependency: &ConsumedOutcome) -> ProviderOutcomeKey {
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
    let root = Path::new(&provider.worktree_path);
    let expected_reference = format!(".ai/evidence/{}.verification.json", provider.work_item_id);
    if !outcome
        .evidence_refs
        .iter()
        .any(|reference| reference == &expected_reference)
    {
        return false;
    }
    let Some(publication) = projection.events.iter().find(|event| {
        event.kind == cockpit_protocol::CoordinationEventKind::OutcomePublished
            && event.repository_id == provider.repository_id
            && event.work_item_id == provider.work_item_id
            && event.generation == provider.generation
            && event
                .outcome_ids
                .iter()
                .any(|outcome_id| outcome_id == &outcome.outcome_id)
            && event
                .evidence_refs
                .iter()
                .any(|reference| reference == &expected_reference)
    }) else {
        return false;
    };
    let Ok(bytes) = read_registered_worktree_file(root, &expected_reference) else {
        return false;
    };
    if publication.evidence_digests.get(&expected_reference)
        != Some(&cockpit_core::Digest::sha256_bytes(&bytes))
    {
        return false;
    }
    let Ok(envelope) = serde_json::from_slice::<crate::VerificationEvidenceV2>(&bytes) else {
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
    let Ok(contract) = read_registered_contract(provider) else {
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
    crate::verification_evidence_state(root, &contract, &snapshot, false, Some(&runtime))
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
            let mut offset = 0;
            while offset < count {
                if count - offset < header_size {
                    return Err(format!(
                        "malformed directory mutation event for {reference}"
                    ));
                }
                let mask = u32::from_ne_bytes(
                    buffer[offset + 4..offset + 8]
                        .try_into()
                        .expect("fixed inotify event mask width"),
                );
                let name_len = u32::from_ne_bytes(
                    buffer[offset + 12..offset + 16]
                        .try_into()
                        .expect("fixed inotify event name width"),
                ) as usize;
                let event_size = header_size.checked_add(name_len).ok_or_else(|| {
                    format!("directory mutation event length overflow for {reference}")
                })?;
                if event_size > count - offset {
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
            change.flags = (libc::EV_ADD | libc::EV_CLEAR) as u16;
            change.fflags = (libc::NOTE_RENAME | libc::NOTE_DELETE) as u32;
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
    let mut opened = open_registered_worktree_file_with_context(root, reference)?;
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
        let outcome_ids = if event.outcome_ids.is_empty() {
            provider
                .declaration
                .provided_outcomes
                .iter()
                .map(|outcome| outcome.outcome_id.clone())
                .collect::<Vec<_>>()
        } else {
            event.outcome_ids.clone()
        };
        for outcome_id in outcome_ids {
            pending.push_back((
                event.clone(),
                ProviderOutcomeKey {
                    provider_work_item_id: provider.work_item_id.clone(),
                    outcome_id,
                },
            ));
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
        schema_version: 1,
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
        composition_applicability: "not_observed".into(),
        target_merge_state: "not_observed".into(),
        cleanup_state: "not_observed".into(),
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
        admit_collaboration_action(&store, work_item_id, registration.generation, action)
    });
    let mut admission_unknowns = Vec::new();
    let blockers = match admission {
        Some(Ok(admission)) => {
            admission_unknowns = admission.unknowns;
            admission.blockers
        }
        Some(Err(error)) => {
            admission_unknowns.push(format!("collaboration_admission:{error}"));
            Vec::new()
        }
        None => Vec::new(),
    };
    let mut unknowns = projection.unknowns.clone();
    unknowns.extend(admission_unknowns);
    let (
        (composition_state, target_merge_state, cleanup_state, reusable_checks),
        composition_applicability,
    ) = match latest_composition_attempt(store.root(), work_item_id) {
        Ok(Some(attempt)) => (
            composition_facts(store.root(), &attempt),
            composition_applicability(store.root(), &attempt, &projection),
        ),
        Ok(None) => (
            (
                "not_observed".into(),
                "not_observed".into(),
                "not_observed".into(),
                Vec::new(),
            ),
            "not_observed".into(),
        ),
        Err(error) => {
            unknowns.push(format!("composition_projection:{error}"));
            (
                (
                    "unknown".into(),
                    "unknown".into(),
                    "unknown".into(),
                    Vec::new(),
                ),
                "unknown".into(),
            )
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
        schema_version: 1,
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
        composition_state,
        composition_applicability,
        target_merge_state,
        cleanup_state,
        revalidation: if invalidated_event_ids.is_empty() {
            "not_required"
        } else {
            "required"
        }
        .into(),
        reusable_checks,
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
) -> Result<Option<CompositionAttempt>, String> {
    let directory = coordination_root.join("compositions");
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let mut latest = None;
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if !metadata.file_type().is_file() {
            return Err(format!(
                "composition record is not a regular file: {}",
                path.display()
            ));
        }
        let attempt: CompositionAttempt =
            serde_json::from_slice(&fs::read(&path).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        if !attempt
            .binding
            .participant_work_items
            .iter()
            .any(|participant| participant == work_item_id)
        {
            continue;
        }
        if latest.as_ref().is_none_or(|current: &CompositionAttempt| {
            (attempt.recorded_at_unix_nanos, &attempt.attempt_id)
                > (current.recorded_at_unix_nanos, &current.attempt_id)
        }) {
            latest = Some(attempt);
        }
    }
    Ok(latest)
}

fn composition_facts(
    root: &Path,
    attempt: &CompositionAttempt,
) -> (String, String, String, Vec<String>) {
    let composition_state = if attempt.passed {
        "passed"
    } else if attempt.failure.as_deref() == Some("in_progress") {
        "in_progress"
    } else {
        "failed"
    };
    let target_merge_state = target_merge_state(root, attempt);
    let cleanup_state = match &attempt.cleanup {
        Some(cleanup) if cleanup.removed => "cleaned",
        Some(cleanup) if cleanup.attempted => "failed",
        Some(_) => "not_observed",
        None => "unknown",
    };
    let reusable_checks = attempt
        .execution_records
        .iter()
        .filter(|record| record.reused)
        .map(|record| record.node_id.clone())
        .collect();
    (
        composition_state.into(),
        target_merge_state,
        cleanup_state.into(),
        reusable_checks,
    )
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
    let registration = projection
        .registrations
        .iter()
        .find(|registration| registration.work_item_id == work_item_id);
    let mut blockers = projection
        .blockers
        .get(work_item_id)
        .cloned()
        .unwrap_or_default();
    let mut unknowns = projection.unknowns.clone();
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
            let invalidated_outcomes = transitive_invalidated_outcomes(&projection);
            let outcome_blockers = dependency_blockers_for_registration(
                &projection,
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
    Ok(CollaborationAdmission {
        work_item_id: work_item_id.into(),
        generation,
        allowed: blockers.is_empty() && unknowns.is_empty(),
        affected,
        blockers,
        unknowns,
        refreshed_events: projection.events.len(),
    })
}

pub fn run_admitted_composition(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    input: CompositionInput,
) -> Result<CompositionAttempt, CollaborationExecutionError> {
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
    let admission = admit_collaboration_action(
        store,
        work_item_id,
        generation,
        CollaborationAction {
            kind: CollaborationActionKind::Composition,
            consumer_work_item_id: work_item_id.into(),
            outcomes,
        },
    )?;
    if !admission.allowed {
        return Err(CollaborationExecutionError::Blocked {
            work_item_id: work_item_id.into(),
            blockers: admission.blockers,
            unknowns: admission.unknowns,
        });
    }
    let mut input = input;
    let identity_blockers =
        verify_composition_identity(store, work_item_id, generation, &mut input)?;
    if !identity_blockers.is_empty() {
        return Err(CollaborationExecutionError::Blocked {
            work_item_id: work_item_id.into(),
            blockers: identity_blockers,
            unknowns: Vec::new(),
        });
    }
    // The caller may be in a linked worktree. Composition facts are shared
    // coordination evidence, so they must be written below the Git common
    // directory rather than the caller's private checkout.
    input.state_dir = store.root().join("compositions");
    Ok(run_composition(input)?)
}

fn read_registered_contract(
    registration: &WorktreeRegistration,
) -> Result<cockpit_protocol::Contract, CoordinationError> {
    let root = Path::new(&registration.worktree_path);
    let reference = format!(
        ".ai/work-items/active/{}.contract.json",
        registration.work_item_id
    );
    let bytes = read_registered_worktree_file(root, &reference).map_err(|error| {
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
    let mut projection = collaboration_projection(store)?;
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
    };
    let published_key = ProviderOutcomeKey {
        provider_work_item_id: work_item_id.into(),
        outcome_id: outcome_id.into(),
    };
    let verification_required = projection.registrations.iter().any(|consumer| {
        consumer
            .declaration
            .consumed_outcomes
            .iter()
            .any(|dependency| {
                provider_outcome_key(dependency) == published_key
                    && dependency.verification_required
            })
    });
    if verification_required {
        projection.events.push(event.clone());
        if !verification_evidence_is_complete(&projection, &provider, outcome) {
            return Err(CoordinationError::RecoveryRequired(format!(
                "outcome {outcome_id} requires a current successful verification receipt before publication"
            )));
        }
    }
    store.publish_event(event)
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
    admit_collaboration_action(
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
