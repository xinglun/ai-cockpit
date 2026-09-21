//! Source-version identity checks for the release boundary.
//!
//! The shell entrypoint remains responsible for process setup and public
//! provider checks.  The source-side version/document rules live here so the
//! release workflow and local checks use one typed implementation.

use std::{fs, path::Path, process::Command};

use serde::{Deserialize, Serialize};

use crate::error::ReleaseError;

const RELEASE_DISTRIBUTION_DOCS: [&str; 3] = [
    "docs/release/distribution.md",
    "docs/release/distribution.ja.md",
    "docs/release/distribution.zh-CN.md",
];
const ARCHITECTURE_RELEASE_DOCS: [&str; 3] = [
    "docs/architecture/release-distribution.md",
    "docs/architecture/release-distribution.ja.md",
    "docs/architecture/release-distribution.zh-CN.md",
];
const VERSIONING_DOCS: [&str; 3] = [
    "docs/architecture/versioning.md",
    "docs/architecture/versioning.ja.md",
    "docs/architecture/versioning.zh-CN.md",
];
const OPERATIONS_DOCS: [&str; 3] = [
    "docs/operations/README.md",
    "docs/operations/README.ja.md",
    "docs/operations/README.zh-CN.md",
];

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
    source: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VersionConsistencyReport {
    pub state: &'static str,
    pub version: String,
    pub tag: String,
    pub workspace_package_count: usize,
    pub checked_document_count: usize,
}

/// Validate the source repository's current release identity and documentation
/// baseline.  Provider-side checks intentionally remain outside this function.
pub fn validate_source(repository: &Path) -> Result<VersionConsistencyReport, ReleaseError> {
    let repository = fs::canonicalize(repository)?;
    let metadata = cargo_metadata(&repository)?;
    let workspace_packages = metadata
        .packages
        .iter()
        .filter(|package| package.source.is_none())
        .collect::<Vec<_>>();
    let cockpit_cli = workspace_packages
        .iter()
        .filter(|package| package.name == "cockpit-cli")
        .collect::<Vec<_>>();
    if cockpit_cli.len() != 1 {
        return Err(invalid(format!(
            "cockpit-cli workspace package is ambiguous: found {} local packages",
            cockpit_cli.len()
        )));
    }
    let version = cockpit_cli[0].version.clone();
    if !is_plain_semantic_version(&version) {
        return Err(invalid(format!(
            "workspace version is not a plain semantic version: {version}"
        )));
    }
    if let Some(package) = workspace_packages
        .iter()
        .find(|package| package.version != version)
    {
        return Err(invalid(format!(
            "workspace package {} has version {}, expected {}",
            package.name, package.version, version
        )));
    }

    let tag = format!("v{version}");
    let mut checked_document_count = 0;
    for path in RELEASE_DISTRIBUTION_DOCS {
        let text = read_document(&repository, path)?;
        require_contains(path, &text, &tag)?;
        require_contains(path, &text, &format!("ai-cockpit-{tag}-"))?;
        checked_document_count += 1;
    }
    for path in ARCHITECTURE_RELEASE_DOCS {
        let text = read_document(&repository, path)?;
        require_contains(path, &text, &tag)?;
        if !contains_any_case_insensitive(&text, &["baseline", "基线", "ベースライン"]) {
            return Err(invalid(format!(
                "{path} does not declare a release baseline"
            )));
        }
        checked_document_count += 1;
    }
    for path in VERSIONING_DOCS {
        let text = read_document(&repository, path)?;
        require_contains(path, &text, &version)?;
        checked_document_count += 1;
    }
    for path in OPERATIONS_DOCS {
        let text = read_document(&repository, path)?;
        require_contains(path, &text, "x86_64-unknown-linux-gnu")?;
        if contains_version_token(&text) {
            return Err(invalid(format!(
                "{path} hard-codes a release version in the current operations baseline"
            )));
        }
        checked_document_count += 1;
    }

    for path in RELEASE_DISTRIBUTION_DOCS
        .into_iter()
        .chain(ARCHITECTURE_RELEASE_DOCS)
    {
        let text = read_document(&repository, path)?;
        for line in text.lines().filter(|line| contains_baseline_marker(line)) {
            if !line.contains(&tag) {
                return Err(invalid(format!(
                    "{path} has a stale current baseline: {line}"
                )));
            }
        }
    }

    Ok(VersionConsistencyReport {
        state: "passed",
        version,
        tag,
        workspace_package_count: workspace_packages.len(),
        checked_document_count,
    })
}

fn cargo_metadata(repository: &Path) -> Result<CargoMetadata, ReleaseError> {
    let output = Command::new("cargo")
        .args(["metadata", "--locked", "--format-version", "1"])
        .current_dir(repository)
        .output()?;
    if !output.status.success() {
        let diagnostic = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(invalid(format!(
            "cargo metadata failed{}",
            if diagnostic.is_empty() {
                String::new()
            } else {
                format!(": {diagnostic}")
            }
        )));
    }
    serde_json::from_slice(&output.stdout).map_err(ReleaseError::from)
}

fn read_document(repository: &Path, relative: &str) -> Result<String, ReleaseError> {
    fs::read_to_string(repository.join(relative)).map_err(ReleaseError::from)
}

fn require_contains(path: &str, text: &str, expected: &str) -> Result<(), ReleaseError> {
    if text.contains(expected) {
        Ok(())
    } else {
        Err(invalid(format!(
            "{path} does not contain current value: {expected}"
        )))
    }
}

fn contains_any_case_insensitive(text: &str, needles: &[&str]) -> bool {
    let lowered = text.to_lowercase();
    needles
        .iter()
        .any(|needle| lowered.contains(&needle.to_lowercase()))
}

fn contains_baseline_marker(line: &str) -> bool {
    contains_any_case_insensitive(
        line,
        &[
            "current installation baseline",
            "current immutable public baseline",
            "現在の installation baseline",
            "現在の immutable public baseline",
            "当前安装基线",
            "当前不可变公开基线",
        ],
    )
}

fn contains_version_token(text: &str) -> bool {
    let bytes = text.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'v' {
            continue;
        }
        let mut cursor = index + 1;
        for component in 0..3 {
            let start = cursor;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                cursor += 1;
            }
            if cursor == start {
                break;
            }
            if component < 2 {
                if bytes.get(cursor) != Some(&b'.') {
                    break;
                }
                cursor += 1;
            } else {
                return true;
            }
        }
    }
    false
}

fn is_plain_semantic_version(value: &str) -> bool {
    let components = value.split('.').collect::<Vec<_>>();
    components.len() == 3
        && components.iter().all(|component| {
            !component.is_empty() && component.bytes().all(|byte| byte.is_ascii_digit())
        })
}

fn invalid(message: impl Into<String>) -> ReleaseError {
    ReleaseError::Invalid(message.into())
}

#[cfg(test)]
mod tests {
    use super::{contains_version_token, is_plain_semantic_version};

    #[test]
    fn detects_semantic_version_tokens_in_operations_text() {
        assert!(contains_version_token(
            "install v1.2.3 from the public Release"
        ));
        assert!(contains_version_token(
            "v1.2.3.4 is still detected as a release token"
        ));
        assert!(!contains_version_token("the current baseline is immutable"));
    }

    #[test]
    fn accepts_only_plain_three_component_versions() {
        assert!(is_plain_semantic_version("0.2.105"));
        assert!(!is_plain_semantic_version("0.2.105-alpha"));
        assert!(!is_plain_semantic_version("0.2"));
    }
}
