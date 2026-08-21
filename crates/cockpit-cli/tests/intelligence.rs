use std::{fs, process::Command};

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    directory
}

#[test]
fn intelligence_commands_emit_repository_bound_json_and_unknowns() {
    let directory = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(directory.path())
        .output()
        .expect("attach");
    assert!(attach.status.success());
    let start = Command::new(binary)
        .args(["start", "--repo"])
        .arg(directory.path())
        .args([
            "--id",
            "WI-INTELLIGENCE",
            "--intent",
            "traceable approach",
            "--goal",
            "test outputs",
            "--scope",
            "crates/**",
            "--authority",
            "authorized",
        ])
        .output()
        .expect("start");
    assert!(
        start.status.success(),
        "{}",
        String::from_utf8_lossy(&start.stderr)
    );

    let approach = Command::new(binary)
        .args(["work-item", "approach", "--repo"])
        .arg(directory.path())
        .args(["--id", "WI-INTELLIGENCE"])
        .output()
        .expect("approach");
    assert!(approach.status.success());
    let approach_json: serde_json::Value = serde_json::from_slice(&approach.stdout).expect("JSON");
    assert_eq!(approach_json["schemaVersion"], 2);
    assert!(!approach_json["facts"].as_array().expect("facts").is_empty());

    let inspect = Command::new(binary)
        .args(["work-item", "inspect", "--repo"])
        .arg(directory.path())
        .args(["--id", "WI-INTELLIGENCE"])
        .output()
        .expect("inspect");
    assert!(inspect.status.success());
    let inspect_json: serde_json::Value = serde_json::from_slice(&inspect.stdout).expect("JSON");
    assert_eq!(inspect_json["compatibility"]["compatible"], false);
    assert_eq!(
        inspect_json["compatibility"]["reasons"][0],
        "parallel_compatibility_not_declared"
    );

    let declare = Command::new(binary)
        .args(["work-item", "declare", "--repo"])
        .arg(directory.path())
        .args(["--id", "WI-INTELLIGENCE", "--parallelizable"])
        .output()
        .expect("declare");
    assert!(declare.status.success());
    let declared: serde_json::Value = serde_json::from_slice(&declare.stdout).expect("JSON");
    assert_eq!(declared["parallelizable"], true);
    let inspected = Command::new(binary)
        .args(["work-item", "inspect", "--repo"])
        .arg(directory.path())
        .args(["--id", "WI-INTELLIGENCE"])
        .output()
        .expect("inspect declared");
    assert!(inspected.status.success());
    let inspected_json: serde_json::Value =
        serde_json::from_slice(&inspected.stdout).expect("JSON");
    assert_eq!(inspected_json["compatibility"]["compatible"], true);

    let capability = Command::new(binary)
        .args(["capability", "show", "--repo"])
        .arg(directory.path())
        .output()
        .expect("capability");
    assert!(capability.status.success());
    let capability_json: serde_json::Value =
        serde_json::from_slice(&capability.stdout).expect("JSON");
    assert_eq!(capability_json["repositoryId"].as_str().unwrap().len(), 71);

    let diagnosis = Command::new(binary)
        .args(["diagnose", "--repo"])
        .arg(directory.path())
        .output()
        .expect("diagnose");
    assert!(diagnosis.status.success());
    let diagnosis_json: serde_json::Value =
        serde_json::from_slice(&diagnosis.stdout).expect("JSON");
    assert_eq!(diagnosis_json["state"], "unknown");
    assert!(
        diagnosis_json["unknowns"]
            .as_array()
            .expect("unknowns")
            .iter()
            .any(|item| item == "work_item_not_selected")
    );

    let knowledge = Command::new(binary)
        .args(["knowledge", "query", "--repo"])
        .arg(directory.path())
        .args(["--v2"])
        .output()
        .expect("knowledge");
    assert!(knowledge.status.success());
    let knowledge_json: serde_json::Value =
        serde_json::from_slice(&knowledge.stdout).expect("JSON");
    assert_eq!(knowledge_json["schemaVersion"], 2);
    assert!(
        directory
            .path()
            .join(".ai/work-items/active/WI-INTELLIGENCE.approach.json")
            .is_file()
    );
    fs::remove_dir_all(directory).expect("cleanup");
}
