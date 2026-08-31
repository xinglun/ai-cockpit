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
| Knowledge | repository-local の完了済み evidence を query し、derived projection を明示的に materialize する。 | `ai-cockpit knowledge query --repo <path>` | filtered result と repository-local write boundary。第二の fact source ではない。 |
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

Runtime の実装や provider configuration は target に copy しません。初期 profile は
`calibration_required` です。controlled reuse の前に quality command を確認します。

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

新しい Runtime で repository を開くと、`migrate plan` と `status` は
`historicalFinalization` inventory も返します。有効な close binding を持つ旧 receipt は
`historical_verified`、assurance は `historical_low` として投影され、pending または読めない
predecessor は `recovery_required`/`invalid` のままです。明示的な recovery または完全な
direct-merge `finalize` の前に read-only の `work-item finalize-recovery-plan` で事実を確認します。
履歴 bytes は書き換えられず、historical status は新しい Work Item を authorize しません。

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

managed section は Agent を `.ai/README.md` に案内します。ここが repository-local の canonical な
利用 handoff であり、すべての repository-bound command に明示的な `--repo` を要求し、governed
lifecycle を示します。provider authorization や global MCP endpoint は設定しません。

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

いずれのコマンドも通常の次の Work Item を作る前に repository-scoped entry gate を評価します。Contract より前に存在する `.ai` 以外の変更、detached HEAD、現在の HEAD とローカルで検出された remote default revision の不一致、または有効な close decision のない archived Work Item は fail closed になります。gate は archived records を書き換えません。identity-bound recovery decision から作られた successor は predecessor の継続であり、gate を迂回する独立 Work Item ではありません。

top-level `status` の `readiness` に同じ read-only readiness projection が含まれます。名前付きの clean branch が検出された default revision に一致し、active Work Item と close 待ち archived Work Item がない場合だけ `readyOnBase` は `true` です。remote metadata が欠落または曖昧なら `state: unknown` であり、green にはなりません。`blocked` と `unclosedArchivedWorkItems` が正確な修復境界を示します。

skeleton の作成は repository と Work Item ID ごとに repository-local の exclusive reservation で
直列化されます。同じ ID に対する `work-item new` が競合した場合、Contract と summary を作成できるのは
正確に 1 件だけで、もう 1 件は fail closed になります。ペアのファイルが commit された後に reservation
は削除されます。異なる repository では reservation が分離され、同じ ID を並行して作成できます。

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

active Work Item に対する `preflight` は、decision、Contract digest、snapshot digest も summary に記録します。
次のステップで収集する verification evidence がまだないための yellow は checkpoint へ進めますが、red は進めません。
verification 完了時、Runtime は最終 snapshot に対して記録済み decision を再評価します。`finish` にはその結果の green と
ちょうど 1 回の checkpoint が必要です。checkpoint は一度だけの直列 transition であり、重複または順序外の command は
fail closed になります。失敗しても active record は残り、不足したステップを再実行して復旧できます。

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
`closed` です。`finish` は同じ Work Item と current repository snapshot の passed verification receipt、記録済みの
green preflight decision、ちょうど 1 回の checkpoint を要求します。Contract の `requiredEvidenceClasses` に verification
がなくても、`archive` と `close` は同じ serial state と verification evidence を再検証します。失敗したら Work Item を
残し、evidence を修復します。record を削除して状態を隠してはいけません。

`finish`、`archive`、`close` は bound された `outcome` object を stdout JSON に保持し、
既定では同じ localize 済み human report を stderr に render します。各 `--json` は stderr
handoff だけを抑止します。block された `finish` は永続化済みの赤/黄 Outcome を render してから
元の nonzero error を維持します。Agent は handoff を独立した会話メッセージとして明示してください。
ファイルにだけ保存された結果や折りたたまれた結果は delivery confirmation ではありません。
CLI は host UI の展開を強制できません。host は stderr を提示するか、`work-item outcome` を再生できます。
後者は既定で localize 済み report を stdout に表示し、その `--json` は stable object を返します。
[人間向け Outcome](reference/outcome-report.ja.md) を参照してください。

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
ai-cockpit work-item status --repo /path/to/repository --id WI-123 --json
ai-cockpit work-item status --repo /path/to/repository --all --json
ai-cockpit knowledge query --repo /path/to/repository --topic installation
```

Knowledge は repository-local evidence の projection で、第二の source of truth ではありません。
明示的な `knowledge query` は `.ai/knowledge/` の derived index を作成または rebuild することがあり、
`projection.writeBoundary=repository-local-derived` を返します。これは新しい変更を authorize しません。
Work Item や receipt が missing、stale、invalid なら新しい claim に変換しません。all-Work-Item
projection は ID 順で安定に並べ、green/yellow/red/unknown の count と item ごとの diagnostic を返し、
current repository snapshot と deterministic な index digest の両方に bind します。malformed または
foreign な member は可視の `unknown` のままで、他の member を隠したり fail open したりしません。
`observe`、`capability show`、status projection を繰り返しても request-scoped read のままで、tracked
capability/status file や observer cache を作りません。
`work-item inspect` も同じ read-only boundary に従い、implementation approach をメモリ上で計算するだけで
`.ai/work-items/active/<id>.approach.json` を作成・更新しません。repository-local の approach artifact を
明示的に保存する場合だけ `work-item approach` を使用します。

### Traceability、Outcome、parallel readiness

v2 intelligence projection は fact と derivation を分離し、人間が決める intent や authority を補いません。

```bash
ai-cockpit work-item approach --repo /path/to/repository --id WI-123
ai-cockpit work-item outcome --repo /path/to/repository --id WI-123
ai-cockpit work-item inspect --repo /path/to/repository --id WI-123
ai-cockpit work-item declare --repo /path/to/repository --id WI-123 \
  --depends-on WI-100 --conflicts-with WI-124 --parallelizable
ai-cockpit knowledge query --repo /path/to/repository --v2
ai-cockpit capability show --repo /path/to/repository
ai-cockpit diagnose --repo /path/to/repository --work-item WI-123
```

`approach` は observed fact、名前付き derivation、evidence reference、未解決の human input を出力します。
`outcome` は verified implementation evidence と Human Benefit Report を分離し、宣言されていない user benefit は
`unknown` のままです。新しい OutcomeV2 には evidence-bound section、`failedGate`/`recoveryCondition`、
append-only の `<id>.events.jsonl` を持つ strict な `taskOutcomeReport` も含まれます。`finish` が stream を作り、
`archive` が digest を bind し、`close` が validated report を `finalReport` として receipt に記録します。
過去の record は backfill しません。これは presentation/evidence projection であり approval source ではなく、
完全な event-sourced recovery は別 boundary です。Capability Registry は observed な technical fact と
adopter-facing Runtime claim を分離します。adopter state は `runtime_supported`、`repository_bound`、
`observed`、`profile_confirmed`、`adopter_accepted`、`external`、`unknown` です。Runtime は自身の identity、
current snapshot、strict profile、repository interface が裏付ける level だけを出力します。file の存在は
adopter acceptance ではなく、明示的な acceptance evidence がなければ static catalog は
`adopter_accepted` を出力しません。hosted CI、signing、SBOM、production sandbox などの exclusion は
external boundary のままです。missing、malformed、stale、foreign な input は verified claim ではなく
stable unknown になります。この registry は installed-surface manifest ではなく、reference template の
`templateFiles`、`installedFiles`、schema/entrypoint list、`verifyInstalledSurface` check を複製しません。
installer-surface manifest は external Release/adopter boundary として残り、repository には copy しません。

### Project capability と profile policy の宣言

Project は `.ai/project/` に repository-owned な JSON 宣言を置けます。

- `capabilities.json`: capability、non-capability、critical domain、Contract の明示的な
  operation が参照する厳密な `operationMappings`。
- `success_criteria.json`: project criteria と evidence hint の可視 projection。Contract の
  acceptance を置き換えず、approval も作りません。
- `profile-policy.json`: approved path boundary、critical path、review requirement、明示的な
  unknown。`.ai/project.json` は strict な identity と observed-quality profile として残ります。

各宣言は strict schema、regular file、repository identity、review 時点の repository snapshot に bind されます。
`capability show` と MCP `capability_show` は semantic digest、authority を持たない success criteria、stable unknown code を表示しますが、宣言を
書き換えません。Contract に `operation` または `requestedOperation` が明示される場合、Preflight は一致し十分な
mapping を要求します。missing、malformed、foreign、stale、conflict、insufficient な宣言は yellow/unknown のままです。
Contract の intent prose や検出した file から mapping を推論しません。明示的な operation がない legacy Contract は従来どおりです。

これは Rust-native governance projection であり、reference Python runtime、Make target、installer manifest の copy ではありません。
`attach` は宣言を発明せず、project success criteria は Work Item を authorize しません。
`inspect` は dependency、conflict、scope compatibility が明示的に分からない場合に
parallel execution を fail closed にします。Scope compatibility は Windows の `\\` separator を正規化し、exact path
と nested prefix の overlap（`src/**` と `src/main.rs`、`src/test/**` など）を検出します。交差を証明できない
pattern は `scope_overlap_unknown` になり、unknown または空の scope は parallel execution と互換になりません。
Diagnosis は実測した snapshot/verification cost だけを報告し、benchmark を装いません。

Verification evidence は strict な v2 envelope です。unknown な envelope field、malformed な captured
receipt、nested Work Item/repository/Runtime identity の欠落は fail closed になります。現在の CLI lifecycle
command は実行した Runtime の version と digest に evidence を bind するため、foreign Runtime は current
Work Item を authorize できません。pre-v2 evidence は immutable な historical input です。Outcome は黄色の
`legacy_evidence_historical` として表示し、現在の赤い失敗や fresh green として扱いません。current v2 evidence
を作るには verification を再実行してください。

### MCP を使う

explicit repository binding で server を起動します。

```bash
ai-cockpit mcp --repo /path/to/repository
```

`status`、`work_item_get`、`work_item_outcome`、`work_item_status`、`work_item_validate`、`work_item_list`、`blockers`、`safe_actions`、`knowledge_query`、
`evidence_get`、`delegated_evidence_list`、`repository_observe`、`capability_show`、`preflight`、`work_item_controls`、
`work_item_recover`、`verify`、`work_item_parallel` の 18 tools を提供します。
`tools/list` で JSON-RPC schema を確認できます。`preflight` は repository-relative `contract`、
`verify` は `command`、string array の `args`、optional `workItemId` を受け取ります。repository
binding のない call は fail closed です。result には `structuredContent`、text content、`isError`
が含まれ、CLI と同じ repository-bound verification policy を使います。
`work_item_get` は machine-oriented な record lookup です。`work_item_status` は read-only の request-scoped lifecycle projection で、
`{"all": true}` を渡すと stable な repository index を返します。`capability_show` は CLI と同じ Runtime-bound
registry を公開します。人間向けの結果が必要な場合、Agent は明示的な
`workItemId` で `work_item_outcome` を呼び、conversation の `language` を任意で渡します。text content は
CLI と同じ localized human handoff であり、`structuredContent.outcome` は安定した OutcomeV2 object です。
handoff には status marker、unknown、evidence、有効な structured human decision、次の action が含まれます。
MCP は Contract source text を翻訳せず、人間の decision も発明しません。
人間向け projection は validated OutcomeV2 の presentation layer であり、ガバナンス権限そのものではありません。

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

## Parallel Work Item の境界

並列実行は Contract が明示的に許可した場合だけ行います。`work-item boundary --repo <path> --id <id> --file <boundary.json>`
で任意の `concurrencyBoundary` を設定できます。そこには `implementationPaths`、`generatedEvidencePaths`、
`verificationOutputPaths`、`serializedProjectionPaths`、`reason`、`schemaVersion`、`maxWorkers` を含めます。
既存の intelligence sidecar は depends/conflicts/parallelizable を宣言する投影として残り、Contract の境界と
sidecar の両方を満たす必要があります。欠落、壊れた JSON、絶対/親パス、判定できない glob は保守的に serialize
され、slot の取得を fail closed にします。`maxWorkers` は repository-local slot の容量であり、単一の
`verify --workers` とは別の値です。

`work-item slot acquire|release|list --repo <path>` で `.ai/parallel/leases/` の lease を管理します。lease は
repositoryId、workItemId、slot、leaseId を持ち、自動 expiry はありません。repository 間で状態を共有せず、MCP の
`work_item_parallel` も `inspect`、`acquire`、`release`、`list` を同じ境界で提供します。
