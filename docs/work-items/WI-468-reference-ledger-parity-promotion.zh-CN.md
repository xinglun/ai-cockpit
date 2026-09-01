---
author: AI Cockpit maintainers
title: "WI-468——reference ledger parity promotion"
description: "在不可变的 WI-467 基础上补齐三语 parity 台账登记，并重新交付 manifest 派生的当前快照。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-468-reference-ledger-parity-promotion
predecessorWorkItemId: WI-467-reference-ledger-projection
status: in_progress
authority: authorized
lastVerifiedBy: WI-468-reference-ledger-parity-promotion
---

# WI-468——reference ledger parity promotion

## 意图与边界

WI-468 是不可变 WI-467 的明确 successor。前置 WI 的 Contract、evidence、
Outcome、archive 和 recovery receipt 保持不变。本 Work Item 重新交付同一
有界的 manifest 派生当前快照，并在中、英、日三份 reference-parity 台账中
登记，使仓库治理门能够在合并前验证文档事实。

## 范围与验收

- 三份 comparison 页面继续从 canonical inventory manifest 派生。
- 三份 reference-parity 页面各增加一条一致的 WI-468 记录。
- 历史 sections 与前置 WI bytes 保持不可变。
- 当前统计或 parity 登记分叉时，文档门必须 fail closed。

参考源 checkout 只是本地语义参考，不是 Runtime 或 wire format 依赖。生成的
archive、evidence 和 decision 记录由 Runtime 管理，不手工编辑。

## 验证

使用安装版 Runtime，并在每个命令中显式提供 repository 路径；执行 Contract
声明的文档、conformance 和 workspace gates。

## 链接

[English](WI-468-reference-ledger-parity-promotion.md) ·
[日本語](WI-468-reference-ledger-parity-promotion.ja.md)
