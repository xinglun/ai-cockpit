---
author: AI Cockpit maintainers
title: "機能一覧と境界"
description: "AI Cockpit runtime の現在の機能と、外部に残る責任を reader-first に説明します。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - cli_lifecycle
  - mcp_adapter
  - agent_discovery_adapter
  - bounded_verification
---

# 機能一覧と境界

## 目的

このページを現在の機能 index として使ってください。各行に、利用者ができること、
開始 command、生成される state または evidence を示します。

## 開始前

`ai-cockpit` binary を install または build し、Git repository に向けてください。
`inspect` は read-only、`attach` は推奨する明示的な準備操作で `.ai/` を作成できます。
protocol file がない場合、`start` も bootstrap できます。evidence reuse の前に attached
profile を確認してください。

## 用語

- **snapshot**: Git と関連 file digest を含む一回の repository 観測結果。
- **profile**: controlled reuse に使う quality command の明示的な確認結果。
- **receipt**: 一回の verification result に content-bound された evidence。
- **bounded verification**: worker 上限、timeout、bounded output capture 付きの実行。
- **reuse**: すべての identity binding が一致したときだけ command を省略すること。
- **fail closed**: evidence が不足・矛盾したら rerun、unknown、または停止にすること。

## 機能一覧

| 機能 | 利用者ができること | 開始点 | 結果 |
| --- | --- | --- | --- |
| Inspect | repository state を変更せず読む。 | `ai-cockpit inspect --repo <path>` | Git identity、changed paths、digest、runtime identity。 |
| Attach | minimum の repository-owned governance scaffold を作る。 | `ai-cockpit attach --repo <path>` | `.ai/` protocol file、discovery manifest、state directory、calibration state。 |
| Compatibility と migration | installed Runtime が repository を安全に扱えるか確認し、必要なときだけ明示的な schema migration を適用する。 | `compatibility`、`migrate plan`、`migrate apply --approved` | `COMPATIBLE`、`MIGRATION_REQUIRED`、`INCOMPATIBLE`。承認済み migration は Runtime-bound receipt を作る。 |
| Observe | attached profile と repository facts を読む。 | `ai-cockpit observe --repo <path>` | observation と evolution signal。 |
| Preflight | edit 前に Work Item contract を評価する。 | `ai-cockpit preflight --repo <path> --contract <file>` | green、yellow、red の governance decision。 |
| Work Item lifecycle | bounded work を start、checkpoint、finish、archive、close する。 | `start`、`checkpoint`、`finish`、`archive`、`close` | 明示的な state transition と receipt。 |
| Verification | allowlist/profile command を制限内で実行する。 | `ai-cockpit verify --repo <path> ...` | pass/fail/unknown と execution evidence。 |
| Evidence reuse | identity binding が一致するときだけ再実行を省略する。 | confirmed profile + automatic `verify` | reuse または fail-closed rerun。 |
| Knowledge | repository-local の完了済み evidence を query する。 | `ai-cockpit knowledge query --repo <path>` | filtered result。第二の fact source ではない。 |
| MCP | 同じ repository service を MCP client に公開する。 | `ai-cockpit mcp --repo <path>` | explicit binding 付き JSON-RPC result。 |
| Doctor | runtime と repository の readiness を診断する。 | `ai-cockpit doctor --repo <path>` | action 可能な診断。黙って修復しない。 |
| Profile confirmation | controlled reuse 用の quality command を確認する。 | `ai-cockpit profile confirm --repo <path> --program cargo --args test,--workspace` | review 可能な profile version。 |
| Work Item scaffold | governance decision を発明せず validator-readable skeleton を作る。 | `ai-cockpit work-item new --repo <path> --id <id> --mode <mode>` | `not_ready` Contract、snapshot facts、人間入力の一覧。 |
| Profile proposal | formal baseline を変更せず candidate profile amendment を作る。 | `ai-cockpit profile propose --repo <path>` | read-only の `candidate`/`proposed` output。 |
| Agent adapter | 選択した Agent host が repository-owned section からこの repository を発見できるようにする。 | `ai-cockpit agent list/install/doctor --repo <path>` | repository-bound discovery、ownership、state、安全な action。global config は変更しない。 |

## 利用者向けの詳細 path

### Repository を inspect する

**依頼の例:** 「変更せずに repository state を表示して。」

```bash
ai-cockpit inspect --repo /path/to/repository
```

repository root、Git head、changed paths、tree/diff digest、dependency fingerprint、read/hash
counter、runtime identity を報告します。discover または Git が失敗したら停止し、path を修正してください。

### Repository を attach・observe する

```bash
ai-cockpit attach --repo /path/to/repository
ai-cockpit observe --repo /path/to/repository
```

Attach は minimum の repository-owned scaffold を作ります。

```text
.ai/
├── cockpit.toml
├── project.json
├── agent-interface.json
├── work-items/active/
├── work-items/archive/
├── evidence/
├── decisions/
└── knowledge/
```

Rust source、V1 runtime file、Python helper、provider instruction、runtime schema は target に copy しません。
初期 profile は `calibration_required` です。controlled reuse の前に quality command を確認します。

```bash
ai-cockpit profile confirm --repo /path/to/repository \
  --program cargo --args test,--workspace
```

`agent-interface.json` は repository-local の discovery fact です。stable repository identity と Runtime capability
だけを記録し、Agent prompt、provider install、authorization、global MCP setting にはなりません。

### Runtime upgrade と repository migration

Runtime upgrade と repository migration は別の操作です。互換性のある Runtime upgrade は `.ai/` を
書き換えず、global な current repository も作りません。まず明示した repository と installed Runtime
の互換性を確認します。

```bash
ai-cockpit compatibility --repo /path/to/repository
ai-cockpit migrate plan --repo /path/to/repository
```

結果が `MIGRATION_REQUIRED` なら plan を確認してから明示的に承認します。

```bash
ai-cockpit migrate apply --repo /path/to/repository --approved
```

Migration receipt は source/target schema、前後 digest、Runtime version、Runtime digest を記録します。
変更するのは versioned protocol file と migration record だけで、archive Work Item、evidence、decision、
knowledge は byte-for-byte の履歴として保持します。`INCOMPATIBLE` は書き込み前に停止し、その schema を
理解する Runtime が必要です。

attached protocol file が揃っている場合、stateful な governance operation（`preflight`、Work Item の
作成/lifecycle、`verify`、knowledge/profile の書き込み、Agent adapter の書き込み、governed MCP call）は
`COMPATIBLE` でなければ実行できません。`MIGRATION_REQUIRED` または `INCOMPATIBLE` なら新しい record や
evidence を作る前に停止します。compatibility、migration plan、observe、status、doctor などの read-only
diagnostic は利用でき、次の安全な操作を確認できます。

### Agent を明示的に接続する

`attach` は repository fact だけを作成し、`AGENTS.md`、`CLAUDE.md`、`GEMINI.md`、`.cursor/`、
home directory の設定は変更しません。Agent host にこの repository を発見させる場合は provider を明示します。

```bash
ai-cockpit agent list --repo /path/to/repository
ai-cockpit agent install --repo /path/to/repository --provider codex
ai-cockpit agent doctor --repo /path/to/repository --json
```

Adapter が書き込むのは選択した repository surface の marker 付き section と
`.ai/adapters/<provider>.json` だけで、無関係な bytes は保持します。`doctor` は現在の fact から
`UNATTACHED`、`DISCOVERY_AVAILABLE`、`VERIFIED`、`DEGRADED`、`CONFLICT` を導出し、prompt を governance authority にしません。
managed section が変更・重複・不明確な場合、`repair` と `detach` は上書きせず拒否します。

```bash
ai-cockpit agent repair --repo /path/to/repository --provider codex
ai-cockpit agent detach --repo /path/to/repository --provider codex
```

Discovery、adapter install、connection、verification、compliance は別の state です。MCP は optional であり、
CLI は MCP なしでも使用できます。これらの command は provider の global configuration を変更しません。

### Work Item skeleton を作る

人間の判断がまだ準備できていない場合は scaffold を使います。

```bash
ai-cockpit work-item new --repo /path/to/repository \
  --id payment-refund-guard --mode code
```

自動入力されるのは `repositoryId`、`baseRevision`、`projectProfileDigest`、`repositorySnapshotDigest` だけです。
`intent`、`scope`、`acceptanceCriteria`、`authority` は空または `unknown` のままです。Contract と summary は
`not_ready` になり、`passed`、`approved`、`verified`、`completed` は生成しません。CLI は既知の fact と不足している
人間入力を表示します。移行期の `start` は同じ scaffold writer を明示的な human field とともに使います。

### Profile amendment を提案する

```bash
ai-cockpit profile propose --repo /path/to/repository
```

read-only の `candidate`/`proposed` amendment を出力します。formal `.ai/project.json` の bytes/digest は変更せず、
baseline の変更には将来の明示的な apply decision が必要です。

### Work Item を Preflight する

`start` が `preflight` 用の contract を作成します。

```bash
ai-cockpit start --repo /path/to/repository --id WI-123 \
  --intent "Improve documentation" \
  --goal "Explain installation clearly" \
  --scope 'docs/**' --authority authorized \
  --acceptance "examples work"
ai-cockpit preflight --repo /path/to/repository \
  --contract .ai/work-items/active/WI-123.contract.json
```

current snapshot に対して contract を評価します。authority の欠落、stale contract、scope
violation、矛盾した fact は stop condition です。

### Governed Work Item を実行する

**依頼の例:** 「bounded change を開始し、進捗を記録し、review 後にだけ close して。」

```bash
# preflight が受け入れられた後、docs/** だけを編集する
ai-cockpit checkpoint --repo /path/to/repository --id WI-123
ai-cockpit verify --repo /path/to/repository --work-item WI-123 \
  --command cargo --args test,--workspace --workers 2
ai-cockpit finish --repo /path/to/repository --id WI-123
ai-cockpit archive --repo /path/to/repository --id WI-123
ai-cockpit close --repo /path/to/repository --id WI-123 \
  --human-decision approved
```

期待される state は `implementation_active`、`checkpointed`、`finish_ready`、`archived`、
`closed` です。`finish` は同じ Work Item と current repository snapshot の passed verification
receipt を要求し、`close` は archive manifest と human decision を要求します。失敗したら
Work Item を残し、evidence を修復します。record を削除して状態を隠してはいけません。

### Verification と reuse

Explicit command と Work Item-bound verification は常に fresh です。

```bash
ai-cockpit verify --repo /path/to/repository \
  --command cargo --args test,--workspace --workers 2
```

Automatic detection は confirmed profile を使い、persisted receipt を reuse できます。

```bash
ai-cockpit verify --repo /path/to/repository
ai-cockpit verify --repo /path/to/repository
```

2 回目に `nodesReused: 1`、`processesSpawned: 0` になる場合があります。repository snapshot、
source/base revision、profile、toolchain、environment、executable identity、scope、policy、
stage、runner、command、output identity がすべて一致した場合だけ reuse します。protected gate、
explicit command、Work Item run は fresh です。不一致は rerun または unknown/blocked になります。

制限は command timeout 300 秒、stdout/stderr 各 64 KiB、positive worker count です。output が
truncated と表示されることがあります。timeout、capture、process-tree failure は pass ではありません。
receipt-store index は 8 MiB、reusable receipt は 1 MiB までです。malformed、oversized、symlink、
inconsistent entry は fail closed になります。

### Knowledge と status を query する

```bash
ai-cockpit status --repo /path/to/repository
ai-cockpit knowledge query --repo /path/to/repository --topic installation
```

Knowledge は repository-local evidence の projection で、第二の source of truth ではありません。
Work Item や receipt が missing、stale、invalid なら新しい claim に変換しません。

### MCP を使う

explicit repository binding で server を起動します。

```bash
ai-cockpit mcp --repo /path/to/repository
```

`status`、`work_item_get`、`work_item_list`、`blockers`、`safe_actions`、`knowledge_query`、
`evidence_get`、`repository_observe`、`preflight`、`verify` の 10 tools を提供します。
`tools/list` で JSON-RPC schema を確認できます。`preflight` は repository-relative `contract`、
`verify` は `command`、string array の `args`、optional `workItemId` を受け取ります。repository
binding のない call は fail closed です。result には `structuredContent`、text content、`isError`
が含まれ、CLI と同じ repository-bound verification policy を使います。

### Readiness を診断する

```bash
ai-cockpit doctor --repo /path/to/repository
```

Doctor は runtime version/digest、protocol state、repository identity、action 可能な問題を報告します。
一般的な security scanner ではなく、external identity、provider、branch、production control の充足も主張しません。

Enterprise adopter は [Enterprise governance boundary](security/enterprise-governance.ja.md) も参照してください。
assurance level、policy precedence、delegated evidence、sensitive data persistence、retention、external audit
export の境界を説明します。

## AI Cockpit が主張しないこと

AI Cockpit は Agent Runtime、Workflow Engine、Security Sandbox、general prompt-injection detector、
identity provider、compliance certificate、human review の代替ではありません。external identity、
branch protection、production isolation、signing、SBOM、provenance、enterprise policy は外部
evidence または adopter の責任です。

## Stop と recovery

missing または矛盾した evidence への安全な応答は、停止し、Work Item と receipt を保持し、gap を説明し、
関連 fact を修復してから rerun することです。green の command output で red の governance state を上書きしません。

## 次に読むもの

1. [Installation と distribution](release/distribution.ja.md)
2. [アーキテクチャ](architecture.ja.md)
3. [設計思想](philosophy.ja.md)
4. [Repository Protocol v1](protocol/v1/specification.ja.md)
