---
author: AI Cockpit maintainers
title: "WI-1110-task-authorization-usability — 有限任务授权与可用性"
description: "Source-stage implementation and later conditional delivery remain separate."
audience:
  - maintainer
  - reviewer
workItemId: WI-1110-task-authorization-usability
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1110-task-authorization-usability
---

# WI-1110-task-authorization-usability — 有限任务授权与可用性

实现进行中。sequence3 的真实用户审阅已记录；17项必需场景仍全部未验证。本治理工作项和共享 Runtime 均未启用候选授权政策。

授权绑定仓库、工作项、目标、文件、操作、验证标准及显式调用者／来源。完成、撤回或已声明的截止时间终止未来授权。源码变化可要求重验，不自动要求再次授权。

本工作项同时消除已发现的跨平台警告，加入准确的 Linux／Windows 无警告质量检查。staged CLI／MCP 起项先完整保存声明，再进行 preflight／checkpoint；既有严格审阅和原生历史检查保留。

finish 前要求源码阶段真实验证和独立复审。最终 HEAD 的 Windows／Linux CI、普通 PR 合并、实际 main 核验及可恢复资源清理仍是后续交付义务；真正 close 另行确认。

token 用量未知，部署收益未测量，旧失败保留。不发布版本，不启用生产、外部 B 或替换共享 Runtime。

Current native Contract: `.ai/work-items/active/WI-1110-task-authorization-usability.contract.json`.

Planned terminal records (not asserted present):

- archive `.ai/work-items/archive/WI-1110-task-authorization-usability.contract.json`
- verification `.ai/evidence/WI-1110-task-authorization-usability.verification.json`
- finalization `.ai/decisions/WI-1110-task-authorization-usability.finalize.json`
- close `.ai/decisions/WI-1110-task-authorization-usability.close.json`
