---
author: AI Cockpit 维护者
title: "WI-370——验证性能预算与精确复用"
description: "在不削弱治理的前提下，通过动态、身份绑定的复用降低重复验证延迟。"
workItemId: WI-370-verification-performance-budget
audience: [adopter, maintainer, reviewer]
status: recovered
authority: human-authorized
lastVerifiedBy: WI-370-verification-performance-budget
capabilityClaims: [verification_performance, evidence_integrity]
---

# WI-370——验证性能预算与精确复用

[English](WI-370-verification-performance-budget.md) · [日本語](WI-370-verification-performance-budget.ja.md)

## 意图与边界

本 Work Item 降低当前仓库和 adopter 仓库的重复验证延迟。检测到的 Work Item
命令可以使用由 Profile 授权的动态路径，但只有身份完全匹配的 receipt 才能复用；
显式自定义命令保持 fresh。仓库快照、Contract、Scope、命令、Stage、Runner、
Runtime、Profile、Toolchain、Dependency 或 Policy 任一发生变化，都会强制重新执行
或按 Policy 升级验证。

必需和受保护的治理检查永远不会跳过；未知影响也不会因计时或缓存状态变成 Green。
Rust Runtime 仍是共享的一份安装；adopter 仓库继承相同的选择规则，但证据和
Repository Identity 保持隔离。

## 验证与验收

- 选择结果稳定报告 executed、reused、escalated、denied 及其原因。
- 复用绑定 repository、profile、Runtime、command、scope、stage、runner、base、
  toolchain、dependency 和 policy context。
- 复用结果为当前 Work Item 写入新的 evidence，不能授权另一个 Work Item。
- 当前工程和发布版 adopter 测量保留 cold/warm 耗时及 Runtime/Repository Identity。
- 三语参考文档说明性能只是成本优化，不改变验证真相或必需门禁。

归档 Contract 和 verification evidence 是机器可读的权威记录。本页是面向读者的
Work Item 投影；只有在 provider merge 与清理验证完成后，才补充最终终态链接。
