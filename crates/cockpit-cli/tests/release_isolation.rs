use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn packaged_cli_emits_isolation_manifest_without_runtime_bootstrap() {
    let root = tempdir().expect("temporary root");
    fs::create_dir(root.path().join("nested")).expect("nested");
    fs::write(root.path().join("nested/data.txt"), b"payload").expect("data");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args([
            "isolation-manifest",
            "--root",
            root.path().to_str().expect("UTF-8 root"),
        ])
        .output()
        .expect("run packaged scanner");
    assert!(output.status.success(), "scanner failed: {:?}", output);
    let text = String::from_utf8(output.stdout).expect("UTF-8 JSONL");
    let records = text
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSONL record"))
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["path"], "nested");
    assert_eq!(records[1]["path"], "nested/data.txt");
    assert_eq!(records[1]["type"], "file");
    assert_eq!(
        records[1]["digest"],
        "sha256:239f59ed55e737c77147cf55ad0c1b030b6d7ee748a7426952f9b852d5a935e5"
    );
}

#[test]
fn packaged_cli_scans_nul_delimited_selected_paths_once() {
    let root = tempdir().expect("temporary root");
    fs::create_dir(root.path().join(".ai")).expect(".ai");
    fs::write(root.path().join(".ai/project.json"), b"{}").expect("project");
    let path_list = root.path().join("paths");
    fs::write(
        &path_list,
        b"missing.txt\0.ai/project.json\0.ai/project.json\0",
    )
    .expect("path list");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args([
            "isolation-manifest",
            "--root",
            root.path().to_str().expect("UTF-8 root"),
            "--paths-file",
            path_list.to_str().expect("UTF-8 path list"),
        ])
        .output()
        .expect("run selected scanner");
    assert!(output.status.success(), "scanner failed: {:?}", output);
    let records = String::from_utf8(output.stdout)
        .expect("UTF-8 JSONL")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSONL record"))
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["path"], ".ai/project.json");
    assert_eq!(records[1]["path"], "missing.txt");
    assert_eq!(records[1]["type"], "missing");
    assert_eq!(records[1]["mode"], "0");
    assert_eq!(records[1]["size"], "0");
    assert_eq!(records[1]["mtime"], "0");
    assert!(records[1]["digest"].is_null());
    assert!(records[1]["target"].is_null());
    assert!(records[1]["resolvedTarget"].is_null());
}
