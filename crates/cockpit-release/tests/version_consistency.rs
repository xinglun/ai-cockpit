use std::fs;

use cockpit_release::version_consistency::validate_source;

fn write_file(root: &std::path::Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    write_file(
        root.path(),
        "Cargo.toml",
        "[workspace]\nmembers = [\"cli\"]\nresolver = \"2\"\n",
    );
    write_file(
        root.path(),
        "Cargo.lock",
        "version = 3\n\n[[package]]\nname = \"cockpit-cli\"\nversion = \"1.2.3\"\n",
    );
    write_file(
        root.path(),
        "cli/Cargo.toml",
        "[package]\nname = \"cockpit-cli\"\nversion = \"1.2.3\"\nedition = \"2021\"\n",
    );
    write_file(root.path(), "cli/src/lib.rs", "pub fn fixture() {}\n");
    for path in [
        "docs/release/distribution.md",
        "docs/release/distribution.ja.md",
        "docs/release/distribution.zh-CN.md",
    ] {
        write_file(
            root.path(),
            path,
            "Current installation baseline: v1.2.3\nai-cockpit-v1.2.3-x86_64-unknown-linux-gnu\n",
        );
    }
    for path in [
        "docs/architecture/release-distribution.md",
        "docs/architecture/release-distribution.ja.md",
        "docs/architecture/release-distribution.zh-CN.md",
    ] {
        write_file(
            root.path(),
            path,
            "Release baseline v1.2.3\nCurrent immutable public baseline: v1.2.3\n",
        );
    }
    for path in [
        "docs/architecture/versioning.md",
        "docs/architecture/versioning.ja.md",
        "docs/architecture/versioning.zh-CN.md",
    ] {
        write_file(root.path(), path, "The current version is 1.2.3.\n");
    }
    for path in [
        "docs/operations/README.md",
        "docs/operations/README.ja.md",
        "docs/operations/README.zh-CN.md",
    ] {
        write_file(
            root.path(),
            path,
            "Current operator target: x86_64-unknown-linux-gnu\n",
        );
    }
    write_file(
        root.path(),
        ".github/workflows/release.yml",
        "cargo metadata --locked\ntests/release/version_consistency.sh\n",
    );
    root
}

fn candidate_fixture() -> tempfile::TempDir {
    let root = fixture();
    write_file(
        root.path(),
        "cli/Cargo.toml",
        "[package]\nname = \"cockpit-cli\"\nversion = \"1.2.4-rc.2\"\nedition = \"2021\"\n",
    );
    write_file(
        root.path(),
        "Cargo.lock",
        "version = 3\n\n[[package]]\nname = \"cockpit-cli\"\nversion = \"1.2.4-rc.2\"\n",
    );
    write_file(
        root.path(),
        "docs/release/candidate.json",
        "{\"schemaVersion\":1,\"channel\":\"prerelease\",\"candidateVersion\":\"1.2.4-rc.2\",\"stableBaselineVersion\":\"1.2.3\",\"target\":\"aarch64-apple-darwin\"}\n",
    );
    write_file(
        root.path(),
        "docs/reference/reference-comparison-metadata.json",
        "{\"workspaceCandidateVersion\":\"1.2.4-rc.2\"}\n",
    );
    for path in [
        "docs/architecture/versioning.md",
        "docs/architecture/versioning.ja.md",
        "docs/architecture/versioning.zh-CN.md",
    ] {
        write_file(
            root.path(),
            path,
            "Stable v1.2.3; candidate 1.2.4-rc.2 is prerelease.\n",
        );
    }
    root
}

#[test]
fn validates_candidate_without_promoting_it_to_stable_installation() {
    let root = candidate_fixture();
    let report = validate_source(root.path()).expect("strict prerelease candidate");
    assert_eq!(report.version, "1.2.4-rc.2");
    assert_eq!(report.tag, "v1.2.4-rc.2");
    assert_eq!(report.checked_document_count, 12);
}

#[test]
fn rejects_candidate_metadata_drift_and_stable_projection() {
    let root = candidate_fixture();
    write_file(
        root.path(),
        "docs/release/candidate.json",
        "{\"schemaVersion\":1,\"channel\":\"prerelease\",\"candidateVersion\":\"1.2.4-rc.1\",\"stableBaselineVersion\":\"1.2.3\",\"target\":\"aarch64-apple-darwin\"}\n",
    );
    assert!(
        validate_source(root.path()).is_err(),
        "candidate metadata drift must fail"
    );

    let root = candidate_fixture();
    write_file(
        root.path(),
        "docs/reference/reference-comparison-metadata.json",
        "{\"workspaceCandidateVersion\":\"1.2.4-rc.1\"}\n",
    );
    assert!(
        validate_source(root.path()).is_err(),
        "reference candidate drift must fail"
    );

    let root = candidate_fixture();
    write_file(
        root.path(),
        "docs/release/candidate.json",
        "{\"schemaVersion\":1,\"channel\":\"stable\",\"candidateVersion\":\"1.2.4-rc.2\",\"stableBaselineVersion\":\"1.2.3\",\"target\":\"aarch64-apple-darwin\"}\n",
    );
    assert!(
        validate_source(root.path()).is_err(),
        "prerelease metadata cannot claim stable"
    );

    let root = candidate_fixture();
    write_file(
        root.path(),
        "docs/release/distribution.md",
        "Current installation baseline: v1.2.4-rc.2\nai-cockpit-v1.2.4-rc.2-aarch64-apple-darwin\n",
    );
    assert!(
        validate_source(root.path()).is_err(),
        "candidate cannot become stable default"
    );

    let root = candidate_fixture();
    write_file(
        root.path(),
        "Cargo.lock",
        "version = 3\n\n[[package]]\nname = \"cockpit-cli\"\nversion = \"1.2.4-rc.1\"\n",
    );
    assert!(
        validate_source(root.path()).is_err(),
        "locked package drift must fail"
    );
}

#[test]
fn rejects_noncanonical_or_unsupported_candidate_syntax() {
    for version in ["1.2.4-rc.02", "1.2.4+local", "1.2.4-preview.2"] {
        let root = candidate_fixture();
        write_file(
            root.path(),
            "cli/Cargo.toml",
            &format!(
                "[package]\nname = \"cockpit-cli\"\nversion = \"{version}\"\nedition = \"2021\"\n"
            ),
        );
        assert!(
            validate_source(root.path()).is_err(),
            "invalid candidate {version} must fail"
        );
    }
}

#[test]
fn validates_source_version_identity_and_document_baselines() {
    let root = fixture();
    let report = validate_source(root.path()).unwrap();
    assert_eq!(report.version, "1.2.3");
    assert_eq!(report.tag, "v1.2.3");
    assert_eq!(report.workspace_package_count, 1);
    assert_eq!(report.checked_document_count, 12);
}

#[test]
fn rejects_versioned_operations_baseline() {
    let root = fixture();
    write_file(
        root.path(),
        "docs/operations/README.md",
        "Current operator target: x86_64-unknown-linux-gnu\nSee v1.2.3 for details.\n",
    );
    let error = validate_source(root.path()).unwrap_err().to_string();
    assert!(error.contains("operations/README.md"));
    assert!(error.contains("hard-codes a release version"));
}

#[test]
fn rejects_stale_current_baseline() {
    let root = fixture();
    write_file(
        root.path(),
        "docs/release/distribution.md",
        "Current installation baseline: v1.2.2\nai-cockpit-v1.2.3-x86_64-unknown-linux-gnu\n",
    );
    let error = validate_source(root.path()).unwrap_err().to_string();
    assert!(error.contains("stale current baseline"), "{error}");
}
