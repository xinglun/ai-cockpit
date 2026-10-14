# WI-1081 composition diagnostic checkpoint

This Agent note preserves an additional failure; it does not claim complete workspace verification.

The full Work Item-bound run with a writable process-local HOME and `RUST_TEST_THREADS=2` completed all 13 package nodes in 457218 ms. The repository node passed. The verification node failed only in `pause_write_cannot_commit_between_reuse_admission_and_receipt_acceptance`, at the existing `attempt.passed` assertion on line 1905. Its original stdout and stderr were captured without truncation in `WI-1081-append-only-evidence-observations.full-verification-writable-home-20261014.runtime.log`; decoded failed-node logs and exit code 1 are retained separately.

The same test passed when run alone through Runtime with the same process configuration (one passing test, 48 filtered out). The complete `composition` integration target then passed all 49 tests through Runtime with `RUST_TEST_THREADS=1`, in 48554 ms. These diagnostic logs are complete. Serializing the test harness retains the test's own concurrent runner and pause-writer threads and every original assertion.

The composition implementation and integration test blobs are unchanged from base `00461714bd43d3f6c42ede66491e01d22d475604`: source blob `9ac7285e402f0813261df88359cbed86d5d5aea3`, test blob `750d88cf1b25cb7fddcd84e2e7df86fe9f3e2303`. No source or assertion was changed to make the retry pass. The exact cause of the earlier intermittent pause failure remains unresolved; concurrent process-observation fixture interference is a hypothesis, not a confirmed cause or a completed source fix.

The latest fresh full plan retains the original `cargo test --workspace --all-targets` declaration, one Runtime worker, 13 nodes to execute, and no authorized node reuse. A new full run under the serial test-harness configuration has not started at this checkpoint. The successful target-only receipt is preserved under a diagnostic filename; it must not be presented as complete coverage of the workspace declaration.

Contract remains `sha256:8f28e50938c1c594db450768be4ca14db02e46f0866c7316893f86e4c5bbb33b`; source snapshot remains `sha256:6ffcd0aaadd80bdba91dc6d7b70dc3baf588d76be913ba8dc89c5123f55d2dcc`. The original ENOSPC failure, confirmed read-only HOME failure, and intermittent composition assertion failure remain distinct facts.
