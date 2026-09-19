use cockpit_protocol::{
    CAPABILITY_SHOW_LANGUAGE_VALUES, WORK_ITEM_OUTCOME_CANONICAL_WORK_ITEM_ID,
    WORK_ITEM_OUTCOME_CLI_DELIVERY, WORK_ITEM_OUTCOME_CLI_JSON, WORK_ITEM_OUTCOME_CLI_LANGUAGE,
    WORK_ITEM_OUTCOME_CLI_VIEW, WORK_ITEM_OUTCOME_CLI_WORK_ITEM_ID,
    WORK_ITEM_OUTCOME_DEFAULT_DELIVERY, WORK_ITEM_OUTCOME_DEFAULT_VIEW,
    WORK_ITEM_OUTCOME_LANGUAGE_VALUES, WORK_ITEM_OUTCOME_MCP_DELIVERY,
    WORK_ITEM_OUTCOME_MCP_DELIVERY_PROGRESS, WORK_ITEM_OUTCOME_MCP_LANGUAGE,
    WORK_ITEM_OUTCOME_MCP_VIEW, WORK_ITEM_OUTCOME_MCP_WORK_ITEM_ID, WORK_ITEM_OUTCOME_VIEW_VALUES,
    normalize_work_item_outcome_language, render_interface_description_markdown,
    work_item_outcome_interface_description, work_item_outcome_interface_specs,
    work_item_outcome_parameter_definition, work_item_outcome_parameter_spec_by_canonical,
};

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
                spec.default,
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
fn surface_specs_reference_one_canonical_parameter_definition() {
    for surface_name in ["cli", "mcp"] {
        let specs = work_item_outcome_interface_specs(surface_name).expect("known surface");
        for spec in specs {
            let definition = work_item_outcome_parameter_definition(spec.canonical_name)
                .unwrap_or_else(|| panic!("missing canonical definition {}", spec.canonical_name));
            assert_eq!(spec.wire_type, definition.wire_type, "{surface_name}");
            assert_eq!(spec.default, definition.default, "{surface_name}");
            assert_eq!(spec.enum_values, definition.enum_values, "{surface_name}");
            assert_eq!(spec.description, definition.description, "{surface_name}");
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
