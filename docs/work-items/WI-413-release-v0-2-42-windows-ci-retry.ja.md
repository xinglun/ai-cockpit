---
author: AI Cockpit maintainers
title: "WI-413 — Windows CI 後の v0.2.42 release recovery"
workItemId: WI-413-release-v0-2-42-windows-ci-retry
description: "不変の WI-412 delivery が Windows CI で拒否された後、v0.2.42 candidate を再配信します。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-413-release-v0-2-42-windows-ci-retry
terminalArchive: .ai/work-items/archive/WI-413-release-v0-2-42-windows-ci-retry.contract.json
terminalVerification: .ai/evidence/WI-413-release-v0-2-42-windows-ci-retry.verification.json
terminalFinalization: .ai/decisions/WI-413-release-v0-2-42-windows-ci-retry.finalize.json
terminalDecision: .ai/decisions/WI-413-release-v0-2-42-windows-ci-retry.close.json
---

# WI-413 — Windows CI 後の v0.2.42 release recovery

WI-412 の bounded recovery successor です。predecessor の archive、verification、
recovery decision、failed PR は immutable のまま保持します。本 successor は、
再利用 receipt だけで満たされる検証計画の platform-dependent な
`execution_elapsed_ms` 投影だけを修正し、release lifecycle を再検証します。

## 範囲

- 実行ノードが 0 の場合に elapsed time を正確に 0 とし、receipt identity、reuse
  authorization、fail-closed semantics は変更しません。
- 継承した v0.2.42 の version/release/documentation candidate と三言語投影を維持します。
- merge 前に workspace、hosted quality、Windows-runtime、reference-oracle を通過させ、
  adopter acceptance は release 後の手順として扱います。

## Recovery 境界

WI-412 と PR #377 は hosted Windows CI の失敗を示す immutable historical recovery
evidence です。本 successor だけを active delivery path とし、predecessor bytes や
reference-parity batch は変更しません。

## 検証

明示的な repository path を指定した installed Runtime を使用します。locked workspace
tests、fmt、warning-denied Clippy、release static gates、governance integrity、docs
consistency、hosted quality/Windows/reference-oracle を実行します。

[English](WI-413-release-v0-2-42-windows-ci-retry.md) · [简体中文](WI-413-release-v0-2-42-windows-ci-retry.zh-CN.md)
