---
author: AI Cockpit maintainers
title: "WI-985——repository resource lifecycle 边界"
description: "在保持公开 API、receipt、错误与恢复行为不变的前提下，将 resource finalization、ordinary cleanup 和关闭时资源校验提取到同 crate 模块。"
audience: [maintainer, reviewer, contributor]
status: implemented
authority: human:user
workItemId: WI-985-repository-resource-lifecycle
lastVerifiedBy: WI-985-repository-resource-lifecycle
terminalArchive: .ai/work-items/archive/WI-985-repository-resource-lifecycle.contract.json
terminalVerification: .ai/evidence/WI-985-repository-resource-lifecycle.verification.json
terminalDecision: .ai/decisions/WI-985-repository-resource-lifecycle.close.json
---

[English](WI-985-repository-resource-lifecycle.md) · [日本語](WI-985-repository-resource-lifecycle.ja.md)

# WI-985——repository resource lifecycle 边界

本 Work Item 收紧 `cockpit-repository` 内已有的同 crate 模块边界。资源
finalization、ordinary cleanup 和关闭时资源校验将移入
`resource_lifecycle.rs`；`lib.rs` 保留稳定公开导出与通用 repository 原语。

本次保持公开函数签名、JSON 与 receipt schema、文件路径、写入顺序、错误语义、
历史读取、恢复行为以及重复、过期、外部绑定、dirty、缺失或未知资源的
fail-closed 处理不变。不声称运行性能提升，也不修改 protocol、CLI/MCP、发布脚本
或历史兼容行为。

验收依据包括 finalization 与 cleanup 聚焦回归、archive/close 与 recovery 回归、
Contract 声明的 repository 和 CLI/MCP 测试，以及 Runtime 绑定的 verification 证据。
