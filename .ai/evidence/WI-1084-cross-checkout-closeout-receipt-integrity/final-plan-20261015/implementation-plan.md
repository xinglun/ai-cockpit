# WI-1084 final implementation plan

Plan only. Raydot's independent technical review passed with no P1/P2 findings. Real policyReview is pending. No implementation code or regression test is written/run; all Contract scenarios remain unverified. The Contract is frozen at sha256:507bedef85c7d3cb7ff2b8fd3a885d5441384ea23b8d6e4b010fa9be8c150107 unless a genuinely new scope decision becomes necessary. checkpointCount remains1; do not repeat checkpoint.

## Small acyclic origin/receipt binding

Use the existing repository-private recovery writer and shared close validator. No public command, protocol-wide framework, provider client, TTL or system change.

New origin file: `.ai/decisions/<id>.cross-checkout-closeout-origin.json`.
- schemaVersion=1, kind=cross_checkout_closeout_origin.
- workItemId, sourceRepositoryId, destinationRepositoryId.
- canonical contractDigest (the existing recovery canonical Contract digest, distinct from file/raw digests below).
- sourceSnapshotDigest, providerFacts, runtimeVersion and runtimeDigest from the validated recovery plan/current writer.
- requiredRecoveryReceipt={path: canonical `.ai/decisions/<id>.cross-checkout-closeout-recovery.json`, schemaVersion:2}.
- closeDecision={path: canonical same-WI close file, digest: SHA-256 of exact historical close bytes, sizeBytes}; it must equal exactly one full payload entry.
- fileDigests: the complete sorted list from the existing validated plan.files, including exact close, archive manifest, raw archived Contract, Summary, Outcome, verification, every manifest-required payload and applicable finalization/recovery decision. Each path is unique, scoped, regular and non-symlink; each raw digest and size must agree. Verify archive completeness; two mutually agreeing truncated lists do not discharge the archive requirements.

New required receipt keeps its canonical existing path and kind, uses schemaVersion=2 and retains current source/destination/Contract/snapshot/provider/Runtime/fileDigests fields. Add originBinding={path: exact same-WI origin path, digest: SHA-256 of the exact persisted origin bytes}. Both records' stable bindings must agree exactly.

DAG: receipt -> origin -> immutable source/close/payload facts. Origin declares the required receipt's path/version but never embeds its digest, so no cyclic content hash is created. Origin has no mutable wall-clock field: serialization is deterministic for the same validated plan. Existing receipt createdAt and destinationSnapshotDigest remain honest receipt metadata under their current checks; neither is elevated to a new TTL or silently backdated. Stable source/provider/Contract/Runtime/payload facts are the origin's identity boundary.

## Presence and compatibility rules

| Observed records | Required result |
| --- | --- |
| Valid new origin + matching v2 receipt + complete exact payload | Preserve current accepted close projection. |
| Origin exists, receipt absent/wrong version/wrong path/invalid | Reject closed; no ordinary or v1 fallback. |
| V2 receipt exists, origin absent/invalid/wrong path/digest | Reject closed; no ordinary or v1 fallback. |
| Either required record is malformed/non-regular, or WI/repository/Contract/source/provider/Runtime/close/payload binding differs | Reject closed with existing close-integrity diagnostics or a narrowly scoped precise diagnostic. |
| No origin and ordinary current/legacy close | Preserve prior ordinary-close semantics. |
| Existing v1 receipt without origin | Preserve existing legacy validation/retry semantics and immutable bytes; do not silently migrate or manufacture provenance. If that old sole receipt was already removed, its former recovery origin is not inferable. |

If both provenance and all dependent proof are removed, an indistinguishable legacy close has no independent trusted anchor. Do not claim general evidence-deletion detection. The bounded fix proves sole required receipt loss after a newly bound recovery. For a destination already containing legacy v1 receipt, retain existing same-byte retry/conflict behavior; never overwrite it with v2 or attach a v2-required origin to an unchanged v1 receipt.

## Writer, retry and interruption order

1. Hold the existing selected-WI lifecycle lock; build and validate the existing exact source recovery plan.
2. Install non-close historical payload with the current immutable installer; reject conflicts without replacing/unlinking published paths.
3. Install/validate the deterministic origin, then the v2 receipt referring to its actual digest. Retain separate private hook events after origin and after receipt; reuse existing before-close and after-close events.
4. Refresh source and destination admission as today; immediately recheck every required payload, origin and receipt before installing the exact original close bytes as completion marker.
5. Read current status after the commit point and compare source historical projection without promoting verification.
6. On interruption preserve exact published paths and report incompleteness or postcommit uncertainty. Supported retry restores a missing origin or receipt only from the same validated source facts and same bound bytes; reject a conflicting existing file. Receipt re-creation records its real new creation time; do not fabricate the deleted timestamp. An already present valid complete pair remains idempotent, changedPaths=[].

A recovered checkout used as a later source must pass the shared close validator first. The next destination creates its own context-bound origin/receipt from the validated plan; do not import a destination-specific pair into conflicting canonical paths. Prove both source and prior proof bytes remain unchanged.

## RED and regression sequence (all pending)

- Before production changes, extend the existing independent-checkout fixture: successful Native recovery, exact closed/non-blocking baseline, remove only the required receipt, then query status. status exit0 remains allowed; require lifecyclePhase=archived, blocking=true and an integrity diagnostic. Preserve raw remaining payload bytes. Watch this assertion fail on the bcd76ae7 production source before fixing.
- Prove an overlapping WI cannot be falsely admitted after that deletion; distinguish the intended archived close-integrity barrier from unrelated dirty-tree/topology failures.
- Add origin-only/receipt-only, malformed, symlink, version downgrade, wrong path/digest/repository/WI/canonical Contract/source/provider/Runtime/close identity, omitted payload and archive completeness cases. No fallback to ordinary close or legacy v1 when required new provenance is present.
- Use existing private hooks for interruption immediately after origin, after receipt, before close and after close. With a fixture requiring import of the close marker, incomplete preclose states stay non-terminal; existing exact close files are never deleted merely to force the fixture model. Preserve current commit-point/reporting semantics and existing published-path/racing-target negatives.
- Add supported restore/retry, unchanged source, full raw historical-byte identity, legacy v1 compatibility and recovered-checkout-as-later-source cases.
- Add narrow CLI and MCP projection parity tests through the shared repository service. Preserve historical verification=not_ready and historical_evidence_not_revalidated with a distinct producer Runtime. No historical test is represented as rerun.
- Run the focused RED/GREEN tests, current recovery/idempotence/failure-preservation cases, Contract cargo test --locked --workspace and required repository gates after real policyReview and fresh action admission. Use an isolated target, one build task/worker, CARGO_INCREMENTAL=0, debug info disabled and the existing900-second verification cap. Preserve actual source/Runtime/toolchain/commands, counts, raw stdout/stderr and failed receipts. Do not weaken assertions or protected gates.

## Execution and governance

One serial WI-1084 implementer/test worker. No registration/lease/composition or parallel claim: formal compatibility remains false because WI-1070–1074 lack compatible boundaries/declarations. Their records are unchanged. Serialize shared documentation projections, PR integration and merges; keep PR#1040 Draft. No new-WI human close, deletion, Task9, Sentinel modification or release is authorized.

First prepare the real EN/zh-CN/ja WI plan pages and parity rows in the already approved scope. Keep status in_progress and scenarios unverified. Then fresh status must admit run_preflight; run the existing preflight to produce the final policy-review request. If another legitimate preparation action is recommended, inspect and satisfy only truthful plan/intent evidence; never mark scenario verification before tests.

The one human confirm_review must bind exact Work Item/repository, canonical Contract digest, preflightDecisionDigest and repositorySnapshotDigest. It is not a materialManifestDigest approval. No implementation before that real approval. When Contract stays unchanged, later ordinary source edits require fresh preflight/verification, not repeated human review of the same Contract. No second checkpoint.
