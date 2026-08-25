---
author: AI Cockpit maintainers
title: "WI-291——CI Contract-aware quality gate"
workItemId: WI-291-ci-contract-aware-gates
description: "保留 hosted stale-parity 阻断后的不可变失败交付；WI-292 重新交付同一有界批次。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-291-ci-contract-aware-gates
authority: canonical
---

# WI-291——CI Contract-aware quality gate

## 目的

WI-291 已交付有界 Rust Contract-aware CI gate，但 hosted quality 发现 parity
在 verification 后才登记，因此拒绝文档投影。其 lifecycle bytes 保持不可变；
WI-292 是显式 successor，不改写本次尝试。

## 边界

- 原样保留 WI-291 archive、verification、阻断的 finalization 与 recovery 记录。
- 不把失败 PR 视为已合并或已发布。
- 仅由 WI-292 从最新远端默认分支重新交付同一实现。

## 对象工程一致性

本仓库与全新 adopter 使用同一份已安装 Runtime、显式 repository context、
fail-closed lifecycle 和可见的人类 Outcome。

## 验证

Hosted PR 结果作为失败交付证据保留；新的 verification 与 provider lifecycle
由 WI-292 负责。
