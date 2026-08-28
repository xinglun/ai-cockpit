---
author: AI Cockpit maintainers
title: "WI-343 — reference inventory foundation reconciliation"
workItemId: WI-343-reference-inventory-foundation-reconciliation
description: "既に比較済みの 5 つの reference path を machine inventory に登録し、Runtime behavior や source tooling は変更しません。"
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

## Intent と boundary

WI-339 は pinned reference の 5 path を一つずつ比較しましたが、generated inventory は
それらを `deferred-next-batch` のまま残していました。本 Work Item はこの ledger gap を
reconcile し、machine inventory、tri-language comparison page、parity register が同じ
review 済みの判断を表すようにします。

変更範囲は inventory generator/manifest、comparison と parity の文書、および本 Work Item
record に限定します。Runtime behavior、source Python/Make tooling、provider integration、
immutable historical evidence、global Agent/MCP configuration、その他の deferred path は対象外です。

## File-by-file decision

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `docs/reference/cross-wi-integration.md` | `reference-only` | Source aggregate report は advisory です。target は Work Item archive、parity ledger、human Outcome を audit boundary とします。 |
| `docs/reference/dependabot-intake.md` | `not-applicable` | Dependabot bot-branch intake は provider-specific です。delegated evidence と dependency fact は外部/Repository の責任です。 |
| `docs/reference/deprecated-assets-registry.json` | `reference-only` | Source cleanup registry は portable Runtime protocol ではありません。target boundary は明示的な lifecycle と resource finalization です。 |
| `docs/reference/deprecated-assets.md` | `reference-only` | Source の obsolete-chain 説明は reference documentation に限定し、Rust は `check-deprecated-assets` を claim しません。 |
| `docs/reference/derived-artifacts.md` | `implemented-different-by-design` | Typed Contract/evidence/archive/status/Outcome projection で fact と view を分離し、derived view は decision を authorize できません。 |

これは semantic/documentation parity であり、source command や JSON-wire parity ではありません。
Source implementation は copy せず、inventory reconciler は governance decision を発明しません。

## Acceptance

- 5 path が pinned inventory に各 1 回だけ現れ、上記 classification となり、
  `deferred-next-batch` と `migrate-gap` はありません。
- Pinned source/target commit で inventory generation と `--check` が deterministic に通ります。
- English、Simplified Chinese、Japanese の comparison/parity page が 5 件の decision と
  current count で一致します。
- Runtime behavior、source tooling、immutable evidence、provider/global configuration は変更しません。
- 宣言済みの documentation、inventory、governance、locked-workspace check が通ります。

## Verification commands

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit a533d49dfa848d95742833f8cd1b5f7e1bb897d5 --check
bash tests/docs/documentation_acceptance.sh
bash tests/docs/getting_started_semantic.sh
python3 tests/ci/governance_integrity_gate.py --repo . --report target/wi343-governance-integrity.json
cargo test --locked --workspace
```

[English](WI-343-reference-inventory-foundation-reconciliation.md) ·
[简体中文](WI-343-reference-inventory-foundation-reconciliation.zh-CN.md)
