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

`repository_id` 在第一次 attach 时生成，后续请求从这个 repository-owned 文件读取。它不会
对绝对路径做哈希，因此移动已 attach 的 repository 不会让 evidence 变成另一个 repository。
Runtime 会验证两个字段并拒绝 identity 不一致。不要把 runtime source 或 V1 文件复制到 `.ai/`。

## `.ai/agent-interface.json`

`attach` 还会写入严格的 repository-local discovery manifest，包含 `schemaVersion`、`protocolVersion`、稳定的
`repositoryId`、`rootBinding: "manifest-parent"`、当前 Runtime 能力和 `adapterState: "unconfigured"`。它是 discovery fact，
不是 provider instruction、授权或全局 MCP 配置。Provider 安装必须是独立的显式操作。

## `.ai/adapters/<provider>.json`

`agent install` 会写入严格的 ownership record，包括 provider、repository ID、repository-relative target、adapter version
以及 managed section 的 digest。`doctor`、`repair` 和 `detach` 以它作为 ownership 依据；记录缺失、内容被修改、marker 重复
或 identity 不一致都会形成 conflict。这里不会保存全局 Agent 或 MCP 配置。

## `.ai/project.json`

`attach` 创建 `state: "calibration_required"` 的 attached profile。`profile confirm` 后 profile
version 增加，并把选定质量命令记录为 verified。wrapper 包含 `profileVersion`、`repositoryId`、
`state`、`profileDigest`、`tests` 和 `buildSystems`；未知 profile 字段会被拒绝。

## `.ai/policy.json`

企业采用者可以选择严格的策略文件，而不改变 TOML 配置格式：

```json
{
  "schemaVersion": 1,
  "organization": {
    "policyId": "org-release-v1",
    "layer": "organization",
    "rules": [{
      "operation": "release",
      "approvalMode": "single_authorized_human",
      "requiredEvidence": ["hosted_ci"]
    }]
  }
}
```

project 层可以增加要求，但不能弱化 organization 层。Work Item contract 可以
携带 `layer: "work_item"` 的 `governancePolicy`；所有策略对象都会拒绝未知字段。
`attach` 不会生成此文件，因为策略是治理决定，不是脚手架。

## Work Item 记录

`start` 在 `.ai/work-items/active/` 下生成：

- `<id>.contract.json`——intent、scope、authority、acceptance、required evidence、base revision、
  profile digest 和 repository snapshot digest；
- `<id>.summary.json`——生命周期状态和 checkpoint 数量。

`work-item new --repo <path> --id <id> --mode <mode>` 复用同一 contract writer，生成 `not_ready` 骨架。它只填充四个可确定
推导事实（`repositoryId`、`baseRevision`、`projectProfileDigest`、`repositorySnapshotDigest`），intent、scope、acceptance criteria
和 authority 保持空值或 `unknown`。`profile propose` 只输出候选 amendment，不改变正式 profile 的 bytes 或 digest。

`verify --work-item <id>` 写入 `.ai/evidence/<id>.verification.json`。`finish` 创建 outcome，`archive`
创建 archive manifest，`close` 记录 human decision。这些记录与内容绑定，不应手工修改来伪造 green。

跨进程 reusable evidence 由 runtime 管理于 `.ai/evidence/reuse/`；schema、identity binding 和资源
限制见 [Protocol v1](../protocol/v1/specification.zh-CN.md)。
