---
author: AI Cockpit maintainers
title: "WI-379 — reference documentation batch 18"
description: "Compare the next ten pinned reference paths and publish bounded Rust-native reader routes."
workItemId: WI-379-reference-documentation-batch-18
audience: [maintainer, reviewer, adopter]
status: recovered
authority: human-authorized
lastVerifiedBy: WI-379-reference-documentation-batch-18
terminalDecision: .ai/decisions/WI-379-reference-documentation-batch-18.recovery.json
capabilityClaims: [reference_comparison, verification_reuse, intelligence, lifecycle_closure]
---

# WI-379 — reference documentation batch 18

[简体中文](WI-379-reference-documentation-batch-18.zh-CN.md) · [日本語](WI-379-reference-documentation-batch-18.ja.md)

## Intent

Compare the next ten paths in the pinned reference inventory and make their
reader-facing governance meaning available through the shared Rust Runtime,
without copying source Python, Make, provider configuration, or historical
decisions.

The reviewed PR #343 delivered the bounded documentation, but this Work Item
was archived before the provider PR identity was known. Its archive, evidence,
Outcome, and pending resource context are immutable historical bytes. The
explicit recovery successor WI-380 completes provider finalization without
rewriting this record.

## Compared paths and decisions

| Pinned path | Decision |
| --- | --- |
| `docs/reference/upgrade.md` | `implemented-different-by-design`; expand the tri-language upgrade route with migration, backup/conflict, rollback, and explicit adapter boundaries. |
| `docs/reference/verification-evidence-reuse-runtime.md` | `implemented-different-by-design`; document typed receipt bindings, protected nodes, planner/adapter separation, and observable reuse. |
| `docs/reference/verification-evidence-reuse.md` | `implemented-different-by-design`; document freshness/invalidation and call-count evidence. |
| `docs/reference/verification-fixture-boundary.md` | `implemented-different-by-design`; document Rust fixture isolation and local-evidence limits. |
| `docs/reference/wi01-wi20-bidirectional-traceability-audit.json` | `reference-only`; generated historical V1 audit bytes are not target authority. |
| `docs/reference/wi01-wi20-bidirectional-traceability-audit.md` | `reference-only`; historical narrative remains source-bound and is not copied. |
| `docs/reference/wiii-v2-integration-audit.md` | `implemented-different-by-design`; document the narrower Rust read-only intelligence projection and identity checks. |
| `docs/reference/work-item-intelligence-performance-baseline.md` | `implemented-different-by-design`; document reproducible local observations without source-number or SLO claims. |
| `docs/reference/work-item-lifecycle-closure.ja.md` | `implemented-different-by-design`; provide a complete Rust-native tri-language closure route. |
| `docs/reference/work-item-lifecycle-closure.md` | `implemented-different-by-design`; provide the English route and explicit recovery boundary. |

## Boundary

This is semantic/documentation parity, not source command, JSON-wire, provider,
or generated-history parity. One installed Runtime serves many repositories via
explicit `--repo`; each repository keeps its own facts, Work Items, evidence,
knowledge, and snapshots. Documentation cannot create authority, approval,
assurance, or verification evidence.

## Acceptance

- Every selected path has one inventory classification and a counterpart or explicit reference-only reason.
- Reader-facing English, Simplified Chinese, and Japanese routes agree on links and semantic/non-wire boundaries.
- Inventory and parity ledgers record the same pinned source commit and batch decision with zero `migrate-gap`.
- Documentation, inventory, conformance, and installed Runtime checks pass without source fallback.
- Contract-language governance facts remain unchanged by presentation localization.

## Verification

The required repository-local checks are the inventory check, inventory-docs
test, inventory regression script, documentation acceptance, status-consistency
check, and `cargo test --locked --workspace`. The installed v0.2.39 Runtime is
used for preflight, checkpoint, verification, finish, archive, finalization,
and close; terminal receipts are added only after reviewed merge.
