---
author: AI Cockpit maintainers
title: Verification コスト観測
description: ガバナンスを弱めずに Verification コストを記録・最適化する境界を説明する。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-146-verification-cost-observation
---

# Verification コスト観測

AI Cockpit は Verification コストを監査可能な advisory projection として公開
する。計画ノード、実行ノード、再利用ノード、リソース単位、経過時間、起動プロセス数、
観測された並列度を記録する。コスト projection が `VerificationTier`、
`EvidenceAssurance`、Policy 要件、protected gate、最終的な governance result を
変更することはない。

## 二つの直交する軸

Verification の強度と Evidence Assurance は分離する。

- `VerificationTier`: `T0`、`T1`、`T2`、`T3`。
- `EvidenceAssurance`: `SelfDeclared`、`RepositoryVerified`、
  `ProviderVerified`、`EnterpriseVerified`。

速い実行は強い Verification を意味せず、高い Tier も Provider/Enterprise の
assurance を意味しない。必要な Verification は Policy と protected gate の参照が
決め、コスト観測は計画と実行の事実だけを記録する。

## Estimate と Observation

`VerificationExecutionPlan::cost_estimate` は実行前の推定、
`VerificationReceipt::cost_observation` は実行後の事実 projection である。どちらも
schema version、明示的な confidence、`advisoryOnly` を持つ。worker/resource 予算、
実行状態、repository/Runtime identity が不明な場合、confidence は `partial` または
`unknown` となり、未知の測定が green の governance result になることはない。

Reuse や affected verification の削減はコスト事実として観測できるが、protected
node や Policy が要求する node を飛ばす権限にはならない。物理実行の再利用も
Work Item ごとの Evidence Receipt とは分離し、各 Work Item に固有の
identity-bound receipt を発行する。

## 単一 Work Item と並列実行

単一 Work Item と独立した並列 node の双方を観測できる。`maxConcurrentProcesses` は
実際に観測した並列度であり、保証や性能目標ではない。リソース予算と依存関係の準備
状態が実行を制約し、コスト推定が不完全でも protected node は実行される。

開発の順序は次のとおりである。

> **Verification Truth before Verification Cost.**

まず Policy、Tier、Assurance、scope、evidence identity、protected gate を守り、
その後にコスト事実を用いて不要な実行を減らす。固定のレイテンシやスループット目標は
assurance の主張ではない。
