---
author: AI Cockpit maintainers
title: "WI-1110-task-authorization-usability — 有限任务授权与可用性"
description: "Source-stage implementation and later conditional delivery remain separate."
audience:
  - maintainer
  - reviewer
workItemId: WI-1110-task-authorization-usability
status: implemented
authority: native-contract
lastVerifiedBy: WI-1110-task-authorization-usability
terminalArchive: .ai/work-items/archive/WI-1110-task-authorization-usability.contract.json
terminalVerification: .ai/evidence/WI-1110-task-authorization-usability.verification.json
terminalFinalization: .ai/decisions/WI-1110-task-authorization-usability.finalize.json
terminalDecision: .ai/decisions/WI-1110-task-authorization-usability.close.json
---

# WI-1110-task-authorization-usability — 有限任务授权与可用性

实现已验证、合并，并经单独确认正式关闭。sequence5 的真实用户计划确认已记录；当时17项必需场景全部未验证，后续结果以原生证据为准。本治理工作项和共享 Runtime 均未启用候选授权政策。

授权绑定仓库、工作项、目标、文件、操作、验证标准及显式调用者／来源。完成、撤回或已声明的截止时间终止未来授权。源码变化可要求重验，不自动要求再次授权。

本工作项同时消除已发现的跨平台警告，加入准确的 Linux／Windows 无警告质量检查。staged CLI／MCP 起项先完整保存声明，再进行 preflight／checkpoint；既有严格审阅和原生历史检查保留。

源码阶段真实验证和独立复审已通过。PR #1072 已普通合并；实际 main CI 40164544197 的四项检查、43 项门禁和 14 个包通过，Linux／Windows 绑定一致且未检出警告。精确资源在可恢复备份核验后已清理，最终化复核通过，用户随后单独确认了 close。这些收尾记录的发布仍须完成其自身 PR 复审、CI 和 main 核验。

token 用量未知，部署收益未测量，旧失败保留。不发布版本，不启用生产、外部 B 或替换共享 Runtime。

当前归档 Contract：`.ai/work-items/archive/WI-1110-task-authorization-usability.contract.json`。

已记录的终态证据（不表示本收尾 PR 已合并）：

- archive `.ai/work-items/archive/WI-1110-task-authorization-usability.contract.json`
- verification `.ai/evidence/WI-1110-task-authorization-usability.verification.json`
- finalization `.ai/decisions/WI-1110-task-authorization-usability.finalize.json`
- close `.ai/decisions/WI-1110-task-authorization-usability.close.json`
