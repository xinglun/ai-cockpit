use cockpit_repository::{CollaborationOutcomeProjection, render_collaboration_outcome};
use cockpit_verification::{CompositionCleanupDisposition, CompositionExecutionOutcome};

fn projection() -> CollaborationOutcomeProjection {
    CollaborationOutcomeProjection {
        schema_version: 2,
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
        composition_state: "unknown".into(),
        execution_outcome: CompositionExecutionOutcome::Passed,
        execution_evidence_complete: true,
        composition_applicability: "stale".into(),
        target_merge_state: "not_observed".into(),
        cleanup_state: "deferred".into(),
        cleanup_disposition: CompositionCleanupDisposition::Deferred,
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
        let applicability_label = match language {
            "en" => "Composition applicability",
            "zh" => "组合适用性",
            "ja" => "構成適用性",
            _ => unreachable!("supported language"),
        };
        let execution_outcome_label = match language {
            "en" => "Execution outcome",
            "zh" => "执行结果",
            "ja" => "実行結果",
            _ => unreachable!("supported language"),
        };
        let cleanup_disposition_label = match language {
            "en" => "Cleanup disposition",
            "zh" => "清理处置",
            "ja" => "クリーンアップ処置",
            _ => unreachable!("supported language"),
        };
        for semantic in [
            "WI-PARITY",
            "WI-PROVIDER",
            "WI-CONSUMER",
            "impact-1",
            "separate_lifecycle_outcome",
            "stale",
            "passed",
            "deferred",
            "true",
            "required",
            "dependency_impact:WI-PROVIDER",
            "refresh affected dependency and composition evidence",
        ] {
            assert!(rendered.contains(semantic), "{language} missing {semantic}");
        }
        assert!(
            rendered.contains(applicability_label),
            "{language} missing composition applicability label"
        );
        assert!(
            rendered.contains(execution_outcome_label),
            "{language} missing execution outcome label"
        );
        assert!(
            rendered.contains(cleanup_disposition_label),
            "{language} missing cleanup disposition label"
        );
    }
}
