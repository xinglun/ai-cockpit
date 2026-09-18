---
author: AI Cockpit maintainers
workItemId: WI-918-release-v0-2-100
title: Outcome 语言、HCI、四方向与 Issue #851 收敛后的最终 v0.2.100 发布
description: 仅在所有前置 Work Item 和 Issue 完成后发布已评审的主线。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-918-release-v0-2-100
---

[English](WI-918-release-v0-2-100.md) · [日本語](WI-918-release-v0-2-100.ja.md)

# WI-918 — Outcome 语言、HCI、四方向与 Issue #851 收敛后的最终 v0.2.100 发布

本发布路径位于 Outcome 语言、HCI、四方向收敛、Issue #851 以及
Rust/工具链工作完成之后，只发布已评审的 main，并独立验收下载制品。

对象仓库不在范围内。除非有直接证据，性能收益和宿主展示确认继续明确为未知。

## 验收

- 工作区版本、锁文件以及当前英文、简体中文、日文发布/参考投影统一标识 v0.2.100。
- 带注释的 v0.2.100 标签不可变并绑定已评审源码提交。
- 公开 Release 绑定 manifest、校验和、SBOM、归档、attestation、源码身份和 workflow handoff。
- 下载制品通过校验和、隔离 fresh-install、v0.2.99 升级和精确临时根目录清理，且不修改对象仓库。
- 最终 Outcome 跟随当前对话语言，性能收益和宿主展示确认保持未知。

## 验证计划

先执行格式、版本、文档、parity、治理和 workspace 廉价检查，再 dispatch 发布 workflow。
消费发布 handoff，执行下载制品 adopter 验收，保留不可变证据，并在精确 provider 与
分支/worktree 清理完成后关闭。
