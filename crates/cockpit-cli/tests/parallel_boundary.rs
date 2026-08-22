use std::process::Command;

fn run(binary: &std::path::Path, repo: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(binary)
        .args(args)
        .arg("--repo")
        .arg(repo)
        .output()
        .expect("run CLI")
}

#[test]
fn cli_binds_contract_boundary_and_manages_slot_lease() {
    let repo = tempfile::tempdir().expect("repo");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(repo.path())
            .status()
            .expect("git init")
            .success()
    );
    let binary = std::path::Path::new(env!("CARGO_BIN_EXE_ai-cockpit"));
    assert!(run(binary, repo.path(), &["attach"]).status.success());
    let start = run(
        binary,
        repo.path(),
        &[
            "start",
            "--id",
            "WI-CLI-PARALLEL",
            "--intent",
            "test boundary",
            "--goal",
            "test slot",
            "--scope",
            "src/main.rs",
            "--authority",
            "authorized",
            "--acceptance",
            "boundary binds",
        ],
    );
    assert!(
        start.status.success(),
        "{}",
        String::from_utf8_lossy(&start.stderr)
    );
    assert!(
        run(
            binary,
            repo.path(),
            &[
                "work-item",
                "declare",
                "--id",
                "WI-CLI-PARALLEL",
                "--parallelizable"
            ]
        )
        .status
        .success()
    );
    let boundary = repo.path().join("boundary.json");
    std::fs::write(
        &boundary,
        serde_json::json!({
            "schemaVersion": 1,
            "implementationPaths": ["src/main.rs"],
            "generatedEvidencePaths": [".ai/evidence/**"],
            "verificationOutputPaths": ["target/**"],
            "serializedProjectionPaths": [".ai/work-items/**"],
            "maxWorkers": 1,
            "reason": "CLI boundary test"
        })
        .to_string(),
    )
    .expect("boundary file");
    // The file path is passed explicitly so the test exercises the same
    // repository-bound invocation as a user command.
    let output = Command::new(binary)
        .args(["work-item", "boundary", "--id", "WI-CLI-PARALLEL", "--file"])
        .arg(&boundary)
        .args(["--repo"])
        .arg(repo.path())
        .output()
        .expect("bind boundary");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = run(
        binary,
        repo.path(),
        &["work-item", "slot", "acquire", "--id", "WI-CLI-PARALLEL"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lease: serde_json::Value = serde_json::from_slice(&output.stdout).expect("lease JSON");
    let lease_id = lease["leaseId"].as_str().expect("lease id");
    let output = Command::new(binary)
        .args([
            "work-item",
            "slot",
            "release",
            "--id",
            "WI-CLI-PARALLEL",
            "--lease-id",
        ])
        .arg(lease_id)
        .args(["--repo"])
        .arg(repo.path())
        .output()
        .expect("release slot");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
