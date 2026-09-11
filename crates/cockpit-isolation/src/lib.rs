//! Single-process filesystem manifests for release-isolation checks.
//!
//! The manifest shape intentionally mirrors the JSONL emitted by the current
//! shell helper. The scanner does not follow symlinks while enumerating; it
//! records their literal target and resolves the target separately for the
//! caller's containment policy.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, Metadata};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use thiserror::Error;

/// The shell compatibility helper treats a target that needs more than forty
/// symlink hops as unresolved.
pub const MAX_SYMLINK_HOPS: u64 = 40;

/// One JSONL record for an entry below the scanned root.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManifestRecord {
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub mode: String,
    pub size: Option<String>,
    pub mtime: Option<String>,
    pub digest: Option<String>,
    pub target: Option<String>,
    #[serde(rename = "resolvedTarget")]
    pub resolved_target: Option<String>,
}

/// Counters describing work performed by one scan request.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ScanStats {
    pub entries: u64,
    pub directories: u64,
    pub file_entries: u64,
    pub special_entries: u64,
    pub symlink_entries: u64,
    pub metadata_reads: u64,
    pub hashed_files: u64,
    pub bytes_hashed: u64,
    pub symlink_hops: u64,
}

/// The complete result of a scan, including its sorted records and counters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanResult {
    pub records: Vec<ManifestRecord>,
    pub stats: ScanStats,
}

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("I/O error while {operation} {path}: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("path is not valid UTF-8 and cannot be represented in JSON: {path}")]
    NonUtf8Path { path: PathBuf },
    #[error("JSONL serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("JSONL output error: {0}")]
    Output(#[source] io::Error),
    #[error("scan counter overflow")]
    CounterOverflow,
    #[error("symlink target is unresolved: {path} -> {target}")]
    SymlinkUnresolved { path: String, target: String },
    #[error("symlink target escapes allowed root: {path} -> {resolved}")]
    SymlinkEscapesRoot { path: String, resolved: String },
}

/// Recursively scan every entry below `root` without following symlinks.
///
/// A missing or non-directory root is compatible with the shell helper and
/// produces an empty result. All returned records are sorted by UTF-8 path
/// bytes, matching deterministic `LC_ALL=C` ordering for representable paths.
pub fn scan_tree(root: impl AsRef<Path>) -> Result<ScanResult, ScanError> {
    let root = root.as_ref();
    match fs::metadata(root) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => return Ok(empty_result()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(empty_result()),
        Err(source) => return Err(io_error("reading root metadata", root, source)),
    }

    let mut state = ScanState::default();
    walk_directory(root, Path::new(""), &mut state)?;
    state
        .records
        .sort_unstable_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    Ok(ScanResult {
        records: state.records,
        stats: state.stats,
    })
}

/// Scan `root` and write one compact, newline-terminated JSON object per line.
pub fn scan_tree_to_jsonl<W: Write>(
    root: impl AsRef<Path>,
    mut output: W,
) -> Result<ScanStats, ScanError> {
    let result = scan_tree(root)?;
    for record in &result.records {
        let bytes = serde_json::to_vec(record)?;
        output.write_all(&bytes).map_err(ScanError::Output)?;
        output.write_all(b"\n").map_err(ScanError::Output)?;
    }
    Ok(result.stats)
}

/// Scan an explicit, already-selected set of relative paths. Git remains the
/// authority for selecting tracked/unignored source paths; Rust owns all
/// metadata, hashing, and deterministic JSONL work in one process.
pub fn scan_paths_to_jsonl<W: Write>(
    root: impl AsRef<Path>,
    paths: impl IntoIterator<Item = PathBuf>,
    output: W,
) -> Result<ScanStats, ScanError> {
    scan_paths_to_jsonl_with_mask(root, paths, output, None)
}

/// Variant of [`scan_paths_to_jsonl`] that preserves the shell helper's
/// source-checkout behavior: directory metadata on ancestors of an excluded
/// output directory is masked because creating the output changes those
/// directory mtimes without changing source facts.
pub fn scan_paths_to_jsonl_with_mask<W: Write>(
    root: impl AsRef<Path>,
    paths: impl IntoIterator<Item = PathBuf>,
    mut output: W,
    mask_ancestors_of: Option<&Path>,
) -> Result<ScanStats, ScanError> {
    let root = root.as_ref();
    match fs::metadata(root) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => return Ok(ScanStats::default()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Ok(ScanStats::default());
        }
        Err(source) => return Err(io_error("reading root metadata", root, source)),
    }
    let mut selected = paths.into_iter().collect::<Vec<_>>();
    selected.sort_unstable_by(|left, right| {
        left.to_string_lossy()
            .as_bytes()
            .cmp(right.to_string_lossy().as_bytes())
    });
    selected.dedup();
    let mut state = ScanState::default();
    for relative in selected {
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| component == std::path::Component::ParentDir)
        {
            return Err(ScanError::Io {
                operation: "validating source manifest path",
                path: relative,
                source: io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "path must be relative and remain below root",
                ),
            });
        }
        let path_string = relative
            .to_str()
            .ok_or_else(|| ScanError::NonUtf8Path {
                path: relative.clone(),
            })?
            .to_string();
        append_entry(&root.join(&relative), path_string, &mut state)?;
    }
    state
        .records
        .sort_unstable_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    for record in &state.records {
        let mut record = record.clone();
        if let Some(excluded) = mask_ancestors_of {
            let is_ancestor = excluded
                .parent()
                .is_some_and(|parent| parent != Path::new(""))
                && excluded.starts_with(Path::new(&record.path))
                && Path::new(&record.path) != excluded;
            if is_ancestor {
                record.size = None;
                record.mtime = None;
            }
        }
        output
            .write_all(&serde_json::to_vec(&record)?)
            .map_err(ScanError::Output)?;
        output.write_all(b"\n").map_err(ScanError::Output)?;
    }
    Ok(state.stats)
}

/// Enforce the release helper's policy that every symlink resolves inside the
/// supplied root. Scanning and policy validation are separate so callers can
/// preserve an unsafe record in the manifest for diagnosis.
pub fn validate_symlink_containment(
    root: impl AsRef<Path>,
    records: &[ManifestRecord],
) -> Result<(), ScanError> {
    let root = root.as_ref();
    let root_real = fs::canonicalize(root)
        .map_err(|source| io_error("canonicalizing containment root", root, source))?;
    for record in records
        .iter()
        .filter(|record| record.entry_type == "symlink")
    {
        let target = record.target.as_deref().unwrap_or("");
        let resolved =
            record
                .resolved_target
                .as_deref()
                .ok_or_else(|| ScanError::SymlinkUnresolved {
                    path: record.path.clone(),
                    target: target.to_string(),
                })?;
        let resolved_path = Path::new(resolved);
        if resolved_path != root_real && !resolved_path.starts_with(&root_real) {
            return Err(ScanError::SymlinkEscapesRoot {
                path: record.path.clone(),
                resolved: resolved.to_string(),
            });
        }
    }
    Ok(())
}

#[derive(Default)]
struct ScanState {
    records: Vec<ManifestRecord>,
    stats: ScanStats,
}

fn empty_result() -> ScanResult {
    ScanResult {
        records: Vec::new(),
        stats: ScanStats::default(),
    }
}

fn walk_directory(
    directory: &Path,
    relative_parent: &Path,
    state: &mut ScanState,
) -> Result<(), ScanError> {
    let mut entries = Vec::new();
    let read_dir = fs::read_dir(directory)
        .map_err(|source| io_error("enumerating directory", directory, source))?;
    for entry in read_dir {
        let entry =
            entry.map_err(|source| io_error("reading directory entry", directory, source))?;
        let path = entry.path();
        let relative = relative_parent.join(entry.file_name());
        let path_string = relative
            .to_str()
            .ok_or_else(|| ScanError::NonUtf8Path {
                path: relative.clone(),
            })?
            .to_string();
        entries.push((path_string, path));
    }
    entries.sort_unstable_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));

    for (path_string, path) in entries {
        let metadata = fs::symlink_metadata(&path)
            .map_err(|source| io_error("reading entry metadata", &path, source))?;
        increment(&mut state.stats.entries)?;
        increment(&mut state.stats.metadata_reads)?;

        let (entry_type, digest, target, resolved_target) = if metadata.file_type().is_symlink() {
            increment(&mut state.stats.symlink_entries)?;
            let target = fs::read_link(&path)
                .map_err(|source| io_error("reading symlink target", &path, source))?;
            let target = target
                .into_os_string()
                .into_string()
                .map_err(|_| ScanError::NonUtf8Path { path: path.clone() })?;
            let resolved_target = resolve_symlink_target(&path, &mut state.stats)?;
            ("symlink", None, Some(target), resolved_target)
        } else if metadata.is_dir() {
            increment(&mut state.stats.directories)?;
            ("directory", None, None, None)
        } else if metadata.is_file() {
            increment(&mut state.stats.file_entries)?;
            let digest = hash_file(&path, &mut state.stats)?;
            ("file", Some(digest), None, None)
        } else {
            increment(&mut state.stats.special_entries)?;
            ("other", None, None, None)
        };

        state.records.push(ManifestRecord {
            path: path_string,
            entry_type: entry_type.to_string(),
            mode: mode_string(&metadata),
            size: Some(metadata.len().to_string()),
            mtime: Some(mtime_seconds(&metadata, &path)?),
            digest,
            target,
            resolved_target,
        });

        if metadata.is_dir() {
            let relative = state
                .records
                .last()
                .expect("record was just pushed")
                .path
                .clone();
            walk_directory(&path, Path::new(&relative), state)?;
        }
    }
    Ok(())
}

fn append_entry(path: &Path, path_string: String, state: &mut ScanState) -> Result<(), ScanError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            increment(&mut state.stats.entries)?;
            state.records.push(ManifestRecord {
                path: path_string,
                entry_type: "missing".to_string(),
                mode: "0".to_string(),
                size: Some("0".to_string()),
                mtime: Some("0".to_string()),
                digest: None,
                target: None,
                resolved_target: None,
            });
            return Ok(());
        }
        Err(source) => return Err(io_error("reading entry metadata", path, source)),
    };
    increment(&mut state.stats.entries)?;
    increment(&mut state.stats.metadata_reads)?;
    let (entry_type, digest, target, resolved_target) = if metadata.file_type().is_symlink() {
        increment(&mut state.stats.symlink_entries)?;
        let target = fs::read_link(path)
            .map_err(|source| io_error("reading symlink target", path, source))?;
        let target = target
            .into_os_string()
            .into_string()
            .map_err(|_| ScanError::NonUtf8Path {
                path: path.to_path_buf(),
            })?;
        let resolved_target = resolve_symlink_target(path, &mut state.stats)?;
        ("symlink", None, Some(target), resolved_target)
    } else if metadata.is_dir() {
        increment(&mut state.stats.directories)?;
        ("directory", None, None, None)
    } else if metadata.is_file() {
        increment(&mut state.stats.file_entries)?;
        let digest = hash_file(path, &mut state.stats)?;
        ("file", Some(digest), None, None)
    } else {
        increment(&mut state.stats.special_entries)?;
        ("other", None, None, None)
    };
    state.records.push(ManifestRecord {
        path: path_string,
        entry_type: entry_type.to_string(),
        mode: mode_string(&metadata),
        size: Some(metadata.len().to_string()),
        mtime: Some(mtime_seconds(&metadata, path)?),
        digest,
        target,
        resolved_target,
    });
    Ok(())
}

fn hash_file(path: &Path, stats: &mut ScanStats) -> Result<String, ScanError> {
    let mut file =
        File::open(path).map_err(|source| io_error("opening file for hashing", path, source))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut bytes_hashed = 0_u64;
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|source| io_error("hashing file", path, source))?;
        if read == 0 {
            break;
        }
        bytes_hashed = bytes_hashed
            .checked_add(read as u64)
            .ok_or(ScanError::CounterOverflow)?;
        hasher.update(&buffer[..read]);
    }
    stats.bytes_hashed = stats
        .bytes_hashed
        .checked_add(bytes_hashed)
        .ok_or(ScanError::CounterOverflow)?;
    increment(&mut stats.hashed_files)?;
    Ok(format!("sha256:{}", hex::encode(hasher.finalize())))
}

fn resolve_symlink_target(path: &Path, stats: &mut ScanStats) -> Result<Option<String>, ScanError> {
    let mut current = path.to_path_buf();
    for _ in 0..MAX_SYMLINK_HOPS {
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) => return Err(io_error("resolving symlink target", &current, source)),
        };
        increment(&mut stats.metadata_reads)?;
        if !metadata.file_type().is_symlink() {
            let canonical = match fs::canonicalize(&current) {
                Ok(canonical) => canonical,
                Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
                Err(source) => {
                    return Err(io_error("canonicalizing symlink target", &current, source));
                }
            };
            return canonical
                .to_str()
                .map(str::to_string)
                .map(Some)
                .ok_or(ScanError::NonUtf8Path { path: canonical });
        }

        increment(&mut stats.symlink_hops)?;
        let target = fs::read_link(&current)
            .map_err(|source| io_error("reading symlink hop", &current, source))?;
        current = if target.is_absolute() {
            target
        } else {
            current
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .join(target)
        };
    }
    Ok(None)
}

fn mode_string(metadata: &Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        format!("{:o}", metadata.permissions().mode() & 0o7777)
    }
    #[cfg(not(unix))]
    {
        if metadata.permissions().readonly() {
            "444".to_string()
        } else {
            "666".to_string()
        }
    }
}

fn mtime_seconds(metadata: &Metadata, path: &Path) -> Result<String, ScanError> {
    let modified = metadata
        .modified()
        .map_err(|source| io_error("reading modification time", path, source))?;
    match modified.duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok(duration.as_secs().to_string()),
        Err(error) => Ok(format!("-{}", error.duration().as_secs())),
    }
}

fn increment(value: &mut u64) -> Result<(), ScanError> {
    *value = value.checked_add(1).ok_or(ScanError::CounterOverflow)?;
    Ok(())
}

fn io_error(operation: &'static str, path: &Path, source: io::Error) -> ScanError {
    ScanError::Io {
        operation,
        path: path.to_path_buf(),
        source,
    }
}
