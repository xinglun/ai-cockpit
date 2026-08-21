---
author: AI Cockpit maintainers
title: "配置参考"
description: "Repository 持有的 TOML 配置、profile 状态和生成的 Work Item 文件。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - configuration
---

# 配置参考

Repository 配置格式是 TOML，不改为 JSON。

## `.ai/cockpit.toml`

`attach` 会创建最小文件：

```toml
protocol_version = 1
repository_id = "sha256:<64 位小写十六进制>"
```

Runtime 会验证两个字段并拒绝 identity 不一致。不要把 runtime source 或 V1 文件复制到 `.ai/`。

## `.ai/project.json`

`attach` 创建 `state: "calibration_required"` 的 attached profile。`profile confirm` 后 profile
version 增加，并把选定质量命令记录为 verified。wrapper 包含 `profileVersion`、`repositoryId`、
`state`、`profileDigest`、`tests` 和 `buildSystems`；未知 profile 字段会被拒绝。

## Work Item 记录

`start` 在 `.ai/work-items/active/` 下生成：

- `<id>.contract.json`——intent、scope、authority、acceptance、required evidence、base revision、
  profile digest 和 repository snapshot digest；
- `<id>.summary.json`——生命周期状态和 checkpoint 数量。

`verify --work-item <id>` 写入 `.ai/evidence/<id>.verification.json`。`finish` 创建 outcome，`archive`
创建 archive manifest，`close` 记录 human decision。这些记录与内容绑定，不应手工修改来伪造 green。

跨进程 reusable evidence 由 runtime 管理于 `.ai/evidence/reuse/`；schema、identity binding 和资源
限制见 [Protocol v1](../protocol/v1/specification.zh-CN.md)。
