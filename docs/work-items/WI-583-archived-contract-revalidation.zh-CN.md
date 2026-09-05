---
author: AI Cockpit maintainers
title: "WI-583——归档 Contract 重验证与后继关闭"
description: "为历史验证后经合法修订的归档 Work Item 提供追加式、有证据绑定的恢复路径。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-583-archived-contract-revalidation
lastVerifiedBy: WI-583-archived-contract-revalidation
---

[English](WI-583-archived-contract-revalidation.md) · [日本語](WI-583-archived-contract-revalidation.ja.md)

# WI-583——归档 Contract 重验证与后继关闭

## 目标

当 Work Item 归档后 Contract 经合法修订时，保留原始 archive 和 evidence
bytes，通过后继 Work Item 记录当前重验证，并在明确的人类授权下完成关闭，
不伪造 provider 结果。

## 边界

Runtime 及其绑定仓库的 CLI 属于本 WI。对象工程
`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` 是外部只读
adopter，本 WI 不修改它。源模板 wire format、发布、CI policy 重设计和
provider 操作不在范围内。

## 设计

`work-item revalidate-archived --repo <repository> --id <predecessor> --successor <successor>`
在验证归档 Contract、archive manifest 和历史 verification evidence 后，写入
追加式 recovery decision。Runtime 绑定当前 Contract digest、历史 evidence
digest、repository identity、manifest 和人类授权，然后生成 successor 骨架。
Successor 必须完成正常生命周期和新的验证，前件才能关闭。

历史 evidence 永不重写，也不会被提升为当前绿色证据。前件关闭记录历史/当前
身份及人类决定。缺失、损坏、过期、外部仓库、符号链接或矛盾证据均保持
fail-closed。

## 验收

1. 覆盖归档后 Contract 修订且原始 verification 不变的回归 fixture。
2. 前件仍待关闭时，可创建并验证 successor 重验证记录。
3. Successor 完成 `start → preflight → checkpoint → verify → finish → archive → finalize → finalize-verify → close`。
4. 仅在 successor 有效后关闭前件，并记录历史/当前证据区别和 lineage。
5. 篡改、缺失、损坏、外部、过期或符号链接证据均拒绝且不写仓库。
6. 三语命令和工作流文档说明追加式及历史证据边界。

## 验证

Contract 声明 Rust protocol/repository/CLI 聚焦测试、workspace 测试、格式化、
clippy 和文档质量门。Evidence 只由带显式仓库上下文的已安装 Runtime 生成。
