use super::*;
use cockpit_git::GitRepository;
use cockpit_protocol::{CollaborationDeclaration, CoordinationEventKind, RuntimeCapabilityBinding};
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

const CHILD_MODE: &str = "COCKPIT_COORDINATION_LOCK_TEST_MODE";
const CHILD_ROOT: &str = "COCKPIT_COORDINATION_LOCK_TEST_ROOT";
const CHILD_PAYLOAD: &str = "COCKPIT_COORDINATION_LOCK_TEST_PAYLOAD";
const CHILD_MARKER: &str = "COCKPIT_COORDINATION_LOCK_TEST_MARKER";
const CHILD_TEST_NAME: &str =
    "coordination_store::lock_recovery_tests::coordination_lock_child_helper";

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let sequence = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "cockpit-coordination-lock-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temporary repository");
        Self(path)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn git(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .expect("run git")
            .success(),
        "git {args:?} failed in {}",
        root.display()
    );
}

fn fixture() -> TestDirectory {
    let root = TestDirectory::new();
    git(&root.0, &["init", "-q"]);
    git(&root.0, &["config", "user.email", "test@example.invalid"]);
    git(&root.0, &["config", "user.name", "Coordination Lock Test"]);
    fs::write(root.0.join("README.md"), "initial\n").expect("write initial commit");
    git(&root.0, &["add", "."]);
    git(&root.0, &["commit", "-qm", "initial"]);
    root
}

fn binding() -> RuntimeCapabilityBinding {
    RuntimeCapabilityBinding {
        schema_version: 1,
        runtime_version: "0.2.113".into(),
        runtime_digest: Digest::sha256_bytes(b"candidate-runtime"),
        capability: cockpit_protocol::COLLABORATION_CAPABILITY.into(),
    }
}

fn store(root: &Path) -> CoordinationStore {
    let git = GitRepository::discover(root).expect("discover repository");
    CoordinationStore::open(&git, binding()).expect("open coordination store")
}

fn registration(root: &Path, work_item_id: &str, generation: u64) -> WorktreeRegistration {
    if !root.join(".ai/cockpit.toml").exists() {
        crate::attach(root).expect("attach repository");
    }
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    if !contract_path.exists() {
        crate::start_work_item_with_options(
            root,
            work_item_id,
            "coordination lock test",
            "recover a dead coordination lock owner",
            &[".ai/**".into(), "README.md".into()],
            &crate::WorkItemStartOptions {
                authority: "authorized".into(),
                out_of_scope: vec!["target/**".into()],
                acceptance_criteria: vec!["registration retry is crash-safe".into()],
                ..crate::WorkItemStartOptions::default()
            },
        )
        .expect("create test Work Item");
    }
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("read test Contract"))
            .expect("parse test Contract");
    let contract_digest = cockpit_protocol::digest_json(&contract).expect("digest Contract");
    let git = GitRepository::discover(root).expect("discover repository");
    let topology = git.topology().expect("read repository topology");
    WorktreeRegistration {
        schema_version: COLLABORATION_SCHEMA_VERSION,
        repository_id: crate::repository_id(root),
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

fn impact_event(registration: &WorktreeRegistration) -> CoordinationEvent {
    CoordinationEvent {
        schema_version: COLLABORATION_SCHEMA_VERSION,
        event_id: format!(
            "auto-impact-{}-{}",
            registration.work_item_id, registration.generation
        ),
        repository_id: registration.repository_id.clone(),
        work_item_id: registration.work_item_id.clone(),
        generation: registration.generation,
        kind: CoordinationEventKind::Impact,
        source: "registration-identity-changed".into(),
        evidence_refs: Vec::new(),
        evidence_digests: BTreeMap::new(),
        outcome_ids: Vec::new(),
    }
}

fn child_helper(mode: &str, root: &Path, payload: &str, marker: &Path) -> Child {
    Command::new(std::env::current_exe().expect("current unit-test executable"))
        .arg("--exact")
        .arg(CHILD_TEST_NAME)
        .arg("--nocapture")
        .env(CHILD_MODE, mode)
        .env(CHILD_ROOT, root)
        .env(CHILD_PAYLOAD, payload)
        .env(CHILD_MARKER, marker)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn unit-test child process")
}

fn wait_for_marker(marker: &Path, child: &mut Child) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if marker.is_file() {
            return;
        }
        if let Some(status) = child.try_wait().expect("check child status") {
            let stderr =
                std::io::BufReader::new(child.stderr.as_mut().expect("captured child stderr"));
            let mut stderr_bytes = Vec::new();
            stderr
                .take(4096)
                .read_to_end(&mut stderr_bytes)
                .expect("read child stderr");
            panic!(
                "child exited before its synchronization marker with {status}; stderr: {}",
                String::from_utf8_lossy(&stderr_bytes)
            );
        }
        assert!(Instant::now() < deadline, "child did not create marker");
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn coordination_lock_child_helper() {
    let Ok(mode) = std::env::var(CHILD_MODE) else {
        return;
    };
    let root = PathBuf::from(std::env::var_os(CHILD_ROOT).expect("child repository root"));
    let payload = std::env::var(CHILD_PAYLOAD).expect("child payload");
    let marker = PathBuf::from(std::env::var_os(CHILD_MARKER).expect("child marker"));
    let store = store(&root);
    match mode.as_str() {
        "hold" => {
            let event: CoordinationEvent =
                serde_json::from_str(&payload).expect("parse child impact event");
            let event_path = store
                .root
                .join("events")
                .join(format!("{}.json", event.event_id));
            store
                .with_lock(|| -> Result<(), CoordinationError> {
                    store.atomic_write(&event_path, &event)?;
                    fs::write(&marker, b"impact-durable-lock-held\n").map_err(|source| {
                        CoordinationError::Io {
                            path: marker.clone(),
                            source,
                        }
                    })?;
                    loop {
                        thread::sleep(Duration::from_secs(1));
                    }
                })
                .expect("hold production coordination lock");
        }
        "retry" => {
            let registration: WorktreeRegistration =
                serde_json::from_str(&payload).expect("parse retry registration");
            store
                .register(registration)
                .expect("retry registration after killed owner");
            fs::write(&marker, b"registration-reconciled\n").expect("write retry marker");
        }
        unknown => panic!("unknown child mode {unknown}"),
    }
}

#[test]
fn killed_lock_owner_releases_coordination_lock_and_registration_retry_reconciles_event() {
    let root = fixture();
    let store = store(&root.0);
    let original = registration(&root.0, "WI-LOCK-RECOVERY", 1);
    store
        .register(original.clone())
        .expect("register initial generation");

    fs::write(root.0.join("changed.txt"), "changed\n").expect("write changed source");
    git(&root.0, &["add", "."]);
    git(&root.0, &["commit", "-qm", "changed identity"]);
    let updated = registration(&root.0, "WI-LOCK-RECOVERY", 2);
    let event = impact_event(&updated);
    let event_json = serde_json::to_string(&event).expect("serialize deterministic impact");
    let owner_marker = root.0.join("owner-ready");
    let mut owner = child_helper("hold", &root.0, &event_json, &owner_marker);
    wait_for_marker(&owner_marker, &mut owner);
    let interrupted_registration: WorktreeRegistration = store
        .read_json(&store.registration_path("WI-LOCK-RECOVERY"))
        .expect("read still-authoritative registration");
    assert_eq!(
        interrupted_registration.generation, 1,
        "the new generation must not become authoritative before reconciliation"
    );
    let persisted_impact: CoordinationEvent = store
        .read_json(
            &store
                .root
                .join("events/auto-impact-WI-LOCK-RECOVERY-2.json"),
        )
        .expect("read durable deterministic impact event");
    assert_eq!(
        persisted_impact, event,
        "child must hold the production lock after persisting the deterministic impact"
    );

    let live_owner_error = store
        .register(updated.clone())
        .expect_err("a live lock owner must not be evicted");
    assert!(
        live_owner_error
            .to_string()
            .contains("coordination lock did not become available"),
        "live owner should retain the lock through timeout: {live_owner_error}"
    );
    assert!(
        owner.try_wait().expect("check live owner").is_none(),
        "the live lock owner must remain alive after a competing registration times out"
    );

    owner.kill().expect("kill lock-owning process");
    let owner_status = owner.wait().expect("wait for killed lock owner");
    assert!(
        !owner_status.success(),
        "lock owner was expected to be killed"
    );

    let retry_marker = root.0.join("retry-complete");
    let updated_json = serde_json::to_string(&updated).expect("serialize retry registration");
    let mut retry = child_helper("retry", &root.0, &updated_json, &retry_marker);
    let deadline = Instant::now() + Duration::from_secs(15);
    let status = loop {
        if let Some(status) = retry.try_wait().expect("check retry process") {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = retry.kill();
            panic!("fresh-process registration retry did not finish");
        }
        thread::sleep(Duration::from_millis(10));
    };
    let output = retry
        .wait_with_output()
        .expect("collect retry process output");
    assert!(
        status.success(),
        "fresh-process retry failed with {status}; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        retry_marker.is_file(),
        "retry process must report completion"
    );

    let inspection = store.inspect().expect("inspect reconciled registration");
    assert_eq!(
        inspection
            .events
            .iter()
            .filter(|event| event.event_id == "auto-impact-WI-LOCK-RECOVERY-2")
            .count(),
        1,
        "the deterministic impact event must remain exactly-once"
    );
    let authoritative = inspection
        .registrations
        .iter()
        .find(|registration| registration.work_item_id == "WI-LOCK-RECOVERY")
        .expect("reconciled registration");
    assert_eq!(authoritative.generation, 2);
    assert_eq!(authoritative, &updated);
}
