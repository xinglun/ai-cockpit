---
author: AI Cockpit maintainers
title: "WI-539 — Source governance checker comparison batch 36"
description: "Compare ten pinned reference governance checkers one by one and record their Rust-native or external boundaries."
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
workItemId: WI-539-reference-file-comparison-batch-36
lastVerifiedBy: WI-539-reference-file-comparison-batch-36
terminalArchive: .ai/work-items/archive/WI-539-reference-file-comparison-batch-36.contract.json
terminalVerification: .ai/evidence/WI-539-reference-file-comparison-batch-36.verification.json
terminalFinalization: .ai/decisions/WI-539-reference-file-comparison-batch-36.finalize.json
terminalDecision: .ai/decisions/WI-539-reference-file-comparison-batch-36.close.json
---

# WI-539 — Source governance checker comparison batch 36

## Objective

Read the next ten maintained reference checker modules at pinned source commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`, then record an evidence-backed
semantic classification for every current path. This is a parity and adopter
inheritance review, not a request to copy Python, Make, YAML, or source JSON
wire formats into the shared Rust Runtime.

## File-level result

| Reference path | Decision | Rust boundary |
| --- | --- | --- |
| `scripts/ai_check_guidelines.py` | `implemented-different-by-design` | Typed Contract guidelines remain human-owned; numbered acceptance/evidence bindings prove completion. No untyped `guidelinesCompliance` claim is inferred. |
| `scripts/ai_check_pr.py` | `implemented-different-by-design` | Archive, recovery, scope, and evidence checks are distributed across typed lifecycle gates; PR identity and hosted checks remain provider evidence. |
| `scripts/ai_check_reference_impact.py` | `reference-only` | Static AST/text impact scanning remains source/provider tooling. Rust operation-time scope checks are fail-closed but do not infer callers, external consumers, or monitoring. |
| `scripts/ai_check_registry.py` | `implemented-different-by-design` | Versioned gate manifests and typed receipts provide deterministic registration, deduplication, and explicit unavailable-gate reasons. |
| `scripts/ai_check_review_policy.py` | `implemented-different-by-design` | Contract/preflight and provider PR review carry authority; a second YAML policy or report-only focus list is not installed. |
| `scripts/ai_check_scope.py` | `implemented-different-by-design` | Repository-relative scope/out-of-scope, dependency, parallel-boundary, and snapshot checks are typed Runtime gates. |
| `scripts/ai_check_serial_order.py` | `implemented-different-by-design` | Predecessor, merged PR, closure, exact resource cleanup, and synchronized-base requirements are lifecycle and ready-on-base checks. |
| `scripts/ai_check_status.py` | `implemented-different-by-design` | Request-scoped typed status and human Outcome projections replace generated `current_status.md` as authority. |
| `scripts/ai_check_status_consistency.py` | `implemented-different-by-design` | Read-only status derives active/archive ownership and rejects ambiguity; Runtime has no silent generated-status repair authority. |
| `scripts/ai_check_summary.py` | `implemented-different-by-design` | Strict Contract, evidence, archive, and Outcome bindings cover the portable boundary without claiming source Summary JSON compatibility or inventing human claims. |

## Findings and adopter inheritance

No portable implementation omission was found. The reference-impact scanner is
explicitly `reference-only`, not a hidden Runtime gap: static callers and
external-consumer facts require adopter/provider or human-owned evidence, and
unknown impact remains fail-closed. The other nine responsibilities are
represented by typed Protocol, repository lifecycle, gate-manifest, status, and
Outcome boundaries.

Every attached object/adopter project inherits one shared Runtime with explicit
`--repo` binding, isolated Contract/evidence/knowledge, fail-closed lifecycle,
and human Outcome presentation. It does not inherit source checkers, provider
policy values, or stack-specific commands. Source and target JSON wire shapes
remain independent.

## Acceptance

- The inventory records exactly these ten current paths at the pinned source
  commit, with a non-empty reason and counterpart or explicit boundary.
- No selected path remains `deferred-next-batch` or `migrate-gap`; retired
  history remains append-only.
- English, Simplified Chinese, and Japanese comparison pages and this Work Item
  page state the same decisions and adopter boundary.
- Inventory, documentation, formatting, lint, and workspace verification checks
  pass before the Work Item is finished.
