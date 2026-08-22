---
author: AI Cockpit maintainers
workItemId: WI-121-contract-v2
title: Contract V2 の意味論、strict validation、fail-closed preflight
description: 構造化 Contract V2 の意味論、strict parsing、fail-closed preflight review を追加する。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-121-contract-v2
---

# WI-121 — Contract V2 の意味論と fail-closed review

## 目的

参考源の Runtime をコピーせず、Rust Contract の境界を整合させる。Contract
は intent、scope、authority、evidence、human decision を明示的に保持し、
未知または不正な governance 入力は実装前に停止させる。

## 範囲

- legacy read を維持する typed Contract V2 の追加；
- 構造化 intent、sources、verification、capability、execution 宣言；
- unknown field、duplicate key、schema、cross-field の strict validation；
- 構造化 preflight human-decision request と repository-bound review receipt；
- fail-closed checkpoint と lifecycle transition validation；
- 三言語 CLI/MCP の machine/human projection。

scenario/final-dimension aggregation は WI-122、Contract の parallel slot と
serialized projection lease は WI-123 の範囲とする。Contract 原文の自動翻訳
と過去 bytes の書き換えは行わない。

## 検証

protocol、preflight、lifecycle、projection の focused regression と locked
Rust workspace の quality gate を実行する。Human Outcome は marker、unknown、
evidence、decision、next action を保持する。
