use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use cockpit_repository::{attach, scaffold_work_item};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn repository(name: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "cockpit-repository-context-{name}-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .expect("git init")
            .success()
    );
    root
}

#[test]
fn parallel_repository_contexts_do_not_share_scaffold_state() {
    let left = repository("left");
    let right = repository("right");
    fs::write(left.join("left.txt"), "left\n").expect("left fact");
    fs::write(right.join("right.txt"), "right\n").expect("right fact");
    std::thread::scope(|scope| {
        let left_handle = scope.spawn(|| {
            let profile = attach(&left).expect("attach left");
            let scaffold = scaffold_work_item(&left, "WI-LEFT", "code").expect("scaffold left");
            (profile, scaffold)
        });
        let right_handle = scope.spawn(|| {
            let profile = attach(&right).expect("attach right");
            let scaffold = scaffold_work_item(&right, "WI-RIGHT", "docs").expect("scaffold right");
            (profile, scaffold)
        });
        let (left_profile, left_scaffold) = left_handle.join().expect("left thread");
        let (right_profile, right_scaffold) = right_handle.join().expect("right thread");
        assert_ne!(left_profile.repository_id, right_profile.repository_id);
        assert_eq!(
            left_scaffold.known_facts.repository_id,
            left_profile.repository_id
        );
        assert_eq!(
            right_scaffold.known_facts.repository_id,
            right_profile.repository_id
        );
        assert_ne!(
            left_scaffold.known_facts.repository_snapshot_digest,
            right_scaffold.known_facts.repository_snapshot_digest
        );
        assert!(
            left.join(".ai/work-items/active/WI-LEFT.contract.json")
                .is_file()
        );
        assert!(
            !left
                .join(".ai/work-items/active/WI-RIGHT.contract.json")
                .exists()
        );
        assert!(
            right
                .join(".ai/work-items/active/WI-RIGHT.contract.json")
                .is_file()
        );
        assert!(
            !right
                .join(".ai/work-items/active/WI-LEFT.contract.json")
                .exists()
        );
    });
    fs::remove_dir_all(left).expect("cleanup left");
    fs::remove_dir_all(right).expect("cleanup right");
}
