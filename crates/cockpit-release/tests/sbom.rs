use std::{fs, path::PathBuf};

use cockpit_release::{
    archive::{ArchiveTarget, PackageInput, package_archive},
    sbom::{
        RELEASE_ARCHIVE_SPDX_ID, RELEASE_BINARY_SPDX_ID, bind_sbom_file, validate_sbom_binding,
    },
};

fn spdx_skeleton() -> serde_json::Value {
    serde_json::json!({
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": "source dependency scan",
        "documentNamespace": "https://example.invalid/source-scan",
        "creationInfo": {
            "created": "2026-08-24T00:00:00Z",
            "creators": ["Tool: test"]
        },
        "packages": [],
        "files": [],
        "relationships": []
    })
}

struct BoundFixture {
    _dir: tempfile::TempDir,
    archive: PathBuf,
    sbom: PathBuf,
    target: ArchiveTarget,
}

fn bound_fixture(rust_target: &str) -> BoundFixture {
    let dir = tempfile::tempdir().unwrap();
    let target = ArchiveTarget::from_rust_target(rust_target).unwrap();
    let executable = dir.path().join(target.executable_name);
    let license = dir.path().join("LICENSE");
    let readme = dir.path().join("README");
    fs::write(&executable, b"exact release executable bytes").unwrap();
    fs::write(&license, b"MIT License\n").unwrap();
    fs::write(&readme, b"README\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let extension = if rust_target == "x86_64-pc-windows-msvc" {
        "zip"
    } else {
        "tar.gz"
    };
    let archive = dir
        .path()
        .join(format!("ai-cockpit-v0.1.0-{rust_target}.{extension}"));
    package_archive(
        &PackageInput {
            executable,
            license,
            readme,
            target,
        },
        &archive,
    )
    .unwrap();
    let sbom = dir
        .path()
        .join(format!("ai-cockpit-v0.1.0-{rust_target}.spdx.json"));
    fs::write(&sbom, serde_json::to_vec(&spdx_skeleton()).unwrap()).unwrap();
    bind_sbom_file(&sbom, &archive, target, "0.1.0").unwrap();
    BoundFixture {
        _dir: dir,
        archive,
        sbom,
        target,
    }
}

#[test]
fn tar_and_zip_sboms_bind_the_exact_archive_and_binary_with_standard_spdx_edges() {
    for rust_target in ["aarch64-apple-darwin", "x86_64-pc-windows-msvc"] {
        let fixture = bound_fixture(rust_target);
        let binding =
            validate_sbom_binding(&fixture.sbom, &fixture.archive, fixture.target, "0.1.0")
                .unwrap();
        assert_eq!(
            binding.archive_filename,
            fixture.archive.file_name().unwrap().to_str().unwrap()
        );
        assert_eq!(binding.binary_filename, fixture.target.executable_name);
        assert_ne!(binding.archive_sha256, "0".repeat(64));
        assert_ne!(binding.binary_sha256, "0".repeat(64));

        let document: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.sbom).unwrap()).unwrap();
        let relationships = document["relationships"].as_array().unwrap();
        assert!(relationships.iter().any(|relationship| {
            relationship["spdxElementId"] == "SPDXRef-DOCUMENT"
                && relationship["relationshipType"] == "DESCRIBES"
                && relationship["relatedSpdxElement"] == RELEASE_ARCHIVE_SPDX_ID
        }));
        assert!(relationships.iter().any(|relationship| {
            relationship["spdxElementId"] == RELEASE_ARCHIVE_SPDX_ID
                && relationship["relationshipType"] == "CONTAINS"
                && relationship["relatedSpdxElement"] == RELEASE_BINARY_SPDX_ID
        }));
    }
}

fn mutate_and_validate(
    fixture: &BoundFixture,
    mutate: impl FnOnce(&mut serde_json::Value),
) -> String {
    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.sbom).unwrap()).unwrap();
    mutate(&mut document);
    fs::write(&fixture.sbom, serde_json::to_vec(&document).unwrap()).unwrap();
    validate_sbom_binding(&fixture.sbom, &fixture.archive, fixture.target, "0.1.0")
        .expect_err("invalid binding must fail closed")
        .to_string()
}

#[test]
fn missing_duplicate_zero_and_mismatched_sbom_bindings_fail_closed() {
    let fixture = bound_fixture("aarch64-apple-darwin");
    let error = mutate_and_validate(&fixture, |document| {
        document["packages"]
            .as_array_mut()
            .unwrap()
            .retain(|package| package["SPDXID"] != RELEASE_ARCHIVE_SPDX_ID);
    });
    assert!(error.contains("release archive Package"), "{error}");

    let fixture = bound_fixture("aarch64-apple-darwin");
    let error = mutate_and_validate(&fixture, |document| {
        let packages = document["packages"].as_array_mut().unwrap();
        let package = packages
            .iter()
            .find(|package| package["SPDXID"] == RELEASE_ARCHIVE_SPDX_ID)
            .unwrap()
            .clone();
        packages.push(package);
    });
    assert!(error.contains("release archive Package"), "{error}");

    let fixture = bound_fixture("aarch64-apple-darwin");
    let error = mutate_and_validate(&fixture, |document| {
        let package = document["packages"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|package| package["SPDXID"] == RELEASE_ARCHIVE_SPDX_ID)
            .unwrap();
        package["checksums"][0]["checksumValue"] = serde_json::Value::String("0".repeat(64));
    });
    assert!(error.contains("archive digest"), "{error}");

    let fixture = bound_fixture("aarch64-apple-darwin");
    let error = mutate_and_validate(&fixture, |document| {
        let file = document["files"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|file| file["SPDXID"] == RELEASE_BINARY_SPDX_ID)
            .unwrap();
        file["fileName"] = serde_json::Value::String("wrong-binary".into());
    });
    assert!(error.contains("binary filename"), "{error}");

    let fixture = bound_fixture("aarch64-apple-darwin");
    let error = mutate_and_validate(&fixture, |document| {
        document["relationships"]
            .as_array_mut()
            .unwrap()
            .retain(|relationship| relationship["relationshipType"] != "CONTAINS");
    });
    assert!(error.contains("CONTAINS"), "{error}");
}

#[test]
fn validator_rejects_a_wrong_target_or_version() {
    let fixture = bound_fixture("aarch64-apple-darwin");
    let wrong_target = ArchiveTarget::from_rust_target("x86_64-apple-darwin").unwrap();
    assert!(validate_sbom_binding(&fixture.sbom, &fixture.archive, wrong_target, "0.1.0").is_err());
    assert!(
        validate_sbom_binding(&fixture.sbom, &fixture.archive, fixture.target, "0.1.1").is_err()
    );

    let wrong_name = fixture.sbom.with_file_name("ai-cockpit-build.spdx.json");
    fs::rename(&fixture.sbom, &wrong_name).unwrap();
    assert!(validate_sbom_binding(&wrong_name, &fixture.archive, fixture.target, "0.1.0").is_err());
}
