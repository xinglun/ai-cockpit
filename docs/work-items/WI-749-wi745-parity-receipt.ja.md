---
author: AI Cockpit maintainers
title: "WI-749 — WI-745 復旧 parity receipt"
description: "三言語の parity 投影に現在の不変な復旧決定を登録する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-749-wi745-parity-receipt
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-749-wi745-parity-receipt
terminalArchive: .ai/work-items/archive/WI-749-wi745-parity-receipt.contract.json
terminalVerification: .ai/evidence/WI-749-wi745-parity-receipt.verification.json
terminalFinalization: .ai/decisions/WI-749-wi745-parity-receipt.finalize.json
terminalDecision: .ai/decisions/WI-749-wi745-parity-receipt.close.json
---

# WI-749 — WI-745 復旧 parity receipt

これは docs-only の Work Item であり、復旧された WI-745 の行が Runtime
の現在の復旧 receipt を参照するよう、三つの reference-parity 投影を更新
する。WI-745 と WI-746 の履歴記録は変更しない。

[English](WI-749-wi745-parity-receipt.md) · [简体中文](WI-749-wi745-parity-receipt.zh-CN.md)

## 範囲

- 三つの reference-parity 文書の WI-745 行だけを更新する。
- `Recovered` の状態と既存の verification バインディングを維持する。
- Runtime の挙動、テスト、履歴 receipt、第四・第五部分の独立トラックは変更しない。
