//! Offline parsing only. Every returned byte/claim remains untrusted.
//! No network, filesystem extraction, store, command execution or acceptance.

use cockpit_core::Digest;
use cockpit_protocol::ci_admission::UntrustedGithubNativeEnvelope;
use cockpit_verification::VerificationReceipt;
use thiserror::Error;

pub const MAX_GITHUB_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_GITHUB_EXPANDED_BYTES: u64 = 128 * 1024 * 1024;
pub const MAX_GITHUB_ARCHIVE_ENTRIES: usize = 64;
pub const MAX_GITHUB_ARCHIVE_ENTRY_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_GITHUB_EXPANSION_RATIO: u64 = 100;
pub const MAX_GITHUB_NODE_CLAIMS: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum GithubArchiveError {
    #[error("archive raw byte limit exceeded")]
    RawSize,
    #[error("archive digest mismatch")]
    ArchiveDigest,
    #[error("invalid or truncated ZIP structure")]
    ZipStructure,
    #[error("archive directory entry limit exceeded")]
    DirectoryLimit,
    #[error("archive directory or member offset is out of bounds")]
    DirectoryBounds,
    #[error("unsupported or encrypted archive format")]
    UnsupportedFormat,
    #[error("noncanonical archive path or path alias")]
    Path,
    #[error("duplicate archive member")]
    DuplicateEntry,
    #[error("archive member is not a regular file")]
    SpecialFile,
    #[error("expanded archive or member size limit exceeded")]
    ExpandedSize,
    #[error("archive expansion ratio limit exceeded")]
    ExpansionRatio,
    #[error("archive checksum or compressed stream mismatch")]
    Checksum,
    #[error("strict manifest or exact inventory is invalid")]
    Manifest,
    #[error("manifest member byte count or digest mismatch")]
    EntryDigest,
    #[error("raw native receipt structure or accounting is invalid")]
    NativeReceipt,
    #[error("archive node claim limit exceeded")]
    NodeLimit,
    #[error("bounded archive parser is not implemented (RED scaffold)")]
    NotImplemented,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UntrustedGithubArchiveMember {
    pub path: String,
    pub digest: Digest,
    /// Exact original member bytes. Never execute or deserialize as authority.
    pub raw: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UntrustedGithubArchiveInspection {
    pub archive_digest: Digest,
    pub members: Vec<UntrustedGithubArchiveMember>,
    pub receipts: Vec<UntrustedGithubNativeEnvelope<VerificationReceipt>>,
    /// Missing native coverage is a gap, not proof of complete Contract checks.
    /// Provider/trust/uploader/producer/consumer proofs remain unavailable.
    pub gaps: Vec<String>,
}

/// Accepts only caller-owned bounded bytes and an untrusted expected digest.
/// Matching the digest or decoding the archive confers no GitHub authority.
pub fn inspect_untrusted_github_archive(
    raw: &[u8],
    expected_archive_digest: &Digest,
) -> Result<UntrustedGithubArchiveInspection, GithubArchiveError> {
    inspect_with_decoder(raw, expected_archive_digest, decode_checked_zip)
}

pub(super) fn check_raw_archive_size(bytes: u64) -> Result<(), GithubArchiveError> {
    if bytes > MAX_GITHUB_ARCHIVE_BYTES {
        Err(GithubArchiveError::RawSize)
    } else {
        Ok(())
    }
}

// GREEN will fill this bounded index only after checking the classic ZIP
// directory count/offsets/sizes without invoking ZipArchive or allocating from
// hostile declared counts. It deliberately has no "authenticated" property.
pub(super) struct CheckedDirectory;

pub(super) fn inspect_with_decoder<F>(
    raw: &[u8],
    expected_archive_digest: &Digest,
    decoder: F,
) -> Result<UntrustedGithubArchiveInspection, GithubArchiveError>
where
    F: FnOnce(
        &[u8],
        CheckedDirectory,
    ) -> Result<UntrustedGithubArchiveInspection, GithubArchiveError>,
{
    check_raw_archive_size(raw.len() as u64)?;
    if Digest::sha256_bytes(raw) != *expected_archive_digest {
        return Err(GithubArchiveError::ArchiveDigest);
    }
    let checked = preflight_directory(raw)?;
    decoder(raw, checked)
}

fn preflight_directory(_raw: &[u8]) -> Result<CheckedDirectory, GithubArchiveError> {
    // Conservative scaffold: no archive, claim or positive proof is accepted.
    Err(GithubArchiveError::NotImplemented)
}

fn decode_checked_zip(
    _raw: &[u8],
    _checked: CheckedDirectory,
) -> Result<UntrustedGithubArchiveInspection, GithubArchiveError> {
    Err(GithubArchiveError::NotImplemented)
}
