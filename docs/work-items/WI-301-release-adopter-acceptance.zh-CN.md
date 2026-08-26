---
author: AI Cockpit maintainers
title: "WI-301——v0.2.33 公开 Release adopter 验收"
workItemId: WI-301-release-adopter-acceptance
description: "在全新隔离 adopter 中验证不可变 v0.2.33 binary，并验证公开 N-1 升级。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
lastVerifiedBy: WI-301-release-adopter-acceptance
terminalArchive: .ai/work-items/archive/WI-301-release-adopter-acceptance.contract.json
terminalVerification: .ai/evidence/WI-301-release-adopter-acceptance.verification.json
terminalFinalization: .ai/decisions/WI-301-release-adopter-acceptance.finalize.json
terminalDecision: .ai/decisions/WI-301-release-adopter-acceptance.close.json
authority: canonical
---

# WI-301——v0.2.33 公开 Release adopter 验收

## 意图

证明公开发布且不可变的 v0.2.33 Release binary 可以从零治理新仓库，并且
由公开 v0.2.31 binary 创建的仓库升级时不会丢失历史证据。

## 范围

本验收只在 `aarch64-apple-darwin` 上使用下载的公开 Release 制品。记录归档与
可执行文件 SHA-256、repository identity、Runtime identity、attach/profile/Agent
doctor 输出、保持 `first-adopter-smoke` 为 `not_ready` 的 Contract skeleton、证据
复用、完整 Work Item 生命周期、N-1 升级的历史保持、隔离 manifest 与临时根清理。

验收 receipt 保存在：

- `.ai/evidence/external/v0.2.33/adopter-aarch64-apple-darwin/`
- `.ai/evidence/external/v0.2.33/upgrade-v0.2.31-to-v0.2.33/`

## 证据边界

`runtime.json` 将 tag `v0.2.33`、归档 digest
`sha256:c8019db3d8509d62418afed114b986689df7b0ef570ff7199a4b845c7d932ca4` 与解压
binary digest
`sha256:eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4` 绑定。
升级 receipt 将公开 N-1 tag `v0.2.31` 与 v0.2.33 绑定，并逐字节保持旧证据。
`acceptance.json` 报告 `releasePublished: true`、`adopterAcceptance: passed` 和
`cleanupState: passed`；发布后失败只作为失败证据保留，不能改写 Release truth。

HOME 与 XDG_CONFIG_HOME 是禁止写入的 root；TMPDIR 与 CARGO_HOME 是明确隔离的
Runtime 写入 root。cleanup receipt 证明每个经过校验的临时 `run_root`（包括失败安全
路径）都已删除。

该 harness 产生的是发布后证据。第二技术栈 adopter 仍由独立 Work Item 负责，本记录不作此声明。
