---
author: AI Cockpit maintainers
title: "WI-272 — reference Agent rule batch"
workItemId: WI-272-reference-agent-rule-batch
description: "Compare the reference Agent/rules surfaces file by file and project their boundaries into the Rust repository without copying template implementation."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-272-reference-agent-rule-batch
authority: canonical
---

# WI-272 — reference Agent rule batch

## Intent

Compare the reference Agent rule template, risk gate, and regression corpus one
file at a time. Preserve the governance meaning in repository-local guidance,
generated Rust Agent adapters, typed Runtime boundaries, tests, and parity
evidence without copying the reference Python modules, Make commands, or
provider-global configuration.

## Scope

- Add the missing delivery-order, retry-checkout, Outcome terminality, factual
  evidence, and current-Work-Item repair boundaries to the generated adapter,
  `AGENTS.md`, `.ai/README.md`, and tri-language Agent workflow docs.
- Add adapter regression assertions for the projected rules.
- Classify the four deferred reference Agent/rules files with exact Rust
  counterparts and explicit different-by-design reasons in the pinned ledger.
- Keep this batch limited to Agent discovery/rule projection; Runtime
  architecture cleanup and unrelated CI/release comparison remain later work.

## Boundary

The reference Python risk gate and tests are specification evidence, not files
to copy. Existing Rust Contract/preflight/checkpoint/lifecycle behavior is
mapped and tested where it is already authoritative. Any deeper typed
checkpoint-evidence or repository-wide parallel enforcement gap remains a
separate later batch rather than being hidden behind documentation claims.

## Verification

- installed Runtime with explicit `--repo`
- `cargo test --locked -p cockpit-agent --all-targets`
- reference inventory, parity, documentation, and repository governance gates
- full workspace quality and hosted checks
- visible `Outcome: 🟢`, `Outcome: 🟡`, or `Outcome: 🔴` containing status,
  unknowns, evidence, human decision, and next action

## Terminal evidence (planned)

- Archive: `.ai/work-items/archive/WI-272-reference-agent-rule-batch.contract.json`
- Verification: `.ai/evidence/WI-272-reference-agent-rule-batch.verification.json`
- Finalization: `.ai/decisions/WI-272-reference-agent-rule-batch.finalize.json`
- Close: `.ai/decisions/WI-272-reference-agent-rule-batch.close.json`
