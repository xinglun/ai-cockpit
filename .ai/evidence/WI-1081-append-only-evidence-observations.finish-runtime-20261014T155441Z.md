# WI-1081 actual Runtime finish material-gate rejection

The real finish entry was called once on reviewed evidence head `29a6778d106c479be5e81ff2bf5de2a4c0f38f40`, after checking its 15 review-package files, source/Contract/verification bindings, installed Runtime hash, and current admission. The pre-action status admitted finish and required no human decision. Runtime doctor was compatible/ok and agent doctor verified.

The command was `/tmp/wi1079-toolchain/runtime/ai-cockpit finish --repo /workspace/ai-cockpit-wi1081-append-only-evidence-observations --id WI-1081-append-only-evidence-observations --language zh-CN`. It returned exit 1 in 2.484196945 seconds. Stdout is genuinely empty and is preserved as a zero-byte `.stdout.log`; stderr contains the full returned Chinese red Outcome and the exact error:

```text
finish is blocked by canonical material Unknowns: repository_material_inspection_unavailable
```

The Runtime generated `failedGate=finish.lifecycle`, `outcomeState=blocked`, a red active Outcome, and an append-only blocked event at `2026-10-14T15:54:45Z`. The phase remains checkpointed; no finish-ready transition or archive occurred. Post-action status is blocking (`governance_red`), verification unknown, evidence freshness `stale_or_invalid` for current action authorization, `humanDecisionRequired=false`, and no finish/archive admission. This is the actual gate result, not an inference from an Unknown alone.

The canonical full 13/13 verification receipt raw bytes remain `0db7600f62320951278d52711f384b417acbf109cc8fa9346b7c6e9a2db7c368`, with receipt digest `sha256:543c87888662d72e04d38f7dfe1223b713fda41a0f9da043befc0e0a47a83439`. Source snapshot `sha256:1df671d690e6b41ba964633198718f2d06d36f63cced491d4af9ebb7aa85cdc1` and Contract `sha256:8f28e50938c1c594db450768be4ca14db02e46f0866c7316893f86e4c5bbb33b` are unchanged. The prior technical verification and CI success are retained as true history; they do not authorize the current red lifecycle status.

## Precise material boundary

The canonical request remains `sha256:98843586541b509a2a5e354520f0bbe8a447595279fbfb01d37a76664c84aef6`, manifest `sha256:473b910ab8f7ca67167ef10ba8b7e9f8372bcb31ddaf4d0136a20ad70dfb30f4`, material source-tree digest `sha256:bbf6dc828843cf3264ee925c6bde0bca943428dd9c36e59f94059f58ab182f31`. The different formal snapshot digest is the normalized RepositorySnapshot envelope, not a stale material tree.

There is no material Finding. The single reviewable member is `crates/cockpit-repository/src/resource_lifecycle.rs`, cause `readable_committed_rust_syntax_unknown`, changed-hunk digest `sha256:52fab746486a2527c9b766a5d489c7150cb8030b5fa00fd050398bdc9bfce577`, after-blob digest `sha256:9f0afabe32d525c93daf29baa3c4993e2c8732ba14e8ee0012b4e76fe5314734`. The complete canonical request and machine-readable required-review bindings are preserved with the attempt.

Current `reviewEnabled=false`, because this Contract has neither `/governanceProfile/materialInspectionReview` nor required capability `material-inspection-review`; `record_material_review_decision` is not admitted. Its supported decision is typed `accept_permitted_unknowns`, with `assurance=self_declared`, a concrete `agent:Raydot`, authority source matching the Contract profile, exact current requestDigest, real evidence references/digests, rationale, and residual risk. A normal record action assigns its own Runtime timestamp and provenance. This is technical review, not human/provider/release approval.

A supported opt-in would require a real Contract amendment. That has not been made: the coordinator explicitly requested reporting source/Contract changes first. If Raydot chooses this path, regenerate its new canonical request and query real admission; do not reuse the old request/Contract approval or blindly apply the old full verification receipt to a new Contract. Existing user-benefit/intent-alignment/dependency-confidence Unknowns must remain truthful. The earlier Raydot formal-verification judgment is preserved, without pretending it already accepted this new canonical material-review decision.

No security configuration, safety gate, assertion, source, Contract, or generated record was hand-edited. No material-review receipt, human identity, or approval timestamp was fabricated. Archive, merge, resource deletion, and human close were not invoked. This checkpoint saves the failed real operation and necessary governance records for Raydot's next technical ruling.
