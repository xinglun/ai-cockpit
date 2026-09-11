---
author: AI Cockpit maintainers
title: "WI-797 — 残存リソースの cleanup"
description: "v0.2.90 delivery 後、superseded な PR を close し、inventory 済みの branch と worktree だけを削除する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized-release-and-residual-cleanup
workItemId: WI-797-residual-resource-cleanup
lastVerifiedBy: WI-797-residual-resource-cleanup
terminalArchive: .ai/work-items/archive/WI-797-residual-resource-cleanup.contract.json
terminalVerification: .ai/evidence/WI-797-residual-resource-cleanup.verification.json
terminalFinalization: .ai/decisions/WI-797-residual-resource-cleanup.finalize.66aa81cb9de929b6c46e8a4020ae5349db3cbb3b58db47e6e53a3284aa7caaf5.json
terminalDecision: .ai/decisions/WI-797-residual-resource-cleanup.close.json
---

[English](WI-797-residual-resource-cleanup.md) · [简体中文](WI-797-residual-resource-cleanup.zh-CN.md)

# WI-797 — 残存リソースの cleanup

## Objective と boundary

WI-797 は v0.2.90 delivery 後に発見された残存リソースの bounded cleanup を記録
します。対象は inventory 済みの superseded な Pull Request、remote/local branch、
worktree だけです。close 済みの PR は immutable history として保持し、正確な recovery
stash も recoverability のため保持します。

product/Runtime behavior、published release bytes、historical governance records、
無関係な branch/worktree、global Agent/MCP configuration は変更しません。

## Terminal lifecycle state

cleanup inventory と action は Runtime により検証され archive 済みです。この Work
Item の PR #774 は hosted checks を通過し、
`01caa7f3c7bfce9ec0add01ef98ea858495acd42` として merge されました。その後 Runtime は
merge observation と WI-797 の branch/worktree の exact deletion を記録しました。PR
history と四つの recovery stash は保持されています。

## Verification boundary

- archive: `.ai/work-items/archive/WI-797-residual-resource-cleanup.contract.json`
- verification: `.ai/evidence/WI-797-residual-resource-cleanup.verification.json`
- finalization: `.ai/decisions/WI-797-residual-resource-cleanup.finalize.66aa81cb9de929b6c46e8a4020ae5349db3cbb3b58db47e6e53a3284aa7caaf5.json`
- close: `.ai/decisions/WI-797-residual-resource-cleanup.close.json`

三言語の page と parity row は同じ facts と operational consequence を保持
します。page 自体は provider または Runtime gate を bypass する authority を与えません。
