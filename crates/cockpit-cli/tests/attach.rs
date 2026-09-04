use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

fn fixture_repository() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-attach-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    directory
}

#[test]
fn attach_creates_only_protocol_state_and_is_idempotent() {
    let directory = fixture_repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let first = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&directory)
        .output()
        .expect("attach");
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(directory.join(".ai/cockpit.toml").is_file());
    assert!(directory.join(".ai/project.json").is_file());
    let manifest_path = directory.join(".ai/agent-interface.json");
    assert!(manifest_path.is_file());
    let manifest_before = fs::read(&manifest_path).expect("manifest");
    let manifest_json: serde_json::Value =
        serde_json::from_slice(&manifest_before).expect("manifest JSON");
    let capabilities = manifest_json["capabilities"]
        .as_array()
        .expect("capabilities array");
    for expected in cockpit_protocol::AGENT_INTERFACE_CAPABILITIES {
        assert!(
            capabilities
                .iter()
                .any(|value| value.as_str() == Some(expected)),
            "attach discovery manifest must advertise {expected}"
        );
    }
    assert_eq!(
        capabilities.len(),
        cockpit_protocol::AGENT_INTERFACE_CAPABILITIES.len(),
        "capability registry must not silently drift"
    );
    assert!(directory.join(".ai/work-items/active").is_dir());
    assert!(!directory.join("scripts").exists());
    for forbidden in ["AGENTS.md", "CLAUDE.md", "GEMINI.md", ".cursor"] {
        assert!(
            !directory.join(forbidden).exists(),
            "attach wrote {forbidden}"
        );
    }
    let second = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&directory)
        .output()
        .expect("attach twice");
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(fs::read(manifest_path).expect("manifest"), manifest_before);
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn attached_repository_identity_survives_a_repository_move() {
    let directory = fixture_repository();
    let moved = directory.with_file_name(format!(
        "{}-moved",
        directory
            .file_name()
            .expect("fixture name")
            .to_string_lossy()
    ));
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let first = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&directory)
        .output()
        .expect("attach");
    assert!(first.status.success());
    let first_json: serde_json::Value = serde_json::from_slice(&first.stdout).expect("profile");
    let first_id = first_json["repositoryId"].clone();

    fs::rename(&directory, &moved).expect("move repository");
    let second = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&moved)
        .output()
        .expect("reattach moved repository");
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_json: serde_json::Value = serde_json::from_slice(&second.stdout).expect("profile");
    assert_eq!(second_json["repositoryId"], first_id);
    fs::remove_dir_all(moved).expect("cleanup");
}

#[test]
fn status_reports_calibration_required_before_first_profile_confirmation() {
    let directory = fixture_repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&directory)
        .output()
        .expect("attach");
    assert!(attach.status.success());
    let status = Command::new(binary)
        .args(["status", "--repo"])
        .arg(&directory)
        .output()
        .expect("status");
    assert!(
        status.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&status.stdout).expect("JSON");
    assert_eq!(json["state"], "calibration_required");
    fs::remove_dir_all(directory).expect("cleanup");
}

fn migration_fixture_repository() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-migration-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("directory");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&directory)
            .status()
            .expect("git init")
            .success()
    );
    directory
}

fn migration_run(
    binary: &str,
    repository: &std::path::Path,
    args: &[&str],
) -> std::process::Output {
    Command::new(binary)
        .args(args)
        .args(["--repo"])
        .arg(repository)
        .output()
        .expect("command")
}

#[test]
fn old_repository_requires_explicit_migration_and_preserves_history() {
    let repository = migration_fixture_repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attached = migration_run(binary, &repository, &["attach"]);
    assert!(attached.status.success(), "attach: {:?}", attached);

    let project_path = repository.join(".ai/project.json");
    let manifest_path = repository.join(".ai/agent-interface.json");
    let config_path = repository.join(".ai/cockpit.toml");
    for path in [&project_path, &manifest_path] {
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(path).expect("JSON bytes")).expect("JSON");
        value
            .as_object_mut()
            .expect("object")
            .remove("repositorySchemaVersion");
        fs::write(path, serde_json::to_vec_pretty(&value).expect("JSON")).expect("write JSON");
    }
    let config = fs::read_to_string(&config_path).expect("config");
    fs::write(
        &config_path,
        config
            .lines()
            .filter(|line| !line.starts_with("repository_schema_version"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .expect("write legacy config");

    let historical = repository.join(".ai/evidence/historical.json");
    let archived = repository.join(".ai/work-items/archive/historical.contract.json");
    fs::write(&historical, br#"{"kind":"historical","immutable":true}"#).expect("evidence");
    fs::write(
        &archived,
        br#"{"workItemId":"historical","state":"closed"}"#,
    )
    .expect("archive");
    let historical_before = fs::read(&historical).expect("evidence");
    let archived_before = fs::read(&archived).expect("archive");

    let compatibility = migration_run(binary, &repository, &["compatibility"]);
    assert!(
        compatibility.status.success(),
        "compatibility: {compatibility:?}"
    );
    let compatibility_json: serde_json::Value =
        serde_json::from_slice(&compatibility.stdout).expect("compatibility JSON");
    assert_eq!(compatibility_json["state"], "MIGRATION_REQUIRED");
    assert_eq!(compatibility_json["repositorySchemaVersion"], 1);

    let plan = migration_run(binary, &repository, &["migrate", "plan"]);
    assert!(plan.status.success(), "plan: {plan:?}");
    let plan_json: serde_json::Value = serde_json::from_slice(&plan.stdout).expect("plan JSON");
    assert_eq!(plan_json["state"], "MIGRATION_REQUIRED");
    assert_eq!(plan_json["humanApprovalRequired"], true);
    assert_eq!(plan_json["currentSchema"], 1);

    let before_apply = (
        fs::read(&config_path).expect("config"),
        fs::read(&project_path).expect("project"),
        fs::read(&manifest_path).expect("manifest"),
    );
    let denied = migration_run(binary, &repository, &["migrate", "apply"]);
    assert!(!denied.status.success());
    assert_eq!(fs::read(&config_path).expect("config"), before_apply.0);
    assert_eq!(fs::read(&project_path).expect("project"), before_apply.1);
    assert_eq!(fs::read(&manifest_path).expect("manifest"), before_apply.2);

    let applied = migration_run(binary, &repository, &["migrate", "apply", "--approved"]);
    assert!(applied.status.success(), "apply: {applied:?}");
    let receipt: serde_json::Value = serde_json::from_slice(&applied.stdout).expect("receipt JSON");
    assert_eq!(receipt["fromSchema"], 1);
    assert_eq!(receipt["toSchema"], 2);
    assert_eq!(receipt["result"], "completed");
    assert!(
        receipt["runtimeDigest"]
            .as_str()
            .unwrap_or_default()
            .starts_with("sha256:")
    );

    let migrated = migration_run(binary, &repository, &["compatibility"]);
    assert!(migrated.status.success(), "compatibility: {migrated:?}");
    let migrated_json: serde_json::Value = serde_json::from_slice(&migrated.stdout).expect("JSON");
    assert_eq!(migrated_json["state"], "COMPATIBLE");
    assert_eq!(migrated_json["repositorySchemaVersion"], 2);
    assert_eq!(fs::read(&historical).expect("evidence"), historical_before);
    assert_eq!(fs::read(&archived).expect("archive"), archived_before);
    assert!(
        repository
            .join(".ai/migrations")
            .read_dir()
            .expect("migrations")
            .next()
            .is_some()
    );

    let repeated = migration_run(binary, &repository, &["migrate", "apply", "--approved"]);
    assert!(!repeated.status.success());
    fs::remove_dir_all(repository).expect("cleanup");
}

#[test]
fn migration_rejects_an_unreviewed_schema_without_skipping_history() {
    let repository = migration_fixture_repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attached = migration_run(binary, &repository, &["attach"]);
    assert!(attached.status.success(), "attach: {attached:?}");

    let config_path = repository.join(".ai/cockpit.toml");
    let project_path = repository.join(".ai/project.json");
    let manifest_path = repository.join(".ai/agent-interface.json");
    let mut config = fs::read_to_string(&config_path).expect("config");
    config = config.replace(
        "repository_schema_version = 2",
        "repository_schema_version = 0",
    );
    fs::write(&config_path, config).expect("write config");
    for path in [&project_path, &manifest_path] {
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(path).expect("JSON bytes")).expect("JSON");
        value["repositorySchemaVersion"] = serde_json::json!(0);
        fs::write(path, serde_json::to_vec_pretty(&value).expect("JSON")).expect("write JSON");
    }
    let before = (
        fs::read(&config_path).expect("config"),
        fs::read(&project_path).expect("project"),
        fs::read(&manifest_path).expect("manifest"),
    );
    let plan = migration_run(binary, &repository, &["migrate", "plan"]);
    assert!(!plan.status.success(), "unreviewed schema must fail closed");
    assert!(String::from_utf8_lossy(&plan.stderr).contains("no reviewed adjacent migration"));
    let apply = migration_run(binary, &repository, &["migrate", "apply", "--approved"]);
    assert!(
        !apply.status.success(),
        "unreviewed schema must not be skipped"
    );
    assert_eq!(fs::read(&config_path).expect("config"), before.0);
    assert_eq!(fs::read(&project_path).expect("project"), before.1);
    assert_eq!(fs::read(&manifest_path).expect("manifest"), before.2);
    fs::remove_dir_all(repository).expect("cleanup");
}
