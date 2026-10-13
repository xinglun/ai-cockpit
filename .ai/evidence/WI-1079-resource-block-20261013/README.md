# WI-1079 resource failure recovery export

These are exact copies of diagnostic files, not signed Runtime receipts. Preserve the originals and all committed historical receipts. The empty formal-verification.json is the real zero-byte stdout from an exit-1 execution; it is deliberately not represented as a valid JSON verification result. export-manifest.json records byte hashes and known process exit codes. Local status and Outcome are historical snapshots, not admission in a new environment.

## GitHub identities

Branch: codex/wi1079-post-merge-lifecycle-layout. Prior accepted code/evidence head: 5d4818f814c2623e760e9ba30e3d41ff9bc33c1b. Production change: d5c88149331d4212793aaf02c90cb751b1208b5c. PR: https://github.com/xinglun/ai-cockpit/pull/1035. Full green CI: https://github.com/xinglun/ai-cockpit/actions/runs/38351446777 on the prior head, not on the commit adding this export. No source or test implementation is changed by this export.

Candidate SHA, current Contract/material binding and recovery instructions are in recovery-checkpoint-reviewed.txt. The source and scoped material receipt are already in the remote branch. The old candidate executable remains in the preserved environment; rebuilding elsewhere must report its actual hash and must not claim byte identity without checking. Historical approvals remain evidence of the specific recorded decisions, not blanket new-environment approval.

## Actual failure and diagnosis

Serial Runtime verify with unchanged candidate and Contract: 11 nodes reported passing, repository node failed; Git snapshot observation then failed to create threaded lstat. No new canonical attempt persisted. Same-environment repository package diagnosis independently failed with explicit thread creation errno11. Resource observations show zombies reparented to PID1 but do not prove the precise limiting quota or original parent chain. Time-window process attribution is explicitly bounded in the diagnostic JSON. No tests, system limits, permissions, source assertions or test concurrency were changed. No active verifier remains.

A normal Git status also failed. A later command-local `git -c index.threads=1 status --short` succeeded and reported clean before adding this export. This is only an indexing-operation workaround, not recovery of the test execution environment; it does not write global config or alter test concurrency.

## Resume

Clone the exact branch and preserve this export and existing .ai history. Read AGENTS and relevant guides, obtain a supported Runtime and report actual identity, inspect status/environment drift and re-admit through supported controls. Do not copy this status/Outcome into new facts or replace failed evidence with CI results. Run only the admitted required formal verification after environment readiness. Accepted verification precedes finish/archive. Merge, resource deletion and real human close remain separate authorization boundaries. WI-1078/PR1034 waits for integration of this repair.

Credentials/private-key/email scanning found no matches in the exported files. Resource JSON contains only selected process identities and counters, not raw command lines or environment variables. The old environment is retained; no restart, migration, batch kill or system-limit change was performed.
