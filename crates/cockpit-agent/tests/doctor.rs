use cockpit_protocol::{
    AgentAdapterCompatibility, AgentInterfaceAvailability, AgentInterfaceManifest, AgentInterfaces,
    AgentProvider, AgentRootBinding,
};
use std::{fs, path::Path};

fn repository(mcp: bool) -> tempfile::TempDir {
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
                available: mcp,
                transport: mcp.then(|| "stdio".into()),
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
fn doctor_reports_unattached_and_attached() {
    let unattached = tempfile::tempdir().expect("unattached");
    let report = cockpit_agent::doctor(unattached.path()).expect("doctor unattached");
    assert_eq!(report.state, "UNATTACHED");
    assert_eq!(report.repository_id, None);

    let attached = repository(false);
    let report = cockpit_agent::doctor(attached.path()).expect("doctor attached");
    assert_eq!(report.state, "ATTACHED");
    assert!(report.repository_id.is_some());
}

#[test]
fn doctor_requires_matching_repository_probe_for_verified() {
    let repository = repository(true);
    fs::write(repository.path().join("AGENTS.md"), "rules\n").expect("target");
    cockpit_agent::install_adapter(repository.path(), AgentProvider::Codex).expect("install");
    let manifest = repository.path().join(".ai/agent-interface.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest).expect("manifest")).expect("json");
    value["repositoryId"] = serde_json::Value::String(
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
    );
    fs::write(&manifest, serde_json::to_vec(&value).expect("json")).expect("mutate manifest");
    let report = cockpit_agent::doctor(repository.path()).expect("doctor");
    assert_ne!(report.state, "VERIFIED");
    assert!(!report.problems.is_empty());
}

#[test]
fn doctor_reports_degraded_without_mcp() {
    let repository = repository(false);
    fs::write(repository.path().join("AGENTS.md"), "rules\n").expect("target");
    cockpit_agent::install_adapter(repository.path(), AgentProvider::Codex).expect("install");
    let report = cockpit_agent::doctor(repository.path()).expect("doctor");
    assert_eq!(report.state, "DEGRADED");
    assert_eq!(report.interfaces.mcp, "unavailable");
}

#[test]
fn modified_section_blocks_detach() {
    let repository = repository(false);
    let target = repository.path().join("AGENTS.md");
    fs::write(&target, "rules\n").expect("target");
    cockpit_agent::install_adapter(repository.path(), AgentProvider::Codex).expect("install");
    let mut content = fs::read_to_string(&target).expect("content");
    content = content.replace("repository-governance interface", "changed interface");
    fs::write(&target, content).expect("modify");
    assert!(cockpit_agent::detach_adapter(repository.path(), AgentProvider::Codex).is_err());
    assert!(repository.path().join(".ai/adapters/codex.json").is_file());
}

#[test]
fn unchanged_section_detaches_only_owned_bytes() {
    let repository = repository(false);
    let target = repository.path().join("AGENTS.md");
    fs::write(&target, "rules\n\nkeep this\n").expect("target");
    cockpit_agent::install_adapter(repository.path(), AgentProvider::Codex).expect("install");
    let before = fs::read_to_string(&target).expect("content");
    cockpit_agent::detach_adapter(repository.path(), AgentProvider::Codex).expect("detach");
    assert_eq!(
        fs::read_to_string(&target).expect("after"),
        "rules\n\nkeep this\n"
    );
    assert!(!repository.path().join(".ai/adapters/codex.json").exists());
    assert!(before.contains("rules\n\nkeep this\n"));
}

#[test]
fn repair_refuses_conflict_without_force() {
    let repository = repository(false);
    let target = repository.path().join("AGENTS.md");
    fs::write(&target, "rules\n").expect("target");
    cockpit_agent::install_adapter(repository.path(), AgentProvider::Codex).expect("install");
    let mut content = fs::read_to_string(&target).expect("content");
    content = content.replace(
        "This repository is attached",
        "This repository was manually edited",
    );
    fs::write(&target, content).expect("modify");
    assert!(cockpit_agent::repair_adapter(repository.path(), AgentProvider::Codex).is_err());
}

#[test]
fn exit_codes_match_state() {
    assert_eq!(cockpit_agent::AgentState::Verified.exit_code().code(), 0);
    assert_eq!(cockpit_agent::AgentState::Degraded.exit_code().code(), 1);
    assert_eq!(cockpit_agent::AgentState::Unattached.exit_code().code(), 2);
    assert_eq!(cockpit_agent::AgentState::Conflict.exit_code().code(), 3);
}

#[test]
fn doctor_does_not_follow_unknown_surface_symlink() {
    let _repository = repository(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let outside = _repository.path().join("outside.md");
        fs::write(&outside, "outside\n").expect("outside");
        symlink(&outside, _repository.path().join("AGENTS.md")).expect("link");
        let report = cockpit_agent::doctor(_repository.path()).expect("doctor");
        assert!(report.state == "CONFLICT" || report.state == "ATTACHED");
    }
}

#[allow(dead_code)]
fn _assert_relative(path: &Path) {
    assert!(path.is_relative());
}
