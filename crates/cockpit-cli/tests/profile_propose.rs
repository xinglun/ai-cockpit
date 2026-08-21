use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn repository() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "cockpit-profile-propose-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .expect("git init")
            .success()
    );
    root
}

#[test]
fn profile_propose_is_candidate_only_and_does_not_change_baseline() {
    let root = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&root)
        .output()
        .expect("attach");
    assert!(attach.status.success());
    let project_path = root.join(".ai/project.json");
    let baseline = fs::read(&project_path).expect("baseline");
    fs::write(root.join("playwright.config.ts"), "export default {};\n").expect("new capability");

    let output = Command::new(binary)
        .args(["profile", "propose", "--repo"])
        .arg(&root)
        .output()
        .expect("proposal");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let proposal: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("proposal JSON");
    assert_eq!(proposal["kind"], "project_profile_amendment");
    assert_eq!(proposal["state"], "candidate");
    assert_eq!(proposal["status"], "proposed");
    assert_eq!(proposal["formalBaselineChanged"], false);
    assert!(proposal["proposal"].is_object());
    assert_eq!(
        fs::read(project_path).expect("baseline after proposal"),
        baseline
    );
    assert!(!root.join(".ai/decisions/profile-proposal.json").exists());
    fs::remove_dir_all(root).expect("cleanup");
}
