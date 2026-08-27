---
author: AI Cockpit maintainers
title: "WI-314——finalization reconciliation redelivery"
workItemId: WI-314-finalization-reconciliation-redelivery
description: "在不可变 hosted quality 失败后重新交付 cleanup-before-close 与 append-only finalization reconciliation。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-314-finalization-reconciliation-redelivery
---

# WI-314——finalization reconciliation redelivery

## 意图与边界

WI-312 作为不可变历史交付保留。其 retained provider finalization 与条件 parity
投影在合并后暴露了顺序缺口。第一次 Runtime 修正由 WI-313 交付，但 PR #277 被
hosted 文档质量门在合并前正确拒绝。本 successor 从同步后的默认分支重新交付有界
修正，并记录明确的 W312 recovery，不改写任何 predecessor。

## 范围与验收

- 新 Work Item 在 provider finalization 为 retained、blocked 或 unknown 时不得 close；
  只有绑定身份的 deleted 结果才能满足 close。
- 历史 closed 记录只能在 predecessor、repository、Runtime、sequence 与精确 cleanup
  后置条件全部匹配时追加一次 append-only deleted transition。
- W312 显示为 `Recovered`，其原始 Contract、evidence、archive、finalization 与 close
  字节保持不变。没有有效 recovery 或 reconciliation 绑定的条件终态 parity 行继续失败。
- 英语、简体中文、日语 parity/work-item 投影在 verification 前同步，并保留精确 evidence 链接。

## 验证

先运行 finalization 与文档专项回归，再运行 `cargo fmt`、禁止 warning 的 clippy 和
locked 全 workspace 测试。合并前，精确审阅分支必须通过 hosted CI。治理接口使用已安装
Runtime；源码构建不作为发布验收替代。

## 相关历史

- W312：由本 successor recovery 的不可变合并交付。
- W313 / PR #277：不可变 hosted 失败交付；其分支与 archive 保留为外部审计历史，不重新激活。

[English](WI-314-finalization-reconciliation-redelivery.md) ·
[日本語](WI-314-finalization-reconciliation-redelivery.ja.md)
