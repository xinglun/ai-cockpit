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

すべての repository command は明示的な `--repo <path>` を受け取ります。record や decision を出す command は通常 JSON ですが、
`work-item outcome` は既定で localize された人間向け handoff を表示します。機械処理には `--json` を指定します。failed/unknown は pass ではありません。

| Group | Commands | Boundary |
| --- | --- | --- |
| Read-only | `inspect`、`observe`、`status`、`compatibility`、`migrate plan`、`knowledge query`、`capability show`、`diagnose`、`doctor` | repository state/evidence を読み、黙って修復しない。 |
| Setup | `attach`、`profile confirm`、`profile propose` | protocol state の作成/更新、profile の確認、read-only candidate の出力。 |
| Migration | `migrate apply --approved` | review 済みの repository schema migration だけを適用し、Runtime-bound migration receipt を作る。 |
| Governance | `preflight` | Contract を読み green/yellow/red decision と `reviewState` を返す。不完全・不確実な Contract は human-review yellow となり checkpoint を越えられない。 |
| Work Item | `work-item new`、`start`、`checkpoint`、`finish`、`archive`、`close` | skeleton または lifecycle record を作る。`close` は human decision が必要。 |
| Verification | `verify` | bounded command を実行し evidence を記録する。Work Item に bind できる。 |
| External evidence | `evidence import`、`evidence list`、`evidence policy`、`evidence purge-plan` | exact provider bytes の bind、bounded persistence policy の宣言、または決定論的な非破壊 disposal plan の生成。 |
| Audit | `audit export` | repository-bound な安定 event bundle を外部 retention owner へ handoff する。local immutability は主張しない。 |
| Adapter | `agent list/install/doctor/repair/detach`、`mcp` | 明示的に選択した repository-local Agent adapter を管理し、または stdio で JSON-RPC を提供する。すべて `--repo` に bind する。 |

## Important options

- `verify --command <program> --args <comma-separated>` は explicit command を常に fresh に実行します。
  `--work-item <id>` は receipt を記録し、同じく fresh execution を強制します。
- `--command` なしの `verify` は Cargo または npm を検出し、confirmed profile で cross-process reuse できます。
- `verify --workers <n>` は positive worker count を要求し concurrency を制限します。
- `start` は `--id`、`--intent`、`--goal` が必須です。green governed flow には `--authority authorized` が必要です。
- `work-item new --repo <path> --id <id> --mode <mode>` は `not_ready` skeleton を作ります。snapshot-derived facts だけを埋め、
  human field は空または `unknown` のままです。移行期の `start` も同じ writer を使います。repository-local の
  exclusive reservation により重複競合は fail closed になり、同じ ID では 1 件だけが成功し、異なる repository は独立して動作します。
- `work-item outcome --repo <path> --id <id>` は完了内容、問題、停止、リスク、不明点、判断、検証、影響、次の action の順で人間向け結果を表示します。
  automation には `--json` を使います。status marker と言語規則は[人間向け Outcome](outcome-report.ja.md)を参照してください。
- `profile propose --repo <path>` は read-only の `candidate`/`proposed` amendment を出力し、profile baseline を適用しません。
- `agent list --repo <path>` は read-only です。`agent install` だけが通常の adapter write entry point で、
  `--provider` が必要です（`auto` は安全な surface が 1 つだけの場合に限り、`AGENTS.md` では Codex を選びます）。`agent doctor --repo <path> --json`
  は strict state report を返し、0（verified）、1（degraded）、2（configuration error）、3（human intervention）の exit code を使います。
  managed section または ownership record が変更されていれば `repair` と `detach` は fail closed し、global Agent/MCP config は変更しません。
- `preflight --contract` は通常 `start` が作る `.ai/work-items/active/<id>.contract.json` を指します。
- `work-item new` は `not_ready` の skeleton を作ります。これを `preflight` すると意図的に
  `yellow` と `reviewState: needs_human_confirmation` になり、人の項目を埋めてから再度 preflight して checkpoint します。
- `close --human-decision approved|rejected` は human decision record であり verification evidence ではありません。
- `evidence import --repo <path> --work-item <id> --metadata <metadata.json>
  --raw <provider-output>` は strict な `DelegatedEvidence` metadata を exact raw-byte
  digest と照合し、`.ai/evidence/external/` に repository/Work Item-bound receipt を書きます。
  `evidence list` は receipt を再検証し、expired/revoked provider claim を authority に変えません。
- `evidence policy --repo <path> --work-item <id> --classification <value>
  --persistence <value> --retention-days <n>|--expires-at <timestamp>
  --disposal-action <action>` は strict な retention policy を bind します。
  `secret_prohibited` は `full_capture` と `redacted_capture` を拒否し、
  `digest_only` は command の raw output を保存せず、`no_persistence` は completion
  evidence を保存できない場合に fail closed します。`evidence purge-plan --repo <path>`
  は決定論的な plan だけを出力し、自動削除はしません。
- `audit export --repo <path> [--output <file>]` は event ID、subject digest、repository/Work Item identity、
  Runtime identity を含む安定した `AuditEvent` を出力します。manifest は
  `externalRetentionRequired: true` を設定し、output file は idempotent です。これは SIEM、WORM、
  S3 Object Lock など外部 retention owner への handoff に限られます。
- 監査可能な decision には `--actor`、`--authority-source`、`--reason`、`--decided-at` と、任意の
  `--evidence-ref`、`--policy-ref`、`--resume-condition` を指定します。結果の `structuredDecision` は
  `.ai/decisions/<id>.close.json` に保存されます。legacy flag も明示的なまま、`legacy-cli` provenance を付けて記録します。
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

`tests/conformance/final_replacement_acceptance.sh` は source repository の最終置換 boundary です。installed Runtime identity、固定した
reference oracle、conformance/adversarial/performance gate、コピーなし検査を記録し、`acceptance.json` と `SHA256SUMS` を生成します。
