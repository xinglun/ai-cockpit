---
author: AI Cockpit maintainers
title: "WI-822 — resource finalization base binding"
description: "実際にレビューされた PR base に provider finalization を束縛し、close recovery を保持する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-822-resource-finalization-base-binding
lastVerifiedBy: WI-822-resource-finalization-base-binding
terminalArchive: .ai/work-items/archive/WI-822-resource-finalization-base-binding.contract.json
terminalVerification: .ai/evidence/WI-822-resource-finalization-base-binding.verification.json
terminalFinalization: .ai/decisions/WI-822-resource-finalization-base-binding.finalize.json
terminalDecision: .ai/decisions/WI-822-resource-finalization-base-binding.close.json
---

[English](WI-822-resource-finalization-base-binding.md) · [简体中文](WI-822-resource-finalization-base-binding.zh-CN.md)

# WI-822 — resource finalization base binding

## Intent と boundary

WI-822 は Contract base、レビュー済み PR の比較 base、公開実行の identity を分離する。
merge 済み PR、正確な branch、正確な worktree state を確認した後だけ provider cleanup
を記録する。predecessor の archive と historical evidence は変更しない。

## Verification

Runtime の正式検証は予定された 12 個すべての workspace node に合格した。provider
finalization receipt は merge 済み PR と正確な branch/worktree cleanup を記録し、close
decision はその finalization head に bind している。
