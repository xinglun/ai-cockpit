---
author: AI Cockpit maintainers
title: "WI-341 — archived pull request の Runtime shadow"
workItemId: WI-341-runtime-shadow-archived-state
description: "active Contract がある場合だけ immutable Runtime shadow を実行し、archive 後の pull request を誤って失敗させない。"
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-341-runtime-shadow-archived-state
terminalArchive: .ai/work-items/archive/WI-341-runtime-shadow-archived-state.contract.json
terminalVerification: .ai/evidence/WI-341-runtime-shadow-archived-state.verification.json
terminalFinalization: .ai/decisions/WI-341-runtime-shadow-archived-state.finalize.cd2a636790b3f88c1ffc793bfee4a02e4d068f26788080b34472110e69deaf4e.json
terminalDecision: .ai/decisions/WI-341-runtime-shadow-archived-state.close.json
---

# WI-341 — archived pull request の Runtime shadow

この Work Item は、public Runtime shadow とその artifact upload を active
Contract に結び付けます。`finish` と `archive` の後に active Contract がない
archived pull request でも、通常の repository gate は実行し、Contract 不在を
理由に誤って失敗させません。

変更範囲は workflow 条件、その回帰 assertion、同期された reference 文書に
限定します。Runtime Core、release artifact、adopter acceptance、provider 設定は
変更しません。

受入れは archive Contract と verification evidence に記録し、review 済み
pull request の merge と branch/worktree の正確な cleanup を確認してから
close しました。

[English](WI-341-runtime-shadow-archived-state.md) ·
[简体中文](WI-341-runtime-shadow-archived-state.zh-CN.md)
