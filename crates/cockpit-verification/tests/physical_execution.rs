use cockpit_core::Digest;
use cockpit_verification::{
    ExecutionResult, PhysicalExecutionKey, PhysicalSingleFlightCoordinator, WorkItemEvidenceReceipt,
};
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicUsize, Ordering},
};

fn digest(fill: char) -> Digest {
    format!("sha256:{}", fill.to_string().repeat(64))
        .parse()
        .expect("digest")
}

fn key(snapshot: char) -> PhysicalExecutionKey {
    PhysicalExecutionKey::new(
        digest('a'),
        digest(snapshot),
        digest('c'),
        digest('d'),
        digest('e'),
        digest('f'),
    )
}

#[test]
fn one_physical_result_binds_distinct_work_item_receipts() {
    let coordinator = Arc::new(PhysicalSingleFlightCoordinator::default());
    let calls = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(3));
    let physical_key = key('b');
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..2 {
            let coordinator = Arc::clone(&coordinator);
            let calls = Arc::clone(&calls);
            let barrier = Arc::clone(&barrier);
            let physical_key = physical_key.clone();
            handles.push(scope.spawn(move || {
                barrier.wait();
                coordinator.execute(physical_key, |physical| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    ExecutionResult::new(&physical, true, digest('1'))
                })
            }));
        }
        barrier.wait();
        let first = handles.remove(0).join().expect("first").expect("result");
        let second = handles.remove(0).join().expect("second").expect("result");
        assert!(Arc::ptr_eq(&first, &second));

        let receipt_a = WorkItemEvidenceReceipt::bind("WI-A", &first).expect("receipt A");
        let receipt_b = WorkItemEvidenceReceipt::bind("WI-B", &second).expect("receipt B");
        assert_ne!(receipt_a.receipt_digest, receipt_b.receipt_digest);
        receipt_a
            .validate_for("WI-A", &physical_key.repository_id, &first)
            .expect("A binding");
        receipt_b
            .validate_for("WI-B", &physical_key.repository_id, &second)
            .expect("B binding");
        assert_eq!(first.physical_execution_id, second.physical_execution_id);
    });
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(coordinator.active_count().expect("count"), 0);
}

#[test]
fn cross_work_item_receipt_reference_is_rejected() {
    let physical = cockpit_verification::PhysicalExecutionKey::new(
        digest('a'),
        digest('b'),
        digest('c'),
        digest('d'),
        digest('e'),
        digest('f'),
    );
    let execution =
        cockpit_verification::PhysicalExecution::new(physical.clone()).expect("physical");
    let result = ExecutionResult::new(&execution, true, digest('1')).expect("result");
    let mut receipt = WorkItemEvidenceReceipt::bind("WI-A", &result).expect("receipt");
    receipt.work_item_id = "WI-B".into();
    assert!(
        receipt
            .validate_for("WI-B", &physical.repository_id, &result)
            .is_err()
    );
    assert!(
        receipt
            .validate_for("WI-A", &physical.repository_id, &result)
            .is_err()
    );
}

#[test]
fn physical_key_mismatch_cannot_coalesce_or_bind_foreign_repository() {
    let coordinator = PhysicalSingleFlightCoordinator::default();
    let first = coordinator
        .execute(key('a'), |physical| {
            ExecutionResult::new(&physical, true, digest('1'))
        })
        .expect("first result");
    let second = coordinator
        .execute(key('b'), |physical| {
            ExecutionResult::new(&physical, true, digest('1'))
        })
        .expect("second result");
    assert_ne!(first.physical_execution_id, second.physical_execution_id);

    let foreign_repository = digest('2');
    let receipt = WorkItemEvidenceReceipt::bind("WI-A", &first).expect("receipt");
    assert!(
        receipt
            .validate_for("WI-A", &foreign_repository, &first)
            .is_err()
    );
}

#[test]
fn tampered_execution_result_fails_closed() {
    let execution = cockpit_verification::PhysicalExecution::new(key('b')).expect("physical");
    let mut result = ExecutionResult::new(&execution, true, digest('1')).expect("result");
    result.passed = false;
    assert!(result.validate().is_err());
}

#[test]
fn coordinator_rejects_result_bound_to_a_different_physical_key() {
    let coordinator = PhysicalSingleFlightCoordinator::default();
    let requested = key('b');
    let foreign = key('c');
    let error = coordinator
        .execute(requested, |physical| {
            let foreign_execution = cockpit_verification::PhysicalExecution::new(foreign)
                .expect("foreign physical execution");
            let _ = physical;
            ExecutionResult::new(&foreign_execution, true, digest('1'))
        })
        .expect_err("foreign result must fail closed");
    assert_eq!(error, "physical_execution_result_identity_mismatch");
}
