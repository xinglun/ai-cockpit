use cockpit_core::Digest;
use cockpit_protocol::PROTOCOL_VERSION;
use cockpit_protocol::RuntimeContext;
use cockpit_repository::attach;
use std::fs;
use std::path::Path;
use std::process::Command;
const WORK_ITEM_ID: &str = "WI-REPOSITORY-CROSS-CHECKOUT-CLOSEOUT";

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "1.0.0-repository-closeout-test".into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"repository-closeout-recovery-test-runtime"),
    }
}

fn command(root: &Path, program: &str, args: &[&str]) -> std::process::Output {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .expect("run command");
    assert!(
        output.status.success(),
        "{program} {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn git(root: &Path, args: &[&str]) {
    command(root, "git", args);
}

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("source repository");
    git(directory.path(), &["init", "--quiet"]);
    attach(directory.path()).expect("attach source repository");
    git(
        directory.path(),
        &["config", "user.name", "Repository Closeout Test"],
    );
    git(
        directory.path(),
        &[
            "config",
            "user.email",
            "repository-closeout-test@example.invalid",
        ],
    );
    fs::write(directory.path().join("seed.txt"), "baseline\n").expect("write baseline");
    git(directory.path(), &["add", "-A"]);
    git(
        directory.path(),
        &["commit", "--quiet", "-m", "fixture baseline"],
    );
    directory
}

#[test]
fn cross_checkout_public_factory_requires_independent_protected_approval() {
    let directory = repository();
    let root = directory.path();
    let project = root.join(".ai/project.json");
    let before = fs::read(&project).unwrap();
    let error =
        cockpit_repository::recover_cross_checkout_closeout(root, root, WORK_ITEM_ID, &runtime())
            .unwrap_err();
    assert!(
        error.to_string().contains("authority_unavailable"),
        "{error}"
    );
    assert_eq!(fs::read(project).unwrap(), before);
    assert!(
        !root
            .join(format!(
                ".ai/decisions/{WORK_ITEM_ID}.cross-checkout-closeout-recovery.json"
            ))
            .exists()
    );
}
