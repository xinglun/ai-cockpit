# Current local formal verification handoff

## Exact bindings

- Source/test commit: `07f005f793ded957cba8a6d676e3d0aef05630bd`; production correction: `e7b568b030a50cf42e883c79f3b0625072756961`. The subsequent checkpoint records evidence only.
- Contract: `sha256:507bedef85c7d3cb7ff2b8fd3a885d5441384ea23b8d6e4b010fa9be8c150107`; original raw Contract bytes unchanged. Ray's genuine bound confirm_review remains recorded; no repeat review/checkpoint or closure decision is invented.
- Candidate Runtime: `target/wi1084-closeout-receipt-integrity/debug/ai-cockpit`, SHA-256 `5307ba5324d397e0fb971aa8b926ba8176336a0a6e76dfae7bd3a6e5cd11fa38`; Rust/Cargo1.98.1. The actual build command, source hashes and process environment are in `07f-runtime-binding.json` and `post-cleanup-runtime-build.execution.json`.
- Formal receipt: `.ai/evidence/WI-1084-cross-checkout-closeout-receipt-integrity.verification.json`; raw SHA-256 `3e50cfff0224aaf40694b75be5d315364dcdf37ee1a0ba15913cd05ca9a19554`; receiptDigest `sha256:5028a26bff9c475816189737edeeda0ec873878a767787bebee030b4f1b53214`; repositorySnapshot `sha256:8a7631eab8bbec1e4f20e359ee54a942a93a94dcbe4ec96d1f412628e71e2c5e`.

## Observed results

Native executed the Contract workspace command as13 serial package nodes, all13 passed,13 spawned,0 reused,0 timeout/spawn failures,maxConcurrentProcesses1. Default per-node timeout remained600 seconds, with no rejected900-second override reused. The outer bound was900 seconds; actual outer elapsed645.2456 seconds. Runtime elapsed642300ms is separately recorded. The successful attempt53a38629... and canonical receipt are both preserved.

The current fresh Native status says verification=verified, evidenceFreshness=fresh, blockers=[], humanDecisionRequired=false. Six declared scenarios and intent alignment were recorded through supported work-item controls, using actual local evidence. Validation returns verified with no unknowns. This records local checks; Windows CI, human technical acceptance, archive, close and merge remain separate.

Strict workspace clippy(all targets/features,-D warnings), format and the governance document gate pass. Linux recovery regressions18/18 and repository/CLI/MCP integration checks14/14 remain preserved. The only post-e7 code change pins core.autocrlf=false on the second-hop test clone to preserve Windows checkout bytes; production validation and all assertions are unchanged. The exact a468 Windows113/114 failure remains in the raw log.

Original failed/rejected evidence is retained: actual receipt-only deletion RED; actual recovered-Retained-to-Deleted combination RED; compile/diagnostic intermediate failures; the706.82-second12/13 Native execution_failed attemptd1e1... caused by read-only HOME; and precondition-rejected timeout-override attempt1de752... with0 spawned processes. The new successful13/13 run uses process-local HOME=/tmp/wi1084-runtime-home-20261015 without global Git/environment/permission/security changes. Repository child stdout remains bounded/truncated by Native; every execution's exit0/pass/timeout state and complete stderr remain recorded.

Ray's exact cache approval was executed: only the2112 manifest-listed stable non-executable files were removed; no directories. All700 excluded entries and10 protected Runtime paths remain unchanged. Actual workspace available-byte increase2330488832, not the estimated allocation total. Full authority, identity checks, exclusions and raw df/df-i remain in ../batch-cache-cleanup-20261015.

## Handoff and limits

PR#1040 stays Draft. The source-head07f CI observation is recorded separately and must not stand in for any later evidence head. Automatic current-head required CI, including the real Windows library regression, must complete before technical acceptance. No manual rerun of historical green CI is requested.

Native human Outcome is green for declared verification, lifecyclecheckpointed. Its generic human-decision field remains unrecorded and user-visible benefit is undeclared; this does not erase the genuine preflight confirm_review or create close/merge authority. Usage coverage and host display acknowledgement remain unknown. Parent Raydot receives the handoff through the platform at turn completion; no separate external message is claimed.

No finish/archive, Ready/merge, human close, Task9, Sentinel change, branch/worktree deletion, WI-1078 change or additional cache deletion was performed. Review of the full current-head evidence is the next boundary.
