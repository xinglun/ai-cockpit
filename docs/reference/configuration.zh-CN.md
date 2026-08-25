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

## `.ai/project/` 声明

采用者可以添加三个严格的、由 repository 所有的 JSON 声明：

- `capabilities.json`：capability、non-capability、critical domain，以及精确的 operation-to-capability mapping；
- `success_criteria.json`：只用于展示项目 criteria 与 evidence hint；Contract acceptance 仍然具有权威；
- `profile-policy.json`：批准的边界、critical path、review requirement 与显式 unknown。这是 reference profile policy 的
  JSON projection，`.ai/project.json` 仍然负责 identity 与 observed-quality profile。

每个文件只接受 regular file，拒绝未知字段和重复 JSON 键，并绑定 repository ID 与审查时的 snapshot digest。
`capability show` 和 MCP `capability_show` 只报告语义 declaration digest，不写入它们。Contract 显式声明
`operation`/`requestedOperation` 时，Preflight 必须找到足够的 mapping；输入缺失、格式错误、foreign、过期或冲突时
保持 yellow/unknown。intent prose 不能满足 mapping，project criterion 也不能批准或完成 Work Item。

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
      "requiredEvidence": ["delegated:github"]
    }]
  }
}
```

project 层可以增加要求，但不能弱化 organization 层。Work Item contract 可以
携带 `layer: "work_item"` 的 `governancePolicy`；所有策略对象都会拒绝未知字段。
`attach` 不会生成此文件，因为策略是治理决定，不是脚手架。

外部证明通过 `evidence import` 单独导入；metadata JSON 必须是严格的
`DelegatedEvidence` 对象，raw 文件按 bytes 计算 digest。使用 `evidence list`（或 MCP
`delegated_evidence_list`）查看已绑定的 receipt。

## Work Item 记录

`start` 在 `.ai/work-items/active/` 下生成：

- `<id>.contract.json`——intent、scope、authority、acceptance、required evidence、base revision、
  profile digest 和 repository snapshot digest；
- `<id>.summary.json`——生命周期状态、恰好一次的 checkpoint 记录，以及绑定 repository/Contract 的 preflight 决策
  （`preflightState`、decision digest、snapshot digest 和时间）。

生命周期采用 fail-closed 串行规则：Work Item 必须先记录非 red 的 preflight，再执行唯一一次 checkpoint；verification
完成后会刷新该决策，`finish` 要求结果为 green。重复 checkpoint 或乱序执行 finish/archive/close 都会被拒绝。失败的转换
会保留 active 记录，以便修复后恢复。

选择 typed checkpoint 控制的 Contract 可以使用动态 verification profile：低风险或只读工作使用
`light`，普通代码变更使用 `standard`，治理/CI 敏感变更使用 `strict`，不可变 artifact 和外部
evidence 边界使用 `release`。required checks 与 stages 仍必须由 policy 显式声明，Runtime 不会根据
标签推断或静默升级它们。profile 不代表 Evidence Assurance；T3 以及 provider/enterprise assurance
仍是独立的 policy requirement。

`work-item new --repo <path> --id <id> --mode <mode>` 复用同一 contract writer，生成 `not_ready` 骨架。它只填充四个可确定
推导事实（`repositoryId`、`baseRevision`、`projectProfileDigest`、`repositorySnapshotDigest`），intent、scope、acceptance criteria
和 authority 保持空值或 `unknown`。`profile propose` 只输出候选 amendment，不改变正式 profile 的 bytes 或 digest。

## Contract V2 语义边界

Contract 可以在保持 `protocolVersion` 不变的情况下选择性声明 `contractVersion: 2`。
V2 的 `intent` 可以是结构化对象（`businessGoal`、`userGoal`、`problem`、`constraints`、
`nonGoals`、`rationale`），也可以继续读取历史的一行字符串。`sources` 和 `verification`
支持带有 `path/reason`、`check/required` 的结构化形式；旧字符串形式只用于兼容历史 bytes。

Contract 顶层和结构化字段均采用严格未知字段校验，重复 JSON 键、错误类型和不兼容 schema
必须 fail closed。未知项、`notCodable`、Agent capability 或 execution decision 表明需要
人工判断时，preflight 会返回 `reviewState: needs_human_confirmation` 和结构化
`humanDecisionRequest`；该请求不是批准，必须完成 Contract 并重新 preflight 后才能 checkpoint。

场景覆盖、最终验收维度和并行边界是后续 Contract 扩展，不能把 `verify --workers` 当作
Work Item 并行授权。Contract 原文仍保持 owner 的语言，不由 Runtime 自动翻译。

### Contract V2 的 lineage 与治理字段

以下可选字段是 typed Contract 数据。protocol-v1 记录会保持省略或空值，
Contract V2 记录可以使用它们：

- `baseCommit` 和 `baselineDirtyPaths` 绑定 Work Item 的起始 revision，并记录开始前已存在的 dirty 文件（`path`、`status`、`fingerprint`）。
  旧字段名 `baseRevision` 继续保留。
- `archiveSequence` 只是正整数顺序 metadata，不能替代 archive manifest 自身的 digest 绑定。
- `resumeHistory` 记录连续且已关闭的 predecessor transition；每项包含 old/new base、分支 identity、Contract digest、manifest path 和 predecessor closure flags。
- `synchronizationCheckpoint` 必须明确写入 `authorized: true` 和非空 reason。`synchronizationHistory` 记录 base/rebase transition，不能用来隐藏无关 dirty path。
- `guidelines`、`preReviewWarnings` 和可选 `acceptance` 保留人工编写的指示与稳定的验收声明；空 guideline 会被拒绝。
- `authorityEvidence` 与 `restrictedWriteApproval` 是 repository-local provenance record，不是身份认证。Destructive approval evidence 必须 typed 地包含 identity level、actor、scope 和 evidence payload；provider/enterprise 声明仍需外部验证。

对于 `contractVersion: 2`，`mode` 必须是 `investigate`、`author_todo`、`code`、`review` 或 `cleanup` 之一；`code` Contract 的 `unknowns` 必须为空且 `notCodable: false`。
未选择 Contract V2 的历史记录仍可读取 legacy `mode: implementation`。
非法 lineage、approval、mode 或 cross-field 组合会在 Contract validation 阶段停止。历史 Contract bytes 不会被回填或重写。

`verify --work-item <id>` 写入 `.ai/evidence/<id>.verification.json`。`finish` 创建 outcome，`archive`
创建 archive manifest，`close` 记录 human decision。这些记录与内容绑定，不应手工修改来伪造 green。

跨进程 reusable evidence 由 runtime 管理于 `.ai/evidence/reuse/`；schema、identity binding 和资源
限制见 [Protocol v1](../protocol/v1/specification.zh-CN.md)。
