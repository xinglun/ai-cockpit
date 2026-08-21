---
author: AI Cockpit maintainers
title: "Protocol 兼容规则"
description: "当前 runtime 对 Repository Protocol v1 的兼容行为。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - protocol_compatibility
---

# Protocol 兼容规则

## Request envelope 兼容性

Core 仅在 envelope 的 `schemaVersion` 为 `2` 时接受
`RequestedOperationV2` 和 `CapabilityMappingV2`。这个 request-envelope 版本是
adapter/Core contract，不是 repository schema 版本。未知的未来 envelope 版本会
fail closed；不会降级为 raw request，也不会被静默当成已授权。

以下是当前 runtime 实现的兼容规则：

1. 在不执行 repository material 的情况下解析 protocol version。
2. 在操作读取或写入治理状态前，拒绝 malformed 或不支持的 protocol version。
3. 仅在消费该记录的具体操作验证所需字段后接受 protocol major version `1`。
4. 不静默升级可选 capability，也不把不支持的请求转换为 pass；返回明确 error、unknown 或停止状态。
5. 兼容性检查不能重写历史 artifact。

当前 runtime 支持 protocol major `1`，没有宣称更宽的 minor/patch range。只要保持 v1 存储 contract，
runtime 的 minor/patch release 就不是 migration。Protocol major migration 必须创建新 Work Item，
保留旧 evidence，并记录 source/target protocol version。
