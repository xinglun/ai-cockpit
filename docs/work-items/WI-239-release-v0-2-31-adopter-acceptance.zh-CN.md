---
author: AI Cockpit maintainers
title: "WI-239——v0.2.31 公开 Release adopter 验收"
workItemId: WI-239-release-v0-2-31-adopter-acceptance
description: "用隔离的新 adopter 验收不可变的 v0.2.31 Release，并把安装的 Runtime 绑定到本仓库。"
audience:
  - maintainer
  - reviewer
  - adopter
status: current
authority: canonical
---

# WI-239——v0.2.31 公开 Release adopter 验收

本 Work Item 定义 v0.2.31 的发布后验收边界。Runtime 操作只使用公开
Release archive；源码 checkout 和 workspace binary 不作为 Runtime fallback。

## 验收边界

- 公开 tag 不是 draft 或 prerelease，archive、manifest、checksums 对不可变
  Release identity 一致。
- 在隔离的 HOME、XDG_CONFIG_HOME、TMPDIR、CARGO_HOME 中创建全新 adopter，
  attach、profile 确认、Agent doctor、repository identity、isolation 均通过。
- `first-adopter-smoke` 保持 `not_ready`；脚手架不擅自生成 intent、scope、验收
  标准、authority、approval 或 completion。
- 第二次 verify 复用证据且不重新启动进程，完整 lifecycle 以结构化 human decision
  receipt close。
- 成功时校验并删除临时 acceptance run root。
- 安装的 v0.2.31 binary 对本仓库显式执行 repository-bound inspect、status、doctor
  和 Agent doctor 并通过。

## 证据

验收 receipt、Runtime identity、Release manifest、隔离 manifest、证据复用输出、
lifecycle evidence 和安装 Runtime 检查保存在
`.ai/evidence/WI-239-release-v0-2-31-adopter-acceptance/`。

## 参考

- [发布与分发](../release/distribution.zh-CN.md)
- [Outcome 报告](../reference/outcome-report.zh-CN.md)
- [Agent 工作流](../reference/agent-workflow.zh-CN.md)
