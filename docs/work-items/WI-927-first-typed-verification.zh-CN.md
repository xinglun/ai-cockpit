---
author: AI Cockpit maintainers
workItemId: WI-927-first-typed-verification
title: 首次 typed required verification 执行边界
description: 允许首次声明的 required verification 创建正式 receipt，同时不放宽终态生命周期门禁。
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: authorized
lastVerifiedBy: WI-927-first-typed-verification
---

# WI-927 — 首次 typed verification 执行边界

本 Work Item 继续修复 [Issue #893](https://github.com/xinglun/ai-cockpit/issues/893)。
前置 WI 的 Contract 因治理字段格式错误而按 Runtime 规则保留并替换；本 WI
修复 Runtime 的前置循环：有效的 checkpointed Work Item 在 Summary 尚无
typed required verification 条目时，不能启动首次已声明验证，而这些条目只能
由本次验证正式写入。

当 Contract、repository snapshot、checkpoint 和非 red preflight 都是当前有效
状态，且唯一待处理的治理事实是缺少 typed verification 条目时，Runtime 可以
执行一次已声明检查。正式的 identity-bound receipt 仍必须先写入 Summary，
`finish`、`archive` 和 `close` 才能继续。失败、重复、过期、外部来源或格式错误
的证据，仍在进程启动前 fail-closed。

用户可见收益在定向与 hosted evidence 完整前保持未声明。
