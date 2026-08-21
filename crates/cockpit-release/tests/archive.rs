use std::{fs, io::Write};

use cockpit_release::archive::{ArchiveTarget, PackageInput, inspect_archive, package_archive};

fn fixture() -> (tempfile::TempDir, PackageInput) {
    let dir = tempfile::tempdir().unwrap();
    let executable = dir.path().join("source-binary");
    let license = dir.path().join("LICENSE");
    let readme = dir.path().join("ARCHIVE-README");
    fs::write(&executable, b"binary").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fs::write(&license, b"MIT License\n").unwrap();
    fs::write(&readme, b"ai-cockpit archive\n").unwrap();
    (
        dir,
        PackageInput {
            executable,
            license,
            readme,
            target: ArchiveTarget::from_rust_target("aarch64-apple-darwin").unwrap(),
        },
    )
}

#[test]
fn tar_archive_has_stable_exact_members() {
    let (dir, mut input) = fixture();
    input.target = ArchiveTarget::from_rust_target("aarch64-apple-darwin").unwrap();
    let first = dir.path().join("first.tar.gz");
    let second = dir.path().join("second.tar.gz");
    let first_record = package_archive(&input, &first).unwrap();
    let second_record = package_archive(&input, &second).unwrap();
    assert_eq!(first_record.bytes, second_record.bytes);
    assert_eq!(first_record.sha256, second_record.sha256);
    let inspection = inspect_archive(&first, input.target).unwrap();
    assert_eq!(
        inspection.members,
        vec![
            "LICENSE".to_string(),
            "README".to_string(),
            "ai-cockpit".to_string()
        ]
    );
}

#[test]
fn windows_zip_has_stable_exact_members() {
    let (dir, mut input) = fixture();
    input.target = ArchiveTarget::from_rust_target("x86_64-pc-windows-msvc").unwrap();
    let first = dir.path().join("first.zip");
    let second = dir.path().join("second.zip");
    let first_record = package_archive(&input, &first).unwrap();
    let second_record = package_archive(&input, &second).unwrap();
    assert_eq!(first_record.sha256, second_record.sha256);
    let inspection = inspect_archive(&first, input.target).unwrap();
    assert_eq!(
        inspection.members,
        vec![
            "LICENSE".to_string(),
            "README".to_string(),
            "ai-cockpit.exe".to_string()
        ]
    );
}

#[test]
fn archive_with_extra_zip_member_is_rejected() {
    let (dir, mut input) = fixture();
    input.target = ArchiveTarget::from_rust_target("x86_64-pc-windows-msvc").unwrap();
    let archive = dir.path().join("bad.zip");
    let file = fs::File::create(&archive).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    writer.start_file("ai-cockpit.exe", options).unwrap();
    writer.write_all(b"binary").unwrap();
    writer.start_file("LICENSE", options).unwrap();
    writer.write_all(b"MIT License\n").unwrap();
    writer.start_file("README", options).unwrap();
    writer.write_all(b"readme\n").unwrap();
    writer.start_file("unexpected", options).unwrap();
    writer.write_all(b"bad\n").unwrap();
    writer.finish().unwrap();
    let error = inspect_archive(&archive, input.target).expect_err("extra member must fail");
    assert!(error.to_string().contains("member"));
}

#[test]
fn zip_symlink_member_is_rejected() {
    let (dir, mut input) = fixture();
    input.target = ArchiveTarget::from_rust_target("x86_64-pc-windows-msvc").unwrap();
    let archive = dir.path().join("symlink.zip");
    let file = fs::File::create(&archive).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    let regular = zip::write::SimpleFileOptions::default().unix_permissions(0o100644);
    let symlink = zip::write::SimpleFileOptions::default().unix_permissions(0o120777);
    writer.start_file("LICENSE", regular).unwrap();
    writer.write_all(b"MIT License\n").unwrap();
    writer.start_file("README", regular).unwrap();
    writer.write_all(b"readme\n").unwrap();
    writer
        .add_symlink("ai-cockpit.exe", "other-target", symlink)
        .unwrap();
    writer.finish().unwrap();
    let error = inspect_archive(&archive, input.target).expect_err("symlink must fail");
    assert!(error.to_string().contains("symlink") || error.to_string().contains("special"));
}

#[cfg(unix)]
#[test]
fn unix_archive_requires_executable_mode() {
    use std::os::unix::fs::PermissionsExt;

    let (dir, mut input) = fixture();
    input.target = ArchiveTarget::from_rust_target("x86_64-unknown-linux-gnu").unwrap();
    fs::set_permissions(&input.executable, fs::Permissions::from_mode(0o644)).unwrap();
    let archive = dir.path().join("non-executable.tar.gz");
    let error = package_archive(&input, &archive).expect_err("non-executable input must fail");
    assert!(error.to_string().contains("executable"));
}

#[test]
fn target_mapping_rejects_unknown_triple() {
    let error = ArchiveTarget::from_rust_target("wasm32-unknown-unknown")
        .expect_err("unknown target must fail");
    assert!(error.to_string().contains("unsupported target"));
}
