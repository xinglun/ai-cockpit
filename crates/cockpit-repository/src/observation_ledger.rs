use cockpit_core::Digest;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ObserverError;

#[derive(Clone, Debug, PartialEq, Eq)]
struct PathFingerprint {
    kind: &'static str,
    digest: Option<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CandidateFingerprint {
    kind: &'static str,
    digest: Option<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileDependency {
    label: String,
    path: PathBuf,
    fingerprint: PathFingerprint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CandidateSetDependency {
    label: String,
    path: PathBuf,
    matcher: CandidateMatcher,
    members: BTreeMap<String, CandidateFingerprint>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ObservationLedgerCost {
    pub(crate) fingerprint_reads: usize,
    pub(crate) candidate_directory_reads: usize,
    pub(crate) candidate_member_fingerprint_reads: usize,
    pub(crate) content_reads: usize,
    pub(crate) content_hashes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CachedPathObservation {
    fingerprint: PathFingerprint,
    content: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CachedDirectory {
    Missing,
    Invalid(&'static str),
    Entries(Vec<(String, PathBuf)>),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ObservationReadCache {
    paths: BTreeMap<PathBuf, CachedPathObservation>,
    directories: BTreeMap<PathBuf, CachedDirectory>,
    cost: ObservationLedgerCost,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RecheckResult {
    pub(crate) digest: Digest,
    pub(crate) cost: ObservationLedgerCost,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CandidateMatcher {
    PrefixSuffix {
        prefix: Option<String>,
        suffix: Option<String>,
    },
}

impl CandidateMatcher {
    fn matches(&self, name: &str) -> bool {
        match self {
            Self::PrefixSuffix { prefix, suffix } => {
                prefix
                    .as_deref()
                    .is_none_or(|prefix| name.starts_with(prefix))
                    && suffix
                        .as_deref()
                        .is_none_or(|suffix| name.ends_with(suffix))
            }
        }
    }
}

/// Request-local, content-bound observations used by Outcome assembly.
///
/// The ledger deliberately records content digests and type changes rather
/// than mtime/size signals. Candidate sets are retained separately from file
/// dependencies so a new or removed decision cannot be hidden by reusing the
/// files that were already read.
#[derive(Clone, Debug, Default)]
pub(crate) struct ObservationLedger {
    root: PathBuf,
    files: BTreeMap<PathBuf, FileDependency>,
    candidate_sets: Vec<CandidateSetDependency>,
    cache: ObservationReadCache,
}

impl ObservationLedger {
    pub(crate) fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            files: BTreeMap::new(),
            candidate_sets: Vec::new(),
            cache: ObservationReadCache::default(),
        }
    }

    pub(crate) fn register_file(
        &mut self,
        label: impl Into<String>,
        path: &Path,
    ) -> Result<(), ObserverError> {
        let key = self.relative_key(path);
        let fingerprint = self.cached_path_observation(path)?.fingerprint.clone();
        self.files.insert(
            key.clone(),
            FileDependency {
                label: label.into(),
                path: path.to_path_buf(),
                fingerprint,
            },
        );
        Ok(())
    }

    pub(crate) fn register_candidate_set(
        &mut self,
        label: impl Into<String>,
        directory: &Path,
        matcher: CandidateMatcher,
    ) -> Result<(), ObserverError> {
        let members = self.candidate_fingerprints(directory, &matcher)?;
        self.candidate_sets.push(CandidateSetDependency {
            label: label.into(),
            path: directory.to_path_buf(),
            matcher,
            members,
        });
        Ok(())
    }

    pub(crate) fn digest(&self, external_facts: &[u8]) -> Digest {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.facts_digest().to_string().as_bytes());
        bytes.extend_from_slice(b"external\0");
        bytes.extend_from_slice(external_facts);
        Digest::sha256_bytes(&bytes)
    }

    pub(crate) fn facts_digest(&self) -> Digest {
        let mut bytes = Vec::new();
        append_fingerprint_bytes(&mut bytes, &self.files, &self.candidate_sets);
        Digest::sha256_bytes(&bytes)
    }

    #[cfg(test)]
    pub(crate) fn read_counters(&self) -> ObservationLedgerCost {
        self.cache.cost
    }

    pub(crate) fn cached_file_bytes(&self, path: &Path) -> Result<Option<&[u8]>, ObserverError> {
        let observation = self
            .cache
            .paths
            .get(path)
            .ok_or_else(|| ObserverError::State {
                path: path.to_path_buf(),
                message: "observation dependency was not registered before cached read".into(),
            })?;
        Ok(observation.content.as_deref())
    }

    pub(crate) fn cached_directory_entries(
        &self,
        directory: &Path,
    ) -> Result<Vec<(String, PathBuf)>, ObserverError> {
        match self.cache.directories.get(directory) {
            Some(CachedDirectory::Entries(entries)) => Ok(entries.clone()),
            Some(CachedDirectory::Missing | CachedDirectory::Invalid(_)) => Ok(Vec::new()),
            None => Err(ObserverError::State {
                path: directory.to_path_buf(),
                message: "observation candidate directory was not registered before cached read"
                    .into(),
            }),
        }
    }

    pub(crate) fn recheck_digest(
        &mut self,
        external_facts: &[u8],
    ) -> Result<Digest, ObserverError> {
        Ok(self.recheck_with_cost(external_facts)?.digest)
    }

    pub(crate) fn recheck_with_cost(
        &mut self,
        external_facts: &[u8],
    ) -> Result<RecheckResult, ObserverError> {
        // A final boundary must observe the filesystem again.  Replacing the
        // request-local cache here makes the invalidation explicit while still
        // deduplicating paths that are shared by fixed files and candidates.
        self.cache = ObservationReadCache::default();
        let mut current_files = BTreeMap::new();
        for dependency in self.files.values().cloned().collect::<Vec<_>>() {
            let current = self
                .cached_path_observation(&dependency.path)?
                .fingerprint
                .clone();
            if current != dependency.fingerprint {
                return Err(drift_error(
                    &dependency.label,
                    &dependency.path,
                    "file content or type changed",
                ));
            }
            current_files.insert(
                self.relative_key(&dependency.path),
                FileDependency {
                    label: dependency.label.clone(),
                    path: dependency.path.clone(),
                    fingerprint: current,
                },
            );
        }
        let mut current_candidate_sets = Vec::with_capacity(self.candidate_sets.len());
        for dependency in self.candidate_sets.clone() {
            let current = self.candidate_fingerprints(&dependency.path, &dependency.matcher)?;
            if current != dependency.members {
                return Err(drift_error(
                    &dependency.label,
                    &dependency.path,
                    "candidate set or member content changed",
                ));
            }
            current_candidate_sets.push(CandidateSetDependency {
                label: dependency.label.clone(),
                path: dependency.path.clone(),
                matcher: dependency.matcher.clone(),
                members: current,
            });
        }
        self.files = current_files;
        self.candidate_sets = current_candidate_sets;
        Ok(RecheckResult {
            digest: self.digest(external_facts),
            cost: self.cache.cost,
        })
    }

    fn cached_path_observation(
        &mut self,
        path: &Path,
    ) -> Result<&CachedPathObservation, ObserverError> {
        let key = path.to_path_buf();
        if !self.cache.paths.contains_key(&key) {
            let observation = fingerprint_path(path, &mut self.cache.cost)?;
            self.cache.paths.insert(key.clone(), observation);
        }
        self.cache
            .paths
            .get(&key)
            .ok_or_else(|| ObserverError::State {
                path: path.to_path_buf(),
                message: "observation path cache was not initialized".into(),
            })
    }

    fn candidate_fingerprints(
        &mut self,
        directory: &Path,
        matcher: &CandidateMatcher,
    ) -> Result<BTreeMap<String, CandidateFingerprint>, ObserverError> {
        self.cached_directory_entries_for_scan(directory)?;
        let directory_state = self
            .cache
            .directories
            .get(directory)
            .cloned()
            .ok_or_else(|| ObserverError::State {
                path: directory.to_path_buf(),
                message: "candidate directory cache was not initialized".into(),
            })?;
        let entries = match directory_state {
            CachedDirectory::Missing => {
                return Ok(BTreeMap::from([(
                    "<directory-missing>".into(),
                    CandidateFingerprint {
                        kind: "missing",
                        digest: None,
                    },
                )]));
            }
            CachedDirectory::Invalid(kind) => {
                return Ok(BTreeMap::from([(
                    "<directory-invalid>".into(),
                    CandidateFingerprint { kind, digest: None },
                )]));
            }
            CachedDirectory::Entries(entries) => entries,
        };
        let mut members = BTreeMap::new();
        for (name, path) in entries {
            if !matcher.matches(&name) {
                continue;
            }
            self.cache.cost.candidate_member_fingerprint_reads +=
                usize::from(!self.cache.paths.contains_key(&path));
            let fingerprint = self.cached_path_observation(&path)?.fingerprint.clone();
            members.insert(
                name,
                CandidateFingerprint {
                    kind: fingerprint.kind,
                    digest: fingerprint.digest,
                },
            );
        }
        Ok(members)
    }

    fn cached_directory_entries_for_scan(
        &mut self,
        directory: &Path,
    ) -> Result<Vec<(String, PathBuf)>, ObserverError> {
        if let Some(cached) = self.cache.directories.get(directory) {
            return Ok(match cached {
                CachedDirectory::Entries(entries) => entries.clone(),
                CachedDirectory::Missing | CachedDirectory::Invalid(_) => Vec::new(),
            });
        }
        let metadata = match fs::symlink_metadata(directory) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.cache
                    .directories
                    .insert(directory.to_path_buf(), CachedDirectory::Missing);
                return Ok(Vec::new());
            }
            Err(source) => {
                return Err(ObserverError::Read {
                    path: directory.to_path_buf(),
                    source,
                });
            }
        };
        if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
            let kind = if metadata.file_type().is_symlink() {
                "symlink"
            } else {
                "other"
            };
            self.cache
                .directories
                .insert(directory.to_path_buf(), CachedDirectory::Invalid(kind));
            return Ok(Vec::new());
        }
        self.cache.cost.candidate_directory_reads += 1;
        let mut entries = Vec::new();
        for entry in fs::read_dir(directory).map_err(|source| ObserverError::Read {
            path: directory.to_path_buf(),
            source,
        })? {
            let entry = entry.map_err(|source| ObserverError::Read {
                path: directory.to_path_buf(),
                source,
            })?;
            entries.push((
                entry.file_name().to_string_lossy().into_owned(),
                entry.path(),
            ));
        }
        self.cache.directories.insert(
            directory.to_path_buf(),
            CachedDirectory::Entries(entries.clone()),
        );
        Ok(entries)
    }

    fn relative_key(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.root)
            .map_or_else(|_| path.to_path_buf(), Path::to_path_buf)
    }
}

fn append_fingerprint_bytes(
    bytes: &mut Vec<u8>,
    files: &BTreeMap<PathBuf, FileDependency>,
    candidate_sets: &[CandidateSetDependency],
) {
    for (key, dependency) in files {
        bytes.extend_from_slice(key.to_string_lossy().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(dependency.label.as_bytes());
        bytes.push(0);
        append_path_fingerprint(bytes, &dependency.fingerprint);
    }
    for dependency in candidate_sets {
        bytes.extend_from_slice(dependency.path.to_string_lossy().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(dependency.label.as_bytes());
        bytes.push(0);
        match &dependency.matcher {
            CandidateMatcher::PrefixSuffix { prefix, suffix } => {
                bytes.extend_from_slice(prefix.as_deref().unwrap_or("<any>").as_bytes());
                bytes.push(0);
                bytes.extend_from_slice(suffix.as_deref().unwrap_or("<any>").as_bytes());
                bytes.push(0);
            }
        }
        for (name, member) in &dependency.members {
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(0);
            append_candidate_fingerprint(bytes, member);
        }
    }
}

fn append_path_fingerprint(bytes: &mut Vec<u8>, fingerprint: &PathFingerprint) {
    bytes.extend_from_slice(fingerprint.kind.as_bytes());
    bytes.push(0);
    if let Some(digest) = &fingerprint.digest {
        bytes.extend_from_slice(digest.to_string().as_bytes());
    }
    bytes.push(0xff);
}

fn append_candidate_fingerprint(bytes: &mut Vec<u8>, fingerprint: &CandidateFingerprint) {
    bytes.extend_from_slice(fingerprint.kind.as_bytes());
    bytes.push(0);
    if let Some(digest) = &fingerprint.digest {
        bytes.extend_from_slice(digest.to_string().as_bytes());
    }
    bytes.push(0xff);
}

fn fingerprint_path(
    path: &Path,
    cost: &mut ObservationLedgerCost,
) -> Result<CachedPathObservation, ObserverError> {
    cost.fingerprint_reads += 1;
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(CachedPathObservation {
                fingerprint: PathFingerprint {
                    kind: "missing",
                    digest: None,
                },
                content: None,
            });
        }
        Err(source) => {
            return Err(ObserverError::Read {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        let target = fs::read_link(path).map_err(|source| ObserverError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let mut target_facts = Vec::new();
        target_facts.extend_from_slice(target.to_string_lossy().as_bytes());
        target_facts.push(0);
        match fs::canonicalize(path)
            .and_then(|resolved| fs::metadata(&resolved).map(|m| (resolved, m)))
        {
            Ok((resolved, metadata)) if metadata.file_type().is_file() => {
                let bytes = fs::read(&resolved).map_err(|source| ObserverError::Read {
                    path: resolved.clone(),
                    source,
                })?;
                cost.content_reads += 1;
                cost.content_hashes += 1;
                target_facts.extend_from_slice(b"file\0");
                target_facts.extend_from_slice(Digest::sha256_bytes(&bytes).to_string().as_bytes());
            }
            Ok((_, metadata)) if metadata.file_type().is_dir() => {
                target_facts.extend_from_slice(b"directory\0");
            }
            Ok(_) => target_facts.extend_from_slice(b"other\0"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                target_facts.extend_from_slice(b"missing\0");
            }
            Err(source) => {
                return Err(ObserverError::Read {
                    path: path.to_path_buf(),
                    source,
                });
            }
        }
        return Ok(CachedPathObservation {
            fingerprint: PathFingerprint {
                kind: "symlink",
                digest: Some(Digest::sha256_bytes(&target_facts)),
            },
            content: None,
        });
    }
    if file_type.is_file() {
        let bytes = fs::read(path).map_err(|source| ObserverError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        cost.content_reads += 1;
        cost.content_hashes += 1;
        return Ok(CachedPathObservation {
            fingerprint: PathFingerprint {
                kind: "file",
                digest: Some(Digest::sha256_bytes(&bytes)),
            },
            content: Some(bytes),
        });
    }
    Ok(CachedPathObservation {
        fingerprint: PathFingerprint {
            kind: if file_type.is_dir() {
                "directory"
            } else {
                "other"
            },
            digest: None,
        },
        content: None,
    })
}

fn drift_error(label: &str, path: &Path, detail: &str) -> ObserverError {
    ObserverError::State {
        path: path.to_path_buf(),
        message: format!("observation dependency {label} drifted: {detail}"),
    }
}

#[cfg(test)]
mod tests {
    use super::CandidateMatcher;
    use super::ObservationLedger;
    use std::fs;

    #[test]
    fn detects_file_and_candidate_set_drift() {
        let directory = tempfile::tempdir().expect("tempdir");
        let decisions = directory.path().join("decisions");
        fs::create_dir(&decisions).expect("decisions directory");
        let fact = decisions.join("WI-1.close.json");
        fs::write(&fact, b"{\"state\":\"closed\"}").expect("fact");

        let mut ledger = ObservationLedger::new(directory.path());
        ledger.register_file("close", &fact).expect("register file");
        ledger
            .register_candidate_set(
                "decisions",
                &decisions,
                CandidateMatcher::PrefixSuffix {
                    prefix: None,
                    suffix: Some(".json".into()),
                },
            )
            .expect("register candidates");

        fs::write(&fact, b"{\"state\":\"open\"}").expect("mutate fact");
        fs::write(decisions.join("WI-1.recovery.json"), b"{}").expect("add candidate");

        let drift = ledger
            .recheck_digest(&[])
            .expect_err("drift must fail closed");
        assert!(drift.to_string().contains("close"), "{drift}");
        assert!(drift.to_string().contains("decisions"), "{drift}");
    }

    #[test]
    fn request_cache_deduplicates_initial_reads_but_refreshes_at_boundary() {
        let directory = tempfile::tempdir().expect("tempdir");
        let decisions = directory.path().join("decisions");
        fs::create_dir(&decisions).expect("decisions directory");
        let fact = decisions.join("WI-1.close.json");
        fs::write(&fact, b"{\"state\":\"closed\"}").expect("fact");

        let mut ledger = ObservationLedger::new(directory.path());
        ledger.register_file("close", &fact).expect("register file");
        ledger
            .register_candidate_set(
                "decisions",
                &decisions,
                CandidateMatcher::PrefixSuffix {
                    prefix: None,
                    suffix: Some(".json".into()),
                },
            )
            .expect("register candidates");

        let initial = ledger.read_counters();
        assert_eq!(initial.fingerprint_reads, 1);
        assert_eq!(initial.candidate_directory_reads, 1);
        assert_eq!(initial.candidate_member_fingerprint_reads, 0);
        assert_eq!(
            initial.content_reads, 1,
            "shared candidate must reuse file read"
        );
        assert_eq!(
            initial.content_hashes, 1,
            "shared candidate must reuse content hash"
        );

        ledger.recheck_digest(&[]).expect("stable boundary");
        let boundary = ledger.read_counters();
        assert_eq!(boundary.fingerprint_reads, 1);
        assert_eq!(boundary.candidate_directory_reads, 1);
        assert_eq!(boundary.candidate_member_fingerprint_reads, 0);
        assert_eq!(
            boundary.content_reads, 1,
            "final boundary must re-read the file instead of reusing the initial cache"
        );
        assert_eq!(boundary.content_hashes, 1);
    }

    #[cfg(unix)]
    #[test]
    fn detects_type_and_symlink_target_drift() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("dependency");
        fs::write(&path, b"one").expect("dependency");
        let mut ledger = ObservationLedger::new(directory.path());
        ledger.register_file("dependency", &path).expect("register");
        fs::remove_file(&path).expect("remove dependency");
        fs::create_dir(&path).expect("replace dependency with directory");
        assert!(
            ledger.recheck_digest(&[]).is_err(),
            "type replacement must drift"
        );

        fs::remove_dir(&path).expect("remove directory");
        let target_a = directory.path().join("target-a");
        let target_b = directory.path().join("target-b");
        fs::write(&target_a, b"a").expect("target a");
        fs::write(&target_b, b"b").expect("target b");
        symlink(&target_a, &path).expect("symlink");
        let mut symlink_ledger = ObservationLedger::new(directory.path());
        symlink_ledger
            .register_file("symlink", &path)
            .expect("register symlink");
        fs::write(&target_a, b"changed").expect("mutate symlink target contents");
        assert!(
            symlink_ledger.recheck_digest(&[]).is_err(),
            "symlink target content replacement must drift"
        );
        fs::write(&target_a, b"a").expect("restore target a");
        fs::remove_file(&path).expect("remove symlink");
        symlink(&target_b, &path).expect("replace symlink target");
        assert!(
            symlink_ledger.recheck_digest(&[]).is_err(),
            "symlink target replacement must drift"
        );
    }
}
