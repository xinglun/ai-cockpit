use cockpit_protocol::{
    AgentAdapterCompatibility, AgentInterfaceAvailability, AgentInterfaceManifest, AgentInterfaces,
    AgentRootBinding,
};
use std::{fs, path::Path};

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary repository");
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(directory.path())
        .status()
        .expect("git init");
    directory
}

fn manifest(repository_id: &str) -> AgentInterfaceManifest {
    AgentInterfaceManifest {
        schema_version: 1,
        protocol_version: 1,
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
    }
}

fn write_manifest(root: &Path, repository_id: &str) {
    let ai = root.join(".ai");
    fs::create_dir_all(&ai).expect(".ai");
    fs::write(
        ai.join("cockpit.toml"),
        format!("protocol_version = 1\nrepository_id = \"{repository_id}\"\n"),
    )
    .expect("config");
    let bytes = serde_json::to_vec_pretty(&manifest(repository_id)).expect("manifest JSON");
    fs::write(ai.join("agent-interface.json"), bytes).expect("manifest");
}

#[test]
fn manifest_parent_resolves_repository_context() {
    let repository = repository();
    write_manifest(
        repository.path(),
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    let context = cockpit_agent::load_agent_context(repository.path()).expect("context");
    assert_eq!(
        context.root,
        repository.path().canonicalize().expect("root")
    );
    assert_eq!(
        context.manifest_path,
        context.root.join(".ai/agent-interface.json")
    );
    assert_eq!(context.manifest.repository_id, context.repository_id);
}

#[test]
fn missing_manifest_is_unattached() {
    let repository = repository();
    assert!(cockpit_agent::load_agent_context(repository.path()).is_err());
}

#[test]
fn manifest_repository_mismatch_fails_closed() {
    let repository = repository();
    write_manifest(
        repository.path(),
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    fs::write(
        repository.path().join(".ai/cockpit.toml"),
        "protocol_version = 1\nrepository_id = \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\n",
    )
    .expect("different attached identity");
    assert!(cockpit_agent::load_agent_context(repository.path()).is_err());
}

#[cfg(unix)]
#[test]
fn manifest_symlink_is_rejected() {
    use std::os::unix::fs::symlink;

    let repository = repository();
    let ai = repository.path().join(".ai");
    fs::create_dir_all(&ai).expect(".ai");
    let repository_id = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    fs::write(
        ai.join("cockpit.toml"),
        format!("protocol_version = 1\nrepository_id = \"{repository_id}\"\n"),
    )
    .expect("config");
    let target = repository.path().join("manifest-target.json");
    fs::write(
        &target,
        serde_json::to_vec(&manifest(repository_id)).expect("manifest JSON"),
    )
    .expect("target");
    symlink(&target, ai.join("agent-interface.json")).expect("manifest symlink");
    assert!(cockpit_agent::load_agent_context(repository.path()).is_err());
}

#[test]
fn sha256_file_is_bounded_and_streaming() {
    let repository = repository();
    let target = repository.path().join("adapter.txt");
    fs::write(&target, b"adapter bytes").expect("target");
    let digest = cockpit_agent::sha256_file(&target).expect("digest");
    assert_eq!(
        digest,
        cockpit_core::Digest::sha256_bytes(b"adapter bytes").to_string()
    );

    let oversized = repository.path().join("oversized");
    let file = fs::File::create(&oversized).expect("oversized target");
    file.set_len(1024 * 1024 + 1).expect("sparse target");
    assert!(cockpit_agent::sha256_file(&oversized).is_err());
}
