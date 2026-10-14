use cockpit_core::Digest;
use cockpit_protocol::{AssuranceLevel, DelegatedEvidence, EvidenceValidity, RuntimeContext};
use cockpit_repository::{
    WorkItemStartOptions, attach, import_delegated_evidence, import_delegated_evidence_observation,
    list_delegated_evidence, list_delegated_evidence_observations, start_work_item_with_options,
};
use std::{fs, path::Path, process::Command};

const WORK_ITEM_ID: &str = "WI-OBSERVATION-TEST";

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(root.path())
            .status()
            .expect("git init")
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit-test@example.invalid",
                "commit",
                "--allow-empty",
                "-m",
                "fixture baseline",
            ])
            .current_dir(root.path())
            .status()
            .expect("git baseline commit")
            .success()
    );
    attach(root.path()).expect("attach");
    start_work_item_with_options(
        root.path(),
        WORK_ITEM_ID,
        "append delegated evidence observations",
        "prove same-byte source corrections remain selectable without rewriting history",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("start Work Item");
    root
}

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "1.0.1-test".into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"observation test runtime"),
    }
}

fn evidence(raw: &[u8], origin: &str) -> DelegatedEvidence {
    DelegatedEvidence {
        provider: "github".into(),
        subject: "https://github.com/example/repo/actions/runs/42".into(),
        origin: origin.into(),
        assurance: AssuranceLevel::ProviderVerified,
        collected_at: "2026-10-14T10:00:00Z".into(),
        digest: Digest::sha256_bytes(raw),
        validity: EvidenceValidity::Valid,
        raw_evidence_ref: ".ai/evidence/external/run-42.json".into(),
    }
}

fn receipt_path(root: &Path, digest: &Digest) -> std::path::PathBuf {
    root.join(".ai/evidence/external").join(format!(
        "{WORK_ITEM_ID}.{}.delegated.json",
        digest
            .as_str()
            .strip_prefix("sha256:")
            .expect("sha256 digest")
    ))
}

#[test]
fn same_raw_digest_source_correction_appends_and_selects_without_replacing_legacy_bytes() {
    let root = repository();
    let raw = br#"{"run":42,"conclusion":"success"}"#;
    let original = evidence(
        raw,
        "https://api.github.com/repos/example/repo/actions/runs/41",
    );
    let legacy = import_delegated_evidence(root.path(), WORK_ITEM_ID, &original, raw, &runtime())
        .expect("bind original legacy receipt");
    let raw_path = root.path().join(&original.raw_evidence_ref);
    let legacy_receipt_path = receipt_path(root.path(), &original.digest);
    let raw_before = fs::read(&raw_path).expect("original raw bytes");
    let legacy_before = fs::read(&legacy_receipt_path).expect("original receipt bytes");

    let first = import_delegated_evidence_observation(
        root.path(),
        WORK_ITEM_ID,
        &original,
        raw,
        &runtime(),
    )
    .expect("append original observation");
    let corrected = evidence(
        raw,
        "https://api.github.com/repos/example/repo/actions/runs/42",
    );
    let corrected_observation = import_delegated_evidence_observation(
        root.path(),
        WORK_ITEM_ID,
        &corrected,
        raw,
        &runtime(),
    )
    .expect("append corrected source observation with same raw digest");
    let repeated = import_delegated_evidence_observation(
        root.path(),
        WORK_ITEM_ID,
        &corrected,
        raw,
        &runtime(),
    )
    .expect("repeated corrected observation is idempotent");

    assert_eq!(
        legacy.evidence.digest,
        corrected_observation.receipt.evidence.digest
    );
    assert_ne!(
        first.selector.observation_digest, corrected_observation.selector.observation_digest,
        "source metadata correction creates a distinct observation identity"
    );
    assert_eq!(
        corrected_observation.selector, repeated.selector,
        "the same complete source binding resolves to the exact stored observation"
    );
    assert_eq!(
        corrected_observation.receipt.bound_at,
        repeated.receipt.bound_at
    );
    assert_eq!(
        fs::read(&raw_path).expect("raw evidence remains readable"),
        raw_before,
        "same-digest observation must not rewrite raw evidence"
    );
    assert_eq!(
        fs::read(&legacy_receipt_path).expect("legacy receipt remains readable"),
        legacy_before,
        "new observation must not rewrite the legacy receipt"
    );
    let observations = list_delegated_evidence_observations(root.path(), WORK_ITEM_ID)
        .expect("list both append-only observations");
    assert_eq!(observations.len(), 2);
    assert!(
        observations
            .iter()
            .any(|item| item.selector == first.selector)
    );
    assert!(
        observations
            .iter()
            .any(|item| item.selector == corrected_observation.selector)
    );
    assert_eq!(
        list_delegated_evidence(root.path(), WORK_ITEM_ID).unwrap(),
        vec![legacy]
    );
}

#[test]
fn concurrent_identical_observations_share_one_immutable_selector() {
    let root = repository();
    let root_path = root.path().to_path_buf();
    let raw = br#"{"run":84,"conclusion":"success"}"#.to_vec();
    let evidence = evidence(
        &raw,
        "https://api.github.com/repos/example/repo/actions/runs/84",
    );
    let runtime = runtime();
    let workers = (0..4)
        .map(|_| {
            let root = root_path.clone();
            let raw = raw.clone();
            let evidence = evidence.clone();
            let runtime = runtime.clone();
            std::thread::spawn(move || {
                import_delegated_evidence_observation(
                    &root,
                    WORK_ITEM_ID,
                    &evidence,
                    &raw,
                    &runtime,
                )
                .expect("concurrent identical import")
                .selector
            })
        })
        .collect::<Vec<_>>();
    let selectors = workers
        .into_iter()
        .map(|worker| worker.join().expect("worker joins"))
        .collect::<Vec<_>>();
    assert!(selectors.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(
        list_delegated_evidence_observations(root.path(), WORK_ITEM_ID)
            .expect("one immutable observation")
            .len(),
        1
    );
}

#[test]
fn concurrent_conflicting_raw_bytes_cannot_replace_the_shared_evidence_path() {
    let root = repository();
    let root_path = root.path().to_path_buf();
    let first_raw = br#"{"run":86,"source":"first"}"#.to_vec();
    let second_raw = br#"{"run":86,"source":"second"}"#.to_vec();
    let first_evidence = evidence(
        &first_raw,
        "https://api.github.com/repos/example/repo/actions/runs/86",
    );
    let second_evidence = evidence(
        &second_raw,
        "https://api.github.com/repos/example/repo/actions/runs/86",
    );
    let runtime = runtime();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let workers = [
        (first_evidence, first_raw.clone()),
        (second_evidence, second_raw.clone()),
    ]
    .into_iter()
    .map(|(evidence, raw)| {
        let root = root_path.clone();
        let runtime = runtime.clone();
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            barrier.wait();
            import_delegated_evidence_observation(&root, WORK_ITEM_ID, &evidence, &raw, &runtime)
        })
    })
    .collect::<Vec<_>>();
    let outcomes = workers
        .into_iter()
        .map(|worker| worker.join().expect("worker joins"))
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes.iter().filter(|outcome| outcome.is_ok()).count(),
        1,
        "one conflicting raw write wins and the other fails closed"
    );
    let winner = if outcomes[0].is_ok() {
        first_raw.as_slice()
    } else {
        second_raw.as_slice()
    };
    assert_eq!(
        fs::read(root.path().join(".ai/evidence/external/run-42.json")).unwrap(),
        winner,
        "the losing writer cannot replace the complete winner"
    );
    assert_eq!(
        list_delegated_evidence_observations(root.path(), WORK_ITEM_ID)
            .expect("list winning observation")
            .len(),
        1
    );
}

#[test]
fn observation_binding_covers_runtime_protocol_identity() {
    let root = repository();
    let raw = br#"{"run":85}"#;
    let evidence = evidence(
        raw,
        "https://api.github.com/repos/example/repo/actions/runs/85",
    );
    let observation = import_delegated_evidence_observation(
        root.path(),
        WORK_ITEM_ID,
        &evidence,
        raw,
        &runtime(),
    )
    .expect("record observation");
    assert_eq!(
        observation.receipt.protocol_version,
        cockpit_protocol::PROTOCOL_VERSION
    );
    let mut altered = observation.receipt.clone();
    altered.protocol_version += 1;
    assert!(cockpit_protocol::validate_delegated_evidence_observation_receipt(&altered).is_err());
}

#[test]
fn tampered_raw_bytes_are_rejected_without_being_replaced_by_a_later_import() {
    let root = repository();
    let raw = br#"{"run":99}"#;
    let evidence = evidence(
        raw,
        "https://api.github.com/repos/example/repo/actions/runs/99",
    );
    let observation = import_delegated_evidence_observation(
        root.path(),
        WORK_ITEM_ID,
        &evidence,
        raw,
        &runtime(),
    )
    .expect("initial observation");
    let raw_path = root.path().join(&evidence.raw_evidence_ref);
    fs::write(&raw_path, b"tampered").expect("tamper fixture raw bytes");
    assert!(
        import_delegated_evidence_observation(
            root.path(),
            WORK_ITEM_ID,
            &evidence,
            raw,
            &runtime(),
        )
        .is_err(),
        "an existing raw path is immutable even when it no longer matches the receipt"
    );
    assert_eq!(fs::read(&raw_path).unwrap(), b"tampered");
    assert!(list_delegated_evidence_observations(root.path(), WORK_ITEM_ID).is_err());
    assert!(
        observation
            .selector
            .observation_digest
            .as_str()
            .starts_with("sha256:")
    );
}

#[cfg(unix)]
#[test]
fn unsafe_observation_directory_does_not_publish_raw_evidence_first() {
    let root = repository();
    let raw = br#"{"run":117}"#;
    let evidence = evidence(
        raw,
        "https://api.github.com/repos/example/repo/actions/runs/117",
    );
    let external = root.path().join(".ai/evidence/external");
    fs::create_dir_all(&external).expect("external evidence directory");
    let observations = root.path().join(".ai/evidence/observations");
    std::os::unix::fs::symlink(&external, &observations).expect("unsafe observations symlink");

    assert!(
        import_delegated_evidence_observation(
            root.path(),
            WORK_ITEM_ID,
            &evidence,
            raw,
            &runtime(),
        )
        .is_err(),
        "observation storage symlinks must fail closed"
    );
    assert!(
        !external.join("run-42.json").exists(),
        "a rejected observation must not publish raw evidence before validating its receipt path"
    );
}
