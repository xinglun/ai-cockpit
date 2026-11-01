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

// Classic ZIP metadata is read with checked slices before any decoder exists.
// This profile deliberately does not accept ZIP64, self-extracting prefixes,
// archive signatures, non-ASCII paths, encryption or non-regular members.
#[derive(Clone, Debug)]
pub(super) struct CheckedDirectory {
    members: Vec<CheckedMember>,
}

#[derive(Clone, Debug)]
struct CheckedMember {
    name: String,
    method: u16,
    crc: u32,
    expanded: u64,
    compressed: u64,
    data: std::ops::Range<usize>,
    span: std::ops::Range<usize>,
}

#[derive(Clone, Debug)]
struct CentralMember {
    name: std::ops::Range<usize>,
    extra: std::ops::Range<usize>,
    needed: u16,
    flags: u16,
    method: u16,
    crc: u32,
    compressed: u64,
    expanded: u64,
    external_attributes: u32,
    local: usize,
}

fn read_u16(raw: &[u8], offset: usize) -> Result<u16, GithubArchiveError> {
    let end = offset
        .checked_add(2)
        .ok_or(GithubArchiveError::ZipStructure)?;
    let bytes = raw
        .get(offset..end)
        .ok_or(GithubArchiveError::ZipStructure)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32(raw: &[u8], offset: usize) -> Result<u32, GithubArchiveError> {
    let end = offset
        .checked_add(4)
        .ok_or(GithubArchiveError::ZipStructure)?;
    let bytes = raw
        .get(offset..end)
        .ok_or(GithubArchiveError::ZipStructure)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn bounded_end(start: usize, length: usize, limit: usize) -> Result<usize, GithubArchiveError> {
    let end = start
        .checked_add(length)
        .ok_or(GithubArchiveError::DirectoryBounds)?;
    if end > limit {
        return Err(GithubArchiveError::DirectoryBounds);
    }
    Ok(end)
}

pub(super) fn check_declared_budgets(sizes: &[(u64, u64)]) -> Result<(), GithubArchiveError> {
    let mut expanded = 0_u64;
    let mut compressed = 0_u64;
    // Expanded aggregate has priority over ratios and local header checks.
    for &(input, output) in sizes {
        if output > MAX_GITHUB_ARCHIVE_ENTRY_BYTES {
            return Err(GithubArchiveError::ExpandedSize);
        }
        expanded = expanded
            .checked_add(output)
            .ok_or(GithubArchiveError::ExpandedSize)?;
        compressed = compressed
            .checked_add(input)
            .ok_or(GithubArchiveError::ExpansionRatio)?;
        if expanded > MAX_GITHUB_EXPANDED_BYTES {
            return Err(GithubArchiveError::ExpandedSize);
        }
    }
    for &(input, output) in sizes {
        if u128::from(output) > u128::from(input) * u128::from(MAX_GITHUB_EXPANSION_RATIO) {
            return Err(GithubArchiveError::ExpansionRatio);
        }
    }
    if u128::from(expanded) > u128::from(compressed) * u128::from(MAX_GITHUB_EXPANSION_RATIO) {
        return Err(GithubArchiveError::ExpansionRatio);
    }
    Ok(())
}

fn canonical_path(raw: &[u8]) -> Result<String, GithubArchiveError> {
    if raw.is_empty() || raw.len() > 1024 || !raw.is_ascii() || raw[0] == b'/' {
        return Err(GithubArchiveError::Path);
    }
    if raw
        .iter()
        .any(|byte| !(byte.is_ascii_alphanumeric() || b"._-/".contains(byte)))
    {
        return Err(GithubArchiveError::Path);
    }
    let path = std::str::from_utf8(raw).map_err(|_| GithubArchiveError::Path)?;
    for part in path.split('/') {
        if part.is_empty() || part == "." || part == ".." || part.ends_with('.') {
            return Err(GithubArchiveError::Path);
        }
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        {
            return Err(GithubArchiveError::Path);
        }
    }
    if !path.ends_with(".json") && !path.to_ascii_lowercase().ends_with(".json") {
        return Err(GithubArchiveError::Path);
    }
    Ok(path.to_owned())
}

fn validate_extras(raw: &[u8]) -> Result<(), GithubArchiveError> {
    let mut position = 0;
    while position < raw.len() {
        let id = read_u16(raw, position)?;
        let length = usize::from(read_u16(raw, position + 2)?);
        position = bounded_end(position, 4, raw.len())?;
        position = bounded_end(position, length, raw.len())?;
        // ZIP64, Unicode path/comment overrides, AES encryption.
        if matches!(id, 0x0001 | 0x7075 | 0x6375 | 0x9901) {
            return Err(GithubArchiveError::UnsupportedFormat);
        }
    }
    Ok(())
}

fn preflight_directory(raw: &[u8]) -> Result<CheckedDirectory, GithubArchiveError> {
    if raw.len() < 22 {
        return Err(GithubArchiveError::ZipStructure);
    }
    let tail = raw.len().saturating_sub(22 + usize::from(u16::MAX));
    let mut found = None;
    for position in tail..=raw.len() - 22 {
        if raw[position..position + 4] == [0x50, 0x4b, 0x05, 0x06]
            && position.checked_add(22 + usize::from(read_u16(raw, position + 20)?))
                == Some(raw.len())
            && found.replace(position).is_some()
        {
            return Err(GithubArchiveError::ZipStructure);
        }
    }
    let eocd = found.ok_or(GithubArchiveError::ZipStructure)?;
    let count = read_u16(raw, eocd + 10)?;
    let directory_bytes = read_u32(raw, eocd + 12)?;
    let directory_start = read_u32(raw, eocd + 16)?;
    if read_u16(raw, eocd + 4)? != 0
        || read_u16(raw, eocd + 6)? != 0
        || read_u16(raw, eocd + 8)? != count
        || count == u16::MAX
        || directory_bytes == u32::MAX
        || directory_start == u32::MAX
    {
        return Err(GithubArchiveError::UnsupportedFormat);
    }
    if usize::from(count) > MAX_GITHUB_ARCHIVE_ENTRIES {
        return Err(GithubArchiveError::DirectoryLimit);
    }
    let directory_start =
        usize::try_from(directory_start).map_err(|_| GithubArchiveError::DirectoryBounds)?;
    let directory_bytes =
        usize::try_from(directory_bytes).map_err(|_| GithubArchiveError::DirectoryBounds)?;
    let directory_end = bounded_end(directory_start, directory_bytes, eocd)?;
    if directory_end != eocd {
        return Err(GithubArchiveError::DirectoryBounds);
    }
    let mut central = Vec::with_capacity(usize::from(count));
    let mut position = directory_start;
    for _ in 0..count {
        let fixed_end = bounded_end(position, 46, directory_end)?;
        if read_u32(raw, position)? != 0x0201_4b50 {
            return Err(GithubArchiveError::ZipStructure);
        }
        let compressed = read_u32(raw, position + 20)?;
        let expanded = read_u32(raw, position + 24)?;
        let local = read_u32(raw, position + 42)?;
        if compressed == u32::MAX
            || expanded == u32::MAX
            || local == u32::MAX
            || read_u16(raw, position + 34)? != 0
        {
            return Err(GithubArchiveError::UnsupportedFormat);
        }
        let name_end = bounded_end(
            fixed_end,
            usize::from(read_u16(raw, position + 28)?),
            directory_end,
        )?;
        let extra_end = bounded_end(
            name_end,
            usize::from(read_u16(raw, position + 30)?),
            directory_end,
        )?;
        let end = bounded_end(
            extra_end,
            usize::from(read_u16(raw, position + 32)?),
            directory_end,
        )?;
        central.push(CentralMember {
            name: fixed_end..name_end,
            extra: name_end..extra_end,
            needed: read_u16(raw, position + 6)?,
            flags: read_u16(raw, position + 8)?,
            method: read_u16(raw, position + 10)?,
            crc: read_u32(raw, position + 16)?,
            compressed: u64::from(compressed),
            expanded: u64::from(expanded),
            external_attributes: read_u32(raw, position + 38)?,
            local: usize::try_from(local).map_err(|_| GithubArchiveError::DirectoryBounds)?,
        });
        position = end;
    }
    if position != directory_end {
        return Err(GithubArchiveError::ZipStructure);
    }
    check_declared_budgets(
        &central
            .iter()
            .map(|entry| (entry.compressed, entry.expanded))
            .collect::<Vec<_>>(),
    )?;
    let mut members = Vec::with_capacity(central.len());
    let mut names = std::collections::BTreeSet::new();
    for entry in central {
        if entry.needed > 20
            || !matches!(entry.method, 0 | 8)
            || entry.flags & !(0x0800 | 0x0008 | 0x0006) != 0
            || (entry.method == 0 && entry.flags & 0x0006 != 0)
        {
            return Err(GithubArchiveError::UnsupportedFormat);
        }
        let name = canonical_path(&raw[entry.name.clone()])?;
        if !names.insert(name.to_ascii_lowercase()) {
            return Err(GithubArchiveError::DuplicateEntry);
        }
        let file_type = (entry.external_attributes >> 16) & 0o170000;
        if !matches!(file_type, 0 | 0o100000)
            || entry.external_attributes & (0x10 | 0x40 | 0x400) != 0
        {
            return Err(GithubArchiveError::SpecialFile);
        }
        validate_extras(&raw[entry.extra])?;
        let header_end = bounded_end(entry.local, 30, directory_start)?;
        if read_u32(raw, entry.local)? != 0x0403_4b50
            || read_u16(raw, entry.local + 4)? != entry.needed
            || read_u16(raw, entry.local + 6)? != entry.flags
            || read_u16(raw, entry.local + 8)? != entry.method
        {
            return Err(GithubArchiveError::ZipStructure);
        }
        let name_end = bounded_end(
            header_end,
            usize::from(read_u16(raw, entry.local + 26)?),
            directory_start,
        )?;
        let data_start = bounded_end(
            name_end,
            usize::from(read_u16(raw, entry.local + 28)?),
            directory_start,
        )?;
        if raw[header_end..name_end] != raw[entry.name] {
            return Err(GithubArchiveError::ZipStructure);
        }
        validate_extras(&raw[name_end..data_start])?;
        let local_crc = read_u32(raw, entry.local + 14)?;
        let local_compressed = u64::from(read_u32(raw, entry.local + 18)?);
        let local_expanded = u64::from(read_u32(raw, entry.local + 22)?);
        let descriptor = entry.flags & 0x0008 != 0;
        if (!descriptor
            && (local_crc != entry.crc
                || local_compressed != entry.compressed
                || local_expanded != entry.expanded))
            || (descriptor
                && ((local_crc != 0 && local_crc != entry.crc)
                    || (local_compressed != 0 && local_compressed != entry.compressed)
                    || (local_expanded != 0 && local_expanded != entry.expanded)))
        {
            return Err(GithubArchiveError::ZipStructure);
        }
        let data_end = bounded_end(
            data_start,
            usize::try_from(entry.compressed).map_err(|_| GithubArchiveError::DirectoryBounds)?,
            directory_start,
        )?;
        let mut span_end = data_end;
        if descriptor {
            let signed = read_u32(raw, data_end)? == 0x0807_4b50;
            span_end = bounded_end(data_end, if signed { 16 } else { 12 }, directory_start)?;
            let body = data_end + if signed { 4 } else { 0 };
            if read_u32(raw, body)? != entry.crc
                || u64::from(read_u32(raw, body + 4)?) != entry.compressed
                || u64::from(read_u32(raw, body + 8)?) != entry.expanded
            {
                return Err(GithubArchiveError::ZipStructure);
            }
        }
        members.push(CheckedMember {
            name,
            method: entry.method,
            crc: entry.crc,
            expanded: entry.expanded,
            compressed: entry.compressed,
            data: data_start..data_end,
            span: entry.local..span_end,
        });
    }
    let mut spans = members
        .iter()
        .map(|entry| entry.span.clone())
        .collect::<Vec<_>>();
    spans.sort_by_key(|span| span.start);
    let mut end = 0;
    for span in spans {
        if span.start != end || span.end < span.start {
            return Err(GithubArchiveError::ZipStructure);
        }
        end = span.end;
    }
    if end != directory_start {
        return Err(GithubArchiveError::ZipStructure);
    }
    Ok(CheckedDirectory { members })
}

pub(super) fn decode_deflate_exact(
    compressed: &[u8],
    declared: u64,
    remaining: u64,
) -> Result<Vec<u8>, GithubArchiveError> {
    if declared > MAX_GITHUB_ARCHIVE_ENTRY_BYTES || declared > remaining {
        return Err(GithubArchiveError::ExpandedSize);
    }
    let mut inflater = flate2::Decompress::new(false);
    let mut output = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let before_in = inflater.total_in();
        let before_out = inflater.total_out();
        let offset = usize::try_from(before_in).map_err(|_| GithubArchiveError::Checksum)?;
        let input = compressed
            .get(offset..)
            .ok_or(GithubArchiveError::Checksum)?;
        let available = declared
            .min(remaining)
            .saturating_sub(before_out)
            .saturating_add(1);
        let capacity = usize::try_from(available.min(buffer.len() as u64))
            .map_err(|_| GithubArchiveError::ExpandedSize)?;
        let status = inflater
            .decompress(
                input,
                &mut buffer[..capacity],
                flate2::FlushDecompress::None,
            )
            .map_err(|_| GithubArchiveError::Checksum)?;
        let wrote = usize::try_from(
            inflater
                .total_out()
                .checked_sub(before_out)
                .ok_or(GithubArchiveError::Checksum)?,
        )
        .map_err(|_| GithubArchiveError::ExpandedSize)?;
        if wrote > buffer.len()
            || inflater.total_out() > declared
            || inflater.total_out() > remaining
        {
            return Err(GithubArchiveError::ExpandedSize);
        }
        output.extend_from_slice(&buffer[..wrote]);
        if status == flate2::Status::StreamEnd {
            if inflater.total_in() != compressed.len() as u64 || inflater.total_out() != declared {
                return Err(GithubArchiveError::Checksum);
            }
            return Ok(output);
        }
        if inflater.total_in() == before_in && inflater.total_out() == before_out {
            return Err(GithubArchiveError::Checksum);
        }
    }
}

fn member_bytes(
    raw: &[u8],
    member: &CheckedMember,
    remaining: u64,
) -> Result<Vec<u8>, GithubArchiveError> {
    let compressed = &raw[member.data.clone()];
    let output = if member.method == 0 {
        if member.compressed != member.expanded || member.expanded > remaining {
            return Err(GithubArchiveError::ExpandedSize);
        }
        compressed.to_vec()
    } else {
        decode_deflate_exact(compressed, member.expanded, remaining)?
    };
    let mut crc = flate2::Crc::new();
    crc.update(&output);
    if crc.sum() != member.crc {
        return Err(GithubArchiveError::Checksum);
    }
    Ok(output)
}

const MAX_JSON_DEPTH: usize = 64;
const MAX_JSON_NODES: usize = 262_144;
const MAX_JSON_ARRAY: usize = MAX_GITHUB_NODE_CLAIMS;

#[derive(Default)]
struct JsonBudget {
    nodes: usize,
    strings: usize,
    limit_hit: bool,
}

struct StrictJson<'a> {
    budget: &'a std::cell::RefCell<JsonBudget>,
    depth: usize,
    array_limit: usize,
    remaining_claims: usize,
    allowed: bool,
}

impl StrictJson<'_> {
    fn string<E: serde::de::Error>(&self, value: &str) -> Result<(), E> {
        let mut budget = self.budget.borrow_mut();
        budget.strings = budget
            .strings
            .checked_add(value.len())
            .ok_or_else(|| E::custom("JSON string budget"))?;
        if budget.strings as u64 > MAX_GITHUB_EXPANDED_BYTES {
            budget.limit_hit = true;
            return Err(E::custom("JSON string budget"));
        }
        Ok(())
    }
    fn child(&self, array_limit: usize, allowed: bool) -> StrictJson<'_> {
        StrictJson {
            budget: self.budget,
            depth: self.depth + 1,
            array_limit,
            remaining_claims: self.remaining_claims,
            allowed,
        }
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for StrictJson<'_> {
    type Value = serde_json::Value;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        {
            let mut budget = self.budget.borrow_mut();
            if !self.allowed || self.depth > MAX_JSON_DEPTH || budget.nodes >= MAX_JSON_NODES {
                budget.limit_hit = true;
                return Err(serde::de::Error::custom("JSON node/depth/array budget"));
            }
            budget.nodes += 1;
        }
        deserializer.deserialize_any(self)
    }
}

impl<'de> serde::de::Visitor<'de> for StrictJson<'_> {
    type Value = serde_json::Value;
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("bounded JSON without duplicate keys")
    }
    fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(value.into())
    }
    fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(value.into())
    }
    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(value.into())
    }
    fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Self::Value, E> {
        serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }
    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        self.string(value)?;
        Ok(value.into())
    }
    fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
        self.string(&value)?;
        Ok(serde_json::Value::String(value))
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(serde_json::Value::Null)
    }
    fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(serde_json::Value::Null)
    }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        self,
        mut sequence: A,
    ) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = sequence
            .next_element_seed(self.child(MAX_JSON_ARRAY, values.len() < self.array_limit))?
        {
            values.push(value);
        }
        Ok(serde_json::Value::Array(values))
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut values = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            self.string::<A::Error>(&key)?;
            if key.len() > 1024 || values.len() >= MAX_JSON_ARRAY {
                self.budget.borrow_mut().limit_hit = true;
                return Err(serde::de::Error::custom("JSON object/key budget"));
            }
            if values.contains_key(&key) {
                return Err(serde::de::Error::custom("duplicate JSON key"));
            }
            let limit = match key.as_str() {
                "results" | "executionRecords" | "nodeIds" | "commandDigests" => {
                    self.remaining_claims
                }
                "entries" => MAX_GITHUB_ARCHIVE_ENTRIES,
                _ => MAX_JSON_ARRAY,
            };
            let value = map.next_value_seed(self.child(limit, true))?;
            values.insert(key, value);
        }
        Ok(serde_json::Value::Object(values))
    }
}

fn strict_json(
    raw: &[u8],
    budget: &std::cell::RefCell<JsonBudget>,
    remaining_claims: usize,
    invalid: GithubArchiveError,
) -> Result<serde_json::Value, GithubArchiveError> {
    use serde::de::DeserializeSeed as _;
    let mut deserializer = serde_json::Deserializer::from_slice(raw);
    let value = StrictJson {
        budget,
        depth: 0,
        array_limit: MAX_JSON_ARRAY,
        remaining_claims,
        allowed: true,
    }
    .deserialize(&mut deserializer);
    match value {
        Ok(value) => {
            deserializer.end().map_err(|_| invalid)?;
            Ok(value)
        }
        Err(_) if budget.borrow().limit_hit => Err(GithubArchiveError::NodeLimit),
        Err(_) => Err(invalid),
    }
}

fn archive_digest_valid(value: &Digest) -> bool {
    let text = value.as_str();
    text.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn decode_checked_zip(
    raw: &[u8],
    checked: CheckedDirectory,
) -> Result<UntrustedGithubArchiveInspection, GithubArchiveError> {
    if !checked
        .members
        .iter()
        .any(|member| member.name == "manifest.json")
    {
        return Err(GithubArchiveError::Manifest);
    }
    let mut members = Vec::with_capacity(checked.members.len());
    let mut remaining = MAX_GITHUB_EXPANDED_BYTES;
    for entry in &checked.members {
        let decoded = member_bytes(raw, entry, remaining)?;
        remaining = remaining
            .checked_sub(decoded.len() as u64)
            .ok_or(GithubArchiveError::ExpandedSize)?;
        members.push(UntrustedGithubArchiveMember {
            path: entry.name.clone(),
            digest: Digest::sha256_bytes(&decoded),
            raw: decoded,
        });
    }
    let budget = std::cell::RefCell::new(JsonBudget::default());
    let manifest_raw = &members
        .iter()
        .find(|entry| entry.path == "manifest.json")
        .ok_or(GithubArchiveError::Manifest)?
        .raw;
    let value = strict_json(
        manifest_raw,
        &budget,
        MAX_GITHUB_NODE_CLAIMS,
        GithubArchiveError::Manifest,
    )?;
    let manifest: cockpit_protocol::ci_admission::UntrustedGithubArchiveManifest =
        serde_json::from_value(value).map_err(|_| GithubArchiveError::Manifest)?;
    if manifest.schema_version
        != cockpit_protocol::ci_admission::GITHUB_ARCHIVE_MANIFEST_SCHEMA_VERSION
        || manifest.entries.is_empty()
        || manifest.entries.len() != members.len().saturating_sub(1)
    {
        return Err(GithubArchiveError::Manifest);
    }
    let mut inventory = std::collections::BTreeSet::new();
    let mut receipts = Vec::with_capacity(manifest.entries.len());
    let mut claims = 0_usize;
    let mut missing_coverage = false;
    for entry in manifest.entries {
        canonical_path(entry.path.as_bytes()).map_err(|_| GithubArchiveError::Manifest)?;
        if entry.path == "manifest.json" || !inventory.insert(entry.path.clone()) {
            return Err(GithubArchiveError::Manifest);
        }
        let member = members
            .iter()
            .find(|member| member.path == entry.path)
            .ok_or(GithubArchiveError::Manifest)?;
        if entry.bytes != member.raw.len() as u64
            || !archive_digest_valid(&entry.digest)
            || entry.digest != member.digest
        {
            return Err(GithubArchiveError::EntryDigest);
        }
        let value = strict_json(
            &member.raw,
            &budget,
            MAX_GITHUB_NODE_CLAIMS - claims,
            GithubArchiveError::NativeReceipt,
        )?;
        let envelope = validate_native_envelope(value)?;
        claims = claims
            .checked_add(envelope.receipt.results.len())
            .ok_or(GithubArchiveError::NodeLimit)?;
        if claims > MAX_GITHUB_NODE_CLAIMS {
            return Err(GithubArchiveError::NodeLimit);
        }
        missing_coverage |= envelope
            .receipt
            .plan_receipt
            .as_ref()
            .and_then(|plan| plan.coverage_manifest.as_ref())
            .is_none();
        receipts.push(envelope);
    }
    let mut gaps = [
        "provider_origin_unverified",
        "workflow_execution_closure_unverified",
        "producer_runtime_unverified",
        "artifact_job_attempt_binding_unavailable",
        "current_consumer_command_binding_unavailable",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    gaps.push(
        if missing_coverage {
            "native_coverage_manifest_missing"
        } else {
            "native_coverage_manifest_unverified"
        }
        .to_owned(),
    );
    Ok(UntrustedGithubArchiveInspection {
        archive_digest: Digest::sha256_bytes(raw),
        members,
        receipts,
        gaps,
    })
}

// Integration fragment for ci_admission/archive.rs. The caller must already
// have rejected duplicate JSON keys at every depth and enforced parse budgets.
// All successful results remain untrusted declarations, including coverage.
fn validate_native_envelope(
    value: serde_json::Value,
) -> Result<UntrustedGithubNativeEnvelope<VerificationReceipt>, GithubArchiveError> {
    // Check inventories before decoding or relational checks so an excessive
    // inventory is reported as NodeLimit even when its accounting is invalid.
    for pointer in [
        "/receipt/results",
        "/receipt/executionRecords",
        "/receipt/receiptCandidates",
        "/receipt/planReceipt/executedNodes",
        "/receipt/planReceipt/reusedNodes",
        "/receipt/planReceipt/sharedNodes",
        "/receipt/planReceipt/coverageManifest/workspaceMembers",
        "/receipt/planReceipt/coverageManifest/nodeIds",
        "/receipt/planReceipt/coverageManifest/commandDigests",
    ] {
        if value
            .pointer(pointer)
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| items.len() > MAX_GITHUB_NODE_CLAIMS)
        {
            return Err(GithubArchiveError::NodeLimit);
        }
    }
    for field in [
        "nodesPlanned",
        "nodesExecuted",
        "nodesReused",
        "rerunStale",
        "rerunUnknown",
        "protectedNodesExecuted",
        "protectedNodesSkipped",
        "processesSpawned",
        "maxConcurrentProcesses",
        "processSpawnFailures",
    ] {
        if value["receipt"][field]
            .as_u64()
            .is_some_and(|count| count > MAX_GITHUB_NODE_CLAIMS as u64)
        {
            return Err(GithubArchiveError::NodeLimit);
        }
    }

    // Native receiptDigest hashes its raw semantic JSON. Serializing the
    // decoded Rust struct directly would change field order and omitted fields.
    let actual_receipt_digest = cockpit_protocol::digest_json(
        value
            .get("receipt")
            .ok_or(GithubArchiveError::NativeReceipt)?,
    )
    .map_err(|_| GithubArchiveError::NativeReceipt)?;
    let envelope: UntrustedGithubNativeEnvelope<VerificationReceipt> =
        serde_json::from_value(value).map_err(|_| GithubArchiveError::NativeReceipt)?;
    let receipt = &envelope.receipt;
    if envelope.protocol_version != 1
        || envelope.evidence_schema_version != 2
        || envelope.capture_mode
            != cockpit_protocol::ci_admission::UntrustedGithubCaptureMode::FullCapture
        || !envelope.passed
        || !receipt.passed
        || !github_native_work_item_id(&envelope.work_item_id)
        || !github_native_runtime_version(&envelope.runtime_version)
        || envelope.repository_id.parse::<Digest>().is_err()
        || envelope.runtime_digest.as_str().parse::<Digest>().is_err()
        || envelope.contract_digest.as_str().parse::<Digest>().is_err()
        || envelope
            .repository_snapshot_digest
            .as_str()
            .parse::<Digest>()
            .is_err()
        || envelope.receipt_digest.as_str().parse::<Digest>().is_err()
        || actual_receipt_digest != envelope.receipt_digest
        || chrono::DateTime::parse_from_rfc3339(&envelope.created_at).is_err()
        || receipt.work_item_id.as_deref() != Some(envelope.work_item_id.as_str())
        || receipt.repository_id.as_deref() != Some(envelope.repository_id.as_str())
        || receipt.runtime_version.as_deref() != Some(envelope.runtime_version.as_str())
        || receipt.runtime_digest.as_deref() != Some(envelope.runtime_digest.as_str())
    {
        return Err(GithubArchiveError::NativeReceipt);
    }
    let timeout_seconds = receipt
        .timeout_seconds
        .ok_or(GithubArchiveError::NativeReceipt)?;
    cockpit_verification::validate_timeout_seconds(timeout_seconds, "github-native-receipt")
        .map_err(|_| GithubArchiveError::NativeReceipt)?;
    let timeout_ms = u128::from(timeout_seconds)
        .checked_mul(1_000)
        .ok_or(GithubArchiveError::NativeReceipt)?;
    let count = receipt.results.len();
    if count == 0
        || receipt.nodes_planned != count
        || receipt.nodes_executed != count
        || receipt.execution_records.len() != count
        || receipt.processes_spawned != count
        || receipt.nodes_reused != 0
        || receipt.protected_nodes_skipped != 0
        || receipt.process_spawn_failures != 0
        || receipt.max_concurrent_processes == 0
        || receipt.max_concurrent_processes > count
        || receipt.receipt_candidates.len() > count
    {
        return Err(GithubArchiveError::NativeReceipt);
    }

    let mut results = std::collections::BTreeMap::new();
    let mut protected_count = 0_usize;
    let mut stale_count = 0_usize;
    let mut unknown_count = 0_usize;
    for result in &receipt.results {
        if !github_native_text_identity(&result.node_id)
            || results.insert(result.node_id.as_str(), result).is_some()
            || !result.passed
            || result.reused
            || result.action != cockpit_verification::PlannedAction::Execute
            || result.satisfied_by != cockpit_verification::PlannedSatisfaction::Execution
            || result.output_truncated
            || result.timed_out
            || result
                .output_digest
                .as_deref()
                .is_none_or(|digest| digest.parse::<Digest>().is_err())
            || result
                .receipt_id
                .as_deref()
                .is_some_and(|digest| digest.parse::<Digest>().is_err())
        {
            return Err(GithubArchiveError::NativeReceipt);
        }
        // Only states/reasons actually emitted for a native Execute action
        // are valid. Fresh denotes reuse; dependency reruns become Stale.
        let state_and_reason_valid = match (&result.state, result.reason.as_str()) {
            (cockpit_verification::PlannedState::NotApplicable, "reuse_not_configured") => {
                !result.protected
            }
            (cockpit_verification::PlannedState::Unknown, "protected_node") => result.protected,
            (
                cockpit_verification::PlannedState::Unknown,
                "evidence_missing" | "receipt_invalid" | "receipt_failed" | "receipt_from_future",
            ) => !result.protected,
            (
                cockpit_verification::PlannedState::Stale,
                "evidence_expired" | "binding_mismatch" | "dependency_rerun_required",
            ) => !result.protected,
            _ => false,
        };
        if !state_and_reason_valid {
            return Err(GithubArchiveError::NativeReceipt);
        }
        if result.protected {
            protected_count = protected_count
                .checked_add(1)
                .ok_or(GithubArchiveError::NativeReceipt)?;
        }
        if result.state == cockpit_verification::PlannedState::Stale {
            stale_count = stale_count
                .checked_add(1)
                .ok_or(GithubArchiveError::NativeReceipt)?;
        }
        if result.state == cockpit_verification::PlannedState::Unknown {
            unknown_count = unknown_count
                .checked_add(1)
                .ok_or(GithubArchiveError::NativeReceipt)?;
        }
    }
    if receipt.protected_nodes_executed != protected_count
        || receipt.rerun_stale != stale_count
        || receipt.rerun_unknown != unknown_count
    {
        return Err(GithubArchiveError::NativeReceipt);
    }

    let mut records = std::collections::BTreeMap::new();
    let mut total_record_elapsed_ms = 0_u128;
    let mut longest_record_elapsed_ms = 0_u128;
    for record in &receipt.execution_records {
        let result = results
            .get(record.node_id.as_str())
            .ok_or(GithubArchiveError::NativeReceipt)?;
        if records.insert(record.node_id.as_str(), record).is_some()
            || record.command_digest.parse::<Digest>().is_err()
            || !record.spawned
            || !record.passed
            || record.exit_code != Some(0)
            || record.termination_signal.is_some()
            || record.stdout_truncated
            || record.stderr_truncated
            || record.timed_out
            || record.timeout_seconds != timeout_seconds
            || record.deadline_ms.checked_sub(timeout_ms).is_none()
            || record.elapsed_ms > timeout_ms
        {
            return Err(GithubArchiveError::NativeReceipt);
        }
        let stdout = github_native_capture_bytes(&record.stdout_hex)?;
        let stderr = github_native_capture_bytes(&record.stderr_hex)?;
        // Match cockpit-verification::OutputIdentity exactly, including field
        // order and the explicit null terminationSignal field. A Value would
        // sort keys and calculate a different digest.
        let output_identity = GithubNativeOutputIdentity {
            success: record.passed,
            exit_code: record.exit_code,
            termination_signal: record.termination_signal,
            stdout: &stdout,
            stderr: &stderr,
            stdout_truncated: record.stdout_truncated,
            stderr_truncated: record.stderr_truncated,
            timed_out: record.timed_out,
        };
        let output_json =
            serde_json::to_vec(&output_identity).map_err(|_| GithubArchiveError::NativeReceipt)?;
        let output_digest = Digest::sha256_bytes(&output_json);
        if result.output_digest.as_deref() != Some(output_digest.as_str()) {
            return Err(GithubArchiveError::NativeReceipt);
        }
        total_record_elapsed_ms = total_record_elapsed_ms
            .checked_add(record.elapsed_ms)
            .ok_or(GithubArchiveError::NativeReceipt)?;
        longest_record_elapsed_ms = longest_record_elapsed_ms.max(record.elapsed_ms);
    }
    // Counts plus unique lookup already establish exact result/record sets.
    let accounted_elapsed_ms = receipt
        .planning_elapsed_ms
        .checked_add(receipt.execution_elapsed_ms)
        .ok_or(GithubArchiveError::NativeReceipt)?;
    let available_process_ms = receipt
        .execution_elapsed_ms
        .checked_mul(receipt.max_concurrent_processes as u128)
        .ok_or(GithubArchiveError::NativeReceipt)?;
    if receipt.elapsed_ms < accounted_elapsed_ms
        || receipt.execution_elapsed_ms < longest_record_elapsed_ms
        || total_record_elapsed_ms > available_process_ms
    {
        return Err(GithubArchiveError::NativeReceipt);
    }

    if let Some(plan) = &receipt.plan_receipt {
        if plan.validate_monotonic().is_err()
            || plan.work_item_id.as_deref() != Some(envelope.work_item_id.as_str())
            || plan.repository_id.as_deref() != Some(envelope.repository_id.as_str())
            || plan.repository_snapshot_digest.as_deref()
                != Some(envelope.repository_snapshot_digest.as_str())
            || plan.timeout_seconds != Some(timeout_seconds)
            || plan.planning_elapsed_ms != receipt.planning_elapsed_ms
            || plan.execution_elapsed_ms != receipt.execution_elapsed_ms
            || !plan.reused_nodes.is_empty()
            || !plan.shared_nodes.is_empty()
            || plan.saved_executions != 0
            || plan.executed_nodes.len() != count
            || plan
                .base_revision
                .as_deref()
                .is_some_and(|base| !github_native_object_id(base))
            || (plan.stage.requires_base_revision() && plan.base_revision.is_none())
            || plan
                .policy_refs
                .iter()
                .any(|item| !github_native_text_identity(item))
            || plan
                .affected_paths
                .iter()
                .any(|item| !github_native_text_identity(item))
            || plan
                .selection_reasons
                .iter()
                .any(|item| item.trim().is_empty())
            || plan.escalations.iter().any(|item| item.trim().is_empty())
        {
            return Err(GithubArchiveError::NativeReceipt);
        }
        let mut planned = std::collections::BTreeSet::new();
        for node in &plan.executed_nodes {
            if !planned.insert(node.as_str()) || !results.contains_key(node.as_str()) {
                return Err(GithubArchiveError::NativeReceipt);
            }
        }
        if let Some(coverage) = &plan.coverage_manifest {
            if coverage.validate().is_err() {
                return Err(GithubArchiveError::NativeReceipt);
            }
            for (node, digest) in coverage.node_ids.iter().zip(&coverage.command_digests) {
                let record = records
                    .get(node.as_str())
                    .ok_or(GithubArchiveError::NativeReceipt)?;
                if record.command_digest != *digest {
                    return Err(GithubArchiveError::NativeReceipt);
                }
            }
            // This is only an internally consistent declared inventory. The
            // caller must not infer Contract coverage or provider authority.
        }
    }
    if let Some(cost) = &receipt.cost_observation {
        receipt
            .validate_cost_observation(cost)
            .map_err(|_| GithubArchiveError::NativeReceipt)?;
    }

    // A fresh native execution may contain newly generated reusable receipt
    // candidates; checking them confers no reuse capability. Imported producer
    // reuse itself has already been rejected above.
    let mut candidate_nodes = std::collections::BTreeSet::new();
    let mut candidate_ids = std::collections::BTreeSet::new();
    for candidate in &receipt.receipt_candidates {
        let result = results
            .get(candidate.node_id.as_str())
            .ok_or(GithubArchiveError::NativeReceipt)?;
        let record = records
            .get(candidate.node_id.as_str())
            .ok_or(GithubArchiveError::NativeReceipt)?;
        if candidate.validate().is_err()
            || !candidate.passed
            || result.protected
            || !candidate_nodes.insert(candidate.node_id.as_str())
            || !candidate_ids.insert(candidate.receipt_id.as_str())
            || result.receipt_id.as_deref() != Some(candidate.receipt_id.as_str())
            || result.output_digest.as_deref() != Some(candidate.output_digest.as_str())
            || candidate.context.command_digest != record.command_digest
            || cockpit_protocol::VerificationStage::parse(&candidate.context.stage).is_err()
            || receipt.plan_receipt.as_ref().is_some_and(|plan| {
                candidate.context.stage != plan.stage.as_str()
                    || plan
                        .base_revision
                        .as_deref()
                        .is_some_and(|base| candidate.context.diff.base_commit != base)
            })
        {
            return Err(GithubArchiveError::NativeReceipt);
        }
    }

    if let Some(retention_value) = &envelope.retention {
        let policy: cockpit_protocol::EvidenceRetentionPolicy =
            serde_json::from_value(retention_value.clone())
                .map_err(|_| GithubArchiveError::NativeReceipt)?;
        if policy.schema_version != 1
            || policy.repository_id != envelope.repository_id
            || policy.work_item_id != envelope.work_item_id
            || chrono::DateTime::parse_from_rfc3339(&policy.created_at).is_err()
            || cockpit_protocol::validate_evidence_retention(&policy.retention).is_err()
            || policy.retention.persistence != cockpit_protocol::EvidencePersistence::FullCapture
            || policy
                .retention
                .retention_days
                .is_some_and(|days| days == 0 || days.checked_mul(86_400).is_none())
            || policy
                .retention
                .expires_at
                .as_deref()
                .is_some_and(|expiry| {
                    expiry.parse::<u64>().is_err()
                        && chrono::DateTime::parse_from_rfc3339(expiry).is_err()
                })
        {
            return Err(GithubArchiveError::NativeReceipt);
        }
    }
    Ok(envelope)
}

fn github_native_work_item_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn github_native_runtime_version(value: &str) -> bool {
    // Structural version syntax only; no installed Runtime identity is read.
    if value.len() > 128 || !value.is_ascii() {
        return false;
    }
    let (version, build) = value
        .split_once('+')
        .map_or((value, None), |(version, suffix)| (version, Some(suffix)));
    let (core, prerelease) = version
        .split_once('-')
        .map_or((version, None), |(core, suffix)| (core, Some(suffix)));
    let numeric_identifier = |part: &str| {
        !part.is_empty()
            && part.bytes().all(|byte| byte.is_ascii_digit())
            && (part.len() == 1 || !part.starts_with('0'))
            && part.parse::<u64>().is_ok()
    };
    let suffix_valid = |suffix: &str, numeric_no_leading_zero: bool| {
        suffix.split('.').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && (!numeric_no_leading_zero
                    || !part.bytes().all(|byte| byte.is_ascii_digit())
                    || part.len() == 1
                    || !part.starts_with('0'))
        })
    };
    core.split('.').count() == 3
        && core.split('.').all(numeric_identifier)
        && prerelease.is_none_or(|suffix| suffix_valid(suffix, true))
        && build.is_none_or(|suffix| suffix_valid(suffix, false))
}

fn github_native_text_identity(value: &str) -> bool {
    !value.is_empty() && value.trim() == value && !value.chars().any(char::is_control)
}

fn github_native_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn github_native_capture_bytes(value: &str) -> Result<Vec<u8>, GithubArchiveError> {
    let maximum_hex_bytes = cockpit_verification::MAX_CAPTURE_BYTES_PER_STREAM
        .checked_mul(2)
        .ok_or(GithubArchiveError::NativeReceipt)?;
    if value.len() > maximum_hex_bytes || !value.len().is_multiple_of(2) {
        return Err(GithubArchiveError::NativeReceipt);
    }
    let nibble = |byte: u8| match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    };
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let high = nibble(pair[0]).ok_or(GithubArchiveError::NativeReceipt)?;
            let low = nibble(pair[1]).ok_or(GithubArchiveError::NativeReceipt)?;
            Ok((high << 4) | low)
        })
        .collect()
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GithubNativeOutputIdentity<'a> {
    success: bool,
    exit_code: Option<i32>,
    termination_signal: Option<i32>,
    stdout: &'a [u8],
    stderr: &'a [u8],
    stdout_truncated: bool,
    stderr_truncated: bool,
    timed_out: bool,
}

/// A bounded data projection, never an executable receipt or source capability.
pub(super) struct UntrustedPortableView {
    pub(super) integrity_valid: bool,
    pub(super) claims: serde_json::Value,
    pub(super) gaps: Vec<String>,
}

/// Parse duplicate-free portable JSON before any Value can swallow a key.
/// Malformed/unsupported wire formats are rejected; well-shaped failed or
/// inconsistent native results remain viewable and cannot become complete proof.
pub(super) fn inspect_untrusted_portable_evidence(
    raw: &[u8],
) -> Result<UntrustedPortableView, GithubArchiveError> {
    check_raw_archive_size(raw.len() as u64)?;
    let budget = std::cell::RefCell::new(JsonBudget::default());
    let value = strict_json(
        raw,
        &budget,
        MAX_GITHUB_NODE_CLAIMS,
        GithubArchiveError::NativeReceipt,
    )?;
    let bundle: cockpit_protocol::ci_admission::PortableExecutionEvidence<serde_json::Value> =
        serde_json::from_value(value).map_err(|_| GithubArchiveError::NativeReceipt)?;
    if bundle.schema_version != cockpit_protocol::ci_admission::PORTABLE_EVIDENCE_SCHEMA_VERSION {
        return Err(GithubArchiveError::NativeReceipt);
    }
    // This strict shape is not authority. In particular nativeInput can contain
    // arbitrary supplied source flags, none of which reaches trust decisions.
    let envelope: UntrustedGithubNativeEnvelope<VerificationReceipt> =
        serde_json::from_value(bundle.native_evidence.clone())
            .map_err(|_| GithubArchiveError::NativeReceipt)?;
    if envelope.protocol_version != 1
        || envelope.evidence_schema_version != 2
        || envelope.capture_mode
            != cockpit_protocol::ci_admission::UntrustedGithubCaptureMode::FullCapture
    {
        return Err(GithubArchiveError::NativeReceipt);
    }
    // Keep the archive's complete-success validator unchanged. It checks raw
    // semantic receipt digest, output identity, exact inventories/accounting,
    // retention and supported execution semantics. Failed claims are data, not
    // a reason to prevent inspection, and are never normalized into success.
    let integrity_valid = validate_native_envelope(bundle.native_evidence).is_ok();
    let mut gaps = Vec::new();
    if !integrity_valid {
        gaps.push("native_receipt_integrity_invalid_or_incomplete".into());
        gaps.push("native_verification_not_complete_success".into());
    }
    // Do not echo raw command output, caller authority, arbitrary nativeInput,
    // or unbounded identity strings into the public report.
    let mut hidden_identity = false;
    let mut identity = |text: &str| {
        if text.len() <= 1024 && !text.chars().any(char::is_control) {
            serde_json::Value::String(text.into())
        } else {
            hidden_identity = true;
            serde_json::Value::Null
        }
    };
    let claims = serde_json::json!({
        "passed": envelope.passed,
        "workItemId": identity(&envelope.work_item_id),
        "repositoryId": identity(&envelope.repository_id),
        "runtimeVersion": identity(&envelope.runtime_version),
        "runtimeDigest": identity(envelope.runtime_digest.as_str()),
        "contractDigest": identity(envelope.contract_digest.as_str()),
        "repositorySnapshotDigest": identity(envelope.repository_snapshot_digest.as_str()),
        "receiptDigest": identity(envelope.receipt_digest.as_str()),
        "receiptPassed": envelope.receipt.passed,
        "nodesPlanned": envelope.receipt.nodes_planned,
        "nodesExecuted": envelope.receipt.nodes_executed,
        "nodesReused": envelope.receipt.nodes_reused,
        "processesSpawned": envelope.receipt.processes_spawned,
    });
    if hidden_identity {
        gaps.push("native_claim_identity_not_displayable".into());
    }
    Ok(UntrustedPortableView {
        integrity_valid,
        claims,
        gaps,
    })
}

/// Bounded archive data only; neither matching digest establishes source trust.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UntrustedPortableArchiveInspection {
    pub archive_digest: Digest,
    pub portable_digest: Digest,
    /// Exact portable JSON member bytes, never a source capability.
    pub portable_raw: Vec<u8>,
}

/// Inspect the exact single-file upload profile without extracting to disk.
///
/// The intended member is only `portable-native-evidence.json`. Both expected
/// digests are untrusted inputs. Successful parsing must retain the existing
/// ZIP/JSON budgets and require the current portable inspector's native
/// integrity checks, while conferring no origin or execution authority.
pub fn inspect_untrusted_portable_archive(
    raw: &[u8],
    expected_archive_digest: &Digest,
    expected_portable_digest: &Digest,
) -> Result<UntrustedPortableArchiveInspection, GithubArchiveError> {
    check_raw_archive_size(raw.len() as u64)?;
    let archive_digest = Digest::sha256_bytes(raw);
    if archive_digest != *expected_archive_digest {
        return Err(GithubArchiveError::ArchiveDigest);
    }
    // Reuse the complete classic ZIP preflight before choosing this profile.
    // This preserves shared limits and structural-error priority over inventory.
    let checked = preflight_directory(raw)?;
    let [member] = checked.members.as_slice() else {
        return Err(GithubArchiveError::Manifest);
    };
    if member.name != "portable-native-evidence.json" {
        return Err(GithubArchiveError::Manifest);
    }
    let portable_raw = member_bytes(raw, member, MAX_GITHUB_EXPANDED_BYTES)?;
    let portable_digest = Digest::sha256_bytes(&portable_raw);
    if portable_digest != *expected_portable_digest {
        return Err(GithubArchiveError::EntryDigest);
    }
    // The bounded portable parser rejects deep duplicate keys and unsupported
    // wire shapes; its native validator also requires complete success/accounting.
    // A hash-consistent failed/incomplete receipt remains non-admitting data.
    let inspected = inspect_untrusted_portable_evidence(&portable_raw)?;
    if !inspected.integrity_valid {
        return Err(GithubArchiveError::NativeReceipt);
    }
    Ok(UntrustedPortableArchiveInspection {
        archive_digest,
        portable_digest,
        portable_raw,
    })
}
