use std::{fs, path::Path};

use serde_json::{Map, Value, json};

use crate::{
    archive::{ArchiveTarget, archive_executable_sha256},
    error::ReleaseError,
    manifest::sha256_file,
};

pub const RELEASE_ARCHIVE_SPDX_ID: &str = "SPDXRef-AiCockpitReleaseArchive";
pub const RELEASE_BINARY_SPDX_ID: &str = "SPDXRef-AiCockpitReleaseBinary";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SbomBinding {
    pub archive_filename: String,
    pub archive_sha256: String,
    pub binary_filename: String,
    pub binary_sha256: String,
}

pub fn bind_sbom_file(
    sbom: &Path,
    archive: &Path,
    target: ArchiveTarget,
    version: &str,
) -> Result<SbomBinding, ReleaseError> {
    let expected = expected_binding(sbom, archive, target, version)?;
    let mut document: Value = serde_json::from_slice(&fs::read(sbom)?)?;
    validate_spdx_document_identity(&document)?;
    reject_reserved_ids(&document)?;

    array_mut(&mut document, "packages")?.push(json!({
        "SPDXID": RELEASE_ARCHIVE_SPDX_ID,
        "name": "ai-cockpit release archive",
        "versionInfo": version,
        "packageFileName": expected.archive_filename,
        "downloadLocation": "NOASSERTION",
        "filesAnalyzed": false,
        "checksums": [{
            "algorithm": "SHA256",
            "checksumValue": expected.archive_sha256,
        }],
        "primaryPackagePurpose": "APPLICATION"
    }));
    array_mut(&mut document, "files")?.push(json!({
        "SPDXID": RELEASE_BINARY_SPDX_ID,
        "fileName": expected.binary_filename,
        "checksums": [{
            "algorithm": "SHA256",
            "checksumValue": expected.binary_sha256,
        }],
        "licenseConcluded": "NOASSERTION",
        "copyrightText": "NOASSERTION"
    }));
    array_mut(&mut document, "relationships")?.extend([
        json!({
            "spdxElementId": "SPDXRef-DOCUMENT",
            "relationshipType": "DESCRIBES",
            "relatedSpdxElement": RELEASE_ARCHIVE_SPDX_ID
        }),
        json!({
            "spdxElementId": RELEASE_ARCHIVE_SPDX_ID,
            "relationshipType": "CONTAINS",
            "relatedSpdxElement": RELEASE_BINARY_SPDX_ID
        }),
    ]);

    validate_document_binding(&document, &expected, version)?;
    let mut bytes = serde_json::to_vec_pretty(&document)?;
    bytes.push(b'\n');
    fs::write(sbom, bytes)?;
    Ok(expected)
}

pub fn validate_sbom_binding(
    sbom: &Path,
    archive: &Path,
    target: ArchiveTarget,
    version: &str,
) -> Result<SbomBinding, ReleaseError> {
    let expected = expected_binding(sbom, archive, target, version)?;
    let document: Value = serde_json::from_slice(&fs::read(sbom)?)?;
    validate_document_binding(&document, &expected, version)?;
    Ok(expected)
}

fn expected_binding(
    sbom: &Path,
    archive: &Path,
    target: ArchiveTarget,
    version: &str,
) -> Result<SbomBinding, ReleaseError> {
    let parsed = semver::Version::parse(version)
        .map_err(|error| ReleaseError::Invalid(format!("invalid release version: {error}")))?;
    if parsed.to_string() != version {
        return Err(ReleaseError::Invalid(
            "release version must be canonical semver".into(),
        ));
    }
    let extension = match target.kind {
        crate::archive::ArchiveKind::TarGz => "tar.gz",
        crate::archive::ArchiveKind::Zip => "zip",
    };
    let archive_filename = archive
        .file_name()
        .and_then(|filename| filename.to_str())
        .ok_or_else(|| ReleaseError::Invalid("archive must have a UTF-8 filename".into()))?;
    let expected_archive = format!("ai-cockpit-v{version}-{}.{extension}", target.rust_target);
    if archive_filename != expected_archive {
        return Err(ReleaseError::Invalid(format!(
            "archive filename must be {expected_archive} for the target and version"
        )));
    }
    let sbom_filename = sbom
        .file_name()
        .and_then(|filename| filename.to_str())
        .ok_or_else(|| ReleaseError::Invalid("SBOM must have a UTF-8 filename".into()))?;
    let expected_sbom = format!("ai-cockpit-v{version}-{}.spdx.json", target.rust_target);
    if sbom_filename != expected_sbom {
        return Err(ReleaseError::Invalid(format!(
            "SBOM filename must be {expected_sbom} for the target and version"
        )));
    }
    let archive_sha256 = sha256_file(archive)?;
    let binary_sha256 = archive_executable_sha256(archive, target)?;
    reject_zero_digest(&archive_sha256, "archive")?;
    reject_zero_digest(&binary_sha256, "binary")?;
    Ok(SbomBinding {
        archive_filename: archive_filename.into(),
        archive_sha256,
        binary_filename: target.executable_name.into(),
        binary_sha256,
    })
}

fn validate_document_binding(
    document: &Value,
    expected: &SbomBinding,
    version: &str,
) -> Result<(), ReleaseError> {
    validate_spdx_document_identity(document)?;
    let package = exactly_one_by_id(document, "packages", RELEASE_ARCHIVE_SPDX_ID)
        .map_err(|_| invalid("SBOM must contain exactly one release archive Package"))?;
    require_string(
        package,
        "name",
        "ai-cockpit release archive",
        "archive package name",
    )?;
    require_string(
        package,
        "packageFileName",
        &expected.archive_filename,
        "archive filename",
    )?;
    require_string(package, "versionInfo", version, "archive version")?;
    require_string(
        package,
        "downloadLocation",
        "NOASSERTION",
        "archive download location",
    )?;
    if package.get("filesAnalyzed") != Some(&Value::Bool(false)) {
        return Err(invalid(
            "release archive Package must set filesAnalyzed=false",
        ));
    }
    require_string(
        package,
        "primaryPackagePurpose",
        "APPLICATION",
        "archive package purpose",
    )?;
    validate_sha256(package, &expected.archive_sha256, "archive")?;

    let file = exactly_one_by_id(document, "files", RELEASE_BINARY_SPDX_ID)
        .map_err(|_| invalid("SBOM must contain exactly one release binary File"))?;
    require_string(
        file,
        "fileName",
        &expected.binary_filename,
        "binary filename",
    )?;
    require_string(
        file,
        "licenseConcluded",
        "NOASSERTION",
        "binary license conclusion",
    )?;
    require_string(
        file,
        "copyrightText",
        "NOASSERTION",
        "binary copyright text",
    )?;
    validate_sha256(file, &expected.binary_sha256, "binary")?;

    exactly_one_relationship(
        document,
        "SPDXRef-DOCUMENT",
        "DESCRIBES",
        RELEASE_ARCHIVE_SPDX_ID,
    )?;
    exactly_one_relationship(
        document,
        RELEASE_ARCHIVE_SPDX_ID,
        "CONTAINS",
        RELEASE_BINARY_SPDX_ID,
    )?;
    Ok(())
}

fn validate_spdx_document_identity(document: &Value) -> Result<(), ReleaseError> {
    let object = document
        .as_object()
        .ok_or_else(|| invalid("SPDX document must be a JSON object"))?;
    require_string(object, "spdxVersion", "SPDX-2.3", "SPDX version")?;
    require_string(
        object,
        "SPDXID",
        "SPDXRef-DOCUMENT",
        "SPDX document identifier",
    )?;
    Ok(())
}

fn reject_reserved_ids(document: &Value) -> Result<(), ReleaseError> {
    for key in ["packages", "files"] {
        for element in array(document, key)? {
            if matches!(
                element.get("SPDXID").and_then(Value::as_str),
                Some(RELEASE_ARCHIVE_SPDX_ID | RELEASE_BINARY_SPDX_ID)
            ) {
                return Err(invalid(
                    "source SBOM already uses a reserved release SPDXID",
                ));
            }
        }
    }
    Ok(())
}

fn exactly_one_by_id<'a>(
    document: &'a Value,
    key: &str,
    spdx_id: &str,
) -> Result<&'a Map<String, Value>, ReleaseError> {
    let matches = array(document, key)?
        .iter()
        .filter(|element| element.get("SPDXID").and_then(Value::as_str) == Some(spdx_id))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(invalid("reserved SPDX element cardinality mismatch"));
    }
    matches[0]
        .as_object()
        .ok_or_else(|| invalid("reserved SPDX element must be an object"))
}

fn exactly_one_relationship(
    document: &Value,
    source: &str,
    relationship_type: &str,
    related: &str,
) -> Result<(), ReleaseError> {
    let count = array(document, "relationships")?
        .iter()
        .filter(|relationship| {
            relationship.get("spdxElementId").and_then(Value::as_str) == Some(source)
                && relationship.get("relationshipType").and_then(Value::as_str)
                    == Some(relationship_type)
                && relationship
                    .get("relatedSpdxElement")
                    .and_then(Value::as_str)
                    == Some(related)
        })
        .count();
    if count != 1 {
        return Err(invalid(format!(
            "SBOM must contain exactly one {relationship_type} release binding"
        )));
    }
    Ok(())
}

fn validate_sha256(
    element: &Map<String, Value>,
    expected: &str,
    label: &str,
) -> Result<(), ReleaseError> {
    let checksums = element
        .get("checksums")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(format!("{label} checksums must be an array")))?;
    let sha256 = checksums
        .iter()
        .filter(|checksum| checksum.get("algorithm").and_then(Value::as_str) == Some("SHA256"))
        .collect::<Vec<_>>();
    if sha256.len() != 1 {
        return Err(invalid(format!(
            "{label} must contain exactly one SHA256 checksum"
        )));
    }
    let actual = sha256[0]
        .get("checksumValue")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid(format!("{label} SHA256 checksum must be a string")))?;
    reject_zero_digest(actual, label)?;
    if actual != expected {
        return Err(invalid(format!(
            "{label} digest does not match staged bytes"
        )));
    }
    Ok(())
}

fn reject_zero_digest(digest: &str, label: &str) -> Result<(), ReleaseError> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || digest.bytes().all(|byte| byte == b'0')
    {
        return Err(invalid(format!("invalid {label} digest")));
    }
    Ok(())
}

fn require_string(
    object: &Map<String, Value>,
    key: &str,
    expected: &str,
    label: &str,
) -> Result<(), ReleaseError> {
    if object.get(key).and_then(Value::as_str) != Some(expected) {
        return Err(invalid(format!("{label} mismatch")));
    }
    Ok(())
}

fn array<'a>(document: &'a Value, key: &str) -> Result<&'a Vec<Value>, ReleaseError> {
    document
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(format!("SPDX {key} must be an array")))
}

fn array_mut<'a>(document: &'a mut Value, key: &str) -> Result<&'a mut Vec<Value>, ReleaseError> {
    let object = document
        .as_object_mut()
        .ok_or_else(|| invalid("SPDX document must be a JSON object"))?;
    if !object.contains_key(key) {
        object.insert(key.into(), Value::Array(Vec::new()));
    }
    object
        .get_mut(key)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| invalid(format!("SPDX {key} must be an array")))
}

fn invalid(message: impl Into<String>) -> ReleaseError {
    ReleaseError::Invalid(message.into())
}
