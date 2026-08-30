---
author: AI Cockpit maintainers
title: "版本策略"
description: "Runtime 和 Repository Protocol 的版本 identity 与迁移边界。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - versioning
---

# 版本策略

## 相邻迁移链

Repository schema migration 是由审核过的相邻边组成的显式链。Runtime 根据当前
schema 解析下一条边，并拒绝未知来源、未来 schema，或跳过未经审核的中间版本的
直接迁移。每个批准的步骤都会写入绑定 Runtime 的 receipt，其中包含步骤 identity、
链长度、保留历史 evidence 的 digest 以及 Runtime version/digest。历史 evidence、
decision、knowledge 和归档 Work Item 按字节保留；migration 永远不会重写它们。

Runtime version、Repository Protocol version 和 repository schema version 是独立的 identity。

```text
ai-cockpit --version
0.2.45

repository:
protocol_version = 1
repository_schema_version = 2
```

CLI version 标识 executable package；protocol version 标识 repository storage contract。Runtime
version、runtime digest 和 protocol version 会在 `inspect`、`doctor`、MCP `initialize`、verification
evidence 等 identity-bearing surface 一起提供；`--version` 只是简短的 package-version 命令，
不承诺完整 identity envelope。

Runtime-only 升级在 compatibility 为 `COMPATIBLE` 时保持 repository 的 `.ai/` bytes 不变。
Runtime identity 会记录在新的 verification 和 migration receipt 中，但 Runtime 不持有全局
active repository 或 Work Item 状态。

当前 Repository Protocol 仍为 Protocol 1，attached repository 的目标 schema 是 2。旧 schema
不会被静默改写，先检查边界：

```bash
ai-cockpit compatibility --repo /path/to/repository
ai-cockpit migrate plan --repo /path/to/repository
ai-cockpit migrate apply --repo /path/to/repository --approved
```

`COMPATIBLE` 允许正常 lifecycle 命令；`MIGRATION_REQUIRED` 只允许 inspect 和只读 plan，
在人工审查并批准显式 migration 前，lifecycle、Agent、MCP 和 verification 都会停止；
`INCOMPATIBLE` 是 fail-closed stop，需要安装理解该 schema 的 Runtime。Migration receipt 绑定
from/to schema、迁移前后 digest、runtime version 和 runtime digest。Work Item、evidence、decision、
knowledge 和 archive history 不会被该迁移重写。

历史 Work Item 保留决策边界使用的 Project Profile digest 和 protocol version。Major migration
必须是单独审查的 Work Item，并保留旧 evidence。
