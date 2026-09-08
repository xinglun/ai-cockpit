---
author: AI Cockpit maintainers
title: WI-650 — Blocking wait for verification child processes
description: Replace the fixed 10ms busy-poll wait for command completion with a blocking wait on Unix, with measured before/after evidence.
workItemId: WI-650-verification-wait-blocking
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-650-verification-wait-blocking
terminalArchive: .ai/work-items/archive/WI-650-verification-wait-blocking.contract.json
terminalVerification: .ai/evidence/WI-650-verification-wait-blocking.verification.json
terminalFinalization: .ai/decisions/WI-650-verification-wait-blocking.finalize.json
terminalDecision: .ai/decisions/WI-650-verification-wait-blocking.close.json
---

# WI-650 — Blocking wait for verification child processes

This Work Item is P1 of the AI Cockpit performance initiative: it removes a
fixed-interval busy-poll wait identified during P0 reconnaissance
(`crates/cockpit-verification/src/lib.rs`, in `execute_captured`'s child
process wait loop).

## Defect found and measured

The wait loop for every verification child process was:

```rust
let deadline = Instant::now() + Duration::from_secs(MAX_EXECUTION_SECONDS);
let (status, mut timed_out) = loop {
    match child.try_wait() {
        Ok(Some(status)) => break (Some(status), false),
        Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
        Ok(None) => { terminate_process_tree(&mut child, child_id); break (child.wait().ok(), true); }
        Err(_) => { terminate_process_tree(&mut child, child_id); break (None, false); }
    }
};
```

An isolated micro-benchmark (spawning `/bin/true` and `sleep 1` directly, in a
tight loop, outside the CLI, to remove unrelated startup/git/identity noise)
quantified the cost of this loop before changing any production code:

| scenario | busy-poll | blocking `wait()` |
|---|---|---|
| `true` (n=200) | mean 12.192 ms, p50 12.523 ms | mean 1.188 ms, p50 1.193 ms |
| `sleep 1` (n=5) | mean 1013.309 ms | mean 1009.251 ms |

For a near-instant command, the busy-poll loop added ~11 ms of pure waiting
(a ~10x latency inflation for that scenario); for a multi-second command, the
two mechanisms are statistically indistinguishable, exactly as the polling
interval's relative weight would predict.

## Fix

- `execute_captured`'s wait logic is extracted into `wait_for_child(child,
  child_id, deadline)`, with a `#[cfg(unix)]` implementation and an unchanged
  `#[cfg(windows)]` implementation.
- On Unix, the child is moved into a dedicated thread that calls the
  blocking `child.wait()` and sends the result over an `mpsc::sync_channel`;
  the caller does a single bounded `recv_timeout(remaining-until-deadline)`.
  On timeout, the process is killed by pid (`libc::kill`, requiring only the
  `Copy` pid, not the moved `Child` value) and descendants are terminated
  exactly as before, then the final exit status is still collected from the
  channel. This preserves the existing timeout, process-tree termination,
  and output-capture guarantees (output capture is unchanged — it already
  used a bounded `libc::poll`, not a busy sleep loop).
- On Windows, `wait_for_child` is textually identical to the previous loop.
  This platform is **not verified** by this Work Item (no Windows
  environment available); changing it without verification would be an
  unjustified risk, so it is deliberately left alone.
- `terminate_process_tree` (which needs `&mut Child`, incompatible with the
  Unix thread-owned `Child`) is now `#[cfg(windows)]`-only.

## Correctness evidence

- Three new unit tests (`crates/cockpit-verification/src/lib.rs`, `#[cfg(all(test,
  unix))] mod wait_for_child_tests`) call `wait_for_child` directly with a
  short synthetic deadline instead of the real 300s `MAX_EXECUTION_SECONDS`
  (impractical to exercise end-to-end): a command well under its deadline
  (not timed out, correct exit status), a command past a 100ms deadline
  (`sleep 5`, confirmed killed and reported `timed_out=true` in under 2s, not
  5s), and exit-code preservation (`sh -c 'exit 7'` reports code 7). All
  three run in ~0.1s total.
- The full existing `cockpit-verification` test suite (56 tests across 9
  files, including `bounded_execution_reports_plan_and_process_telemetry`,
  `detached_descendant_pipe_is_cancelled_and_fails_closed`, and other
  execution/timeout-adjacent tests) passes unchanged.
- `cargo test --locked --workspace` passes unchanged (120 test result blocks,
  0 failures).

## Measured performance (advisory)

On 2026-09-08, macOS arm64, `ai-cockpit verify --repo <fixture> --command
true` (a fresh-always custom command, never reused) against a small attached
fixture repository, 12 iterations, baseline (installed v0.2.87) vs. this
build:

- baseline: ~104-108 ms typical (excluding one 123 ms outlier)
- candidate: ~94-98 ms typical (excluding one 505 ms first-call outlier)

The ~10 ms end-to-end reduction matches the ~11 ms isolated micro-benchmark
finding closely, confirming the improvement is attributable to the removed
polling wait and not measurement noise. This is a local process-latency
observation, not a provider or enterprise guarantee.

## Out of scope

- The Windows wait path (unchanged, unverified here).
- Bounded parallel file read/hash (a separate P1 candidate; not pursued in
  this Work Item).
- Any change to output-stream capture, dependency scheduling, or resource
  budgeting.
