---
author: AI Cockpit maintainers
title: "WI-593 — WI-592 parity recovery revalidation"
description: "Re-deliver the missing parity registration through an append-only successor without rewriting WI-592 history."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-593-wi591-doc-parity-recovery
lastVerifiedBy: WI-593-wi591-doc-parity-recovery
terminalArchive: .ai/work-items/archive/WI-593-wi591-doc-parity-recovery.contract.json
terminalVerification: .ai/evidence/WI-593-wi591-doc-parity-recovery.verification.json
terminalFinalization: .ai/decisions/WI-593-wi591-doc-parity-recovery.finalize.f1adebd4711b32d6c621eb97a93219fe741ed770d738229ff4e65c712b470b4b.json
terminalDecision: .ai/decisions/WI-593-wi591-doc-parity-recovery.close.json
---

[简体中文](WI-593-wi591-doc-parity-recovery.zh-CN.md) · [日本語](WI-593-wi591-doc-parity-recovery.ja.md)

# WI-593 — WI-592 parity recovery revalidation

## Objective

Re-deliver the three-language parity registration identified by the WI-592
recovery decision and produce current verification evidence. WI-592 archive,
Contract, Summary, Outcome, and historical verification bytes remain immutable.

## Boundary

This successor changes only the three reference-parity projections and its
own Work Item documentation/evidence. Runtime behavior, release artifacts,
object repositories, global Agent/MCP configuration, and WI-592 immutable
bytes are out of scope.

## Acceptance

1. The English, Chinese, and Japanese parity gates pass on the latest reviewed
   `main` without rewriting WI-592 archive/evidence bytes.
2. The recovery decision remains bound to the WI-592 repository identity and
   immutable digests.
3. Verification and documentation outputs make no unsupported completion or
   governance claims.

## Verification

Run `cargo test --locked --workspace`,
`tests/docs/parity_status_check.sh`,
`python3 tests/docs/work_item_status_consistency.py --repo <repository>`, and
`tests/docs/documentation_acceptance.sh` with an explicit repository context.
