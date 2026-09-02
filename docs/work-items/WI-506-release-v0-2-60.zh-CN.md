---
author: AI Cockpit maintainers
title: "WI-506——发布 v0.2.60 与公开 adopter 验收"
description: "发布下一版身份绑定 Runtime，并在恢复参考源比对前验证公开制品。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-506-release-v0-2-60
terminalArchive: .ai/work-items/archive/WI-506-release-v0-2-60.contract.json
terminalVerification: .ai/evidence/WI-506-release-v0-2-60.verification.json
terminalFinalization: .ai/decisions/WI-506-release-v0-2-60.finalize.json
terminalDecision: .ai/decisions/WI-506-release-v0-2-60.close.json
workItemId: WI-506-release-v0-2-60
---

# WI-506——发布 v0.2.60 与公开 adopter 验收

[English](WI-506-release-v0-2-60.md) · [日本語](WI-506-release-v0-2-60.ja.md)

## 意图

从经过评审的 main 发布 v0.2.60，并在恢复参考源逐文件比对前，使用隔离的
adopter 验收证明不可变公开制品。

## 范围

- 在不改写历史事实的前提下，将 workspace package、lockfile 及当前三语发布/版本文档
  对齐到 v0.2.60。
- 在三语 reference-parity ledger 登记本发布 Work Item。
- 真实 PR 评审和 hosted checks 通过、默认分支同步后，创建 annotated tag；发布 archive、
  checksum、SBOM、provenance 与 manifest。
- 只使用下载的不可变制品执行公开 adopter 与 N-1 验收，包含隔离、证据绑定、
  `not_ready` 脚手架和临时目录清理证明。
- 在本仓库安装公开 binary，并验证 inspect、status、doctor、Agent doctor 及文档晋级健康。

## 范围外

本地参考源、对象/adopter 工程、全局 Agent/MCP 或 Homebrew 配置、源码/workspace binary
fallback、无关 Runtime 重构，以及手工编辑生成治理记录。

## 验收标准

1. Workspace package 与 lockfile 标识 v0.2.60；当前三语发布指引更新且不改写历史发布记录。
2. 在同步默认分支创建 annotated v0.2.60 tag 前，评审 PR 的 hosted checks 全部通过。
3. 公开 Release 提供身份绑定的 archive、SHA256 checksum、SBOM、provenance 与 release manifest。
4. 公开 adopter 与 N-1 验收只使用下载的不可变制品，证明隔离与临时目录清理，并保持
   `first-adopter-smoke=not_ready`。
5. 将公开 binary 安装到本仓库后，inspect、status、doctor、Agent doctor 与 post-close 文档检查健康。
6. 本 Work Item 具备可见人类 Outcome、archive、finalization、close 及精确 branch/worktree 清理记录。

## 验证

```text
cargo test --locked --workspace
```

Pull Request quality gate 是只读的执行前门禁。本发布 Contract 有意保持
`requiredEvidenceClasses` 为空，使门禁可以在完成 receipt 尚不存在时运行；显式
`verify` 步骤负责记录完成证据，后续生命周期门禁再对其校验。

发布与公开验收属于发布后证据。失败发布保持不可变失败历史，不重新标记或复用。

## 边界

Runtime binary 共享，但本仓库的 Protocol、Work Item、evidence、knowledge 与 adapter 保持仓库级隔离。
发布 Runtime 不会隐式 attach 或修改其他仓库。
