---
author: AI Cockpit maintainers
title: "WI-160 — Resource finalization and branch/worktree closure baseline"
description: "Define and statically guard the resource-finalization boundary after a reviewed PR is merged."
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-160-resource-finalization-baseline
workItemId: WI-160-resource-finalization-baseline
---

# WI-160 — Resource finalization and branch/worktree closure baseline

## Intent

Merge and Work Item closure are different facts. This Work Item prevents a
reviewed PR from being treated as fully closed while its exact branch or
worktree is dirty, unidentified, retained without a decision, or still present.

## Boundary

The policy baseline is:

```text
finalize-plan → finalize → finalize-verify → close
```

`finalize-plan` records the exact branch, worktree, provider PR, merged head,
remote, default branch, and cleanup intent without deleting anything.
`finalize` acts only on that exact merged resource after identity, protection,
and dirty-state checks. `finalize-verify` proves synchronized default-branch
state, clean relevant worktrees, and exact local/remote branch removal.

An observation failure or provider error is `unknown` and keeps the Work Item
open for recovery. `retain` is allowed only as an explicit, bounded human
decision with owner, reason, scope, and expiry/review condition; it is never a
silent cleanup success. `close` before successful finalization is forbidden.

Runtime `0.2.17` now exposes these commands; WI-159 supplies the typed Runtime
command and receipt integration. WI-161 adds the historical-evidence close
compatibility rule without rewriting archived bytes.

Verification: `.ai/evidence/WI-160-resource-finalization-baseline.verification.json`.
Archive: `.ai/work-items/archive/WI-160-resource-finalization-baseline.archive.json`.
Decision: `.ai/decisions/WI-160-resource-finalization-baseline.close.json`.

## In scope

- Three-language `agent-workflow` and `reference-parity` contract language.
- A static/regression gate at `tests/workflow/resource_finalization_policy.sh`
  and its test wrapper.
- A tri-lingual Work Item description of the boundary and its Runtime evidence
  compatibility status.

## Out of scope

- Runtime source or `crates/**` changes.
- Provider-side branch deletion, GitHub workflow changes, or global Agent/MCP
  configuration.
- Deleting or modifying existing branches and worktrees.

## Acceptance

1. All three workflow pages require `finalize-plan`, `finalize`, and
   `finalize-verify`, preserve `unknown`/`retain`, forbid silent deletion and
   close-before-cleanup, and describe the Runtime command boundary.
2. All three parity pages state the same implemented boundary and identify the
   historical-evidence compatibility rule.
3. The static gate passes for the repository and fails when a required closure
   rule is removed from any language page.
4. The change remains limited to `docs/` and `tests/`; no Runtime source or
   generated governance receipt is hand-edited.

## Verification

Run `tests/workflow/resource_finalization_policy_test.sh` and the repository
documentation acceptance gate. Runtime lifecycle evidence binds this Contract
and the resulting verification receipt; historical evidence remains immutable.
