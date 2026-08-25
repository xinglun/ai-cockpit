---
author: AI Cockpit maintainers
title: "WI-267 — Finalization parity regression repair"
workItemId: WI-267-finalization-parity-regression-repair
description: "Repair bounded finalization/parity append semantics exposed by hosted quality while preserving WI-266."
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-267-finalization-parity-regression-repair
authority: canonical
---

# WI-267 — Finalization parity regression repair

## Intent

Hosted quality exposed a regression in WI-266: a pending parity registry append
was treated as implementation drift after finalization. This successor keeps
WI-266 immutable and makes the exception explicit and bounded.

## Scope and evidence boundary

- Allow only the pending parity registry as a repository-level governance
  append after a reviewed finalization head; code, tests, unrelated evidence,
  and arbitrary documentation remain rejected.
- Build fixtures with a true append-only finalization history and keep the
  pending-parity regression green, including default-branch and adversarial
  cases.
- Synchronize the governance gate documentation and parity rows in all three
  supported languages.
- Complete hosted review, Runtime finalization verification, exact cleanup,
  and structured close before promotion.

WI-266 archive, evidence, finalization, and close bytes remain immutable. This
Work Item does not change release-version consistency, quality-route migration,
or global Agent/MCP configuration.

## Verification

- `bash tests/ci/governance_integrity_gate_test.sh`
- `bash tests/docs/pending_parity_registry_test.sh`
- `bash tests/docs/parity_status_check_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `cargo fmt --all -- --check`
- `cargo test --locked --workspace`
- installed Runtime lifecycle and visible human Outcome with explicit `--repo`

The final handoff must be a visible `Outcome: 🟢`, `Outcome: 🟡`, or
`Outcome: 🔴` with status, unknowns, evidence, human decision, and next action.
