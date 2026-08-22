---
author: AI Cockpit maintainers
workItemId: WI-137-release-v0-2-11
title: 发布 v0.2.11 并完成不可变 adopter 验收
description: 发布已合并的 Runtime 修复，并在隔离 adopter 与当前工程流程中验证公开 binary。
audience:
  - adopter
  - maintainer
status: release-preparation
authority: canonical
lastVerifiedBy: WI-137-release-v0-2-11
---

# WI-137——发布 v0.2.11 并完成不可变 adopter 验收

## 意图

发布包含 WI-135 repository-bound retention/evidence 校验和 WI-136 Task
Outcome 报告的首个不可变 Runtime，并证明下载的公开 binary 可以治理全新的
adopter 以及当前工程。

## 范围与边界

- 将 workspace 与当前发布文档更新到 `v0.2.11`。
- 执行源码质量、发布策略、全新 adopter 和 N-1 验收。
- 只安装公开发布的 v0.2.11 artifact，验证当前工程。
- 将发布验收 artifact 与 repository 历史分开保存。

本 WI 不新增 Runtime 功能、不改写历史 evidence、不修改全局 Agent/MCP
配置、不修改外部 Homebrew tap，也不使用源码或 workspace binary 作为发布验收证据。

## 验收标准

1. Cargo metadata、archive 名称、manifest 和三语文档一致指向 v0.2.11；历史 N-1 引用保持明确。
2. 全新 adopter 验收只下载并校验不可变 v0.2.11 Release，保留
   `first-adopter-smoke = not_ready`，记录 repository/runtime identity、证据复用、生命周期、隔离和清理 receipt。
3. N-1 验收证明 v0.2.10 → v0.2.11 兼容，不改写旧字节或 Release truth。
4. 安装的公开 binary 在当前工程通过 inspect、status、doctor、Agent doctor 和面向人的 Outcome；该 binary 能读取 WI-136 新报告。

## 验证与证据

必须保留 workspace verification receipt、公开 fresh-adopter receipt、N-1
升级 receipt、Runtime identity（版本、archive digest、binary digest、target、下载源）和最终 Outcome。
