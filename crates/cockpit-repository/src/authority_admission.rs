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
        check_declarations(&self.root, self.work_item_id.as_deref())?;
        Ok(CommitBoundary { _private: () })
    }
}

pub(crate) fn enter(
    root: &Path,
    work_item_id: Option<&str>,
) -> Result<AdmissionContext, ObserverError> {
    check_declarations(root, work_item_id)?;
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
    if let Ok(canonical) = absolute.canonicalize() {
        if canonical != absolute {
            paths.push(canonical);
        }
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

fn check_locator(parent: &Dir, name: &str, path: &Path) -> Result<(), ObserverError> {
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
    let Some(declaration) = value.pointer("/governanceProfile/goalAuthority") else {
        return Ok(());
    };
    match declaration.get("required").and_then(Value::as_bool) {
        Some(false) => Ok(()),
        Some(true) => Err(ObserverError::State {
            path: path.into(),
            message: "goal_authority_source_unsupported: no authenticated production source, enabled-mode binding or source commit fence is configured".into(),
        }),
        None => Err(ObserverError::State {
            path: path.into(),
            message: "goal_authority_declaration_invalid: required must be an explicit boolean".into(),
        }),
    }
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
            check_locator(&directory, &name, &section_path.join(&name))?;
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
                check_locator(&directory, name, &section_path.join(name))?;
            }
        }
    }
    Ok(())
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
}
