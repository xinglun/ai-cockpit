---
author: Ray
title: "WI-353 — Runtime recovery delivery binding"
workItemId: WI-353-runtime-recovery-delivery-binding
description: "WI-351 の不変な履歴を保持したまま、復旧した Runtime delivery を実際の reviewed PR に bind する。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: translation
canonical: docs/work-items/WI-353-runtime-recovery-delivery-binding.md
lastVerifiedBy: WI-353-runtime-recovery-delivery-binding
predecessor: WI-351-runtime-recovery-binding
terminalArchive: .ai/work-items/archive/WI-353-runtime-recovery-delivery-binding.archive.json
terminalVerification: .ai/evidence/WI-353-runtime-recovery-delivery-binding.verification.json
capabilityClaims:
  - recovery_delivery_binding
---

# WI-353 — Runtime recovery delivery binding

[English](WI-353-runtime-recovery-delivery-binding.md) · [简体中文](WI-353-runtime-recovery-delivery-binding.zh-CN.md)

## Intent と boundary

この successor Work Item は WI-351 の immutable archive を保持し、復旧した Runtime
delivery を実際の reviewed GitHub PR #318 に bind します。finalization の前に、正確な
`main`/`origin` base、branch、worktree、Runtime 自身の evidence を記録します。

対象は recovery binding、fail-closed regression coverage、およびこの delivery に必要な
governance record に限定します。Sentinel business code、Provider discovery、trading
decision、gate、execution、position sizing、global configuration、WI-351 history の書き換え
は対象外です。

## Verification と delivery boundary

- successor を archive する前に locked workspace tests、formatting check、clippy を通過させます。
- PR resource context は [PR #318](https://github.com/xinglun/ai-cockpit/pull/318) に bind し、
  base は `main`/`origin`、専用の recovery worktree を使用します。
- Provider finalization、正確な branch/worktree cleanup、structured close は reviewed PR
  merge 後まで保留します。merge 前の状態を完了済みとは報告しません。

predecessor の archive と evidence は immutable のまま保持し、この successor が delivery
と finalization の boundary を担います。predecessor bytes は書き換えません。
