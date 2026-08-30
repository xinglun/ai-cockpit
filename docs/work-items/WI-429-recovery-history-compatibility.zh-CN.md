---
author: AI Cockpit maintainers
title: "WI-429——历史 recovery 投影"
description: 在不削弱 fail-closed 校验的前提下解决归档 recovery 残留。
audience: [contributor, maintainer, reviewer]
status: implemented
authority: human-authorized
workItemId: WI-429-recovery-history-compatibility
lastVerifiedBy: WI-429-recovery-history-compatibility
terminalArchive: .ai/work-items/archive/WI-429-recovery-history-compatibility.contract.json
terminalVerification: .ai/evidence/WI-429-recovery-history-compatibility.verification.json
terminalFinalization: .ai/decisions/WI-429-recovery-history-compatibility.finalize.json
terminalDecision: .ai/decisions/WI-429-recovery-history-compatibility.close.json
---

# WI-429——历史 recovery 投影

## 意图与边界

归档 recovery chain 可能保留一个从未完成目标绑定的旧 successor 尝试，随后又有
有效的 supersede receipt。Runtime 必须投影有效的终态决定，同时不改写不可变历史。

范围包括：

- 只识别严格限定的历史 successor-binding 残留；
- 按记录的决定时间让较新的有效 `supersede` 胜出；
- malformed、foreign、被篡改或较新但无效的记录继续 fail closed；
- 增加 Rust 回归测试及三语工作流/parity 文档。

不包括：重写历史治理 bytes、全面 recovery graph 重构、release/CI 路由，或全局 Agent/MCP 配置。

## 验收与证据

当最新可信 recovery decision 是有效 supersede 时，前置项必须可展示并可关闭；没有该决定时，
同样的残留必须保持可见失败。Contract、Summary、Outcome、Events、Evidence 和 recovery receipt
bytes 必须原样保留。

评审 PR 合并后，在 `.ai/evidence/` 和 `.ai/decisions/` 记录 verification 与终态 receipts。

[English](WI-429-recovery-history-compatibility.md) · [日本語](WI-429-recovery-history-compatibility.ja.md)
