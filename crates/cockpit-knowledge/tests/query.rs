use cockpit_knowledge::{KnowledgeIndex, KnowledgeRecord, Query, query, query_with_metrics};

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
