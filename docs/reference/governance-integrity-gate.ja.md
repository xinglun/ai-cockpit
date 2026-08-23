---
author: AI Cockpit maintainers
title: Governance integrity gate
description: "current Work Item、evidence、terminal decision、文書 binding を fail-closed で動的に点検します。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-198-governance-gate-default-branch-discovery
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

## detached pull-request checkout

Hosted pull request job は、`refs/remotes/origin/HEAD` や event の base branch metadata
を持たない detached merge checkout で実行されることがあります。その場合、gate は不変な
Contract の `resourceContext.baseBranch` だけを狭い default branch fallback として使います。
receipt と Contract の resource context が完全一致する場合だけ受理し、repository、PR
URL/number、provider、remote、branch、worktree、base/head revision、runtime、evidence、
Contract digest の検査はすべて必須です。外部 event または remote が別の base branch を示す
場合、receipt は拒否されます。identity の欠落や矛盾は引き続き fail-closed です。

この gate は verification tier や assurance を選択しません。risk/stage/policy による
選択と reference source の逐文件 conformance は別の検証境界であり、この inventory から
推測してはなりません。
