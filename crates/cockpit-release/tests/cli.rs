use std::{fs, process::Command};

use chrono::{Duration, Utc};
use cockpit_release::handoff::{Destination, HandoffDocument, Issuer, ReleaseBinding};
use sha2::{Digest, Sha256};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_cockpit-release")
}

#[test]
fn help_lists_all_release_boundary_commands() {
    let output = Command::new(binary()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "package",
        "inspect",
        "bind-sbom",
        "validate",
        "manifest",
        "checksums",
        "formula",
        "handoff",
        "validate-handoff",
        "acceptance-plan",
        "acceptance-record",
    ] {
        assert!(
            stdout.contains(command),
            "missing command {command}: {stdout}"
        );
    }
}

#[test]
fn validate_handoff_binds_public_release_and_asset_digests() {
    let dir = tempfile::tempdir().unwrap();
    let handoff_path = dir.path().join("homebrew-handoff.json");
    let manifest = dir.path().join("release-manifest.json");
    let formula = dir.path().join("ai-cockpit.rb");
    let manifest_bytes = b"manifest\n";
    let formula_bytes = b"formula\n";
    fs::write(&manifest, manifest_bytes).unwrap();
    fs::write(&formula, formula_bytes).unwrap();
    let digest = |bytes: &[u8]| hex::encode(Sha256::digest(bytes));
    let commit = "0123456789abcdef0123456789abcdef01234567";
    let now = Utc::now();
    let handoff = HandoffDocument::new(
        Issuer {
            repository: "xinglun/ai-cockpit".into(),
            workflow_ref: format!("xinglun/ai-cockpit/.github/workflows/release.yml@{commit}"),
            run_id: 42,
        },
        Destination {
            repository: "xinglun/homebrew-tap".into(),
            base_ref: "main".into(),
            path: "Formula/ai-cockpit.rb".into(),
        },
        ReleaseBinding {
            tag: "v0.2.91".into(),
            commit: commit.into(),
            provider_release_id: 123,
            manifest_sha256: digest(manifest_bytes),
            formula_sha256: digest(formula_bytes),
        },
        "open_pull_request".into(),
        now,
        now + Duration::hours(1),
    )
    .unwrap();
    fs::write(&handoff_path, handoff.canonical_bytes().unwrap()).unwrap();

    let output = Command::new(binary())
        .args([
            "validate-handoff",
            "--handoff",
            handoff_path.to_str().unwrap(),
            "--tag",
            "v0.2.91",
            "--commit",
            commit,
            "--provider-release-id",
            "123",
            "--manifest-sha256",
            &digest(manifest_bytes),
            "--formula-sha256",
            &digest(formula_bytes),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let changed = Command::new(binary())
        .args([
            "validate-handoff",
            "--handoff",
            handoff_path.to_str().unwrap(),
            "--tag",
            "v0.2.92",
            "--commit",
            commit,
            "--provider-release-id",
            "123",
            "--manifest-sha256",
            &digest(manifest_bytes),
            "--formula-sha256",
            &digest(formula_bytes),
        ])
        .output()
        .unwrap();
    assert!(!changed.status.success());
}

fn identity_json(version: &str, manifest: &str) -> serde_json::Value {
    serde_json::json!({
        "source": {
            "repository": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "commit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "cargoLockDigest": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        },
        "candidate": {
            "version": version,
            "tag": format!("v{version}"),
            "manifestDigest": format!("sha256:{manifest}"),
            "assets": {"archive.tar.gz": "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"}
        },
        "previous": null,
        "runtime": {
            "version": version,
            "digest": "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
        },
        "target": "x86_64-unknown-linux-gnu",
        "isolation": {
            "home": "/tmp/acceptance/home",
            "xdgConfigHome": "/tmp/acceptance/xdg",
            "tmp": "/tmp/acceptance/tmp",
            "cargoHome": "/tmp/acceptance/cargo"
        }
    })
}

#[test]
fn acceptance_commands_persist_resume_and_fail_closed_on_identity_change() {
    let dir = tempfile::tempdir().unwrap();
    let identity = dir.path().join("identity.json");
    let changed_identity = dir.path().join("changed-identity.json");
    let evidence = dir.path().join("evidence.json");
    let receipts = dir.path().join("phase-receipts.json");
    let plan = dir.path().join("plan.json");
    fs::write(
        &identity,
        serde_json::to_vec(&identity_json("0.2.91", &"e".repeat(64))).unwrap(),
    )
    .unwrap();
    fs::write(
        &changed_identity,
        serde_json::to_vec(&identity_json("0.2.92", &"f".repeat(64))).unwrap(),
    )
    .unwrap();
    fs::write(&evidence, b"bound evidence\n").unwrap();

    for (phase, id) in [
        ("prepare", "prepare"),
        ("source_verification_build", "build"),
        ("candidate_acceptance", "candidate"),
        ("publish", "publish"),
    ] {
        let result = Command::new(binary())
            .args([
                "acceptance-record",
                "--identity",
                identity.to_str().unwrap(),
                "--receipts",
                receipts.to_str().unwrap(),
                "--phase",
                phase,
                "--evidence",
                evidence.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "record {id} failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }

    let result = Command::new(binary())
        .args([
            "acceptance-plan",
            "--identity",
            identity.to_str().unwrap(),
            "--receipts",
            receipts.to_str().unwrap(),
            "--output",
            plan.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(result.status.success());
    let plan_json: serde_json::Value = serde_json::from_slice(&fs::read(&plan).unwrap()).unwrap();
    for phase in ["prepare", "source_verification_build", "publish"] {
        assert_eq!(
            plan_json["actions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|action| action["phase"] == phase)
                .unwrap()["action"],
            "reuse"
        );
    }

    let candidate_plan_path = dir.path().join("candidate-plan.json");
    let candidate_result = Command::new(binary())
        .args([
            "acceptance-plan",
            "--scope",
            "candidate",
            "--identity",
            identity.to_str().unwrap(),
            "--receipts",
            receipts.to_str().unwrap(),
            "--output",
            candidate_plan_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(candidate_result.status.success());
    let candidate_plan: serde_json::Value =
        serde_json::from_slice(&fs::read(candidate_plan_path).unwrap()).unwrap();
    assert_eq!(
        candidate_plan["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|action| action["phase"] == "close")
            .unwrap()["action"],
        "run"
    );

    let changed = Command::new(binary())
        .args([
            "acceptance-plan",
            "--identity",
            changed_identity.to_str().unwrap(),
            "--receipts",
            receipts.to_str().unwrap(),
            "--output",
            dir.path().join("changed-plan.json").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!changed.status.success());
    assert!(String::from_utf8_lossy(&changed.stderr).contains("identity"));
}

#[test]
fn package_command_emits_record_and_inspect_command_accepts_archive() {
    let dir = tempfile::tempdir().unwrap();
    let executable = dir.path().join("binary");
    let license = dir.path().join("LICENSE");
    let readme = dir.path().join("README");
    let archive = dir.path().join("archive.tar.gz");
    fs::write(&executable, b"binary").unwrap();
    fs::write(&license, b"MIT License\n").unwrap();
    fs::write(&readme, b"README\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let package = Command::new(binary())
        .args([
            "package",
            "--executable",
            executable.to_str().unwrap(),
            "--license",
            license.to_str().unwrap(),
            "--readme",
            readme.to_str().unwrap(),
            "--target",
            "aarch64-apple-darwin",
            "--output",
            archive.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        package.status.success(),
        "{}",
        String::from_utf8_lossy(&package.stderr)
    );
    let inspect = Command::new(binary())
        .args([
            "inspect",
            "--archive",
            archive.to_str().unwrap(),
            "--target",
            "aarch64-apple-darwin",
        ])
        .output()
        .unwrap();
    assert!(
        inspect.status.success(),
        "{}",
        String::from_utf8_lossy(&inspect.stderr)
    );
}
