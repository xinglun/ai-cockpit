use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiptStoreBinding {
    pub repository_id: String,
    pub profile_digest: String,
    pub node_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReceiptStoreLoad {
    Candidate {
        receipt: Box<cockpit_evidence::ReusableReceipt>,
        files_read: usize,
    },
    Unavailable {
        reason: String,
        files_read: usize,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReceiptStoreWrite {
    pub files_read: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReceiptStoreIndex {
    schema_version: u32,
    repository_id: String,
    profile_digest: String,
    receipts: BTreeMap<String, String>,
}

pub fn load_reusable_receipt(
    root: &Path,
    binding: &ReceiptStoreBinding,
) -> Result<ReceiptStoreLoad, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if binding.repository_id != repository_id(&root).to_string() {
        return Ok(unavailable_receipt("repository_identity_mismatch", 0));
    }
    if !valid_sha256_digest(&binding.profile_digest) || !valid_node_id(&binding.node_id) {
        return Ok(unavailable_receipt("store_binding_invalid", 0));
    }

    let root_dir =
        Dir::open_ambient_dir(&root, cap_std::ambient_authority()).map_err(|source| {
            ObserverError::Read {
                path: root.clone(),
                source,
            }
        })?;
    let ai = match open_cap_directory_nofollow(&root_dir, ".ai", &root.join(".ai")) {
        Ok(directory) => directory,
        Err(reason) => return Ok(unavailable_receipt(reason, 0)),
    };
    let evidence = match open_cap_directory_nofollow(&ai, "evidence", &root.join(".ai/evidence")) {
        Ok(directory) => directory,
        Err(reason) => return Ok(unavailable_receipt(reason, 0)),
    };
    let reuse_path = root.join(".ai/evidence/reuse");
    let reuse = match open_cap_directory_nofollow(&evidence, "reuse", &reuse_path) {
        Ok(directory) => directory,
        Err(reason) => return Ok(unavailable_receipt(reason, 0)),
    };
    let lock_path = reuse_path.join("index.lock");
    let lock = match open_cap_existing_nofollow(&reuse, "index.lock", &lock_path) {
        Ok(lock) => lock,
        Err(_) => return Ok(unavailable_receipt("index_invalid", 0)),
    };
    if lock.lock_shared().is_err() {
        return Ok(unavailable_receipt("index_unreadable", 0));
    }
    #[cfg(windows)]
    match read_optional_cap_file_nofollow_bounded(
        &reuse,
        "index.pending",
        &reuse_path.join("index.pending"),
        1024,
    ) {
        Ok(Some(_)) => return Ok(unavailable_receipt("index_commit_uncertain", 0)),
        Ok(None) => {}
        Err(ObserverError::State { message, .. }) if message.contains("symlink") => {
            return Ok(unavailable_receipt("symlink_rejected", 0));
        }
        Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
    }
    #[cfg(not(windows))]
    match reuse.symlink_metadata("index.pending") {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            return Ok(unavailable_receipt("index_commit_uncertain", 0));
        }
        Ok(_) => return Ok(unavailable_receipt("index_invalid", 0)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
    }
    let index_path = reuse_path.join("index.json");
    #[cfg(windows)]
    let index_bytes = match read_optional_cap_file_nofollow_bounded(
        &reuse,
        "index.json",
        &index_path,
        MAX_RECEIPT_INDEX_BYTES,
    ) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(unavailable_receipt("evidence_missing", 0)),
        Err(ObserverError::State { message, .. }) if message.contains("symlink") => {
            return Ok(unavailable_receipt("symlink_rejected", 0));
        }
        Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
    };
    #[cfg(not(windows))]
    let index_bytes = {
        let index_metadata = match reuse.symlink_metadata("index.json") {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(unavailable_receipt("evidence_missing", 0));
            }
            Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
        };
        if index_metadata.file_type().is_symlink() {
            return Ok(unavailable_receipt("symlink_rejected", 0));
        }
        if !index_metadata.is_file() {
            return Ok(unavailable_receipt("index_invalid", 0));
        }
        match read_cap_file_nofollow_bounded(
            &reuse,
            "index.json",
            &index_path,
            MAX_RECEIPT_INDEX_BYTES,
        ) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
        }
    };
    let index: ReceiptStoreIndex = match serde_json::from_slice(&index_bytes) {
        Ok(index) => index,
        Err(_) => return Ok(unavailable_receipt("index_invalid", 1)),
    };
    if index.schema_version != 1 {
        return Ok(unavailable_receipt("index_invalid", 1));
    }
    if index.repository_id != binding.repository_id {
        return Ok(unavailable_receipt("repository_identity_mismatch", 1));
    }
    if index.profile_digest != binding.profile_digest {
        return Ok(unavailable_receipt("profile_identity_mismatch", 1));
    }
    let Some(receipt_id) = index.receipts.get(&binding.node_id) else {
        return Ok(unavailable_receipt("evidence_missing", 1));
    };
    if !valid_sha256_digest(receipt_id) {
        return Ok(unavailable_receipt("index_invalid", 1));
    }

    let receipts_path = reuse_path.join("receipts");
    let receipts = match open_cap_directory_nofollow(&reuse, "receipts", &receipts_path) {
        Ok(directory) => directory,
        Err(reason) => return Ok(unavailable_receipt(reason, 1)),
    };
    let receipt_name = receipt_file_name(receipt_id);
    let receipt_path = receipts_path.join(&receipt_name);
    #[cfg(windows)]
    let receipt_bytes = match read_optional_cap_file_nofollow_bounded(
        &receipts,
        &receipt_name,
        &receipt_path,
        MAX_REUSABLE_RECEIPT_BYTES,
    ) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(unavailable_receipt("receipt_missing", 1)),
        Err(ObserverError::State { message, .. }) if message.contains("symlink") => {
            return Ok(unavailable_receipt("symlink_rejected", 1));
        }
        Err(_) => return Ok(unavailable_receipt("receipt_unreadable", 1)),
    };
    #[cfg(not(windows))]
    let receipt_bytes = {
        let receipt_metadata = match receipts.symlink_metadata(&receipt_name) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(unavailable_receipt("receipt_missing", 1));
            }
            Err(_) => return Ok(unavailable_receipt("receipt_unreadable", 1)),
        };
        if receipt_metadata.file_type().is_symlink() {
            return Ok(unavailable_receipt("symlink_rejected", 1));
        }
        if !receipt_metadata.is_file() {
            return Ok(unavailable_receipt("receipt_invalid", 1));
        }
        match read_cap_file_nofollow_bounded(
            &receipts,
            &receipt_name,
            &receipt_path,
            MAX_REUSABLE_RECEIPT_BYTES,
        ) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(unavailable_receipt("receipt_unreadable", 1)),
        }
    };
    let receipt: cockpit_evidence::ReusableReceipt = match serde_json::from_slice(&receipt_bytes) {
        Ok(receipt) => receipt,
        Err(_) => return Ok(unavailable_receipt("receipt_invalid", 2)),
    };
    if receipt.validate().is_err()
        || receipt.receipt_id != *receipt_id
        || receipt.node_id != binding.node_id
        || receipt.context.profile_digest != binding.profile_digest
    {
        return Ok(unavailable_receipt("receipt_invalid", 2));
    }
    Ok(ReceiptStoreLoad::Candidate {
        receipt: Box::new(receipt),
        files_read: 2,
    })
}

pub fn persist_reusable_receipt(
    root: &Path,
    binding: &ReceiptStoreBinding,
    receipt: &cockpit_evidence::ReusableReceipt,
) -> Result<ReceiptStoreWrite, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let expected_repository_id = repository_id(&root).to_string();
    if binding.repository_id != expected_repository_id
        || !valid_sha256_digest(&binding.profile_digest)
        || !valid_node_id(&binding.node_id)
        || receipt.validate().is_err()
        || !receipt.passed
        || receipt.node_id != binding.node_id
        || receipt.context.profile_digest != binding.profile_digest
    {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence/reuse"),
            message: "invalid reusable receipt store binding".into(),
        });
    }

    let root_dir =
        Dir::open_ambient_dir(&root, cap_std::ambient_authority()).map_err(|source| {
            ObserverError::Read {
                path: root.clone(),
                source,
            }
        })?;
    let ai_path = root.join(".ai");
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &ai_path)?;
    let evidence_path = ai_path.join("evidence");
    let evidence = create_and_open_cap_directory(&ai, "evidence", &evidence_path)?;
    let reuse_path = evidence_path.join("reuse");
    let reuse = create_and_open_cap_directory(&evidence, "reuse", &reuse_path)?;
    let receipts_path = reuse_path.join("receipts");
    let receipts = create_and_open_cap_directory(&reuse, "receipts", &receipts_path)?;

    let lock_path = reuse_path.join("index.lock");
    let lock = open_or_create_cap_nofollow(&reuse, "index.lock", &lock_path)?;
    lock.lock().map_err(|source| ObserverError::Read {
        path: lock_path,
        source,
    })?;

    let receipt_name = receipt_file_name(&receipt.receipt_id);
    let receipt_path = receipts_path.join(&receipt_name);
    let receipt_bytes =
        serde_json::to_vec_pretty(receipt).map_err(|error| ObserverError::State {
            path: receipt_path.clone(),
            message: error.to_string(),
        })?;
    let mut files_read =
        write_cap_immutable(&receipts, &receipt_name, &receipt_path, &receipt_bytes)?;

    let index_path = reuse_path.join("index.json");
    let mut index = if let Some(bytes) = read_optional_cap_file_nofollow_bounded(
        &reuse,
        "index.json",
        &index_path,
        MAX_RECEIPT_INDEX_BYTES,
    )? {
        files_read += 1;
        let existing: ReceiptStoreIndex =
            serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
                path: index_path.clone(),
                message: format!("invalid reusable receipt index: {error}"),
            })?;
        if existing.schema_version != 1 || existing.repository_id != binding.repository_id {
            return Err(ObserverError::State {
                path: index_path,
                message: "reusable receipt index binding mismatch".into(),
            });
        }
        if existing.profile_digest == binding.profile_digest {
            existing
        } else {
            ReceiptStoreIndex {
                schema_version: 1,
                repository_id: binding.repository_id.clone(),
                profile_digest: binding.profile_digest.clone(),
                receipts: BTreeMap::new(),
            }
        }
    } else {
        ReceiptStoreIndex {
            schema_version: 1,
            repository_id: binding.repository_id.clone(),
            profile_digest: binding.profile_digest.clone(),
            receipts: BTreeMap::new(),
        }
    };
    index
        .receipts
        .insert(binding.node_id.clone(), receipt.receipt_id.clone());
    let index_bytes = serde_json::to_vec_pretty(&index).map_err(|error| ObserverError::State {
        path: index_path.clone(),
        message: error.to_string(),
    })?;
    write_cap_commit_marker(&reuse, &reuse_path)?;
    atomic_replace_cap_strict(&reuse, "index.json", &index_path, &index_bytes)?;
    reuse
        .remove_file("index.pending")
        .map_err(|source| ObserverError::Read {
            path: reuse_path.join("index.pending"),
            source,
        })?;
    // The index rename is the logical commit. If syncing marker removal fails,
    // a crash can only resurrect the marker, which makes future reads fail closed.
    let _ = sync_cap_directory(&reuse, &reuse_path.join("index.pending"));
    Ok(ReceiptStoreWrite { files_read })
}

fn unavailable_receipt(reason: &str, files_read: usize) -> ReceiptStoreLoad {
    ReceiptStoreLoad::Unavailable {
        reason: reason.into(),
        files_read,
    }
}

pub(super) fn valid_sha256_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn receipt_file_name(receipt_id: &str) -> String {
    format!("{}.json", &receipt_id["sha256:".len()..])
}

fn valid_node_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn open_cap_directory_nofollow(
    parent: &Dir,
    name: &str,
    _display_path: &Path,
) -> Result<Dir, &'static str> {
    #[cfg(windows)]
    {
        // cap-std's Windows `symlink_metadata` implementation asks
        // `CreateFileAtW` for a zero-access handle.  On the hosted Windows
        // runner that relative metadata probe returns `ERROR_ACCESS_DENIED`
        // even for an ordinary child directory.  Open the directory handle
        // directly instead, then inspect the handle metadata; this also
        // avoids a metadata/open TOCTOU window.
        let directory = match parent.open_dir_nofollow(name) {
            Ok(directory) => directory,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err("evidence_missing");
            }
            Err(_) => return Err("store_path_unreadable"),
        };
        let metadata = directory
            .dir_metadata()
            .map_err(|_| "store_path_unreadable")?;
        if metadata.file_type().is_symlink() {
            return Err("symlink_rejected");
        }
        if !metadata.is_dir() {
            return Err("store_path_invalid");
        }
        return Ok(directory);
    }

    #[cfg(not(windows))]
    match parent.symlink_metadata(name) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err("symlink_rejected"),
        Ok(metadata) if !metadata.is_dir() => Err("store_path_invalid"),
        Ok(_) => parent
            .open_dir_nofollow(name)
            .map_err(|_| "store_path_unreadable"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err("evidence_missing"),
        Err(_) => Err("store_path_unreadable"),
    }
}

pub(super) fn open_cap_directory_nofollow_strict(
    parent: &Dir,
    name: &str,
    display_path: &Path,
) -> Result<Dir, ObserverError> {
    open_cap_directory_nofollow(parent, name, display_path).map_err(|reason| ObserverError::State {
        path: display_path.to_path_buf(),
        message: format!("receipt store directory rejected: {reason}"),
    })
}

pub(super) fn create_and_open_cap_directory(
    parent: &Dir,
    name: &str,
    display_path: &Path,
) -> Result<Dir, ObserverError> {
    match parent.create_dir(name) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(source) => {
            return Err(ObserverError::Read {
                path: display_path.to_path_buf(),
                source,
            });
        }
    }
    open_cap_directory_nofollow_strict(parent, name, display_path)
}

fn cap_read_options() -> CapOpenOptions {
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    options
}

fn open_cap_existing_nofollow(
    parent: &Dir,
    name: &str,
    display_path: &Path,
) -> Result<fs::File, ObserverError> {
    let file = parent
        .open_with(name, &cap_read_options())
        .map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?
        .into_std();
    if !file
        .metadata()
        .map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?
        .is_file()
    {
        return Err(ObserverError::State {
            path: display_path.to_path_buf(),
            message: "receipt store entry must be a real file".into(),
        });
    }
    Ok(file)
}

pub(super) fn open_or_create_cap_nofollow(
    parent: &Dir,
    name: &str,
    display_path: &Path,
) -> Result<fs::File, ObserverError> {
    let mut existing = CapOpenOptions::new();
    existing.read(true).write(true).follow(FollowSymlinks::No);
    let file = match parent.open_with(name, &existing) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut create = CapOpenOptions::new();
            create
                .read(true)
                .write(true)
                .create_new(true)
                .follow(FollowSymlinks::No);
            match parent.open_with(name, &create) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => parent
                    .open_with(name, &existing)
                    .map_err(|source| ObserverError::Read {
                        path: display_path.to_path_buf(),
                        source,
                    })?,
                Err(source) => {
                    return Err(ObserverError::Read {
                        path: display_path.to_path_buf(),
                        source,
                    });
                }
            }
        }
        Err(source) => {
            return Err(ObserverError::Read {
                path: display_path.to_path_buf(),
                source,
            });
        }
    }
    .into_std();
    if !file
        .metadata()
        .map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?
        .is_file()
    {
        return Err(ObserverError::State {
            path: display_path.to_path_buf(),
            message: "receipt store lock must be a real file".into(),
        });
    }
    Ok(file)
}

pub(super) fn read_cap_file_nofollow_bounded(
    parent: &Dir,
    name: &str,
    display_path: &Path,
    maximum_bytes: u64,
) -> Result<Vec<u8>, ObserverError> {
    let mut file = open_cap_existing_nofollow(parent, name, display_path)?;
    let mut bytes = Vec::new();
    std::io::Read::by_ref(&mut file)
        .take(maximum_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?;
    if bytes.len() as u64 > maximum_bytes {
        return Err(ObserverError::State {
            path: display_path.to_path_buf(),
            message: "receipt store entry exceeds the bounded read limit".into(),
        });
    }
    Ok(bytes)
}

fn read_optional_cap_file_nofollow_bounded(
    parent: &Dir,
    name: &str,
    display_path: &Path,
    maximum_bytes: u64,
) -> Result<Option<Vec<u8>>, ObserverError> {
    #[cfg(windows)]
    {
        // See `open_cap_directory_nofollow`: probing a child with
        // `symlink_metadata` is not usable with the Windows capability
        // handle implementation on the hosted runner.  Open once with
        // no-follow semantics and read from that pinned handle.
        //
        // cap-std currently reports `ERROR_ACCESS_DENIED` for a missing leaf
        // when it performs that relative open.  Use the canonical display
        // path only to distinguish the absent case; any existing entry is
        // still opened and read through the capability handle below, so the
        // ambient probe cannot authorize a substituted file.
        match fs::symlink_metadata(display_path) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(ObserverError::Read {
                    path: display_path.to_path_buf(),
                    source,
                });
            }
        }
        let mut file = match parent.open_with(name, &cap_read_options()) {
            Ok(file) => file.into_std(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(ObserverError::Read {
                    path: display_path.to_path_buf(),
                    source,
                });
            }
        };
        let metadata = file.metadata().map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?;
        if metadata.file_type().is_symlink() {
            return Err(ObserverError::State {
                path: display_path.to_path_buf(),
                message: "receipt store entry must not be a symlink".into(),
            });
        }
        if !metadata.is_file() {
            return Err(ObserverError::State {
                path: display_path.to_path_buf(),
                message: "receipt store entry must be a real file".into(),
            });
        }
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(maximum_bytes.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|source| ObserverError::Read {
                path: display_path.to_path_buf(),
                source,
            })?;
        if bytes.len() as u64 > maximum_bytes {
            return Err(ObserverError::State {
                path: display_path.to_path_buf(),
                message: "receipt store entry exceeds the bounded read limit".into(),
            });
        }
        return Ok(Some(bytes));
    }

    #[cfg(not(windows))]
    match parent.symlink_metadata(name) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(ObserverError::State {
                path: display_path.to_path_buf(),
                message: "receipt store entry must be a real file".into(),
            })
        }
        Ok(_) => {
            read_cap_file_nofollow_bounded(parent, name, display_path, maximum_bytes).map(Some)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        }),
    }
}

fn write_cap_immutable(
    parent: &Dir,
    name: &str,
    display_path: &Path,
    bytes: &[u8],
) -> Result<usize, ObserverError> {
    let sequence = NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let temporary = format!("{name}.receipt-tmp-{}-{sequence}", std::process::id());
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = parent
        .open_with(&temporary, &options)
        .map_err(|source| ObserverError::Read {
            path: display_path.with_file_name(&temporary),
            source,
        })?
        .into_std();
    if let Err(source) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = parent.remove_file(&temporary);
        return Err(ObserverError::Read {
            path: display_path.with_file_name(&temporary),
            source,
        });
    }
    drop(file);
    let installed = match parent.hard_link(&temporary, parent, name) {
        Ok(()) => Ok(0),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if read_cap_file_nofollow_bounded(parent, name, display_path, bytes.len() as u64)?
                == bytes
            {
                Ok(1)
            } else {
                Err(ObserverError::State {
                    path: display_path.to_path_buf(),
                    message: "immutable receipt already exists with different content".into(),
                })
            }
        }
        Err(source) => Err(ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        }),
    };
    let _ = parent.remove_file(&temporary);
    let files_read = installed?;
    sync_cap_directory(parent, display_path)?;
    Ok(files_read)
}

fn write_cap_commit_marker(parent: &Dir, display_path: &Path) -> Result<(), ObserverError> {
    let marker_path = display_path.join("index.pending");
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut marker = parent
        .open_with("index.pending", &options)
        .map_err(|source| ObserverError::Read {
            path: marker_path.clone(),
            source,
        })?
        .into_std();
    marker
        .write_all(b"pending-index-commit-v1\n")
        .and_then(|()| marker.sync_all())
        .map_err(|source| ObserverError::Read {
            path: marker_path.clone(),
            source,
        })?;
    sync_cap_directory(parent, &marker_path)
}

fn atomic_replace_cap_strict(
    parent: &Dir,
    name: &str,
    display_path: &Path,
    bytes: &[u8],
) -> Result<(), ObserverError> {
    if let Ok(metadata) = parent.symlink_metadata(name)
        && (metadata.file_type().is_symlink() || !metadata.is_file())
    {
        return Err(ObserverError::State {
            path: display_path.to_path_buf(),
            message: "receipt store entry must be a real file".into(),
        });
    }
    let sequence = NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let temporary = format!("{name}.tmp-{}-{sequence}", std::process::id());
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    #[cfg(windows)]
    {
        use cap_std::fs::OpenOptionsExt;
        use windows_sys::Win32::{
            Foundation::{GENERIC_READ, GENERIC_WRITE},
            Storage::FileSystem::DELETE,
        };
        // FileRenameInfoEx requires DELETE access on the source handle.  A
        // write-only temporary file is sufficient on Unix but is rejected by
        // Windows with ERROR_ACCESS_DENIED during the first index publish.
        options.access_mode(GENERIC_READ | GENERIC_WRITE | DELETE);
    }
    let mut file = parent
        .open_with(&temporary, &options)
        .map_err(|source| ObserverError::Read {
            path: display_path.with_file_name(&temporary),
            source,
        })?
        .into_std();
    if let Err(source) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = parent.remove_file(&temporary);
        return Err(ObserverError::Read {
            path: display_path.with_file_name(&temporary),
            source,
        });
    }
    let replace_result = replace_cap_entry(parent, &file, &temporary, name, display_path);
    drop(file);
    if let Err(source) = replace_result {
        let _ = parent.remove_file(&temporary);
        return Err(ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        });
    }
    sync_cap_directory(parent, display_path)?;
    Ok(())
}

fn sync_cap_directory(parent: &Dir, display_path: &Path) -> Result<(), ObserverError> {
    #[cfg(windows)]
    {
        let _ = (parent, display_path);
        return Ok(());
    }
    #[cfg(not(windows))]
    #[cfg(not(windows))]
    {
        // `cap-std` intentionally opens read-only directories with `O_PATH`
        // on Linux. `fsync(O_PATH)` returns EBADF, so reopen `.` relative to
        // the already-pinned directory capability with ordinary read access
        // before syncing. This remains handle-relative and avoids an ambient
        // path race.
        let mut options = CapOpenOptions::new();
        options.read(true);
        parent
            .open_with(".", &options)
            .map(|directory| directory.into_std().sync_all())
            .and_then(|result| result)
            .map_err(|source| ObserverError::Read {
                path: display_path.parent().unwrap_or(display_path).to_path_buf(),
                source,
            })
    }
}

#[cfg(not(windows))]
fn replace_cap_entry(
    parent: &Dir,
    _temporary_file: &std::fs::File,
    temporary: &str,
    name: &str,
    _display_path: &Path,
) -> std::io::Result<()> {
    parent.rename(temporary, parent, name)
}

#[cfg(windows)]
fn replace_cap_entry(
    _parent: &Dir,
    temporary_file: &std::fs::File,
    _temporary: &str,
    _name: &str,
    display_path: &Path,
) -> std::io::Result<()> {
    use std::os::windows::{ffi::OsStrExt, io::AsRawHandle};
    use windows_sys::Win32::{
        Storage::FileSystem::{
            FILE_RENAME_INFO, FILE_RENAME_INFO_0, FileRenameInfoEx, SetFileInformationByHandle,
        },
        System::WindowsProgramming::{
            FILE_RENAME_FLAG_POSIX_SEMANTICS, FILE_RENAME_FLAG_REPLACE_IF_EXISTS,
        },
    };

    // FileRenameInfoEx rejects a relative name paired with RootDirectory on
    // the Windows runners (ERROR_INVALID_PARAMETER). The parent capability
    // remains open with delete sharing disabled, so its absolute path cannot
    // be redirected while this operation is in flight. Keeping the Ex class
    // preserves replace/POSIX semantics for an existing target.
    let display_path = display_path.to_string_lossy();
    // cap-std canonical paths on Windows use the extended-length `\\?\\`
    // prefix. FileRenameInfoEx accepts the ordinary DOS spelling for these
    // short local paths, while treating the extended spelling as an invalid
    // rename target on some runner images.
    let display_path = display_path.strip_prefix(r"\\?\").unwrap_or(&display_path);
    let name = std::ffi::OsStr::new(display_path)
        .encode_wide()
        .collect::<Vec<_>>();
    let header_size = std::mem::offset_of!(FILE_RENAME_INFO, FileName);
    // `FILE_RENAME_INFO` declares `FileName[1]`.  Even though
    // `FileNameLength` excludes a terminator, the buffer passed to
    // `SetFileInformationByHandle` must still include that inline slot.
    let byte_len = header_size + (name.len() + 1) * std::mem::size_of::<u16>();
    let word_len = byte_len.div_ceil(std::mem::size_of::<usize>());
    let mut storage = vec![0_usize; word_len];
    let information = storage.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    unsafe {
        (*information).Anonymous = FILE_RENAME_INFO_0 {
            Flags: FILE_RENAME_FLAG_REPLACE_IF_EXISTS | FILE_RENAME_FLAG_POSIX_SEMANTICS,
        };
        (*information).RootDirectory = std::ptr::null_mut();
        (*information).FileNameLength = (name.len() * std::mem::size_of::<u16>()) as u32;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            (*information).FileName.as_mut_ptr(),
            name.len(),
        );
    }
    let result = unsafe {
        SetFileInformationByHandle(
            temporary_file.as_raw_handle().cast(),
            FileRenameInfoEx,
            information.cast(),
            byte_len as u32,
        )
    };
    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
