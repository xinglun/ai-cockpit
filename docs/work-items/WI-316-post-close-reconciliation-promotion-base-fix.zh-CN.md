---
author: AI Cockpit maintainers
title: "WI-316——post-close reconciliation promotion base fix"
workItemId: WI-316-post-close-reconciliation-promotion-base-fix
description: "在不改写 W315 历史的前提下，将 recovered promotion 修正绑定到最新远端默认基线。"
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-316-post-close-reconciliation-promotion-base-fix
---

# WI-316——post-close reconciliation promotion base fix

## 意图与边界

W315 是不可变的归档交付。Hosted CI 在执行前拒绝了它，因为 Contract 的
`baseRevision` 仍指向旧分支头。本有界 successor 从最新 `origin/main` 开始，绑定实际
CI 基线，并在不改写历史的情况下重新交付已审阅的 W314/W315 修正。

## 范围与验收

- Contract 记录 hosted CI 使用的最新远端默认 revision。
- 有效 successor/supersede recovery 继续作为历史 promotion 例外；retry、格式错误和
  foreign recovery 继续 fail-closed。
- W315 archive 与所有 predecessor evidence 保持逐字节不可变。
- 英语、简体中文和日语 Work Item/parity 投影在 verification 前同步。

## 验证

运行 promotion/文档回归、`cargo fmt`、禁止 warning 的 clippy、locked workspace 测试，以及
精确审阅分支的 hosted CI。治理接口使用已安装 Runtime。

## 相关历史

- W315：被 hosted base-revision gate 拒绝的不可变交付。
- W316：修正远端基线绑定的有界 successor。

[English](WI-316-post-close-reconciliation-promotion-base-fix.md) ·
[日本語](WI-316-post-close-reconciliation-promotion-base-fix.ja.md)
