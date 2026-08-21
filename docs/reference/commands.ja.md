---
author: AI Cockpit maintainers
title: "Command reference"
description: "現在の CLI command surface と mutation/evidence boundary。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - cli_commands
---

# Command reference

特に記載がない限り repository command は明示的な `--repo <path>` を受け取ります。record や
decision を出す command は JSON を出力し、failed/unknown は pass ではありません。

| Group | Commands | Boundary |
| --- | --- | --- |
| Read-only | `inspect`、`observe`、`status`、`knowledge query`、`doctor` | repository state/evidence を読み、黙って修復しない。 |
| Setup | `attach`、`profile confirm` | protocol state を明示的に作成/更新、または profile を確認する。 |
| Governance | `preflight` | Contract を読み green/yellow/red decision を返す。 |
| Work Item | `start`、`checkpoint`、`finish`、`archive`、`close` | lifecycle record を書く。`close` は human decision が必要。 |
| Verification | `verify` | bounded command を実行し evidence を記録する。Work Item に bind できる。 |
| Adapter | `mcp` | stdio で JSON-RPC を提供する。`--repo` が repository tool を bind する。 |

## Important options

- `verify --command <program> --args <comma-separated>` は explicit command を常に fresh に実行します。
  `--work-item <id>` は receipt を記録し、同じく fresh execution を強制します。
- `--command` なしの `verify` は Cargo または npm を検出し、confirmed profile で cross-process reuse できます。
- `verify --workers <n>` は positive worker count を要求し concurrency を制限します。
- `start` は `--id`、`--intent`、`--goal` が必須です。green governed flow には `--authority authorized` が必要です。
- `preflight --contract` は通常 `start` が作る `.ai/work-items/active/<id>.contract.json` を指します。
- `close --human-decision approved|rejected` は human decision record であり verification evidence ではありません。

## Runtime identity

`inspect`、`doctor`、MCP `initialize`、verification evidence は runtime version、runtime digest、protocol version を示します。
`ai-cockpit --version` は短い executable version だけを出します。
