---
author: AI Cockpit maintainers
title: "WI-503 — 发布 v0.2.59 并完成公开 adopter 验收"
description: "发布下一个绑定身份的 Runtime 版本，并在恢复参考源比对前证明公开构件可用。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-503-release-v0-2-59
terminalArchive: .ai/work-items/archive/WI-503-release-v0-2-59.contract.json
terminalVerification: .ai/evidence/WI-503-release-v0-2-59.verification.json
terminalFinalization: .ai/decisions/WI-503-release-v0-2-59.finalize.json
terminalDecision: .ai/decisions/WI-503-release-v0-2-59.close.json
workItemId: WI-503-release-v0-2-59
---

# WI-503 — 发布 v0.2.59 并完成公开 adopter 验收

[English](WI-503-release-v0-2-59.md) · [日本語](WI-503-release-v0-2-59.ja.md)

## 意图

在 WI-502 终态文档门禁修复后发布不可变的 `v0.2.59` Runtime。公开 binary
必须能在隔离的全新 adopter 中从零运行，通过 N-1 验收，并在开始下一批参考源
逐文件比较前安装到本工程。

## 范围

- 在不改写历史事实的前提下，将 workspace package、锁文件和当前三语发布/版本
  指南统一到 `v0.2.59`。
- 在三语 reference-parity 台账登记本发布 Work Item。
- 在同步后的 `main` 上先通过 reviewed hosted PR，再创建 annotated tag；发布
  archive、校验和、SBOM、provenance 与 manifest。
- 只使用下载的不可变构件执行公开 adopter 与 N-1 验收，包含隔离、evidence
  绑定、`not_ready` scaffold 与临时目录清理证明。
- 将已发布 binary 安装到本工程，验证 inspect、status、doctor、Agent doctor
  和文档 promotion 健康状态。

## 范围之外

本地参考源、对象/adopter 工程、全局 Agent/MCP 或 Homebrew 配置、源码/工作区
binary fallback、无关 Runtime 重构，以及手工编辑生成的治理记录。

## 验收标准

1. workspace package、`Cargo.lock` 和当前三语发布文档识别 `v0.2.59`，且不改写
   之前版本的历史事实。
2. reviewed PR 的 hosted checks 在从同步后的默认分支创建 annotated `v0.2.59`
   tag 前全部通过。
3. 公开 Release 提供绑定身份的 archive、SHA256 校验和、SBOM、provenance 和
   release manifest。
4. 公开 adopter 与 N-1 验收只使用下载的不可变构件，证明隔离与临时目录清理，
   并保持 `first-adopter-smoke=not_ready`。
5. 已发布 binary 安装到本工程后，inspect、status、doctor、Agent doctor 和
   close 后文档检查保持健康。
6. 本 Work Item 在发布前具有可见的人类 Outcome、archive、finalization、close
   和精确的分支/worktree 清理记录。

## 验证

```text
cargo test --locked --workspace
```

发布与公开验收属于发布后的 evidence。失败的发布保持不可变失败历史，不重新标记
也不复用。

## 边界

Runtime binary 共享，但本工程的 Protocol、Work Item、evidence、knowledge 和
adapter 保持 repository-local。发布 Runtime 不会隐式 attach 或修改其他工程。
