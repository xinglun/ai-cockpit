---
author: AI Cockpit maintainers
title: "WI-596——v0.2.78 发布与对象工程恢复交接"
description: "发布包含归档 Work Item 恢复兼容修复的 Runtime，并验收公开产物。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-596-release-v0-2-78
lastVerifiedBy: WI-596-release-v0-2-78
---

[English](WI-596-release-v0-2-78.md) · [日本語](WI-596-release-v0-2-78.ja.md)

# WI-596——v0.2.78 发布与对象工程恢复交接

## 目标

从已审查且同步的默认分支发布 v0.2.78。该补丁发布公开已经审查的
Contract amendment predecessor close 恢复修复，保留失败的 v0.2.77 标签作为不可变历史，
并为对象工程提供可重复的公开产物验收交接。

## 边界

本 Work Item 只修改包版本元数据和发布文档。Runtime 源码行为、对象工程、全局
Agent/MCP 配置、历史证据字节及参考源实现均不在范围内。公开 adopter 与 N-1
验收属于发布后证据，只能使用下载的不可变产物，不得使用源码 checkout 或 workspace 构建。

## 验收

1. Cargo 元数据和锁文件解析为 v0.2.78；v0.2.77 保留为失败的未公开历史，不能重新打标签或作为安装基线。
2. 发布策略检查把 annotated tag、五个目标产物、校验和、SBOM/来源证明及 Runtime identity 绑定到同一个审查提交。
3. 发布后的 adopter 与 N-1 harness 只使用 v0.2.78 产物，证明禁止写入根和临时运行目录在成功/失败路径均被清理。
4. 不修改对象工程；发布后向其团队提供准确的 compatibility、recovery 和 revalidation 命令。
5. 发布或 adopter 失败时保留已发布事实并记录 failure receipt，不改写失败标签或历史证据。
6. 英文、简体中文和日文 release/versioning 文档对当前公开基线和安装命令保持一致。

## 验证

执行 Contract 声明的 locked workspace、文档、Parity、发布策略、staged acceptance 和发布后公开产物检查。只有在审查 PR 检查通过、v0.2.78 Release 发布、adopter/N-1 receipt 保留且精确分支/worktree 清理完成后，才完成 lifecycle。
