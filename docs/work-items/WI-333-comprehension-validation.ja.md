---
author: AI Cockpit maintainers
workItemId: WI-333-comprehension-validation
title: "WI-333 — Reference comprehension-validation protocol と participant record"
description: "Pinned comprehension-validation study files を比較し、移植不可能な target boundary を記録します。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-333-comprehension-validation
terminalArchive: .ai/work-items/archive/WI-333-comprehension-validation.contract.json
terminalVerification: .ai/evidence/WI-333-comprehension-validation.verification.json
terminalFinalization: .ai/decisions/WI-333-comprehension-validation.finalize.c6e13e0df12bdce915174643c5ca06ca89b004152f1ca4632cbdd580713b0fa5.json
terminalDecision: .ai/decisions/WI-333-comprehension-validation.close.json
capabilityClaims:
  - reference_parity
---

# WI-333 — Reference comprehension-validation protocol と participant record

## Intent

Pinned source を一つずつ比較し、安全で監査可能な classification を記録します。この Work Item は
target の境界を定義するだけで、participant study を実施せず human-subject evidence を移植しません。

## Scope と決定

以下の 12 path はすべて `reference-only` です。procedure、匿名 identifier、revision、answer、sample
count、study conclusion は reference repository に固有で target に移せません。Target counterpart は
自身の reader route、Agent workflow、Contract、Outcome、Runtime evidence boundary を説明します。
source response/result bytes を copy せず、target の comprehension、release、safety、security、
enterprise claim を source study から導きません。

| Pinned source path | Target counterpart | 決定 |
| --- | --- | --- |
| `docs/reference/comprehension-validation-protocol.md` | `docs/README.md`、Agent workflow、Outcome report | `reference-only`; 外部 eligibility、consent、interview |
| `docs/reference/comprehension-validation-protocol.zh-CN.md` | Chinese README、Agent workflow、Outcome report | `reference-only`; target study を意味しません |
| `docs/reference/comprehension-validation-protocol.ja.md` | Japanese README、Agent workflow、Outcome report | `reference-only`; source ethics は Runtime policy ではありません |
| `docs/reference/comprehension-validation-response.schema.json` | `.ai/README.md`、Outcome report | `reference-only`; Runtime Contract/evidence schema ではありません |
| `docs/reference/comprehension-validation-responses/peter_01.en.json` | English README、human-benefit report | `reference-only`; source historical response |
| `docs/reference/comprehension-validation-responses/peter_02.en.json` | English README、human-benefit report | `reference-only`; participant data を import しません |
| `docs/reference/comprehension-validation-responses/tanaka_01.ja.json` | Japanese README、human-benefit report | `reference-only`; adopter evidence ではありません |
| `docs/reference/comprehension-validation-responses/tanaka_02.ja.json` | Japanese README、human-benefit report | `reference-only`; source revision-bound |
| `docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json` | Chinese README、human-benefit report | `reference-only`; target score を claim しません |
| `docs/reference/comprehension-validation-responses/xiaoli_02.zh-CN.json` | Chinese README、human-benefit report | `reference-only`; raw text を copy しません |
| `docs/reference/comprehension-validation-results.json` | tri-language human-benefit report、comparison | `reference-only`; result は source revision に bind |
| `docs/reference/comprehension-validation-results.md` | English human-benefit、Outcome report | `reference-only`; source limitation は target evidence ではありません |

## Acceptance

- Inventory は WI-333 の 12 record を持ち、すべて `reference-only`、counterpart/reason は non-empty です。
- WI-333 に deferred/migrate はなく、participant response/result を target evidence に copy しません。
- tri-language comparison、parity、Work Item documentation は同じ boundary を示します。
- docs/inventory check、installed Runtime evidence、reviewed PR、merge、close、cleanup を完了します。

## Object/adopter boundary

Adopter は target の documentation route、Contract、evidence、Agent workflow を継承しますが、他
repository の human-subject record は継承しません。将来 study を行うには consent、retention、privacy、
evidence を定義した独立した human-owned Contract が必要です。

Language versions: [English](WI-333-comprehension-validation.md) · [简体中文](WI-333-comprehension-validation.zh-CN.md)
