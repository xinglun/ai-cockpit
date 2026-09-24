use cockpit_repository::{CollaborationOutcomeProjection, render_collaboration_outcome};

fn projection() -> CollaborationOutcomeProjection {
    CollaborationOutcomeProjection {
        schema_version: 1,
        work_item_id: "WI-PARITY".into(),
        state: "blocked".into(),
        providers: vec!["WI-PROVIDER".into()],
        consumers: vec!["WI-CONSUMER".into()],
        waiting_edges: vec!["WI-PARITY->WI-PROVIDER".into()],
        invalidated_event_ids: vec!["impact-1".into()],
        unhandled_requests: Vec::new(),
        integration_owner: Some("WI-PARITY".into()),
        composition_order: vec!["WI-PROVIDER".into(), "WI-PARITY".into()],
        implementation_state: "separate_lifecycle_outcome".into(),
        composition_state: "not_observed".into(),
        target_merge_state: "not_observed".into(),
        cleanup_state: "not_observed".into(),
        revalidation: "required".into(),
        reusable_checks: vec!["none".into()],
        blockers: vec!["dependency_impact:WI-PROVIDER".into()],
        unknowns: vec!["none".into()],
        human_decision_required: false,
        next_action: "refresh affected dependency and composition evidence".into(),
    }
}

#[test]
fn collaboration_outcome_projection_preserves_semantics_in_all_languages() {
    for language in ["en", "zh", "ja"] {
        let rendered = render_collaboration_outcome(&projection(), language);
        for semantic in [
            "WI-PARITY",
            "WI-PROVIDER",
            "WI-CONSUMER",
            "impact-1",
            "separate_lifecycle_outcome",
            "not_observed",
            "required",
            "dependency_impact:WI-PROVIDER",
            "refresh affected dependency and composition evidence",
        ] {
            assert!(rendered.contains(semantic), "{language} missing {semantic}");
        }
    }
}
