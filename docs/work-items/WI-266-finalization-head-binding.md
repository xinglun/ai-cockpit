---
author: AI Cockpit maintainers
title: "WI-266 — Finalization head binding successor"
workItemId: WI-266-finalization-head-binding
description: "Bind repository finalization receipts to the exact reviewed provider head."
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-267-finalization-parity-regression-repair
authority: canonical
---

# WI-266 — Finalization head binding successor (recovered predecessor)

This Work Item is immutable recovered history. WI-267 is the clean successor
for the hosted-quality regression discovered after WI-266 was archived.

## Intent

The failed WI-261 delivery exposed that a self-consistent finalization receipt
is not enough: the reviewed checkout must be the receipt's exact head, except
for a bounded append of the Runtime governance record itself. This successor
redelivers that control from the current default branch while preserving the
failed predecessor as immutable history.

## Scope and evidence boundary

- Bind feature and pull-request finalization receipts to the provider-reviewed
  checkout head.
- Allow only the canonical Runtime finalization append and explicitly bounded
  same-Work-Item governance records after that head; reject code or unrelated
  repository drift.
- Keep the governance-integrity fixture, regression tests, reference docs, and
  English/Simplified Chinese/Japanese parity synchronized before archive.
- Complete hosted review, Runtime finalization verification, exact cleanup, and
  structured close before promotion to Implemented.

The failed WI-261 archive, evidence, branch, and PR remain historical. This
Work Item does not migrate quality-route execution into Rust or change global
Agent/MCP configuration.

## Failure and recovery cases

The governance gate must fail closed for a missing/foreign reviewed head,
post-finalization code drift, unrelated files, malformed transition records, or
missing parity. Append-only governance evidence is accepted only when it is
bound to the same Work Item and reviewed head.

## Verification

- `bash tests/ci/governance_integrity_gate_test.sh`
- `bash tests/docs/parity_status_check_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `cargo fmt --all -- --check`
- `cargo test --locked --workspace`
- installed Runtime lifecycle, finalization verification, and human Outcome
  with an explicit `--repo`

The final handoff must be a visible `Outcome: 🟢`, `Outcome: 🟡`, or
`Outcome: 🔴` and include status, unknowns, evidence, human decision, and next
action.
