use cockpit_core::Digest;
use cockpit_knowledge::{
    KnowledgeIndex, KnowledgeRecord, Query, project_record, project_record_v2_with_context, query,
    query_with_metrics,
};

fn record(id: &str, topic: &str, state: &str) -> KnowledgeRecord {
    KnowledgeRecord {
        work_item_id: id.into(),
        topic: topic.into(),
        component: "OrderService".into(),
        state: state.into(),
        knowledge_path: format!(".ai/knowledge/{id}.json"),
        evidence_refs: vec![format!(".ai/work-items/archive/{id}.archive.json")],
    }
}

#[test]
fn deterministic_query_uses_conjunctive_exact_filters_and_stable_order() {
    let index = KnowledgeIndex::from_records(vec![
        record("WI-2", "orders", "verified"),
        record("WI-1", "orders", "verified"),
    ]);
    let results = query(
        &index,
        &Query {
            topic: Some("orders".into()),
            component: Some("OrderService".into()),
            state: Some("verified".into()),
            work_item_id: None,
        },
    );
    assert_eq!(
        results
            .iter()
            .map(|item| item.work_item_id.as_str())
            .collect::<Vec<_>>(),
        vec!["WI-1", "WI-2"]
    );
}

#[test]
fn empty_result_does_not_infer_historical_existence() {
    let index = KnowledgeIndex::from_records(vec![record("WI-1", "orders", "partial")]);
    let results = query(
        &index,
        &Query {
            topic: Some("payments".into()),
            component: None,
            state: None,
            work_item_id: None,
        },
    );
    assert!(results.is_empty());
}

#[test]
fn unrelated_query_avoids_historical_record_access() {
    let records = (0..10_000)
        .map(|index| record(&format!("WI-{index}"), "orders", "verified"))
        .collect();
    let index = KnowledgeIndex::from_records(records);
    let (results, accessed) = query_with_metrics(
        &index,
        &Query {
            topic: Some("payments".into()),
            component: None,
            state: None,
            work_item_id: None,
        },
    );
    assert!(results.is_empty());
    assert_eq!(accessed, 0);
}

#[test]
fn indexed_candidates_are_materialized_by_position() {
    let index = KnowledgeIndex::from_records(vec![
        record("WI-1", "orders", "verified"),
        record("WI-2", "payments", "verified"),
        record("WI-3", "orders", "partial"),
    ]);
    let (results, accessed) = query_with_metrics(
        &index,
        &Query {
            topic: Some("orders".into()),
            component: Some("OrderService".into()),
            state: Some("verified".into()),
            work_item_id: None,
        },
    );
    assert_eq!(accessed, 1);
    assert_eq!(results[0].work_item_id, "WI-1");
}

#[test]
fn any_missing_explicit_filter_short_circuits_before_other_candidates() {
    let records = (0..10_000)
        .map(|index| record(&format!("WI-{index}"), "orders", "verified"))
        .collect();
    let index = KnowledgeIndex::from_records(records);
    let (results, accessed) = query_with_metrics(
        &index,
        &Query {
            topic: Some("orders".into()),
            component: Some("MissingService".into()),
            state: None,
            work_item_id: None,
        },
    );
    assert!(results.is_empty());
    assert_eq!(accessed, 0);
}

#[test]
fn context_projection_derives_bounded_topic_and_component() {
    let v1 = project_record(
        "WI-KNOWLEDGE",
        "Repair knowledge cache correctness",
        "archived",
        ".ai/work-items/archive/WI-KNOWLEDGE.archive.json",
    );
    let contextual = cockpit_knowledge::project_record_with_context(
        "WI-KNOWLEDGE",
        "Repair knowledge cache correctness",
        &["crates/cockpit-knowledge/src/lib.rs".into()],
        "archived",
        ".ai/work-items/archive/WI-KNOWLEDGE.archive.json",
    );
    assert_eq!(v1.topic, "knowledge");
    assert_eq!(contextual.topic, "knowledge");
    assert_eq!(contextual.component, "cockpit-knowledge");

    let v2 = project_record_v2_with_context(
        "repo",
        "WI-KNOWLEDGE",
        "Repair knowledge cache correctness",
        &["crates/cockpit-knowledge/src/lib.rs".into()],
        "archived",
        ".ai/work-items/archive/WI-KNOWLEDGE.archive.json",
        Digest::sha256_bytes(b"snapshot"),
    );
    assert_eq!(v2.topic, contextual.topic);
    assert_eq!(v2.component, contextual.component);
    assert!(v2.unknowns.is_empty());
}

#[test]
fn missing_context_remains_explicitly_unknown() {
    let record = cockpit_knowledge::project_record_with_context(
        "WI-UNKNOWN",
        "Repair knowledge cache correctness",
        &[],
        "archived",
        "archive.json",
    );
    assert_eq!(record.topic, "knowledge");
    assert_eq!(record.component, "unknown");
}

#[test]
fn index_validates_derived_structures_and_record_digest() {
    let mut index = KnowledgeIndex::from_records(vec![record("WI-1", "orders", "verified")]);
    assert!(index.is_structurally_valid());
    index.records[0].topic = "tampered".into();
    assert!(!index.is_structurally_valid());
}
