use std::fs;

fn repository_file(name: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(name)).expect("repository governance file")
}

#[test]
fn agent_rules_keep_terminality_and_successor_boundaries_visible() {
    let agents = repository_file("AGENTS.md");
    let readme = repository_file(".ai/README.md");
    let workflow = repository_file("docs/reference/agent-workflow.md");
    let combined = format!("{agents}\n{readme}\n{workflow}");
    let normalized = combined.split_whitespace().collect::<Vec<_>>().join(" ");
    for required in [
        "current Work Item",
        "before opening another Work Item or Issue",
        "Outcome: 🟢",
        "status=completed",
        "humanStatusColor=green",
        "successor",
        "scope",
        "authority",
        "base",
    ] {
        assert!(
            normalized.contains(required),
            "missing Agent rule: {required}"
        );
    }
}

#[test]
fn source_terminality_rule_is_projected_without_template_copy() {
    let agents = repository_file("AGENTS.md");
    let readme = repository_file(".ai/README.md");
    let workflow = repository_file("docs/reference/agent-workflow.md");
    let combined = format!("{agents}\n{readme}\n{workflow}");
    let normalized = combined.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(normalized.contains("Repair an in-scope defect in the current Work Item"));
    assert!(normalized.contains("genuinely different scope, authority, or base"));
    assert!(normalized.contains("direct human-visible delivery"));
    assert!(normalized.contains("Missing, folded-only, stale"));
}
