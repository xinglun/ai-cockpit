use cockpit_protocol::{
    AgentAdapterCompatibility, AgentInterfaceAvailability, AgentInterfaceManifest, AgentInterfaces,
    AgentProvider, AgentRootBinding,
};
use std::{fs, path::Path};

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary repository");
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(directory.path())
        .status()
        .expect("git init");
    let ai = directory.path().join(".ai");
    fs::create_dir_all(&ai).expect(".ai");
    let repository_id = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    fs::write(
        ai.join("cockpit.toml"),
        format!("protocol_version = 1\nrepository_id = \"{repository_id}\"\n"),
    )
    .expect("config");
    let manifest = AgentInterfaceManifest {
        schema_version: 1,
        protocol_version: 1,
        repository_schema_version: 1,
        interface_version: 1,
        repository_id: repository_id.into(),
        root_binding: AgentRootBinding {
            binding_type: "manifest-parent".into(),
        },
        capabilities: vec!["status".into()],
        interfaces: AgentInterfaces {
            cli: AgentInterfaceAvailability {
                available: true,
                transport: None,
            },
            mcp: AgentInterfaceAvailability {
                available: false,
                transport: None,
            },
        },
        adapter: AgentAdapterCompatibility { required: false },
        adapter_state: "unconfigured".into(),
    };
    fs::write(
        ai.join("agent-interface.json"),
        serde_json::to_vec(&manifest).expect("manifest"),
    )
    .expect("manifest file");
    directory
}

#[test]
fn detection_is_read_only() {
    let repository = repository();
    fs::write(repository.path().join("AGENTS.md"), "user instructions\n").expect("AGENTS");
    let before = fs::read(repository.path().join("AGENTS.md")).expect("before");
    let detected = cockpit_agent::detect_providers(repository.path()).expect("detect");
    assert!(!detected.is_empty());
    assert_eq!(
        fs::read(repository.path().join("AGENTS.md")).expect("after"),
        before
    );
}

#[test]
fn auto_lists_only_safe_surfaces() {
    let repository = repository();
    fs::write(repository.path().join("AGENTS.md"), "user instructions\n").expect("AGENTS");
    fs::write(repository.path().join("CLAUDE.md"), "claude instructions\n").expect("CLAUDE");
    fs::create_dir_all(repository.path().join(".cursor/rules")).expect("cursor rules");
    let detected = cockpit_agent::detect_providers(repository.path()).expect("detect");
    let canonical_root = repository.path().canonicalize().expect("canonical root");
    let providers = detected
        .iter()
        .map(|item| item.provider.clone())
        .collect::<Vec<_>>();
    assert!(providers.contains(&AgentProvider::GenericAgentsMd));
    assert!(providers.contains(&AgentProvider::Codex));
    assert!(providers.contains(&AgentProvider::Claude));
    assert!(providers.contains(&AgentProvider::Cursor));
    assert!(!providers.contains(&AgentProvider::Gemini));
    for item in detected {
        assert!(item.target.starts_with(&canonical_root));
        assert!(item.conflict.is_none());
    }
}

#[test]
fn provider_target_is_repository_relative() {
    let repository = repository();
    fs::write(repository.path().join("AGENTS.md"), "user instructions\n").expect("AGENTS");
    let plan = cockpit_agent::plan_install(repository.path(), AgentProvider::Codex).expect("plan");
    let canonical_root = repository.path().canonicalize().expect("canonical root");
    assert_eq!(plan.provider, AgentProvider::Codex);
    assert_eq!(plan.target, canonical_root.join("AGENTS.md"));
    assert!(plan.target.starts_with(&canonical_root));
}

#[test]
fn cursor_uses_provider_native_mdc_target() {
    let repository = repository();
    fs::create_dir_all(repository.path().join(".cursor/rules")).expect("cursor rules");
    let plan = cockpit_agent::plan_install(repository.path(), AgentProvider::Cursor).expect("plan");
    let canonical_root = repository.path().canonicalize().expect("canonical root");
    assert_eq!(
        plan.target,
        canonical_root.join(".cursor/rules/ai-cockpit.mdc")
    );
}

#[test]
fn cursor_legacy_managed_md_remains_discoverable() {
    let repository = repository();
    let target = repository.path().join(".cursor/rules/ai-cockpit.md");
    fs::create_dir_all(target.parent().expect("rules")).expect("cursor rules");
    fs::write(
        &target,
        "<!-- AI_COCKPIT_ADAPTER_BEGIN provider=cursor -->\nlegacy\n<!-- AI_COCKPIT_ADAPTER_END -->\n",
    )
    .expect("legacy adapter");
    let plan = cockpit_agent::plan_install(repository.path(), AgentProvider::Cursor).expect("plan");
    assert_eq!(
        plan.target,
        target.canonicalize().expect("canonical target")
    );
}

#[test]
fn duplicate_marker_is_a_conflict() {
    let repository = repository();
    let target = repository.path().join("AGENTS.md");
    fs::write(
        &target,
        "<!-- AI_COCKPIT_ADAPTER_BEGIN -->\nold\n<!-- AI_COCKPIT_ADAPTER_END -->\n<!-- AI_COCKPIT_ADAPTER_BEGIN -->\nother\n<!-- AI_COCKPIT_ADAPTER_END -->\n",
    )
    .expect("markers");
    let plan = cockpit_agent::plan_install(repository.path(), AgentProvider::Codex).expect("plan");
    assert!(plan.conflict.is_some());
    assert!(!plan.executable);
}

#[test]
fn unsupported_host_surface_is_not_selected() {
    let repository = repository();
    fs::write(repository.path().join(".random-agent-rules"), "rules\n").expect("unknown");
    let detected = cockpit_agent::detect_providers(repository.path()).expect("detect");
    assert!(detected.is_empty());
    assert!(Path::new(".random-agent-rules").is_relative());
}
