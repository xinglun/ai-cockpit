---
author: AI Cockpit maintainers
title: Governance integrity gate
description: "current Work Item、evidence、terminal decision、文書 binding を fail-closed で動的に点検します。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-195-governance-recovery-gate
---

# Governance integrity gate

`tests/ci/governance_integrity_gate.py` は固定 ID リストではなく repository record
から Work Item を動的に検出します。current release cycle、evidence identity、terminal
decision、Outcome、三言語 parity binding を検査し、finding は決定的に生成されます。
不完全な状態は fail-closed です。

## Recovery は完了ではない

正当な `.ai/decisions/<WI>.recovery.json` を持つ predecessor は
`lifecycleState: recovered` として報告されます。receipt は predecessor、空でない
successor、repository identity、reason、evidence refs を束縛しなければなりません。
Recovered predecessor は赤/blocked の不変 Outcome を保持できますが、緑へ昇格されず、
merge や release の承認ともみなされません。

欠落、malformed、foreign、binding 不足の recovery receipt は引き続き error です。
successor は独立して Contract、evidence、Outcome、parity、terminal decision を通過
しなければなりません。

この gate は verification tier や assurance を選択しません。risk/stage/policy による
選択と reference source の逐文件 conformance は別の検証境界であり、この inventory から
推測してはなりません。
