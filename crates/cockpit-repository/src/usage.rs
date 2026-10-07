use super::{
    ObserverError, acquire_lifecycle_lock, create_and_open_cap_directory,
    open_cap_directory_nofollow_strict, open_or_create_cap_nofollow,
    read_cap_file_nofollow_bounded, reject_duplicate_json_keys, repository_id,
    require_current_action_admission, validate_work_item_id,
};
use cap_fs_ext::OpenOptionsFollowExt;
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use chrono::{DateTime, SecondsFormat, Utc};
use cockpit_core::Digest;
use cockpit_protocol::{
    RuntimeContext, USAGE_SCHEMA_VERSION, UsageAssurance, UsageCoverage, UsageReceipt,
    UsageReceiptRef, UsageRecordRequest, UsageSubtotal, UsageSummary, UsageTokenCounts,
};
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::io::Write as _;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_USAGE_RECEIPT_BYTES: u64 = 64 * 1024;
const MAX_USAGE_EVIDENCE_BYTES: usize = 4 * 1024 * 1024;
static NEXT_USAGE_WRITE_ID: AtomicU64 = AtomicU64::new(0);

fn write_usage_receipt_immutable(
    parent: &Dir,
    name: &str,
    path: &Path,
    bytes: &[u8],
) -> Result<(), ObserverError> {
    let sequence = NEXT_USAGE_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let temporary = format!("{name}.receipt-tmp-{}-{sequence}", std::process::id());
    let temporary_path = path.with_file_name(&temporary);
    let mut options = cap_std::fs::OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(cap_fs_ext::FollowSymlinks::No);
    let mut file = parent
        .open_with(&temporary, &options)
        .map_err(|source| read_error(&temporary_path, source))?
        .into_std();
    if let Err(source) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = parent.remove_file(&temporary);
        return Err(read_error(&temporary_path, source));
    }
    drop(file);
    let installed = match parent.hard_link(&temporary, parent, name) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            read_cap_file_nofollow_bounded(parent, name, path, bytes.len() as u64).and_then(
                |existing| {
                    if existing == bytes {
                        Ok(())
                    } else {
                        Err(state_error(
                            path,
                            "immutable usage receipt already exists with different content",
                        ))
                    }
                },
            )
        }
        Err(source) => Err(read_error(path, source)),
    };
    let _ = parent.remove_file(&temporary);
    installed?;
    #[cfg(not(windows))]
    {
        let mut options = cap_std::fs::OpenOptions::new();
        options.read(true);
        parent
            .open_with(".", &options)
            .map(|directory| directory.into_std().sync_all())
            .and_then(|result| result)
            .map_err(|source| read_error(path.parent().unwrap_or(path), source))?;
    }
    Ok(())
}

fn state_error(path: &Path, message: impl Into<String>) -> ObserverError {
    ObserverError::State {
        path: path.to_path_buf(),
        message: message.into(),
    }
}

fn read_error(path: &Path, source: std::io::Error) -> ObserverError {
    ObserverError::Read {
        path: path.to_path_buf(),
        source,
    }
}

fn root_directory(root: &Path) -> Result<Dir, ObserverError> {
    Dir::open_ambient_dir(root, ambient_authority()).map_err(|source| read_error(root, source))
}

fn usage_directory(root: &Path, create: bool) -> Result<Option<Dir>, ObserverError> {
    let base = root_directory(root)?;
    let ai = open_cap_directory_nofollow_strict(&base, ".ai", &root.join(".ai"))?;
    let evidence = open_cap_directory_nofollow_strict(&ai, "evidence", &root.join(".ai/evidence"))?;
    let path = root.join(".ai/evidence/usage");
    if create {
        return create_and_open_cap_directory(&evidence, "usage", &path).map(Some);
    }
    match evidence.symlink_metadata("usage") {
        Ok(_) => open_cap_directory_nofollow_strict(&evidence, "usage", &path).map(Some),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(source) => Err(read_error(&path, source)),
    }
}

fn usage_lock(root: &Path, create: bool) -> Result<fs::File, ObserverError> {
    let base = root_directory(root)?;
    let ai = open_cap_directory_nofollow_strict(&base, ".ai", &root.join(".ai"))?;
    let locks_path = root.join(".ai/locks");
    let locks = if create {
        create_and_open_cap_directory(&ai, "locks", &locks_path)?
    } else {
        open_cap_directory_nofollow_strict(&ai, "locks", &locks_path)?
    };
    let path = locks_path.join("usage.lock");
    if create {
        open_or_create_cap_nofollow(&locks, "usage.lock", &path)
    } else {
        let mut options = cap_std::fs::OpenOptions::new();
        options.read(true).follow(cap_fs_ext::FollowSymlinks::No);
        let file = locks
            .open_with("usage.lock", &options)
            .map_err(|source| read_error(&path, source))?
            .into_std();
        if !file
            .metadata()
            .map_err(|source| read_error(&path, source))?
            .is_file()
        {
            return Err(state_error(&path, "usage lock must be a regular file"));
        }
        Ok(file)
    }
}

pub(super) fn now_nanos() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true)
}

fn request_identity(request: &UsageRecordRequest) -> Digest {
    Digest::sha256_bytes(
        &serde_json::to_vec(&(
            &request.repository_id,
            &request.work_item_id,
            request.source_kind,
            &request.source_event_id,
        ))
        .expect("usage identity serializes"),
    )
}

fn receipt_name(request: &UsageRecordRequest) -> String {
    format!(
        "{}.json",
        &request_identity(request).to_string()["sha256:".len()..]
    )
}

fn receipt_assurance(request: &UsageRecordRequest) -> (UsageAssurance, UsageAssurance) {
    (
        if request.reported_model.is_some() {
            UsageAssurance::CallerClaim
        } else {
            UsageAssurance::Unknown
        },
        if [
            request.input_tokens,
            request.output_tokens,
            request.cached_input_tokens,
            request.reasoning_tokens,
        ]
        .iter()
        .any(Option::is_some)
        {
            UsageAssurance::CallerClaim
        } else {
            UsageAssurance::Unknown
        },
    )
}

fn receipt_records(
    root: &Path,
    usage: &Dir,
) -> Result<Vec<(UsageReceipt, UsageReceiptRef)>, ObserverError> {
    let mut records = Vec::new();
    let usage_path = root.join(".ai/evidence/usage");
    for entry in usage
        .read_dir(".")
        .map_err(|source| read_error(&usage_path, source))?
    {
        let entry = entry.map_err(|source| read_error(&usage_path, source))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| state_error(&usage_path, "usage directory name is not UTF-8"))?;
        validate_work_item_id(&name)?;
        let work_item_path = usage_path.join(&name);
        let work_item = open_cap_directory_nofollow_strict(usage, &name, &work_item_path)?;
        for file in work_item
            .read_dir(".")
            .map_err(|source| read_error(&work_item_path, source))?
        {
            let file = file.map_err(|source| read_error(&work_item_path, source))?;
            let file_name = file
                .file_name()
                .into_string()
                .map_err(|_| state_error(&work_item_path, "usage receipt name is not UTF-8"))?;
            if file_name.contains(".receipt-tmp-") {
                continue;
            }
            let path = work_item_path.join(&file_name);
            if file_name.len() != 69
                || !file_name.ends_with(".json")
                || !file_name[..64]
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                return Err(state_error(&path, "unexpected usage receipt entry"));
            }
            let bytes = read_cap_file_nofollow_bounded(
                &work_item,
                &file_name,
                &path,
                MAX_USAGE_RECEIPT_BYTES,
            )?;
            reject_duplicate_json_keys(&bytes)
                .map_err(|message| state_error(&path, format!("invalid usage JSON: {message}")))?;
            let receipt: UsageReceipt = serde_json::from_slice(&bytes)
                .map_err(|error| state_error(&path, format!("invalid usage receipt: {error}")))?;
            let request = &receipt.request;
            let expected_assurance = receipt_assurance(request);
            if receipt.schema_version != USAGE_SCHEMA_VERSION
                || request.validate().is_err()
                || request.repository_id != repository_id(root).to_string()
                || request.work_item_id != name
                || receipt_name(request) != file_name
                || receipt.receipt_id
                    != Digest::sha256_bytes(
                        &serde_json::to_vec(request).expect("usage request serializes"),
                    )
                || DateTime::parse_from_rfc3339(&receipt.received_at).is_err()
                || receipt.source_observed_at.is_some()
                || request.source_observed_at.is_some()
                || (receipt.model_assurance, receipt.token_assurance) != expected_assurance
            {
                return Err(state_error(
                    &path,
                    "usage receipt binding or provenance is invalid",
                ));
            }
            records.push((
                receipt,
                UsageReceiptRef {
                    path: format!(".ai/evidence/usage/{name}/{file_name}"),
                    digest: Digest::sha256_bytes(&bytes),
                },
            ));
        }
    }
    records.sort_by(|left, right| left.1.path.cmp(&right.1.path));
    Ok(records)
}

fn checked_sum(total: Option<u64>, next: Option<u64>, first: bool) -> Result<Option<u64>, ()> {
    if first {
        return Ok(next);
    }
    match (total, next) {
        (Some(total), Some(next)) => total.checked_add(next).map(Some).ok_or(()),
        _ => Ok(None),
    }
}

fn add_counts(
    counts: &mut UsageTokenCounts,
    request: &UsageRecordRequest,
    first: bool,
) -> Result<(), ()> {
    counts.input_tokens = checked_sum(counts.input_tokens, request.input_tokens, first)?;
    counts.output_tokens = checked_sum(counts.output_tokens, request.output_tokens, first)?;
    counts.cached_input_tokens = checked_sum(
        counts.cached_input_tokens,
        request.cached_input_tokens,
        first,
    )?;
    counts.reasoning_tokens =
        checked_sum(counts.reasoning_tokens, request.reasoning_tokens, first)?;
    Ok(())
}

fn summarize(
    root: &Path,
    work_item_id: &str,
    cutoff: String,
    reported_model: Option<&str>,
    records: &[(UsageReceipt, UsageReceiptRef)],
) -> Result<UsageSummary, ObserverError> {
    let cutoff_time = DateTime::parse_from_rfc3339(&cutoff)
        .map_err(|_| state_error(root, "usage cutoff must be RFC3339 with offset"))?;
    let mut summary = UsageSummary::unknown(work_item_id, cutoff, "no_usage_receipts");
    let mut groups: BTreeMap<(Option<String>, String, String), UsageSubtotal> = BTreeMap::new();
    let mut count = 0_u64;
    let mut counts_unknown = false;
    let mut known_sums = [0_u64; 4];
    for (receipt, reference) in records {
        if receipt.request.work_item_id != work_item_id
            || reported_model
                .is_some_and(|model| receipt.request.reported_model.as_deref() != Some(model))
            || DateTime::parse_from_rfc3339(&receipt.received_at)
                .expect("validated receipt timestamp")
                > cutoff_time
        {
            continue;
        }
        let request = &receipt.request;
        for (sum, value) in known_sums.iter_mut().zip([
            request.input_tokens,
            request.output_tokens,
            request.cached_input_tokens,
            request.reasoning_tokens,
        ]) {
            if let Some(value) = value {
                *sum = sum
                    .checked_add(value)
                    .ok_or_else(|| state_error(root, "usage known token sum overflows"))?;
            }
        }
        add_counts(&mut summary.totals, request, count == 0)
            .map_err(|_| state_error(root, "usage token totals overflow"))?;
        count = count
            .checked_add(1)
            .ok_or_else(|| state_error(root, "usage record count overflow"))?;
        let key = (
            request.reported_model.clone(),
            request.role.clone(),
            request.phase.clone(),
        );
        let subtotal = groups.entry(key.clone()).or_insert_with(|| UsageSubtotal {
            reported_model: key.0,
            role: key.1,
            phase: key.2,
            record_count: 0,
            counts: UsageTokenCounts::default(),
        });
        add_counts(&mut subtotal.counts, request, subtotal.record_count == 0)
            .map_err(|_| state_error(root, "usage subtotal overflows"))?;
        subtotal.record_count = subtotal
            .record_count
            .checked_add(1)
            .ok_or_else(|| state_error(root, "usage subtotal count overflow"))?;
        counts_unknown |= request.input_tokens.is_none()
            || request.output_tokens.is_none()
            || request.cached_input_tokens.is_none()
            || request.reasoning_tokens.is_none();
        summary.receipt_refs.push(reference.clone());
    }
    if count > 0 {
        summary.coverage = UsageCoverage::Partial;
        summary.unknown_reasons = vec!["caller_reported_usage_only".into()];
        if counts_unknown {
            summary.unknown_reasons.push("token_count_unknown".into());
        }
        summary.subtotals = groups.into_values().collect();
    }
    Ok(summary)
}

/// Append one caller-submitted usage claim through a fresh Runtime admission.
/// No caller-supplied source kind can raise model or count assurance.
pub fn record_work_item_usage(
    root: &Path,
    request: &UsageRecordRequest,
    runtime: &RuntimeContext,
) -> Result<UsageReceipt, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| read_error(root, source))?;
    request
        .validate()
        .map_err(|message| state_error(&root, message))?;
    if request.repository_id != repository_id(&root).to_string() {
        return Err(state_error(&root, "usage repository identity mismatch"));
    }
    if request.source_observed_at.is_some() {
        return Err(state_error(
            &root,
            "caller sourceObservedAt has no trusted adapter provenance",
        ));
    }
    let _lifecycle_lock = acquire_lifecycle_lock(&root, &request.work_item_id)?;
    require_current_action_admission(&root, &request.work_item_id, "record_usage", runtime)?;
    let evidence = super::collaboration::read_registered_worktree_file_bounded(
        &root,
        &request.evidence_ref,
        MAX_USAGE_EVIDENCE_BYTES,
    )
    .map_err(|message| state_error(&root.join(&request.evidence_ref), message))?;
    if Digest::sha256_bytes(&evidence) != request.evidence_digest {
        return Err(state_error(
            &root.join(&request.evidence_ref),
            "usage evidence is too large or its digest differs from observed bytes",
        ));
    }
    let lock = usage_lock(&root, true)?;
    lock.lock()
        .map_err(|source| read_error(&root.join(".ai/locks/usage.lock"), source))?;
    let usage = usage_directory(&root, true)?.expect("created usage directory");
    let records = receipt_records(&root, &usage)?;
    if let Some((existing, _)) = records
        .iter()
        .find(|(receipt, _)| receipt.request.source_event_id == request.source_event_id)
    {
        if existing.request.work_item_id == request.work_item_id && existing.request == *request {
            return Ok(existing.clone());
        }
        return Err(state_error(
            &root.join(".ai/evidence/usage"),
            "usage source event is bound to different content, source kind, or Work Item",
        ));
    }
    if records.iter().any(|(receipt, _)| {
        receipt.request.work_item_id == request.work_item_id && receipt.request.unit != request.unit
    }) {
        return Err(state_error(
            &root.join(".ai/evidence/usage"),
            "mixed invocation and turn units could overlap within one Work Item",
        ));
    }
    let prospective = UsageReceipt {
        schema_version: USAGE_SCHEMA_VERSION,
        receipt_id: Digest::sha256_bytes(
            &serde_json::to_vec(request).expect("validated usage request serializes"),
        ),
        request: request.clone(),
        received_at: now_nanos(),
        source_observed_at: None,
        model_assurance: receipt_assurance(request).0,
        token_assurance: receipt_assurance(request).1,
    };
    let mut candidate = records.clone();
    candidate.push((
        prospective.clone(),
        UsageReceiptRef {
            path: String::new(),
            digest: Digest::sha256_bytes(b"pending usage receipt"),
        },
    ));
    summarize(
        &root,
        &request.work_item_id,
        prospective.received_at.clone(),
        None,
        &candidate,
    )?;
    let work_item_path = root.join(".ai/evidence/usage").join(&request.work_item_id);
    let work_item = create_and_open_cap_directory(&usage, &request.work_item_id, &work_item_path)?;
    let name = receipt_name(request);
    let path = work_item_path.join(&name);
    let bytes = serde_json::to_vec_pretty(&prospective)
        .map_err(|error| state_error(&path, error.to_string()))?;
    write_usage_receipt_immutable(&work_item, &name, &path, &bytes)?;
    Ok(prospective)
}

/// Read immutable receipt metadata and nullable totals without writing state.
pub fn read_work_item_usage(
    root: &Path,
    work_item_id: &str,
    cutoff: Option<&str>,
) -> Result<UsageSummary, ObserverError> {
    query_work_item_usage(root, work_item_id, cutoff, None)
}

/// Apply an exact filter to the reported model. The configured model is
/// retained in receipts but never substituted for the reported model.
pub fn query_work_item_usage(
    root: &Path,
    work_item_id: &str,
    cutoff: Option<&str>,
    reported_model: Option<&str>,
) -> Result<UsageSummary, ObserverError> {
    validate_work_item_id(work_item_id)?;
    if reported_model.is_some_and(str::is_empty) {
        return Err(state_error(root, "reported model filter is empty"));
    }
    let root = fs::canonicalize(root).map_err(|source| read_error(root, source))?;
    let Some(usage) = usage_directory(&root, false)? else {
        let cutoff = cutoff.map(str::to_owned).unwrap_or_else(now_nanos);
        DateTime::parse_from_rfc3339(&cutoff)
            .map_err(|_| state_error(&root, "usage cutoff must be RFC3339 with offset"))?;
        return Ok(UsageSummary::unknown(
            work_item_id,
            cutoff,
            "no_usage_receipts",
        ));
    };
    let lock = usage_lock(&root, false)?;
    fs::File::lock_shared(&lock)
        .map_err(|source| read_error(&root.join(".ai/locks/usage.lock"), source))?;
    let cutoff = cutoff.map(str::to_owned).unwrap_or_else(now_nanos);
    let records = receipt_records(&root, &usage)?;
    summarize(&root, work_item_id, cutoff, reported_model, &records)
}

/// Return typed receipt metadata without loading the referenced source
/// payload. Callers can verify each receipt against the query's digest refs.
pub fn read_work_item_usage_receipts(
    root: &Path,
    work_item_id: &str,
    cutoff: Option<&str>,
) -> Result<Vec<UsageReceipt>, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| read_error(root, source))?;
    let cutoff = cutoff.map(str::to_owned).unwrap_or_else(now_nanos);
    let cutoff_time = DateTime::parse_from_rfc3339(&cutoff)
        .map_err(|_| state_error(&root, "usage cutoff must be RFC3339 with offset"))?;
    let Some(usage) = usage_directory(&root, false)? else {
        return Ok(Vec::new());
    };
    let lock = usage_lock(&root, false)?;
    fs::File::lock_shared(&lock)
        .map_err(|source| read_error(&root.join(".ai/locks/usage.lock"), source))?;
    let records = receipt_records(&root, &usage)?;
    Ok(records
        .into_iter()
        .filter(|(receipt, _)| {
            receipt.request.work_item_id == work_item_id
                && DateTime::parse_from_rfc3339(&receipt.received_at)
                    .expect("validated receipt timestamp")
                    <= cutoff_time
        })
        .map(|(receipt, _)| receipt)
        .collect())
}
