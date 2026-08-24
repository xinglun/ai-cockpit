use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use tar::{Archive, Builder, Header};
use zip::{
    CompressionMethod, DateTime,
    read::ZipArchive,
    write::{SimpleFileOptions, ZipWriter},
};

use crate::{error::ReleaseError, manifest::sha256_file};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveKind {
    TarGz,
    Zip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchiveTarget {
    pub rust_target: &'static str,
    pub kind: ArchiveKind,
    pub executable_name: &'static str,
}

impl ArchiveTarget {
    pub fn from_rust_target(target: &str) -> Result<Self, ReleaseError> {
        match target {
            "aarch64-apple-darwin"
            | "x86_64-apple-darwin"
            | "aarch64-unknown-linux-gnu"
            | "x86_64-unknown-linux-gnu" => Ok(Self {
                rust_target: match target {
                    "aarch64-apple-darwin" => "aarch64-apple-darwin",
                    "x86_64-apple-darwin" => "x86_64-apple-darwin",
                    "aarch64-unknown-linux-gnu" => "aarch64-unknown-linux-gnu",
                    _ => "x86_64-unknown-linux-gnu",
                },
                kind: ArchiveKind::TarGz,
                executable_name: "ai-cockpit",
            }),
            "x86_64-pc-windows-msvc" => Ok(Self {
                rust_target: "x86_64-pc-windows-msvc",
                kind: ArchiveKind::Zip,
                executable_name: "ai-cockpit.exe",
            }),
            other => Err(ReleaseError::Invalid(format!("unsupported target {other}"))),
        }
    }

    pub fn archive_filename(self) -> String {
        let extension = match self.kind {
            ArchiveKind::TarGz => "tar.gz",
            ArchiveKind::Zip => "zip",
        };
        format!("ai-cockpit-{}.{}", self.rust_target, extension)
    }
}

#[derive(Clone, Debug)]
pub struct PackageInput {
    pub executable: PathBuf,
    pub license: PathBuf,
    pub readme: PathBuf,
    pub target: ArchiveTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveRecord {
    pub filename: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveInspection {
    pub members: Vec<String>,
}

pub fn package_archive(input: &PackageInput, output: &Path) -> Result<ArchiveRecord, ReleaseError> {
    let executable_metadata = fs::metadata(&input.executable)?;
    if !executable_metadata.is_file() {
        return Err(ReleaseError::Invalid(
            "executable input is not a regular file".into(),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if executable_metadata.permissions().mode() & 0o111 == 0 {
            return Err(ReleaseError::Invalid(
                "executable input has no executable mode".into(),
            ));
        }
    }
    for path in [&input.license, &input.readme] {
        if !fs::metadata(path)?.is_file() {
            return Err(ReleaseError::Invalid(format!(
                "archive input is not a regular file: {}",
                path.display()
            )));
        }
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    match input.target.kind {
        ArchiveKind::TarGz => write_tar(input, output)?,
        ArchiveKind::Zip => write_zip(input, output)?,
    }
    let bytes = fs::metadata(output)?.len();
    let sha256 = sha256_file(output)?;
    let filename = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ReleaseError::Invalid("archive output must have a UTF-8 filename".into()))?;
    if !filename.ends_with(match input.target.kind {
        ArchiveKind::TarGz => ".tar.gz",
        ArchiveKind::Zip => ".zip",
    }) {
        return Err(ReleaseError::Invalid(
            "archive output extension does not match target".into(),
        ));
    }
    Ok(ArchiveRecord {
        filename: filename.to_string(),
        bytes,
        sha256,
    })
}

fn write_tar(input: &PackageInput, output: &Path) -> Result<(), ReleaseError> {
    let file = File::create(output)?;
    let encoder = GzEncoder::new(file, Compression::best());
    let mut builder = Builder::new(encoder);
    append_tar_file(&mut builder, "LICENSE", &input.license, 0o644)?;
    append_tar_file(&mut builder, "README", &input.readme, 0o644)?;
    append_tar_file(
        &mut builder,
        input.target.executable_name,
        &input.executable,
        0o755,
    )?;
    let encoder = builder.into_inner()?;
    encoder.finish()?.sync_all().map_err(ReleaseError::Io)?;
    Ok(())
}

fn append_tar_file(
    builder: &mut Builder<GzEncoder<File>>,
    name: &str,
    path: &Path,
    mode: u32,
) -> Result<(), ReleaseError> {
    let data = fs::read(path)?;
    let mut header = Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(mode);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_cksum();
    builder.append_data(&mut header, name, data.as_slice())?;
    Ok(())
}

fn write_zip(input: &PackageInput, output: &Path) -> Result<(), ReleaseError> {
    let file = File::create(output)?;
    let mut writer = ZipWriter::new(file);
    let common = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(DateTime::default());
    append_zip_file(
        &mut writer,
        "LICENSE",
        &input.license,
        common.unix_permissions(0o100644),
    )?;
    append_zip_file(
        &mut writer,
        "README",
        &input.readme,
        common.unix_permissions(0o100644),
    )?;
    append_zip_file(
        &mut writer,
        input.target.executable_name,
        &input.executable,
        common.unix_permissions(0o100755),
    )?;
    writer.finish()?.sync_all()?;
    Ok(())
}

fn append_zip_file<W: Write + std::io::Seek>(
    writer: &mut ZipWriter<W>,
    name: &str,
    path: &Path,
    options: SimpleFileOptions,
) -> Result<(), ReleaseError> {
    writer.start_file(name, options)?;
    let mut file = File::open(path)?;
    std::io::copy(&mut file, writer)?;
    Ok(())
}

pub fn inspect_archive(
    path: &Path,
    target: ArchiveTarget,
) -> Result<ArchiveInspection, ReleaseError> {
    let members = match target.kind {
        ArchiveKind::TarGz => inspect_tar(path)?,
        ArchiveKind::Zip => inspect_zip(path)?,
    };
    let expected = vec![
        "LICENSE".to_string(),
        "README".to_string(),
        target.executable_name.to_string(),
    ];
    if members != expected {
        return Err(ReleaseError::Invalid(format!(
            "archive member set mismatch: {:?}",
            members
        )));
    }
    Ok(ArchiveInspection { members })
}

pub fn archive_executable_sha256(
    path: &Path,
    target: ArchiveTarget,
) -> Result<String, ReleaseError> {
    inspect_archive(path, target)?;
    match target.kind {
        ArchiveKind::TarGz => {
            let file = File::open(path)?;
            let decoder = GzDecoder::new(file);
            let mut archive = Archive::new(decoder);
            for entry in archive.entries()? {
                let mut entry = entry?;
                let entry_path = entry.path()?;
                if safe_member_name(entry_path.as_ref())? == target.executable_name {
                    return sha256_reader(&mut entry);
                }
            }
        }
        ArchiveKind::Zip => {
            let file = File::open(path)?;
            let mut archive = ZipArchive::new(file)?;
            for index in 0..archive.len() {
                let mut entry = archive.by_index(index)?;
                if safe_member_name(Path::new(entry.name()))? == target.executable_name {
                    return sha256_reader(&mut entry);
                }
            }
        }
    }
    Err(ReleaseError::Invalid(
        "archive does not contain the target executable".into(),
    ))
}

fn sha256_reader(reader: &mut impl Read) -> Result<String, ReleaseError> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn inspect_tar(path: &Path) -> Result<Vec<String>, ReleaseError> {
    let file = File::open(path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let mut members = Vec::new();
    for entry in archive.entries()? {
        let entry = entry?;
        if !entry.header().entry_type().is_file() {
            return Err(ReleaseError::Invalid(
                "archive member is not a regular file".into(),
            ));
        }
        let path = entry.path()?.to_path_buf();
        let name = safe_member_name(&path)?;
        if members.iter().any(|member| member == &name) {
            return Err(ReleaseError::Invalid("duplicate archive member".into()));
        }
        if name == "ai-cockpit" {
            #[cfg(unix)]
            if entry.header().mode()? & 0o111 == 0 {
                return Err(ReleaseError::Invalid(
                    "Unix executable member has no executable mode".into(),
                ));
            }
        }
        members.push(name);
    }
    Ok(members)
}

fn inspect_zip(path: &Path) -> Result<Vec<String>, ReleaseError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut members = Vec::new();
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        if entry.is_dir() {
            return Err(ReleaseError::Invalid("archive contains a directory".into()));
        }
        let name = safe_member_name(Path::new(entry.name()))?;
        if members.iter().any(|member| member == &name) {
            return Err(ReleaseError::Invalid("duplicate archive member".into()));
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 != 0 && mode & 0o170000 != 0o100000)
        {
            return Err(ReleaseError::Invalid(
                "archive member is a symlink or special file".into(),
            ));
        }
        if name == "ai-cockpit.exe" && entry.unix_mode().is_some_and(|mode| mode & 0o111 == 0) {
            return Err(ReleaseError::Invalid(
                "Windows executable member has no executable mode".into(),
            ));
        }
        members.push(name);
    }
    Ok(members)
}

fn safe_member_name(path: &Path) -> Result<String, ReleaseError> {
    let mut components = path.components();
    let component = components.next();
    if components.next().is_some() || !matches!(component, Some(Component::Normal(_))) {
        return Err(ReleaseError::Invalid(
            "archive member path traversal".into(),
        ));
    }
    let name = component
        .unwrap()
        .as_os_str()
        .to_str()
        .ok_or_else(|| ReleaseError::Invalid("archive member is not UTF-8".into()))?;
    Ok(name.to_string())
}
