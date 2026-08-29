---
author: AI Cockpit maintainers
title: "WI-378 — reference documentation batch 17"
description: "pinned reference の次の文書群を比較し、境界を明示した Rust-native 三言語対応を公開します。"
workItemId: WI-378-reference-documentation-batch-17
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-378-reference-documentation-batch-17
capabilityClaims: [reference_comparison, documentation_governance, adopter_readiness]
---

# WI-378 — reference documentation batch 17

[English](WI-378-reference-documentation-batch-17.md) · [简体中文](WI-378-reference-documentation-batch-17.zh-CN.md)

## Intent

pinned reference inventory の次の 10 path を一つずつ比較し、source の Python、Make、provider 設定、
historical decision をコピーせず、shared Rust Runtime の reader-facing governance semantics を整備します。

## 比較した path と決定

| Pinned path | 決定 |
| --- | --- |
| `docs/reference/remediation-instruction-traceability.json` | `reference-only`。generated historical plan trace は target authority ではありません。 |
| `docs/reference/repository-workflow.ja.md` | Rust-native 三言語 workflow documentation。 |
| `docs/reference/schemas.md` | Rust-native 三言語 record family / validation map。 |
| `docs/reference/test-architecture.md` | Rust-native 三言語 layered test / evidence model。 |
| `docs/reference/test-weakening-guard.{md,zh-CN.md,ja.md}` | snapshot-derived weakening route と bounded policy。 |
| `docs/reference/troubleshooting.{md,ja.md}` | explicit repository recovery と toolchain boundary。 |
| `docs/reference/upgrade.ja.md` | Runtime upgrade と repository migration の Rust-native boundary。 |

source の英語 `upgrade.md` は deferred inventory に残し、別の bounded batch で比較します。選択した日本語 route
を完全にするため、target の三言語 upgrade page はこの batch で用意します。

## Boundary

これは semantic/documentation parity であり、source JSON-wire、command、Python、Make、provider parity ではありません。
すべての adopter は一つの installed Runtime と明示的な `--repo` を使い、repository fact、Work Item、evidence、decision を分離します。
Documentation は authority、approval、assurance、verification evidence を発明しません。

## Acceptance

- 選択した各 path に classification と target counterpart、または明示的な `reference-only` decision がある。
- 英語・中国語・日本語の reader route が同じ boundary と相互リンクを持つ。
- inventory と parity ledger が source commit、Work Item、classification、`migrate-gap=0` で一致する。
- documentation/conformance check と installed v0.2.39 Runtime verification が成功する。
- Contract の原文 fact を保持し、semantic parity を wire compatibility と表現しない。

## Verification plan

明示的な repository context で inventory、documentation、conformance、governance、installed Runtime check を実行します。
terminal archive、verification、finalization、close receipt は Runtime 検証後にだけ生成します。

