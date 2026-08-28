---
author: AI Cockpit maintainers
title: "WI-344 — reference documentation batch 14"
workItemId: WI-344-reference-documentation-batch-14
description: "5 つの pinned reference acceptance/recovery document を一つずつ比較し、source history を取り込まず Rust の bounded counterpart を記録します。"
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

## Intent と boundary

この Work Item は pinned reference の次の 5 document を一つずつ比較します。
recovery usability、final North Star acceptance、source WIII remediation audit、
source full-remediation baseline について、Rust-native な reader/Runtime boundary
で表現されるか、target に移せない source-specific history かを記録します。

範囲は inventory generator/manifest、tri-language comparison/parity page、本 Work Item
record に限定します。Runtime behavior、source Python/Make tooling、provider/global
Agent configuration、immutable historical evidence、release/adopter execution は対象外です。

## File-by-file decision

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `docs/reference/failure-recovery-usability.md` | `implemented-different-by-design` | Repository-bound recovery、failed gate/recovery condition、Task Outcome、人間向け handoff が現在の boundary です。source の 9 scenario Python report wire shape は copy せず、関連 script/test は別 batch で扱います。 |
| `docs/reference/final-north-star-acceptance.json` | `implemented-different-by-design` | target の final-replacement acceptance route と exact dimension/parity record が evidence と external adopter/provider limitation を保持し、source decision bytes は import しません。 |
| `docs/reference/final-north-star-acceptance.md` | `implemented-different-by-design` | Design Philosophy、Product Boundary、Outcome、final-replacement acceptance が North Star boundary を保持します。local check は external evidence の代替ではありません。 |
| `docs/reference/final-wiii-remediation-closure-audit.md` | `reference-only` | source 固有の WIII PR identity、reviewer、historical closure claim は target evidence ではありません。Rust は自身の Work Item intelligence/parallelism documentation を保持します。 |
| `docs/reference/full-remediation-acceptance.md` | `reference-only` | source WI-01–WI-19 remediation sequence は internal history です。target は自身の evidence-bound acceptance route だけを保持し、source progress/Release claim を公開しません。 |

これは semantic/documentation parity であり、source command や JSON-wire parity ではありません。
object/adopter boundary は shared Runtime、repository ごとの state isolation、独立した evidence です。

## Acceptance と verification

- 5 path は pinned inventory に各 1 回だけ現れ、上記 classification で deferred/migrate-gap はありません。
- English、Simplified Chinese、Japanese の comparison/parity page が同じ decision と current count を示します。
- source implementation、internal progress history、provider identity、external evidence は copy/promote しません。
- inventory、documentation、governance、locked-workspace check が通ります。

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit a533d49dfa848d95742833f8cd1b5f7e1bb897d5 --check
bash tests/docs/documentation_acceptance.sh
bash tests/docs/getting_started_semantic.sh
python3 tests/ci/governance_integrity_gate.py --repo . --report target/wi344-governance-integrity.json
cargo test --locked --workspace
```

[English](WI-344-reference-documentation-batch-14.md) ·
[简体中文](WI-344-reference-documentation-batch-14.zh-CN.md)
