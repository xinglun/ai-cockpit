# Separate tests-only plan: strict procfs fail-closed diagnostic envelopes

Status: proposed only; no new Work Item, branch, worktree, or source edit was made. The Ray-approved WI-1081 recovery does not authorize this implementation.

## Evidence and scope

CI run 38594384692 on source/evidence head 3e53201e failed quality; route, oracle and Windows passed. Parent Raydot independently read the exact failure: composition_attempt_reports_execution_and_cleanup_separately calls the cwd-only classifier at tests/composition.rs:1766, but the observer reported procfs.fd-directory PermissionDenied/EACCES 13, with its after-stat observation failing ESRCH 3. Production correctly returns Unknown/Retained. This environment independently queried the failed CI status, read the current classifier and producer formatting, and attempted to download the diagnostics artifact; the download returned HTTP 403. It has not claimed an independent raw-artifact read. Preserve original CI artifact and failure; no repeated rerun to obtain a green result.

Proposed Work Item: WI-1082-procfs-fail-closed-test-envelope (ID not reserved; check availability at entry).

Exactly two file scopes:

1. crates/cockpit-verification/tests/composition.rs — extend the test's narrow diagnostic classifier and synthetic positive/negative coverage. Keep the existing stable-cwd-EACCES branch and all execution/cleanup/result assertions.
2. crates/cockpit-verification/src/composition.rs — only its existing #[cfg(test)] module. Add a producer-side regression for the exact fd-directory EACCES / after-stat ESRCH diagnostic and conservative result. No production functions or type definitions may change.

## Minimal behavior and acceptance

- Existing stable procfs.cwd EACCES 13 with matching pid/starttime/pgid/sid stays accepted; state-only changes stay allowed. Existing identity-change negatives remain.
- Add only the observed fd-directory EACCES 13 / observed before-identity / after-stat ESRCH 3 envelope. Validate the prefix, phase, exact error-kind/errno, UID filter, parseable before identity and matching PID, exact after-stat ESRCH observation, and message. Never treat this as a stable identity or a successful cleanup.
- Keep attempt.passed=false, executionOutcome=unknown, execution_evidence_complete=false, owned_tree_termination_unknown=true, cleanupDisposition=retained, cleanup absent, and actual worktree present/registered.
- Wrong errno (including 130/5), wrong phase, missing fields, malformed before identity, PID mismatch, wrong after-stat error, changed observed identity, unrelated Unknown reasons and broad "PermissionDenied" substring matches must reject. Unknown/retained may not become passed/cleaned.
- Unit producer regression pins the complete diagnostic and fail-closed result. External classifier tests pin both accepted envelopes and every rejection above. Normal passed/cleaned path and cache/reuse assertions stay unchanged. Unix/Linux guards and Windows behavior stay unchanged.
- No production observer change, skipped test, relaxed aggregate gate, permission/dumpability change, or catch-all flaky-test acceptance.

## Legal entry and bounded execution

Use a separate ordinary implementation Work Item on a fresh clean branch/worktree at a verified default-branch base, after Raydot approves this separate scope and actual Runtime entry admits it. First fetch normally, resolve/pin the real base and confirm the two relevant file blobs. Read Runtime inspect/status/doctor and existing active/unclosed scope/resource bindings. A global readyOnBase flag alone does not prove new-entry admission. If entry reports a real predecessor/scope/resource conflict, report that exact binding; do not import the WI-1081 retry/human authorization into this Work Item or create a successor without its own required authority.

Proposed ordinary scaffold entry (not executed): ai-cockpit work-item new --repo <new-clean-worktree> --id WI-1082-procfs-fail-closed-test-envelope --mode implementation. Bind an explicit two-file tests-only Contract through supported Runtime operations before source changes. Preserve all original WI-1078/WI-1080/WI-1081 worktrees, dirty files, archives and evidence.

Implement meaningful RED tests against the current classifier first, then the minimal test-only change, then run exact targeted tests and the full declared verification through Runtime under fresh admission. One build job, isolated target, writable HOME/TMPDIR, 900-second host bound. Do not start independent tool-sandbox monitors while procfs observer fixtures run. Preserve every failure and confirm both files contain only the approved test changes before commit/push. New Work Item/Contract/source bindings require their own formal receipt; old WI-1081 local or hosted successes are not its acceptance.
