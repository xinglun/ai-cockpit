---
author: AI Cockpit maintainers
title: "WI-368 — Reference file comparison batch 16"
description: "Pinned reference の 11 file を逐一比較し、Rust-native boundary を明示する。"
workItemId: WI-368-reference-file-comparison-batch-16
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-368-reference-file-comparison-batch-16
terminalArchive: .ai/work-items/archive/WI-368-reference-file-comparison-batch-16.contract.json
terminalVerification: .ai/evidence/WI-368-reference-file-comparison-batch-16.verification.json
terminalFinalization: .ai/decisions/WI-368-reference-file-comparison-batch-16.finalize.json
terminalDecision: .ai/decisions/WI-368-reference-file-comparison-batch-16.close.json
capabilityClaims:
  - reference_parity
---

# WI-368 — Reference file comparison batch 16

[English](WI-368-reference-file-comparison-batch-16.md) · [简体中文](WI-368-reference-file-comparison-batch-16.zh-CN.md)

## Intent と boundary

この Work Item は pinned reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の 11 path を 1 file ずつ比較し、
Rust Runtime が担う責任、明示的な external boundary、歴史資料を区別します。

target は 1 つの installed Runtime と明示的な `--repo` context を維持します。
source の Python/Make/YAML orchestration、generated history、provider-global configuration、
source JSON wire compatibility、public release creation は out of scope です。semantic mapping
は command/field の同一性を意味しません。

## File-by-file decision

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `docs/reference/pre-release-documentation-alignment.md` | `reference-only` | Historical generated alignment evidence。target の current documentation gate は自身の evidence を使う。 |
| `docs/reference/pre-release-documentation-review.json` | `reference-only` | Historical five-strategy review。source status/finding は target release を authorize しない。 |
| `docs/reference/project-test-timing-baseline.json` | `implemented-different-by-design` | timing を identity-bound Rust performance sample/advisory budget に投影し、verification を弱めない。 |
| `docs/reference/provider-backed-governance-validation.md` | `implemented-different-by-design` | provider config、branch protection、reviewer identity、hosted control は delegated evidence のまま。 |
| `docs/reference/real-absurd-injection-cases.md` | `implemented-different-by-design` | canonical manifest と Rust test で 15 structured case と 12 named RAI case の semantic boundary を保持。 |
| `docs/reference/real-absurd-injection-cases.zh-CN.md` | `implemented-different-by-design` | Chinese semantic boundary を保持し、source prose を Runtime authority にしない。 |
| `docs/reference/real-absurd-injection-cases.ja.md` | `implemented-different-by-design` | Japanese semantic boundary を保持し、general language fluency を主張しない。 |
| `docs/reference/real-adopter-reference-validation.md` | `implemented-different-by-design` | immutable public Release adopter/upgrade acceptance で isolated repository、Runtime、lifecycle、cleanup evidence を確認。 |
| `docs/reference/reference-impact-gate.md` | `reference-only` | source static scanner/schema/Make command は提供せず、operation-time policy は declared facts の狭い boundary に留める。 |
| `docs/reference/reference-impact-gate.zh-CN.md` | `reference-only` | 同じ bounded gap を明示し、source scanner/provider claim を持ち込まない。 |
| `docs/reference/reference-impact-gate.ja.md` | `reference-only` | 同じ bounded gap を明示し、source scanner/provider claim を持ち込まない。 |

Reference source の reference-impact page が target Standard profile の overclaim を示したため、
三語の profile page を修正しました。Standard は明示された impact evidence を要求しますが、
static caller/dynamic reference/external consumer/monitoring scanner を暗黙に提供しません。
既存の operation-time evaluator は declared operation/target/scope/authority/freshness/trust/impact
を評価できますが、その scanner の代替ではありません。

Real-absurdity の source 三語 page は named scenario count が一致しません。target では
canonical manifest（15 structured wording case、12 named RAI case）を machine truth とし、差異を隠しません。

## Acceptance と verification

- 11 path は inventory に各 1 回だけ現れ、non-empty reason を持ち、deferred/migrate-gap を残さない。
- historical/provider record は non-authoritative、timing/cost は advisory、adopter evidence は immutable public Release と isolated repository に bind される。
- 三語 adversarial route の deterministic semantics は一致し、source count discrepancy は明示される。
- 三語 Standard profile は reference-impact scanner を overclaim せず、operation-time limitation をリンクする。
- inventory、documentation metadata/link、parity、governance integrity、targeted test が pass し、source Python/Make/V1 と global Agent/MCP config は追加しない。
- installed v0.2.38 Runtime で repository-bound lifecycle を実行し、merge/close/cleanup 前に human Outcome を可視化する。

Pinned reference commit: `e5acb677da6621004d96f0ef353c58fe8d3acfbf`。
