---
author: AI Cockpit maintainers
title: "WI-188 — Governance integrity gate"
description: "A dynamic fail-closed inventory for current Work Item, evidence, decision, Outcome, documentation, and CI coverage."
audience:
  - maintainer
  - reviewer
workItemId: WI-188-governance-integrity-gate
status: implemented
authority: canonical
lastVerifiedBy: WI-188-governance-integrity-gate
---

# WI-188 — Governance integrity gate

WI-188 replaces the fixed WI-177–WI-186 parity list with a repository inventory.
The gate discovers active and archived Contracts, derives the current release
cycle from Cargo metadata plus Contract/archive creation time, and verifies current
Summary, archive, verification, terminal decision, Outcome, and three-language
parity bindings. Older records remain visible as historical or legacy entries;
unknown current problems fail closed.

An archived feature-branch Work Item may report `awaiting_merge_close` only
when its Runtime finalize receipt proves an unmerged PR, a present branch, a
clean worktree, `blocked` disposition, the sole failure code
`unmerged_pull_request`, no unknown codes, and a reason beginning with the
`awaiting_merge_close` audit token. That receipt is not terminal closure: on
the default branch the gate still requires an exact close or recovery decision.
The exception also binds the repository identity, raw archived Contract digest,
verification Runtime identity, actual remote default branch, and internally
consistent PR, branch, worktree, and Contract resource context.

CI uses one manifest for documentation, workflow, conformance, performance,
and release gates. Workspace package tests are derived from `cargo metadata`,
executed serially, and bound to a deterministic JSON receipt.

[简体中文](WI-188-governance-integrity-gate.zh-CN.md) ·
[日本語](WI-188-governance-integrity-gate.ja.md)
