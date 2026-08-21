use std::{fs, io::Write, path::Path};

use cockpit_release::manifest::ReleaseManifest;
use sha2::{Digest, Sha256};

const COMMIT: &str = "0000000000000000000000000000000000000000";
const LOCK_DIGEST: &str = "1111111111111111111111111111111111111111111111111111111111111111";

fn artifact(target: &str, os: &str, architecture: &str, archive: &str, sbom: &str) -> String {
    let runner = match target {
        "aarch64-apple-darwin" => "macos-15",
        "aarch64-unknown-linux-gnu" => "ubuntu-24.04-arm",
        "x86_64-apple-darwin" => "macos-15-intel",
        "x86_64-pc-windows-msvc" => "windows-2025",
        "x86_64-unknown-linux-gnu" => "ubuntu-24.04",
        _ => unreachable!(),
    };
    format!(
        r#"{{"target":"{target}","os":"{os}","architecture":"{architecture}","runnerImage":"{runner}","archive":{{"filename":"{archive}","bytes":3,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}},"sbom":{{"filename":"{sbom}","bytes":3,"sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}},"provenanceSubject":"{archive}"}}"#
    )
}

fn valid_json() -> String {
    let artifacts = [
        artifact(
            "aarch64-apple-darwin",
            "macos",
            "arm64",
            "ai-cockpit-v0.1.0-aarch64-apple-darwin.tar.gz",
            "ai-cockpit-v0.1.0-aarch64-apple-darwin.spdx.json",
        ),
        artifact(
            "aarch64-unknown-linux-gnu",
            "linux",
            "arm64",
            "ai-cockpit-v0.1.0-aarch64-unknown-linux-gnu.tar.gz",
            "ai-cockpit-v0.1.0-aarch64-unknown-linux-gnu.spdx.json",
        ),
        artifact(
            "x86_64-apple-darwin",
            "macos",
            "x86_64",
            "ai-cockpit-v0.1.0-x86_64-apple-darwin.tar.gz",
            "ai-cockpit-v0.1.0-x86_64-apple-darwin.spdx.json",
        ),
        artifact(
            "x86_64-pc-windows-msvc",
            "windows",
            "x86_64",
            "ai-cockpit-v0.1.0-x86_64-pc-windows-msvc.zip",
            "ai-cockpit-v0.1.0-x86_64-pc-windows-msvc.spdx.json",
        ),
        artifact(
            "x86_64-unknown-linux-gnu",
            "linux",
            "x86_64",
            "ai-cockpit-v0.1.0-x86_64-unknown-linux-gnu.tar.gz",
            "ai-cockpit-v0.1.0-x86_64-unknown-linux-gnu.spdx.json",
        ),
    ]
    .join(",");
    format!(
        r#"{{"schemaVersion":1,"product":"ai-cockpit","package":"cockpit-cli","version":"0.1.0","tag":"v0.1.0","commit":"{COMMIT}","cargoLockSha256":"{LOCK_DIGEST}","artifacts":[{artifacts}]}}"#
    )
}

#[test]
fn valid_manifest_has_five_sorted_targets_and_stable_bytes() {
    let json = valid_json();
    let manifest = ReleaseManifest::parse_str(&json).expect("valid manifest");
    assert_eq!(manifest.artifacts().len(), 5);
    assert_eq!(
        manifest.canonical_bytes().unwrap(),
        manifest.canonical_bytes().unwrap()
    );
    assert!(manifest.canonical_bytes().unwrap().ends_with(b"\n"));
}

#[test]
fn unknown_top_level_field_is_rejected() {
    let json = valid_json().replace("\"artifacts\"", "\"futurePolicy\":true,\"artifacts\"");
    let error = ReleaseManifest::parse_str(&json).expect_err("unknown field must fail");
    assert!(error.to_string().contains("unknown"));
}

#[test]
fn unknown_nested_field_is_rejected() {
    let json = valid_json().replace(
        "\"target\":\"aarch64",
        "\"extra\":true,\"target\":\"aarch64",
    );
    let error = ReleaseManifest::parse_str(&json).expect_err("unknown nested field must fail");
    assert!(error.to_string().contains("unknown"));
}

#[test]
fn duplicate_target_is_rejected() {
    let json = valid_json().replace(
        "\"target\":\"x86_64-pc-windows-msvc\"",
        "\"target\":\"aarch64-apple-darwin\"",
    );
    let error = ReleaseManifest::parse_str(&json).expect_err("duplicate target must fail");
    assert!(error.to_string().contains("duplicate"));
}

#[test]
fn malformed_digest_is_rejected() {
    let json = valid_json().replace(
        LOCK_DIGEST,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    );
    let error = ReleaseManifest::parse_str(&json).expect_err("uppercase digest must fail");
    assert!(error.to_string().contains("digest"));
}

#[test]
fn missing_target_is_rejected() {
    let mut value: serde_json::Value = serde_json::from_str(&valid_json()).unwrap();
    value["artifacts"].as_array_mut().unwrap().pop();
    let json = serde_json::to_string(&value).unwrap();
    let error = ReleaseManifest::parse_str(&json).expect_err("wrong cardinality must fail");
    assert!(error.to_string().contains("target") || error.to_string().contains("artifact"));
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn staged_manifest(dist: &Path) -> String {
    let targets = [
        (
            "aarch64-apple-darwin",
            "macos",
            "arm64",
            "ai-cockpit-v0.1.0-aarch64-apple-darwin.tar.gz",
            "ai-cockpit-v0.1.0-aarch64-apple-darwin.spdx.json",
        ),
        (
            "aarch64-unknown-linux-gnu",
            "linux",
            "arm64",
            "ai-cockpit-v0.1.0-aarch64-unknown-linux-gnu.tar.gz",
            "ai-cockpit-v0.1.0-aarch64-unknown-linux-gnu.spdx.json",
        ),
        (
            "x86_64-apple-darwin",
            "macos",
            "x86_64",
            "ai-cockpit-v0.1.0-x86_64-apple-darwin.tar.gz",
            "ai-cockpit-v0.1.0-x86_64-apple-darwin.spdx.json",
        ),
        (
            "x86_64-pc-windows-msvc",
            "windows",
            "x86_64",
            "ai-cockpit-v0.1.0-x86_64-pc-windows-msvc.zip",
            "ai-cockpit-v0.1.0-x86_64-pc-windows-msvc.spdx.json",
        ),
        (
            "x86_64-unknown-linux-gnu",
            "linux",
            "x86_64",
            "ai-cockpit-v0.1.0-x86_64-unknown-linux-gnu.tar.gz",
            "ai-cockpit-v0.1.0-x86_64-unknown-linux-gnu.spdx.json",
        ),
    ];
    let mut artifacts = Vec::new();
    let mut checksums = Vec::new();
    for (target, os, architecture, archive, sbom) in targets {
        let archive_bytes = b"archive";
        let sbom_bytes = b"sbom";
        fs::write(dist.join(archive), archive_bytes).unwrap();
        fs::write(dist.join(sbom), sbom_bytes).unwrap();
        artifacts.push(serde_json::json!({
            "target": target,
            "os": os,
            "architecture": architecture,
            "runnerImage": match target {
                "aarch64-apple-darwin" => "macos-15",
                "aarch64-unknown-linux-gnu" => "ubuntu-24.04-arm",
                "x86_64-apple-darwin" => "macos-15-intel",
                "x86_64-pc-windows-msvc" => "windows-2025",
                "x86_64-unknown-linux-gnu" => "ubuntu-24.04",
                _ => unreachable!(),
            },
            "archive": {"filename": archive, "bytes": archive_bytes.len(), "sha256": digest(archive_bytes)},
            "sbom": {"filename": sbom, "bytes": sbom_bytes.len(), "sha256": digest(sbom_bytes)},
            "provenanceSubject": archive,
        }));
        checksums.push((archive, digest(archive_bytes)));
        checksums.push((sbom, digest(sbom_bytes)));
    }
    checksums.sort_unstable_by(|left, right| left.0.cmp(right.0));
    let checksum_text = checksums
        .iter()
        .map(|(filename, hash)| format!("{hash}  {filename}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(dist.join("SHA256SUMS"), format!("{checksum_text}\n")).unwrap();
    serde_json::json!({
        "schemaVersion": 1,
        "product": "ai-cockpit",
        "package": "cockpit-cli",
        "version": "0.1.0",
        "tag": "v0.1.0",
        "commit": COMMIT,
        "cargoLockSha256": LOCK_DIGEST,
        "artifacts": artifacts,
    })
    .to_string()
}

#[test]
fn staged_files_and_checksums_are_verified() {
    let dist = tempfile::tempdir().unwrap();
    let manifest = ReleaseManifest::parse_str(&staged_manifest(dist.path())).unwrap();
    let validated = manifest.validate_staged(dist.path()).unwrap();
    assert_eq!(validated.files.len(), 10);
}

#[test]
fn manifest_can_be_built_from_the_five_staged_targets() {
    let dist = tempfile::tempdir().unwrap();
    let manifest =
        ReleaseManifest::from_staged_dist("0.1.0", "v0.1.0", COMMIT, LOCK_DIGEST, dist.path());
    assert!(manifest.is_err(), "missing staged files must fail closed");

    let json = staged_manifest(dist.path());
    let mut parsed = ReleaseManifest::parse_str(&json).unwrap();
    for artifact in &mut parsed.artifacts {
        artifact.runner_image = match artifact.target.as_str() {
            "aarch64-apple-darwin" => "macos-15",
            "aarch64-unknown-linux-gnu" => "ubuntu-24.04-arm",
            "x86_64-apple-darwin" => "macos-15-intel",
            "x86_64-pc-windows-msvc" => "windows-2025",
            "x86_64-unknown-linux-gnu" => "ubuntu-24.04",
            _ => unreachable!(),
        }
        .into();
    }
    let rebuilt =
        ReleaseManifest::from_staged_dist("0.1.0", "v0.1.0", COMMIT, LOCK_DIGEST, dist.path())
            .unwrap();
    assert_eq!(parsed, rebuilt);
}

#[test]
fn checksum_extra_file_is_rejected() {
    let dist = tempfile::tempdir().unwrap();
    let manifest = ReleaseManifest::parse_str(&staged_manifest(dist.path())).unwrap();
    fs::OpenOptions::new()
        .append(true)
        .open(dist.path().join("SHA256SUMS"))
        .unwrap()
        .write_all(
            b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  unexpected.txt\n",
        )
        .unwrap();
    let error = manifest
        .validate_staged(dist.path())
        .expect_err("extra checksum entry must fail");
    assert!(error.to_string().contains("SHA256SUMS"));
}

#[test]
fn checksum_order_and_duplicate_lines_are_rejected() {
    let dist = tempfile::tempdir().unwrap();
    let manifest = ReleaseManifest::parse_str(&staged_manifest(dist.path())).unwrap();
    let checksum_path = dist.path().join("SHA256SUMS");
    let mut lines: Vec<_> = fs::read_to_string(&checksum_path)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    lines.reverse();
    fs::write(&checksum_path, format!("{}\n", lines.join("\n"))).unwrap();
    let error = manifest
        .validate_staged(dist.path())
        .expect_err("unsorted checksum lines must fail");
    assert!(error.to_string().contains("sorted"));

    let dist = tempfile::tempdir().unwrap();
    let manifest = ReleaseManifest::parse_str(&staged_manifest(dist.path())).unwrap();
    let checksum_path = dist.path().join("SHA256SUMS");
    let first = fs::read_to_string(&checksum_path)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    fs::OpenOptions::new()
        .append(true)
        .open(&checksum_path)
        .unwrap()
        .write_all(format!("{first}\n").as_bytes())
        .unwrap();
    let error = manifest
        .validate_staged(dist.path())
        .expect_err("duplicate checksum lines must fail");
    assert!(error.to_string().contains("duplicate"));
}

#[test]
fn archive_and_sbom_must_have_distinct_filenames() {
    let mut value: serde_json::Value = serde_json::from_str(&valid_json()).unwrap();
    let archive_name = value["artifacts"][0]["archive"]["filename"].clone();
    value["artifacts"][0]["sbom"]["filename"] = archive_name;
    let error = ReleaseManifest::parse_str(&serde_json::to_string(&value).unwrap())
        .expect_err("archive and SBOM filename collision must fail");
    assert!(error.to_string().contains("filename"));
}

#[test]
fn runner_image_must_match_target() {
    let json = valid_json().replace(
        "runnerImage\":\"macos-15\"",
        "runnerImage\":\"wrong-image\"",
    );
    let error = ReleaseManifest::parse_str(&json).expect_err("runner image must be bound");
    assert!(error.to_string().contains("runner"));
}

#[test]
fn archive_filename_must_be_exact_target_template() {
    let json = valid_json().replace(
        "ai-cockpit-v0.1.0-aarch64-apple-darwin.tar.gz",
        "ai-cockpit-v0.1.0-aarch64-apple-darwin.extra.tar.gz",
    );
    let error = ReleaseManifest::parse_str(&json).expect_err("archive template must be exact");
    assert!(error.to_string().contains("archive filename"));
}

#[test]
fn sbom_filename_must_be_exact_target_template() {
    let json = valid_json().replace(
        "ai-cockpit-v0.1.0-aarch64-apple-darwin.spdx.json",
        "ai-cockpit-v0.1.0-aarch64-apple-darwin.extra.spdx.json",
    );
    let error = ReleaseManifest::parse_str(&json).expect_err("SBOM template must be exact");
    assert!(error.to_string().contains("SBOM filename"));
}

#[test]
fn version_must_be_canonical_semver() {
    let json = valid_json()
        .replace("v0.1.0", "v01.2.3")
        .replace("\"version\":\"0.1.0\"", "\"version\":\"01.2.3\"");
    let error = ReleaseManifest::parse_str(&json).expect_err("non-canonical semver must fail");
    assert!(error.to_string().contains("version"));
}
