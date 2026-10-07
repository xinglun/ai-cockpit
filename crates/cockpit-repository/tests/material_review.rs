use cockpit_protocol::Contract;
use cockpit_repository::{MaterialUnknownCause, material_review_request};
use serde_json::json;
use std::{fs, path::Path, process::Command};

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn fixture() -> (tempfile::TempDir, Contract) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "-q"]);
    fs::write(root.join("README.md"), "baseline\n").unwrap();
    git(root, &["add", "."]);
    git(
        root,
        &[
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "base",
        ],
    );
    let base = git(root, &["rev-parse", "HEAD"]);
    let contract: Contract = serde_json::from_value(json!({
        "protocolVersion": 1,
        "repositoryId": cockpit_repository::repository_id(root).to_string(),
        "workItemId": "WI-MATERIAL",
        "intent": "review bounded committed material",
        "goal": "retain exact material evidence",
        "scope": ["README.md", "src/material.rs"],
        "outOfScope": [],
        "risk": "high",
        "authority": "authorized",
        "acceptanceCriteria": ["exact material request"],
        "requiredEvidenceClasses": [],
        "baseRevision": base,
        "projectProfileDigest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "repositorySnapshotDigest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
    })).unwrap();
    (directory, contract)
}

fn commit(root: &Path) {
    git(root, &["add", "."]);
    git(
        root,
        &[
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "candidate",
        ],
    );
}

#[test]
fn committed_manifest_is_scanned_with_empty_worktree_diff_and_profile_absent() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    let (marker, _) = include_str!(
        "../../../tests/conformance/fixtures/repository-prompt-injection/repository/material.txt"
    )
    .trim()
    .split_once(';')
    .unwrap();
    let operation = ["de", "lete"].concat();
    fs::write(root.join("src/material.rs"), format!("fn material() {{ let marker = {marker:?}; let operation = {operation:?}; consume(marker, operation); }}\n")).unwrap();
    fs::write(root.join("README.md"), "new source\n").unwrap();
    commit(root);
    let request = material_review_request(root, &contract).unwrap();
    assert_eq!(request.entries.len(), 2);
    assert!(
        request
            .entries
            .iter()
            .any(|entry| entry.path == "README.md")
    );
    let rust = request
        .entries
        .iter()
        .find(|entry| entry.path == "src/material.rs")
        .unwrap();
    assert_eq!(
        rust.unknown_cause,
        Some(MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown)
    );
    assert_eq!(
        request.raw_unknown_codes,
        ["repository_material_inspection_unavailable"]
    );
    assert!(!request.review_enabled);
    assert_eq!(
        request.review_diagnostic.as_deref(),
        Some("material_review_not_enabled")
    );
    assert!(!request.material_manifest_digest.as_str().is_empty());
}

#[test]
fn dirty_and_untracked_source_refuse_request() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::write(root.join("README.md"), "dirty\n").unwrap();
    assert!(material_review_request(root, &contract).is_err());
    git(root, &["checkout", "--", "README.md"]);
    fs::write(root.join("untracked.txt"), "new\n").unwrap();
    assert!(material_review_request(root, &contract).is_err());
}

#[test]
fn untracked_unicode_ai_fact_is_not_misclassified_as_dirty_source() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join(".ai")).unwrap();
    fs::write(root.join(".ai/évidence.json"), "{}\n").unwrap();

    let request = material_review_request(root, &contract).unwrap();
    assert!(request.entries.is_empty());
}

#[test]
fn finding_is_never_reviewable() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    let payload = include_str!(
        "../../../tests/conformance/fixtures/repository-prompt-injection/repository/material.txt"
    )
    .trim();
    fs::write(
        root.join("src/material.rs"),
        format!("fn material() {{ let instruction = {payload:?}; }}\n"),
    )
    .unwrap();
    commit(root);
    let request = material_review_request(root, &contract).unwrap();
    assert!(request.blocked_by_finding);
    assert!(request.entries.iter().all(|entry| entry.unknown_cause
        != Some(MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown)));
}

#[cfg(unix)]
#[test]
fn committed_symlink_refuses_request() {
    use std::os::unix::fs::symlink;
    let (directory, contract) = fixture();
    let root = directory.path();
    symlink("README.md", root.join("linked.txt")).unwrap();
    commit(root);
    assert!(material_review_request(root, &contract).is_err());
}

#[test]
fn ai_only_descendant_retains_request_identity_but_source_change_stales_it() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::write(root.join("README.md"), "candidate\n").unwrap();
    commit(root);
    let first = material_review_request(root, &contract).unwrap();
    fs::create_dir(root.join(".ai")).unwrap();
    fs::write(root.join(".ai/note.txt"), "metadata only\n").unwrap();
    commit(root);
    let descendant = material_review_request(root, &contract).unwrap();
    assert_ne!(first.reviewed_source_head, descendant.reviewed_source_head);
    assert_eq!(
        first.material_manifest_digest,
        descendant.material_manifest_digest
    );
    assert_eq!(
        first.source_snapshot_digest,
        descendant.source_snapshot_digest
    );
    assert_eq!(first.request_digest, descendant.request_digest);
    fs::write(root.join("README.md"), "different source\n").unwrap();
    commit(root);
    let different = material_review_request(root, &contract).unwrap();
    assert_ne!(
        first.material_manifest_digest,
        different.material_manifest_digest
    );
    assert_ne!(first.request_digest, different.request_digest);
}

#[test]
fn normalized_crlf_checkout_is_not_compared_to_committed_blob_bytes() {
    let (directory, contract) = fixture();
    let root = directory.path();
    git(root, &["config", "core.autocrlf", "true"]);
    fs::write(root.join("README.md"), b"candidate\r\n").unwrap();
    commit(root);

    let checkout_bytes = fs::read(root.join("README.md")).unwrap();
    let blob_id = git(root, &["rev-parse", "HEAD:README.md"]);
    let committed_blob = Command::new("git")
        .args(["cat-file", "blob", &blob_id])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(committed_blob.status.success());
    assert_eq!(checkout_bytes, b"candidate\r\n");
    assert_eq!(committed_blob.stdout, b"candidate\n");

    let request = material_review_request(root, &contract).unwrap();
    let readme = request
        .entries
        .iter()
        .find(|entry| entry.path == "README.md")
        .unwrap();
    assert!(readme.after_blob_digest.is_some());
}

#[cfg(unix)]
#[test]
fn no_hunk_text_changes_still_bind_complete_committed_source() {
    use std::os::unix::fs::PermissionsExt;

    let (directory, mut contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("src/chmod.rs"), "fn permission() {}\n").unwrap();
    commit(root);
    contract.base_revision = git(root, &["rev-parse", "HEAD"]);

    fs::write(root.join("src/empty.rs"), "").unwrap();
    let chmod_file = root.join("src/chmod.rs");
    let mut permissions = fs::metadata(&chmod_file).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&chmod_file, permissions).unwrap();
    commit(root);

    let request = material_review_request(root, &contract).unwrap();
    for path in ["src/chmod.rs", "src/empty.rs"] {
        let entry = request
            .entries
            .iter()
            .find(|entry| entry.path == path)
            .unwrap();
        assert_eq!(entry.content_state, "text");
        assert_eq!(entry.unknown_cause, None);
        assert!(entry.after_blob_digest.is_some());
    }
}

#[test]
fn staged_source_refuses_request() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::write(root.join("README.md"), "staged\n").unwrap();
    git(root, &["add", "README.md"]);
    assert!(material_review_request(root, &contract).is_err());
}

#[test]
fn staged_source_rename_into_ai_does_not_hide_original_source_change() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join(".ai")).unwrap();
    git(root, &["mv", "README.md", ".ai/readme.md"]);

    assert!(material_review_request(root, &contract).is_err());
}

#[test]
fn non_ancestor_rebase_cannot_become_a_canonical_request() {
    let (directory, contract) = fixture();
    let root = directory.path();
    git(root, &["checkout", "-q", "--orphan", "unrelated"]);
    git(root, &["rm", "-q", "-f", "README.md"]);
    fs::write(root.join("README.md"), "unrelated history\n").unwrap();
    commit(root);
    assert!(material_review_request(root, &contract).is_err());
}

#[test]
fn assume_unchanged_source_cannot_masquerade_as_committed_material() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::write(root.join("README.md"), "candidate\n").unwrap();
    commit(root);
    git(root, &["update-index", "--assume-unchanged", "README.md"]);
    fs::write(root.join("README.md"), "uncommitted substitute\n").unwrap();
    let outcome = material_review_request(root, &contract);
    git(
        root,
        &["update-index", "--no-assume-unchanged", "README.md"],
    );
    assert!(outcome.is_err());
}

#[test]
fn binary_rust_material_is_unknown_but_not_reviewable() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("src/material.rs"), b"binary\0material").unwrap();
    commit(root);
    let request = material_review_request(root, &contract).unwrap();
    let entry = request
        .entries
        .iter()
        .find(|entry| entry.path == "src/material.rs")
        .unwrap();
    assert_eq!(
        entry.unknown_cause,
        Some(MaterialUnknownCause::NonTextMaterial)
    );
    assert!(!entry.reviewable);
    assert!(
        request
            .raw_unknown_codes
            .contains(&"repository_material_inspection_unavailable".into())
    );
}

#[test]
fn bounded_patch_in_large_committed_rust_source_uses_committed_blob() {
    let (directory, mut contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    let large = format!(
        "fn material() {{ let value = 1; }}\n{}",
        "// filler\n".repeat(35_000)
    );
    assert!(large.len() > cockpit_git::MAX_CHANGE_TEXT_BYTES);
    fs::write(root.join("src/material.rs"), &large).unwrap();
    commit(root);
    contract.base_revision = git(root, &["rev-parse", "HEAD"]);
    fs::write(
        root.join("src/material.rs"),
        large.replacen("value = 1", "value = 2", 1),
    )
    .unwrap();
    commit(root);
    let request = material_review_request(root, &contract).unwrap();
    let entry = request
        .entries
        .iter()
        .find(|entry| entry.path == "src/material.rs")
        .unwrap();
    assert!(!entry.reviewable);
    assert_eq!(entry.unknown_cause, None);
}

#[test]
fn bounded_patch_with_over_budget_source_is_typed_unreviewable() {
    let (directory, mut contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    let huge = format!(
        "fn material() {{ let value = 1; }}\n/*{}*/\n",
        "x".repeat(4 * 1024 * 1024)
    );
    fs::write(root.join("src/material.rs"), &huge).unwrap();
    commit(root);
    contract.base_revision = git(root, &["rev-parse", "HEAD"]);
    fs::write(
        root.join("src/material.rs"),
        huge.replacen("value = 1", "value = 2", 1),
    )
    .unwrap();
    commit(root);
    let request = material_review_request(root, &contract).unwrap();
    let entry = request
        .entries
        .iter()
        .find(|entry| entry.path == "src/material.rs")
        .unwrap();
    assert_eq!(
        entry.unknown_cause,
        Some(MaterialUnknownCause::SourceOverBudget)
    );
    assert!(!entry.reviewable);
}

#[cfg(unix)]
#[test]
fn unreadable_committed_source_refuses_request() {
    use std::os::unix::fs::PermissionsExt;
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::write(root.join("README.md"), "candidate\n").unwrap();
    commit(root);
    let file = root.join("README.md");
    let original = fs::metadata(&file).unwrap().permissions();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).unwrap();
    let outcome = material_review_request(root, &contract);
    fs::set_permissions(&file, original).unwrap();
    assert!(outcome.is_err());
}
