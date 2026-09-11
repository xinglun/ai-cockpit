---
author: AI Cockpit maintainers
title: "WI-797 — 残存リソースの cleanup"
description: "v0.2.90 delivery 後、superseded な PR を close し、inventory 済みの branch と worktree だけを削除する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-release-and-residual-cleanup
workItemId: WI-797-residual-resource-cleanup
lastVerifiedBy: WI-797-residual-resource-cleanup
terminalArchive: .ai/work-items/archive/WI-797-residual-resource-cleanup.contract.json
terminalVerification: .ai/evidence/WI-797-residual-resource-cleanup.verification.json
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

## Current lifecycle state

cleanup inventory と action は Runtime により検証され archive 済みです。この Work
Item の provider merge、finalization、close、exact cleanup が terminal lifecycle
boundary であり、それらの record が検証されるまではこのページを `in_progress` の
まま保持します。

## Verification boundary

- archive: `.ai/work-items/archive/WI-797-residual-resource-cleanup.contract.json`
- verification: `.ai/evidence/WI-797-residual-resource-cleanup.verification.json`
- reviewed PR と Runtime finalization/close record は、対応する provider/Runtime
  transition が実行された後にだけ追加します。

三言語の page と parity row は同じ facts と pending operational consequence を保持
します。page 自体は provider または Runtime gate を bypass する authority を与えません。
