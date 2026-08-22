---
author: AI Cockpit maintainers
title: "运维"
description: "运行、验证、恢复、升级并验收 AI Cockpit repository。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - repository_operations
---

# 运维

- 按[功能与边界](../capabilities.zh-CN.md)执行受治理 Work Item 流程并处理停止条件。
- 在[参考](../reference/README.zh-CN.md)查找精确命令和恢复细节。
- 在[发布与分发](../release/distribution.zh-CN.md)查找不可变 Release 验证、升级、回滚和发布后 adopter 验收。
- 用[版本策略](../architecture/versioning.zh-CN.md)区分共享 Runtime 升级和显式 repository migration。
- 用[性能验收](../../tests/performance/README.zh-CN.md)与[对抗性验证](../security/adversarial-validation.zh-CN.md)查找测量或负向 evidence。

当前公开 adopter acceptance 基线只有 `x86_64-unknown-linux-gnu` 完整通过；发布一致性 gate
从 Cargo workspace metadata 推导基线版本。其他 Release target 只有 build 或 smoke evidence，除非另有独立验收记录。
历史 evidence 仍是历史资料，不能提升为新的 green verification。

通过 MCP 交付结果时，用 `work_item_outcome` 输出面向人的 handoff，用 `work_item_get` 做机器查询。发布
adopter receipt 必须包含带类型的隔离 manifest 和清理证明；允许的临时写入只限于明确隔离的 TMPDIR 与 CARGO_HOME。

[当前路线](../current/README.zh-CN.md) | [快速开始](../getting-started/README.zh-CN.md) |
[English](README.md) | [日本語](README.ja.md)
