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
| Read-only | `inspect`、`observe`、`status`、`compatibility`、`migrate plan`、`knowledge query`、`doctor` | repository state/evidence を読み、黙って修復しない。 |
| Setup | `attach`、`profile confirm`、`profile propose` | protocol state の作成/更新、profile の確認、read-only candidate の出力。 |
| Migration | `migrate apply --approved` | review 済みの repository schema migration だけを適用し、Runtime-bound migration receipt を作る。 |
| Governance | `preflight` | Contract を読み green/yellow/red decision を返す。 |
| Work Item | `work-item new`、`start`、`checkpoint`、`finish`、`archive`、`close` | skeleton または lifecycle record を作る。`close` は human decision が必要。 |
| Verification | `verify` | bounded command を実行し evidence を記録する。Work Item に bind できる。 |
| Adapter | `agent list/install/doctor/repair/detach`、`mcp` | 明示的に選択した repository-local Agent adapter を管理し、または stdio で JSON-RPC を提供する。すべて `--repo` に bind する。 |

## Important options

- `verify --command <program> --args <comma-separated>` は explicit command を常に fresh に実行します。
  `--work-item <id>` は receipt を記録し、同じく fresh execution を強制します。
- `--command` なしの `verify` は Cargo または npm を検出し、confirmed profile で cross-process reuse できます。
- `verify --workers <n>` は positive worker count を要求し concurrency を制限します。
- `start` は `--id`、`--intent`、`--goal` が必須です。green governed flow には `--authority authorized` が必要です。
- `work-item new --repo <path> --id <id> --mode <mode>` は `not_ready` skeleton を作ります。snapshot-derived facts だけを埋め、
  human field は空または `unknown` のままです。移行期の `start` も同じ writer を使います。
- `profile propose --repo <path>` は read-only の `candidate`/`proposed` amendment を出力し、profile baseline を適用しません。
- `agent list --repo <path>` は read-only です。`agent install` だけが通常の adapter write entry point で、
  `--provider` が必要です（`auto` は安全な surface が 1 つだけの場合に限り、`AGENTS.md` では Codex を選びます）。`agent doctor --repo <path> --json`
  は strict state report を返し、0（verified）、1（degraded）、2（configuration error）、3（human intervention）の exit code を使います。
  managed section または ownership record が変更されていれば `repair` と `detach` は fail closed し、global Agent/MCP config は変更しません。
- `preflight --contract` は通常 `start` が作る `.ai/work-items/active/<id>.contract.json` を指します。
- `close --human-decision approved|rejected` は human decision record であり verification evidence ではありません。
- `compatibility --repo <path>` は installed Runtime と attached repository schema の
  `COMPATIBLE`、`MIGRATION_REQUIRED`、`INCOMPATIBLE` を返します。`migrate plan` は read-only です。
  `migrate apply` は `--approved` がなければ書き込まず、Work Item、evidence、decision、knowledge、
  archive history を書き換えません。
- attached protocol file が揃った後の stateful governance command は `COMPATIBLE` を要求します。
  `MIGRATION_REQUIRED` と `INCOMPATIBLE` は、新しい Work Item、lifecycle record、verification evidence、
  profile/adapter write、governed MCP operation を作成する前に fail closed します。migration review 用の
  read-only diagnostic は引き続き利用できます。

## Runtime identity

`inspect`、`doctor`、MCP `initialize`、verification evidence は runtime version、runtime digest、protocol version を示します。
`ai-cockpit --version` は短い executable version だけを出します。

## Release acceptance の境界

`tests/release/adopter_acceptance.sh` は maintainer 向けの post-release harness であり、Runtime command ではありません。
public Release binary を download して pin し、isolated directory で adopter lifecycle を実行し、`acceptance.json` と `SHA256SUMS` を生成します。
workspace build や local target binary で代用してはならず、acceptance failure が公開済み Release truth を変更することもありません。
