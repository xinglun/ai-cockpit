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
    for required in [
        "current Work Item",
        "another Work Item or Issue",
        "Outcome: 🟢",
        "status=completed",
        "successor",
        "scope",
        "authority",
        "base",
    ] {
        assert!(
            combined.contains(required),
            "missing Agent rule: {required}"
        );
    }
}
