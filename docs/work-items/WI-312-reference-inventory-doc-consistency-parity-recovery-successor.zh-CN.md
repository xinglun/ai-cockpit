---
author: AI Cockpit maintainers
title: "WI-312——参考 inventory 文档 parity 恢复 successor"
workItemId: WI-312-reference-inventory-doc-consistency-parity-recovery-successor
description: "在 WI-311 不可变重试边界后，重新交付 manifest 派生的 inventory 计数与归档前三语 parity 登记。"
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-312-reference-inventory-doc-consistency-parity-recovery-successor
---

# WI-312——参考 inventory 文档 parity 恢复 successor

## 意图与边界

本不可变交付因 retained provider finalization 无法满足后续 cleanup gate 而保留为历史事实。
其 Contract、Summary、Outcome、Events、archive、verification、finalization 与 close
字节不会被改写。WI-314 是从同步后的默认分支重新交付有界修正和 reconciliation 边界的显式 successor。

## 范围与验收

三份比较页必须包含由 5,119 条 inventory 记录派生且完全一致的 marker：4,262 条
generated history、182 条 implemented-different、1 条 equivalent、3 条
not-applicable、2 条 reference-only、669 条 deferred，且没有 migrate gap。确定性
conformance test 必须拒绝过期、错误、缺失或三语不一致的 marker。三份 parity 页必须
在 verification evidence 之前登记本行，三份 Work Item 文档必须保持相同的有界范围并含
`lastVerifiedBy` 元数据。

验证使用已安装 Runtime 以及 repository 文档和 inventory gates。源工程是语义参考，
不是 wire 格式或 Runtime 依赖。
