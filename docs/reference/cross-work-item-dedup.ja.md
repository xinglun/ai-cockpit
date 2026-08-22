---
author: AI Cockpit maintainers
title: Work Item 間の物理実行再利用
description: 共有実行コストと Work Item 認可 Evidence を分離します。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-144-cross-work-item-dedup
---

# Work Item 間の物理実行再利用

次の物理 identity がすべて一致する場合、AI Cockpit は 1 回の Verification
実行コストを共有できます。

`repository + repository snapshot + command + environment + Runtime + toolchain`

結果は `PhysicalExecution` と `ExecutionResult` で表します。どちらも Work
Item identity を含まず、認可を与えません。

各 Work Item は結果から独自の `WorkItemEvidenceReceipt` を作成します。Work
Item ID は receipt digest に含まれるため、A と B が物理実行を共有しても
Evidence Receipt は別々です。

> ある Work Item が別の Work Item の Evidence Receipt を自分の認可 Evidence
> として参照してはなりません。

物理再利用はコスト最適化に限られ、Policy の VerificationTier、
EvidenceAssurance、protected gate、authority、freshness 要件を弱めません。
repository、snapshot、Runtime、command、toolchain の不一致は別実行にし、
identity が不明な場合は fail closed にします。

実装 Evidence: `crates/cockpit-verification/src/lib.rs` と
`crates/cockpit-verification/tests/physical_execution.rs`。
