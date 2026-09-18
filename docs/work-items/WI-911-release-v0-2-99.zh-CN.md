---
author: AI Cockpit maintainers
workItemId: WI-911-release-v0-2-99
title: Outcome 语言、HCI、四方向与 Issue #851 收敛后的最终 v0.2.99 发布
description: 仅在所有前置 Work Item 和 Issue 完成后发布已评审的主线。
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-911-release-v0-2-99
terminalArchive: .ai/work-items/archive/WI-911-release-v0-2-99.contract.json
terminalVerification: .ai/evidence/WI-911-release-v0-2-99.verification.json
terminalDecision: .ai/decisions/WI-911-release-v0-2-99.close.json
---

[English](WI-911-release-v0-2-99.md) · [日本語](WI-911-release-v0-2-99.ja.md)

# WI-911 — Outcome 语言、HCI、四方向与 Issue #851 收敛后的最终 v0.2.99 发布

## 意图与边界

本发布 Work Item 位于 Outcome 按对话语言输出、HCI 修正、四方向收敛、Issue
#851 以及 Rust/工具链升级完成并合并之后。它只更新发布身份，并独立证明公开制品，
不把源码检出当作发布制品。

对象仓库明确不在范围内。本 Work Item 不新增 Outcome、宿主展示、性能、生命周期或
治理行为。

## 验收

- 工作区 package 版本、锁文件、当前发布/参考文档和生成归档统一标识 v0.2.99。
- 带注释的 v0.2.99 标签不可变并绑定已评审源码提交；已有标签保持不变。
- Provider Release 稳定，并绑定 manifest、校验和、SBOM、归档、attestation、源码
  身份和 workflow handoff。
- 下载的 v0.2.99 制品通过校验和、隔离 fresh-install 以及 v0.2.98 升级验收，临时
  根目录被精确清理；对象仓库没有变化。
- 英文、简体中文和日文投影同步；最终 Outcome 跟随对话语言，性能收益和宿主展示确认
  继续如实保留为未知。

## 验证计划

先执行格式、版本、文档、parity、治理和 workspace 检查，再执行昂贵的发布任务。随后
使用已评审 main 和仅 dispatch 的发布 workflow，保留失败候选证据，消费发布 handoff，
执行下载制品的 adopter 验收，最后绑定精确 provider 与分支/worktree 清理证据后关闭。
