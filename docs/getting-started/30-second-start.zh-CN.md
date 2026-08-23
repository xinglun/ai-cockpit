---
author: AI Cockpit maintainers
title: "30 秒开始"
description: "从已安装 Runtime 到 attach repository 的最短安全路径。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# 30 秒开始

使用已经校验的不可变公开 Runtime。先只读检查 repository，再允许第一次
repository-local 写入：

```bash
repo=/path/to/repository
ai-cockpit inspect --repo "$repo"
ai-cockpit attach --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

`inspect` 是只读的第一步。`attach` 只创建 repository-owned `.ai/` 状态；
不会安装 Agent 指令，也不会修改全局 MCP 配置。如果目标不是预期的 Git checkout、
worktree 有无法解释的变更，或 `doctor` 不是 `ok`，请停止。

接着完成[首次校准](first-calibration.zh-CN.md)，再运行[首个 Work Item](first-work-item.zh-CN.md)。
binary 安装与 digest 校验见[安装](installation.zh-CN.md)。

[快速开始](README.zh-CN.md) | [English](30-second-start.md) | [日本語](30-second-start.ja.md)
