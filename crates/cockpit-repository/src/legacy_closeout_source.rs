//! Explicit present-day proof obligations for legacy-v1 closeouts.
//!
//! A canonical name is an obligation even when its content is damaged. Parsing
//! or a foreign identity must never make that Work Item disappear from scope.
use super::{ObserverError, open_cap_directory_nofollow_strict, validate_work_item_id};
use cap_std::fs::Dir;
use std::{collections::BTreeSet, fs, path::Path};

pub(super) const REQUIREMENT_SUFFIX: &str = ".legacy-closeout-source-requirement.json";
pub(super) const REGISTRATION_SUFFIX: &str = ".legacy-closeout-source-registration.json";
pub(super) const EXTENSION_SUFFIX: &str = ".legacy-closeout-source-extension.json";

pub(super) fn has_obligation(root: &Path, id: &str) -> bool {
    [REQUIREMENT_SUFFIX, REGISTRATION_SUFFIX, EXTENSION_SUFFIX]
        .iter()
        .any(|suffix| {
            match fs::symlink_metadata(root.join(".ai/decisions").join(format!("{id}{suffix}"))) {
                Ok(_) => true,
                Err(error) => error.kind() != std::io::ErrorKind::NotFound,
            }
        })
}

pub(super) fn obligation_ids(root: &Path) -> Result<BTreeSet<String>, ObserverError> {
    let path = root.join(".ai/decisions");
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(source) => return Err(ObserverError::Read { path, source }),
        Ok(_) => {}
    }
    let ambient = Dir::open_ambient_dir(root, cap_std::ambient_authority()).map_err(|source| {
        ObserverError::Read {
            path: root.into(),
            source,
        }
    })?;
    let ai = open_cap_directory_nofollow_strict(&ambient, ".ai", &root.join(".ai"))?;
    let decisions = open_cap_directory_nofollow_strict(&ai, "decisions", &path)?;
    let entries = decisions.entries().map_err(|source| ObserverError::Read {
        path: path.clone(),
        source,
    })?;
    let mut ids = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        for suffix in [REQUIREMENT_SUFFIX, REGISTRATION_SUFFIX, EXTENSION_SUFFIX] {
            if let Some(id) = name.strip_suffix(suffix)
                && validate_work_item_id(id).is_ok()
            {
                ids.insert(id.to_owned());
            }
        }
    }
    Ok(ids)
}

pub(super) fn integrity_allows_close(root: &Path, id: &str, _repository_id: &str) -> bool {
    // Until a complete registration is validated, a surviving obligation is
    // pending. Ordinary/unregistered legacy records keep their existing lane.
    !has_obligation(root, id)
}
