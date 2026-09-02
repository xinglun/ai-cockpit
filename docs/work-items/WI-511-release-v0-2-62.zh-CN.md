---
author: AI Cockpit maintainers
title: "WI-511——发布 v0.2.62 与公开 adopter 验收"
description: "发布下一个绑定身份的 Runtime 版本，在继续参考源比对前验证公开 artifact。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-511-release-v0-2-62
terminalArchive: .ai/work-items/archive/WI-511-release-v0-2-62.contract.json
terminalVerification: .ai/evidence/WI-511-release-v0-2-62.verification.json
terminalFinalization: .ai/decisions/WI-511-release-v0-2-62.finalize.json
terminalDecision: .ai/decisions/WI-511-release-v0-2-62.close.json
workItemId: WI-511-release-v0-2-62
---

# WI-511——发布 v0.2.62 与公开 adopter 验收

[English](WI-511-release-v0-2-62.md) · [日本語](WI-511-release-v0-2-62.ja.md)

## 意图

从经过审查的 main 发布 v0.2.62，用隔离的 adopter 与 N-1 验收证明不可变
公开 artifact，在本仓库安装该发布 binary，然后继续下一批参考源逐文件比对。

## 范围

- 将 workspace package、lockfile 和当前三语发布/版本文档对齐到 v0.2.62，保留历史事实。
- 在三语 reference-parity 台账登记本发布 Work Item。
- 同步 main 后先通过审查过的 hosted PR，再创建 annotated tag；发布 archive、校验和、SBOM、provenance 与 manifest。
- 仅使用下载的不可变 artifact 执行 public adopter 与 N-1 验收，包含隔离、evidence 绑定、not_ready scaffold 与临时根清理证明。
- 在本仓库安装公开 binary，验证 inspect、status、doctor、Agent doctor 和文档晋级健康状态。

## 范围外

本地参考源、对象/adopter 工程、全局 Agent/MCP 或 Homebrew 配置、源码/workspace binary fallback、无关 Runtime 重构，以及手工编辑生成治理记录。

## 验收标准

1. workspace package 与 lockfile 标识 v0.2.62；当前三语发布文档更新且不改写历史发布记录。
2. 在同步 main 创建 annotated v0.2.62 tag 前，审查过的 PR hosted checks 全部通过。
3. 公开 Release 提供绑定身份且字节一致的 archive、SHA256 校验和、SBOM、provenance 与 release manifest。
4. public adopter 与 N-1 验收只使用下载的不可变 artifact，证明隔离和临时根清理，并保留 first-adopter-smoke=not_ready。
5. 在本仓库安装公开 binary 后，inspect、status、doctor、Agent doctor 与 close 后文档检查保持健康。
6. 本 Work Item 在发布前具备可见 human Outcome、archive、finalization、close 以及精确 branch/worktree 清理。

## 验证

```text
cargo test --locked --workspace
```

发布与公开验收属于发布后 evidence。发布失败必须保留为不可变失败历史，不能重新标记或复用。

## 边界

Runtime binary 共享，但本仓库的 Protocol、Work Item、evidence、knowledge 与 adapter 保持仓库级隔离。发布 Runtime 不会隐式 attach 或修改其他仓库。
