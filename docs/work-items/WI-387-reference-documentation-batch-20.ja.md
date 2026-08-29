---
author: AI Cockpit maintainers
title: "WI-387 — reference documentation batch 20"
workItemId: WI-387-reference-documentation-batch-20
description: "4 つの pinned security / supply-chain 文書を逐一比較し、source authority をコピーせず Rust-native parity を記録する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-387-reference-documentation-batch-20
---

# WI-387 — reference documentation batch 20

## Intent と boundary

pinned source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の deferred security / supply-chain 文書 4 件を一つずつ比較し、inventory と三言語 parity ledger に一件ずつ bounded decision を記録します。

対象は semantic/documentation parity であり、source command、JSON-wire、provider-state compatibility ではありません。Rust-native Runtime は宣言された governance fact と矛盾する repository operation を拒否/停止できますが、general prompt-injection detector ではありません。provenance、signature、SBOM、vulnerability result、trust root は外部 delegated evidence の責任です。source Python、Make、provider configuration、historical evidence は current authority としてコピーしません。

## File decisions

| Pinned path | Decision | Maintained target boundary |
| --- | --- | --- |
| `docs/security/injection-boundary.ja.md` | `implemented-different-by-design` | Japanese `adversarial-validation`、`input-trust-dataflow`、`operation-time-policy-reevaluation` が bounded injection handling、fail-closed reevaluation、external-control limit を保持します。 |
| `docs/security/injection-boundary.md` | `implemented-different-by-design` | Rust-native security / trust-flow 文書が repository-governance boundary を保持します。untrusted text は data のままで general detector claim はしません。 |
| `docs/security/injection-boundary.zh-CN.md` | `implemented-different-by-design` | Chinese Rust-native security / trust-flow 文書が deterministic fail-closed handling と明示的な non-claims を保持します。 |
| `docs/security/supply-chain.md` | `implemented-different-by-design` | threat-model、ci-release-evidence、distribution、security-release-verification が delegated evidence ownership と exact artifact binding を保持し、Runtime は external assurance を生成しません。 |

## Acceptance

- 4 つの pinned source file を読み、各 file に一つの inventory classification、Rust-native counterpart、bounded reason を記録し、`migrate-gap` を 0 に保つ。
- English、Chinese、Japanese の comparison/parity ledger が同じ 4 決定と更新後の count (`4262/298/1/4/47/507/0`) を示す。
- Injection / supply-chain boundary が local governance evidence と external provider/security control を区別し、source command や historical claim をコピーしない。
- すべての attach 済み object/adopter project は shared Runtime から同じ Rust-native documentation boundary を継承し、repository fact、Work Item、evidence、snapshot は明示的な `--repo` で分離される。
- documentation、inventory、governance、installed Runtime lifecycle check が通り、無関係な Runtime code や historical evidence を変更しない。

## Verification

reference inventory documentation/script test、documentation/status consistency、governance integrity gate、および明示的 repository context を使う installed Runtime の `inspect`、`status`、`doctor`、`preflight`、`checkpoint`、`verify`、`finish`、`archive`、`close` を宣言します。

[English](WI-387-reference-documentation-batch-20.md) · [简体中文](WI-387-reference-documentation-batch-20.zh-CN.md)
