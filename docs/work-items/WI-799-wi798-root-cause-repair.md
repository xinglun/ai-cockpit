---
author: AI Cockpit maintainers
title: "WI-799 — WI-798 root-cause repair"
description: "Repair the observed collaboration, comparison-diff, scope, governance, and recovery-loop root causes before release."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized-root-cause-repair-and-release
workItemId: WI-799-wi798-root-cause-repair
lastVerifiedBy: WI-799-wi798-root-cause-repair
terminalArchive: .ai/work-items/archive/WI-799-wi798-root-cause-repair.contract.json
terminalVerification: .ai/evidence/WI-799-wi798-root-cause-repair.verification.json
terminalFinalization: .ai/decisions/WI-799-wi798-root-cause-repair.finalize.json
terminalDecision: .ai/decisions/WI-799-wi798-root-cause-repair.close.json
predecessorWorkItemId: WI-798-collaboration-observability-performance
recoveryDecision: .ai/decisions/WI-798-collaboration-observability-performance.recovery.7f58122c0b99798e61e08bed7f3f3c3ccf8dd4da8dd8dffebd1b4f7be68fd96c.json
---

[简体中文](WI-799-wi798-root-cause-repair.zh-CN.md) · [日本語](WI-799-wi798-root-cause-repair.ja.md)

# WI-799 — WI-798 root-cause repair

## Recovery boundary

WI-799 is the bounded successor of WI-798. WI-798 remains immutable historical
evidence; this Work Item repairs the root causes found while revalidating its
implementation and release path. It does not rewrite predecessor records or
turn a technical retry into an unrelated successor.

## Root-cause boundary

The current repair covers structured CLI failure reporting, amendment
revalidation, governance and parity registration, committed PR comparison
facts on clean worktrees, and shared Contract scope-glob semantics. The
comparison gate now binds the Contract baseline, CI comparison baseline, and
actual committed HEAD diff separately, and the Runtime-owned governance paths
are declared explicitly in the Contract.

## Acceptance

- Cheap deterministic preconditions identify ordering, scope, stale evidence,
  and governance-registration failures before expensive verification.
- Failed verification returns structured evidence and a non-zero exit without
  pretending that failure is completion evidence.
- A clean hosted-style checkout observes the committed comparison diff,
  including changed tests and governance records.
- Contract scope patterns and their evaluator use the same supported semantics,
  with a regression proving filename globs do not cross directories.
- English, Simplified Chinese, and Japanese documentation and parity rows bind
  the same predecessor, evidence, and future terminal lifecycle.
- The reviewed branch passes the strict local route before any provider CI or
  release action is resumed.

## Verification boundary

- current Contract: `.ai/work-items/active/WI-799-wi798-root-cause-repair.contract.json`
- verification: `.ai/evidence/WI-799-wi798-root-cause-repair.verification.json`
- planned terminal archive: `.ai/work-items/archive/WI-799-wi798-root-cause-repair.contract.json`
- planned terminal verification: `.ai/evidence/WI-799-wi798-root-cause-repair.verification.json`
- planned finalization: `.ai/decisions/WI-799-wi798-root-cause-repair.finalize.json`
- planned close: `.ai/decisions/WI-799-wi798-root-cause-repair.close.json`

The three language pages and parity rows preserve the same facts. Provider CI,
review, merge, publication, and public-artifact acceptance remain pending
until their own evidence exists.
