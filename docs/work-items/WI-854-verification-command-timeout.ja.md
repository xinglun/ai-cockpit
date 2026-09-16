---
author: AI Cockpit maintainers
title: "WI-854 — 検証コマンドの governed timeout"
description: "互換性と永続化された証拠を保ちながら、検証コマンドの実行時間を制限する。"
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-854-verification-command-timeout
lastVerifiedBy: WI-854-verification-command-timeout
terminalArchive: .ai/work-items/archive/WI-854-verification-command-timeout.contract.json
terminalVerification: .ai/evidence/WI-854-verification-command-timeout.verification.json
terminalDecision: .ai/decisions/WI-854-verification-command-timeout.close.json
---

[English](WI-854-verification-command-timeout.md) · [简体中文](WI-854-verification-command-timeout.zh-CN.md)

# WI-854 — 検証コマンドの governed timeout

WI-854 は Contract が認可した有限の timeout を検証コマンドに追加します。
既定値は互換性を維持し、不正な override は spawn 前に拒否します。timeout
した試行は exit code、deadline、経過時間、ログの identity を保存し、timeout
値は検証再利用の identity に含めるため、変更時に stale evidence を再利用しません。
