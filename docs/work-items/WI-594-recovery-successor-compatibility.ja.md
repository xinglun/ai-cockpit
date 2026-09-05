---
author: AI Cockpit maintainers
title: "WI-594 — recovery successor compatibility Runtime 修正"
description: "有効な successor 再検証後に archived predecessor を close する append-only Runtime path を提供します。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-594-recovery-successor-compatibility
lastVerifiedBy: WI-594-recovery-successor-compatibility
terminalArchive: .ai/work-items/archive/WI-594-recovery-successor-compatibility.contract.json
terminalVerification: .ai/evidence/WI-594-recovery-successor-compatibility.verification.json
terminalFinalization: .ai/decisions/WI-594-recovery-successor-compatibility.finalize.398ee773f1fe88e7e80c09c29b12129d2e1289bc35e7a555421836702d86dc49.json
terminalDecision: .ai/decisions/WI-594-recovery-successor-compatibility.close.json
---

[English](WI-594-recovery-successor-compatibility.md) · [简体中文](WI-594-recovery-successor-compatibility.zh-CN.md)

# WI-594 — recovery successor compatibility Runtime 修正

## Objective

古い recovery/finalization bytes を書き換えず、有効な successor/revalidation
record で archived predecessor を close できるようにします。無効・foreign・
contradictory record は fail-closed のままです。

## Boundary

repository-bound の append-only Runtime change です。PR finalization を
direct merge に再分類せず、object repository や release artifact は変更しません。

## Verification

archive、current verification evidence、verified finalization head、structured
close decision が揃い、head は reviewed PR、merge、branch、worktree cleanup の事実に束縛されています。
