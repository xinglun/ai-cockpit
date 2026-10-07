use super::{ObserverError, export_audit_events, repository_id, usage, validate_work_item_id};
use chrono::{DateTime, SecondsFormat, Utc};
use chrono_tz::Tz;
use cockpit_core::Digest;
use cockpit_protocol::{
    AUDIT_QUERY_SCHEMA_VERSION, AuditEvidenceRef, AuditQueryCoverage, AuditQueryFilters,
    AuditQueryItem, AuditQueryPage, RuntimeContext, UsageAssurance, UsageCoverage, UsageSubtotal,
    UsageTokenCounts,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

fn lifecycle_summary_items(root: &Path, zone: &str) -> Result<Vec<AuditQueryItem>, ObserverError> {
    let mut items = Vec::new();
    for phase in ["active", "archive"] {
        let directory = root.join(format!(".ai/work-items/{phase}"));
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(ObserverError::Read {
                    path: directory,
                    source,
                });
            }
        };
        for entry in entries {
            let entry = entry.map_err(|source| ObserverError::Read {
                path: directory.clone(),
                source,
            })?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| error(root, "audit summary filename is not UTF-8"))?;
            let Some(work_item_id) = name.strip_suffix(".summary.json") else {
                continue;
            };
            validate_work_item_id(work_item_id)?;
            if !entry
                .file_type()
                .map_err(|source| ObserverError::Read {
                    path: entry.path(),
                    source,
                })?
                .is_file()
            {
                return Err(error(root, "audit summary must be a regular file"));
            }
            let relative = format!(".ai/work-items/{phase}/{name}");
            let bytes = super::collaboration::read_registered_worktree_file_bounded(
                root,
                &relative,
                4 * 1024 * 1024,
            )
            .map_err(|message| error(root, message))?;
            let value: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|source| error(root, format!("invalid audit summary: {source}")))?;
            if value["workItemId"] != work_item_id
                || value["repositoryId"] != repository_id(root).to_string()
            {
                return Err(error(root, "audit summary identity differs"));
            }
            let source_digest = Digest::sha256_bytes(&bytes);
            for (key, event_type) in [
                ("start", "work_item_started"),
                ("finish", "work_item_finished"),
            ] {
                let Some(fact) = value["lifecycleFacts"].get(key) else {
                    continue;
                };
                if fact["eventType"] != event_type || fact["actorProvenance"] != "unknown" {
                    return Err(error(
                        root,
                        "audit lifecycle fact identity or provenance differs",
                    ));
                }
                let occurred = fact["occurredAt"]
                    .as_str()
                    .ok_or_else(|| error(root, "audit lifecycle occurredAt missing"))?;
                let recorded = fact["recordedAt"]
                    .as_str()
                    .ok_or_else(|| error(root, "audit lifecycle recordedAt missing"))?;
                let occurred = DateTime::parse_from_rfc3339(occurred)
                    .map_err(|_| error(root, "audit lifecycle occurredAt invalid"))?
                    .with_timezone(&Utc)
                    .to_rfc3339_opts(SecondsFormat::Nanos, true);
                let recorded = DateTime::parse_from_rfc3339(recorded)
                    .map_err(|_| error(root, "audit lifecycle recordedAt invalid"))?
                    .with_timezone(&Utc)
                    .to_rfc3339_opts(SecondsFormat::Nanos, true);
                let event_id = cockpit_protocol::digest_json(&serde_json::json!({
                    "repositoryId": repository_id(root).to_string(), "workItemId": work_item_id,
                    "eventType": event_type, "recordedAt": recorded,
                }))
                .map_err(|source| error(root, source.to_string()))?
                .to_string();
                let (event_runtime_version, event_runtime_digest) =
                    source_runtime_identity(root, fact)?;
                items.push(AuditQueryItem {
                    event_id,
                    event_type: event_type.into(),
                    work_item_id: Some(work_item_id.into()),
                    occurred_at: Some(occurred),
                    recorded_at: Some(recorded.clone()),
                    source_observed_at: None,
                    received_at: None,
                    reported_model: None,
                    configured_model: None,
                    actor: None,
                    actor_provenance: "unknown".into(),
                    event_runtime_version,
                    event_runtime_digest,
                    role: None,
                    phase: None,
                    source_kind: None,
                    model_assurance: None,
                    token_assurance: None,
                    wall_elapsed_ms: None,
                    token_counts: UsageTokenCounts::default(),
                    evidence_refs: vec![AuditEvidenceRef {
                        path: relative.clone(),
                        digest: Some(source_digest.clone()),
                    }],
                    display_timezone: zone.into(),
                    display_time: Some(recorded),
                });
            }
        }
    }
    Ok(items)
}

fn lifecycle_boundary_items(root: &Path, zone: &str) -> Result<Vec<AuditQueryItem>, ObserverError> {
    let mut items = Vec::new();
    for (directory_relative, suffix, expected_type, expected_state) in [
        (
            ".ai/work-items/archive",
            ".archive.json",
            "work_item_archived",
            "archived",
        ),
        (".ai/decisions", ".close.json", "work_item_closed", "closed"),
    ] {
        let directory = root.join(directory_relative);
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(ObserverError::Read {
                    path: directory,
                    source,
                });
            }
        };
        for entry in entries {
            let entry = entry.map_err(|source| ObserverError::Read {
                path: directory.clone(),
                source,
            })?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| error(root, "audit boundary filename is not UTF-8"))?;
            let Some(work_item_id) = name.strip_suffix(suffix) else {
                continue;
            };
            validate_work_item_id(work_item_id)?;
            if !entry
                .file_type()
                .map_err(|source| ObserverError::Read {
                    path: entry.path(),
                    source,
                })?
                .is_file()
            {
                return Err(error(
                    root,
                    "audit lifecycle boundary must be a regular file",
                ));
            }
            let relative = format!("{directory_relative}/{name}");
            let bytes = super::collaboration::read_registered_worktree_file_bounded(
                root,
                &relative,
                4 * 1024 * 1024,
            )
            .map_err(|message| error(root, message))?;
            let value: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|source| error(root, format!("invalid audit boundary: {source}")))?;
            // Historical records with no Runtime-recorded lifecycle fact stay
            // unknown; createdAt and human decidedAt are not substitutes.
            let occurred = value.get("occurredAt");
            let recorded = value.get("recordedAt");
            if occurred.is_none() && recorded.is_none() {
                continue;
            }
            let (Some(occurred), Some(recorded)) = (
                occurred.and_then(serde_json::Value::as_str),
                recorded.and_then(serde_json::Value::as_str),
            ) else {
                return Err(error(
                    root,
                    "audit lifecycle boundary has a partial Runtime clock fact",
                ));
            };
            if value["workItemId"] != work_item_id
                || value["repositoryId"] != repository_id(root).to_string()
                || value["state"] != expected_state
            {
                return Err(error(root, "audit lifecycle boundary identity differs"));
            }
            if expected_type == "work_item_closed"
                && !super::close_decision_is_valid_for_status(
                    root,
                    work_item_id,
                    &repository_id(root).to_string(),
                )
            {
                return Err(error(root, "audit close decision is not valid"));
            }
            let occurred = DateTime::parse_from_rfc3339(occurred)
                .map_err(|_| error(root, "audit lifecycle occurredAt invalid"))?
                .with_timezone(&Utc)
                .to_rfc3339_opts(SecondsFormat::Nanos, true);
            let recorded = DateTime::parse_from_rfc3339(recorded)
                .map_err(|_| error(root, "audit lifecycle recordedAt invalid"))?
                .with_timezone(&Utc)
                .to_rfc3339_opts(SecondsFormat::Nanos, true);
            let actor = if expected_type == "work_item_closed" {
                value["structuredDecision"]["actor"]
                    .as_str()
                    .map(str::to_owned)
            } else {
                None
            };
            let event_id = cockpit_protocol::digest_json(&serde_json::json!({
                "repositoryId": repository_id(root).to_string(), "workItemId": work_item_id,
                "eventType": expected_type, "recordedAt": recorded,
            }))
            .map_err(|source| error(root, source.to_string()))?
            .to_string();
            let (event_runtime_version, event_runtime_digest) =
                source_runtime_identity(root, &value)?;
            items.push(AuditQueryItem {
                event_id,
                event_type: expected_type.into(),
                work_item_id: Some(work_item_id.into()),
                occurred_at: Some(occurred),
                recorded_at: Some(recorded.clone()),
                source_observed_at: None,
                received_at: None,
                reported_model: None,
                configured_model: None,
                actor,
                actor_provenance: if expected_type == "work_item_closed" {
                    "structured_decision".into()
                } else {
                    "unknown".into()
                },
                event_runtime_version,
                event_runtime_digest,
                role: None,
                phase: None,
                source_kind: None,
                model_assurance: None,
                token_assurance: None,
                wall_elapsed_ms: None,
                token_counts: UsageTokenCounts::default(),
                evidence_refs: vec![AuditEvidenceRef {
                    path: relative,
                    digest: Some(Digest::sha256_bytes(&bytes)),
                }],
                display_timezone: zone.into(),
                display_time: Some(recorded),
            });
        }
    }
    Ok(items)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PageCursor {
    snapshot: Digest,
    filters: Digest,
    last_event_id: String,
}

fn error(root: &Path, message: impl Into<String>) -> ObserverError {
    ObserverError::State {
        path: root.join(".ai/audit"),
        message: message.into(),
    }
}

fn source_runtime_identity(
    root: &Path,
    value: &serde_json::Value,
) -> Result<(Option<String>, Option<Digest>), ObserverError> {
    match (value.get("runtimeVersion"), value.get("runtimeDigest")) {
        (None, None) => Ok((None, None)),
        (Some(version), Some(digest)) => {
            let version = version
                .as_str()
                .filter(|version| !version.is_empty())
                .ok_or_else(|| error(root, "audit source runtimeVersion is invalid"))?;
            let digest: Digest = serde_json::from_value(digest.clone())
                .map_err(|_| error(root, "audit source runtimeDigest is invalid"))?;
            Ok((Some(version.into()), Some(digest)))
        }
        _ => Err(error(root, "audit source Runtime identity is partial")),
    }
}

fn legacy_evidence_refs(
    root: &Path,
    event: &cockpit_protocol::AuditEvent,
    cursor_present: bool,
) -> Result<Vec<AuditEvidenceRef>, ObserverError> {
    if event.event_type != "external_evidence_bound" {
        return Ok(event
            .evidence_refs
            .iter()
            .map(|path| AuditEvidenceRef {
                path: path.clone(),
                digest: None,
            })
            .collect());
    }
    let [receipt_ref, raw_ref] = event.evidence_refs.as_slice() else {
        return Err(error(
            root,
            "delegated audit event has incomplete source refs",
        ));
    };
    let (receipt, receipt_digest) = super::validated_delegated_audit_source(root, receipt_ref)
        .map_err(|source| {
            if cursor_present {
                error(root, format!("stale_cursor: {source}"))
            } else {
                source
            }
        })?;
    if receipt.repository_id != repository_id(root).to_string()
        || event.work_item_id.as_deref() != Some(receipt.work_item_id.as_str())
        || receipt.evidence.raw_evidence_ref != *raw_ref
        || receipt.evidence.digest != event.digest
    {
        return Err(error(root, "delegated audit receipt binding differs"));
    }
    let raw_digest = receipt.evidence.digest;
    Ok(vec![
        AuditEvidenceRef {
            path: receipt_ref.clone(),
            digest: Some(receipt_digest),
        },
        AuditEvidenceRef {
            path: raw_ref.clone(),
            digest: Some(raw_digest),
        },
    ])
}

fn parsed_bound(root: &Path, value: Option<&str>) -> Result<Option<DateTime<Utc>>, ObserverError> {
    value
        .map(|value| {
            DateTime::parse_from_rfc3339(value)
                .map(|time| time.with_timezone(&Utc))
                .map_err(|_| error(root, "audit query time bound must be RFC3339 with offset"))
        })
        .transpose()
}

fn add_count(left: Option<u64>, right: Option<u64>) -> Result<Option<u64>, ()> {
    match (left, right) {
        (Some(left), Some(right)) => left.checked_add(right).map(Some).ok_or(()),
        _ => Ok(None),
    }
}

fn page_usage_totals(
    root: &Path,
    items: &[AuditQueryItem],
) -> Result<UsageTokenCounts, ObserverError> {
    let mut totals = UsageTokenCounts::default();
    let mut first = true;
    for item in items
        .iter()
        .filter(|item| item.event_type == "usage_recorded")
    {
        if first {
            totals = item.token_counts.clone();
            first = false;
            continue;
        }
        totals.input_tokens = add_count(totals.input_tokens, item.token_counts.input_tokens)
            .map_err(|_| error(root, "audit page input tokens overflow"))?;
        totals.output_tokens = add_count(totals.output_tokens, item.token_counts.output_tokens)
            .map_err(|_| error(root, "audit page output tokens overflow"))?;
        totals.cached_input_tokens = add_count(
            totals.cached_input_tokens,
            item.token_counts.cached_input_tokens,
        )
        .map_err(|_| error(root, "audit page cached tokens overflow"))?;
        totals.reasoning_tokens =
            add_count(totals.reasoning_tokens, item.token_counts.reasoning_tokens)
                .map_err(|_| error(root, "audit page reasoning tokens overflow"))?;
    }
    Ok(totals)
}

fn page_usage_subtotals(
    root: &Path,
    items: &[AuditQueryItem],
) -> Result<Vec<UsageSubtotal>, ObserverError> {
    let mut groups = std::collections::BTreeMap::new();
    for item in items
        .iter()
        .filter(|item| item.event_type == "usage_recorded")
    {
        let key = (
            item.reported_model.clone(),
            item.role
                .clone()
                .ok_or_else(|| error(root, "usage role missing"))?,
            item.phase
                .clone()
                .ok_or_else(|| error(root, "usage phase missing"))?,
        );
        let subtotal = groups.entry(key.clone()).or_insert_with(|| UsageSubtotal {
            reported_model: key.0,
            configured_models: Vec::new(),
            source_kinds: Vec::new(),
            model_assurance: item.model_assurance.unwrap_or(UsageAssurance::Unknown),
            token_assurance: item.token_assurance.unwrap_or(UsageAssurance::Unknown),
            role: key.1,
            phase: key.2,
            record_count: 0,
            counts: UsageTokenCounts::default(),
        });
        if let Some(model) = item.configured_model.as_ref()
            && !subtotal.configured_models.contains(model)
        {
            subtotal.configured_models.push(model.clone());
            subtotal.configured_models.sort();
        }
        if let Some(kind) = item.source_kind
            && !subtotal.source_kinds.contains(&kind)
        {
            subtotal.source_kinds.push(kind);
            subtotal.source_kinds.sort();
        }
        if subtotal.model_assurance != item.model_assurance.unwrap_or(UsageAssurance::Unknown) {
            subtotal.model_assurance = UsageAssurance::Unknown;
        }
        if subtotal.token_assurance != item.token_assurance.unwrap_or(UsageAssurance::Unknown) {
            subtotal.token_assurance = UsageAssurance::Unknown;
        }
        if subtotal.record_count == 0 {
            subtotal.counts = item.token_counts.clone();
        } else {
            subtotal.counts.input_tokens =
                add_count(subtotal.counts.input_tokens, item.token_counts.input_tokens)
                    .map_err(|_| error(root, "audit subtotal input tokens overflow"))?;
            subtotal.counts.output_tokens = add_count(
                subtotal.counts.output_tokens,
                item.token_counts.output_tokens,
            )
            .map_err(|_| error(root, "audit subtotal output tokens overflow"))?;
            subtotal.counts.cached_input_tokens = add_count(
                subtotal.counts.cached_input_tokens,
                item.token_counts.cached_input_tokens,
            )
            .map_err(|_| error(root, "audit subtotal cached tokens overflow"))?;
            subtotal.counts.reasoning_tokens = add_count(
                subtotal.counts.reasoning_tokens,
                item.token_counts.reasoning_tokens,
            )
            .map_err(|_| error(root, "audit subtotal reasoning tokens overflow"))?;
        }
        subtotal.record_count = subtotal
            .record_count
            .checked_add(1)
            .ok_or_else(|| error(root, "audit subtotal record count overflow"))?;
    }
    Ok(groups.into_values().collect())
}

/// Read existing audit sources and usage receipts without creating repository
/// evidence. Temporal filters use only Runtime recordedAt.
pub fn query_audit_events(
    root: &Path,
    runtime: &RuntimeContext,
    filters: &AuditQueryFilters,
) -> Result<AuditQueryPage, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if let Some(work_item_id) = filters.work_item_id.as_deref() {
        validate_work_item_id(work_item_id)?;
    }
    for value in [
        filters.reported_model.as_deref(),
        filters.actor.as_deref(),
        filters.event_type.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if value.is_empty() {
            return Err(error(&root, "audit exact filter cannot be empty"));
        }
    }
    let limit = filters.limit.unwrap_or(50);
    if !(1..=100).contains(&limit) {
        return Err(error(&root, "audit query limit must be 1..=100"));
    }
    let from = parsed_bound(&root, filters.from.as_deref())?;
    let to = parsed_bound(&root, filters.to.as_deref())?;
    if from.zip(to).is_some_and(|(from, to)| from >= to) {
        return Err(error(&root, "audit query from must be earlier than to"));
    }
    let zone = filters.display_timezone.as_deref().unwrap_or("UTC");
    let display_zone: Tz = zone
        .parse()
        .map_err(|_| error(&root, "unsupported IANA displayTimezone"))?;
    let mut normalized = filters.clone();
    normalized.cursor = None;
    normalized.limit = Some(limit);
    normalized.display_timezone = Some(zone.into());
    normalized.from = from.map(|time| time.to_rfc3339_opts(SecondsFormat::AutoSi, true));
    normalized.to = to.map(|time| time.to_rfc3339_opts(SecondsFormat::AutoSi, true));
    let normalized_digest = cockpit_protocol::digest_json(&normalized)
        .map_err(|source| error(&root, source.to_string()))?;

    let legacy = export_audit_events(&root, runtime).map_err(|source| {
        if filters.cursor.is_some() {
            error(&root, format!("stale_cursor: {source}"))
        } else {
            source
        }
    })?;
    let mut items = legacy
        .events
        .into_iter()
        .map(|event| -> Result<AuditQueryItem, ObserverError> {
            let evidence_refs = legacy_evidence_refs(&root, &event, filters.cursor.is_some())?;
            Ok(AuditQueryItem {
                event_id: event.event_id,
                event_type: event.event_type,
                work_item_id: event.work_item_id,
                occurred_at: DateTime::parse_from_rfc3339(&event.timestamp)
                    .ok()
                    .map(|time| {
                        time.with_timezone(&Utc)
                            .to_rfc3339_opts(SecondsFormat::Nanos, true)
                    }),
                recorded_at: None,
                source_observed_at: None,
                received_at: None,
                reported_model: None,
                configured_model: None,
                actor: None,
                actor_provenance: "unknown".into(),
                event_runtime_version: None,
                event_runtime_digest: None,
                role: None,
                phase: None,
                source_kind: None,
                model_assurance: None,
                token_assurance: None,
                wall_elapsed_ms: None,
                token_counts: UsageTokenCounts::default(),
                evidence_refs,
                display_timezone: zone.into(),
                display_time: None,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    items.extend(lifecycle_summary_items(&root, zone)?);
    items.extend(lifecycle_boundary_items(&root, zone)?);
    for (receipt, reference) in usage::all_usage_receipts(&root)? {
        let request = receipt.request;
        let source_bytes = super::collaboration::read_registered_worktree_file_bounded(
            &root,
            &request.evidence_ref,
            4 * 1024 * 1024,
        )
        .map_err(|message| {
            if filters.cursor.is_some() {
                error(&root, "stale_cursor: referenced usage source changed")
            } else {
                error(
                    &root,
                    format!("usage source evidence is unavailable: {message}"),
                )
            }
        })?;
        if Digest::sha256_bytes(&source_bytes) != request.evidence_digest {
            return Err(if filters.cursor.is_some() {
                error(&root, "stale_cursor: referenced usage source changed")
            } else {
                error(&root, "usage source evidence digest differs")
            });
        }
        let recorded_at = DateTime::parse_from_rfc3339(&receipt.received_at)
            .map_err(|_| error(&root, "invalid usage receivedAt"))?
            .with_timezone(&Utc)
            .to_rfc3339_opts(SecondsFormat::Nanos, true);
        items.push(AuditQueryItem {
            event_id: receipt.receipt_id.to_string(),
            event_type: "usage_recorded".into(),
            work_item_id: Some(request.work_item_id),
            occurred_at: None,
            recorded_at: Some(recorded_at.clone()),
            source_observed_at: receipt.source_observed_at,
            received_at: Some(recorded_at.clone()),
            reported_model: request.reported_model,
            configured_model: request.configured_model,
            actor: request.actor,
            actor_provenance: "caller_claim".into(),
            event_runtime_version: None,
            event_runtime_digest: None,
            role: Some(request.role),
            phase: Some(request.phase),
            source_kind: Some(request.source_kind),
            model_assurance: Some(receipt.model_assurance),
            token_assurance: Some(receipt.token_assurance),
            wall_elapsed_ms: None,
            token_counts: UsageTokenCounts {
                input_tokens: request.input_tokens,
                output_tokens: request.output_tokens,
                cached_input_tokens: request.cached_input_tokens,
                reasoning_tokens: request.reasoning_tokens,
            },
            evidence_refs: vec![
                AuditEvidenceRef {
                    path: reference.path,
                    digest: Some(reference.digest),
                },
                AuditEvidenceRef {
                    path: request.evidence_ref,
                    digest: Some(request.evidence_digest),
                },
            ],
            display_timezone: zone.into(),
            display_time: Some(recorded_at),
        });
    }
    for item in &mut items {
        item.display_time = item
            .recorded_at
            .as_deref()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&display_zone).to_rfc3339());
    }
    let starts = items
        .iter()
        .filter(|item| item.event_type == "work_item_started")
        .filter_map(|item| Some((item.work_item_id.clone()?, item.recorded_at.clone()?)))
        .collect::<std::collections::BTreeMap<_, _>>();
    for item in &mut items {
        if item.event_type != "work_item_closed" {
            continue;
        }
        let Some(started) = item.work_item_id.as_ref().and_then(|id| starts.get(id)) else {
            continue;
        };
        let Some((start, end)) = DateTime::parse_from_rfc3339(started).ok().zip(
            item.recorded_at
                .as_deref()
                .and_then(|time| DateTime::parse_from_rfc3339(time).ok()),
        ) else {
            continue;
        };
        item.wall_elapsed_ms = (end - start).num_milliseconds().try_into().ok();
    }
    items.sort_by(|left, right| {
        (&left.recorded_at, &left.event_id).cmp(&(&right.recorded_at, &right.event_id))
    });
    let source_items = items
        .iter()
        .cloned()
        .map(|mut item| {
            item.display_timezone.clear();
            item.display_time = None;
            item
        })
        .collect::<Vec<_>>();
    let source_snapshot_digest = cockpit_protocol::digest_json(&source_items)
        .map_err(|source| error(&root, source.to_string()))?;
    let cursor_last_event_id = if let Some(encoded) = filters.cursor.as_deref() {
        let bytes = hex::decode(encoded).map_err(|_| error(&root, "invalid_cursor"))?;
        let cursor: PageCursor =
            serde_json::from_slice(&bytes).map_err(|_| error(&root, "invalid_cursor"))?;
        if cursor.snapshot != source_snapshot_digest {
            return Err(error(&root, "stale_cursor"));
        }
        if cursor.filters != normalized_digest {
            return Err(error(&root, "invalid_cursor: filters changed"));
        }
        Some(cursor.last_event_id)
    } else {
        None
    };
    let filtered = items
        .into_iter()
        .filter(|item| {
            normalized
                .work_item_id
                .as_ref()
                .is_none_or(|value| item.work_item_id.as_ref() == Some(value))
                && normalized
                    .reported_model
                    .as_ref()
                    .is_none_or(|value| item.reported_model.as_ref() == Some(value))
                && normalized
                    .actor
                    .as_ref()
                    .is_none_or(|value| item.actor.as_ref() == Some(value))
                && normalized
                    .event_type
                    .as_ref()
                    .is_none_or(|value| item.event_type == *value)
        })
        .collect::<Vec<_>>();
    let unknown_sources = filtered
        .iter()
        .filter(|item| item.recorded_at.is_none())
        .flat_map(|item| {
            item.evidence_refs
                .iter()
                .map(|reference| reference.path.clone())
        })
        .collect::<Vec<_>>();
    let known_count = filtered
        .iter()
        .filter(|item| item.recorded_at.is_some())
        .count() as u64;
    let unknown_count = filtered.len() as u64 - known_count;
    let filtered = filtered
        .into_iter()
        .filter(|item| {
            if from.is_none() && to.is_none() {
                return true;
            }
            let Some(time) = item
                .recorded_at
                .as_deref()
                .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            else {
                return false;
            };
            from.is_none_or(|from| time >= from) && to.is_none_or(|to| time < to)
        })
        .collect::<Vec<_>>();
    let start = if let Some(last_event_id) = cursor_last_event_id.as_ref() {
        filtered
            .iter()
            .position(|item| &item.event_id == last_event_id)
            .ok_or_else(|| error(&root, "invalid_cursor: event is missing"))?
            + 1
    } else {
        0
    };
    let page = filtered
        .iter()
        .skip(start)
        .take(limit as usize)
        .cloned()
        .collect::<Vec<_>>();
    let truncated = start + page.len() < filtered.len();
    let next_cursor = if truncated {
        let cursor = PageCursor {
            snapshot: source_snapshot_digest.clone(),
            filters: normalized_digest,
            last_event_id: page.last().expect("nonempty page").event_id.clone(),
        };
        Some(hex::encode(
            serde_json::to_vec(&cursor).map_err(|source| error(&root, source.to_string()))?,
        ))
    } else {
        None
    };
    let page_usage_totals = page_usage_totals(&root, &page)?;
    let page_usage_subtotals = page_usage_subtotals(&root, &page)?;
    Ok(AuditQueryPage {
        schema_version: AUDIT_QUERY_SCHEMA_VERSION,
        repository_id: repository_id(&root).to_string(),
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        as_of: Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true),
        source_snapshot_digest,
        filters: normalized,
        items: page.clone(),
        coverage: AuditQueryCoverage {
            state: if unknown_count > 0 {
                UsageCoverage::Partial
            } else if filtered.is_empty() {
                UsageCoverage::Unknown
            } else {
                UsageCoverage::Complete
            },
            known_count,
            unknown_count: Some(unknown_count),
            unknown_sources,
            unknown_reasons: if unknown_count > 0 {
                vec!["recorded_at_missing".into()]
            } else {
                Vec::new()
            },
        },
        page_usage_totals,
        page_usage_subtotals,
        returned_count: page.len(),
        truncated,
        next_cursor,
    })
}

/// Filtered export deliberately returns the same typed page as audit query.
/// The existing no-filter `export_audit_events` retains its schema-v1 wire
/// contract and explicit local output behavior in the transport adapter.
pub fn export_audit_events_filtered(
    root: &Path,
    runtime: &RuntimeContext,
    filters: &AuditQueryFilters,
) -> Result<AuditQueryPage, ObserverError> {
    query_audit_events(root, runtime, filters)
}
