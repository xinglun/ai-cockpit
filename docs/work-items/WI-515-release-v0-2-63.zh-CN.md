---
author: AI Cockpit maintainers
title: "WI-515——发布 v0.2.63 与历史 adopter 收尾验收"
description: "发布 legacy shared-worktree 与 direct-merge recovery 修复，并提供不可变 adopter 证据。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
workItemId: WI-515-release-v0-2-63
lastVerifiedBy: WI-515-release-v0-2-63
terminalArchive: .ai/work-items/archive/WI-515-release-v0-2-63.contract.json
terminalVerification: .ai/evidence/WI-515-release-v0-2-63.verification.json
terminalFinalization: .ai/decisions/WI-515-release-v0-2-63.finalize.json
terminalDecision: .ai/decisions/WI-515-release-v0-2-63.close.json
---

[English](WI-515-release-v0-2-63.md) · [日本語](WI-515-release-v0-2-63.ja.md)

# WI-515——发布 v0.2.63 与历史 adopter 收尾验收

## 意图

发布 truthful historical shared-primary `retained` finalization 与无 PR
direct-merge recovery 修复。对象/adopter 工程保持只读，并从公开 artifact
自行验收。

## 范围

- 将 workspace 版本及当前三语发布/版本文档对齐到 v0.2.63，不改写历史发布事实。
- 在三语 parity 台账登记本发布 Work Item。
- 从同步后的 main 创建 annotated tag 前，先通过审查 PR 的 hosted checks。
- 发布 archive、SHA256SUMS、SBOM、provenance 与 manifest，并只用下载的不可变 artifact
  执行 public adopter 与 N-1 验收。
- 在本仓库安装公开 binary，验证 inspect、status、doctor、Agent doctor 和文档晋级健康状态。

## 范围外

本地参考源、对象/adopter 工程、全局 Agent/MCP 或 Homebrew 配置、源码/workspace
fallback、无关 Runtime 改动，以及手工编辑生成治理记录。

## 验收标准

1. workspace package 与 lockfile 标识 v0.2.63，同时保留历史发布事实。
2. 在同步 main 创建 annotated tag 前，审查 PR 的全部 hosted checks 通过。
3. 公开 release 的 archive、SHA256SUMS、SBOM、provenance、manifest 在 tag、bytes 与 digest 上一致。
4. public adopter 与 N-1 只使用下载的不可变 artifact，证明隔离和临时根清理，并保留
   `first-adopter-smoke=not_ready`。
5. 在本仓库安装公开 binary 后，健康检查与 close 后文档门禁保持绿色。
6. 发布完成前记录可见 human Outcome、archive、finalization、close 和精确清理。

## 验证

```text
cargo test --locked --workspace
```

发布与对象工程验收是独立 evidence 边界。发布失败保留为不可变历史，不能重新标记或复用。

## 边界

Runtime binary 共享；每个 repository 继续隔离拥有 Protocol、Contract、evidence、knowledge
和 adapter。发布 v0.2.63 不会隐式 attach 或修改其他 repository。
