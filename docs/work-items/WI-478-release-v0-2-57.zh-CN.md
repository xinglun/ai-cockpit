---
author: AI Cockpit maintainers
title: "WI-478——发布 v0.2.57 与公开 adopter 验收"
description: "在 v0.2.56 失败发布之后按修正顺序发布 Runtime，并用隔离环境验证公开制品。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implementation_active
authority: canonical
lastVerifiedBy: WI-478-release-v0-2-57
workItemId: WI-478-release-v0-2-57
---

# WI-478——发布 v0.2.57 与公开 adopter 验收

[English](WI-478-release-v0-2-57.md) · [日本語](WI-478-release-v0-2-57.ja.md)

## 意图

在 v0.2.56 失败发布之后，按修正后的顺序发布新的不可变 Runtime。公开
binary 必须能够在隔离的 adopter 中从零使用，然后安装到本仓库；不修改
本地参考源或任何对象工程。

## 范围

- 将 workspace package、lockfile 和当前三语发布/版本文档统一到 `v0.2.57`，并保留失败发布历史与历史 evidence。
- 在三语 reference-parity 台账登记本 Work Item。
- 先通过 reviewed hosted PR，再创建 annotated tag；发布 archive、校验和、SBOM、provenance、manifest 与 Runtime identity。
- 只使用公开下载制品运行 adopter 与 N-1 验收，验证禁止写入、evidence 绑定和临时目录清理。
- 在本仓库安装已发布 binary，检查 inspect、status、doctor、Agent doctor 与 ready-on-base。
- 在打 tag 前完成 verification、人工 Outcome、archive、资源收尾、close、文档 promotion 及精确分支/worktree 清理。

## 不在范围内

本地参考源、`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`、其他 adopter、全局 Agent/MCP 或 Homebrew 配置、源码/工作区 binary fallback、无关 Runtime 重构，以及手动修改生成的 status、evidence、receipt、archive、decision。

## 验收标准

1. workspace package、lockfile 与必需三语发布文档在不改写历史事实的前提下标识 `v0.2.57`。
2. reviewed PR 的 hosted checks 全部通过后才 merge；annotated `v0.2.57` tag 必须精确指向同步后的 reviewed 默认分支，并且只能在本 Work Item close 后创建。
3. 公开 Release 提供 archive、SHA256、SBOM、provenance 与绑定 identity 的 release manifest。
4. adopter 与 N-1 只使用不可变公开制品，保留 `first-adopter-smoke=not_ready`，绑定 repository/Runtime identity，证明隔离并证明成功/失败路径都会清理临时根目录。
5. 在本仓库安装公开 binary；inspect、status、doctor、Agent doctor 和文档 promotion 证明 attach 健康且可继续工作。
6. Work Item 产生包含 `🟢`/`🟡`/`🔴` 的可见人工 Outcome，然后完成 archive/finalization/close 和精确清理。

## 验证

```text
cargo test --locked --workspace
```

发布与公开验收属于发布后 evidence。失败发布保持不可变失败历史，不能改标或复用。

## 边界

安装的 Runtime 可以共享，但本仓库的 Protocol、Work Item、evidence、knowledge 和 adapter 始终私有。发布 Runtime 不会隐式 attach 或修改目标仓库。
