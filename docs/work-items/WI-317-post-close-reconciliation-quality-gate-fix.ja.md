---
author: AI Cockpit maintainers
title: "WI-317 — post-close reconciliation quality gate fix"
workItemId: WI-317-post-close-reconciliation-quality-gate-fix
description: "不変の履歴を書き換えず、W316 の bounded quality-gate correction を再配信する。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-317-post-close-reconciliation-quality-gate-fix
terminalArchive: .ai/work-items/archive/WI-317-post-close-reconciliation-quality-gate-fix.contract.json
terminalVerification: .ai/evidence/WI-317-post-close-reconciliation-quality-gate-fix.verification.json
terminalFinalization: .ai/decisions/WI-317-post-close-reconciliation-quality-gate-fix.finalize.ef51268d4b7db25d8f189d4bbd6b87faa306e48150888b884c32006a428f4f1d.json
terminalDecision: .ai/decisions/WI-317-post-close-reconciliation-quality-gate-fix.close.json
---

# WI-317 — post-close reconciliation quality gate fix

## Intent と boundary

W316 は immutable な archived delivery であり、hosted quality run は三つの bounded defect
を示しました。parity row が recovery decision に追随していないこと、中国語の
resource-finalization page に明示的な close order rule がないこと、promotion regression が
古い error message を assert していることです。本 successor は W316 の bytes を保持し、
最新の `origin/main` からこれらの correction だけを再配信します。

## Scope と acceptance

- W316 Contract、evidence、Outcome、Events、archive、recovery、PR #280 の履歴は byte-for-byte 不変です。
- 三つの parity ledger は W312 を Implemented、W314/W315 を Recovered として正しく分類し、正確な recovery evidence path を示します。
- 三言語の resource-finalization workflow page は `finalize-verify` 成功前の `close` を禁止します。
- promotion regression は現在の helper error と一致し、gate を弱めずに focused/full/hosted quality gate が通ります。
- successor は最新 remote default base から開始し、reviewed hosted checks 後にのみ merge し、finalize、close、exact cleanup を完了します。

## Verification

installed Runtime を governance interface として使用し、documentation/promotion/resource-finalization regression、documentation acceptance、
single-process の locked workspace test、および正確な reviewed branch の hosted CI を実行します。

## Related history

- W316: hosted quality check に拒否された immutable delivery。bytes は historical evidence として保持します。
- W317: その run で発見された問題だけを修正する bounded successor。

[English](WI-317-post-close-reconciliation-quality-gate-fix.md) ·
[简体中文](WI-317-post-close-reconciliation-quality-gate-fix.zh-CN.md)
