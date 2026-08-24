---
author: AI Cockpit maintainers
title: "WI-246 — Pending parity merge-ref recovery"
workItemId: WI-246-pending-parity-merge-ref-recovery
description: "Recover the WI-244 delivery and bind parity to decisions contributed by a hosted merge ref."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-249-parity-finalization-registration
authority: canonical
---

# WI-246 — Pending parity merge-ref recovery

WI-244 delivered the typed pending parity registry and reached an immutable
verified archive on PR #196. The push tree passed, but the hosted PR merge ref
also contained the authoritative WI-243 close receipt newly present on the
default branch. The three WI-243 rows still named only the pre-merge finalize
receipt, so the governance gate correctly failed the combined tree.

## Recovery boundary

- Runtime receipt `.ai/decisions/WI-244-pending-parity-registry.recovery.json`
  binds the exact predecessor Contract, Summary, Outcome, and Events digests.
- WI-244 archive, verification, finalization, PR #196, and hosted-run bytes are
  immutable. WI-246 projects them without rewriting the predecessor.
- The Contract base is `3fd982560ee28563bfab69d414f60575f3b2894a` from
  `origin/main`; recovery bootstrap commit `3a5693a` is governance history, not
  a substitute base.
- Draft PR #197 and its exact branch/worktree context were bound through
  `finalize-plan` before checkpoint and implementation.

## Acceptance

The three WI-243 rows retain the pre-merge finalize path and add the close
path. WI-244 is shown as Recovered with its recovery receipt. PR #197 merged as
`98d6575`; the append-only WI-246 chain records merge observation, exact
branch/worktree cleanup, and the structured close receipt, so WI-246 is now
Implemented. A deterministic regression constructs a
base-plus-feature merge tree: omission of the base close decision yields three
`missing_parity_decision` findings, while all three rows containing both paths
pass. The pending registry's strict schema, identity, Git ancestry, symlink,
and lifecycle checks remain unchanged. WI-247 is retained as an immutable
recovered predecessor after its own parity row was projected too late; WI-249
registers that recovery and prevents the same archive-ordering defect.

## Verification

Focused governance, pending-registry, manifest, route, documentation, and
parity tests run before the strict typed repository gate. Rustfmt, Clippy, and
the full workspace suite remain required. Runtime v0.2.31 records the final
verification, visible human Outcome, archive, and append-only finalization.
