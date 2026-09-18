---
author: AI Cockpit maintainers
workItemId: WI-899-release-v0-2-98
title: Outcome、HCI、四方向与 Issue #851 收敛后的最终 v0.2.98 发布
description: 仅在所有前置 Work Item 和 Issue 完成后发布已评审的主线。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: user:release-after-all-work-items
lastVerifiedBy: WI-899-release-v0-2-98
---

[English](WI-899-release-v0-2-98.md) · [日本語](WI-899-release-v0-2-98.ja.md)

# WI-899 — Outcome、HCI、四方向与 Issue #851 收敛后的最终 v0.2.98 发布

## 意图与边界

本发布 Work Item 位于 Outcome 交付、HCI 修正、四方向收敛、Issue #851 以及
Rust/工具链升级完成并合并之后。它只更新发布身份，并独立证明公开制品，
不把源码检出当作发布制品。

对象仓库明确不在范围内。本 Work Item 不新增 Outcome、宿主展示、性能、
生命周期或治理行为。

## 验收

- 工作区 package 版本、锁文件、当前发布文档和生成归档统一标识 v0.2.98。
- 带注释的 v0.2.98 标签不可变并绑定已评审源码提交；已有标签保持不变。
- Provider Release 稳定，并绑定 manifest、校验和、SBOM、归档、源码身份和
  workflow handoff。
- 下载的 v0.2.98 制品通过校验和、隔离 fresh-install 以及 v0.2.97 升级验收，
  临时根目录被精确清理。
- 英文、简体中文和日文发布/参考投影同步，最终 Outcome 如实报告事实、限制和
  下一步。

## 验证计划

先执行格式、版本和 Contract 前检，再执行昂贵的发布任务。随后使用已评审 PR
和仅 dispatch 的发布 workflow，保留任何失败候选证据，消费发布 handoff，执行
下载制品的 adopter 验收，最后绑定精确分支/worktree 清理和发布后证据后关闭。
