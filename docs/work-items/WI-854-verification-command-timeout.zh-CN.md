---
author: AI Cockpit maintainers
title: "WI-854——受治理的验证命令超时"
description: "在保持兼容和持久化证据的同时，为验证命令增加有界执行时间。"
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-854-verification-command-timeout
lastVerifiedBy: WI-854-verification-command-timeout
terminalArchive: .ai/work-items/archive/WI-854-verification-command-timeout.contract.json
terminalVerification: .ai/evidence/WI-854-verification-command-timeout.verification.json
terminalDecision: .ai/decisions/WI-854-verification-command-timeout.close.json
---

[English](WI-854-verification-command-timeout.md) · [日本語](WI-854-verification-command-timeout.ja.md)

# WI-854——受治理的验证命令超时

WI-854 为验证命令增加由 Contract 授权的有限超时。默认行为保持兼容；
无效覆盖值在启动子进程前拒绝；超时尝试保留退出码、截止时间、耗时和日志
身份，便于后续诊断。有效超时属于验证复用身份，变更超时不会静默复用过期证据。
