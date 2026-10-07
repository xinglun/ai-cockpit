use cockpit_protocol::{AuditQueryFilters, AuditQueryPage};

#[test]
fn audit_filters_are_strict_and_keep_source_and_display_time_distinct() {
    let filters = AuditQueryFilters {
        work_item_id: Some("WI-AUDIT".into()),
        from: Some("2026-10-07T00:00:00+09:00".into()),
        display_timezone: Some("Asia/Tokyo".into()),
        ..Default::default()
    };
    let value = serde_json::to_value(&filters).expect("filters JSON");
    assert_eq!(value["from"], "2026-10-07T00:00:00+09:00");
    assert_eq!(value["displayTimezone"], "Asia/Tokyo");
    assert_eq!(
        serde_json::from_value::<AuditQueryFilters>(value).expect("round trip"),
        filters
    );
    assert!(
        serde_json::from_value::<AuditQueryFilters>(serde_json::json!({
            "workItemId":"WI-AUDIT", "unexpected":"not permitted"
        }))
        .is_err()
    );
}

#[test]
fn query_page_preserves_nullable_unknown_counts_and_source_fields() {
    let value = serde_json::json!({
        "schemaVersion":1,
        "repositoryId":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "runtimeVersion":"1.0.1",
        "runtimeDigest":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "asOf":"2026-10-07T00:00:00Z",
        "sourceSnapshotDigest":"sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "filters":{},
        "items":[],
        "coverage":{"state":"unknown","knownCount":0,"unknownCount":null,"unknownSources":[],"unknownReasons":["source_indeterminate"]},
        "pageUsageTotals":{"inputTokens":null,"outputTokens":null,"cachedInputTokens":null,"reasoningTokens":null},
        "returnedCount":0,"truncated":false,"nextCursor":null
    });
    let page: AuditQueryPage = serde_json::from_value(value.clone()).expect("typed page");
    assert_eq!(page.coverage.unknown_count, None);
    assert_eq!(page.page_usage_totals.input_tokens, None);
    assert_eq!(serde_json::to_value(page).expect("round trip"), value);
}
