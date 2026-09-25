use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::{
    COLLABORATION_CAPABILITY, CollaborationDeclaration, CoordinationEvent, CoordinationEventKind,
    CoordinationIntent, CoordinationRequest, CoordinationRequestState, OutcomeStage,
    ProvidedOutcome, ResourceClaim, ResourceClaimMode, ResourceReservation,
    RuntimeCapabilityBinding, WorktreeRegistration,
};
use cockpit_repository::{
    CoordinationError, CoordinationStore, WorkItemStartOptions, attach, repository_id,
    start_work_item_with_options,
};
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
    fs::create_dir_all(root.path().join("target/evidence")).expect("evidence directory");
    fs::create_dir_all(root.path().join("evidence")).expect("recovery evidence directory");
    fs::write(root.path().join("target/evidence.json"), "{}\n").expect("event evidence");
    fs::write(root.path().join("target/impact.json"), "{}\n").expect("impact evidence");
    fs::write(root.path().join("evidence/impact.json"), "{}\n").expect("recovery evidence");
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

fn contract_digest(root: &Path, work_item_id: &str) -> Digest {
    if !root.join(".ai/cockpit.toml").exists() {
        attach(root).expect("attach repository");
    }
    let path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    if !path.exists() {
        start_work_item_with_options(
            root,
            work_item_id,
            "coordination test",
            "bind coordination records to observed facts",
            &[".ai/**".into(), "README.md".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                out_of_scope: vec!["target/**".into()],
                acceptance_criteria: vec!["identity remains bound".into()],
                ..WorkItemStartOptions::default()
            },
        )
        .expect("start test Work Item");
    }
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(path).expect("Contract bytes")).expect("Contract JSON");
    cockpit_protocol::digest_json(&value).expect("Contract digest")
}

fn registration(root: &Path, work_item_id: &str, generation: u64) -> WorktreeRegistration {
    let contract_digest = contract_digest(root, work_item_id);
    let git = GitRepository::discover(root).expect("discover");
    let topology = git.topology().expect("topology");
    WorktreeRegistration {
        schema_version: 1,
        repository_id: repository_id(root),
        work_item_id: work_item_id.into(),
        contract_digest,
        worktree_path: topology.repository_root.to_string_lossy().into_owned(),
        branch: topology.branch.expect("branch"),
        head: topology.head.expect("head"),
        generation,
        declaration: CollaborationDeclaration::default(),
        runtime: binding(),
    }
}

fn registration_with_outcome(root: &Path, work_item_id: &str) -> WorktreeRegistration {
    let mut value = registration(root, work_item_id, 1);
    value.declaration.provided_outcomes = vec![ProvidedOutcome {
        outcome_id: "api".into(),
        interface_contract: "api-v1".into(),
        behavior_contract: "stable behavior".into(),
        published_head: value.head.clone(),
        stage: OutcomeStage::ComposableHead,
        evidence_refs: vec!["target/evidence.json".into()],
    }];
    value
}

#[cfg(unix)]
fn replace_target_directory_with_external_symlink(root: &Path, outside: &Path) {
    use std::os::unix::fs::symlink;

    let target = root.join("target");
    let backup = root.join("target-before-parent-symlink");
    fs::rename(&target, &backup).expect("preserve target directory");
    symlink(outside, target).expect("link target directory outside worktree");
}

fn reservation(
    root: &Path,
    work_item_id: &str,
    reservation_id: &str,
    resources: &[&str],
) -> ResourceReservation {
    ResourceReservation {
        schema_version: 1,
        repository_id: repository_id(root),
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

fn coordination_request(
    root: &Path,
    request_id: &str,
    state: CoordinationRequestState,
) -> CoordinationRequest {
    CoordinationRequest {
        schema_version: 1,
        request_id: request_id.into(),
        repository_id: repository_id(root),
        target_work_item_id: "WI-REQUEST".into(),
        target_generation: 1,
        intent: CoordinationIntent::RequestSafePause,
        state,
        reason: "test coordination request".into(),
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
        repository_id: repository_id(root.path()),
        work_item_id: "WI-A".into(),
        generation: 1,
        kind: CoordinationEventKind::Impact,
        source: "test".into(),
        evidence_refs: vec!["target/evidence.json".into()],
        evidence_digests: Default::default(),
        outcome_ids: Vec::new(),
    };
    let published = store.publish_event(event.clone()).unwrap();
    assert_eq!(store.publish_event(event.clone()).unwrap(), published);
    let serialized = serde_json::to_value(published).expect("serialized event");
    assert_eq!(
        serialized["evidenceDigests"],
        serde_json::json!({
            "target/evidence.json": Digest::sha256_bytes(b"{}\n").to_string()
        })
    );
}

#[test]
fn request_transition_rejects_traversal_and_mismatched_record_identity() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(root.path(), "WI-REQUEST", 1))
        .expect("register request target");

    let escaped_path = store.root().join("events/victim.json");
    let escaped_record = coordination_request(
        root.path(),
        "victim-record",
        CoordinationRequestState::Requested,
    );
    fs::write(
        &escaped_path,
        serde_json::to_vec_pretty(&escaped_record).expect("serialize escaped request"),
    )
    .expect("write request-shaped record outside requests directory");
    let escaped_before = fs::read(&escaped_path).expect("read escaped request before transition");
    let traversal_result =
        store.transition_request("../events/victim", CoordinationRequestState::Acknowledged);
    assert!(
        traversal_result.is_err(),
        "request IDs must be validated before they become filesystem paths"
    );
    assert_eq!(
        fs::read(&escaped_path).expect("read escaped request after transition"),
        escaped_before,
        "a rejected traversal must not modify the out-of-directory record"
    );

    let stored = coordination_request(
        root.path(),
        "stored-request",
        CoordinationRequestState::Requested,
    );
    store
        .request_coordination(stored)
        .expect("create valid request");
    let request_path = store.root().join("requests/stored-request.json");
    let mut mismatched: serde_json::Value =
        serde_json::from_slice(&fs::read(&request_path).expect("read request"))
            .expect("parse request");
    mismatched["requestId"] = serde_json::json!("different-record");
    fs::write(
        &request_path,
        serde_json::to_vec_pretty(&mismatched).expect("serialize mismatched request"),
    )
    .expect("write mismatched identity fixture");
    let mismatched_before = fs::read(&request_path).expect("read mismatched request before");
    let mismatched_result =
        store.transition_request("stored-request", CoordinationRequestState::Acknowledged);
    assert!(
        mismatched_result.is_err(),
        "the file identity must match the requested coordination ID"
    );
    assert_eq!(
        fs::read(&request_path).expect("read mismatched request after"),
        mismatched_before,
        "a rejected embedded-ID mismatch must leave its record unchanged"
    );
}

#[test]
fn request_transition_rejects_tampered_target_work_item_path() {
    let root = repository();
    let store = store(root.path());
    let registration = registration(root.path(), "WI-REQUEST", 1);
    store
        .register(registration.clone())
        .expect("register request target");
    store
        .request_coordination(coordination_request(
            root.path(),
            "tampered-target",
            CoordinationRequestState::Requested,
        ))
        .expect("create valid request");

    let escaped_registration_path = store.root().join("events/victim.json");
    fs::write(
        &escaped_registration_path,
        serde_json::to_vec_pretty(&registration).expect("serialize registration fixture"),
    )
    .expect("place valid registration-shaped data outside registrations directory");

    let request_path = store.root().join("requests/tampered-target.json");
    let mut tampered: serde_json::Value =
        serde_json::from_slice(&fs::read(&request_path).expect("read request"))
            .expect("parse request");
    tampered["targetWorkItemId"] = serde_json::json!("../events/victim");
    fs::write(
        &request_path,
        serde_json::to_vec_pretty(&tampered).expect("serialize tampered request"),
    )
    .expect("tamper persisted target identity");
    let request_before = fs::read(&request_path).expect("read request before transition");

    let result =
        store.transition_request("tampered-target", CoordinationRequestState::Acknowledged);
    assert!(
        result.is_err(),
        "stored target identity must not escape registration path or bind to a different Work Item"
    );
    assert_eq!(
        fs::read(&request_path).expect("read request after transition"),
        request_before,
        "a rejected tampered target must leave the request unchanged"
    );

    let alias_registration_path = store.registration_path("WI-ALIAS");
    fs::write(
        &alias_registration_path,
        serde_json::to_vec_pretty(&registration).expect("serialize aliased registration"),
    )
    .expect("write registration whose stored identity differs from its path");
    tampered["targetWorkItemId"] = serde_json::json!("WI-ALIAS");
    fs::write(
        &request_path,
        serde_json::to_vec_pretty(&tampered).expect("serialize aliased request"),
    )
    .expect("tamper persisted target to a valid path with a different registration identity");
    let aliased_request_before = fs::read(&request_path).expect("read aliased request before");

    let aliased_result =
        store.transition_request("tampered-target", CoordinationRequestState::Acknowledged);
    assert!(
        aliased_result.is_err(),
        "loaded registration identity must match the stored request target"
    );
    assert_eq!(
        fs::read(&request_path).expect("read aliased request after"),
        aliased_request_before,
        "a rejected registration identity mismatch must leave the request unchanged"
    );
}

#[test]
fn request_creation_rejects_registration_target_mismatch_before_fact_validation() {
    let root = repository();
    let store = store(root.path());
    let registration = registration(root.path(), "WI-REQUEST", 1);
    store
        .register(registration.clone())
        .expect("register request target");

    let alias_path = store.registration_path("WI-ALIAS");
    let mut tampered_registration =
        serde_json::to_value(&registration).expect("serialize registration fixture");
    tampered_registration["workItemId"] = serde_json::json!("../events/victim");
    fs::write(
        &alias_path,
        serde_json::to_vec_pretty(&tampered_registration)
            .expect("serialize traversal registration fixture"),
    )
    .expect("write registration with traversal identity under safe alias path");

    let mut request = coordination_request(
        root.path(),
        "request-traversal-registration",
        CoordinationRequestState::Requested,
    );
    request.target_work_item_id = "WI-ALIAS".into();
    let error = store
        .request_coordination(request)
        .expect_err("a mismatched embedded registration ID must be rejected first");
    assert!(
        error
            .to_string()
            .contains("coordination request target differs from registration identity"),
        "identity mismatch must be rejected before registration facts use the embedded ID: {error}"
    );
    assert!(
        !store
            .root()
            .join("requests/request-traversal-registration.json")
            .exists(),
        "rejected request creation must not persist a request record"
    );
}

#[test]
fn request_transition_rejects_registration_traversal_before_fact_validation() {
    let root = repository();
    let store = store(root.path());
    let registration = registration(root.path(), "WI-REQUEST", 1);
    store
        .register(registration.clone())
        .expect("register request target");
    store
        .request_coordination(coordination_request(
            root.path(),
            "request-traversal-registration",
            CoordinationRequestState::Requested,
        ))
        .expect("create valid request");

    let alias_path = store.registration_path("WI-ALIAS");
    let mut tampered_registration =
        serde_json::to_value(&registration).expect("serialize registration fixture");
    tampered_registration["workItemId"] = serde_json::json!("../events/victim");
    fs::write(
        &alias_path,
        serde_json::to_vec_pretty(&tampered_registration)
            .expect("serialize traversal registration fixture"),
    )
    .expect("write registration with traversal identity under safe alias path");

    let request_path = store
        .root()
        .join("requests/request-traversal-registration.json");
    let mut request: serde_json::Value =
        serde_json::from_slice(&fs::read(&request_path).expect("read request"))
            .expect("parse request");
    request["targetWorkItemId"] = serde_json::json!("WI-ALIAS");
    fs::write(
        &request_path,
        serde_json::to_vec_pretty(&request).expect("serialize request with safe alias target"),
    )
    .expect("write request with safe alias target");
    let request_before = fs::read(&request_path).expect("read tampered request before transition");

    let error = store
        .transition_request(
            "request-traversal-registration",
            CoordinationRequestState::Acknowledged,
        )
        .expect_err("a traversal registration identity must be rejected first");
    assert!(
        error
            .to_string()
            .contains("coordination request target differs from registration identity"),
        "identity mismatch must be rejected before registration facts use the embedded ID: {error}"
    );
    assert_eq!(
        fs::read(&request_path).expect("read tampered request after transition"),
        request_before,
        "a rejected traversal registration identity must leave request bytes unchanged"
    );
}

#[test]
fn request_creation_accepts_only_the_requested_initial_state() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(root.path(), "WI-REQUEST", 1))
        .expect("register request target");

    for (request_id, state) in [
        ("request-resumed", CoordinationRequestState::Resumed),
        ("request-paused", CoordinationRequestState::SafelyPaused),
    ] {
        let result =
            store.request_coordination(coordination_request(root.path(), request_id, state));
        assert!(
            result.is_err(),
            "request creation must reject pre-acknowledged state {state:?}"
        );
    }
    assert!(
        store
            .inspect()
            .expect("inspect rejected requests")
            .requests
            .is_empty(),
        "rejected initial states must not create durable request records"
    );

    let requested = coordination_request(
        root.path(),
        "request-valid",
        CoordinationRequestState::Requested,
    );
    store
        .request_coordination(requested)
        .expect("Requested remains the valid initial state");
    assert_eq!(
        store
            .transition_request("request-valid", CoordinationRequestState::Acknowledged)
            .expect("valid request follows the transition API")
            .state,
        CoordinationRequestState::Acknowledged
    );
}

#[test]
fn event_publication_rejects_a_caller_supplied_digest_that_differs_from_file_bytes() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(root.path(), "WI-A", 1))
        .expect("register provider");

    let event: CoordinationEvent = serde_json::from_value(serde_json::json!({
        "schemaVersion": 1,
        "eventId": "event-wrong-digest",
        "repositoryId": repository_id(root.path()),
        "workItemId": "WI-A",
        "generation": 1,
        "kind": "impact",
        "source": "test",
        "evidenceRefs": ["target/evidence.json"],
        "evidenceDigests": {"target/evidence.json": digest("wrong")},
        "outcomeIds": []
    }))
    .expect("parse evidence-digest event");

    let error = store
        .publish_event(event)
        .expect_err("caller-supplied digest must not override observed evidence bytes");
    assert!(
        error.to_string().contains("evidence digest"),
        "unexpected publication error: {error}"
    );
}

#[cfg(unix)]
#[test]
fn registration_rejects_evidence_reached_through_parent_symlink() {
    let root = repository();
    let store = store(root.path());
    let registration = registration_with_outcome(root.path(), "WI-PARENT-LINK");
    let outside = tempfile::tempdir().expect("outside evidence directory");
    fs::write(outside.path().join("evidence.json"), "outside evidence\n")
        .expect("write outside evidence");
    replace_target_directory_with_external_symlink(root.path(), outside.path());

    let result = store.register(registration);

    assert!(
        result.is_err(),
        "registration must reject an evidence file reached through a symlinked parent"
    );
}

#[cfg(unix)]
#[test]
fn event_publication_rejects_evidence_reached_through_parent_symlink() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(root.path(), "WI-EVENT-PARENT-LINK", 1))
        .expect("register provider");
    let outside = tempfile::tempdir().expect("outside evidence directory");
    fs::write(outside.path().join("evidence.json"), "outside evidence\n")
        .expect("write outside evidence");
    replace_target_directory_with_external_symlink(root.path(), outside.path());

    let result = store.publish_event(CoordinationEvent {
        schema_version: 1,
        event_id: "outside-parent-event".into(),
        repository_id: repository_id(root.path()),
        work_item_id: "WI-EVENT-PARENT-LINK".into(),
        generation: 1,
        kind: CoordinationEventKind::Impact,
        source: "parent-symlink-test".into(),
        evidence_refs: vec!["target/evidence.json".into()],
        evidence_digests: Default::default(),
        outcome_ids: Vec::new(),
    });

    assert!(
        result.is_err(),
        "event publication must not digest evidence reached through a parent symlink"
    );
    assert!(
        store.inspect().expect("inspect store").events.is_empty(),
        "rejected outside evidence must not persist an event"
    );
}

#[cfg(unix)]
#[test]
fn inspection_reports_registration_evidence_reached_through_parent_symlink() {
    let root = repository();
    let store = store(root.path());
    let registration = registration_with_outcome(root.path(), "WI-INSPECT-PARENT-LINK");
    fs::write(
        store.registration_path(&registration.work_item_id),
        serde_json::to_vec_pretty(&registration).expect("serialize registration"),
    )
    .expect("write registered record");
    let outside = tempfile::tempdir().expect("outside evidence directory");
    fs::write(outside.path().join("evidence.json"), "outside evidence\n")
        .expect("write outside evidence");
    replace_target_directory_with_external_symlink(root.path(), outside.path());

    let inspection = store
        .inspect()
        .expect("inspection preserves unknown record");

    assert!(
        inspection
            .unknowns
            .iter()
            .any(|unknown| unknown.contains("evidence")),
        "inspection must surface a registration whose evidence parent is a symlink: {inspection:?}"
    );
    assert!(inspection.registrations.is_empty());
}

#[test]
fn coordination_event_round_trips_outcome_specific_invalidation() {
    let value = serde_json::json!({
        "schemaVersion": 1,
        "eventId": "impact-api",
        "repositoryId": digest("repository"),
        "workItemId": "WI-PROVIDER",
        "generation": 1,
        "kind": "impact",
        "source": "api-contract-changed",
        "evidenceRefs": [],
        "outcomeIds": ["api"]
    });
    let parsed = serde_json::from_value::<CoordinationEvent>(value.clone());
    assert!(
        parsed.is_ok(),
        "the event protocol must represent which provided outcome changed: {parsed:?}"
    );
    assert_eq!(
        serde_json::to_value(parsed.unwrap()).expect("serialize event")["outcomeIds"],
        value["outcomeIds"]
    );
}

#[test]
fn registration_identity_change_appends_an_impact_event() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(root.path(), "WI-AUTO", 1))
        .expect("register first generation");

    fs::write(root.path().join("changed.txt"), "changed\n").expect("write change");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "changed identity"]);

    store
        .register(registration(root.path(), "WI-AUTO", 2))
        .expect("register changed generation");
    let inspection = store.inspect().expect("inspect automatic impact");
    let event = inspection
        .events
        .iter()
        .find(|event| event.event_id == "auto-impact-WI-AUTO-2")
        .expect("automatic impact event");
    assert_eq!(event.kind, CoordinationEventKind::Impact);
    assert_eq!(event.generation, 2);
    assert_eq!(event.source, "registration-identity-changed");
}

#[test]
fn interrupted_identity_change_keeps_old_registration_and_retry_appends_impact_once() {
    let root = repository();
    let store = store(root.path());
    let original = registration(root.path(), "WI-RECOVER-IMPACT", 1);
    store
        .register(original.clone())
        .expect("register original identity");

    fs::write(root.path().join("changed.txt"), "changed\n").expect("write change");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "changed identity"]);
    let updated = registration(root.path(), "WI-RECOVER-IMPACT", 2);
    let event_path = store
        .root()
        .join("events/auto-impact-WI-RECOVER-IMPACT-2.json");
    fs::create_dir_all(&event_path).expect("inject event write interruption");

    assert!(
        store.register(updated.clone()).is_err(),
        "a failed impact write must not report registration success"
    );
    let persisted: WorktreeRegistration = serde_json::from_slice(
        &fs::read(store.registration_path("WI-RECOVER-IMPACT")).expect("registration record"),
    )
    .expect("registration JSON");
    assert_eq!(
        persisted, original,
        "new repository facts must not become authoritative before the impact event"
    );

    fs::remove_dir(&event_path).expect("remove injected failure");
    assert_eq!(
        store
            .register(updated.clone())
            .expect("retry identical registration after interruption"),
        updated
    );
    let inspection = store.inspect().expect("inspect reconciled store");
    assert_eq!(
        inspection
            .events
            .iter()
            .filter(|event| event.event_id == "auto-impact-WI-RECOVER-IMPACT-2")
            .count(),
        1
    );
}

#[test]
fn retry_completes_registration_after_event_was_durable_before_interruption() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(root.path(), "WI-CRASH-WINDOW", 1))
        .expect("register original identity");
    fs::write(root.path().join("changed.txt"), "changed\n").expect("write change");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "changed identity"]);
    let updated = registration(root.path(), "WI-CRASH-WINDOW", 2);

    // Model process death at the exact durable boundary: the deterministic
    // impact record exists, while registration remains at generation one.
    let event = CoordinationEvent {
        schema_version: 1,
        event_id: "auto-impact-WI-CRASH-WINDOW-2".into(),
        repository_id: updated.repository_id.clone(),
        work_item_id: updated.work_item_id.clone(),
        generation: updated.generation,
        kind: CoordinationEventKind::Impact,
        source: "registration-identity-changed".into(),
        evidence_refs: Vec::new(),
        evidence_digests: Default::default(),
        outcome_ids: Vec::new(),
    };
    let event_path = store
        .root()
        .join("events/auto-impact-WI-CRASH-WINDOW-2.json");
    fs::write(
        &event_path,
        serde_json::to_vec_pretty(&event).expect("serialize impact event"),
    )
    .expect("persist impact before simulated interruption");
    let still_old: WorktreeRegistration = serde_json::from_slice(
        &fs::read(store.registration_path("WI-CRASH-WINDOW")).expect("old registration"),
    )
    .expect("old registration JSON");
    assert_eq!(still_old.generation, 1);

    assert_eq!(
        store
            .register(updated.clone())
            .expect("retry after impact persisted"),
        updated
    );
    let inspection = store.inspect().expect("inspect resumed registration");
    assert_eq!(
        inspection
            .events
            .iter()
            .filter(|event| event.event_id == "auto-impact-WI-CRASH-WINDOW-2")
            .count(),
        1
    );
    assert_eq!(
        inspection
            .registrations
            .iter()
            .find(|registration| registration.work_item_id == "WI-CRASH-WINDOW")
            .unwrap()
            .generation,
        2
    );
}

#[test]
fn conflicting_resources_are_atomic_and_partial_reservation_rolls_back() {
    let root = repository();
    let store = Arc::new(store(root.path()));
    store
        .register(registration(root.path(), "WI-A", 1))
        .expect("register A");
    store
        .register(registration(root.path(), "WI-B", 1))
        .expect("register B");
    let first = {
        let store = Arc::clone(&store);
        let root = root.path().to_path_buf();
        thread::spawn(move || {
            store.reserve_resources(reservation(&root, "WI-A", "r-a", &["src/a", "src/b"]))
        })
    };
    let second = {
        let store = Arc::clone(&store);
        let root = root.path().to_path_buf();
        thread::spawn(move || {
            store.reserve_resources(reservation(&root, "WI-B", "r-b", &["src/b", "src/c"]))
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
        repository_id: repository_id(root.path()),
        work_item_id: "WI-A".into(),
        generation: 1,
        kind: CoordinationEventKind::Impact,
        source: "late-process".into(),
        evidence_refs: Vec::new(),
        evidence_digests: Default::default(),
        outcome_ids: Vec::new(),
    };
    assert!(matches!(
        store.publish_event(late),
        Err(CoordinationError::StaleGeneration { .. })
    ));

    let registration_path = store.registration_path("WI-A");
    fs::write(&registration_path, b"{partial").expect("corrupt record");
    let inspection = store.inspect().expect("inspect corrupt");
    assert!(!inspection.unknowns.is_empty());
    assert!(!store.recover().expect("recover").unknowns.is_empty());
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
        repository_id: repository_id(root.path()),
        work_item_id: "WI-PROVIDER".into(),
        generation: 1,
        kind: CoordinationEventKind::Impact,
        source: "recovery-test".into(),
        evidence_refs: vec!["evidence/impact.json".into()],
        evidence_digests: Default::default(),
        outcome_ids: Vec::new(),
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
    store
        .register(registration(root.path(), "WI-PROVIDER", 2))
        .expect("new provider generation");
    let current = store
        .recover_event("recoverable-impact", "WI-CONSUMER", 2)
        .expect("cross-generation recovery");
    assert_eq!(current.provider_generation, 1);
    assert_eq!(current.current_provider_generation, Some(2));
    assert_eq!(store.inspect().expect("inspect").unknowns.len(), 0);
}

#[test]
fn corrupt_reservation_is_not_treated_as_free() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(root.path(), "WI-A", 1))
        .expect("register");
    store
        .reserve_resources(reservation(root.path(), "WI-A", "r-a", &["src/a"]))
        .expect("reserve");
    fs::write(
        store.root().join("reservations/r-a.json"),
        b"partial-record",
    )
    .expect("corrupt reservation");
    store
        .register(registration(root.path(), "WI-B", 1))
        .expect("register B");
    let result = store.reserve_resources(reservation(root.path(), "WI-B", "r-b", &["src/a"]));
    assert!(matches!(
        result,
        Err(CoordinationError::InvalidRecord { .. })
    ));
}

#[test]
fn stale_generation_cannot_release_an_existing_resource_reservation() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(root.path(), "WI-A", 1))
        .expect("register first generation");
    store
        .reserve_resources(reservation(root.path(), "WI-A", "r-a", &["src/a"]))
        .expect("reserve resource");

    fs::write(root.path().join("generation-two.txt"), "changed\n").expect("write change");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "generation two"]);
    store
        .register(registration(root.path(), "WI-A", 2))
        .expect("register second generation");

    let result = store.release_resources("r-a", "WI-A", 1);
    assert!(matches!(
        result,
        Err(CoordinationError::StaleGeneration { .. })
    ));
    assert_eq!(store.inspect().expect("inspect").reservations.len(), 1);
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

type RegistrationMutation<'a> = (&'a str, Box<dyn Fn(&mut WorktreeRegistration) + 'a>);

#[test]
fn registration_identity_is_rejected_before_persistence_when_observed_facts_differ() {
    let root = repository();
    let store = store(root.path());
    let valid = registration(root.path(), "WI-IDENTITY", 1);
    let cases: Vec<RegistrationMutation<'_>> = vec![
        (
            "repository",
            Box::new(|value: &mut WorktreeRegistration| {
                value.repository_id = digest("forged-repository");
            }),
        ),
        (
            "contract",
            Box::new(|value: &mut WorktreeRegistration| {
                value.contract_digest = digest("forged-contract");
            }),
        ),
        (
            "branch",
            Box::new(|value: &mut WorktreeRegistration| {
                value.branch = "codex/forged-branch".into();
            }),
        ),
        (
            "head",
            Box::new(|value: &mut WorktreeRegistration| {
                value.head = "1111111111111111111111111111111111111111".into();
            }),
        ),
        (
            "worktree",
            Box::new(|value: &mut WorktreeRegistration| {
                value.worktree_path = root
                    .path()
                    .parent()
                    .expect("temporary parent")
                    .to_string_lossy()
                    .into_owned();
            }),
        ),
    ];
    for (label, mutate) in cases {
        let mut forged = valid.clone();
        mutate(&mut forged);
        assert!(
            store.register(forged).is_err(),
            "{label} mismatch must fail closed"
        );
        assert!(!store.registration_path("WI-IDENTITY").exists());
    }
}

#[test]
fn registration_rejects_a_contract_with_a_different_declared_identity() {
    for (field, replacement) in [
        ("workItemId", serde_json::json!("WI-OTHER")),
        (
            "repositoryId",
            serde_json::json!("sha256:foreign-repository"),
        ),
    ] {
        let root = repository();
        let store = store(root.path());
        let work_item_id = "WI-IDENTITY";
        let _contract_digest = contract_digest(root.path(), work_item_id);
        let contract_path = root
            .path()
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"));
        let mut contract: serde_json::Value =
            serde_json::from_slice(&fs::read(&contract_path).expect("Contract bytes"))
                .expect("Contract JSON");
        contract[field] = replacement;
        fs::write(
            &contract_path,
            serde_json::to_vec_pretty(&contract).expect("serialize Contract"),
        )
        .expect("write misbound Contract");

        let mut registration = registration(root.path(), work_item_id, 1);
        registration.contract_digest =
            cockpit_protocol::digest_json(&contract).expect("misbound Contract digest");
        let result = store.register(registration);

        assert!(
            matches!(result, Err(CoordinationError::RecoveryRequired(_))),
            "Contract {field} mismatch must fail closed"
        );
        assert!(
            !store.registration_path(work_item_id).exists(),
            "Contract {field} mismatch must not persist a registration"
        );
    }
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
