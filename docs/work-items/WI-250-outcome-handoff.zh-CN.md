---
author: AI Cockpit maintainers
title: "WI-250——直接生命周期 Outcome 交接"
workItemId: WI-250-outcome-handoff
description: "在不破坏 JSON 接口的前提下，让生命周期命令直接展示已校验的人类 Outcome。"
audience:
  - adopter
  - maintainer
status: recovered
lastVerifiedBy: WI-250-outcome-handoff
authority: canonical
---

# WI-250——直接生命周期 Outcome 交接

生命周期修改原先只在 stdout JSON 内返回 `outcome`。该记录对机器稳定，但嵌入的
Agent 或终端可能把面向人的交接折叠在工具输出中。WI-250 在 CLI 边界增加直接且
向后兼容的 handoff。

## 行为

- `finish`、`archive`、`close` 保持现有可解析 stdout JSON，并默认在 stderr
  渲染同一份经 Runtime 校验的本地化人类 Outcome。
- `--json` 只抑制 stderr handoff，保留机器专用模式。
- 被阻止的 `finish` 先渲染已持久化的红色或黄色 Outcome，再返回原有 nonzero
  错误；展示层绝不会绕过生命周期门禁。
- renderer 保留固定 `Outcome: 🔴/🟡/🟢` 标记，以及未知项、人工决定、
  证据和下一步章节；结构化 close decision 也通过同一投影可见。

## 边界

CLI 无法强制宿主应用打开或展开对话面板。宿主必须展示 stderr；人工可以用
`work-item outcome` 确定性重放持久交接。OutcomeV2、archive truth、MCP 与现有
历史 Work Item bytes 均不改变。

## 验证

CLI 集成测试覆盖三种语言、stdout 兼容、机器模式抑制、结构化 close decision 和
blocked fail-closed 行为。documentation acceptance、parity/governance gate、Rustfmt、
Clippy 与 locked workspace suite 仍为必需项。
