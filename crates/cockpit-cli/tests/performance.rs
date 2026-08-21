use std::{
    fs,
    process::Command,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn fixture() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "cockpit-performance-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    fs::write(directory.join("Cargo.toml"), "[workspace]\nmembers=[]\n").expect("cargo");
    directory
}

#[test]
fn status_warm_startup_is_measured_and_bounded() {
    let repo = fixture();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&repo)
        .status()
        .expect("attach");
    assert!(attach.success());

    let mut samples = Vec::new();
    for _ in 0..12 {
        let started = Instant::now();
        let output = Command::new(binary)
            .args(["status", "--repo"])
            .arg(&repo)
            .output()
            .expect("status");
        assert!(output.status.success());
        samples.push(started.elapsed());
    }
    samples.sort_unstable();
    let median = samples[samples.len() / 2];
    eprintln!(
        "{{\"benchmark\":\"status-startup\",\"samples\":{},\"medianMs\":{}}}",
        samples.len(),
        median.as_millis()
    );
    assert!(
        median < Duration::from_secs(1),
        "status median was {median:?}"
    );
    fs::remove_dir_all(repo).expect("cleanup");
}

#[test]
fn observation_medium_fixture_is_measured_once() {
    let repo = fixture();
    for index in 0..200 {
        fs::write(
            repo.join(format!("src-{index}.rs")),
            "pub fn fixture() {}\n",
        )
        .expect("source");
    }
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&repo)
        .output()
        .expect("attach");
    assert!(attach.status.success());
    let warm = Command::new(binary)
        .args(["observe", "--repo"])
        .arg(&repo)
        .output()
        .expect("warm observe");
    assert!(warm.status.success());
    let started = Instant::now();
    let output = Command::new(binary)
        .args(["observe", "--repo"])
        .arg(&repo)
        .output()
        .expect("observe");
    assert!(output.status.success());
    let elapsed = started.elapsed();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert!(json["filesRead"].as_u64().unwrap_or_default() >= 200);
    assert_eq!(json["cacheHit"], true);
    eprintln!(
        "{{\"benchmark\":\"observation-medium\",\"filesRead\":{},\"elapsedMs\":{}}}",
        json["filesRead"],
        elapsed.as_millis()
    );
    assert!(
        elapsed < Duration::from_secs(2),
        "observation took {elapsed:?}"
    );
    fs::remove_dir_all(repo).expect("cleanup");
}
