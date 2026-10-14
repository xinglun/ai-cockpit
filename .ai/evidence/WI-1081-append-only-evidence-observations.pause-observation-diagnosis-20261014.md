# WI-1081 pause assertion: observed failure mechanism

The original complete run failed at composition.rs:1905 because `attempt.passed` was false. It was not a timeout: the verification package exited 101 after 37903 ms. The assertion did not print the internal attempt fields, so the exact internal failure or PID of that historical run cannot be recovered retrospectively.

The composition source and test blobs remain identical to base `00461714bd43d3f6c42ede66491e01d22d475604`. No production code, test condition, or safety gate was changed during this diagnosis.

## Bounded, read-only capture experiments

The capture helper watches only the existing `reuse-pause-race` fixture state directory and retains original attempt JSON bytes via read-only file descriptors before test-owned temporary files disappear. It starts the installed Runtime, which runs the unchanged test binary. Every experiment uses the same source snapshot, writable process-local HOME, TMPDIR, isolated Cargo target, one Cargo build job, and `RUST_TEST_THREADS=2`.

1. Probe 1 ran the complete composition integration target: 49 tests passed. The final second attempt had a passed reused node and cleaned worktree.
2. Probe 2 ran only the pause test and launched an independent tool monitor after receiving the acceptance marker. It passed. Captured timestamps show that the tool monitor started about 1.2 seconds after the final successful attempt was already persisted, so it was not an overlapping failure experiment.
3. Probe 3 pre-armed a normal independent tool process, then forked its ordinary read-only monitor child when the capture helper observed the reused receipt had been accepted. The child performed a filesystem-capacity query, stayed alive for two seconds, and was reaped. No process permissions, dumpability, security configuration, or production observation rule was changed. The unchanged pause test failed at the same line 1905.

## Actual Probe 3 values

- Reuse was already accepted: node `reusable` had `reused=true`, `passed=true`, `spawned=false`, `exitCode=0`, `timedOut=false`.
- Attempt spawned zero verification processes and its reuse decision was `reuse`.
- Final attempt: `passed=false`, `executionOutcome=unknown`, `executionEvidenceComplete=false`, `cleanupDisposition=retained`.
- Failure: `verifier_process_state_unknown`, `phase=procfs.cwd`, `PermissionDenied`, `errno=13`, PID `2009914`, UID filter `1000`, with stable before/after process identity.
- Capture process user namespace: `user:[4026532373]`; independent monitor child: `user:[4026532369]`; both UID 1000.
- Reuse acceptance was observed at Unix ns `1791980621427820898`; monitor child started at `1791980621428941053`.
- Captured attempt: `composition-sha256:45fa11ecedaac9d592be8333f6263f69ecd3ca797a29db6f2a4afe4a73240039`.

The observer conservatively retained the worktree after it could not inspect a newly started process in another tool sandbox. The accepted cache record and admission synchronization remained valid. Therefore `attempt.passed` can fail after successful cache acceptance because of the final cleanup observation; the assertion failure alone does not establish a pause-lock race.

This is a confirmed mechanism reproducing the historical assertion, not proof of the exact historical PID/error. The historical failed receipt remains unchanged. The independent monitor child exited normally and was explicitly reaped.

## Existing-Contract validation path

Run the unchanged declared `cargo test --workspace --all-targets` through the installed Runtime, one worker, `RUST_TEST_THREADS=1`, and the writable process-local HOME. Keep all assertions and the pause test's concurrent runner/pause-writer threads. Monitor resource usage and progress inside the already-running parent process; avoid starting independent tool-sandbox processes during process-observation fixtures. No observer refactor or assertion weakening is included.

Only a new Runtime receipt covering all 13 workspace package nodes can establish complete local Contract coverage. Scoped diagnostic successes and the current partial green projection cannot replace it.
