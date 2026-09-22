use cockpit_core::Digest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[cfg(test)]
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

#[cfg(test)]
static FROM_RECORDS_CALLS: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
static FROM_RECORDS_TEST_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRecord {
    pub work_item_id: String,
    pub topic: String,
    pub component: String,
    pub state: String,
    pub knowledge_path: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeIndex {
    pub records: Vec<KnowledgeRecord>,
    /// Digest of the canonical archived inputs used to build this index.
    /// This is a cache validator only; archived records remain the source of
    /// truth and the index is always reconstructible.
    #[serde(default, rename = "sourceDigest")]
    pub source_digest: String,
    /// Digest of the serialized records.  This protects the derived cache
    /// from accepting a record mutation that leaves the source boundary
    /// unchanged.
    #[serde(default, rename = "recordDigest")]
    pub record_digest: String,
    /// Git revision at which the archive source was content-validated.  A
    /// clean revision allows a cache hit without rereading every archive
    /// file; dirty or unknown source state falls back to content hashing.
    #[serde(default, rename = "sourceRevision")]
    pub source_revision: Option<String>,
    pub dependencies: BTreeMap<String, Vec<String>>,
    pub by_topic: BTreeMap<String, Vec<String>>,
    pub by_component: BTreeMap<String, Vec<String>>,
    pub by_state: BTreeMap<String, Vec<String>>,
    pub by_work_item: BTreeMap<String, String>,
    /// Direct record positions avoid scanning `records` after index
    /// intersection has produced candidate IDs.
    #[serde(default, rename = "recordPositions")]
    pub record_positions: BTreeMap<String, usize>,
}

impl KnowledgeIndex {
    pub fn from_records(mut records: Vec<KnowledgeRecord>) -> Self {
        #[cfg(test)]
        {
            let _guard = FROM_RECORDS_TEST_LOCK.lock().unwrap();
            FROM_RECORDS_CALLS.fetch_add(1, Ordering::Relaxed);
        }
        records.sort_by(|left, right| left.work_item_id.cmp(&right.work_item_id));
        let mut dependencies = BTreeMap::new();
        let mut by_topic = BTreeMap::new();
        let mut by_component = BTreeMap::new();
        let mut by_state = BTreeMap::new();
        let mut by_work_item = BTreeMap::new();
        let mut record_positions = BTreeMap::new();
        for (position, record) in records.iter().enumerate() {
            for evidence_ref in &record.evidence_refs {
                dependencies
                    .entry(evidence_ref.clone())
                    .or_insert_with(Vec::new)
                    .push(record.work_item_id.clone());
            }
            by_topic
                .entry(record.topic.clone())
                .or_insert_with(Vec::new)
                .push(record.work_item_id.clone());
            by_component
                .entry(record.component.clone())
                .or_insert_with(Vec::new)
                .push(record.work_item_id.clone());
            by_state
                .entry(record.state.clone())
                .or_insert_with(Vec::new)
                .push(record.work_item_id.clone());
            by_work_item.insert(record.work_item_id.clone(), record.work_item_id.clone());
            record_positions.insert(record.work_item_id.clone(), position);
        }
        Self {
            record_digest: records_digest(&records),
            records,
            source_digest: String::new(),
            source_revision: None,
            dependencies,
            by_topic,
            by_component,
            by_state,
            by_work_item,
            record_positions,
        }
    }

    pub fn from_records_with_source_digest(
        records: Vec<KnowledgeRecord>,
        source_digest: impl Into<String>,
    ) -> Self {
        let mut index = Self::from_records(records);
        index.source_digest = source_digest.into();
        index
    }

    pub fn with_source_metadata(
        records: Vec<KnowledgeRecord>,
        source_digest: impl Into<String>,
        source_revision: Option<String>,
    ) -> Self {
        let mut index = Self::from_records_with_source_digest(records, source_digest);
        index.source_revision = source_revision;
        index
    }

    pub fn is_structurally_valid(&self) -> bool {
        if self.record_digest.is_empty() || self.record_positions.len() != self.records.len() {
            return false;
        }
        if self.record_digest != records_digest(&self.records) {
            return false;
        }

        let mut ids = std::collections::BTreeSet::new();
        for (position, record) in self.records.iter().enumerate() {
            if !ids.insert(record.work_item_id.clone())
                || self.record_positions.get(&record.work_item_id) != Some(&position)
                || self.by_work_item.get(&record.work_item_id) != Some(&record.work_item_id)
            {
                return false;
            }
        }

        self.by_work_item.len() == self.records.len()
            && validate_single_index(&self.by_topic, &self.records, |record| &record.topic)
            && validate_single_index(&self.by_component, &self.records, |record| {
                &record.component
            })
            && validate_single_index(&self.by_state, &self.records, |record| &record.state)
            && validate_dependency_index(&self.dependencies, &self.records)
    }
}

fn validate_single_index<F>(
    index: &BTreeMap<String, Vec<String>>,
    records: &[KnowledgeRecord],
    key_for: F,
) -> bool
where
    F: for<'a> Fn(&'a KnowledgeRecord) -> &'a str,
{
    let mut offsets = BTreeMap::<&str, usize>::new();
    for record in records {
        let key = key_for(record);
        let Some(ids) = index.get(key) else {
            return false;
        };
        let offset = offsets.entry(key).or_default();
        if ids.get(*offset) != Some(&record.work_item_id) {
            return false;
        }
        *offset += 1;
    }

    index.len() == offsets.len()
        && index
            .iter()
            .all(|(key, ids)| offsets.get(key.as_str()) == Some(&ids.len()))
}

fn validate_dependency_index(
    index: &BTreeMap<String, Vec<String>>,
    records: &[KnowledgeRecord],
) -> bool {
    let mut offsets = BTreeMap::<&str, usize>::new();
    for record in records {
        for key in &record.evidence_refs {
            let Some(ids) = index.get(key) else {
                return false;
            };
            let offset = offsets.entry(key.as_str()).or_default();
            if ids.get(*offset) != Some(&record.work_item_id) {
                return false;
            }
            *offset += 1;
        }
    }

    index.len() == offsets.len()
        && index
            .iter()
            .all(|(key, ids)| offsets.get(key.as_str()) == Some(&ids.len()))
}

fn records_digest(records: &[KnowledgeRecord]) -> String {
    let bytes = serde_json::to_vec(records).expect("KnowledgeRecord serialization is infallible");
    Digest::sha256_bytes(&bytes).to_string()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Query {
    pub topic: Option<String>,
    pub component: Option<String>,
    pub state: Option<String>,
    pub work_item_id: Option<String>,
}

pub fn query(index: &KnowledgeIndex, filter: &Query) -> Vec<KnowledgeRecord> {
    query_with_metrics(index, filter).0
}

fn intersect_candidates(
    candidates: &mut Option<std::collections::BTreeSet<String>>,
    ids: &[String],
) {
    let ids = ids
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    *candidates = Some(candidates.take().map_or(ids.clone(), |current| {
        current.intersection(&ids).cloned().collect()
    }));
}

pub fn query_with_metrics(index: &KnowledgeIndex, filter: &Query) -> (Vec<KnowledgeRecord>, usize) {
    let mut candidates: Option<std::collections::BTreeSet<String>> = None;
    let has_filter = filter.topic.is_some()
        || filter.component.is_some()
        || filter.state.is_some()
        || filter.work_item_id.is_some();
    if let Some(value) = filter.topic.as_ref() {
        let Some(ids) = index.by_topic.get(value) else {
            return (Vec::new(), 0);
        };
        intersect_candidates(&mut candidates, ids);
    }
    if let Some(value) = filter.component.as_ref() {
        let Some(ids) = index.by_component.get(value) else {
            return (Vec::new(), 0);
        };
        intersect_candidates(&mut candidates, ids);
    }
    if let Some(value) = filter.state.as_ref() {
        let Some(ids) = index.by_state.get(value) else {
            return (Vec::new(), 0);
        };
        intersect_candidates(&mut candidates, ids);
    }
    if let Some(value) = filter.work_item_id.as_ref() {
        let Some(id) = index.by_work_item.get(value) else {
            return (Vec::new(), 0);
        };
        intersect_candidates(&mut candidates, std::slice::from_ref(id));
    }
    let candidates = candidates.unwrap_or_else(|| {
        if has_filter {
            std::collections::BTreeSet::new()
        } else {
            index
                .records
                .iter()
                .map(|record| record.work_item_id.clone())
                .collect()
        }
    });
    let accessed = candidates.len();
    let results = if has_filter {
        candidates
            .iter()
            .filter_map(|work_item_id| {
                let position = index
                    .record_positions
                    .get(work_item_id)
                    .copied()
                    .or_else(|| {
                        // A legacy in-memory index may predate recordPositions;
                        // the repository cache path rebuilds it before reuse.
                        index
                            .records
                            .iter()
                            .position(|record| &record.work_item_id == work_item_id)
                    })?;
                let record = index.records.get(position)?;
                (filter
                    .topic
                    .as_ref()
                    .is_none_or(|value| &record.topic == value)
                    && filter
                        .component
                        .as_ref()
                        .is_none_or(|value| &record.component == value)
                    && filter
                        .state
                        .as_ref()
                        .is_none_or(|value| &record.state == value)
                    && filter
                        .work_item_id
                        .as_ref()
                        .is_none_or(|value| &record.work_item_id == value))
                .then(|| record.clone())
            })
            .collect::<Vec<_>>()
    } else {
        index.records.clone()
    };
    (results, accessed)
}

#[derive(Debug, Error)]
pub enum KnowledgeError {
    #[error("knowledge JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn project_record(
    work_item_id: &str,
    intent: &str,
    state: &str,
    evidence_ref: &str,
) -> KnowledgeRecord {
    project_record_with_context(work_item_id, intent, &[], state, evidence_ref)
}

pub fn project_record_with_context(
    work_item_id: &str,
    intent: &str,
    scope: &[String],
    state: &str,
    evidence_ref: &str,
) -> KnowledgeRecord {
    KnowledgeRecord {
        work_item_id: work_item_id.into(),
        topic: derive_topic(intent),
        component: derive_component(scope),
        state: state.into(),
        knowledge_path: format!(".ai/knowledge/{work_item_id}.json"),
        evidence_refs: vec![evidence_ref.into()],
    }
}

pub fn derive_topic(intent: &str) -> String {
    let tokens = intent_tokens(intent);
    const TOPICS: &[&str] = &[
        "knowledge",
        "outcome",
        "release",
        "verification",
        "lifecycle",
        "repository",
        "protocol",
        "governance",
        "performance",
        "cleanup",
        "documentation",
        "compatibility",
        "recovery",
        "security",
    ];
    tokens
        .iter()
        .find(|token| TOPICS.contains(&token.as_str()))
        .cloned()
        .or_else(|| tokens.into_iter().next())
        .unwrap_or_else(|| "unknown".into())
}

pub fn derive_component(scope: &[String]) -> String {
    for raw_scope in scope {
        for candidate in raw_scope.split(';') {
            let mut components = candidate
                .split(['/', '\\'])
                .filter(|part| !part.is_empty() && *part != "**" && *part != "*");
            if components.next() == Some("crates")
                && let Some(crate_name) = components.next()
                && !crate_name.is_empty()
            {
                return crate_name.to_owned();
            }
        }
    }
    "unknown".into()
}

fn intent_tokens(intent: &str) -> Vec<String> {
    const STOP_WORDS: &[&str] = &[
        "a",
        "an",
        "and",
        "for",
        "from",
        "in",
        "of",
        "on",
        "the",
        "to",
        "with",
        "without",
        "repair",
        "fix",
        "improve",
        "add",
        "update",
        "implement",
        "ensure",
        "preserve",
        "make",
        "support",
        "introduce",
        "refactor",
        "separate",
        "derive",
        "build",
        "project",
        "clean",
    ];
    intent
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .filter(|token| !STOP_WORDS.contains(&token.as_str()))
        .collect()
}

/// Project the same archive record into the provenance-aware v2 shape.  The
/// legacy index remains readable; callers opt into v2 when they need to show
/// the snapshot binding and unresolved facts.
pub fn project_record_v2(
    repository_id: &str,
    work_item_id: &str,
    intent: &str,
    state: &str,
    evidence_ref: &str,
    snapshot_digest: cockpit_core::Digest,
) -> cockpit_protocol::KnowledgeV2Record {
    project_record_v2_with_context(
        repository_id,
        work_item_id,
        intent,
        &[],
        state,
        evidence_ref,
        snapshot_digest,
    )
}

pub fn project_record_v2_with_context(
    repository_id: &str,
    work_item_id: &str,
    intent: &str,
    scope: &[String],
    state: &str,
    evidence_ref: &str,
    snapshot_digest: cockpit_core::Digest,
) -> cockpit_protocol::KnowledgeV2Record {
    let component = derive_component(scope);
    cockpit_protocol::KnowledgeV2Record {
        schema_version: 2,
        repository_id: repository_id.into(),
        work_item_id: work_item_id.into(),
        topic: derive_topic(intent),
        component: component.clone(),
        state: state.into(),
        truth_state: cockpit_protocol::TruthState::Derived,
        confidence: "medium".into(),
        knowledge_path: format!(".ai/knowledge/{work_item_id}.v2.json"),
        evidence_refs: vec![evidence_ref.into()],
        unknowns: if component == "unknown" {
            vec!["component_not_observed_from_contract".into()]
        } else {
            Vec::new()
        },
        source_snapshot_digest: snapshot_digest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: &str, topic: &str) -> KnowledgeRecord {
        KnowledgeRecord {
            work_item_id: id.into(),
            topic: topic.into(),
            component: "component".into(),
            state: "archived".into(),
            knowledge_path: format!(".ai/knowledge/{id}.json"),
            evidence_refs: vec![format!(".ai/work-items/archive/{id}.archive.json")],
        }
    }

    #[test]
    fn structural_validation_does_not_rebuild_the_derived_index() {
        let index = KnowledgeIndex::from_records(vec![
            record("WI-2", "release"),
            record("WI-1", "knowledge"),
        ]);
        let _guard = FROM_RECORDS_TEST_LOCK.lock().unwrap();
        FROM_RECORDS_CALLS.store(0, Ordering::Relaxed);

        assert!(index.is_structurally_valid());
        assert_eq!(FROM_RECORDS_CALLS.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn structural_validation_rejects_derived_index_tampering() {
        let mut index = KnowledgeIndex::from_records(vec![record("WI-1", "knowledge")]);
        index.by_topic.get_mut("knowledge").unwrap().clear();

        assert!(!index.is_structurally_valid());
    }
}
