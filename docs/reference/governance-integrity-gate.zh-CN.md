---
author: AI Cockpit maintainers
title: 治理完整性门
description: "对当前 Work Item、证据、终态决定和文档绑定执行 fail-closed 的动态盘点。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-198-governance-gate-default-branch-discovery
---

# 治理完整性门

`tests/ci/governance_integrity_gate.py` 从仓库记录动态发现 Work Item，不维护固定
ID 清单。它检查当前发布周期、证据身份、终态决定、Outcome 和三语 parity 绑定。
Finding 是确定性的，任何不完整都会 fail-closed。

## Recovery 不是完成

拥有合法 `.ai/decisions/<WI>.recovery.json` 的前置 Work Item 会被报告为
`lifecycleState: recovered`。该回执必须绑定前置 Work Item、非空 successor、
repository identity、原因和 evidence refs。Recovered predecessor 可以保留红色/阻断的
不可变 Outcome；门不会把它提升为绿色，也不会把它当作合并或发布批准。

缺失、malformed、foreign 或绑定不足的 recovery 回执仍然是错误。Successor 必须独立
通过自己的 Contract、证据、Outcome、parity 和终态决定检查。

## 分离的 pull request 检出

托管 pull request 作业可能使用 detached merge checkout，既没有
`refs/remotes/origin/HEAD`，也没有事件中的 base branch 元数据。此时质量门只把不可变
Contract 的 `resourceContext.baseBranch` 作为窄范围的默认分支回退。只有当回执与 Contract
的 resource context 完全一致时才接受；repository、PR URL/number、provider、remote、分支、
worktree、base/head revision、Runtime、证据和 Contract digest 检查仍然全部必须通过。如果
外部事件或 remote 声明了不同的 base branch，回执会被拒绝。缺失或矛盾的 identity 仍然
fail-closed。

## Finalization head 绑定

在 `feature_branch` 与 `pull_request` 阶段，pre-merge finalization receipt 只有在其
branch、pull request 和 worktree head 都解析到 reviewed checkout head 时才有效。后续
checkout 只允许 canonical finalization transition 或明确列出的同一 Work Item 治理记录
进行受限 append。pending parity registry 是唯一明确的 repository 级治理追加，用于在
三语 parity 行完成前保持已关闭 Work Item 可见；代码、测试、无关证据或其他 repository
变化都会要求重新生成 receipt，并以 fail-closed 处理。

本门不选择 verification tier 或 assurance。风险/阶段/策略选择，以及逐文件参考源
一致性，属于独立验证边界，不能从本盘点中推断。
