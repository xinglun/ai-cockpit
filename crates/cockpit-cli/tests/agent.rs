use std::{fs, process::Command};

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("repository");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(directory.path())
        .status()
        .expect("git init");
    directory
}

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_ai-cockpit")
}

fn run(repo: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(binary())
        .args(args)
        .arg("--repo")
        .arg(repo)
        .output()
        .expect("run ai-cockpit")
}

fn attach(repo: &std::path::Path) {
    let output = run(repo, &["attach"]);
    assert!(
        output.status.success(),
        "attach stderr: {:?}",
        output.stderr
    );
}

#[test]
fn agent_commands_require_repo() {
    let output = Command::new(binary())
        .args(["agent", "doctor"])
        .output()
        .expect("run");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn agent_install_and_doctor_are_repository_bound() {
    let repository = repository();
    attach(repository.path());
    fs::write(repository.path().join("AGENTS.md"), "human rules\n").expect("AGENTS");
    let install = run(
        repository.path(),
        &["agent", "install", "--provider", "codex"],
    );
    assert!(
        install.status.success(),
        "install stderr: {:?}",
        install.stderr
    );
    assert!(repository.path().join(".ai/adapters/codex.json").is_file());
    let doctor = run(repository.path(), &["agent", "doctor", "--json"]);
    assert_eq!(doctor.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&doctor.stdout).expect("doctor JSON");
    assert_eq!(report["state"], "VERIFIED");
    assert_eq!(report["interfaces"]["cli"], "available");
}

#[test]
fn agent_auto_install_selects_the_shared_agents_surface() {
    let repository = repository();
    attach(repository.path());
    fs::write(repository.path().join("AGENTS.md"), "human rules\n").expect("AGENTS");
    let install = run(
        repository.path(),
        &["agent", "install", "--provider", "auto"],
    );
    assert!(
        install.status.success(),
        "install stderr: {:?}",
        install.stderr
    );
    assert!(repository.path().join(".ai/adapters/codex.json").is_file());
}

#[test]
fn agent_install_does_not_touch_global_files() {
    let repository = repository();
    attach(repository.path());
    fs::write(repository.path().join("AGENTS.md"), "human rules\n").expect("AGENTS");
    let home = tempfile::tempdir().expect("home");
    let global = home.path().join("AGENTS.md");
    fs::write(&global, "global rules\n").expect("global");
    let output = Command::new(binary())
        .args(["agent", "install", "--provider", "codex", "--repo"])
        .arg(repository.path())
        .env("HOME", home.path())
        .output()
        .expect("install");
    assert!(output.status.success(), "stderr: {:?}", output.stderr);
    assert_eq!(
        fs::read_to_string(global).expect("global after"),
        "global rules\n"
    );
}

#[test]
fn agent_detach_refuses_modified_content() {
    let repository = repository();
    attach(repository.path());
    let target = repository.path().join("AGENTS.md");
    fs::write(&target, "human rules\n").expect("AGENTS");
    let install = run(
        repository.path(),
        &["agent", "install", "--provider", "codex"],
    );
    assert!(
        install.status.success(),
        "install stderr: {:?}",
        install.stderr
    );
    let content = fs::read_to_string(&target).expect("content");
    fs::write(&target, content.replace("AI Cockpit", "a different tool")).expect("modify");
    let detach = run(
        repository.path(),
        &["agent", "detach", "--provider", "codex"],
    );
    assert!(!detach.status.success());
    assert!(repository.path().join(".ai/adapters/codex.json").is_file());
}

#[test]
fn cli_agent_operation_does_not_require_mcp() {
    let repository = repository();
    attach(repository.path());
    fs::write(repository.path().join("AGENTS.md"), "human rules\n").expect("AGENTS");
    let output = run(
        repository.path(),
        &["agent", "install", "--provider", "codex"],
    );
    assert!(output.status.success(), "stderr: {:?}", output.stderr);
}

#[test]
fn cli_cursor_install_uses_mdc_without_overwriting_legacy_user_rules() {
    let repository = repository();
    attach(repository.path());
    let legacy = repository.path().join(".cursor/rules/ai-cockpit.md");
    fs::create_dir_all(legacy.parent().expect("rules")).expect("cursor rules");
    fs::write(&legacy, "user Cursor rules\n").expect("legacy");
    let install = run(
        repository.path(),
        &["agent", "install", "--provider", "cursor"],
    );
    assert!(
        install.status.success(),
        "install stderr: {:?}",
        install.stderr
    );
    assert_eq!(
        fs::read_to_string(&legacy).expect("legacy"),
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
fn agent_help_exposes_the_first_start_gate() {
    let output = Command::new(binary())
        .args(["agent", "--help"])
        .output()
        .expect("agent help");
    assert!(output.status.success(), "stderr: {:?}", output.stderr);
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("first-start"), "help: {help}");
    assert!(
        help.contains("Before repository operations"),
        "help: {help}"
    );
}

#[test]
fn first_start_prints_the_canonical_gate_without_mutating_an_unattached_repo() {
    let repository = repository();
    let before = fs::read_dir(repository.path())
        .expect("read repository")
        .map(|entry| entry.expect("entry").file_name())
        .collect::<Vec<_>>();
    let output = run(repository.path(), &["agent", "first-start"]);
    assert!(output.status.success(), "stderr: {:?}", output.stderr);
    let procedure = String::from_utf8_lossy(&output.stdout);
    assert!(procedure.contains("Agent First-Start Procedure"));
    assert!(procedure.contains("Establish the exact repository root"));
    assert!(procedure.contains("Do not repeat a failed command blindly"));
    assert!(procedure.contains("candidate"));
    assert!(procedure.contains("fresh-install and N-1 upgrade acceptance in"));
    assert!(procedure.contains("parallel"));
    assert!(procedure.contains("Contract without release or public-artifact"));
    assert!(procedure.contains("artifact stages"));
    assert!(procedure.contains("never invent"));
    assert!(procedure.contains("publish or"));
    assert!(procedure.contains("adopter-acceptance phase"));
    assert!(procedure.contains("amend first,"));
    assert!(procedure.contains("then record one retry bound to the amended Contract"));
    assert!(procedure.contains("work-item finalize-plan"));
    assert!(procedure.contains("before verification"));
    assert!(procedure.contains("finalize-verify"));
    assert!(procedure.contains("complete those declared stages before `archive`"));
    assert!(procedure.contains("ai-cockpit capability show --repo <path>"));
    assert!(procedure.contains("Low-level Rust library helpers"));
    let after = fs::read_dir(repository.path())
        .expect("read repository")
        .map(|entry| entry.expect("entry").file_name())
        .collect::<Vec<_>>();
    assert_eq!(before, after);
}

#[test]
fn first_start_output_matches_the_installed_adapter_projection() {
    let repository = repository();
    attach(repository.path());
    fs::write(repository.path().join("AGENTS.md"), "human rules\n").expect("AGENTS");
    let install = run(
        repository.path(),
        &["agent", "install", "--provider", "codex"],
    );
    assert!(install.status.success(), "stderr: {:?}", install.stderr);
    let first_start = run(repository.path(), &["agent", "first-start"]);
    assert!(
        first_start.status.success(),
        "stderr: {:?}",
        first_start.stderr
    );
    let procedure = String::from_utf8_lossy(&first_start.stdout);
    let adapter = fs::read_to_string(repository.path().join("AGENTS.md")).expect("adapter");
    assert!(
        adapter.contains(procedure.trim()),
        "adapter does not contain first-start gate"
    );
}
