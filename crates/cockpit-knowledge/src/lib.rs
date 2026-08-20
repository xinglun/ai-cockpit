use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

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
    pub dependencies: BTreeMap<String, Vec<String>>,
    pub by_topic: BTreeMap<String, Vec<String>>,
    pub by_component: BTreeMap<String, Vec<String>>,
    pub by_state: BTreeMap<String, Vec<String>>,
    pub by_work_item: BTreeMap<String, String>,
}

impl KnowledgeIndex {
    pub fn from_records(mut records: Vec<KnowledgeRecord>) -> Self {
        records.sort_by(|left, right| left.work_item_id.cmp(&right.work_item_id));
        let mut dependencies = BTreeMap::new();
        let mut by_topic = BTreeMap::new();
        let mut by_component = BTreeMap::new();
        let mut by_state = BTreeMap::new();
        let mut by_work_item = BTreeMap::new();
        for record in &records {
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
        }
        Self {
            records,
            dependencies,
            by_topic,
            by_component,
            by_state,
            by_work_item,
        }
    }
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

pub fn query_with_metrics(index: &KnowledgeIndex, filter: &Query) -> (Vec<KnowledgeRecord>, usize) {
    let mut candidates: Option<std::collections::BTreeSet<String>> = None;
    let has_filter = filter.topic.is_some()
        || filter.component.is_some()
        || filter.state.is_some()
        || filter.work_item_id.is_some();
    let indexes = [
        filter
            .topic
            .as_ref()
            .and_then(|value| index.by_topic.get(value))
            .map(Vec::as_slice),
        filter
            .component
            .as_ref()
            .and_then(|value| index.by_component.get(value))
            .map(Vec::as_slice),
        filter
            .state
            .as_ref()
            .and_then(|value| index.by_state.get(value))
            .map(Vec::as_slice),
        filter
            .work_item_id
            .as_ref()
            .and_then(|value| index.by_work_item.get(value).map(std::slice::from_ref)),
    ];
    for ids in indexes.into_iter().flatten() {
        let ids = ids
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        candidates = Some(candidates.map_or(ids.clone(), |current| {
            current.intersection(&ids).cloned().collect()
        }));
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
    let results = index
        .records
        .iter()
        .filter(|record| candidates.contains(&record.work_item_id))
        .filter(|record| {
            filter
                .topic
                .as_ref()
                .is_none_or(|value| &record.topic == value)
        })
        .filter(|record| {
            filter
                .component
                .as_ref()
                .is_none_or(|value| &record.component == value)
        })
        .filter(|record| {
            filter
                .state
                .as_ref()
                .is_none_or(|value| &record.state == value)
        })
        .filter(|record| {
            filter
                .work_item_id
                .as_ref()
                .is_none_or(|value| &record.work_item_id == value)
        })
        .cloned()
        .collect::<Vec<_>>();
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
    let topic = intent
        .split_whitespace()
        .next()
        .unwrap_or("unknown")
        .trim_matches(':')
        .to_lowercase();
    KnowledgeRecord {
        work_item_id: work_item_id.into(),
        topic,
        component: "unknown".into(),
        state: state.into(),
        knowledge_path: format!(".ai/knowledge/{work_item_id}.json"),
        evidence_refs: vec![evidence_ref.into()],
    }
}
