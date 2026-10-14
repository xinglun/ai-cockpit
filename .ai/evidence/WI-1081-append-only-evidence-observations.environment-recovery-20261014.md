# WI-1081 verification environment recovery

This is an Agent diagnostic note. Runtime receipts and original logs remain authoritative.

Source implementation commit: `8f1a56004a63c24b35f5aee14ad1f6701315516d`.
Published failure checkpoint: `148e2c0f582427d333344bf3a0f55265bd9ce21c`.
Contract: `sha256:8f28e50938c1c594db450768be4ca14db02e46f0866c7316893f86e4c5bbb33b`.
Repository source snapshot: `sha256:6ffcd0aaadd80bdba91dc6d7b70dc3baf588d76be913ba8dc89c5123f55d2dcc`.
Runtime: `/tmp/wi1079-toolchain/runtime/ai-cockpit`, version `1.0.1`, digest `sha256:5cca63a1b051da37b325b7d123842b6a4f7302f25cd235c0d22c12450c6b264e`.

## Distinct preserved failures

- Attempt `14f602c5db16aaddffeee70c4bbdcad9840dd2d730e88ec425dc01d91e5022df` contains actual `/tmp` storage exhaustion (`ENOSPC`).
- Attempt `5e4bfb9cf3f49d6fcc72ad7029da197d15bf74938140f953a85cab14cb058d7c` completed 13 nodes with 12 passing. The repository node failed in `verification_service`; its bounded stdout was truncated. The exact failing test was not recoverable from that receipt alone.
- A fresh, Work Item-bound Runtime diagnostic ran only `cargo test --package cockpit-repository --test verification_service -- --nocapture`. It preserved complete stdout/stderr and identified `linked_worktree_cargo_test_binary_embeds_its_own_manifest_directory`, failing at the existing `run.receipt.passed` assertion. The other 16 tests passed.

## Cause and controlled recovery

The existing test intentionally removes inherited `CARGO_TARGET_DIR` to check implicit target isolation. The current execution environment supplies `HOME=/home/agent`, whose cache directory is read-only in this sandbox. Re-running the same retained tiny Cargo fixture through Runtime, with `CARGO_TARGET_DIR` unset, returned:

```text
error: failed to create directory `/home/agent/.cache/ai-cockpit-verify-target`
Caused by: Read-only file system (os error 30)
```

Changing only the verification process's HOME to `/workspace/.tmp-wi1081-runtime-home-20261014` made that same fixture pass. This is a process-local writable home for test caches; it changes no system permissions or global configuration. No production source, test assertion, or Runtime gate was changed.

The Work Item-bound `verification_service` diagnostic then passed all 17 tests, with complete stdout/stderr and exit code 0. Its exact generated receipt is preserved as `WI-1081-append-only-evidence-observations.verification-service-writable-home-20261014.receipt.json`. This target-only result does not establish complete coverage of the Contract's workspace declaration.

The fresh full plan retains `cargo test --workspace --all-targets`, 13 package nodes, one worker, and no reused nodes (`reuse_not_configured`). Complete declared verification is pending at this diagnostic checkpoint.

## Verification process configuration

- `PATH` prepends `/workspace/.cargo/bin`; Rustup home is `/workspace/.rustup`; Cargo home is `/workspace/.cargo`.
- `CARGO_TARGET_DIR=/tmp/wi1081-build-00461714-20261014`, retained from previous attempts.
- `TMPDIR=/workspace/.tmp-wi1081-tests-20261014`; process-local `HOME=/workspace/.tmp-wi1081-runtime-home-20261014`.
- `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `RUST_TEST_THREADS=2`.
- No explicit timeout override; Runtime's finite package-test default is 600 seconds, within the authorized 900-second upper limit.
- Cross-Work-Item coordination inspection contains no registrations. No shared provider environment record or registration was invented for this serial Work Item.

The original failed receipts, raw Runtime output, decoded stdout/stderr bytes, and exit codes are retained beside this note. The earlier disk exhaustion and the independently confirmed read-only HOME failure must remain distinct historical facts.
