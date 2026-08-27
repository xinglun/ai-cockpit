---
author: AI Cockpit maintainers
title: "WI-330 — capability-truth boundary decision"
workItemId: WI-330-capability-truth-boundary
description: "Close the file-level comparison of the reference capability claim, freshness, and truth-matrix documents without copying V1 assets."
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-330-capability-truth-boundary
terminalArchive: .ai/work-items/archive/WI-330-capability-truth-boundary.contract.json
terminalVerification: .ai/evidence/WI-330-capability-truth-boundary.verification.json
terminalFinalization: .ai/decisions/WI-330-capability-truth-boundary.finalize.0c1ecf840859c3ce2fda21da34d25e8e742386d4d8de7674ade851d217dcdcdc.json
terminalDecision: .ai/decisions/WI-330-capability-truth-boundary.close.json
capabilityClaims:
  - reference_parity
---

# WI-330 — capability-truth boundary decision

## Intent and boundary

This Work Item re-reads four pinned reference files at commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` and closes their semantic
comparison. The target Rust Runtime remains a repository-governance layer;
it does not copy the reference Python checker, source matrix bytes, or V1
runtime state.

## File-level decision

| Pinned source path | Classification | Target responsibility |
| --- | --- | --- |
| `docs/reference/capability-claim-authoring.md` | `reference-only` | Target documentation metadata is descriptive. `capability show` and the repository capability registry report observed, bound facts; they do not authorize public wording through lexical triggers. |
| `docs/reference/capability-evidence-freshness.md` | `reference-only` | Work Item verification receipts have identity/freshness checks. The source Capability Truth row expiry and portable-environment policy are not a current Runtime feature. |
| `docs/reference/capability-truth-matrix.json` | `reference-only` | The source 30-row matrix is not a Rust wire format or authorization source. Target capability truth is request-scoped, repository/snapshot-bound, and explicit about adopter and external exclusions. |
| `docs/reference/capability-truth-matrix.md` | `reference-only` | Target capability/adoption pages explain observed facts, repository evidence, adopter installation, delegated provider evidence, and enterprise boundaries without advertising the source matrix/checker. |

These are explicit product boundaries, not untracked omissions. A future
claim-binding or row-freshness feature would require a separately
human-owned Work Item defining Rust-native schemas, evidence generation,
stale handling, multilingual scope, and adopter acceptance.

## Acceptance

1. Each of the four pinned paths has an explicit classification, counterpart,
   and reason in the inventory and tri-language comparison pages.
2. The tri-language comparison and parity pages state the same non-copy and
   non-authorization boundary; the existing capability index remains outside
   this documentation-only scope.
3. No source Python script, source matrix JSON, V1 state, global Agent/MCP
   configuration, or unsupported capability claim is added.
4. The inventory and documentation gates, Runtime verification, reviewed PR,
   merge, finalization, close, and exact cleanup pass.

[简体中文](WI-330-capability-truth-boundary.zh-CN.md) · [日本語](WI-330-capability-truth-boundary.ja.md)
