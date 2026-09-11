//! Provider Release identity checks used by publication retries.
//!
//! A retry may reuse an existing Release only when the immutable tag points to
//! the candidate commit and the provider exposes the exact candidate asset set
//! with the exact digests. Any missing digest or extra asset is a block, never
//! a reason to overwrite the Release.

use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{error::ReleaseError, manifest::ReleaseManifest};

#[derive(Debug, Deserialize)]
struct ProviderRelease {
    id: u64,
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<ProviderAsset>,
}

#[derive(Debug, Deserialize)]
struct ProviderAsset {
    name: String,
    digest: Option<String>,
    size: u64,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderReuseReceipt {
    pub state: String,
    pub provider_release_id: u64,
    pub tag: String,
    pub commit: String,
    pub asset_count: usize,
}

pub fn verify_existing_release(
    manifest_path: &Path,
    dist: &Path,
    release_json_path: &Path,
    tag_commit: &str,
) -> Result<ProviderReuseReceipt, ReleaseError> {
    let manifest = ReleaseManifest::parse_str(&fs::read_to_string(manifest_path)?)?;
    let validated = manifest.validate_staged(dist)?;
    if manifest.commit != tag_commit {
        return Err(ReleaseError::Invalid(
            "candidate commit does not match the immutable provider tag".into(),
        ));
    }
    let provider: ProviderRelease = serde_json::from_slice(&fs::read(release_json_path)?)?;
    if provider.id == 0
        || provider.tag_name != manifest.tag
        || provider.draft
        || provider.prerelease
    {
        return Err(ReleaseError::Invalid(
            "existing provider Release identity is not the requested published Release".into(),
        ));
    }

    let mut expected = validated.files;
    let sums_path = dist.join("SHA256SUMS");
    let sums = fs::read(&sums_path)?;
    expected.insert(
        "SHA256SUMS".into(),
        crate::manifest::FileRecord {
            filename: "SHA256SUMS".into(),
            bytes: sums.len() as u64,
            sha256: crate::manifest::sha256_file(&sums_path)?,
        },
    );
    let mut observed = BTreeMap::new();
    for asset in provider.assets {
        let digest = asset.digest.ok_or_else(|| {
            ReleaseError::Invalid(format!(
                "provider Release asset {} has no digest",
                asset.name
            ))
        })?;
        let digest = digest.strip_prefix("sha256:").ok_or_else(|| {
            ReleaseError::Invalid(format!("unsupported provider digest for {}", asset.name))
        })?;
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ReleaseError::Invalid(format!(
                "provider Release asset {} has an invalid digest",
                asset.name
            )));
        }
        if observed
            .insert(
                asset.name.clone(),
                (digest.to_ascii_lowercase(), asset.size),
            )
            .is_some()
        {
            return Err(ReleaseError::Invalid(format!(
                "provider Release contains duplicate asset {}",
                asset.name
            )));
        }
    }
    if observed.len() != expected.len() {
        return Err(ReleaseError::Invalid(format!(
            "provider Release asset count mismatch (expected {}, observed {})",
            expected.len(),
            observed.len()
        )));
    }
    for (name, record) in &expected {
        let Some((digest, size)) = observed.get(name) else {
            return Err(ReleaseError::Invalid(format!(
                "provider Release is missing candidate asset {name}"
            )));
        };
        if digest != &record.sha256 || size != &record.bytes {
            return Err(ReleaseError::Invalid(format!(
                "provider Release asset {name} does not match the candidate digest or size"
            )));
        }
    }

    Ok(ProviderReuseReceipt {
        state: "reused".into(),
        provider_release_id: provider.id,
        tag: manifest.tag,
        commit: manifest.commit,
        asset_count: expected.len(),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use super::*;
    use crate::manifest::{sha256_file, write_checksums};

    fn candidate() -> (
        tempfile::TempDir,
        ReleaseManifest,
        String,
        std::path::PathBuf,
    ) {
        let temp = tempfile::tempdir().unwrap();
        let dist = temp.path().join("dist");
        fs::create_dir_all(dist.join("Formula")).unwrap();
        let commit = "a".repeat(40);
        let targets = [
            ("aarch64-apple-darwin", "tar.gz"),
            ("aarch64-unknown-linux-gnu", "tar.gz"),
            ("x86_64-apple-darwin", "tar.gz"),
            ("x86_64-pc-windows-msvc", "zip"),
            ("x86_64-unknown-linux-gnu", "tar.gz"),
        ];
        for (target, extension) in targets {
            fs::write(
                dist.join(format!("ai-cockpit-v0.2.90-{target}.{extension}")),
                format!("archive-{target}"),
            )
            .unwrap();
            fs::write(
                dist.join(format!("ai-cockpit-v0.2.90-{target}.spdx.json")),
                format!("sbom-{target}"),
            )
            .unwrap();
        }
        fs::write(dist.join("Formula/ai-cockpit.rb"), "formula").unwrap();
        let manifest = ReleaseManifest::from_staged_dist(
            "0.2.90",
            "v0.2.90",
            &commit,
            &format!("{}", "b".repeat(64)),
            &dist,
        )
        .unwrap();
        fs::write(
            dist.join("release-manifest.json"),
            manifest.canonical_bytes().unwrap(),
        )
        .unwrap();
        write_checksums(&manifest, &dist).unwrap();
        let validated = manifest.validate_staged(&dist).unwrap();
        let mut assets = BTreeMap::new();
        for (name, record) in validated.files {
            assets.insert(
                name,
                serde_json::json!({"digest": format!("sha256:{}", record.sha256), "size": record.bytes}),
            );
        }
        let sums = dist.join("SHA256SUMS");
        assets.insert(
            "SHA256SUMS".into(),
            serde_json::json!({"digest": format!("sha256:{}", sha256_file(&sums).unwrap()), "size": fs::metadata(&sums).unwrap().len()}),
        );
        let release_json = temp.path().join("release.json");
        let api_assets = assets
            .into_iter()
            .map(|(name, value)| {
                serde_json::json!({
                    "name": name,
                    "digest": value["digest"],
                    "size": value["size"]
                })
            })
            .collect::<Vec<_>>();
        fs::write(
            &release_json,
            serde_json::to_vec(&serde_json::json!({
                "id": 91,
                "tag_name": "v0.2.90",
                "draft": false,
                "prerelease": false,
                "assets": api_assets
            }))
            .unwrap(),
        )
        .unwrap();
        (temp, manifest, commit, release_json)
    }

    #[test]
    fn provider_receipt_reuses_only_an_exact_existing_release() {
        let (temp, manifest, commit, release_json) = candidate();
        let receipt = verify_existing_release(
            &temp.path().join("dist/release-manifest.json"),
            &temp.path().join("dist"),
            &release_json,
            &commit,
        )
        .unwrap();
        assert_eq!(receipt.state, "reused");
        assert_eq!(receipt.provider_release_id, 91);
        assert_eq!(receipt.tag, manifest.tag);
        assert_eq!(receipt.asset_count, 13);
    }

    #[test]
    fn provider_receipt_blocks_digest_or_tag_mismatch() {
        let (temp, _manifest, commit, release_json) = candidate();
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&release_json).unwrap()).unwrap();
        value["assets"][0]["digest"] =
            serde_json::Value::String(format!("sha256:{}", "c".repeat(64)));
        fs::write(&release_json, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(
            verify_existing_release(
                &temp.path().join("dist/release-manifest.json"),
                &temp.path().join("dist"),
                &release_json,
                &commit,
            )
            .is_err()
        );
    }
}
