---
author: AI Cockpit maintainers
title: "WI-491——发布 v0.2.58 与公开 adopter 验收"
description: "发布下一版绑定身份的 Runtime，并在恢复参考源比对前证明公开制品可用。"
audience:
  - adopter
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-491-release-v0-2-58
workItemId: WI-491-release-v0-2-58
---

# WI-491——发布 v0.2.58 与公开 adopter 验收

[English](WI-491-release-v0-2-58.md) · [日本語](WI-491-release-v0-2-58.ja.md)

## 意图

在 WI-490 的文档终态门修复之后发布绑定身份的 `v0.2.58` Runtime。公开
binary 必须能在隔离 adopter 中从零运行，通过 N-1 验收并安装到本仓库，
之后才能开始下一批参考源逐文件比对。

## 范围

- 将 workspace package、lockfile 和当前三语发布/版本文档统一到 `v0.2.58`，保留历史事实。
- 在三语 reference-parity 台账登记本发布 Work Item。
- 先通过 reviewed hosted PR，再从同步的 `main` 创建 annotated tag；发布 archive、校验和、SBOM、provenance 与 manifest。
- 只用下载的不可变制品运行公开 adopter 与 N-1 验收，验证隔离、证据绑定、`not_ready` 脚手架和清理。
- 在本仓库安装公开 binary，检查 inspect、status、doctor、Agent doctor 与文档 promotion。

## 不在范围内

本地参考源、对象/adopter 工程、全局 Agent/MCP 或 Homebrew 配置、源码/工作区
binary fallback、无关 Runtime 重构，以及手动修改生成的治理记录。

## 验收标准

1. workspace package、`Cargo.lock` 与当前三语发布文档标识 `v0.2.58`，不改写历史发布事实。
2. reviewed PR 的 hosted checks 通过后才 merge；annotated `v0.2.58` tag 精确指向同步后的 reviewed 默认分支。
3. 公开 Release 提供绑定身份的 archive、SHA256、SBOM、provenance 与 release manifest。
4. 公开 adopter 与 N-1 只使用不可变下载制品，证明隔离和临时根目录清理，并保留 `first-adopter-smoke=not_ready`。
5. 公开 binary 安装到本仓库后，inspect、status、doctor、Agent doctor 与 post-close 文档检查保持健康。
6. 本 Work Item 在发布前产生可见人工 Outcome，并完成 archive、finalization、close 及精确 branch/worktree 清理。

## 验证

```text
cargo test --locked --workspace
```

发布和公开验收属于发布后 evidence。失败发布保持不可变失败历史，不能改标或复用。

## 边界

Runtime binary 可以共享，但本仓库的 Protocol、Work Item、evidence、knowledge
和 adapter 始终是 repository-local。发布 Runtime 不会隐式 attach 或修改其他仓库。
