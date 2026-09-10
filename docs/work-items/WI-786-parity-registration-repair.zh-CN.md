---
author: AI Cockpit maintainers
title: "WI-786——恢复态 parity 注册修复"
description: "修复治理检查要求的、绑定证据的三语言 parity 投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-successor-recovery
workItemId: WI-786-parity-registration-repair
lastVerifiedBy: WI-786-parity-registration-repair
---

[English](WI-786-parity-registration-repair.md) · [日本語](WI-786-parity-registration-repair.ja.md)

# WI-786——恢复态 parity 注册修复

## 意图与边界

WI-786 是 WI-785 的明确 successor。Hosted 治理检查发现，恢复态 parity
行必须包含实际的带摘要 recovery decision 路径，WI-784 行还必须绑定其
verification evidence。

本 Work Item 只修改三份 reference-parity 文档及自身三语言规划页。WI-785、
WI-784 及更早的 archive、evidence、Outcome、event 和 decision 字节保持
不可变。Runtime 源码、产品行为、发布状态及无关文档不在范围内。

## 验收映射

- WI-781、WI-782 和 WI-784 的英文、简体中文、日文行绑定实际 recovery
  decision 与所需 evidence 路径。
- WI-783 终态事实和 WI-785 恢复事实在三种语言中保持语义等价。
- WI-786 规划页保持 close 前状态，只能在 verified close 后从 Runtime 终态
  evidence 生成投影。

## 验证

`bash tests/docs/parity_status_check.sh`

`bash tests/docs/documentation_acceptance.sh`

`cargo test --locked --workspace`

前序绑定为
`.ai/decisions/WI-785-wi784-doc-promotion.recovery.json`。
