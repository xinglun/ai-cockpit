use std::{fs, process::Command};

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
    ] {
        assert!(
            stdout.contains(command),
            "missing command {command}: {stdout}"
        );
    }
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
