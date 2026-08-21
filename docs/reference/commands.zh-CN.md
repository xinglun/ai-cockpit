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

所有 repository 命令都接受显式 `--repo <path>`。产生记录或 decision 的命令输出为 JSON；
`work-item new` 会先输出简短的人类可读摘要，再输出 JSON 记录。失败或 unknown 不能算 pass。

| 分组 | 命令 | 边界 |
| --- | --- | --- |
| 只读 | `inspect`、`observe`、`status`、`knowledge query`、`doctor` | 读取 repository 状态或 evidence，不静默修复。 |
| 准备 | `attach`、`profile confirm`、`profile propose` | 创建/更新协议状态、确认 profile，或输出只读候选。 |
| 治理 | `preflight` | 读取 Contract，返回 green/yellow/red decision。 |
| Work Item | `work-item new`、`start`、`checkpoint`、`finish`、`archive`、`close` | 创建骨架或写入显式生命周期记录；`close` 要求 human decision。 |
| Verification | `verify` | 执行有界命令、记录 evidence，并可绑定 Work Item。 |
| Adapter | `agent list/install/doctor/repair/detach`、`mcp` | 管理显式选择的 repository-local Agent adapter，或通过 stdio 提供 JSON-RPC；所有操作都绑定 `--repo`。 |

## 重要选项

- `verify --command <program> --args <comma-separated>` 执行显式命令且总是 fresh；`--work-item <id>`
  记录该 Work Item 的 receipt，也总是 fresh。
- 不提供 `--command` 的 `verify` 会检测 Cargo 或 npm，并可能使用已确认 profile 做跨进程 reuse。
- `verify --workers <n>` 要求正数并限制并发。
- `start` 要求 `--id`、`--intent`、`--goal`；要得到 green governed flow 需要 `--authority authorized`。
- `work-item new --repo <path> --id <id> --mode <mode>` 创建 `not_ready` 骨架，只填充 snapshot-derived facts，
  人类字段保持空值或 `unknown`；过渡期 `start` 复用同一 writer。
- `profile propose --repo <path>` 只读输出 `candidate`/`proposed` amendment，不会应用 profile baseline 修改。
- `agent list --repo <path>` 是只读操作；`agent install` 是唯一正常的 adapter 写入口，必须指定
  `--provider`（`auto` 只有在恰好一个无歧义安全 surface 时可用；`AGENTS.md` 默认选择 Codex）。`agent doctor --repo <path> --json`
  返回严格状态报告，并使用 0（verified）、1（degraded）、2（配置错误）、3（需要人工介入）退出码。
  如果 managed section 或 ownership record 被修改，`repair` 和 `detach` 会 fail closed；任何命令都不会写入全局 Agent/MCP 配置。
- `preflight --contract` 通常指向 `start` 生成的 `.ai/work-items/active/<id>.contract.json`。
- `close --human-decision approved|rejected` 是 human decision 记录，不是 verification evidence。

## Runtime identity

`inspect`、`doctor`、MCP `initialize` 和 verification evidence 会提供 runtime version、runtime digest、
protocol version。`ai-cockpit --version` 只输出简短的 executable version。

## Release 验收边界

`tests/release/adopter_acceptance.sh` 是维护者侧的发布后 harness，不是 Runtime 命令。它下载并固定公开
Release binary，在隔离目录中执行 adopter lifecycle，并生成 `acceptance.json` 与 `SHA256SUMS`。不得用 workspace
build 或本地 target binary 替代；验收失败也不会改变已发布 Release truth。
