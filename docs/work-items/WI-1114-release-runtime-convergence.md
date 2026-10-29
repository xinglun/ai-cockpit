---
author: AI Cockpit maintainers
title: "WI-1114-release-runtime-convergence — Runtime release convergence"
description: "Converge Mac behavior, CI cost, public help, Sentinel acceptance and trustworthy verification-to-closeout evidence."
audience:
  - maintainer
  - reviewer
workItemId: WI-1114-release-runtime-convergence
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1114-release-runtime-convergence
---

# WI-1114 — Runtime release convergence

This Work Item has exactly five subtasks. The native Contract and current
Runtime projection govern scope, authority and admission; this page reports
progress and does not grant permission. WI-1113 was delivered to `main`, but
its native closeout remains pending. Its original evidence and recovery
resources remain intact; this Work Item does not silently retire it.

## Scope

1. **Mac:** implement and actually test the supported supervision boundary on
   macOS arm64. Cover ordinary exits, timeout, parent disconnection, protocol
   failures and detached pipe holders. Report limitations and uncertainty;
   keep the Linux and Windows guarantees.
2. **CI cost:** measure compilation, execution, evidence reuse and cleanup.
   Remove only proven duplicate work while retaining complete target coverage.
   Migrate scripts to Rust only where justified by a shared implementation and
   equivalent negative tests; a language change alone is not a speedup.
3. **Help:** derive the public command inventory from Clap, verify rendered
   short/long help and discovery descriptions, and keep the English, Japanese
   and Simplified Chinese references aligned. Private supervisors stay private.
4. **Sentinel:** after an explicitly approved formal publication, pin the same
   immutable package, binary and inspect identities in independent consumer
   copies. Assess #1004, #1041 and the related amendment-provenance regression
   separately, preserving original inputs and failures.
5. **Evidence and closeout:** persist real execution facts before later
   metadata/aggregation errors can erase them. Admit CI evidence only through
   explicit provider, source, Runtime and per-command equivalence checks;
   execute genuine gaps. Progress output is not evidence or authority.

## Verification and delivery order

Use small real positive/negative fixtures first, then independent review,
final source-bound checks and the required platform CI. A local module pass
does not establish complete verification or another platform's behavior.
Performance is measured against the comparable CI13 quality baseline of
4,701 seconds: the target is at least 30% reduction and no more than 55
minutes, with cache conditions and retained coverage disclosed. This target
has not yet been demonstrated.

The total quality CI budget is **8 runs including main**; the release-workflow
budget is **2 runs**. Used counts are currently **0/8** and **0/2**. Local
commands use one worker, at most 900 seconds each, protected finite process
cleanup and at least 1 GiB free disk. Preserve failed and uncertain results;
do not blindly retry or delete recovery evidence.

Implementation and source CI precede exact version/asset approval and formal
publication. Sentinel then validates the published immutable package; its
acceptance cannot be replaced by a candidate, plan-only result or AI Cockpit
self-test. The WI remains in progress while required consumer acceptance is
pending. `v1.1.0` is a proposal, not an approved tag or release.

## Current state and boundaries

Sequence 2 owner confirmation was recorded by the native Runtime and the
initial checkpoint completed. Local execution-checkpoint and help modules
have targeted evidence; whole-run interruption recovery, trusted CI admission,
Mac acceptance, measured speed and formal consumer acceptance remain pending.
No quality CI or release workflow has been started for this Work Item.

Current controls do not authorize publication, shared/production Runtime
replacement, irreversible deletion, early issue closure or a fabricated
finish/archive/close. Exact Runtime changes and native human decisions retain
their separate binding requirements. Any pre-archive documentation registration
is preparation only, not evidence that archive or close has occurred.
