//! Immutable portable-record storage. These bytes never authenticate a publisher.

use crate::{
    ObserverError,
    evidence_store::{
        create_and_open_cap_directory, open_cap_directory_nofollow_strict, valid_sha256_digest,
        write_cap_immutable,
    },
    legacy_closeout_source::{PinnedRoot, directory_identity},
};
use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use serde::Deserialize;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(super) const MAX_PORTABLE_INPUT_BYTES: u64 = 64 * 1024 * 1024;
pub(super) const MAX_PORTABLE_MARKER_BYTES: u64 = 128 * 1024;
const LOCK_WAIT: Duration = Duration::from_secs(2);

/// A cold read returns raw untrusted bytes, never a verified-source capability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PortableStoredRecord {
    pub(super) input: Vec<u8>,
    pub(super) marker: Vec<u8>,
    pub(super) record_ref: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Marker {
    schema_version: u32,
    record_id: String,
    input_digest: String,
}

fn error(path: &Path, code: &str) -> ObserverError {
    ObserverError::State {
        path: path.into(),
        message: format!("portable_store:{code}"),
    }
}

fn io_error(path: &Path, source: std::io::Error) -> ObserverError {
    ObserverError::Read {
        path: path.into(),
        source,
    }
}

fn check_ids(root: &Path, wi: &str, id: &str) -> Result<(), ObserverError> {
    crate::validate_work_item_id(wi)?;
    if !valid_sha256_digest(id) {
        return Err(error(root, "record_id_invalid"));
    }
    Ok(())
}

fn parse_marker(path: &Path, id: &str, bytes: &[u8]) -> Result<Marker, ObserverError> {
    if bytes.len() as u64 > MAX_PORTABLE_MARKER_BYTES {
        return Err(error(path, "marker_byte_limit"));
    }
    crate::reject_duplicate_json_keys(bytes)
        .map_err(|_| error(path, "marker_duplicate_or_invalid_json"))?;
    let marker: Marker =
        serde_json::from_slice(bytes).map_err(|_| error(path, "marker_schema_invalid"))?;
    if marker.schema_version != 1
        || marker.record_id != id
        || !valid_sha256_digest(&marker.input_digest)
    {
        return Err(error(path, "marker_binding_invalid"));
    }
    Ok(marker)
}

fn digest(bytes: &[u8]) -> String {
    cockpit_core::Digest::sha256_bytes(bytes).to_string()
}

struct Namespace {
    root: PinnedRoot,
    root_path: PathBuf,
    names: Vec<String>,
    identities: Vec<[u64; 2]>,
    portable: Dir,
    path: PathBuf,
}

impl Namespace {
    fn open(root: &Path, wi: &str, create: bool) -> Result<Option<Self>, ObserverError> {
        let pinned = PinnedRoot::open(root)?;
        let mut dir = pinned.directory()?;
        let names = vec![
            ".ai".to_owned(),
            "evidence".to_owned(),
            format!("{wi}.verification-admissions"),
            "portable".to_owned(),
        ];
        let mut identities = Vec::with_capacity(names.len());
        let mut path = root.to_path_buf();
        for name in &names {
            path.push(name);
            if create {
                dir = create_and_open_cap_directory(&dir, name, &path)?;
            } else if !exists(&dir, name, &path)? {
                pinned.require_current()?;
                return Ok(None);
            } else {
                dir = open_cap_directory_nofollow_strict(&dir, name, &path)?;
            }
            identities.push(directory_identity(&path, &dir)?);
        }
        let namespace = Self {
            root: pinned,
            root_path: root.into(),
            names,
            identities,
            portable: dir,
            path,
        };
        namespace.require_current()?;
        Ok(Some(namespace))
    }

    fn require_current(&self) -> Result<(), ObserverError> {
        self.root.require_current()?;
        let mut current = self.root.directory()?;
        let mut path = self.root_path.clone();
        for (name, identity) in self.names.iter().zip(&self.identities) {
            path.push(name);
            current = open_cap_directory_nofollow_strict(&current, name, &path)?;
            if directory_identity(&path, &current)? != *identity {
                return Err(error(&path, "directory_identity_changed"));
            }
        }
        Ok(())
    }

    fn child(&self, name: &str, create: bool) -> Result<Option<ChildDir>, ObserverError> {
        let path = self.path.join(name);
        let dir = if create {
            create_and_open_cap_directory(&self.portable, name, &path)?
        } else if !exists(&self.portable, name, &path)? {
            return Ok(None);
        } else {
            open_cap_directory_nofollow_strict(&self.portable, name, &path)?
        };
        Ok(Some(ChildDir {
            identity: directory_identity(&path, &dir)?,
            dir,
            name: name.into(),
            path,
        }))
    }
}

struct ChildDir {
    identity: [u64; 2],
    dir: Dir,
    name: String,
    path: PathBuf,
}

impl ChildDir {
    fn require_current(&self, namespace: &Namespace) -> Result<(), ObserverError> {
        namespace.require_current()?;
        let current =
            open_cap_directory_nofollow_strict(&namespace.portable, &self.name, &self.path)?;
        if directory_identity(&self.path, &current)? != self.identity {
            return Err(error(&self.path, "directory_identity_changed"));
        }
        Ok(())
    }
}

fn exists(parent: &Dir, name: &str, path: &Path) -> Result<bool, ObserverError> {
    match parent.symlink_metadata(name) {
        Ok(_) => Ok(true),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(io_error(path, source)),
    }
}

fn nonblocking_options(write: bool, create_new: bool) -> OpenOptions {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(write)
        .create_new(create_new)
        .follow(FollowSymlinks::No);
    #[cfg(unix)]
    {
        use cap_fs_ext::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    options
}

struct StoreLock {
    _file: File,
    identity: [u64; 2],
}

impl StoreLock {
    fn acquire(
        namespace: &Namespace,
        create: bool,
        exclusive: bool,
    ) -> Result<Self, ObserverError> {
        let path = namespace.path.join(".store.lock");
        let (file, created) = match namespace
            .portable
            .open_with(".store.lock", &nonblocking_options(exclusive, create))
        {
            Ok(file) => (file, create),
            Err(source) if create && source.kind() == std::io::ErrorKind::AlreadyExists => (
                namespace
                    .portable
                    .open_with(".store.lock", &nonblocking_options(exclusive, false))
                    .map_err(|source| io_error(&path, source))?,
                false,
            ),
            Err(source) => return Err(io_error(&path, source)),
        };
        let metadata = file.metadata().map_err(|source| io_error(&path, source))?;
        if !metadata.is_file() {
            return Err(error(&path, "regular_lock_required"));
        }
        let identity = [metadata.dev(), metadata.ino()];
        let file = file.into_std();
        if created {
            file.sync_all().map_err(|source| io_error(&path, source))?;
            sync_directory(&namespace.portable, &namespace.path)?;
        }
        let deadline = Instant::now() + LOCK_WAIT;
        loop {
            let result = if exclusive {
                file.try_lock()
            } else {
                file.try_lock_shared()
            };
            match result {
                Ok(()) => break,
                Err(std::fs::TryLockError::WouldBlock) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(2));
                }
                Err(std::fs::TryLockError::WouldBlock) => {
                    return Err(error(&path, "lock_deadline"));
                }
                Err(std::fs::TryLockError::Error(source)) => return Err(io_error(&path, source)),
            }
        }
        let lock = Self {
            _file: file,
            identity,
        };
        lock.require_current(namespace)?;
        Ok(lock)
    }

    fn require_current(&self, namespace: &Namespace) -> Result<(), ObserverError> {
        namespace.require_current()?;
        let path = namespace.path.join(".store.lock");
        let current = namespace
            .portable
            .open_with(".store.lock", &nonblocking_options(false, false))
            .map_err(|source| io_error(&path, source))?;
        let metadata = current
            .metadata()
            .map_err(|source| io_error(&path, source))?;
        if !metadata.is_file() || [metadata.dev(), metadata.ino()] != self.identity {
            return Err(error(&path, "lock_identity_changed"));
        }
        Ok(())
    }
}

struct ReadFile {
    bytes: Vec<u8>,
    metadata: cap_std::fs::Metadata,
}

fn same_leaf(
    path: &Path,
    before: &cap_std::fs::Metadata,
    after: &cap_std::fs::Metadata,
) -> Result<bool, ObserverError> {
    let modified_same = before.modified().map_err(|source| io_error(path, source))?
        == after.modified().map_err(|source| io_error(path, source))?;
    #[cfg(unix)]
    let changed_same = cap_std::fs::MetadataExt::ctime(before)
        == cap_std::fs::MetadataExt::ctime(after)
        && cap_std::fs::MetadataExt::ctime_nsec(before)
            == cap_std::fs::MetadataExt::ctime_nsec(after);
    #[cfg(not(unix))]
    let changed_same = true;
    Ok(after.is_file()
        && before.len() == after.len()
        && [before.dev(), before.ino()] == [after.dev(), after.ino()]
        && modified_same
        && changed_same)
}

impl ReadFile {
    fn require_current(&self, parent: &Dir, name: &str, path: &Path) -> Result<(), ObserverError> {
        let current = parent
            .open_with(name, &nonblocking_options(false, false))
            .map_err(|source| io_error(path, source))?;
        let metadata = current
            .metadata()
            .map_err(|source| io_error(path, source))?;
        if !same_leaf(path, &self.metadata, &metadata)? {
            return Err(error(path, "leaf_identity_or_content_metadata_changed"));
        }
        Ok(())
    }
}

fn read_regular(
    parent: &Dir,
    name: &str,
    path: &Path,
    maximum: u64,
) -> Result<ReadFile, ObserverError> {
    let mut file = parent
        .open_with(name, &nonblocking_options(false, false))
        .map_err(|source| io_error(path, source))?;
    let before = file.metadata().map_err(|source| io_error(path, source))?;
    if !before.is_file() {
        return Err(error(path, "regular_file_required"));
    }
    if before.len() > maximum {
        return Err(error(path, "file_byte_limit"));
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| io_error(path, source))?;
    let after = file.metadata().map_err(|source| io_error(path, source))?;
    if bytes.len() as u64 > maximum {
        return Err(error(path, "file_byte_limit"));
    }
    if before.len() != bytes.len() as u64 || !same_leaf(path, &before, &after)? {
        return Err(error(path, "file_changed_during_read"));
    }
    let read = ReadFile {
        bytes,
        metadata: before,
    };
    read.require_current(parent, name, path)?;
    Ok(read)
}

// Reopen `.` relative to the capability: cap-std directory handles can be O_PATH.
// Windows directory fsync is not claimed; file sync_all failures still propagate.
fn sync_directory(dir: &Dir, path: &Path) -> Result<(), ObserverError> {
    #[cfg(windows)]
    {
        let _ = (dir, path);
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let mut options = OpenOptions::new();
        options.read(true);
        dir.open_with(".", &options)
            .and_then(|file| file.into_std().sync_all())
            .map_err(|source| io_error(path, source))
    }
}

// The immutable helper also accepts an existing equal destination. Flush that
// actual destination, not only the helper's newly-created comparison temporary.
fn sync_regular_file(
    parent: &Dir,
    name: &str,
    path: &Path,
    maximum: u64,
) -> Result<(), ObserverError> {
    let file = parent
        .open_with(name, &nonblocking_options(true, false))
        .map_err(|source| io_error(path, source))?;
    let before = file.metadata().map_err(|source| io_error(path, source))?;
    if !before.is_file() || before.len() > maximum {
        return Err(error(path, "regular_bounded_file_required"));
    }
    file.sync_all().map_err(|source| io_error(path, source))?;
    let after = file.metadata().map_err(|source| io_error(path, source))?;
    if !same_leaf(path, &before, &after)? {
        return Err(error(path, "file_changed_during_sync"));
    }
    ReadFile {
        bytes: Vec::new(),
        metadata: before,
    }
    .require_current(parent, name, path)
}

fn names(id: &str) -> (String, String) {
    (
        format!("{}.json", &id[7..]),
        format!("{}.pending", &id[7..]),
    )
}

fn read_committed(
    namespace: &Namespace,
    records: &ChildDir,
    id: &str,
    wi: &str,
) -> Result<Option<PortableStoredRecord>, ObserverError> {
    let (name, pending) = names(id);
    if exists(&records.dir, &pending, &records.path.join(&pending))? {
        return Err(error(&records.path, "publication_pending"));
    }
    if !exists(&records.dir, &name, &records.path.join(&name))? {
        records.require_current(namespace)?;
        return Ok(None);
    }
    let marker = read_regular(
        &records.dir,
        &name,
        &records.path.join(&name),
        MAX_PORTABLE_MARKER_BYTES,
    )?;
    let binding = parse_marker(&records.path.join(&name), id, &marker.bytes)?;
    let objects = namespace
        .child("objects", false)?
        .ok_or_else(|| error(&namespace.path, "committed_object_directory_missing"))?;
    let object_name = format!("{}.bin", &binding.input_digest[7..]);
    let input = read_regular(
        &objects.dir,
        &object_name,
        &objects.path.join(&object_name),
        MAX_PORTABLE_INPUT_BYTES,
    )?;
    if digest(&input.bytes) != binding.input_digest {
        return Err(error(&objects.path, "input_digest_mismatch"));
    }
    phase("read_complete", &namespace.root_path)?;
    marker.require_current(&records.dir, &name, &records.path.join(&name))?;
    input.require_current(&objects.dir, &object_name, &objects.path.join(&object_name))?;
    records.require_current(namespace)?;
    objects.require_current(namespace)?;
    Ok(Some(PortableStoredRecord {
        input: input.bytes,
        marker: marker.bytes,
        record_ref: format!(".ai/evidence/{wi}.verification-admissions/portable/records/{name}"),
    }))
}

pub(super) fn publish_portable_record(
    root: &Path,
    wi: &str,
    record_id: &str,
    input: &[u8],
    marker: &[u8],
) -> Result<PortableStoredRecord, ObserverError> {
    check_ids(root, wi, record_id)?;
    if input.len() as u64 > MAX_PORTABLE_INPUT_BYTES {
        return Err(error(root, "input_byte_limit"));
    }
    let binding = parse_marker(root, record_id, marker)?;
    if binding.input_digest != digest(input) {
        return Err(error(root, "input_digest_mismatch"));
    }
    let namespace =
        Namespace::open(root, wi, true)?.ok_or_else(|| error(root, "store_directory_missing"))?;
    let lock = StoreLock::acquire(&namespace, true, true)?;
    let records = namespace
        .child("records", true)?
        .ok_or_else(|| error(root, "records_directory_missing"))?;
    // All known record-id collisions are rejected before creating an object.
    if let Some(existing) = read_committed(&namespace, &records, record_id, wi)? {
        lock.require_current(&namespace)?;
        return if existing.input == input && existing.marker == marker {
            Ok(existing)
        } else {
            Err(error(&records.path, "record_id_conflict"))
        };
    }
    let objects = namespace
        .child("objects", true)?
        .ok_or_else(|| error(root, "objects_directory_missing"))?;
    let (name, pending) = names(record_id);
    let pending_path = records.path.join(&pending);
    // A failed publication retains this reservation; no cold read or retry can
    // mistake an installed-but-unsynced final marker for a complete record.
    write_cap_immutable(&records.dir, &pending, &pending_path, marker)?;
    sync_regular_file(
        &records.dir,
        &pending,
        &pending_path,
        MAX_PORTABLE_MARKER_BYTES,
    )?;
    let pending_reservation = read_regular(
        &records.dir,
        &pending,
        &pending_path,
        MAX_PORTABLE_MARKER_BYTES,
    )?;
    if pending_reservation.bytes != marker {
        return Err(error(&pending_path, "pending_binding_changed"));
    }
    sync_directory(&namespace.portable, &namespace.path)?;
    phase("pending_durable", root)?;
    lock.require_current(&namespace)?;
    objects.require_current(&namespace)?;
    records.require_current(&namespace)?;
    let object_name = format!("{}.bin", &binding.input_digest[7..]);
    let object_path = objects.path.join(&object_name);
    write_cap_immutable(&objects.dir, &object_name, &object_path, input)?;
    sync_regular_file(
        &objects.dir,
        &object_name,
        &object_path,
        MAX_PORTABLE_INPUT_BYTES,
    )?;
    phase("object_durable", root)?;
    lock.require_current(&namespace)?;
    objects.require_current(&namespace)?;
    records.require_current(&namespace)?;
    write_cap_immutable(&records.dir, &name, &records.path.join(&name), marker)?;
    sync_regular_file(
        &records.dir,
        &name,
        &records.path.join(&name),
        MAX_PORTABLE_MARKER_BYTES,
    )?;
    phase("marker_durable", root)?;
    lock.require_current(&namespace)?;
    objects.require_current(&namespace)?;
    records.require_current(&namespace)?;
    // Persist every newly-created ancestor link before clearing uncertainty.
    let mut dir = namespace.root.directory()?;
    let mut path = root.to_path_buf();
    for component in &namespace.names {
        sync_directory(&dir, &path)?;
        path.push(component);
        dir = open_cap_directory_nofollow_strict(&dir, component, &path)?;
    }
    pending_reservation.require_current(&records.dir, &pending, &pending_path)?;
    let pending_current = read_regular(
        &records.dir,
        &pending,
        &pending_path,
        MAX_PORTABLE_MARKER_BYTES,
    )?;
    if pending_current.bytes != pending_reservation.bytes {
        return Err(error(&pending_path, "pending_binding_changed"));
    }
    pending_reservation.require_current(&records.dir, &pending, &pending_path)?;
    records.require_current(&namespace)?;
    lock.require_current(&namespace)?;
    records
        .dir
        .remove_file(&pending)
        .map_err(|source| io_error(&pending_path, source))?;
    if let Err(primary) = phase("pending_release_sync", root)
        .and_then(|()| sync_directory(&records.dir, &records.path))
    {
        // Best effort only: if the filesystem refuses all writes, durable
        // failure cannot be manufactured. Preserve both errors in that case.
        return match write_cap_immutable(&records.dir, &pending, &pending_path, marker).and_then(
            |_| {
                sync_regular_file(
                    &records.dir,
                    &pending,
                    &pending_path,
                    MAX_PORTABLE_MARKER_BYTES,
                )
            },
        ) {
            Ok(_) => Err(primary),
            Err(secondary) => Err(error(
                &pending_path,
                &format!("pending_restore_failed; primary={primary}; restore={secondary}"),
            )),
        };
    }
    lock.require_current(&namespace)?;
    read_committed(&namespace, &records, record_id, wi)?
        .ok_or_else(|| error(&records.path, "final_marker_missing"))
}

pub(super) fn load_portable_record(
    root: &Path,
    wi: &str,
    record_id: &str,
) -> Result<Option<PortableStoredRecord>, ObserverError> {
    check_ids(root, wi, record_id)?;
    let Some(namespace) = Namespace::open(root, wi, false)? else {
        return Ok(None);
    };
    let Some(records) = namespace.child("records", false)? else {
        namespace.require_current()?;
        return Ok(None);
    };
    let (name, pending) = names(record_id);
    if !exists(&records.dir, &name, &records.path.join(&name))?
        && !exists(&records.dir, &pending, &records.path.join(&pending))?
    {
        records.require_current(&namespace)?;
        return Ok(None);
    }
    let lock = StoreLock::acquire(&namespace, false, false)?;
    let result = read_committed(&namespace, &records, record_id, wi)?;
    lock.require_current(&namespace)?;
    Ok(result)
}

#[cfg(test)]
struct PhaseHook {
    name: &'static str,
    action: Box<dyn FnOnce() -> Result<(), ObserverError>>,
}
#[cfg(test)]
thread_local! {
    static PHASE_HOOK: std::cell::RefCell<Option<PhaseHook>> = const { std::cell::RefCell::new(None) };
}

fn phase(name: &str, root: &Path) -> Result<(), ObserverError> {
    #[cfg(test)]
    {
        let hook = PHASE_HOOK.with_borrow_mut(|slot| {
            if slot.as_ref().is_some_and(|hook| hook.name == name) {
                slot.take()
            } else {
                None
            }
        });
        if let Some(hook) = hook {
            return (hook.action)();
        }
    }
    let _ = (name, root);
    Ok(())
}

#[cfg(test)]
pub(super) fn with_portable_store_phase_hook<T>(
    name: &'static str,
    hook: impl FnOnce() -> Result<(), ObserverError> + 'static,
    action: impl FnOnce() -> T,
) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            PHASE_HOOK.with_borrow_mut(|slot| *slot = None);
        }
    }
    PHASE_HOOK.with_borrow_mut(|slot| {
        assert!(slot.is_none(), "portable phase hook already installed");
        *slot = Some(PhaseHook {
            name,
            action: Box::new(hook),
        });
    });
    let _reset = Reset;
    action()
}
