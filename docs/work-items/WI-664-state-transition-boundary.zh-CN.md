---
author: AI Cockpit maintainers
title: "WI-664 — 生命周期状态转换边界"
workItemId: WI-664-state-transition-boundary
description: "明确合法生命周期转换，同时不混淆生命周期、证据、治理决定、授权和历史投影。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-664-state-transition-boundary
authority: canonical
---

# WI-664 — 生命周期状态转换边界

## 目标

让生命周期状态转换显式且失败闭合，同时保留生命周期、证据、治理决定、
人工授权和历史投影之间已有的类型边界。

## 变更前

`cockpit-core::WorkItemState` 只是可序列化的状态词汇，没有经过检查的转换
操作；原有直接测试也只有 enum 相等断言。因此调用方仍需在其他位置重新解释
哪些转换合法。`DecisionState`、`AuthorityState` 和 `EvidenceState` 已经独立
存在，但没有一个纯函数边界明确说明生命周期移动不会创建这些事实。

## 变更后

`WorkItemState::can_transition_to` 定义经过审查的前进和恢复边，
`transition_to` 返回请求的后继状态，或返回类型化的
`WorkItemTransitionError`。正常路径为：

`Created → PreflightReady → ImplementationActive → VerificationPending → FinishReady → Archived → Closed`。

暂停状态通过 `ImplementationActive` 恢复；阻塞或过期状态通过
`PreflightReady` 恢复，因此恢复不会跳过新的检查。终态没有出边。上述方法
纯粹执行领域判断，不读取文件、不调用 Git、不执行命令、不写入记录、不推断
授权，也不生成证据。

## 契约与兼容性

- 既有 `snake_case` serde 值（包括 `finish_ready` 和 `closed`）保持不变。
- 未改变 protocol、repository、verification、文件布局或历史记录格式。
- 这是 `cockpit-core` 的增量 API；观察、授权、证据和写入顺序仍由调用方负责。
- 历史记录按历史事实读取，不为适应内部转换词汇而重写。

## P3 物理执行审计

本 WI 不修改 P3。现有 verification 边界中的 `PhysicalExecutionKey` 不包含
WI 身份；随后由 `WorkItemEvidenceReceipt::bind` 和 `validate_for` 把物理结果
绑定到具体 WI。物理执行测试覆盖不同 WI 的收据、外部收据拒绝、key 不匹配、
篡改和外部执行结果。因此缓存命中或共享执行结果本身不会授予某个 WI 授权或
通过状态。

## 验证与剩余风险

聚焦的 `cockpit-core` 测试覆盖前进、跳跃、回退、恢复和终态转换，以及旧
serde 值和未知值拒绝。关闭前还必须通过 workspace 测试、Runtime 验证、治理
检查和 hosted PR 检查。转换表保持有界，不把所有证据或策略组合塞进一个巨大
enum；调用方仍必须在写入转换前验证当前观察事实和治理事实。

