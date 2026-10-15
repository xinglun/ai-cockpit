# Bounded P2 correction checkpoint

## Observed RED and correction

The real combination RED on012277c4 fails at the final lifecycle assertion (archived vs closed), after successful independent-checkout recovery, normal Native Deleted append, exact preservation of old close/Retained root/origin/receipt and successful finalization-chain verification. Production bytes were unchanged from7546c277 when RED ran. Its raw logs and source fingerprints were committed/pushed ina4680c25.

The correction stays in lifecycle.rs: read and bind the exact historical provider head named by the immutable close; validate current and historical provider/Contract identities before close-head diagnostics; retain the existing full chain resolver and protocol transition validator. A different latest head is allowed only for the single next Deleted transition from the close-bound Retained head, with approved original close and unchanged historical Runtime. Reuse full existing provider/Contract/repository-evidence/local-cleanup validation without recursively requesting a historical close projection. The original origin/receipt still agree on their exact captured facts. Permit only that resolved new transition outside an older origin's complete inventory; when a new source already contains the transition, its new origin binds the latest provider facts and complete inventory including that file. Arbitrary applicable additions and unsupported extensions remain invalid.

## Verification and honest limits

- Shared closeout unit regressions18/18 pass.
- Latest repository/CLI/MCP closeout integrations14/14 pass (6/6/2), including unchanged original bytes, legitimate extension, wrong Runtime/provider/repository/WI/Contract/predecessor/sequence, forks, non-Deleted successor, unrelated applicable decision, actual local resource contradiction, required receipt restoration, extended source recovery, missing captured transition and mutually matching truncated inventories.
- Strict cargo clippy --locked --workspace --all-targets --all-features -- -D warnings passes. Three MCP and one CLI needless_borrow issues were fixed without assertion/gate changes.
- cargo fmt --all -- --check and governance-integrity document gate pass.
- First compilation of the correction had E0609 (missing Result propagation); that raw failure remains. The first full closeout integration run found an existing diagnostic-order assertion; validation order was corrected without altering that assertion, and the subsequent run passes6/6. These are retained separately from the original behavioral RED.
- No full successful Native workspace receipt yet. The prior706.82-second12/13 run remains failed and retained, with the read-only HOME implicit-cache cause proven. The diagnosed task-local process HOME is used by subsequent checks, without global environment/security/permission changes. The next full run must use it, one worker, isolated target, debug info/incremental disabled and the900-second outer bound.
- All6 Contract scenarios remain unverified until required current formal verification and acceptance are complete. No finish/archive, merge/Ready, human close or resource cleanup is inferred from focused results.

## Storage and remote checkpoints

Ray's exact old incremental-cache authorization was executed without scope expansion; actual workspace availability increased637190144 bytes, with before/after df/df-i and full inventory retained in storage-recovery-20261015. The coordinator then requested a one-shot multi-GiB read-only batch proposal before more long work. No additional cache was deleted. The precise non-executable file proposal is in batch-cache-proposal-20261015/proposal.json and its manifest hash8e12a74c56ea839341532475e0fca0eeb77000c0f65552d482e9ddc9d336e539. Current WI1084 output and all Runtime/test/build-script executable files, dynamic libraries, sources/.ai/evidence and WI1078 resources are excluded.

A gh REST readback returned401 mid-query; its partial record is preserved. The existing authorized GitHub App identifiesxinglun and read back all25 cleanup/RED checkpoint files at exacta4680c25 with byte/SHA256 equality. Normal Git ls-remote also succeeds. No credential was read out, guessed or reconfigured. Use the existing Git transport for normal branch pushes and the App for exact readbacks/CI inspection.
