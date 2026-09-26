use cockpit_repository::{WorkItemStartOptions, attach, start_work_item_with_options};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

fn file_bytes_under(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, directory: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        let mut entries = fs::read_dir(directory)
            .expect("read directory")
            .map(|entry| entry.expect("directory entry").path())
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                visit(root, &path, files);
            } else if path.is_file() {
                let relative = path
                    .strip_prefix(root)
                    .expect("path under root")
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(relative, fs::read(path).expect("read file"));
            }
        }
    }

    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

fn git_status(root: &Path) -> Vec<u8> {
    let output = Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=all"])
        .current_dir(root)
        .output()
        .expect("git status");
    assert!(output.status.success());
    output.stdout
}

#[test]
fn inspect_returns_structured_runtime_and_repository_context() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let output = Command::new(binary)
        .args(["inspect", "--repo", env!("CARGO_MANIFEST_DIR")])
        .output()
        .expect("run ai-cockpit");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON output");
    assert_eq!(json["protocolVersion"], 1);
    assert!(json["repositoryRoot"].is_string());
    assert_eq!(json["runtimeVersion"], env!("CARGO_PKG_VERSION"));
    let expected_digest = cockpit_core::Digest::sha256_bytes(
        &std::fs::read(binary).expect("read exact executable under test"),
    )
    .to_string();
    assert_eq!(json["runtimeDigest"], expected_digest);
}

#[test]
fn read_only_diagnostics_accept_explicit_json_flag() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    for command in ["inspect", "status", "doctor"] {
        let output = Command::new(binary)
            .args([command, "--repo"])
            .arg(&repository)
            .arg("--json")
            .output()
            .expect("run ai-cockpit diagnostic");
        assert!(
            output.status.success(),
            "{command} stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<serde_json::Value>(&output.stdout)
            .unwrap_or_else(|error| panic!("{command} JSON output: {error}"));
    }
}

#[test]
fn read_only_queries_leave_governance_bytes_and_git_diff_unchanged() {
    let directory = tempfile::tempdir().expect("repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(directory.path()).expect("attach");
    start_work_item_with_options(
        directory.path(),
        "WI-READ-ONLY-QUERIES",
        "prove repository queries do not persist",
        "keep observation queries separate from write actions",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["query bytes remain unchanged".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start active Work Item");

    let bytes_before = file_bytes_under(&directory.path().join(".ai"));
    let diff_before = git_status(directory.path());
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let queries: [(&str, &[&str]); 3] = [
        ("inspect", &["inspect", "--json"]),
        ("status", &["status", "--json"]),
        ("doctor", &["doctor", "--json"]),
    ];
    for (name, args) in queries {
        let output = Command::new(binary)
            .args(args)
            .arg("--repo")
            .arg(directory.path())
            .output()
            .unwrap_or_else(|error| panic!("run {name}: {error}"));
        assert!(
            output.status.success(),
            "{name} stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let outcome = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(directory.path())
        .args(["--id", "WI-READ-ONLY-QUERIES", "--json"])
        .output()
        .expect("read outcome");
    assert!(
        outcome.status.success(),
        "outcome stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );

    assert_eq!(
        file_bytes_under(&directory.path().join(".ai")),
        bytes_before,
        "inspect/status/doctor/outcome must not mutate governance files"
    );
    assert_eq!(git_status(directory.path()), diff_before);
}
