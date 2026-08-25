use cockpit_protocol::{
    AgentAdapterCompatibility, AgentInterfaceAvailability, AgentInterfaceManifest, AgentInterfaces,
    AgentProvider, AgentRootBinding, ManagedAdapterRecord,
};
use std::fs;

fn repository(repository_id: &str) -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary repository");
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(directory.path())
        .status()
        .expect("git init");
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

fn id(fill: char) -> String {
    format!("sha256:{}", fill.to_string().repeat(64))
}

#[test]
fn install_creates_only_owned_managed_section() {
    let repository = repository(&id('a'));
    let target = repository.path().join("AGENTS.md");
    fs::write(&target, "user instructions\n").expect("AGENTS");
    let before_config = fs::read(repository.path().join(".ai/cockpit.toml")).expect("config");
    let receipt =
        cockpit_agent::install_adapter(repository.path(), AgentProvider::Codex).expect("install");
    let content = fs::read_to_string(&target).expect("target");
    assert!(content.starts_with("user instructions\n"));
    assert!(content.contains("AI_COCKPIT_ADAPTER_BEGIN"));
    assert!(content.contains("Do not infer AI Cockpit state from this file."));
    assert!(content.contains("Read .ai/README.md before acting"));
    assert!(
        content.contains("Every repository-bound command must include an explicit --repo <path>.")
    );
    assert!(content.contains(
        "start or work-item new → preflight → checkpoint → verify → finish → archive → close."
    ));
    assert!(content.contains("latest remote default base → dedicated branch/worktree"));
    assert!(content.contains("Never merge a feature branch into local main before PR review"));
    assert!(content.contains("preserve the retry checkout and identity"));
    assert!(content.contains("status=completed plus humanStatusColor=green"));
    assert!(content.contains("current Work Item"));
    assert!(content.contains("before opening another Work Item or Issue"));
    assert!(content.contains("genuinely different scope, authority, or base"));
    assert!(content.contains("AI_COCKPIT_ADAPTER_END"));
    assert_eq!(
        fs::read(repository.path().join(".ai/cockpit.toml")).expect("config"),
        before_config
    );
    assert_eq!(receipt.provider, AgentProvider::Codex);
    assert!(repository.path().join(".ai/adapters/codex.json").is_file());
}

#[test]
fn repeated_install_is_byte_stable() {
    let repository = repository(&id('a'));
    let target = repository.path().join("AGENTS.md");
    fs::write(&target, "user instructions\n").expect("AGENTS");
    let first = cockpit_agent::install_adapter(repository.path(), AgentProvider::Codex)
        .expect("first install");
    let first_target = fs::read(&target).expect("target");
    let first_record = fs::read(repository.path().join(".ai/adapters/codex.json")).expect("record");
    let second = cockpit_agent::install_adapter(repository.path(), AgentProvider::Codex)
        .expect("second install");
    assert_eq!(first, second);
    assert_eq!(fs::read(&target).expect("target"), first_target);
    assert_eq!(
        fs::read(repository.path().join(".ai/adapters/codex.json")).expect("record"),
        first_record
    );
}

#[test]
fn install_preserves_unrelated_bytes() {
    let repository = repository(&id('a'));
    let target = repository.path().join("AGENTS.md");
    let user_bytes = b"header\r\nuser-owned content\x00\n";
    fs::write(&target, user_bytes).expect("AGENTS");
    cockpit_agent::install_adapter(repository.path(), AgentProvider::Codex).expect("install");
    let installed = fs::read(&target).expect("target");
    assert!(installed.starts_with(user_bytes));
}

#[test]
fn repository_a_and_b_have_independent_ownership_records() {
    let left = repository(&id('a'));
    let right = repository(&id('b'));
    fs::write(left.path().join("AGENTS.md"), "left\n").expect("left");
    fs::write(right.path().join("AGENTS.md"), "right\n").expect("right");
    let left_receipt =
        cockpit_agent::install_adapter(left.path(), AgentProvider::Codex).expect("left install");
    let right_receipt =
        cockpit_agent::install_adapter(right.path(), AgentProvider::Codex).expect("right install");
    assert_ne!(left_receipt.repository_id, right_receipt.repository_id);
    let left_record: ManagedAdapterRecord = serde_json::from_slice(
        &fs::read(left.path().join(".ai/adapters/codex.json")).expect("left record"),
    )
    .expect("left record JSON");
    let right_record: ManagedAdapterRecord = serde_json::from_slice(
        &fs::read(right.path().join(".ai/adapters/codex.json")).expect("right record"),
    )
    .expect("right record JSON");
    assert_eq!(left_record.repository_id, left_receipt.repository_id);
    assert_eq!(right_record.repository_id, right_receipt.repository_id);
    assert_ne!(left_record.installed_digest, right_record.installed_digest);
}

#[test]
fn install_rejects_duplicate_or_malformed_markers() {
    let repository = repository(&id('a'));
    let target = repository.path().join("AGENTS.md");
    fs::write(
        &target,
        "<!-- AI_COCKPIT_ADAPTER_BEGIN -->\nold\n<!-- AI_COCKPIT_ADAPTER_END -->\n<!-- AI_COCKPIT_ADAPTER_BEGIN -->\nother\n<!-- AI_COCKPIT_ADAPTER_END -->\n",
    )
    .expect("markers");
    assert!(cockpit_agent::install_adapter(repository.path(), AgentProvider::Codex).is_err());

    fs::write(&target, "<!-- AI_COCKPIT_ADAPTER_BEGIN -->\nmissing end\n").expect("malformed");
    assert!(cockpit_agent::install_adapter(repository.path(), AgentProvider::Codex).is_err());
}

#[test]
fn cursor_install_writes_mdc_and_preserves_user_owned_legacy_md() {
    let repository = repository(&id('a'));
    let legacy = repository.path().join(".cursor/rules/ai-cockpit.md");
    fs::create_dir_all(legacy.parent().expect("rules")).expect("cursor rules");
    fs::write(&legacy, "user Cursor rules\n").expect("legacy user rules");
    let receipt =
        cockpit_agent::install_adapter(repository.path(), AgentProvider::Cursor).expect("install");
    assert_eq!(
        receipt.target,
        repository
            .path()
            .canonicalize()
            .expect("canonical root")
            .join(".cursor/rules/ai-cockpit.mdc")
    );
    assert_eq!(
        fs::read_to_string(legacy).expect("legacy"),
        "user Cursor rules\n"
    );
    assert!(
        repository
            .path()
            .join(".cursor/rules/ai-cockpit.mdc")
            .is_file()
    );
}

#[test]
fn cursor_legacy_managed_adapter_is_repaired_in_place() {
    let repository = repository(&id('a'));
    let canonical_root = repository.path().canonicalize().expect("canonical root");
    let legacy = canonical_root.join(".cursor/rules/ai-cockpit.md");
    fs::create_dir_all(legacy.parent().expect("rules")).expect("cursor rules");
    let canonical = canonical_root.join(".cursor/rules/ai-cockpit.mdc");
    let first =
        cockpit_agent::install_adapter(repository.path(), AgentProvider::Cursor).expect("install");
    let mut record: ManagedAdapterRecord = serde_json::from_slice(
        &fs::read(repository.path().join(".ai/adapters/cursor.json")).expect("record"),
    )
    .expect("record JSON");
    fs::rename(&canonical, &legacy).expect("legacy rename");
    record.target = ".cursor/rules/ai-cockpit.md".into();
    fs::write(
        repository.path().join(".ai/adapters/cursor.json"),
        serde_json::to_vec_pretty(&record).expect("record bytes"),
    )
    .expect("legacy record");
    assert_eq!(first.target, canonical);
    let legacy_plan =
        cockpit_agent::plan_install(repository.path(), AgentProvider::Cursor).expect("legacy plan");
    assert_eq!(legacy_plan.target, legacy);
    let mut content = fs::read_to_string(&legacy).expect("content");
    content = content.replace("This repository is attached", "This repository was edited");
    fs::write(&legacy, content).expect("edit");
    assert!(cockpit_agent::repair_adapter(repository.path(), AgentProvider::Cursor).is_err());
    assert!(repository.path().join(".ai/adapters/cursor.json").is_file());
}

#[test]
fn cursor_legacy_managed_adapter_detaches_only_owned_bytes() {
    let repository = repository(&id('a'));
    let legacy = repository.path().join(".cursor/rules/ai-cockpit.md");
    fs::create_dir_all(legacy.parent().expect("rules")).expect("cursor rules");
    fs::write(&legacy, "user Cursor rules\n").expect("legacy");
    cockpit_agent::install_adapter(repository.path(), AgentProvider::Cursor).expect("install");
    cockpit_agent::detach_adapter(repository.path(), AgentProvider::Cursor).expect("detach");
    assert_eq!(
        fs::read_to_string(&legacy).expect("legacy"),
        "user Cursor rules\n"
    );
    assert!(!repository.path().join(".ai/adapters/cursor.json").exists());
}

#[cfg(unix)]
#[test]
fn install_rejects_symlink_target() {
    use std::os::unix::fs::symlink;

    let repository = repository(&id('a'));
    let real = repository.path().join("real.md");
    fs::write(&real, "user\n").expect("real");
    symlink(&real, repository.path().join("AGENTS.md")).expect("AGENTS symlink");
    assert!(cockpit_agent::install_adapter(repository.path(), AgentProvider::Codex).is_err());
    assert!(std::path::Path::new("AGENTS.md").is_relative());
}
