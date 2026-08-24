use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::Path,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::ReleaseError;

const EXPECTED_TARGETS: [&str; 5] = [
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FileRecord {
    pub filename: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRecord {
    pub target: String,
    pub os: String,
    pub architecture: String,
    #[serde(rename = "runnerImage")]
    pub runner_image: String,
    pub archive: FileRecord,
    pub sbom: FileRecord,
    #[serde(rename = "provenanceSubject")]
    pub provenance_subject: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReleaseManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub product: String,
    pub package: String,
    pub version: String,
    pub tag: String,
    pub commit: String,
    #[serde(rename = "cargoLockSha256")]
    pub cargo_lock_sha256: String,
    pub artifacts: Vec<ArtifactRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedRelease {
    pub files: BTreeMap<String, FileRecord>,
}

impl ReleaseManifest {
    pub fn from_staged_dist(
        version: &str,
        tag: &str,
        commit: &str,
        cargo_lock_sha256: &str,
        dist: &Path,
    ) -> Result<Self, ReleaseError> {
        let mut artifacts = Vec::with_capacity(EXPECTED_TARGETS.len());
        for target in EXPECTED_TARGETS {
            let (os, architecture, runner_image, extension) = match target {
                "aarch64-apple-darwin" => ("macos", "arm64", "macos-15", "tar.gz"),
                "aarch64-unknown-linux-gnu" => ("linux", "arm64", "ubuntu-24.04-arm", "tar.gz"),
                "x86_64-apple-darwin" => ("macos", "x86_64", "macos-15-intel", "tar.gz"),
                "x86_64-pc-windows-msvc" => ("windows", "x86_64", "windows-2025", "zip"),
                "x86_64-unknown-linux-gnu" => ("linux", "x86_64", "ubuntu-24.04", "tar.gz"),
                _ => unreachable!(),
            };
            let archive_filename = format!("ai-cockpit-{tag}-{target}.{extension}");
            let sbom_filename = format!("ai-cockpit-{tag}-{target}.spdx.json");
            let archive_path = dist.join(&archive_filename);
            let sbom_path = dist.join(&sbom_filename);
            let archive = file_record(&archive_path, archive_filename)?;
            let sbom = file_record(&sbom_path, sbom_filename)?;
            artifacts.push(ArtifactRecord {
                target: target.to_string(),
                os: os.to_string(),
                architecture: architecture.to_string(),
                runner_image: runner_image.to_string(),
                provenance_subject: archive.filename.clone(),
                archive,
                sbom,
            });
        }
        let manifest = Self {
            schema_version: 1,
            product: "ai-cockpit".into(),
            package: "cockpit-cli".into(),
            version: version.into(),
            tag: tag.into(),
            commit: commit.into(),
            cargo_lock_sha256: cargo_lock_sha256.into(),
            artifacts,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn parse_str(input: &str) -> Result<Self, ReleaseError> {
        let manifest: Self = serde_json::from_str(input)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn artifacts(&self) -> &[ArtifactRecord] {
        &self.artifacts
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ReleaseError> {
        let mut bytes = serde_json::to_vec(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    pub fn validate_staged(&self, dist: &Path) -> Result<ValidatedRelease, ReleaseError> {
        self.validate()?;
        let mut files = BTreeMap::new();
        for artifact in &self.artifacts {
            for record in [&artifact.archive, &artifact.sbom] {
                let path = dist.join(&record.filename);
                let actual_bytes = std::fs::metadata(&path)?.len();
                if actual_bytes != record.bytes {
                    return Err(ReleaseError::Invalid(format!(
                        "byte count mismatch for {}",
                        record.filename
                    )));
                }
                let actual_digest = sha256_file(&path)?;
                if actual_digest != record.sha256 {
                    return Err(ReleaseError::Invalid(format!(
                        "digest mismatch for {}",
                        record.filename
                    )));
                }
                files.insert(record.filename.clone(), record.clone());
            }
        }

        for record in self.auxiliary_public_records(dist)? {
            if files.insert(record.filename.clone(), record).is_some() {
                return Err(ReleaseError::Invalid(
                    "duplicate public release asset filename".into(),
                ));
            }
        }
        self.validate_publishable_inventory(dist)?;

        let checksums = std::fs::read_to_string(dist.join("SHA256SUMS"))?;
        let mut listed = BTreeMap::new();
        let mut previous_filename = None;
        for line in checksums.lines().filter(|line| !line.trim().is_empty()) {
            let mut parts = line.split_whitespace();
            let digest = parts
                .next()
                .ok_or_else(|| ReleaseError::Invalid("malformed SHA256SUMS line".into()))?;
            let filename = parts
                .next()
                .ok_or_else(|| ReleaseError::Invalid("malformed SHA256SUMS line".into()))?;
            if parts.next().is_some() {
                return Err(ReleaseError::Invalid("malformed SHA256SUMS line".into()));
            }
            validate_digest(digest, "checksum")?;
            if listed.contains_key(filename) {
                return Err(ReleaseError::Invalid(
                    "duplicate SHA256SUMS filename".into(),
                ));
            }
            if previous_filename
                .as_deref()
                .is_some_and(|previous| previous >= filename)
            {
                return Err(ReleaseError::Invalid(
                    "SHA256SUMS entries must be sorted by filename".into(),
                ));
            }
            previous_filename = Some(filename.to_string());
            listed.insert(filename.to_string(), digest.to_string());
        }

        let expected: BTreeMap<_, _> = files
            .iter()
            .map(|(filename, record)| (filename.clone(), record.sha256.clone()))
            .collect();
        if listed != expected {
            return Err(ReleaseError::Invalid(
                "SHA256SUMS does not cover exactly the public release assets".into(),
            ));
        }

        Ok(ValidatedRelease { files })
    }

    pub fn validate(&self) -> Result<(), ReleaseError> {
        if self.schema_version != 1 {
            return Err(ReleaseError::Invalid("unsupported manifest schema".into()));
        }
        if self.product != "ai-cockpit" || self.package != "cockpit-cli" {
            return Err(ReleaseError::Invalid(
                "manifest product or package is not ai-cockpit/cockpit-cli".into(),
            ));
        }
        let parsed_version = semver::Version::parse(&self.version)
            .map_err(|error| ReleaseError::Invalid(format!("invalid version: {error}")))?;
        if parsed_version.to_string() != self.version || self.tag != format!("v{}", self.version) {
            return Err(ReleaseError::Invalid(
                "manifest version and tag do not match".into(),
            ));
        }
        validate_commit(&self.commit)?;
        validate_digest(&self.cargo_lock_sha256, "cargo lock")?;
        if self.artifacts.len() != EXPECTED_TARGETS.len() {
            return Err(ReleaseError::Invalid(format!(
                "manifest must contain exactly {} targets",
                EXPECTED_TARGETS.len()
            )));
        }
        let mut targets = BTreeMap::new();
        let mut filenames = BTreeMap::new();
        let mut previous = None;
        for artifact in &self.artifacts {
            if targets.insert(artifact.target.clone(), ()).is_some() {
                return Err(ReleaseError::Invalid(format!(
                    "duplicate target {}",
                    artifact.target
                )));
            }
            if previous.is_some_and(|value| value >= artifact.target.as_str()) {
                return Err(ReleaseError::Invalid(
                    "artifacts must be sorted by target".into(),
                ));
            }
            previous = Some(artifact.target.as_str());
            let expected_index = EXPECTED_TARGETS
                .iter()
                .position(|target| *target == artifact.target)
                .ok_or_else(|| {
                    ReleaseError::Invalid(format!("unsupported target {}", artifact.target))
                })?;
            let (expected_os, expected_architecture, expected_runner, suffix) = match expected_index
            {
                0 => ("macos", "arm64", "macos-15", "tar.gz"),
                1 => ("linux", "arm64", "ubuntu-24.04-arm", "tar.gz"),
                2 => ("macos", "x86_64", "macos-15-intel", "tar.gz"),
                3 => ("windows", "x86_64", "windows-2025", "zip"),
                4 => ("linux", "x86_64", "ubuntu-24.04", "tar.gz"),
                _ => unreachable!(),
            };
            if artifact.os != expected_os
                || artifact.architecture != expected_architecture
                || artifact.runner_image != expected_runner
            {
                return Err(ReleaseError::Invalid(format!(
                    "platform metadata mismatch (OS, architecture, or runner image) for {}",
                    artifact.target
                )));
            }
            validate_filename(&artifact.archive.filename)?;
            validate_filename(&artifact.sbom.filename)?;
            if filenames
                .insert(artifact.archive.filename.clone(), ())
                .is_some()
                || filenames
                    .insert(artifact.sbom.filename.clone(), ())
                    .is_some()
            {
                return Err(ReleaseError::Invalid(
                    "archive and SBOM filenames must be unique".into(),
                ));
            }
            let expected_archive =
                format!("ai-cockpit-{}-{}.{}", self.tag, artifact.target, suffix);
            let expected_sbom = format!("ai-cockpit-{}-{}.spdx.json", self.tag, artifact.target);
            if artifact.archive.filename != expected_archive {
                return Err(ReleaseError::Invalid(format!(
                    "archive filename mismatch for {}",
                    artifact.target
                )));
            }
            if artifact.sbom.filename != expected_sbom {
                return Err(ReleaseError::Invalid(format!(
                    "SBOM filename mismatch for {}",
                    artifact.target
                )));
            }
            if artifact.provenance_subject != artifact.archive.filename {
                return Err(ReleaseError::Invalid(format!(
                    "provenance subject mismatch for {}",
                    artifact.target
                )));
            }
            validate_digest(&artifact.archive.sha256, "archive")?;
            validate_digest(&artifact.sbom.sha256, "SBOM")?;
        }
        Ok(())
    }

    fn auxiliary_public_records(&self, dist: &Path) -> Result<Vec<FileRecord>, ReleaseError> {
        let manifest_path = dist.join("release-manifest.json");
        if std::fs::read(&manifest_path)? != self.canonical_bytes()? {
            return Err(ReleaseError::Invalid(
                "release-manifest.json does not match the validated manifest".into(),
            ));
        }
        Ok(vec![
            file_record(&manifest_path, "release-manifest.json".into())?,
            file_record(&dist.join("Formula/ai-cockpit.rb"), "ai-cockpit.rb".into())?,
        ])
    }

    fn validate_publishable_inventory(&self, dist: &Path) -> Result<(), ReleaseError> {
        let mut expected = self
            .artifacts
            .iter()
            .flat_map(|artifact| {
                [
                    artifact.archive.filename.clone(),
                    artifact.sbom.filename.clone(),
                ]
            })
            .collect::<BTreeSet<_>>();
        expected.extend([
            "release-manifest.json".into(),
            "SHA256SUMS".into(),
            "Formula/ai-cockpit.rb".into(),
        ]);

        let mut observed = BTreeSet::new();
        for entry in std::fs::read_dir(dist)? {
            let entry = entry?;
            let filename = entry
                .file_name()
                .to_str()
                .ok_or_else(|| ReleaseError::Invalid("non-UTF-8 staged filename".into()))?
                .to_string();
            let metadata = std::fs::symlink_metadata(entry.path())?;
            if metadata.file_type().is_symlink() && is_publishable_filename(&filename) {
                return Err(ReleaseError::Invalid(format!(
                    "publishable asset must be a regular file: {filename}"
                )));
            }
            if metadata.is_file() && is_publishable_filename(&filename) {
                observed.insert(filename);
            } else if metadata.is_dir() && filename == "Formula" {
                for formula_entry in std::fs::read_dir(entry.path())? {
                    let formula_entry = formula_entry?;
                    let formula_name = formula_entry
                        .file_name()
                        .to_str()
                        .ok_or_else(|| ReleaseError::Invalid("non-UTF-8 Formula filename".into()))?
                        .to_string();
                    if formula_name.ends_with(".rb") {
                        if !std::fs::symlink_metadata(formula_entry.path())?.is_file() {
                            return Err(ReleaseError::Invalid(format!(
                                "publishable asset must be a regular file: Formula/{formula_name}"
                            )));
                        }
                        observed.insert(format!("Formula/{formula_name}"));
                    }
                }
            }
        }
        if observed != expected {
            let orphan = observed.difference(&expected).cloned().collect::<Vec<_>>();
            let missing = expected.difference(&observed).cloned().collect::<Vec<_>>();
            return Err(ReleaseError::Invalid(format!(
                "orphan or missing publishable release assets (orphan={orphan:?}, missing={missing:?})"
            )));
        }
        Ok(())
    }
}

fn file_record(path: &Path, filename: String) -> Result<FileRecord, ReleaseError> {
    if !std::fs::symlink_metadata(path)?.is_file() {
        return Err(ReleaseError::Invalid(format!(
            "release asset is not a regular file: {}",
            path.display()
        )));
    }
    Ok(FileRecord {
        filename,
        bytes: std::fs::metadata(path)?.len(),
        sha256: sha256_file(path)?,
    })
}

pub fn sha256_file(path: &Path) -> Result<String, ReleaseError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn write_checksums(manifest: &ReleaseManifest, dist: &Path) -> Result<(), ReleaseError> {
    manifest.validate()?;
    let mut owned_records = manifest
        .artifacts
        .iter()
        .flat_map(|artifact| [artifact.archive.clone(), artifact.sbom.clone()])
        .collect::<Vec<_>>();
    owned_records.extend(manifest.auxiliary_public_records(dist)?);
    let mut records = owned_records
        .iter()
        .map(|record| (record.filename.as_str(), record.sha256.as_str()))
        .collect::<Vec<_>>();
    records.sort_unstable_by(|left, right| left.0.cmp(right.0));
    let mut lines = records
        .into_iter()
        .map(|(filename, digest)| format!("{digest}  {filename}"))
        .collect::<Vec<_>>();
    lines.push(String::new());
    std::fs::write(dist.join("SHA256SUMS"), lines.join("\n"))?;
    Ok(())
}

fn is_publishable_filename(filename: &str) -> bool {
    filename == "release-manifest.json"
        || filename == "SHA256SUMS"
        || filename == "ai-cockpit.rb"
        || filename.ends_with(".spdx.json")
        || filename.ends_with(".tar.gz")
        || filename.ends_with(".zip")
}

fn validate_filename(filename: &str) -> Result<(), ReleaseError> {
    if filename.is_empty()
        || filename.contains('/')
        || filename.contains('\\')
        || filename == "."
        || filename == ".."
    {
        return Err(ReleaseError::Invalid(format!(
            "unsafe artifact filename {filename}"
        )));
    }
    Ok(())
}

fn validate_digest(digest: &str, name: &str) -> Result<(), ReleaseError> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ReleaseError::Invalid(format!("invalid {name} digest")));
    }
    Ok(())
}

fn validate_commit(commit: &str) -> Result<(), ReleaseError> {
    if commit.len() != 40
        || !commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ReleaseError::Invalid(
            "invalid source commit identity".into(),
        ));
    }
    Ok(())
}
