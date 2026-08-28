---
author: AI Cockpit maintainers
title: "WI-344 — reference documentation batch 14"
workItemId: WI-344-reference-documentation-batch-14
description: "Compare five pinned reference acceptance/recovery documents individually and record bounded Rust counterparts without importing source history."
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-344-reference-documentation-batch-14
terminalArchive: .ai/work-items/archive/WI-344-reference-documentation-batch-14.contract.json
terminalVerification: .ai/evidence/WI-344-reference-documentation-batch-14.verification.json
terminalFinalization: .ai/decisions/WI-344-reference-documentation-batch-14.finalize.json
terminalDecision: .ai/decisions/WI-344-reference-documentation-batch-14.close.json
capabilityClaims:
  - reference_parity
---

# WI-344 — reference documentation batch 14

## Intent and boundary

This Work Item compares the next five pinned reference documents one by one:
recovery usability, final North Star acceptance, the source WIII remediation
audit, and the source full-remediation baseline. It records whether each
responsibility is represented by a Rust-native reader/runtime boundary or is
source-specific history that must not be imported.

The scope is limited to the inventory generator/manifest, tri-language
comparison and parity pages, and this Work Item record. Runtime behavior,
source Python/Make tooling, provider/global Agent configuration, immutable
historical evidence, and release/adopter execution are out of scope.

## File-by-file decisions

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `docs/reference/failure-recovery-usability.md` | `implemented-different-by-design` | Repository-bound recovery, failed-gate/recovery-condition, Task Outcome, and human handoff routes provide the current boundary. The source nine-scenario Python report wire shape is not copied; its companion script/tests remain separately staged. |
| `docs/reference/final-north-star-acceptance.json` | `implemented-different-by-design` | The target final-replacement acceptance route and exact dimension/parity records preserve the evidence and external-adopter/provider limitation boundary without importing source decision bytes. |
| `docs/reference/final-north-star-acceptance.md` | `implemented-different-by-design` | Design Philosophy, Product Boundary, Outcome, and final-replacement acceptance preserve the North Star; local checks never substitute for external evidence. |
| `docs/reference/final-wiii-remediation-closure-audit.md` | `reference-only` | Source-specific WIII PR identities, reviewers, and historical closure claims are not portable target evidence. Rust keeps its own Work Item intelligence and parallelism documentation. |
| `docs/reference/full-remediation-acceptance.md` | `reference-only` | The source WI-01–WI-19 remediation sequence is internal history. The target keeps its own evidence-bound acceptance route and does not publish source progress or release claims. |

This is semantic/documentation parity, not source command or JSON-wire parity.
The object/adopter boundary remains one shared Runtime with isolated repository
state and independently produced evidence.

## Acceptance and verification

- The five paths occur exactly once in the pinned inventory with the listed
  classifications and no deferred or migrate-gap record.
- English, Simplified Chinese, and Japanese comparison/parity pages agree on
  the decisions and current counts.
- No source implementation, internal progress history, provider identity, or
  external evidence is copied or promoted to current capability.
- Inventory, documentation, governance, and locked-workspace checks pass.

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit a533d49dfa848d95742833f8cd1b5f7e1bb897d5 --check
bash tests/docs/documentation_acceptance.sh
bash tests/docs/getting_started_semantic.sh
python3 tests/ci/governance_integrity_gate.py --repo . --report target/wi344-governance-integrity.json
cargo test --locked --workspace
```

[简体中文](WI-344-reference-documentation-batch-14.zh-CN.md) ·
[日本語](WI-344-reference-documentation-batch-14.ja.md)
