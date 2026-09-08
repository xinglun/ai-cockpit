---
author: AI Cockpit maintainers
title: "WI-691 — P0 architecture responsibility map"
description: "Document the current responsibility and dependency boundaries for the architecture-optimization initiative."
audience: [contributor, maintainer, reviewer]
status: implemented
authority: human:repository-owner
workItemId: WI-691-p0-responsibility-map
lastVerifiedBy: WI-691-p0-responsibility-map
terminalArchive: .ai/work-items/archive/WI-691-p0-responsibility-map.contract.json
terminalVerification: .ai/evidence/WI-691-p0-responsibility-map.verification.json
terminalFinalization: .ai/decisions/WI-691-p0-responsibility-map.finalize.json
terminalDecision: .ai/decisions/WI-691-p0-responsibility-map.close.json
---

[简体中文](WI-691-p0-responsibility-map.zh-CN.md) · [日本語](WI-691-p0-responsibility-map.ja.md)

# WI-691 — P0 architecture responsibility map

## Intent

Create a current, cited map of observation, governance, lifecycle, evidence,
execution, persistence/recovery, and status/Outcome projection boundaries before
the follow-up architecture Work Items.

## Boundary

Documentation only. The Work Item does not change Rust source, tests, public
protocols, governance rules, `.ai/` records, or historical evidence. The map is
based on the latest remote default revision `99f7d2323ffb59b1e3edd6c1c833b00f50fb9698`.

## Acceptance

- The English, Simplified Chinese, and Japanese maps cite current CLI/MCP
  entry points and the observation, contract/policy/evidence, governance,
  lifecycle, execution, projection, and persistence/recovery call chains.
- Each responsibility names its authoritative facts, allowed I/O, owner of
  validation/decision/display, and reusable mechanisms.
- Concrete duplication and mixed responsibility are distinguished from open
  investigation questions; no atomic-snapshot or multi-file-transaction claim
  is inferred from a struct or a single-file rename.
- P1-B, P2-A, P2-B, P2-C, and P3 each have a bounded problem, target boundary,
  compatibility risk, and verification method.
- The three language pages and parity projections remain aligned, and no code
  or runtime behavior changes are included.

## Evidence and verification

Required Runtime evidence is the locked workspace test. Documentation acceptance,
parity status, `git diff --check`, and the hosted governance checks are additional
delivery evidence. The final status must be derived from Runtime archive,
verification, finalization, and close records rather than from this page.

## Findings recorded for follow-up

The current source already separates Git snapshots, typed protocol facts,
capability-scoped evidence storage, lifecycle locking, bounded execution, and
the final human renderer. The remaining questions are end-to-end observation
context ownership, the authoritative commit record for each multi-file
lifecycle operation, and the boundary between reusable verification and
physical execution. These questions are intentionally not implemented here.
