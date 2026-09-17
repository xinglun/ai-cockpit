use cockpit_protocol::{
    render_interface_description_markdown, work_item_outcome_interface_description,
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
    let description = work_item_outcome_interface_description();
    assert_eq!(description.name, "work-item-outcome");
    assert_eq!(description.schema_version, 1);
    assert_eq!(description.runtime_version, env!("CARGO_PKG_VERSION"));

    let cli = surface(&description, "cli");
    let view = parameter(cli, "view");
    assert_eq!(view.wire_type, "enum");
    assert!(!view.required);
    assert_eq!(view.default.as_deref(), Some("summary"));
    assert_eq!(view.enum_values, ["summary", "full"]);

    let delivery = parameter(cli, "delivery");
    assert_eq!(delivery.wire_type, "boolean");
    assert_eq!(delivery.default.as_deref(), Some("false"));

    let mcp = surface(&description, "mcp");
    let identity = parameter(mcp, "workItemId");
    assert!(identity.required);
    assert_eq!(identity.aliases, ["id"]);
    assert_eq!(parameter(mcp, "language").enum_values, ["en", "zh", "ja"]);
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
