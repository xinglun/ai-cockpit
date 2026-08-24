---
author: AI Cockpit maintainers
title: "WI-247——WI-246 close parity 登记"
workItemId: WI-247-parity-close-registration
description: "保留 WI-247 不可变 archive，并恢复其 archive 后 parity 顺序缺陷。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-249-parity-finalization-registration
authority: canonical
---

# WI-247——WI-246 close parity 登记

WI-247 已验证并归档用于投影权威 WI-246 close chain 的文档变更。archive 后，它自己的
parity 行又从 active Contract 投影改成 archive/evidence/finalization 路径。该文档变更不属于
已归档 verification snapshot，因此 PR #198 作为未合并、不可变 predecessor 保留，而不是
绿色交付。

## 恢复边界

Runtime receipt `.ai/decisions/WI-247-parity-close-registration.recovery.json`
绑定准确 Contract、Summary、Outcome、Events、archive manifest、verification evidence、
repository identity 与 Runtime v0.2.31 digest。WI-249 从 recovery bootstrap `f59ff36`
原样导入这些 bytes；不重放 WI-247 实现，也不制造 finalization receipt。

## 根治

WI-249 将 WI-247 保留为“已恢复”，完成 WI-246 终态 ledger 投影，并加入条件式 archive 前
控制。只有 Contract、Summary 或 acceptance 明确拥有 parity ledger 的 Work Item 才必须在
verification 前发布三条 lifecycle-bound 行；普通代码 Work Item 不承担该文档义务。
