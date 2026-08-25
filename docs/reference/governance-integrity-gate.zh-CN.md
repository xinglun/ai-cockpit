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

## Finalization 绑定 reviewed checkout

对于 feature branch 或 pull request checkout，finalization 回执只有在其 branch、pull
request 和 worktree 三个 head 都解析为实际 reviewed checkout 时才有效（普通 checkout
使用 `HEAD`，synthetic merge checkout 使用 reviewed feature parent）。即使回执内部字段
彼此一致，只要指向较旧 commit 也必须拒绝；这样可以防止后续代码提交默默继承较早的
finalization。

祖先回执只能通过同一 Work Item 的受限 append-only governance 更新跨越这个边界：canonical
或 digest-suffixed finalization 记录、repository-local close 决定，以及两个固定的
post-finalize evidence 记录。任何代码或无关记录、modified/deleted/renamed path，或后续
非治理漂移都必须 fail-closed。因此，回执 head 是对 reviewed source 的绑定，而不是把一个
数值复制进回执就算完成。

## 分离的 pull request 检出

托管 pull request 作业可能使用 detached merge checkout，既没有
`refs/remotes/origin/HEAD`，也没有事件中的 base branch 元数据。此时质量门只把不可变
Contract 的 `resourceContext.baseBranch` 作为窄范围的默认分支回退。只有当回执与 Contract
的 resource context 完全一致时才接受；repository、PR URL/number、provider、remote、分支、
worktree、base/head revision、Runtime、证据和 Contract digest 检查仍然全部必须通过。如果
外部事件或 remote 声明了不同的 base branch，回执会被拒绝。缺失或矛盾的 identity 仍然
fail-closed。

本门不选择 verification tier 或 assurance。风险/阶段/策略选择，以及逐文件参考源
一致性，属于独立验证边界，不能从本盘点中推断。
