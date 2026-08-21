use cockpit_evidence::{DiffIdentity, EvidenceContext, ReusableReceipt};
use cockpit_repository::{
    ReceiptStoreBinding, ReceiptStoreLoad, load_reusable_receipt, persist_reusable_receipt,
    repository_id,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn digest(fill: char) -> String {
    format!("sha256:{}", fill.to_string().repeat(64))
}

fn fixture(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "cockpit-receipt-store-{name}-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(root.join(".ai")).expect("fixture");
    root
}

fn context(profile_digest: &str) -> EvidenceContext {
    EvidenceContext {
        content_digest: digest('1'),
        diff: DiffIdentity {
            base_commit: "2".repeat(40),
            head_commit: "3".repeat(40),
            changed_paths_digest: digest('4'),
        },
        environment_digest: digest('5'),
        command_digest: digest('6'),
        scope_digest: digest('7'),
        governance_digest: digest('8'),
        toolchain_digest: digest('9'),
        policy_digest: digest('a'),
        profile_digest: profile_digest.into(),
        stage: "task".into(),
        runner: "local".into(),
    }
}

fn binding(root: &Path) -> ReceiptStoreBinding {
    let canonical_root = fs::canonicalize(root).expect("canonical root");
    ReceiptStoreBinding {
        repository_id: repository_id(&canonical_root).to_string(),
        profile_digest: digest('b'),
        node_id: "profile-test".into(),
    }
}

fn receipt(binding: &ReceiptStoreBinding) -> ReusableReceipt {
    ReusableReceipt::new(
        &binding.node_id,
        true,
        context(&binding.profile_digest),
        &digest('c'),
        100,
        200,
    )
    .expect("receipt")
}

fn unavailable_reason(load: ReceiptStoreLoad) -> String {
    match load {
        ReceiptStoreLoad::Unavailable { reason, .. } => reason,
        ReceiptStoreLoad::Candidate { .. } => panic!("unexpected candidate"),
    }
}

#[test]
fn empty_store_is_unavailable_and_valid_store_loads_only_the_indexed_receipt() {
    let root = fixture("roundtrip");
    let binding = binding(&root);
    assert_eq!(
        unavailable_reason(load_reusable_receipt(&root, &binding).expect("empty load")),
        "evidence_missing"
    );

    let receipt = receipt(&binding);
    persist_reusable_receipt(&root, &binding, &receipt).expect("persist");
    let load = load_reusable_receipt(&root, &binding).expect("load");
    match load {
        ReceiptStoreLoad::Candidate {
            receipt: loaded,
            files_read,
        } => {
            assert_eq!(*loaded, receipt);
            assert_eq!(files_read, 2);
        }
        ReceiptStoreLoad::Unavailable { reason, .. } => panic!("unavailable: {reason}"),
    }
    let receipt_names = fs::read_dir(root.join(".ai/evidence/reuse/receipts"))
        .expect("receipt directory")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    assert!(receipt_names.iter().all(|name| !name.contains(':')));
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn malformed_tampered_and_missing_referenced_receipts_fail_closed() {
    for (name, mutation, expected) in [
        ("malformed", "malformed", "index_invalid"),
        ("missing", "missing", "receipt_missing"),
        ("tampered", "tampered", "receipt_invalid"),
    ] {
        let root = fixture(name);
        let binding = binding(&root);
        let receipt = receipt(&binding);
        persist_reusable_receipt(&root, &binding, &receipt).expect("persist");
        let reuse = root.join(".ai/evidence/reuse");
        if mutation == "malformed" {
            fs::write(reuse.join("index.json"), b"{").expect("malform index");
        } else {
            let receipt_path = reuse
                .join("receipts")
                .join(format!("{}.json", &receipt.receipt_id["sha256:".len()..]));
            if mutation == "missing" {
                fs::remove_file(receipt_path).expect("remove receipt");
            } else {
                let mut value: serde_json::Value =
                    serde_json::from_slice(&fs::read(&receipt_path).expect("read receipt"))
                        .expect("JSON");
                value["outputDigest"] = serde_json::json!(digest('d'));
                fs::write(receipt_path, serde_json::to_vec(&value).expect("JSON"))
                    .expect("tamper receipt");
            }
        }
        assert_eq!(
            unavailable_reason(load_reusable_receipt(&root, &binding).expect("load")),
            expected
        );
        fs::remove_dir_all(root).expect("cleanup");
    }
}

#[test]
fn wrong_repository_profile_or_index_binding_never_returns_a_candidate() {
    let root = fixture("binding");
    let binding = binding(&root);
    persist_reusable_receipt(&root, &binding, &receipt(&binding)).expect("persist");

    let mut wrong_repository = binding.clone();
    wrong_repository.repository_id = digest('e');
    assert_eq!(
        unavailable_reason(load_reusable_receipt(&root, &wrong_repository).expect("load")),
        "repository_identity_mismatch"
    );
    let mut wrong_profile = binding.clone();
    wrong_profile.profile_digest = digest('f');
    assert_eq!(
        unavailable_reason(load_reusable_receipt(&root, &wrong_profile).expect("load")),
        "profile_identity_mismatch"
    );

    let index_path = root.join(".ai/evidence/reuse/index.json");
    let mut index: serde_json::Value =
        serde_json::from_slice(&fs::read(&index_path).expect("index")).expect("JSON");
    index["receipts"]["profile-test"] = serde_json::json!(digest('0'));
    fs::write(index_path, serde_json::to_vec(&index).expect("JSON")).expect("tamper index");
    assert_eq!(
        unavailable_reason(load_reusable_receipt(&root, &binding).expect("load")),
        "receipt_missing"
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn symlinked_index_or_receipt_is_rejected() {
    use std::os::unix::fs::symlink;

    let root = fixture("symlink");
    let binding = binding(&root);
    let receipt = receipt(&binding);
    persist_reusable_receipt(&root, &binding, &receipt).expect("persist");
    let reuse = root.join(".ai/evidence/reuse");
    let index = reuse.join("index.json");
    let real_index = reuse.join("real-index.json");
    fs::rename(&index, &real_index).expect("move index");
    symlink(&real_index, &index).expect("link index");
    assert_eq!(
        unavailable_reason(load_reusable_receipt(&root, &binding).expect("load")),
        "symlink_rejected"
    );

    fs::remove_file(&index).expect("unlink index");
    fs::rename(&real_index, &index).expect("restore index");
    let receipt_path = reuse
        .join("receipts")
        .join(format!("{}.json", &receipt.receipt_id["sha256:".len()..]));
    let real_receipt = reuse.join("real-receipt.json");
    fs::rename(&receipt_path, &real_receipt).expect("move receipt");
    symlink(&real_receipt, &receipt_path).expect("link receipt");
    assert_eq!(
        unavailable_reason(load_reusable_receipt(&root, &binding).expect("load")),
        "symlink_rejected"
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn symlinked_store_parent_directory_is_rejected_before_any_index_read() {
    use std::os::unix::fs::symlink;

    let root = fixture("symlink-parent");
    let binding = binding(&root);
    let outside = fixture("symlink-parent-target");
    fs::create_dir_all(root.join(".ai/evidence")).expect("evidence directory");
    symlink(&outside, root.join(".ai/evidence/reuse")).expect("link reuse parent");

    assert_eq!(
        unavailable_reason(load_reusable_receipt(&root, &binding).expect("load")),
        "symlink_rejected"
    );
    fs::remove_dir_all(root).expect("cleanup root");
    fs::remove_dir_all(outside).expect("cleanup outside");
}

#[cfg(unix)]
#[test]
fn parent_replacement_after_lock_open_cannot_redirect_store_reads() {
    use std::{os::unix::fs::symlink, sync::mpsc, time::Duration};

    let root = fixture("parent-replacement");
    let binding = binding(&root);
    let expected = receipt(&binding);
    persist_reusable_receipt(&root, &binding, &expected).expect("seed store");

    let reuse = root.join(".ai/evidence/reuse");
    let original = root.join(".ai/evidence/reuse-original");
    let attacker = fixture("parent-replacement-attacker");
    fs::create_dir_all(attacker.join("reuse/receipts")).expect("attacker store");
    fs::write(attacker.join("reuse/index.json"), b"{").expect("invalid attacker index");
    fs::write(attacker.join("reuse/index.lock"), b"").expect("attacker lock");

    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(reuse.join("index.lock"))
        .expect("open lock");
    lock.lock().expect("exclusive lock");

    let (started_tx, started_rx) = mpsc::channel();
    let worker_root = root.clone();
    let worker_binding = binding.clone();
    let worker = std::thread::spawn(move || {
        started_tx.send(()).expect("signal");
        load_reusable_receipt(&worker_root, &worker_binding)
    });
    started_rx.recv().expect("started");
    std::thread::sleep(Duration::from_millis(100));

    fs::rename(&reuse, &original).expect("move original store");
    symlink(attacker.join("reuse"), &reuse).expect("redirect store path");
    lock.unlock().expect("unlock");

    assert!(matches!(
        worker.join().expect("reader").expect("load"),
        ReceiptStoreLoad::Candidate { receipt, .. } if *receipt == expected
    ));

    fs::remove_file(&reuse).expect("unlink redirected store");
    fs::remove_dir_all(root).expect("cleanup root");
    fs::remove_dir_all(attacker).expect("cleanup attacker");
}

#[cfg(windows)]
#[test]
fn parent_replacement_after_lock_open_cannot_redirect_store_writes() {
    use std::{sync::mpsc, time::Duration};

    let root = fixture("windows-parent-replacement");
    let first_binding = binding(&root);
    let first = receipt(&first_binding);
    persist_reusable_receipt(&root, &first_binding, &first).expect("seed store");
    let mut second_binding = first_binding.clone();
    second_binding.node_id = "second-node".into();
    let second = receipt(&second_binding);
    let reuse = root.join(".ai/evidence/reuse");
    let original = root.join(".ai/evidence/reuse-original");
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(reuse.join("index.lock"))
        .expect("open lock");
    lock.lock().expect("exclusive lock");
    let (started_tx, started_rx) = mpsc::channel();
    let worker_root = root.clone();
    let worker_binding = second_binding.clone();
    let worker_receipt = second.clone();
    let worker = std::thread::spawn(move || {
        started_tx.send(()).expect("signal");
        persist_reusable_receipt(&worker_root, &worker_binding, &worker_receipt)
    });
    started_rx.recv().expect("started");
    std::thread::sleep(Duration::from_millis(100));
    fs::rename(&reuse, &original).expect("move original store");
    fs::create_dir_all(reuse.join("receipts")).expect("replacement store");
    fs::write(reuse.join("index.lock"), b"").expect("replacement lock");
    fs::write(reuse.join("sentinel"), b"untouched").expect("sentinel");
    lock.unlock().expect("unlock");

    worker
        .join()
        .expect("writer")
        .expect("persist through handle");
    assert_eq!(
        fs::read(reuse.join("sentinel")).expect("sentinel"),
        b"untouched"
    );
    assert!(!reuse.join("index.json").exists());
    fs::remove_dir_all(&reuse).expect("remove replacement");
    fs::rename(&original, &reuse).expect("restore original store");
    assert!(matches!(
        load_reusable_receipt(&root, &second_binding).expect("load second"),
        ReceiptStoreLoad::Candidate { receipt, .. } if *receipt == second
    ));
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn failed_index_replacement_keeps_the_previous_valid_index_authoritative() {
    use std::os::unix::fs::PermissionsExt;

    let root = fixture("atomic-failure");
    let first_binding = binding(&root);
    let first_receipt = receipt(&first_binding);
    persist_reusable_receipt(&root, &first_binding, &first_receipt).expect("persist first");

    let mut second_binding = first_binding.clone();
    second_binding.node_id = "second-node".into();
    let second_receipt = receipt(&second_binding);
    let reuse = root.join(".ai/evidence/reuse");
    let original_permissions = fs::metadata(&reuse).expect("metadata").permissions();
    fs::set_permissions(&reuse, fs::Permissions::from_mode(0o555)).expect("make read only");
    let failed = persist_reusable_receipt(&root, &second_binding, &second_receipt);
    fs::set_permissions(&reuse, original_permissions).expect("restore permissions");
    assert!(
        failed.is_err(),
        "index replacement must fail under read-only parent"
    );

    assert!(matches!(
        load_reusable_receipt(&root, &first_binding).expect("load first"),
        ReceiptStoreLoad::Candidate { receipt, .. } if *receipt == first_receipt
    ));
    assert_eq!(
        unavailable_reason(load_reusable_receipt(&root, &second_binding).expect("load second")),
        "evidence_missing"
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn concurrent_writers_preserve_both_node_mappings() {
    use std::sync::{Arc, Barrier};

    let root = fixture("concurrent");
    let mut first_binding = binding(&root);
    first_binding.node_id = "first-node".into();
    let mut second_binding = first_binding.clone();
    second_binding.node_id = "second-node".into();
    let first_receipt = receipt(&first_binding);
    let second_receipt = receipt(&second_binding);
    let barrier = Arc::new(Barrier::new(2));
    let workers = [
        (first_binding.clone(), first_receipt.clone()),
        (second_binding.clone(), second_receipt.clone()),
    ]
    .into_iter()
    .map(|(binding, receipt)| {
        let root = root.clone();
        let barrier = Arc::clone(&barrier);
        std::thread::spawn(move || {
            barrier.wait();
            persist_reusable_receipt(&root, &binding, &receipt)
        })
    })
    .collect::<Vec<_>>();
    for worker in workers {
        worker.join().expect("writer thread").expect("persist");
    }

    assert!(matches!(
        load_reusable_receipt(&root, &first_binding).expect("first"),
        ReceiptStoreLoad::Candidate { receipt, .. } if *receipt == first_receipt
    ));
    assert!(matches!(
        load_reusable_receipt(&root, &second_binding).expect("second"),
        ReceiptStoreLoad::Candidate { receipt, .. } if *receipt == second_receipt
    ));
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn orphaned_temporary_receipt_does_not_poison_later_publication() {
    let root = fixture("orphan-temp");
    let first_binding = binding(&root);
    persist_reusable_receipt(&root, &first_binding, &receipt(&first_binding)).expect("seed store");
    fs::write(
        root.join(".ai/evidence/reuse/receipts/interrupted.receipt-tmp"),
        b"partial",
    )
    .expect("orphan temp");
    let mut second_binding = first_binding.clone();
    second_binding.node_id = "after-interruption".into();
    let second_receipt = receipt(&second_binding);

    persist_reusable_receipt(&root, &second_binding, &second_receipt)
        .expect("publish after interruption");

    assert!(matches!(
        load_reusable_receipt(&root, &second_binding).expect("load"),
        ReceiptStoreLoad::Candidate { receipt, .. } if *receipt == second_receipt
    ));
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn uncertain_index_commit_marker_fails_closed() {
    let root = fixture("uncertain-index-commit");
    let binding = binding(&root);
    let expected = receipt(&binding);
    persist_reusable_receipt(&root, &binding, &expected).expect("persist");
    fs::write(root.join(".ai/evidence/reuse/index.pending"), b"pending")
        .expect("simulate post-rename durability failure");

    assert_eq!(
        unavailable_reason(load_reusable_receipt(&root, &binding).expect("load")),
        "index_commit_uncertain"
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn existing_hardlinked_commit_marker_is_never_truncated() {
    let root = fixture("hardlinked-marker");
    let first_binding = binding(&root);
    let first = receipt(&first_binding);
    persist_reusable_receipt(&root, &first_binding, &first).expect("seed store");
    let victim = root.join("must-not-change.txt");
    fs::write(&victim, b"protected-content").expect("victim");
    fs::hard_link(&victim, root.join(".ai/evidence/reuse/index.pending")).expect("hardlink marker");
    let mut second_binding = first_binding.clone();
    second_binding.node_id = "second-node".into();
    let second = receipt(&second_binding);

    assert!(persist_reusable_receipt(&root, &second_binding, &second).is_err());
    assert_eq!(
        fs::read(&victim).expect("victim bytes"),
        b"protected-content"
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn oversized_index_and_receipt_fail_closed_without_unbounded_reads() {
    let root = fixture("bounded-store-read");
    let index_binding = binding(&root);
    let expected = receipt(&index_binding);
    persist_reusable_receipt(&root, &index_binding, &expected).expect("seed store");
    let reuse = root.join(".ai/evidence/reuse");
    fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(reuse.join("index.json"))
        .expect("index")
        .set_len(9 * 1024 * 1024)
        .expect("oversized index");
    assert_eq!(
        unavailable_reason(load_reusable_receipt(&root, &index_binding).expect("load large index")),
        "index_unreadable"
    );

    persist_reusable_receipt(&root, &index_binding, &expected)
        .expect_err("invalid index blocks write");
    fs::remove_dir_all(root).expect("cleanup");

    let root = fixture("bounded-receipt-read");
    let binding = binding(&root);
    let expected = receipt(&binding);
    persist_reusable_receipt(&root, &binding, &expected).expect("seed receipt");
    let receipt_path = fs::read_dir(root.join(".ai/evidence/reuse/receipts"))
        .expect("receipts")
        .next()
        .expect("receipt entry")
        .expect("receipt")
        .path();
    fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(receipt_path)
        .expect("receipt file")
        .set_len(2 * 1024 * 1024)
        .expect("oversized receipt");
    assert_eq!(
        unavailable_reason(load_reusable_receipt(&root, &binding).expect("load large receipt")),
        "receipt_unreadable"
    );
    assert!(
        persist_reusable_receipt(&root, &binding, &expected).is_err(),
        "existing oversized content-addressed target must be rejected with a bounded compare"
    );
    fs::remove_dir_all(root).expect("cleanup");
}
