---
author: AI Cockpit maintainers
title: "命令参考"
description: "当前 CLI 命令面及其修改或 evidence 边界。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - cli_commands
---

# 命令参考

除非另有说明，repository 命令都接受显式 `--repo <path>`。产生记录或 decision 的命令输出
为 JSON；失败或 unknown 不能算 pass。

| 分组 | 命令 | 边界 |
| --- | --- | --- |
| 只读 | `inspect`、`observe`、`status`、`knowledge query`、`doctor` | 读取 repository 状态或 evidence，不静默修复。 |
| 准备 | `attach`、`profile confirm` | 显式创建/更新协议状态或确认 profile。 |
| 治理 | `preflight` | 读取 Contract，返回 green/yellow/red decision。 |
| Work Item | `start`、`checkpoint`、`finish`、`archive`、`close` | 写入显式生命周期记录；`close` 要求 human decision。 |
| Verification | `verify` | 执行有界命令、记录 evidence，并可绑定 Work Item。 |
| Adapter | `mcp` | 通过 stdio 提供 JSON-RPC；`--repo` 绑定 repository 工具。 |

## 重要选项

- `verify --command <program> --args <comma-separated>` 执行显式命令且总是 fresh；`--work-item <id>`
  记录该 Work Item 的 receipt，也总是 fresh。
- 不提供 `--command` 的 `verify` 会检测 Cargo 或 npm，并可能使用已确认 profile 做跨进程 reuse。
- `verify --workers <n>` 要求正数并限制并发。
- `start` 要求 `--id`、`--intent`、`--goal`；要得到 green governed flow 需要 `--authority authorized`。
- `preflight --contract` 通常指向 `start` 生成的 `.ai/work-items/active/<id>.contract.json`。
- `close --human-decision approved|rejected` 是 human decision 记录，不是 verification evidence。

## Runtime identity

`inspect`、`doctor`、MCP `initialize` 和 verification evidence 会提供 runtime version、runtime digest、
protocol version。`ai-cockpit --version` 只输出简短的 executable version。
