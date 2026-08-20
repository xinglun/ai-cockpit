use cockpit_git::GitRepository;
use cockpit_repository::{BuildSystem, LanguageSignal, observe, observe_cached};
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
    cockpit_repository::attach(&path).expect("attach for cache");
    let attached_snapshot = git.snapshot().expect("attached snapshot");
    let cached = observe_cached(&path, &attached_snapshot).expect("cached observation");
    assert!(!cached.cache_hit);
    let second_snapshot = git.snapshot().expect("second snapshot");
    let cached_again = observe_cached(&path, &second_snapshot).expect("cache hit");
    assert!(cached_again.cache_hit);
    fs::write(path.join("src.rs"), "fn changed() {}\n").expect("change source");
    let changed_snapshot = git.snapshot().expect("changed snapshot");
    let refreshed = observe_cached(&path, &changed_snapshot).expect("refresh");
    assert!(!refreshed.cache_hit);
    fs::remove_dir_all(path).expect("cleanup");
}
