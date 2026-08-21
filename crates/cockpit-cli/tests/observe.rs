use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn observe_returns_languages_build_systems_and_evolution() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-observe-cli-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("directory");
    fs::write(
        directory.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.1.0'\nedition='2024'\n",
    )
    .expect("cargo");
    fs::write(directory.join("lib.rs"), "pub fn value() -> u8 { 1 }\n").expect("source");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["observe", "--repo"])
        .arg(&directory)
        .output()
        .expect("observe");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert!(
        json["languages"]
            .as_array()
            .expect("languages")
            .iter()
            .any(|value| value == "Rust")
    );
    assert!(
        json["buildSystems"]
            .as_array()
            .expect("build systems")
            .iter()
            .any(|value| value == "Cargo")
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn observe_accepts_the_attached_profile_wrapper() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-observe-attached-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&directory)
        .output()
        .expect("attach");
    assert!(
        attach.status.success(),
        "attach failed: {}",
        String::from_utf8_lossy(&attach.stderr)
    );
    let observe = Command::new(binary)
        .args(["observe", "--repo"])
        .arg(&directory)
        .output()
        .expect("observe");
    assert!(
        observe.status.success(),
        "observe failed: {}",
        String::from_utf8_lossy(&observe.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&observe.stdout).expect("JSON");
    assert!(json.get("evolution").is_some());
    fs::remove_dir_all(directory).expect("cleanup");
}
