---
author: AI Cockpit maintainers
title: "WI-343 — reference inventory foundation reconciliation"
workItemId: WI-343-reference-inventory-foundation-reconciliation
description: "Register five already-compared reference paths in the machine inventory without changing Runtime behavior or copying source tooling."
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-343-reference-inventory-foundation-reconciliation
terminalArchive: .ai/work-items/archive/WI-343-reference-inventory-foundation-reconciliation.contract.json
terminalVerification: .ai/evidence/WI-343-reference-inventory-foundation-reconciliation.verification.json
terminalFinalization: .ai/decisions/WI-343-reference-inventory-foundation-reconciliation.finalize.json
terminalDecision: .ai/decisions/WI-343-reference-inventory-foundation-reconciliation.close.json
capabilityClaims:
  - reference_parity
---

# WI-343 — reference inventory foundation reconciliation

## Intent and boundary

WI-339 compared five pinned reference paths one by one, but the generated
inventory still classified them as `deferred-next-batch`. This Work Item
reconciles that ledger gap so the machine inventory, tri-language comparison
pages, and parity register describe the same reviewed decisions.

The change is limited to the inventory generator/manifest, comparison and
parity documentation, and this Work Item record. Runtime behavior, source
Python/Make tooling, provider integrations, immutable historical evidence,
global Agent/MCP configuration, and other deferred paths are out of scope.

## File-by-file decisions

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `docs/reference/cross-wi-integration.md` | `reference-only` | The source aggregate report is advisory; target per-Work-Item archives, parity ledgers, and human Outcomes are the audit boundary. |
| `docs/reference/dependabot-intake.md` | `not-applicable` | Dependabot bot-branch intake is provider-specific; generic delegated evidence and dependency facts remain external/repository-owned. |
| `docs/reference/deprecated-assets-registry.json` | `reference-only` | Source cleanup registry is not a portable Runtime protocol; explicit lifecycle and resource finalization are the target boundary. |
| `docs/reference/deprecated-assets.md` | `reference-only` | Source obsolete-chain guidance remains reference documentation; Rust does not claim a copied `check-deprecated-assets` command. |
| `docs/reference/derived-artifacts.md` | `implemented-different-by-design` | Typed Contract/evidence/archive/status/Outcome projections preserve fact-versus-view separation; derived views cannot authorize decisions. |

This is semantic/documentation parity, not source command or JSON-wire parity.
No source implementation is copied and no governance decision is invented by
the inventory reconciler.

## Acceptance

- The five paths occur exactly once in the pinned inventory with the listed
  classifications and no `deferred-next-batch` or `migrate-gap` record.
- Inventory generation and `--check` are deterministic at the pinned
  source/target commits.
- English, Simplified Chinese, and Japanese comparison/parity pages agree on
  the five decisions and current counts.
- Runtime behavior, source tooling, immutable evidence, and provider/global
  configuration remain unchanged.
- Declared documentation, inventory, governance, and locked-workspace checks
  pass.

## Verification commands

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit a533d49dfa848d95742833f8cd1b5f7e1bb897d5 --check
bash tests/docs/documentation_acceptance.sh
bash tests/docs/getting_started_semantic.sh
python3 tests/ci/governance_integrity_gate.py --repo . --report target/wi343-governance-integrity.json
cargo test --locked --workspace
```

[简体中文](WI-343-reference-inventory-foundation-reconciliation.zh-CN.md) ·
[日本語](WI-343-reference-inventory-foundation-reconciliation.ja.md)
