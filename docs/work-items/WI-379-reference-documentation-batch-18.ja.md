---
author: AI Cockpit maintainers
title: “WI-379 — reference documentation batch 18”
description: “pinned reference の次の10 pathを比較し、bounded Rust-native reader route を公開する。”
workItemId: WI-379-reference-documentation-batch-18
canonical: docs/work-items/WI-379-reference-documentation-batch-18.md
audience: [maintainer, reviewer, adopter]
status: recovered
authority: translation
lastVerifiedBy: WI-379-reference-documentation-batch-18
terminalDecision: .ai/decisions/WI-379-reference-documentation-batch-18.recovery.json
capabilityClaims: [reference_comparison, verification_reuse, intelligence, lifecycle_closure]
---

# WI-379 — reference documentation batch 18

[English](WI-379-reference-documentation-batch-18.md) · [简体中文](WI-379-reference-documentation-batch-18.zh-CN.md) · [日本語](WI-379-reference-documentation-batch-18.ja.md)

## Intent

pinned inventory の次の10 pathを一つずつ比較し、その reader-facing な governance meaning を
shared Rust Runtime へ写像します。source の Python、Make、provider configuration、historical
decision はコピーしません。

reviewed PR #343 は bounded documentation を届けましたが、provider PR identity が確定する
前に Work Item が archive されました。archive、evidence、Outcome、pending resource context
は immutable な historical bytes です。明示的な recovery successor WI-380 が、この記録を
書き換えずに provider finalization を完了します。

## Path と判断

| Pinned path | 判断 |
| --- | --- |
| `docs/reference/upgrade.md` | `implemented-different-by-design`。migration、backup/conflict、rollback、adapter boundary を三言語 upgrade route に補足。 |
| `docs/reference/verification-evidence-reuse-runtime.md` | `implemented-different-by-design`。typed receipt binding、protected node、planner/adapter separation、observable reuse。 |
| `docs/reference/verification-evidence-reuse.md` | `implemented-different-by-design`。freshness、invalidation、call-count evidence。 |
| `docs/reference/verification-fixture-boundary.md` | `implemented-different-by-design`。Rust fixture isolation と local evidence の限界。 |
| `docs/reference/wi01-wi20-bidirectional-traceability-audit.json` | `reference-only`。generated historical V1 audit bytes は target authority ではありません。 |
| `docs/reference/wi01-wi20-bidirectional-traceability-audit.md` | `reference-only`。source-bound の歴史 narrative はコピーしません。 |
| `docs/reference/wiii-v2-integration-audit.md` | `implemented-different-by-design`。より狭い Rust read-only Intelligence projection と identity check。 |
| `docs/reference/work-item-intelligence-performance-baseline.md` | `implemented-different-by-design`。source 数値/SLO を主張しない再現可能な local observation。 |
| `docs/reference/work-item-lifecycle-closure.ja.md` | `implemented-different-by-design`。Rust-native 三言語 closure route。 |
| `docs/reference/work-item-lifecycle-closure.md` | `implemented-different-by-design`。英語 route と明示的 recovery boundary。 |

## Boundary

これは semantic/documentation parity であり、source command、JSON-wire、provider、generated
history の parity ではありません。1つの installed Runtime が明示的な `--repo` で複数 repository
を扱いますが、fact、Work Item、evidence、knowledge、snapshot は分離します。文書は authority、
approval、assurance、verification evidence を生成しません。

## Acceptance

- 各 selected path に inventory classification と counterpart または明示的な reference-only reason がある。
- English、Simplified Chinese、日本語 route の link と semantic/non-wire boundary が一致する。
- inventory と parity が同じ source commit/batch decision を記録し、`migrate-gap` が 0 である。
- documentation、inventory、conformance、installed Runtime check が source fallback なしで通る。
- presentation localization が Contract language の governance fact を変更しない。

## Verification

inventory、inventory-docs、inventory regression、documentation acceptance、status consistency、
`cargo test --locked --workspace` を実行します。preflight、checkpoint、verify、finish、archive、
finalize、close は installed v0.2.39 を使用し、reviewed merge 後に terminal receipt を追加します。
