use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::{
    COLLABORATION_CAPABILITY, CollaborationDeclaration, CoordinationEvent, CoordinationEventKind,
    ResourceClaim, ResourceClaimMode, ResourceReservation, RuntimeCapabilityBinding,
    WorktreeRegistration,
};
use cockpit_repository::{CoordinationError, CoordinationStore};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::thread;

fn digest(label: &str) -> Digest {
    Digest::sha256_bytes(label.as_bytes())
}

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    run(root.path(), &["init", "-q"]);
    run(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    run(root.path(), &["config", "user.name", "Test"]);
    fs::write(root.path().join("README.md"), "initial\n").expect("write");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "initial"]);
    root
}

fn run(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .expect("git command")
            .success()
    );
}

fn binding() -> RuntimeCapabilityBinding {
    RuntimeCapabilityBinding {
        schema_version: 1,
        runtime_version: "0.2.113".into(),
        runtime_digest: digest("candidate-runtime"),
        capability: COLLABORATION_CAPABILITY.into(),
    }
}

fn store(root: &Path) -> CoordinationStore {
    let git = GitRepository::discover(root).expect("discover");
    CoordinationStore::open(&git, binding()).expect("store")
}

fn registration(root: &Path, work_item_id: &str, generation: u64) -> WorktreeRegistration {
    WorktreeRegistration {
        schema_version: 1,
        repository_id: digest("repository"),
        work_item_id: work_item_id.into(),
        contract_digest: digest(&format!("contract-{work_item_id}")),
        worktree_path: root.to_string_lossy().into_owned(),
        branch: format!("codex/{work_item_id}"),
        head: "0123456789012345678901234567890123456789".into(),
        generation,
        declaration: CollaborationDeclaration::default(),
        runtime: binding(),
    }
}

fn reservation(
    work_item_id: &str,
    reservation_id: &str,
    resources: &[&str],
) -> ResourceReservation {
    ResourceReservation {
        schema_version: 1,
        repository_id: digest("repository"),
        reservation_id: reservation_id.into(),
        work_item_id: work_item_id.into(),
        generation: 1,
        resources: resources
            .iter()
            .map(|resource_id| ResourceClaim {
                resource_id: (*resource_id).into(),
                mode: ResourceClaimMode::Exclusive,
                serial: true,
            })
            .collect(),
    }
}

#[test]
fn duplicate_registration_and_event_are_idempotent() {
    let root = repository();
    let store = store(root.path());
    let registration = registration(root.path(), "WI-A", 1);
    assert_eq!(store.register(registration.clone()).unwrap(), registration);
    assert_eq!(store.register(registration.clone()).unwrap(), registration);

    let event = CoordinationEvent {
        schema_version: 1,
        event_id: "event-1".into(),
        repository_id: digest("repository"),
        work_item_id: "WI-A".into(),
        generation: 1,
        kind: CoordinationEventKind::Impact,
        source: "test".into(),
        evidence_refs: vec!["target/evidence.json".into()],
    };
    assert_eq!(store.publish_event(event.clone()).unwrap(), event);
    assert_eq!(store.publish_event(event.clone()).unwrap(), event);
}

#[test]
fn conflicting_resources_are_atomic_and_partial_reservation_rolls_back() {
    let root = repository();
    let store = Arc::new(store(root.path()));
    let first = {
        let store = Arc::clone(&store);
        thread::spawn(move || {
            store.reserve_resources(reservation("WI-A", "r-a", &["src/a", "src/b"]))
        })
    };
    let second = {
        let store = Arc::clone(&store);
        thread::spawn(move || {
            store.reserve_resources(reservation("WI-B", "r-b", &["src/b", "src/c"]))
        })
    };
    let results = [first.join().unwrap(), second.join().unwrap()];
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    let inspection = store.inspect().expect("inspect");
    assert_eq!(inspection.reservations.len(), 1);
    assert!(inspection.unknowns.is_empty());
}

#[test]
fn stale_generation_and_corrupt_or_moved_records_require_recovery() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(root.path(), "WI-A", 1))
        .expect("register first");
    store
        .register(registration(root.path(), "WI-A", 2))
        .expect("register new generation");
    let late = CoordinationEvent {
        schema_version: 1,
        event_id: "late".into(),
        repository_id: digest("repository"),
        work_item_id: "WI-A".into(),
        generation: 1,
        kind: CoordinationEventKind::Impact,
        source: "late-process".into(),
        evidence_refs: Vec::new(),
    };
    assert!(matches!(
        store.publish_event(late),
        Err(CoordinationError::StaleGeneration { .. })
    ));

    let registration_path = store.registration_path("WI-A");
    fs::write(&registration_path, b"{partial").expect("corrupt record");
    let inspection = store.inspect().expect("inspect corrupt");
    assert!(!inspection.unknowns.is_empty());
    assert!(store.recover().expect("recover").unknowns.len() >= 1);
}

#[test]
fn recovery_consumes_only_matching_event_and_is_idempotent() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(root.path(), "WI-PROVIDER", 1))
        .expect("register provider");
    store
        .register(registration(root.path(), "WI-CONSUMER", 1))
        .expect("register consumer");
    let event = CoordinationEvent {
        schema_version: 1,
        event_id: "recoverable-impact".into(),
        repository_id: digest("repository"),
        work_item_id: "WI-PROVIDER".into(),
        generation: 1,
        kind: CoordinationEventKind::Impact,
        source: "recovery-test".into(),
        evidence_refs: vec!["evidence/impact.json".into()],
    };
    store.publish_event(event.clone()).expect("publish event");

    let consumed = store
        .recover_event("recoverable-impact", "WI-CONSUMER", 1)
        .expect("consume event");
    assert_eq!(consumed.event_id, event.event_id);
    assert_eq!(consumed.consumer_work_item_id, "WI-CONSUMER");
    assert_eq!(
        store
            .recover_event("recoverable-impact", "WI-CONSUMER", 1)
            .expect("duplicate consumption"),
        consumed
    );

    store
        .register(registration(root.path(), "WI-CONSUMER", 2))
        .expect("new consumer generation");
    let stale = store.recover_event("recoverable-impact", "WI-CONSUMER", 1);
    assert!(matches!(
        stale,
        Err(CoordinationError::StaleGeneration { .. })
    ));
    assert_eq!(store.inspect().expect("inspect").unknowns.len(), 0);
}

#[test]
fn corrupt_reservation_is_not_treated_as_free() {
    let root = repository();
    let store = store(root.path());
    store
        .reserve_resources(reservation("WI-A", "r-a", &["src/a"]))
        .expect("reserve");
    fs::write(
        store.root().join("reservations/r-a.json"),
        b"partial-record",
    )
    .expect("corrupt reservation");
    let result = store.reserve_resources(reservation("WI-B", "r-b", &["src/a"]));
    assert!(matches!(
        result,
        Err(CoordinationError::InvalidRecord { .. })
    ));
}

#[test]
fn registration_with_a_different_runtime_identity_is_rejected_before_write() {
    let root = repository();
    let store = store(root.path());
    let mut registration = registration(root.path(), "WI-A", 1);
    registration.runtime.runtime_version = "0.2.114".into();
    let result = store.register(registration);
    assert!(matches!(
        result,
        Err(CoordinationError::Runtime(
            cockpit_protocol::RuntimeCapabilityError::IdentityMismatch
        ))
    ));
    assert!(!store.registration_path("WI-A").exists());
}

#[test]
fn read_only_inspection_does_not_create_coordination_storage() {
    let root = repository();
    let git = GitRepository::discover(root.path()).expect("discover");
    let store = CoordinationStore::open_read_only(&git, binding()).expect("read-only store");
    let projection = store.inspect().expect("inspect absent store");
    assert!(projection.registrations.is_empty());
    assert!(projection.events.is_empty());
    assert!(!store.root().exists());
}
