# WI-1081 corrected-source formal verification

This is the corrected-source verification checkpoint after Raydot accepted the two P2 fixes and the retained-root observation fix. It does not finish, archive, merge, close, or clean up any Work Item or resource.

## Exact bindings

- Tested Git head: `edc4fbfa7b9ce92a7f6858f7f04998963334f3e2`.
- Worktree: `/workspace/ai-cockpit-wi1081-append-only-evidence-observations`, retained in the existing Git common directory.
- Base: `00461714bd43d3f6c42ede66491e01d22d475604`.
- Contract: `sha256:8f28e50938c1c594db450768be4ca14db02e46f0866c7316893f86e4c5bbb33b`.
- Corrected source snapshot: `sha256:1df671d690e6b41ba964633198718f2d06d36f63cced491d4af9ebb7aa85cdc1`.
- Material manifest: `sha256:473b910ab8f7ca67167ef10ba8b7e9f8372bcb31ddaf4d0136a20ad70dfb30f4`.
- Installed Runtime: `/tmp/wi1079-toolchain/runtime/ai-cockpit`, version 1.0.1, SHA-256 `5cca63a1b051da37b325b7d123842b6a4f7302f25cd235c0d22c12450c6b264e`.
- Formal receipt digest: `sha256:543c87888662d72e04d38f7dfe1223b713fda41a0f9da043befc0e0a47a83439`.
- Canonical receipt raw-file SHA-256: `0db7600f62320951278d52711f384b417acbf109cc8fa9346b7c6e9a2db7c368`.

## Invocation and result

The repository-bound Runtime admitted `run_verification`, with no blocker or required human decision. Its fresh plan declared `cargo test --workspace --all-targets`, with 13 package nodes and zero reused nodes. The actual command was:

```text
/tmp/wi1079-toolchain/runtime/ai-cockpit verify --repo /workspace/ai-cockpit-wi1081-append-only-evidence-observations --work-item WI-1081-append-only-evidence-observations --workers 1
```

The invocation attachment records the actual Rust/Cargo versions, binary hash, source head, exact environment, parent PID and user namespace. Build jobs remained one. The test harness used `RUST_TEST_THREADS=2`; all assertions, internal concurrency, Runtime checks and observer behavior remained unchanged. HOME and TMPDIR were process-local writable directories. The isolated Cargo target was `/tmp/wi1081-build-00461714-20261014`.

The parent started the Runtime at `2026-10-14T15:04:10.638562+00:00`; the canonical receipt was created at `2026-10-14T15:13:40Z`. All 13 nodes executed and passed, every exit code was zero, no execution timed out, there were 13 spawned verifier processes, zero spawn failures, and maximum verifier concurrency was one. Runtime elapsed time was 564847 ms; execution elapsed time was 562647 ms; the complete parent invocation took 570.626031918 seconds. The 900-second host cap was not reached. The normal package timeout was unchanged.

The coverage manifest, node identities, command digests, execution records, and returned Runtime output were checked against the canonical receipt. The repository node's stdout has Runtime's bounded-output truncation flag; its passed result, zero exit code, and complete command coverage are retained. The full returned Runtime log is preserved byte-for-byte, including its bounded output, without inventing uncaptured stdout.

Resource monitoring used log reads and `statvfs` inside the existing parent process. No independent tool-sandbox monitor was started while the formal run was active. Before starting, PID 1 was `tail`, with 10621 existing PPID-1 zombies, including 10538 Git zombies, and cgroup `pids.max` was `max`. Those are historical environment observations, not a proven cause of a failure. No system permissions or security configuration was changed, and no bulk process termination or process cleanup was performed. Recorded disk and inode availability remained sufficient.

## Preservation and current limits

The pre-review 13-node receipt remains under its original source snapshot and receipt digest. Its preserved raw SHA-256 is still `903e565659e21d17f3571a58574403377b9ef351cf7b4d8db50570179a75b8a9`. The P2 scoped GREEN receipt was preserved before the formal run; all original RED receipts, previous failures, and diagnostic attachments remain. This corrected-source receipt supplies the complete Contract coverage that those earlier scoped checks did not supply.

The post-run status reports verification `verified`, fresh evidence, no blockers, no required human decision, implementation active, closure open, and cleanup not started. Post-run validation still reports intent alignment unknown (`intent_alignment_missing`); status retains `repository_material_inspection_unavailable` and `user_visible_benefit_not_declared`. These are recorded without converting technical verification into human intent acceptance. WI-1078/WI-1080 history, original worktrees, and old uncommitted files remain preserved.

The evidence commit that saves this checkpoint will be a later Git head than the tested source head. It changes only `.ai` records and evidence; it must not be confused with another source verification or with the CI run for the earlier head. Draft PR #1037 remains the review destination.
