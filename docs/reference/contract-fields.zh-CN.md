---
author: AI Cockpit maintainers
title: "Contract 与 Summary 字段"
description: "Work Item Contract 和 Summary 的 Rust Runtime 字段映射。"
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - contract_field_mapping
---

# Contract 与 Summary 字段

本文说明当前 Rust Runtime 如何映射参考源的 Contract 和 Summary 概念。它是字段映射，
不是第二套 schema，也不表示参考源的每个字段都已实现。Runtime 将 repository protocol
状态保存在 `.ai/`，可执行文件只在被治理 repository 外安装一份。

状态含义：

- **已实现（Implemented）**——当前 Runtime 读取、写入或校验该边界，并有明确的 repository-local 语义。
- **部分实现（Partial）**——可以读取或表示，但不保证参考源的全部语义。
- **外部边界（External）**——事实属于 Agent 宿主、provider、组织或其他系统；Runtime 只能绑定或展示 evidence，不能自行生成。

## Work Item Contract（`*.contract.json`）

| 字段 | Rust Runtime 映射 | 状态 |
| --- | --- | --- |
| `protocolVersion` | Repository Protocol 版本，当前为 `1`。 | Implemented |
| `contractVersion` | 可选的 typed Contract V2 声明；历史 protocol record 仍可读取。 | Implemented |
| `repositoryId` | 从 attach 的 repository 推导的 identity，用于隔离。 | Implemented |
| `workItemId`、`mode`、`state`、`createdAt` | Work Item identity 和生命周期元数据。 | Implemented |
| `intent`、`goal` | 人类拥有的目的；`intent` 支持旧文本或结构化 `businessGoal`、`userGoal`、`problem`、`constraints`、`nonGoals`、`rationale`。 | Implemented |
| `scope`、`outOfScope` | repository-relative 实施边界；不安全或含糊路径 fail-closed。 | Implemented |
| `risk`、`authority` | preflight 使用的风险与权限声明；repository record 不认证个人身份。 | Implemented / External identity boundary |
| `acceptanceCriteria` | 人类拥有的验收声明；编号 `A1:` 可绑定 Summary evidence。 | Implemented |
| `requiredEvidenceClasses` | 生命周期完成所需的 evidence 类别。 | Implemented |
| `sources` | 旧字符串或 typed `{path, reason}` 引用。 | Implemented |
| `verification` | 旧验证字符串或 typed `{check, required}` 声明；声明不能替代新鲜执行。 | Implemented |
| `baseRevision` | Work Item 起始 revision，由 snapshot 推导。 | Implemented |
| `projectProfileDigest`、`repositorySnapshotDigest` | 绑定 project profile 和 repository snapshot 的内容 digest。 | Implemented |
| `baseCommit`、`baselineDirtyPaths` | V2 lineage 与开始时观察到的既有脏路径 fingerprint；两者同时存在时 `baseCommit` 必须与 `baseRevision` 一致。 | Implemented |
| `archiveSequence`、`resumeHistory` | 正数 archive 顺序和连续、已关闭前序 Work Item 的 lineage。 | Implemented |
| `synchronizationCheckpoint`、`synchronizationHistory` | 明确授权的基线同步及其 digest-bound rebase 历史；不完整条目 fail-closed。 | Implemented |
| `guidelines`、`preReviewWarnings`、`acceptance` | 人类编写的指导、review 警告和可选别名；`acceptance` 必须与 `acceptanceCriteria` 一致。 | Implemented |
| `authorityEvidence`、`restrictedWriteApproval`、`destructiveChangePolicy.approvalEvidence` | typed repository-local provenance 和审批 payload；V2 拒绝 malformed/unknown nested 字段，历史 provider 扩展仍可读。 | Implemented / External identity boundary |
| `problemStatement`、`riskAssessment`、`agentCapability`、`executionDecision` | 严格 typed 的可选 V2 安全和 review 输入；非 continue 决定会停止 preflight。 | Implemented |
| `destructiveChangePolicy`、`rollbackNote`、`unknowns`、`notCodable` | 显式安全、恢复和未决状态声明。 | Implemented |
| `scenarioCoverage` | 可选高风险 scenario projection；required/unverified scenario 会在 checkpoint 前 fail-closed。 | Implemented |
| `concurrencyBoundary` | 并行 Work Item 的 Contract-owned 路径边界和 slot 授权。 | Implemented |
| `checkpointPolicy`、`humanDecisionPoints`、`documentationImpact`、`performanceImpact`、`governanceProfile` 等扩展 | 只有在当前 typed validator 定义行为时才有行为保证；通用字段不是隐含批准。 | Partial |

`authority: authorized` 只是 repository-local 声明。企业身份、provider 验证、组织策略和审批真实性仍是外部 evidence，不能从 Contract bytes 推断。

## Change Summary（`*.summary.json`）

| 字段 | Rust Runtime 映射 | 状态 |
| --- | --- | --- |
| `workItemId`、`repositoryId`、`mode`、`state` | Contract/repository 绑定和串行生命周期状态。 | Implemented |
| `changedPaths` | 用于 scope 与 archive 检查的 snapshot 变更路径。 | Implemented |
| `checkpointCount` | 当前生命周期的 exactly-one checkpoint gate。 | Implemented |
| `preflightState`、`preflightAt`、`preflightContractDigest`、`preflightDecisionDigest`、`preflightRepositorySnapshotDigest` | repository-bound preflight 决定及新鲜度绑定。 | Implemented |
| `scenarioCoverage` | 与 Contract 对照校验的 Summary scenario 状态、evidence 和 reason。 | Implemented |
| `acceptanceEvidence` | 稳定 acceptance ID、显式 evidence 和 intent alignment 的映射。 | Implemented |
| `intentAlignment` | 可选 resolved/unresolved projection；缺失时保持 unknown。 | Implemented |
| `finalDimensions` | 完整 20 维度 receipt，决定为 `GO`、`CONDITIONAL_GO` 或 `NO_GO`；`fourPillarProjection` 仅用于展示。 | Implemented |
| `verification` | Runtime 执行 receipt 写入 `.ai/evidence/`，不会仅因文件存在就满足。 | Implemented |
| `outcome`、archive manifest、human decision | 由 Runtime 在 `.ai/work-items/archive/` 与 `.ai/decisions/` 生成的终端 projection。 | Implemented |
| `reviewReadiness`、`residualRisks`、`knownGaps`、`followUps`、`documentationAlignment` | 有参考价值，但当前 Runtime 没有统一 typed Summary contract。 | Partial |
| provider、enterprise、hosted-CI、attestation、SBOM 和组织审批声明 | 可作为 delegated evidence 导入或关联；Runtime 不生成 provider authority。 | External |

## 边界

Runtime 保留 Contract 原文语言，不机器翻译治理事实。Outcome 的本地化只改变标签和展示。
缺失、过期、矛盾、格式错误或 identity 不匹配的字段，按适用 gate 保持 yellow 或 red，不能通过文档 projection 变成 green。

参见[参考源对齐](reference-parity.zh-CN.md)和[命令参考](commands.zh-CN.md)。

## Contract 审查边界

当前 Rust 边界会在治理评估前校验可选的 `scenarioCoverage` 列表结构。
每个条目必须声明 `scenario`、布尔值 `required`、受支持的状态和 evidence
列表；`verified` 条目必须有 evidence，`not_applicable` 条目必须有 reason，
重复名称或未知嵌套字段会 fail closed。这只是结构校验：是否要求场景由风险
策略决定，Runtime 不会替人生成场景、预期结果或 verification plan。

`acceptanceCriteria` 必须是非空的人类声明。带编号的 `A<n>:` 条目仍是
Summary evidence 映射的显式选择；未编号条目仍作为可读的 legacy/源语言声明
保留。`concurrencyBoundary` 同样会校验 schema、正容量和非空理由，然后才能
使用并行 slot。这些检查不会把 verification tier 变成 assurance，也不会把
slot 声明变成授权决定。
