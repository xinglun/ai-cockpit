use cockpit_protocol::{
    AgentAdapterCompatibility, AgentInterfaceAvailability, AgentInterfaceManifest, AgentInterfaces,
    AgentProvider, AgentRootBinding,
};
use std::{fs, thread};

fn repository(id: char) -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("repository");
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(directory.path())
        .status()
        .expect("git init");
    let repository_id = format!("sha256:{}", id.to_string().repeat(64));
    let ai = directory.path().join(".ai");
    fs::create_dir_all(&ai).expect(".ai");
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
        repository_id,
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
    .expect("manifest");
    fs::write(directory.path().join("AGENTS.md"), format!("rules {id}\n")).expect("target");
    directory
}

#[test]
fn parallel_agent_operations_keep_repository_ids_and_records_separate() {
    let left = repository('b');
    let right = repository('c');
    thread::scope(|scope| {
        let left_path = left.path().to_path_buf();
        let right_path = right.path().to_path_buf();
        let left_task = scope.spawn(move || {
            cockpit_agent::install_adapter(&left_path, AgentProvider::Codex).expect("left install")
        });
        let right_task = scope.spawn(move || {
            cockpit_agent::install_adapter(&right_path, AgentProvider::Codex)
                .expect("right install")
        });
        let left_receipt = left_task.join().expect("left join");
        let right_receipt = right_task.join().expect("right join");
        assert_ne!(left_receipt.repository_id, right_receipt.repository_id);
    });
    let left_record =
        fs::read_to_string(left.path().join(".ai/adapters/codex.json")).expect("left record");
    let right_record =
        fs::read_to_string(right.path().join(".ai/adapters/codex.json")).expect("right record");
    assert_ne!(left_record, right_record);
    assert!(left_record.contains("bbbb"));
    assert!(right_record.contains("cccc"));
}
