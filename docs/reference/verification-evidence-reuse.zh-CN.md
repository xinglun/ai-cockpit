---
author: AI Cockpit maintainers
title: Verification 证据复用决策
description: 安全且可测量的 Verification 复用边界。
audience: [adopter, maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/reference/verification-evidence-reuse.md
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Verification 证据复用决策

[English](verification-evidence-reuse.md) · [简体中文](verification-evidence-reuse.zh-CN.md) · [日本語](verification-evidence-reuse.ja.md)

Evidence classifier 先判断 receipt 是 fresh、stale 还是 unknown；planner 消费这个
判断，bounded adapter 执行所需检查。fresh receipt 只能跳过 allowlisted、非受保护
节点；unknown 或 stale 必须重新执行。安全、scope、治理、coverage、source-bound
等受保护节点不能因复用而跳过。

## 必需绑定

复用要求 base/head revision、规范化 changed paths、命令及其 digest、环境/toolchain、
policy、stage、runner、repository/Work Item identity 和输出 receipt digest 精确一致。
任一绑定改变都会失效；Runtime 不从时间、缓存标签或 provider 结果推断安全。

## 成本与限制

adapter 报告 planned、executed、reused、stale、unknown 和 protected 调用次数。只有
无关变更减少真实调用且受保护调用不变，才构成优化观测。不会从本地运行推断 provider
等待、人类等待、P95 或 assurance 提升。参考源的 Python/Make 编排和 JSONL 记录只作
参考；Rust 通过类型化、repository-bound receipt 保持同一信任边界。
