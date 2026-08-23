---
author: AI Cockpit maintainers
title: 治理完整性门
description: "对当前 Work Item、证据、终态决定和文档绑定执行 fail-closed 的动态盘点。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-195-governance-recovery-gate
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

本门不选择 verification tier 或 assurance。风险/阶段/策略选择，以及逐文件参考源
一致性，属于独立验证边界，不能从本盘点中推断。
