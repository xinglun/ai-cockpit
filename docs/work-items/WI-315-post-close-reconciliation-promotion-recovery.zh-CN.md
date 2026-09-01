---
author: AI Cockpit maintainers
title: "WI-315——post-close reconciliation promotion recovery"
workItemId: WI-315-post-close-reconciliation-promotion-recovery
description: "在不改写不可变 W314 历史的前提下修正 recovered predecessor 的 promotion 语义。"
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-315-post-close-reconciliation-promotion-recovery
---

# WI-315——post-close reconciliation promotion recovery

## 意图与边界

W314 是不可变的 hosted 失败交付。其文档质量门暴露了一个投影缺陷：当
predecessor 同时已有 confirmed close 时，仍忽略有效的 successor recovery。本 successor
从最新默认分支修正这一有界 gate 条件，不改写 W314 历史。

## 范围与验收

- 有效且绑定 repository 的 `successor` 或 `supersede` recovery 会使 predecessor 成为历史记录，
  与 predecessor close 投影无关。
- retry、格式错误、foreign 及非规范 recovery 继续走正常 promotion 校验；证据无效时保持
  fail closed。
- 回归覆盖“confirmed approved close + 有效 successor recovery”及无效 recovery 变体。
- 三语文档与 parity 在 verification 前登记 W315，并保留 W314 失败与 recovery 边界。

## 验证

运行文档专项回归、文档验收、`cargo fmt`、禁止 warning 的 clippy，以及 locked 单进程
workspace 测试。合并前，精确审阅分支必须通过 hosted CI。治理接口使用已安装 Runtime。

## 相关历史

- W314：不可变 predecessor，其 hosted quality gate 发现本缺陷。
- W315：只修正 promotion 投影的有界 successor。

[English](WI-315-post-close-reconciliation-promotion-recovery.md) ·
[日本語](WI-315-post-close-reconciliation-promotion-recovery.ja.md)
