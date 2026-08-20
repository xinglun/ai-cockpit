use cockpit_git::GitRepository;
use cockpit_repository::{BuildSystem, LanguageSignal, observe};
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn observer_detects_rust_and_cargo_without_rescanning_per_checker() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("cockpit-observer-{suffix}"));
    fs::create_dir_all(path.join("tests")).expect("directories");
    fs::write(
        path.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.1.0'\nedition='2024'\n",
    )
    .expect("cargo");
    fs::write(path.join("src.rs"), "fn main() {}\n").expect("source");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&path)
        .status()
        .expect("git init");
    let git = GitRepository::discover(&path).expect("discover");
    let snapshot = git.snapshot().expect("snapshot");
    let observation = observe(&path, &snapshot).expect("observe");
    assert!(observation.languages.contains(&LanguageSignal::Rust));
    assert!(observation.build_systems.contains(&BuildSystem::Cargo));
    assert_eq!(observation.snapshot_digest.as_str().len(), 71);
    fs::remove_dir_all(path).expect("cleanup");
}
