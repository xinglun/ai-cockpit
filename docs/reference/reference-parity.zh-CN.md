---
author: AI Cockpit maintainers
title: "参考源对齐"
description: "供维护者和审查者使用的、有证据的产品边界比较记录。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_parity
---

# 参考源对齐

本页是审计比较资料，不是 adopter 使用指令。它记录 Rust Runtime 与参考产品边界的
一致、部分实现、延期能力和外部责任。普通用户应从[当前读者路线](../current/README.zh-CN.md)开始；
字段级映射参见[Contract 与 Summary 字段](contract-fields.zh-CN.md)。

## 真实性状态

矩阵严格使用四种状态：

- **已实现**——所述边界已经实现，并有当前 evidence 覆盖。
- **部分实现**——核心边界存在，但参考源的表面或 assurance 更广。
- **延期**——有意不属于当前 Runtime 边界。
- **外部边界**——由 Agent host、provider、组织或外部系统负责。

## 对齐矩阵

| 参考关注点 | Rust Runtime 状态 | 证据与边界 |
| --- | --- | --- |
| 面向读者的入口和语言切换 | 已实现 | 根 README 与 route README 在 English、简体中文和日本語之间互链。 |
| 目的、问题、架构和功能概览 | 已实现 | 设计思想、架构和功能路线描述当前 Runtime 及其责任方。 |
| 共享 Runtime 与 request-scoped repository context | 已实现 | 显式 `--repo` 绑定和 repository isolation tests 保持 context 与 evidence 隔离。 |
| Repository attach 和最小 scaffold | 已实现 | `attach` 创建 repository-owned Protocol scaffold，不在项目内安装 Runtime 副本。 |
| 显式 Agent Discovery / Adapter 层 | 已实现 | Agent 安装是显式、可拥有、可回滚且 repository-local；生成的 guidance 继承 Contract-first/暂停/Summary/Outcome/closure 语义，Cursor 新安装使用 `.cursor/rules/ai-cockpit.mdc`，已管理的 legacy `.md` 保持可读。 |
| Work Item 生命周期和治理决定 | 部分实现 | 核心生命周期和 human decision record 已存在；参考源更广的 status、cost、recovery projection 尚未统一为一个 adopter 接口。 |
| Task Outcome 与 Human Benefit 报告 | 部分实现 | WI-136 增加 Rust-native 严格报告投影、追加事件流、archive 绑定和 close final report；完整 recovery/event 重建仍不在本边界。证据：`.ai/evidence/WI-136-task-outcome-report.verification.json`。 |
| Contract preflight 人工确认门 | 已实现 | 不完整的 scaffold Contract 返回带显式 `reviewState` 的 yellow，持久化 repository/Contract/snapshot 绑定，未经人工确认不能越过 checkpoint。 |
| Contract V2 结构化 intent 与严格 schema | 已实现 | WI-121 提供结构化 intent、typed sources/verification、严格的 unknown-field/duplicate-key fail-closed、`humanDecisionRequest` 以及 preflight/checkpoint gate。 |
| Contract 跨字段维度（intent/scope/evidence/decision）校验 | 已实现 | WI-122 校验高风险 scenario coverage、稳定 acceptance evidence、intent alignment 和完整 20 维度最终 receipt。可选 `fourPillarProjection` 仅用于展示，协议不含字面 `4D` 字段。 |
| Contract 并行边界与 slot | 已实现 | WI-123 提供 repository-local 边界校验、保守的重叠判断和独占 slot lease；未知或格式错误状态 fail-closed。 |
| 有界验证与 fail-closed evidence reuse | 已实现 | Runtime identity、snapshot/toolchain/environment binding、receipt 和 fail-closed validation 均有记录。 |
| MCP repository binding | 已实现 | repository-bound stdio MCP 以显式绑定提供相同的治理服务。 |
| 面向人的 MCP projection | 已实现 | Runtime 校验 OutcomeV2 并生成本地化 `humanHandoff`；Agent 或对话层负责选择、展示和传递，但不能把 presentation 当作治理授权来源。 |
| 公开 Release 与新 adopter 验收 | 部分实现 | v0.2.15 完整 post-release adopter baseline 只在 `x86_64-unknown-linux-gnu` 执行；其他 target 只有 build/smoke evidence。 |
| 第二技术栈 adopter 验收 | 延期 | 当前 harness 使用 Cargo adopter；第二技术栈属于后续工作。 |
| Runtime-only upgrade 与 repository migration | 已实现 | compatibility 检查和显式 migration 保留历史记录并绑定 Runtime identity。 |
| N-1 旧 adopter 升级验收 | 已实现 | public-artifact harness 覆盖旧 schema 检测、批准、历史保持和继续运行。 |
| Adopter capability manifest 与 status projection | 延期 | 当前 `capability show` 和 `status` 是真实的 Runtime/repository 视图，不等同于参考源完整的 adopter manifest/status projection。 |
| Recovery state machine 与丰富 recovery projection | 部分实现 | 已有 blocked Outcome、append-only recovery receipt、绑定 predecessor 的 retry/successor decision 及人类/MCP 投影；paused/stale/cancelled/rollback 等更广表面仍窄于参考源。 |
| 多语言语义 parity gate | 部分实现 | CLI 面向人的输出已本地化；所有报告逐字段语义一致尚未成为 CI gate。 |
| 历史 evidence 边界 | 已实现 | 历史 evidence 只作为历史输入，永远不能提升为新的 green verification。 |
| Contract 原文语言 | 已实现 | Contract 的 intent、scope、acceptance、authority 保持原文；翻译不重写 Contract bytes。 |
| 安装和 provider 配置 | 外部边界 | binary delivery 与 provider/global configuration 和 repository governance state 分离。 |

矩阵刻意区分已工作的核心和完整参考源表面 parity。某一行的 green 只证明该行边界，
不授予外部 identity、provider authorization、branch protection、production readiness 或组织批准。

## 当前实现基线

当前 `main` 分支包含以下 Contract 和治理边界。Work Item 文档说明面向使用者的范围；
repository evidence 路径是各边界的机器可读验证记录。

| Work Item | 当前 Runtime 状态 | 证据与文档 |
| --- | --- | --- |
| WI-121——Contract V2 | 已实现 | [Work Item](../work-items/WI-121-contract-v2.zh-CN.md)；`.ai/evidence/WI-121-contract-v2.verification.json` |
| WI-122——Scenario、Acceptance 与最终维度 | 已实现 | [Work Item](../work-items/WI-122-scenarios-acceptance-final-dimensions.zh-CN.md)；`.ai/evidence/WI-122-scenarios-acceptance-final-dimensions.verification.json` |
| WI-123——Contract 并行边界与 Slot | 已实现 | [Work Item](../work-items/WI-123-parallel-contract-boundary.zh-CN.md)；`.ai/evidence/WI-123-parallel-contract-boundary.verification.json` |
| WI-125——Contract V2 schema boundary | 已实现 | [Work Item](../work-items/WI-125-contract-schema.zh-CN.md)；`.ai/evidence/WI-125-contract-schema.verification.json` |
| WI-126——只读状态与面向人的交接 | 已实现 | [Work Item](../work-items/WI-126-status-outcome.zh-CN.md)；`.ai/evidence/WI-126-status-outcome.verification.json` |
| WI-128——发布 adopter 验收清理 | 已实现 | [Work Item](../work-items/WI-128-release-acceptance-cleanup.zh-CN.md)；`.ai/evidence/WI-128-release-acceptance-cleanup.verification.json` |
| WI-129——参考源对齐完整性 | 已实现 | [Work Item](../work-items/WI-129-parity-gate.zh-CN.md)；`.ai/evidence/WI-129-parity-gate.verification.json` |
| WI-130——已关闭 Work Item 状态投影 | 已实现 | [Work Item](../work-items/WI-130-status-closed-projection.zh-CN.md)；`.ai/evidence/WI-130-status-closed-projection.verification.json`；`.ai/decisions/WI-130-status-closed-projection.close.json` |
| WI-131——验证证据时间戳 fail-closed 校验 | 已实现 | [Work Item](../work-items/WI-131-evidence-timestamp.zh-CN.md)；`.ai/evidence/WI-131-evidence-timestamp.verification.json`；`.ai/decisions/WI-131-evidence-timestamp.close.json` |
| WI-132——Agent adapter 与 provider 表面一致性 | 已实现 | [Work Item](../work-items/WI-132-agent-adapter-parity.zh-CN.md)；`.ai/evidence/WI-132-agent-adapter-parity.verification.json`；`.ai/decisions/WI-132-agent-adapter-parity.close.json` |
| WI-133——文档事实一致性校正 | 已实现 | [Work Item](../work-items/WI-133-docs-truth.zh-CN.md)；`.ai/evidence/WI-133-docs-truth.verification.json`；`.ai/decisions/WI-133-docs-truth.close.json` |
| WI-135——Repository 绑定的 retention 与关闭证据 | 已实现 | [Work Item](../work-items/WI-135-repository-bound-evidence.zh-CN.md)；`.ai/evidence/WI-135-repository-bound-evidence.verification.json`；`.ai/decisions/WI-135-repository-bound-evidence.close.json` |
| WI-136——Task Outcome 与 Human Benefit report | 已实现 | [Work Item](../work-items/WI-136-task-outcome-report.zh-CN.md)；`.ai/evidence/WI-136-task-outcome-report.verification.json`；`.ai/decisions/WI-136-task-outcome-report.close.json` |
| WI-140——Verification 语义与 Artifact 归档完整性 | 已实现 | [Work Item](../work-items/WI-140-verification-semantics.zh-CN.md)；`.ai/evidence/WI-140-verification-semantics.verification.json`；`.ai/decisions/WI-140-verification-semantics.close.json` |
| WI-141——Policy 驱动的 Verification Planner | 已实现 | [Work Item](../work-items/WI-141-policy-planner.zh-CN.md)；`.ai/evidence/WI-141-policy-planner.verification.json`；`.ai/decisions/WI-141-policy-planner.close.json` |
| WI-142——受影响 Verification 与依赖置信度 | 已实现 | [Work Item](../work-items/WI-142-affected-verification.zh-CN.md)；`.ai/evidence/WI-142-affected-verification.verification.json`；`.ai/decisions/WI-142-affected-verification.close.json` |
| WI-143——Intent、Scenario 与 Stage 绑定 | 已实现 | [Work Item](../work-items/WI-143-intent-scenario-binding.zh-CN.md)；`.ai/evidence/WI-143-intent-scenario-binding.verification.json`；`.ai/decisions/WI-143-intent-scenario-binding.close.json` |
| WI-144——跨 Work Item 的物理执行复用 | 已实现 | [Work Item](../work-items/WI-144-cross-work-item-dedup.zh-CN.md)；`.ai/evidence/WI-144-cross-work-item-dedup.verification.json`；`.ai/decisions/WI-144-cross-work-item-dedup.close.json` |
| WI-145——CI Runtime Verification Shadow | 已实现 | [Work Item](../work-items/WI-145-ci-runtime-shadow.zh-CN.md)；`.ai/evidence/WI-145-ci-runtime-shadow.verification.json`；`.ai/decisions/WI-145-ci-runtime-shadow.close.json` |
| WI-146——Verification 成本观测 | 已实现 | [Work Item](../work-items/WI-146-verification-cost-observation.zh-CN.md)；[参考文档](verification-cost.zh-CN.md)；`.ai/evidence/WI-146-verification-cost-observation.verification.json`；`.ai/decisions/WI-146-verification-cost-observation.close.json` |

## 当前边界

一份已安装 Runtime 可以治理多个相互独立 attach 的 repository。每个 repository 独立拥有
Protocol、Work Item、evidence、knowledge 和 adapter record。后续变化必须保持显式 repository
绑定、evidence 隔离、人类拥有的决定，以及 Runtime 分发和 repository state 的分离。

Work Item 关闭后，必须在同一轮 release audit 中完成三语文档定稿：状态为
`implemented`，链接归档 verification/close evidence，并与 parity baseline 行保持一致。
这条 documentation-truth 规则不会改写历史 evidence。

## Scenario、Acceptance 与最终维度投影

Runtime 现在验证（但绝不替代人生成）三类可选治理投影。高风险 Contract
必须声明 `scenarioCoverage`，Summary 必须提供 `required`、`status`、
`evidence`；当 `status` 为 `not_applicable` 时还必须提供 `reason`。高风险
Work Item 中 required scenario 若仍为未验证状态，则 fail-closed。

形如 `A1: ...` 的编号 Acceptance 会启用稳定 ID 和 Summary 的
`acceptanceEvidence` 映射。没有编号的旧 Acceptance 仍可读取，Runtime 不会
擅自为它们分配 ID。`intentAlignment` 是可选投影：缺失时保持 `unknown`；
`resolved` 或 `unresolved` 必须分别提供明确证据或理由。

最终验收使用参考源的完整 20 个维度名称，决定只能是 `GO`、
`CONDITIONAL_GO` 或 `NO_GO`。`GO` 必须同时具备已验证的 `real_adopter` 和
`provider_evidence`；缺失、额外、格式错误或身份不匹配的维度都会 fail-closed。
可选的 `fourPillarProjection` 仅用于展示；协议中不会引入含义不明确的字面
`4D` 字段，Runtime 也不会合成证据或把本地投影冒充 provider/enterprise assurance。
