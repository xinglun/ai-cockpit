use cockpit_core::Digest;
use cockpit_protocol::{
    CAPABILITY_SHOW_LANGUAGE_VALUES, WORK_ITEM_OUTCOME_CANONICAL_WORK_ITEM_ID,
    WORK_ITEM_OUTCOME_CLI_DELIVERY, WORK_ITEM_OUTCOME_CLI_JSON, WORK_ITEM_OUTCOME_CLI_LANGUAGE,
    WORK_ITEM_OUTCOME_CLI_VIEW, WORK_ITEM_OUTCOME_CLI_WORK_ITEM_ID,
    WORK_ITEM_OUTCOME_DEFAULT_DELIVERY, WORK_ITEM_OUTCOME_DEFAULT_VIEW,
    WORK_ITEM_OUTCOME_LANGUAGE_VALUES, WORK_ITEM_OUTCOME_MCP_DELIVERY,
    WORK_ITEM_OUTCOME_MCP_DELIVERY_PROGRESS, WORK_ITEM_OUTCOME_MCP_LANGUAGE,
    WORK_ITEM_OUTCOME_MCP_VIEW, WORK_ITEM_OUTCOME_MCP_WORK_ITEM_ID, WORK_ITEM_OUTCOME_VIEW_VALUES,
    WorkItemActionExplanation, WorkItemActionIssue, WorkItemActionIssueKind,
    WorkItemAdmissionState, WorkItemStatusSnapshot, normalize_work_item_outcome_language,
    render_interface_description_markdown, work_item_coordination_action_specs,
    work_item_coordination_action_values, work_item_coordination_parameter_specs,
    work_item_outcome_interface_description, work_item_outcome_interface_specs,
    work_item_outcome_mcp_request_parameter_specs, work_item_outcome_parameter_spec_by_canonical,
};
use serde_json::json;
use std::collections::BTreeSet;

fn surface<'a>(
    description: &'a cockpit_protocol::InterfaceDescription,
    name: &str,
) -> &'a cockpit_protocol::InterfaceSurface {
    description
        .surfaces
        .iter()
        .find(|surface| surface.name == name)
        .unwrap_or_else(|| panic!("missing {name} interface surface"))
}

fn parameter<'a>(
    surface: &'a cockpit_protocol::InterfaceSurface,
    name: &str,
) -> &'a cockpit_protocol::InterfaceParameter {
    surface
        .parameters
        .iter()
        .find(|parameter| parameter.name == name)
        .unwrap_or_else(|| panic!("missing {name} parameter"))
}

#[test]
fn cli_description_is_extracted_from_the_shared_outcome_query_parser() {
    let command = cockpit_protocol::work_item_outcome_query_command();
    let description = work_item_outcome_interface_description();
    let cli = surface(&description, "cli");

    for parameter in &cli.parameters {
        let argument = command
            .get_arguments()
            .find(|argument| argument.get_id().as_str() == parameter.name)
            .unwrap_or_else(|| panic!("missing parser argument {}", parameter.name));
        assert_eq!(
            argument.is_required_set(),
            parameter.required,
            "requiredness for {}",
            parameter.name
        );
        assert_eq!(
            argument.get_help().map(ToString::to_string),
            Some(parameter.description.clone()),
            "help for {}",
            parameter.name
        );
        if parameter.wire_type == "boolean" {
            assert!(
                parameter.enum_values.is_empty(),
                "boolean {} must not be represented as an enum",
                parameter.name
            );
        } else {
            assert_eq!(
                argument
                    .get_possible_values()
                    .into_iter()
                    .map(|value| value.get_name().to_owned())
                    .collect::<Vec<_>>(),
                parameter.enum_values,
                "enum values for {}",
                parameter.name
            );
        }
        assert_eq!(
            argument
                .get_default_values()
                .iter()
                .map(|value| value.to_string_lossy().into_owned())
                .next(),
            parameter.default,
            "default for {}",
            parameter.name
        );
    }
}

#[test]
fn public_outcome_value_lists_are_generated_from_the_parser_enum_variants() {
    let view_values = [
        cockpit_protocol::WorkItemOutcomeView::Summary,
        cockpit_protocol::WorkItemOutcomeView::Full,
    ]
    .into_iter()
    .map(|value| value.as_str().to_owned())
    .collect::<Vec<_>>();
    assert_eq!(
        view_values,
        WORK_ITEM_OUTCOME_VIEW_VALUES
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>()
    );

    let language_values = [
        cockpit_protocol::WorkItemOutcomeLanguage::En,
        cockpit_protocol::WorkItemOutcomeLanguage::Zh,
        cockpit_protocol::WorkItemOutcomeLanguage::ZhCn,
        cockpit_protocol::WorkItemOutcomeLanguage::Ja,
    ]
    .into_iter()
    .map(|value| value.as_str().to_owned())
    .collect::<Vec<_>>();
    assert_eq!(
        language_values,
        WORK_ITEM_OUTCOME_LANGUAGE_VALUES
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>()
    );
}

#[test]
fn outcome_description_has_stable_shared_facts() {
    assert_eq!(
        CAPABILITY_SHOW_LANGUAGE_VALUES,
        WORK_ITEM_OUTCOME_LANGUAGE_VALUES
    );
    let description = work_item_outcome_interface_description();
    assert_eq!(description.name, "work-item-outcome");
    assert_eq!(description.schema_version, 1);
    assert_eq!(description.runtime_version, env!("CARGO_PKG_VERSION"));

    let cli_specs = work_item_outcome_interface_specs("cli").expect("CLI specs");
    assert_eq!(cli_specs[0].name, WORK_ITEM_OUTCOME_CLI_WORK_ITEM_ID);
    assert_eq!(cli_specs[1].name, WORK_ITEM_OUTCOME_CLI_DELIVERY);
    assert_eq!(cli_specs[2].name, WORK_ITEM_OUTCOME_CLI_JSON);
    assert_eq!(cli_specs[3].name, WORK_ITEM_OUTCOME_CLI_VIEW);
    assert_eq!(cli_specs[4].name, WORK_ITEM_OUTCOME_CLI_LANGUAGE);
    let mcp_specs = work_item_outcome_interface_specs("mcp").expect("MCP specs");
    assert_eq!(mcp_specs[0].name, WORK_ITEM_OUTCOME_MCP_WORK_ITEM_ID);
    assert_eq!(mcp_specs[1].name, WORK_ITEM_OUTCOME_MCP_LANGUAGE);
    assert_eq!(mcp_specs[2].name, WORK_ITEM_OUTCOME_MCP_VIEW);
    assert_eq!(mcp_specs[3].name, WORK_ITEM_OUTCOME_MCP_DELIVERY);
    assert_eq!(mcp_specs[4].name, WORK_ITEM_OUTCOME_MCP_DELIVERY_PROGRESS);

    let cli = surface(&description, "cli");
    let view = parameter(cli, "view");
    assert_eq!(view.wire_type, "enum");
    assert!(!view.required);
    assert_eq!(
        view.default.as_deref(),
        Some(WORK_ITEM_OUTCOME_DEFAULT_VIEW)
    );
    assert_eq!(
        view.enum_values
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        WORK_ITEM_OUTCOME_VIEW_VALUES
    );

    let delivery = parameter(cli, "delivery");
    assert_eq!(delivery.wire_type, "boolean");
    assert_eq!(
        delivery.default.as_deref(),
        Some(if WORK_ITEM_OUTCOME_DEFAULT_DELIVERY {
            "true"
        } else {
            "false"
        })
    );

    let mcp = surface(&description, "mcp");
    let identity = parameter(mcp, "workItemId");
    assert!(identity.required);
    assert_eq!(identity.aliases, ["id"]);
    assert_eq!(
        parameter(mcp, "language")
            .enum_values
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        WORK_ITEM_OUTCOME_LANGUAGE_VALUES
    );
    assert_eq!(normalize_work_item_outcome_language("zh-CN"), "zh");
    assert_eq!(normalize_work_item_outcome_language("ja-JP"), "ja");
    assert_eq!(normalize_work_item_outcome_language("en-US"), "en");
}

#[test]
fn description_is_materialized_from_protocol_owned_parameter_tables() {
    let description = work_item_outcome_interface_description();
    for surface_name in ["cli", "mcp"] {
        let specs = work_item_outcome_interface_specs(surface_name).expect("known surface");
        let projected = surface(&description, surface_name);
        assert_eq!(
            projected.parameters.len(),
            specs.len(),
            "surface={surface_name}"
        );
        for (parameter, spec) in projected.parameters.iter().zip(specs) {
            assert_eq!(parameter.name, spec.name, "surface={surface_name}");
            assert_eq!(
                parameter.wire_type, spec.wire_type,
                "surface={surface_name}"
            );
            assert_eq!(parameter.required, spec.required, "surface={surface_name}");
            assert_eq!(
                parameter.default.as_deref(),
                spec.default.as_deref(),
                "surface={surface_name}"
            );
            assert_eq!(
                parameter
                    .enum_values
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                spec.enum_values,
                "surface={surface_name}"
            );
            assert_eq!(
                parameter
                    .aliases
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                spec.aliases,
                "surface={surface_name}"
            );
            assert_eq!(
                parameter.description, spec.description,
                "surface={surface_name}"
            );
        }
    }
}

#[test]
fn mcp_request_schema_and_discovery_share_the_same_parameter_projection() {
    let description = work_item_outcome_interface_description();
    let mcp = surface(&description, "mcp");
    let request_parameters = work_item_outcome_mcp_request_parameter_specs();

    assert_eq!(
        request_parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "workItemId",
            "language",
            "view",
            "delivery",
            "deliveryProgress"
        ]
    );
    assert_eq!(
        request_parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>(),
        mcp.parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>()
    );
    let progress = request_parameters
        .iter()
        .find(|parameter| parameter.name == "deliveryProgress")
        .expect("MCP delivery progress parameter");
    assert_eq!(progress.wire_type, "object");
    assert!(!progress.required);
    assert!(progress.default.is_none());
    assert_eq!(
        progress.description,
        "Identity-bound accepted-segment progress for an interrupted delivery."
    );
}

#[test]
fn transport_specs_reuse_the_shared_query_parser_facts() {
    let command = cockpit_protocol::work_item_outcome_query_command();
    for surface_name in ["cli", "mcp"] {
        let specs = work_item_outcome_interface_specs(surface_name).expect("known surface");
        for spec in specs {
            if spec.canonical_name == "deliveryProgress" {
                continue;
            }
            let cli_spec =
                work_item_outcome_parameter_spec_by_canonical("cli", &spec.canonical_name)
                    .expect("shared CLI parser binding");
            let argument = command
                .get_arguments()
                .find(|argument| argument.get_id().as_str() == cli_spec.name)
                .unwrap_or_else(|| panic!("missing parser argument {}", cli_spec.name));
            assert_eq!(
                spec.default.as_deref(),
                argument
                    .get_default_values()
                    .first()
                    .map(|value| value.to_string_lossy())
                    .as_deref(),
                "{surface_name}"
            );
            assert_eq!(
                spec.description,
                argument
                    .get_help()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                "{surface_name}"
            );
        }
    }
}

#[test]
fn adapters_resolve_transport_bindings_from_canonical_parameter_facts() {
    for (surface_name, expected_identity, expected_aliases) in [
        ("cli", "id", Vec::<&str>::new()),
        ("mcp", "workItemId", vec!["id"]),
    ] {
        let spec = work_item_outcome_parameter_spec_by_canonical(
            surface_name,
            WORK_ITEM_OUTCOME_CANONICAL_WORK_ITEM_ID,
        )
        .expect("canonical Work Item identity binding");
        assert_eq!(spec.name, expected_identity, "surface={surface_name}");
        assert_eq!(spec.aliases, expected_aliases, "surface={surface_name}");
    }
}

#[test]
fn cli_and_mcp_share_common_outcome_parameter_facts() {
    let description = work_item_outcome_interface_description();
    let cli = surface(&description, "cli");
    let mcp = surface(&description, "mcp");
    for name in ["delivery", "view", "language"] {
        let cli_parameter = parameter(cli, name);
        let mcp_parameter = parameter(mcp, name);
        assert_eq!(cli_parameter.wire_type, mcp_parameter.wire_type, "{name}");
        assert_eq!(cli_parameter.required, mcp_parameter.required, "{name}");
        assert_eq!(cli_parameter.default, mcp_parameter.default, "{name}");
        assert_eq!(
            cli_parameter.enum_values, mcp_parameter.enum_values,
            "{name}"
        );
        assert_eq!(cli_parameter.aliases, mcp_parameter.aliases, "{name}");
        assert_eq!(
            cli_parameter.description, mcp_parameter.description,
            "{name}"
        );
    }
}

#[test]
fn shared_view_definition_rejects_unknown_values() {
    for value in WORK_ITEM_OUTCOME_VIEW_VALUES {
        assert!(cockpit_protocol::work_item_outcome_view_is_valid(value));
    }
    assert!(!cockpit_protocol::work_item_outcome_view_is_valid(
        "compact"
    ));
}

#[test]
fn outcome_description_and_markdown_are_deterministic() {
    let first = work_item_outcome_interface_description();
    let second = work_item_outcome_interface_description();
    assert_eq!(
        serde_json::to_vec(&first).expect("serialize first"),
        serde_json::to_vec(&second).expect("serialize second")
    );

    for language in ["en", "zh-CN", "ja"] {
        let first_markdown = render_interface_description_markdown(&first, language);
        let second_markdown = render_interface_description_markdown(&second, language);
        assert_eq!(first_markdown, second_markdown, "language {language}");
        assert!(first_markdown.contains("summary"));
        assert!(first_markdown.contains("full"));
        assert!(first_markdown.contains("delivery"));
    }
}

#[test]
fn coordination_interface_specs_bind_one_identity_contract_per_action_variant() {
    let parameters = work_item_coordination_parameter_specs();
    let names = parameters
        .iter()
        .map(|parameter| parameter.name)
        .collect::<BTreeSet<_>>();
    assert_eq!(names.len(), parameters.len(), "MCP properties are unique");
    for (mcp_name, cli_name) in [
        ("providerWorkItemId", "id"),
        ("providerGeneration", "generation"),
        ("outcomeId", "outcome-id"),
        ("eventId", "event-id"),
        ("consumerWorkItemId", "consumer-work-item-id"),
        ("consumerGeneration", "consumer-generation"),
    ] {
        assert_eq!(
            parameters
                .iter()
                .find(|parameter| parameter.name == mcp_name)
                .expect("coordination parameter")
                .cli_name,
            Some(cli_name),
            "CLI/MCP binding for {mcp_name}"
        );
    }
    assert_eq!(
        work_item_coordination_action_values(),
        [
            "inspect",
            "register",
            "report-impact",
            "publish-outcome",
            "request-pause",
            "acknowledge",
            "resume",
            "recover"
        ]
    );

    let actions = work_item_coordination_action_specs();
    let publish = actions
        .iter()
        .filter(|action| action.action == "publish-outcome")
        .collect::<Vec<_>>();
    assert_eq!(
        publish.len(),
        2,
        "one canonical and one legacy publish form"
    );
    assert!(publish.iter().any(|action| {
        !action.legacy_alias
            && action.required_parameters
                == ["providerWorkItemId", "providerGeneration", "outcomeId"]
            && action.allowed_parameters
                == ["providerWorkItemId", "providerGeneration", "outcomeId"]
    }));
    assert!(publish.iter().any(|action| {
        action.legacy_alias
            && action.required_parameters == ["workItemId", "generation", "outcomeId"]
    }));

    let resume = actions
        .iter()
        .find(|action| action.action == "resume")
        .expect("resume action");
    assert_eq!(resume.required_parameters, ["workItemId", "generation"]);
    let recover = actions
        .iter()
        .find(|action| action.action == "recover")
        .expect("recover action");
    assert_eq!(
        recover.required_parameters,
        ["eventId", "consumerWorkItemId", "consumerGeneration"]
    );
}

#[test]
fn action_explanation_is_additive_and_legacy_status_remains_readable() {
    let explanation = WorkItemActionExplanation {
        guide_id: "ordinary-work-item".into(),
        recommended_action: Some("run_preflight".into()),
        recommendation_reason:
            "the current Contract is active and the next admitted action is preflight".into(),
        admission_state: WorkItemAdmissionState::Allowed,
        issues: vec![WorkItemActionIssue {
            kind: WorkItemActionIssueKind::Missing,
            code: "verification_evidence_missing".into(),
            message: "verification evidence is not present yet".into(),
        }],
        human_decision_required: false,
        missing_inputs: vec!["verification evidence".into()],
        admission_digest: Digest::sha256_bytes(b"action-admission"),
    };
    let serialized = serde_json::to_value(&explanation).expect("serialize explanation");
    assert_eq!(serialized["guideId"], "ordinary-work-item");
    assert_eq!(serialized["recommendedAction"], "run_preflight");
    assert_eq!(serialized["admissionState"], "allowed");
    assert_eq!(serialized["issues"][0]["kind"], "missing");

    let legacy = json!({
        "schemaVersion": 1,
        "repositoryId": "sha256:repository",
        "workItemId": "WI-LEGACY",
        "baseCommit": "base",
        "lifecyclePhase": "implementation_active",
        "governanceState": "yellow",
        "activityHealth": "active",
        "blocking": false,
        "humanDecisionRequired": false,
        "progressFacts": {},
        "blockers": [],
        "missingEvidence": [],
        "dependencies": [],
        "humanDecisions": [],
        "risks": [],
        "verification": "not_ready",
        "completionDomains": {},
        "governancePermissions": ["read_status"],
        "sourceDigests": {},
        "unknowns": [],
        "diagnostics": [],
        "snapshotDigest": "sha256:snapshot",
        "evidenceFreshness": {
            "state": "missing",
            "reason": "verification evidence is missing"
        },
        "safeActions": ["run_preflight"],
        "statusDigest": "sha256:status",
        "historical": false
    });
    let snapshot: WorkItemStatusSnapshot =
        serde_json::from_value(legacy).expect("legacy status remains readable");
    assert!(snapshot.action_explanation.is_none());
}

fn generated_region(document: &str) -> &str {
    let begin = "<!-- AI_COCKPIT_INTERFACE_FACTS:BEGIN work-item-outcome -->";
    let end = "<!-- AI_COCKPIT_INTERFACE_FACTS:END work-item-outcome -->";
    let start = document.find(begin).expect("generated region begin marker");
    let finish = document[start..]
        .find(end)
        .map(|offset| start + offset + end.len())
        .expect("generated region end marker");
    &document[start..finish]
}

#[test]
fn checked_in_reference_regions_match_the_protocol_projection() {
    let description = work_item_outcome_interface_description();
    let references = [
        (include_str!("../../../docs/reference/commands.md"), "en"),
        (
            include_str!("../../../docs/reference/commands.zh-CN.md"),
            "zh-CN",
        ),
        (include_str!("../../../docs/reference/commands.ja.md"), "ja"),
    ];

    for (document, language) in references {
        let expected = render_interface_description_markdown(&description, language);
        assert_eq!(
            generated_region(document),
            expected.trim_end(),
            "language={language}"
        );
    }
}
