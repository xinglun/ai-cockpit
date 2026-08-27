---
author: AI Cockpit maintainers
title: "WI-322——生命周期入口安全"
workItemId: WI-322-lifecycle-entry-safety
description: "在 repository closure 或 start 前基础条件未解决时 fail-closed。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-322-lifecycle-entry-safety
terminalArchive: .ai/work-items/archive/WI-322-lifecycle-entry-safety.contract.json
terminalVerification: .ai/evidence/WI-322-lifecycle-entry-safety.verification.json
terminalFinalization: .ai/decisions/WI-322-lifecycle-entry-safety.finalize.json
terminalDecision: .ai/decisions/WI-322-lifecycle-entry-safety.close.json
---

# WI-322——生命周期入口安全

## 意图与边界

当 repository 仍有缺少有效 close decision 的 archived Work Item、start 前的非治理变更、
detached branch 或已知的 branch/base 不一致时，阻止新的治理 Work Item 启动。无法确定的
repository 元数据保持 `unknown`，绝不投影为绿色 readiness。

检查按 repository 隔离，不创建进程级 current project。显式 recovery continuation 继续使用
既有 recovery 路径。

## 范围与验收

- `work-item new` 与 `start` 对未解决的 archived closure fail-closed，并保留不可变 archive 字节。
- `status` 暴露确定性的 `readiness`/`readyOnBase` 事实与 blockers。
- 拒绝 start 前已有的用户变更，同时允许 Runtime 自有 `.ai` 写入。
- 在不访问网络的情况下检查可发现的 remote default ref；缺少元数据时 readiness 为 `unknown`。
- 两个 repository context 保持隔离，并同步三语命令与 Agent workflow 文档。

## 验证

通过 locked workspace 测试、lifecycle-entry 回归、文档 gate 和 hosted CI 验证。所有 repository-bound
Runtime 命令都显式使用 `--repo` 路径。

[English](WI-322-lifecycle-entry-safety.md) ·
[日本語](WI-322-lifecycle-entry-safety.ja.md)
