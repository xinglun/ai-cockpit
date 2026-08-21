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

Runtime version 与 Repository Protocol version 独立。

```text
ai-cockpit --version
0.1.1

repository:
protocol_version = 1
```

CLI version 标识 executable package；protocol version 标识 repository storage contract。Runtime
version、runtime digest 和 protocol version 会在 `inspect`、`doctor`、MCP `initialize`、verification
evidence 等 identity-bearing surface 一起提供；`--version` 只是简短的 package-version 命令，
不承诺完整 identity envelope。

Runtime 升级可以增加能力，同时继续支持 Protocol 1。只有 Protocol 1 → Protocol 2 才属于
repository migration。历史 Work Item 保留决策边界使用的 Project Profile digest 和 protocol
version。Major migration 必须是单独审查的 Work Item，并保留旧 evidence。
