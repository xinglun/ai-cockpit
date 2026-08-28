---
author: AI Cockpit maintainers
title: "操作时策略重新评估"
description: "在高风险操作前立即获取新鲜、fail-closed 的策略事实。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# 操作时策略重新评估

[English](operation-time-policy-reevaluation.md) · [日本語](operation-time-policy-reevaluation.ja.md)

创建脚本、计划或批准并不授权之后执行。执行器即将执行高风险操作时，adapter
可以把严格的 `OperationTimeRequest` 交给 Rust Core evaluator。请求绑定：

- 请求的操作与实际工具调用；
- 目标资源和精确声明范围；
- 之前批准的操作、目标和范围；
- 当前可归属的权限；
- 证据新鲜度、破坏性影响分类和输入信任。

评估器返回 `allow`、`confirm` 或 `block` 事实。它不会执行操作、写入 provider
资源或授予 provider 权限。未知操作、未分类影响、空范围、绑定不一致、过期证据
或非权威输入，都不能自动放行。

支持的高风险词汇包括删除、测试/CI/分支保护变更、写入 secret、push、merge、
release、迁移、执行脚本、外部 API 写入、安装/升级和卸载治理。评估后，Provider
和 Agent 仍需执行自身的权限和保护分支控制。

这是共享 Runtime 能力。每个 adopter 在外围命令/adapter 中提供显式仓库上下文，
不创建全局当前项目或批准状态。操作时评估是策略输入，不是 provider 或企业批准
已经发生的证据。
