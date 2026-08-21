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

すべての repository command は明示的な `--repo <path>` を受け取ります。record や decision を出す command は JSON を出力しますが、
`work-item new` は JSON record の前に短い human-readable summary も表示します。failed/unknown は pass ではありません。

| Group | Commands | Boundary |
| --- | --- | --- |
| Read-only | `inspect`、`observe`、`status`、`knowledge query`、`doctor` | repository state/evidence を読み、黙って修復しない。 |
| Setup | `attach`、`profile confirm`、`profile propose` | protocol state の作成/更新、profile の確認、read-only candidate の出力。 |
| Governance | `preflight` | Contract を読み green/yellow/red decision を返す。 |
| Work Item | `work-item new`、`start`、`checkpoint`、`finish`、`archive`、`close` | skeleton または lifecycle record を作る。`close` は human decision が必要。 |
| Verification | `verify` | bounded command を実行し evidence を記録する。Work Item に bind できる。 |
| Adapter | `mcp` | stdio で JSON-RPC を提供する。`--repo` が repository tool を bind する。 |

## Important options

- `verify --command <program> --args <comma-separated>` は explicit command を常に fresh に実行します。
  `--work-item <id>` は receipt を記録し、同じく fresh execution を強制します。
- `--command` なしの `verify` は Cargo または npm を検出し、confirmed profile で cross-process reuse できます。
- `verify --workers <n>` は positive worker count を要求し concurrency を制限します。
- `start` は `--id`、`--intent`、`--goal` が必須です。green governed flow には `--authority authorized` が必要です。
- `work-item new --repo <path> --id <id> --mode <mode>` は `not_ready` skeleton を作ります。snapshot-derived facts だけを埋め、
  human field は空または `unknown` のままです。移行期の `start` も同じ writer を使います。
- `profile propose --repo <path>` は read-only の `candidate`/`proposed` amendment を出力し、profile baseline を適用しません。
- `preflight --contract` は通常 `start` が作る `.ai/work-items/active/<id>.contract.json` を指します。
- `close --human-decision approved|rejected` は human decision record であり verification evidence ではありません。

## Runtime identity

`inspect`、`doctor`、MCP `initialize`、verification evidence は runtime version、runtime digest、protocol version を示します。
`ai-cockpit --version` は短い executable version だけを出します。
