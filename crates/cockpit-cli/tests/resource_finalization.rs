use sha2::{Digest as ShaDigest, Sha256};
use std::{fs, path::Path, process::Command};

fn run(binary: &str, args: &[&str], repo: &Path) -> std::process::Output {
    Command::new(binary)
        .args(args)
        .args(["--repo"])
        .arg(repo)
        .current_dir(repo)
        .output()
        .expect("run ai-cockpit")
}

fn run_json(binary: &str, args: &[&str], repo: &Path) -> serde_json::Value {
    let output = run(binary, args, repo);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("JSON output")
}

fn runtime_digest(binary: &str) -> String {
    let bytes = fs::read(binary).expect("runtime binary");
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn repository() -> tempfile::TempDir {
    let repo = tempfile::tempdir().expect("temp repo");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(repo.path())
            .status()
            .expect("git init")
            .success()
    );
    for (key, value) in [
        ("user.email", "test@example.invalid"),
        ("user.name", "Test"),
    ] {
        assert!(
            Command::new("git")
                .args(["config", key, value])
                .current_dir(repo.path())
                .status()
                .expect("git config")
                .success()
        );
    }
    fs::write(repo.path().join("README.md"), "fixture baseline\n").expect("baseline README");
    assert!(
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(repo.path())
            .status()
            .expect("git add baseline")
            .success()
    );
    assert!(
        Command::new("git")
            .args(["commit", "-qm", "fixture baseline"])
            .current_dir(repo.path())
            .status()
            .expect("git commit baseline")
            .success()
    );
    repo
}

fn commit_staged(root: &Path, message: &str) -> String {
    git(root, &["commit", "-q", "-m", message]);
    git(root, &["rev-parse", "HEAD"])
}

fn seed_sync_projection_docs(root: &Path, id: &str) {
    let short = id.split('-').take(2).collect::<Vec<_>>().join("-");
    for suffix in ["", ".zh-CN", ".ja"] {
        let parity = root.join(format!("docs/reference/reference-parity{suffix}.md"));
        fs::create_dir_all(parity.parent().unwrap()).unwrap();
        fs::write(
            parity,
            format!("| Work Item | Status | Evidence |\n| --- | --- | --- |\n| {short} | In progress | pending |\n| WI-8888 | Unrelated | unchanged |\n"),
        )
        .unwrap();
        let page = root.join(format!("docs/work-items/{id}{suffix}.md"));
        fs::create_dir_all(page.parent().unwrap()).unwrap();
        fs::write(
            page,
            format!("---\ntitle: Synthetic post-merge fixture\nstatus: in_progress\nlastVerifiedBy: {id}\n---\n\nPreserve this page body.\n"),
        )
        .unwrap();
    }
    git(root, &["add", "docs"]);
    commit_staged(root, "seed synthetic terminal projection docs");
}

fn promote_sync_projection_docs(root: &Path, id: &str) {
    let short = id.split('-').take(2).collect::<Vec<_>>().join("-");
    for (suffix, status) in [
        ("", "Implemented"),
        (".zh-CN", "已实现"),
        (".ja", "Implemented"),
    ] {
        let parity = root.join(format!("docs/reference/reference-parity{suffix}.md"));
        let mut text = fs::read_to_string(&parity).unwrap();
        let row = format!("| {short} | In progress | pending |\n");
        let evidence = format!(
            "[Work Item](../work-items/{id}{suffix}.md); archive `.ai/work-items/archive/{id}.contract.json`; verification `.ai/evidence/{id}.verification.json`; finalization `.ai/decisions/{id}.finalize.synthetic.json`; close `.ai/decisions/{id}.close.json`."
        );
        text = text.replace(&row, &format!("| {short} | {status} | {evidence} |\n"));
        fs::write(parity, text).unwrap();

        let page = root.join(format!("docs/work-items/{id}{suffix}.md"));
        let mut text = fs::read_to_string(&page).unwrap();
        text = text.replace("status: in_progress", "status: implemented");
        text = text.replace(
            &format!("lastVerifiedBy: {id}\n---"),
            &format!("lastVerifiedBy: {id}\nterminalArchive: .ai/work-items/archive/{id}.contract.json\nterminalVerification: .ai/evidence/{id}.verification.json\nterminalFinalization: .ai/decisions/{id}.finalize.synthetic.json\nterminalDecision: .ai/decisions/{id}.close.json\n---"),
        );
        fs::write(page, text).unwrap();
    }
    git(root, &["add", "docs"]);
    commit_staged(root, "promote terminal projection docs");
}

fn assert_success(output: &std::process::Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("git output")
        .trim()
        .to_owned()
}

fn direct_merge_repository_for_cli() -> (tempfile::TempDir, String) {
    let repo = repository();
    let root = repo.path();
    git(root, &["config", "user.email", "tests@example.invalid"]);
    git(root, &["config", "user.name", "AI Cockpit Tests"]);
    git(root, &["branch", "-M", "main"]);
    fs::write(root.join("README.md"), "direct merge CLI fixture\n").expect("write base");
    git(root, &["add", "README.md"]);
    git(root, &["commit", "-q", "-m", "base"]);
    let base = git(root, &["rev-parse", "HEAD"]);
    git(root, &["checkout", "-q", "-b", "feature/direct-merge-cli"]);
    fs::write(root.join("direct-merge.txt"), "historical merge\n").expect("write feature");
    git(root, &["add", "direct-merge.txt"]);
    git(root, &["commit", "-q", "-m", "direct merge feature"]);
    let feature_head = git(root, &["rev-parse", "HEAD"]);
    git(root, &["checkout", "-q", "main"]);
    git(
        root,
        &[
            "merge",
            "--no-ff",
            "-q",
            "feature/direct-merge-cli",
            "-m",
            "direct merge",
        ],
    );
    let merge_commit = git(root, &["rev-parse", "HEAD"]);
    git(root, &["reset", "--hard", "-q", &base]);
    git(root, &["branch", "-D", "feature/direct-merge-cli"]);
    assert!(!feature_head.is_empty());
    (repo, merge_commit)
}

#[test]
fn historical_recovery_commands_are_discoverable_in_work_item_help() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let output = Command::new(binary)
        .args(["work-item", "--help"])
        .output()
        .expect("run work-item help");
    assert_success(&output, "work-item help");
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("finalize-recovery"), "help: {help}");
    assert!(help.contains("finalize-recovery-plan"), "help: {help}");
}

#[test]
fn finalize_recovery_help_describes_first_direct_merge_apply() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let output = Command::new(binary)
        .args(["work-item", "finalize-recovery", "--help"])
        .output()
        .expect("run finalize-recovery help");
    assert_success(&output, "finalize-recovery help");
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("first direct-merge"), "help: {help}");
    assert!(
        help.contains("complete direct-merge receipt"),
        "help: {help}"
    );
}

#[test]
fn cli_direct_merge_plan_round_trips_through_recovery_and_finalize() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let (repo, merge_commit) = direct_merge_repository_for_cli();
    let root = repo.path();
    let id = "WI-DIRECT-MERGE-CLI";
    assert_success(&run(binary, &["attach"], root), "attach");

    let context_file = tempfile::NamedTempFile::new().expect("context file");
    fs::write(
        context_file.path(),
        serde_json::to_vec_pretty(&serde_json::json!({
            "branch": "feature/direct-merge-cli",
            "worktree": root.to_string_lossy(),
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "local",
            "pullRequest": "https://github.com/example/project/pull/999"
        }))
        .expect("context JSON"),
    )
    .expect("write context");
    let context_arg = context_file.path().to_string_lossy().into_owned();
    assert_success(
        &run(
            binary,
            &[
                "start",
                "--id",
                id,
                "--intent",
                "record historical direct merge",
                "--goal",
                "preserve truthful no-PR history",
                "--scope",
                ".ai/**",
                "--authority",
                "authorized",
            ],
            root,
        ),
        "start",
    );
    let plan = run_json(
        binary,
        &[
            "work-item",
            "finalize-plan",
            "--id",
            id,
            "--input",
            context_arg.as_str(),
        ],
        root,
    );
    assert_eq!(plan["state"], "planned");
    assert_success(
        &run(
            binary,
            &[
                "preflight",
                "--contract",
                &format!(".ai/work-items/active/{id}.contract.json"),
            ],
            root,
        ),
        "preflight",
    );
    assert_success(
        &run(binary, &["checkpoint", "--id", id], root),
        "checkpoint",
    );
    assert_success(
        &run(
            binary,
            &["verify", "--work-item", id, "--command", "true"],
            root,
        ),
        "verify",
    );
    assert_success(&run(binary, &["finish", "--id", id], root), "finish");
    assert_success(&run(binary, &["archive", "--id", id], root), "archive");

    let recovery_plan = run_json(
        binary,
        &[
            "work-item",
            "finalize-recovery-plan",
            "--id",
            id,
            "--merge-commit",
            merge_commit.as_str(),
        ],
        root,
    );
    assert_eq!(recovery_plan["state"], "direct_merge_candidate");
    assert_eq!(recovery_plan["historicalKind"], "direct_merge_no_pr");
    assert_eq!(
        recovery_plan["suggestedReceipt"]["pullRequest"]["number"],
        0
    );
    assert_eq!(
        recovery_plan["suggestedReceipt"]["historical"]["mergeCommit"],
        merge_commit
    );
    assert_eq!(
        recovery_plan["suggestedReceipt"]["resourceContext"]["provider"],
        "historical"
    );
    let mut receipt = recovery_plan["suggestedReceipt"].clone();
    receipt["actor"] = "human:test".into();
    receipt["authoritySource"] = "historical-test".into();
    receipt["reason"] = "record a direct merge without inventing a pull request".into();
    receipt["timestamp"] = "2026-09-07T00:00:00Z".into();
    let receipt_file = tempfile::NamedTempFile::new().expect("receipt file");
    fs::write(
        receipt_file.path(),
        serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
    )
    .expect("write receipt");
    let receipt_arg = receipt_file.path().to_string_lossy().into_owned();
    let recovered = run_json(
        binary,
        &[
            "work-item",
            "finalize-recovery",
            "--id",
            id,
            "--input",
            receipt_arg.as_str(),
        ],
        root,
    );
    assert_eq!(recovered["state"], "recorded");
    let replay = run_json(
        binary,
        &[
            "work-item",
            "finalize",
            "--id",
            id,
            "--input",
            receipt_arg.as_str(),
        ],
        root,
    );
    assert_eq!(replay["state"], "idempotent");
    let verified = run_json(binary, &["work-item", "finalize-verify", "--id", id], root);
    assert_eq!(verified["state"], "verified");
    assert_eq!(verified["historicalKind"], "direct_merge_no_pr");
    let closed = run(
        binary,
        &[
            "close",
            "--id",
            id,
            "--human-decision",
            "approved",
            "--actor",
            "human:test",
            "--authority-source",
            "historical-test",
            "--reason",
            "close verified historical direct merge",
            "--evidence-ref",
            &format!(".ai/decisions/{id}.finalize.json"),
            "--decided-at",
            "2026-09-07T00:01:00Z",
        ],
        root,
    );
    assert_success(&closed, "close");
}

#[test]
fn cli_post_merge_sync_retained_root_deleted_transition_verify_and_human_close() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = repository();
    let root = repo.path();
    let id = "WI-9997-post-merge-sync-cli";
    let source_branch = "codex/wi9997-source";
    let sync_branch = "codex/wi9997-sync";
    let worktrees = tempfile::tempdir().expect("worktree parent");
    let worktree_path = worktrees.path().join("sync-worktree");
    let worktree_arg = worktree_path.to_string_lossy().into_owned();
    fs::write(root.join(".gitignore"), ".ai/locks/\n").unwrap();
    git(root, &["add", ".gitignore"]);
    commit_staged(root, "ignore ephemeral lifecycle locks");
    git(root, &["branch", "-M", "main"]);
    git(
        root,
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/example/fixture.git",
        ],
    );
    let base = git(root, &["rev-parse", "HEAD"]);
    git(root, &["update-ref", "refs/remotes/origin/main", &base]);
    git(
        root,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    git(
        root,
        &[
            "worktree",
            "add",
            "-b",
            source_branch,
            worktree_arg.as_str(),
            "main",
        ],
    );
    git(
        root,
        &["branch", "--set-upstream-to=origin/main", source_branch],
    );
    assert_success(
        &run(binary, &["attach"], &worktree_path),
        "attach source worktree",
    );
    assert_success(
        &run(
            binary,
            &[
                "start",
                "--id",
                id,
                "--intent",
                "record synthetic post-merge sync lifecycle",
                "--goal",
                "exercise the public Retained to Deleted finalization path",
                "--scope",
                ".ai/**",
                "--authority",
                "authorized",
                "--acceptance",
                "CLI finalization preserves the merged PR identity and sync cleanup evidence",
                "--required-evidence",
                "verification",
            ],
            &worktree_path,
        ),
        "start synthetic Work Item",
    );
    assert_success(
        &run(
            binary,
            &[
                "preflight",
                "--contract",
                &format!(".ai/work-items/active/{id}.contract.json"),
            ],
            &worktree_path,
        ),
        "preflight synthetic Work Item",
    );
    assert_success(
        &run(binary, &["checkpoint", "--id", id], &worktree_path),
        "checkpoint synthetic Work Item",
    );
    let context = tempfile::NamedTempFile::new().expect("resource context");
    fs::write(
        context.path(),
        serde_json::to_vec_pretty(&serde_json::json!({
            "branch": sync_branch,
            "worktree": worktree_arg,
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": "https://github.com/example/fixture/pull/9001"
        }))
        .unwrap(),
    )
    .unwrap();
    assert_success(
        &run(
            binary,
            &[
                "work-item",
                "finalize-plan",
                "--id",
                id,
                "--input",
                context.path().to_str().unwrap(),
            ],
            &worktree_path,
        ),
        "bind future sync resource context",
    );
    let active_contract = worktree_path.join(format!(".ai/work-items/active/{id}.contract.json"));
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(active_contract).unwrap()).unwrap();
    assert_eq!(contract["resourceContext"]["branch"], sync_branch);
    seed_sync_projection_docs(&worktree_path, id);

    assert_success(
        &run(
            binary,
            &[
                "preflight",
                "--contract",
                &format!(".ai/work-items/active/{id}.contract.json"),
            ],
            &worktree_path,
        ),
        "preflight source PR head",
    );
    let initial_failure = run(
        binary,
        &["verify", "--work-item", id, "--command", "false"],
        &worktree_path,
    );
    assert!(
        !initial_failure.status.success(),
        "the source PR fixture should preserve one real failed verification attempt"
    );
    git(&worktree_path, &["add", "--all", ".ai"]);
    git(
        &worktree_path,
        &[
            "add",
            "-f",
            "--all",
            ".ai/work-items/active",
            ".ai/evidence",
        ],
    );
    commit_staged(&worktree_path, "checkpoint source Work Item contract");
    let source_head = git(&worktree_path, &["rev-parse", "HEAD"]);
    git(
        root,
        &[
            "merge",
            "--no-ff",
            "-q",
            source_branch,
            "-m",
            "merge source PR",
        ],
    );
    let source_merge = git(root, &["rev-parse", "HEAD"]);
    git(
        root,
        &["update-ref", "refs/remotes/origin/main", &source_merge],
    );
    git(&worktree_path, &["checkout", "-q", "-b", sync_branch]);
    git(root, &["branch", "-D", source_branch]);

    assert_success(
        &run(
            binary,
            &["verify", "--work-item", id, "--command", "true"],
            &worktree_path,
        ),
        "verify sync branch",
    );
    let evidence_dir = worktree_path.join(".ai/evidence");
    let attempts = fs::read_dir(&evidence_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&format!("{id}.verification-attempt.")))
                && !git(
                    &worktree_path,
                    &[
                        "status",
                        "--porcelain",
                        "--ignored=matching",
                        "--",
                        path.to_str().unwrap(),
                    ],
                )
                .is_empty()
        })
        .collect::<Vec<_>>();
    assert!(
        !attempts.is_empty(),
        "verify must persist immutable attempt artifacts"
    );
    for attempt in attempts {
        git(&worktree_path, &["add", attempt.to_str().unwrap()]);
        commit_staged(&worktree_path, "append immutable verification attempt");
    }
    assert_success(
        &run(binary, &["finish", "--id", id], &worktree_path),
        "finish sync branch",
    );
    git(&worktree_path, &["add", "--all"]);
    let finish_revision = commit_staged(&worktree_path, "finish verified Work Item");
    assert_success(
        &run(binary, &["archive", "--id", id], &worktree_path),
        "archive sync branch",
    );
    git(&worktree_path, &["add", "-u", ".ai/work-items/active"]);
    git(
        &worktree_path,
        &["add", "-f", "--all", ".ai/work-items/archive"],
    );
    let archive_revision = commit_staged(&worktree_path, "archive finished Work Item");
    promote_sync_projection_docs(&worktree_path, id);
    let docs_revision = git(&worktree_path, &["rev-parse", "HEAD"]);
    assert_ne!(finish_revision, archive_revision);
    assert_ne!(archive_revision, docs_revision);
    assert_ne!(finish_revision, docs_revision);
    git(
        root,
        &["merge", "--no-ff", "-q", sync_branch, "-m", "merge sync PR"],
    );
    let current_main = git(root, &["rev-parse", "HEAD"]);
    git(
        root,
        &["update-ref", "refs/remotes/origin/main", &current_main],
    );
    let source_base = base;
    let common_dir = root.join(".git").canonicalize().unwrap();
    let worktree_git_dir = git(&worktree_path, &["rev-parse", "--absolute-git-dir"]);
    let worktree_git_dir = Path::new(&worktree_git_dir).canonicalize().unwrap();
    let repository_id = run_json(binary, &["status"], root)["repositoryId"]
        .as_str()
        .unwrap()
        .to_owned();
    let archived_contract_path = root.join(format!(".ai/work-items/archive/{id}.contract.json"));
    let archived_contract_bytes = fs::read(&archived_contract_path).expect("archived Contract");
    let archived_contract: serde_json::Value =
        serde_json::from_slice(&archived_contract_bytes).unwrap();

    let source_pr = serde_json::json!({
        "number": 9001,
        "url": "https://github.com/example/fixture/pull/9001",
        "headRevision": source_head,
        "baseBranch": "main",
        "baseRemote": "origin",
        "baseRevision": source_base,
        "mergeCommit": source_merge
    });
    let sync_pr = serde_json::json!({
        "number": 9002,
        "url": "https://github.com/example/fixture/pull/9002",
        "headRevision": docs_revision,
        "baseBranch": "main",
        "baseRemote": "origin",
        "baseRevision": source_merge,
        "mergeCommit": current_main
    });
    let remote_ref = format!("refs/heads/{sync_branch}");
    let observations = [
        (
            "source",
            source_pr["url"].as_str().unwrap().to_owned(),
            "https://api.github.com/repos/example/fixture/pulls/9001".to_owned(),
            serde_json::to_vec(&serde_json::json!({
                "number": 9001,
                "html_url": source_pr["url"],
                "state": "closed",
                "merged": true,
                "merge_commit_sha": source_merge,
                "head": {"ref": source_branch, "sha": source_head, "repo": {"full_name": "example/fixture"}},
                "base": {"ref": "main", "sha": source_base, "repo": {"full_name": "example/fixture"}}
            }))
            .unwrap(),
        ),
        (
            "sync",
            sync_pr["url"].as_str().unwrap().to_owned(),
            "https://api.github.com/repos/example/fixture/pulls/9002".to_owned(),
            serde_json::to_vec(&serde_json::json!({
                "number": 9002,
                "html_url": sync_pr["url"],
                "state": "closed",
                "merged": true,
                "merge_commit_sha": current_main,
                "head": {"ref": sync_branch, "sha": docs_revision, "repo": {"full_name": "example/fixture"}},
                "base": {"ref": "main", "sha": source_merge, "repo": {"full_name": "example/fixture"}}
            }))
            .unwrap(),
        ),
        (
            "remote",
            remote_ref.clone(),
            format!("https://api.github.com/repos/example/fixture/git/ref/heads/{sync_branch}"),
            serde_json::to_vec(&serde_json::json!({
                "ref": remote_ref,
                "object": {"sha": docs_revision, "type": "commit"}
            }))
            .unwrap(),
        ),
    ];
    let mut evidence_digests = Vec::new();
    for (name, subject, origin, raw) in observations {
        let metadata = tempfile::NamedTempFile::new().expect("delegated metadata");
        let raw_file = tempfile::NamedTempFile::new().expect("delegated raw observation");
        let digest = sha256(&raw);
        let raw_ref = format!(".ai/evidence/external/wi9997-{name}.json");
        fs::write(
            metadata.path(),
            serde_json::to_vec_pretty(&serde_json::json!({
                "provider": "github",
                "subject": subject,
                "origin": origin,
                "assurance": "provider_verified",
                "collectedAt": "2026-01-01T00:00:00Z",
                "digest": digest,
                "validity": "valid",
                "rawEvidenceRef": raw_ref
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(raw_file.path(), &raw).unwrap();
        assert_success(
            &run(
                binary,
                &[
                    "evidence",
                    "import",
                    "--work-item",
                    id,
                    "--metadata",
                    metadata.path().to_str().unwrap(),
                    "--raw",
                    raw_file.path().to_str().unwrap(),
                ],
                root,
            ),
            "import provider observation",
        );
        evidence_digests.push(digest);
    }
    let status = run_json(binary, &["status"], root);
    let runtime_digest = runtime_digest(binary);
    let context = archived_contract["resourceContext"].clone();
    let retained = serde_json::json!({
        "schemaVersion": 2,
        "receiptId": "cli-retained-root",
        "operationId": "cli-retained-operation",
        "repositoryId": repository_id,
        "workItemId": id,
        "runtimeVersion": env!("CARGO_PKG_VERSION"),
        "runtimeDigest": runtime_digest,
        "provider": "github",
        "pullRequest": source_pr,
        "branch": {"name": sync_branch, "remote": "origin", "headRevision": docs_revision},
        "worktree": {"worktreeId": "wi9997-sync-worktree", "path": worktree_arg, "branch": sync_branch, "headRevision": docs_revision},
        "before": {"pullRequest": "merged", "branch": "present", "worktree": "clean"},
        "after": {"pullRequest": "merged", "branch": "present", "worktree": "clean"},
        "result": {"disposition": "retained", "failureCodes": [], "unknownCodes": []},
        "actor": "human:test",
        "authoritySource": "synthetic-cli-test",
        "reason": "record the retained resources before append-only cleanup",
        "timestamp": "2999-01-01T00:00:00Z",
        "contractDigest": sha256(&archived_contract_bytes),
        "contractBaseRevision": archived_contract["baseRevision"],
        "resourceContext": context,
        "postMergeSync": {
            "sourcePullRequest": source_pr,
            "syncPullRequest": sync_pr,
            "sourceEvidenceDigest": evidence_digests[0],
            "syncEvidenceDigest": evidence_digests[1],
            "finishRevision": finish_revision,
            "archiveRevision": archive_revision,
            "docsRevision": docs_revision,
            "currentMainRevision": current_main,
            "remoteBranchRef": remote_ref,
            "remoteBranchState": "present",
            "remoteEvidenceDigest": evidence_digests[2],
            "gitCommonDir": common_dir,
            "worktreeGitDir": worktree_git_dir
        }
    });
    assert_eq!(status["repositoryId"], repository_id);
    let root_file = tempfile::NamedTempFile::new().expect("Retained receipt");
    fs::write(
        root_file.path(),
        serde_json::to_vec_pretty(&retained).unwrap(),
    )
    .unwrap();
    assert_eq!(
        run_json(
            binary,
            &[
                "work-item",
                "finalize",
                "--id",
                id,
                "--input",
                root_file.path().to_str().unwrap()
            ],
            root,
        )["state"],
        "recorded"
    );

    git(
        root,
        &["worktree", "remove", "--force", worktree_arg.as_str()],
    );
    git(root, &["branch", "-D", sync_branch]);
    let deleted_raw = serde_json::to_vec(&serde_json::json!({
        "message": "Not Found",
        "documentation_url": "https://docs.github.com/rest/git/refs#get-a-reference",
        "status": "404"
    }))
    .unwrap();
    let metadata = tempfile::NamedTempFile::new().expect("deleted-ref metadata");
    let raw_file = tempfile::NamedTempFile::new().expect("deleted-ref observation");
    let deleted_digest = sha256(&deleted_raw);
    fs::write(
        metadata.path(),
        serde_json::to_vec_pretty(&serde_json::json!({
            "provider": "github",
            "subject": remote_ref,
            "origin": format!("https://api.github.com/repos/example/fixture/git/ref/heads/{sync_branch}"),
            "assurance": "provider_verified",
            "collectedAt": "2026-01-01T00:00:00Z",
            "digest": deleted_digest,
            "validity": "valid",
            "rawEvidenceRef": ".ai/evidence/external/wi9997-deleted-ref.json"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(raw_file.path(), &deleted_raw).unwrap();
    assert_success(
        &run(
            binary,
            &[
                "evidence",
                "import",
                "--work-item",
                id,
                "--metadata",
                metadata.path().to_str().unwrap(),
                "--raw",
                raw_file.path().to_str().unwrap(),
            ],
            root,
        ),
        "import deleted remote-ref observation",
    );
    let predecessor = cockpit_protocol::digest_json(&retained).unwrap();
    let mut deleted = retained;
    deleted["receiptId"] = "cli-deleted-transition".into();
    deleted["operationId"] = "cli-deleted-operation".into();
    deleted["before"] =
        serde_json::json!({"pullRequest": "merged", "branch": "present", "worktree": "clean"});
    deleted["after"] =
        serde_json::json!({"pullRequest": "merged", "branch": "deleted", "worktree": "removed"});
    deleted["result"] =
        serde_json::json!({"disposition": "deleted", "failureCodes": [], "unknownCodes": []});
    deleted["postMergeSync"]["remoteBranchState"] = "deleted".into();
    deleted["postMergeSync"]["remoteEvidenceDigest"] = deleted_digest.into();
    deleted["timestamp"] = "2999-01-01T00:01:00Z".into();
    let transition = serde_json::json!({
        "schemaVersion": 1,
        "transitionId": "cli-delete-transition",
        "sequence": 1,
        "predecessorReceiptDigest": predecessor,
        "receipt": deleted
    });
    let transition_file = tempfile::NamedTempFile::new().expect("Deleted transition");
    fs::write(
        transition_file.path(),
        serde_json::to_vec_pretty(&transition).unwrap(),
    )
    .unwrap();
    assert_eq!(
        run_json(
            binary,
            &[
                "work-item",
                "finalize",
                "--id",
                id,
                "--input",
                transition_file.path().to_str().unwrap()
            ],
            root,
        )["state"],
        "appended"
    );
    let verified = run_json(binary, &["work-item", "finalize-verify", "--id", id], root);
    assert_eq!(verified["state"], "verified");
    assert_eq!(verified["disposition"], "deleted");
    let close = run(
        binary,
        &[
            "close",
            "--id",
            id,
            "--human-decision",
            "approved",
            "--actor",
            "human:test",
            "--authority-source",
            "synthetic-cli-test",
            "--reason",
            "close after verified provider cleanup",
            "--evidence-ref",
            &format!(".ai/decisions/{id}.finalize.json"),
            "--decided-at",
            "2999-01-01T00:02:00Z",
        ],
        root,
    );
    assert_success(&close, "human close after Deleted finalization");
}

#[test]
fn runtime_close_requires_explicit_resource_finalization_receipt() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = repository();
    let root = repo.path();
    assert!(run(binary, &["attach"], root).status.success());
    assert!(
        run(
            binary,
            &[
                "start",
                "--id",
                "WI-FINALIZATION",
                "--intent",
                "bind resources",
                "--goal",
                "prevent stale branches",
                "--scope",
                "src/**",
                "--authority",
                "authorized",
            ],
            root,
        )
        .status
        .success()
    );
    let context_file = tempfile::NamedTempFile::new().expect("context temp file");
    let context_path = context_file.path().to_owned();
    let context_arg = context_path.to_string_lossy().into_owned();
    fs::write(
        &context_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "branch": "feature/finalization",
            "worktree": "/tmp/removed-feature-finalization",
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": "https://github.com/example/ai-cockpit/pull/159"
        }))
        .expect("context JSON"),
    )
    .expect("write context");
    let plan_output = run_json(
        binary,
        &[
            "work-item",
            "finalize-plan",
            "--id",
            "WI-FINALIZATION",
            "--input",
            context_arg.as_str(),
        ],
        root,
    );
    let contract_digest = plan_output["contractDigest"]
        .as_str()
        .expect("contract digest")
        .to_owned();
    assert_eq!(plan_output["state"], "planned");
    assert!(
        run(
            binary,
            &[
                "preflight",
                "--contract",
                ".ai/work-items/active/WI-FINALIZATION.contract.json",
            ],
            root,
        )
        .status
        .success()
    );
    let checkpoint = run(binary, &["checkpoint", "--id", "WI-FINALIZATION"], root);
    assert!(
        checkpoint.status.success(),
        "checkpoint stderr: {}",
        String::from_utf8_lossy(&checkpoint.stderr)
    );
    assert!(
        run(
            binary,
            &[
                "verify",
                "--work-item",
                "WI-FINALIZATION",
                "--command",
                "true"
            ],
            root,
        )
        .status
        .success()
    );
    assert!(
        run(binary, &["finish", "--id", "WI-FINALIZATION"], root)
            .status
            .success()
    );

    let archived = run(binary, &["archive", "--id", "WI-FINALIZATION"], root);
    assert!(
        archived.status.success(),
        "archive stderr: {}",
        String::from_utf8_lossy(&archived.stderr)
    );
    let missing_receipt = run(
        binary,
        &[
            "close",
            "--id",
            "WI-FINALIZATION",
            "--human-decision",
            "approved",
        ],
        root,
    );
    assert!(
        !missing_receipt.status.success(),
        "close must fail before resource finalization"
    );
    let pending_outcome = run(
        binary,
        &["work-item", "outcome", "--id", "WI-FINALIZATION"],
        root,
    );
    assert_success(&pending_outcome, "pending outcome");
    let pending_text = String::from_utf8(pending_outcome.stdout).expect("pending outcome UTF-8");
    assert!(
        pending_text.contains("re-observe") || pending_text.contains("Re-observe"),
        "pending finalization must request inspection/recovery: {pending_text}"
    );
    assert!(
        !pending_text.contains("delete the exact") && !pending_text.contains("Delete the exact"),
        "missing receipt must not repeat a deletion action: {pending_text}"
    );
    let status = run_json(binary, &["status"], root);
    let repository_id = status["repositoryId"].as_str().expect("repository id");
    let archived_contract: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(".ai/work-items/archive/WI-FINALIZATION.contract.json"))
            .expect("archived contract"),
    )
    .expect("archived contract JSON");
    let receipt_file = tempfile::NamedTempFile::new().expect("receipt temp file");
    let receipt_path = receipt_file.path().to_owned();
    let receipt_arg = receipt_path.to_string_lossy().into_owned();
    let receipt = serde_json::json!({
        "schemaVersion": 1,
        "receiptId": "receipt-finalization",
        "operationId": "operation-finalization",
        "repositoryId": repository_id,
        "workItemId": "WI-FINALIZATION",
        "runtimeVersion": env!("CARGO_PKG_VERSION"),
        "runtimeDigest": runtime_digest(binary),
        "provider": "github",
        "pullRequest": {
            "number": 159,
            "url": "https://github.com/example/ai-cockpit/pull/159",
            "headRevision": "abcdef1",
            "baseBranch": "main",
            "baseRemote": "origin",
            "baseRevision": archived_contract["baseRevision"],
            "mergeCommit": "1234567"
        },
        "branch": {
            "name": "feature/finalization",
            "remote": "origin",
            "headRevision": "abcdef1"
        },
        "worktree": {
            "worktreeId": "removed-feature-finalization",
            "path": "/tmp/removed-feature-finalization",
            "branch": "feature/finalization",
            "headRevision": "abcdef1"
        },
        "before": {
            "pullRequest": "merged",
            "branch": "present",
            "worktree": "clean"
        },
        "after": {
            "pullRequest": "merged",
            "branch": "deleted",
            "worktree": "removed"
        },
        "result": {
            "disposition": "deleted",
            "failureCodes": [],
            "unknownCodes": []
        },
        "actor": "human:test",
        "authoritySource": "test-policy",
        "reason": "provider cleanup receipt",
        "timestamp": "2026-08-23T00:00:00Z",
        "contractDigest": contract_digest,
        "resourceContext": {
            "branch": "feature/finalization",
            "worktree": "/tmp/removed-feature-finalization",
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": "https://github.com/example/ai-cockpit/pull/159"
        }
    });
    fs::write(
        &receipt_path,
        serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
    )
    .expect("write receipt");
    let finalized = run_json(
        binary,
        &[
            "work-item",
            "finalize",
            "--id",
            "WI-FINALIZATION",
            "--input",
            receipt_arg.as_str(),
        ],
        root,
    );
    assert_eq!(finalized["state"], "recorded");
    assert_eq!(finalized["disposition"], "deleted");
    let verified = run_json(
        binary,
        &["work-item", "finalize-verify", "--id", "WI-FINALIZATION"],
        root,
    );
    assert_eq!(verified["state"], "verified");
    assert_eq!(verified["disposition"], "deleted");

    let closed = run(
        binary,
        &[
            "close",
            "--id",
            "WI-FINALIZATION",
            "--human-decision",
            "approved",
            "--actor",
            "human:test",
            "--authority-source",
            "test-policy",
            "--reason",
            "resource receipt reviewed",
            "--decided-at",
            "2026-08-23T00:00:00Z",
        ],
        root,
    );
    assert!(
        closed.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&closed.stderr)
    );
}

#[test]
fn runtime_accepts_explicitly_abandoned_unmerged_cleanup() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = repository();
    let root = repo.path();
    let id = "WI-ABANDONED-FINALIZATION";
    assert_success(&run(binary, &["attach"], root), "attach");
    assert_success(
        &run(
            binary,
            &[
                "start",
                "--id",
                id,
                "--intent",
                "record failed delivery cleanup",
                "--goal",
                "preserve truthful non-merge outcome",
                "--scope",
                "src/**",
                "--authority",
                "authorized",
            ],
            root,
        ),
        "start",
    );
    let context_file = tempfile::NamedTempFile::new().expect("context temp file");
    let context_path = context_file.path().to_owned();
    fs::write(
        &context_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "branch": "feature/abandoned-finalization",
            "worktree": "/tmp/removed-abandoned-finalization",
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": "https://github.com/example/ai-cockpit/pull/160"
        }))
        .expect("context JSON"),
    )
    .expect("write context");
    let plan = run_json(
        binary,
        &[
            "work-item",
            "finalize-plan",
            "--id",
            id,
            "--input",
            context_path.to_string_lossy().as_ref(),
        ],
        root,
    );
    let contract_digest = plan["contractDigest"].as_str().unwrap().to_owned();
    assert_success(
        &run(
            binary,
            &[
                "preflight",
                "--contract",
                &format!(".ai/work-items/active/{id}.contract.json"),
            ],
            root,
        ),
        "preflight",
    );
    assert_success(
        &run(binary, &["checkpoint", "--id", id], root),
        "checkpoint",
    );
    assert_success(
        &run(
            binary,
            &["verify", "--work-item", id, "--command", "true"],
            root,
        ),
        "verify",
    );
    assert_success(&run(binary, &["finish", "--id", id], root), "finish");
    assert_success(&run(binary, &["archive", "--id", id], root), "archive");

    let repository_id = run_json(binary, &["status"], root)["repositoryId"]
        .as_str()
        .unwrap()
        .to_owned();
    let archived_contract: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(format!(".ai/work-items/archive/{id}.contract.json")))
            .expect("archived contract"),
    )
    .expect("archived contract JSON");
    let receipt_file = tempfile::NamedTempFile::new().expect("receipt temp file");
    let receipt = serde_json::json!({
        "schemaVersion": 1,
        "receiptId": "receipt-abandoned-finalization",
        "operationId": "operation-abandoned-finalization",
        "repositoryId": repository_id,
        "workItemId": id,
        "runtimeVersion": env!("CARGO_PKG_VERSION"),
        "runtimeDigest": runtime_digest(binary),
        "provider": "github",
        "pullRequest": {
            "number": 160,
            "url": "https://github.com/example/ai-cockpit/pull/160",
            "headRevision": "abcdef2",
            "baseBranch": "main",
            "baseRemote": "origin",
            "baseRevision": archived_contract["baseRevision"],
            "mergeCommit": null
        },
        "branch": {
            "name": "feature/abandoned-finalization",
            "remote": "origin",
            "headRevision": "abcdef2"
        },
        "worktree": {
            "worktreeId": "removed-abandoned-finalization",
            "path": "/tmp/removed-abandoned-finalization",
            "branch": "feature/abandoned-finalization",
            "headRevision": "abcdef2"
        },
        "before": {
            "pullRequest": "unmerged",
            "branch": "present",
            "worktree": "clean"
        },
        "after": {
            "pullRequest": "unmerged",
            "branch": "deleted",
            "worktree": "removed"
        },
        "result": {
            "disposition": "abandoned",
            "failureCodes": ["unmerged_pull_request"],
            "unknownCodes": []
        },
        "actor": "human:test",
        "authoritySource": "test-policy",
        "reason": "reviewed failed delivery was explicitly closed without merge",
        "timestamp": "2026-08-23T00:00:00Z",
        "contractDigest": contract_digest,
        "resourceContext": {
            "branch": "feature/abandoned-finalization",
            "worktree": "/tmp/removed-abandoned-finalization",
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": "https://github.com/example/ai-cockpit/pull/160"
        }
    });
    fs::write(
        receipt_file.path(),
        serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
    )
    .expect("write receipt");
    let finalized = run_json(
        binary,
        &[
            "work-item",
            "finalize",
            "--id",
            id,
            "--input",
            receipt_file.path().to_string_lossy().as_ref(),
        ],
        root,
    );
    assert_eq!(finalized["state"], "recorded");
    assert_eq!(finalized["disposition"], "abandoned");

    let verified = run_json(binary, &["work-item", "finalize-verify", "--id", id], root);
    assert_eq!(verified["state"], "verified");
    assert_eq!(verified["disposition"], "abandoned");

    let closed = run(
        binary,
        &[
            "close",
            "--id",
            id,
            "--human-decision",
            "superseded_failed_delivery",
            "--actor",
            "human:test",
            "--authority-source",
            "test-policy",
            "--reason",
            "reviewed failed delivery was cleaned without merge",
            "--decided-at",
            "2026-08-23T00:00:00Z",
        ],
        root,
    );
    assert_success(&closed, "close abandoned finalization");
}

#[test]
fn cli_appends_governance_bound_merge_observation_and_cleanup() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = repository();
    let root = repo.path();
    let id = "WI-FINALIZATION-APPEND";
    assert_success(&run(binary, &["attach"], root), "attach");
    assert_success(
        &run(
            binary,
            &[
                "start",
                "--id",
                id,
                "--intent",
                "bind governance receipt append",
                "--goal",
                "preserve finalization history",
                "--scope",
                ".ai/**",
                "--authority",
                "authorized",
            ],
            root,
        ),
        "start",
    );
    let context_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(
        context_file.path(),
        serde_json::to_vec_pretty(&serde_json::json!({
            "branch": "feature/finalization-append",
            "worktree": "/tmp/removed-finalization-append",
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": "https://github.com/example/ai-cockpit/pull/191"
        }))
        .unwrap(),
    )
    .unwrap();
    let context_arg = context_file.path().to_string_lossy().into_owned();
    let plan = run_json(
        binary,
        &[
            "work-item",
            "finalize-plan",
            "--id",
            id,
            "--input",
            &context_arg,
        ],
        root,
    );
    let contract_digest = plan["contractDigest"].as_str().unwrap().to_string();
    assert_success(
        &run(
            binary,
            &[
                "preflight",
                "--contract",
                &format!(".ai/work-items/active/{id}.contract.json"),
            ],
            root,
        ),
        "preflight",
    );
    assert_success(
        &run(binary, &["checkpoint", "--id", id], root),
        "checkpoint",
    );
    assert_success(
        &run(
            binary,
            &["verify", "--work-item", id, "--command", "true"],
            root,
        ),
        "verify",
    );
    assert_success(&run(binary, &["finish", "--id", id], root), "finish");
    assert_success(&run(binary, &["archive", "--id", id], root), "archive");

    for args in [
        ["config", "user.email", "tests@example.invalid"],
        ["config", "user.name", "AI Cockpit Tests"],
    ] {
        assert_success(
            &Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .unwrap(),
            "git config",
        );
    }
    assert_success(
        &Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap(),
        "git add archive",
    );
    assert_success(
        &Command::new("git")
            .args(["commit", "-q", "-m", "archive"])
            .current_dir(root)
            .output()
            .unwrap(),
        "git commit archive",
    );
    let archive_head = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    let repository_id = run_json(binary, &["status"], root)["repositoryId"]
        .as_str()
        .unwrap()
        .to_string();
    let archived_contract: serde_json::Value = serde_json::from_slice(
        &fs::read(
            root.join(".ai/work-items/archive")
                .join(format!("{id}.contract.json")),
        )
        .expect("archived contract"),
    )
    .expect("archived contract JSON");
    let mut blocked = serde_json::json!({
        "schemaVersion":1,"receiptId":"blocked-1","operationId":"operation-1",
        "repositoryId":repository_id,"workItemId":id,
        "runtimeVersion":env!("CARGO_PKG_VERSION"),"runtimeDigest":runtime_digest(binary),
        "provider":"github",
        "pullRequest":{"number":191,"url":"https://github.com/example/ai-cockpit/pull/191","headRevision":archive_head,"baseBranch":"main","baseRemote":"origin","baseRevision":archived_contract["baseRevision"]},
        "branch":{"name":"feature/finalization-append","remote":"origin","headRevision":archive_head},
        "worktree":{"worktreeId":"removed-finalization-append","path":"/tmp/removed-finalization-append","branch":"feature/finalization-append","headRevision":archive_head},
        "before":{"pullRequest":"unmerged","branch":"present","worktree":"clean"},
        "after":{"pullRequest":"unmerged","branch":"present","worktree":"clean"},
        "result":{"disposition":"blocked","failureCodes":["unmerged_pull_request"],"unknownCodes":[]},
        "actor":"human:test","authoritySource":"test-policy","reason":"await merge","timestamp":"2026-08-23T00:00:00Z",
        "contractDigest":contract_digest,
        "resourceContext":{"branch":"feature/finalization-append","worktree":"/tmp/removed-finalization-append","baseBranch":"main","baseRemote":"origin","provider":"github","pullRequest":"https://github.com/example/ai-cockpit/pull/191"}
    });
    let blocked_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(
        blocked_file.path(),
        serde_json::to_vec_pretty(&blocked).unwrap(),
    )
    .unwrap();
    let blocked_arg = blocked_file.path().to_string_lossy().into_owned();
    assert_eq!(
        run_json(
            binary,
            &["work-item", "finalize", "--id", id, "--input", &blocked_arg],
            root,
        )["state"],
        "recorded"
    );
    let canonical = format!(".ai/decisions/{id}.finalize.json");
    assert_success(
        &Command::new("git")
            .args(["add", &canonical])
            .current_dir(root)
            .output()
            .unwrap(),
        "git add receipt",
    );
    assert_success(
        &Command::new("git")
            .args(["commit", "-q", "-m", "append receipt"])
            .current_dir(root)
            .output()
            .unwrap(),
        "git commit receipt",
    );
    let append_head = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    let predecessor = cockpit_protocol::digest_json(&blocked).unwrap();
    blocked["receiptId"] = "observed-2".into();
    blocked["operationId"] = "operation-2".into();
    blocked["pullRequest"]["headRevision"] = append_head.clone().into();
    blocked["pullRequest"]["mergeCommit"] = "merge-191".into();
    blocked["branch"]["headRevision"] = append_head.clone().into();
    blocked["worktree"]["headRevision"] = append_head.clone().into();
    blocked["before"] = blocked["after"].clone();
    blocked["after"]["pullRequest"] = "merged".into();
    blocked["result"] =
        serde_json::json!({"disposition":"retained","failureCodes":[],"unknownCodes":[]});
    let observed = serde_json::json!({
        "schemaVersion":1,"transitionId":"transition-1","sequence":1,
        "predecessorReceiptDigest":predecessor,
        "governanceAppendRevision":append_head,
        "receipt":blocked
    });
    let observed_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(
        observed_file.path(),
        serde_json::to_vec_pretty(&observed).unwrap(),
    )
    .unwrap();
    let observed_arg = observed_file.path().to_string_lossy().into_owned();
    let appended = run_json(
        binary,
        &[
            "work-item",
            "finalize",
            "--id",
            id,
            "--input",
            &observed_arg,
        ],
        root,
    );
    assert_eq!(appended["state"], "appended");
    assert_eq!(appended["sequence"], 1);
    let verified = run_json(binary, &["work-item", "finalize-verify", "--id", id], root);
    assert_eq!(verified["sequence"], 1);
    assert_eq!(verified["disposition"], "retained");
}
