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

歴史互換は明示的かつ低 assurance でなければなりません。旧 shared-primary worktree の
記録は `historical.kind=shared_worktree_retained` と
`assurance=historical_low` を使えますが、primary repository worktree に束縛し、明示的な
human close decision を必要とします。PR のないローカル merge は
`historical.kind=direct_merge_no_pr`、`pullRequest.number=0`、
`historical://direct-merge/<mergeCommit>` URL を使えます。ただし実際の merge commit、2 つの
parent、base revision、repository identity、authority を束縛する必要があります。Runtime は
Git と照合し、PR を捏造しません。Readiness は `historicalDebt` と recovery action を示し、
pending-close は引き続き fail-closed です。

新しい Work Item では current finalization head の disposition が `deleted` でなければ
なりません。`retained`、`blocked`、`unknown` の head は close decision を書く前に停止
します。唯一の狭い互換例外は、検証済みの歴史 `shared_worktree_retained` または
`direct_merge_no_pr` receipt です。これは primary worktree を保持できますが、
`assurance=historical_low`、明示的な human authority、repository に束縛された Git facts
が必要です。この例外は新しい Work Item には適用されず、歴史 evidence を provider
assurance に昇格させません。旧 Runtime が作った immutable record については、
`work-item finalize` が close 後に strict に bind された deleted transition を 1 件だけ
legacy reconciliation として append できます。この transition は closed root の path
と digest を束縛し、append-only cleanup observation として検証されます。close receipt は
書き換えません。

`work-item finalize` は最初の receipt を `.ai/decisions/<id>.finalize.json` に保存します。PR base は archived Contract の不変な `baseRevision` と一致する必要があり、record と `finalize-verify` は sequence 0 を含む mismatch を拒否して verified chain と報告しません。archive 前の rebase では active Contract binding を更新し、archive 後は receipt/archive を書き換えず recovery を行います。その不変 root が存在する場合、typed transition envelope は一意な head の predecessor digest と次の sequence を束縛し、Runtime は `.finalize.<digest>.json` を追記します。`finalize-verify` は `headPath`、`headDigest`、`sequence` を返し、`close` はそれらを束縛します。receipt commit が整合した全 head を進めた場合、sequence-1 merge observation は `governanceAppendRevision` も束縛できます。Runtime は ancestor range が追加のみであることを要求します。同一 Work Item の通常 finalization receipt 以外で許可される evidence 追加は、固定 schema の完全な pair `.ai/evidence/<id>/quality-route-post-finalize.json` と `.ai/evidence/<id>/repository-gates-post-finalize.json` だけです。各 path は `A`-only の `100644` regular blob で、archived Contract、PR revision、route digest、manifest、profile、passing gate の binding は一致しなければなりません。この pair は evidence であって authority ではなく、必須の finalization receipt 追加を置き換えません。任意の evidence path や archive の変更を許可するものではありません。

すべての repository command は明示的な `--repo <path>` を受け取ります。record や
decision を作る command は stdout の JSON を維持します。`finish`、`archive`、`close`
は既定で localize された人間向け handoff を stderr にも表示し、各 `--json` はその
handoff だけを抑止します。`work-item outcome` は既定で stdout に人間向け handoff を
表示し、機械処理には `--json` を指定します。failed/unknown は pass ではありません。

| Group | Commands | Boundary |
| --- | --- | --- |
| Read-only | `inspect`、`observe`、`status`、`compatibility`、`migrate plan`、`capability show`、`diagnose`、`doctor` | repository state/evidence を読み、黙って修復しない。 |
| Derived projection | `knowledge query` | 明示的な query のみで repository-local `.ai/knowledge/` index を materialize/reuse し、`projection.writeBoundary=repository-local-derived` を返す。governance authority は変更しない。 |
| Setup | `attach`、`profile confirm`、`profile propose` | protocol state の作成/更新、profile の確認、read-only candidate の出力。 |
| Migration | `migrate apply --approved` | review 済みの repository schema migration だけを適用し、Runtime-bound migration receipt を作る。 |
| Governance | `preflight` | Contract を読み green/yellow/red decision と `reviewState` を返す。不完全・不確実な Contract は human-review yellow となり checkpoint を越えられない。 |
| Work Item | `work-item new`、`start`、`status`、`checkpoint`、`finish`、`archive`、`close`、`validate`、`controls`、`recover`、`finalize-recovery` | request-scoped status projection を読み、または明示的な lifecycle record を作る。`close` と recovery には明示的な human decision が必要。 |
| Parallel Work Item | `work-item boundary`、`work-item declare`、`work-item slot acquire|release|list` | Contract の並列境界を bind し、repository-local slot を管理する。不明な場合は serialize する。 |
| Verification | `verify` | bounded command を実行し evidence を記録する。Work Item に bind できる。 |
| External evidence | `evidence import`、`evidence list`、`evidence policy`、`evidence purge-plan` | exact provider bytes の bind、bounded persistence policy の宣言、または決定論的な非破壊 disposal plan の生成。 |
| Audit | `audit export` | repository-bound な安定 event bundle を外部 retention owner へ handoff する。local immutability は主張しない。 |
| Adapter | `agent list/install/doctor/repair/detach`、`mcp` | 明示的に選択した repository-local Agent adapter を管理し、または stdio で JSON-RPC を提供する。すべて `--repo` に bind する。 |

## Important options

- `verify --command <program> --args <comma-separated>` は explicit command を常に fresh に実行します。
  `--work-item <id>` は receipt を記録しますが、検出された Cargo/npm command は dynamic な
  profile-authorized path を使い、explicit custom command は常に fresh です。
- `--command` なしの `verify` は Cargo または npm を検出し、confirmed profile で cross-process reuse できます。
  現在の repository、snapshot、profile、Runtime、command、scope、stage、runner、base、toolchain、dependency、policy
  identity がすべて exact match の場合だけ reuse を許可します。それ以外は宣言された command を実行し、拒否/昇格理由を返します。
  timing や cache state が required/protected node を省略することはありません。
- `verify --workers <n>` は positive worker count を要求し concurrency を制限します。
- `work-item boundary --repo <path> --id <id> --file <boundary.json>` は optional な
  `concurrencyBoundary` を Contract に bind します。4 種類の path と `maxWorkers` を検証しますが、
  `maxWorkers` は slot 容量であり `verify --workers` とは別です。
- `work-item slot acquire|release|list` は `.ai/parallel/leases/` の exclusive lease を管理します。
  lease は repository と Work Item に bind され、欠落・壊れた boundary、曖昧な path、stale state は fail closed
  になります。global な current Work Item は作りません。
- `start` は `--id`、`--intent`、`--goal` が必須です。green governed flow には `--authority authorized` が必要です。
- `start` または `work-item new` の前に Runtime は repository-scoped entry gate を評価します。`.ai` 以外の作業ツリー変更、detached HEAD、検出された remote default ref と現在の HEAD の不一致、または有効な close decision のない archived Work Item があれば fail closed になります。gate は archived bytes を書き換えません。`work-item recover` の successor は明示的な同じ recovery chain の継続であり、独立した次の Work Item ではありません。
- 同じ entry gate は、通常の Work Item が repository の primary worktree または既知の default branch を使うことも拒否します。feature branch の専用 linked worktree を使用してください。明確な remote default base のない linked worktree は ready とせず拒否します。linked worktree がない local calibration repository は、base が検出可能になるまで `status: unknown` のままです。
- `work-item new --repo <path> --id <id> --mode <mode>` は `not_ready` skeleton を作ります。snapshot-derived facts だけを埋め、
  human field は空または `unknown` のままです。移行期の `start` も同じ writer を使います。repository-local の
  exclusive reservation により重複競合は fail closed になり、同じ ID では 1 件だけが成功し、異なる repository は独立して動作します。
- `start` が skeleton を activate するとき、既知の repository fact から Cargo の既定 verification command を選びます。
  `Cargo.lock` がある repository は `cargo test --locked --workspace`、lockfile がない Cargo repository は
  `cargo test --workspace` を使います。非 Cargo repository には Cargo command を推測せず、owner-approved check の宣言が必要です。
  この選択は deterministic であり、人間が所有する intent や acceptance の代替ではありません。
- `work-item outcome --repo <path> --id <id>` は完了内容、問題、停止、リスク、不明点、判断、検証、影響、次の action の順で人間向け結果を表示します。
  automation には `--json` を使います。status marker と言語規則は[人間向け Outcome](outcome-report.ja.md)を参照してください。
  Work Item の完了時には型付きの `*.task-report.json`、人間向けの `*.task-report.md`、append-only の `*.events.jsonl` も bind されます。
  これらは evidence-bound projection であり、追加の authority でも Contract/verification receipt の代替でもありません。
- `work-item finalize-recovery --repo <path> --id <id> --input <receipt.json>` は immutable な旧
  finalization receipt に対する append-only の Runtime-bound 歴史分類を記録します。入力には正確な
  predecessor digest、repository/Work Item/Contract base、current Runtime、actor、authority、reason、
  timestamp の binding が必要です。旧 primary worktree は `historicalKind=shared_worktree_retained`、
  PR のない merge は完全な `historicalKind=direct_merge_no_pr` finalization receipt を使います。
  predecessor は書き換えられず、recovery record だけで Work Item が green になることもありません。
- `finish`、`archive`、`close` は stdout の lifecycle JSON を変更せず、既定では同じ
  検証済み Human Outcome を stderr に render します。機械専用出力には `--json` を
  指定します。`finish` が block された場合、CLI は永続化済みの赤または黄の Outcome
  を先に表示し、元の nonzero error を返します。failed gate を成功に変換しません。
  CLI は埋め込み先の Agent/UI に会話 panel の表示や展開を強制できません。host は
  stderr handoff を表示するか、`work-item outcome` で決定的に再生する必要があります。
- `work-item status --repo <path> --id <id>` は read-only で lifecycle、governance、activity health、fact count、blocker、unknown、evidence、source digest を返します。scheduler を動かさず、割合を発明しません。
- `work-item inspect --repo <path> --id <id>` は compatibility、implementation approach、parallel slot の read-only projection です。
  approach はメモリ上で計算され、`.ai/work-items/active/<id>.approach.json` は作成・更新されません。
  repository-local の approach artifact が必要な場合は、明示的な write boundary である
  `work-item approach` を使用します。
- 有効な close decision がない archived Work Item は lifecycle blocker であり、完了ではありません。`safeActions` は残りの handoff を明示します。resource-bound item では `finalize_resources` または `cleanup_resources`、`record_finalization`、`finalize_verify`、続いて `close_after_cleanup`（Deleted receipt の検証済みなら `close`）が必要です。外部 resource がない item では `close_after_review` が必要です。Agent はこれらの action に従い、predecessor が close または明示的 recovery されるまで次の Work Item を開始してはいけません。
- top-level `status` には deterministic な `readiness` object も含まれます。名前付きで clean な branch が唯一検出された remote default revision と一致し、active Work Item がなく、close 待ちの archived Work Item もない場合だけ `readyOnBase: true` になります。remote metadata が欠落または曖昧な場合は `state: unknown` であり、green にはなりません。`blocked` は entry blocker を、`unclosedArchivedWorkItems` は close または明示的 recovery が必要な記録を示します。
- `work-item status --repo <path> --all --json` は active/archived Work Item を stable な ID 順で集約し、
  固定の green/yellow/red/unknown count、member diagnostic/digest、current repository snapshot digest、
  deterministic な index digest を返します。malformed または foreign な member は explicit unknown entry
  となり、他の member は可視のままです。この dynamic counterpart は
  `.ai/cockpit/work-items/index.json` や item ごとの status file を書きません。MCP では
  `work_item_status` に `{"all": true}` を渡します。
- `capability show --repo <path>` は Runtime identity と repository に bind した registry を返します。
  observed technical capability、profile confirmation、repository binding、adopter acceptance、external ownership
  は別 state です。file の存在だけでは `adopter_accepted` を証明せず、missing、malformed、stale、foreign
  input は unknown のままです。MCP では `capability_show` を使います。
- `observe`、`capability show`、top-level `status`、single/all Work Item status を繰り返しても、tracked
  repository bytes や observer cache は書きません。
- `work-item validate --repo <path> --id <id> [--json]` は Contract/Summary の scenario coverage、stable acceptance evidence、intent alignment、任意の final-dimensions receipt を read-only で検証します。
  `work-item controls --repo <path> --id <id> --input <json>` は明示された projection field（identity-bound な `decisionEvidence` review receipt を含む）だけを記録し、lifecycle state、Contract fact、verification receipt は変更しません。
- `work-item recover --repo <path> --id <id> --input <receipt.json>` は identity-bound な `retry`、`successor`、または `supersede` decision を記録します。`supersede` には bind 済みの successor Work Item が必要で、predecessor を明示的な履歴 `superseded` 状態へ archive します。元の bytes は書き換えません。receipt は predecessor の Contract、Summary、Outcome、存在する場合は event digest と current Runtime identity に bind されなければなりません。既存 receipt は上書きせず、後続 decision は digest suffix ファイルに append されます。recovery receipt だけで verification を green にしたり predecessor を書き換えたりすることはありません。superseded predecessor は現在の成功・失敗ではなく、後続処理は successor が担います。Outcome/archive consumer は current candidate ごとに regular-file/filename 境界、repository/current Runtime identity、predecessor digest、timestamp、decision shape、successor Contract binding を再検証します。invalid または ambiguous な candidate は `recovery_decision_invalid` として fail closed になり、historical archive bytes と projection は immutable のままです。新しい archive の Contract/Summary/Outcome/Events と predecessor digest が一致しなくなった retry は消費済みの履歴として扱い、static gate は recovered の終端状態を作らず実際の finalization path を投影します。一致する blocked retry は引き続き fail closed の recovery です。
- `profile propose --repo <path>` は read-only の `candidate`/`proposed` amendment を出力し、profile baseline を適用しません。
- `agent list --repo <path>` は read-only です。`agent install` だけが通常の adapter write entry point で、
  `--provider` が必要です（`auto` は安全な surface が 1 つだけの場合に限り、`AGENTS.md` では Codex を選びます）。`agent doctor --repo <path> --json`
  は strict state report を返し、0（verified）、1（degraded）、2（configuration error）、3（human intervention）の exit code を使います。
  managed section または ownership record が変更されていれば `repair` と `detach` は fail closed し、global Agent/MCP config は変更しません。
- `preflight --contract` は通常 `start` が作る `.ai/work-items/active/<id>.contract.json` を指します。
- `work-item new` は `not_ready` の skeleton を作ります。これを `preflight` すると意図的に
  `yellow` と `reviewState: needs_human_confirmation` になり、人の項目を埋めてから再度 preflight して checkpoint します。
- `close --human-decision approved|confirmed|rejected` は human decision record であり verification evidence ではありません。
  `approved` と明示的な `confirmed` は正の terminal choice ですが、`rejected` は Work Item を Implemented に昇格させません。
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
- Task Outcome report は strict typed JSON projection です。各 claim は可能な場合 evidence reference を持ち、明示的な inference は verified fact ではありません。
  event stream は Work Item finish ごとに append-only で、repository/Work Item identity、順序、安全な detail、evidence reference の境界を検証します。
  archive manifest は event stream と report JSON/Markdown digest を bind し、close receipt は final report と digest を含みます。
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

## Close 後の documentation promotion

structured `close` の後に実行します。

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item <id>
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
```

最初の command は exact regular archive Contract と raw digest、passing verification
receipt、unique linear finalization chain、sequence-2 `deleted` head、merged provider
identity、approved close bindings を検証してから controlled documentation fields を
更新します。write target は `status`、`lastVerifiedBy`、4 個の `terminal*`
frontmatter fields、exact tri-language parity rows だけです。2 番目は mandatory
quality/terminal-CI form であり、write しません。missing、foreign、ambiguous、
malformed、symlinked、mismatched、stale input は fail closed です。これらの repository
helper commands は Runtime Core が documentation を自動編集することを意味しません。

## Contract/Summary control validation

repository library は Agent/MCP adapter 向けに
`validate_work_item_governance_controls` を提供します。scenario coverage、
acceptance evidence、intent alignment、任意の final-dimensions receipt を
一つの stable report として read-only で検証します。欠落項目を埋めず、
`blocked` または `unknown` として返します。final receipt は参照源と同じ
20 dimensions を使用し、`fourPillarProjection` は明示された任意の表示用
projection です。`4D` は protocol field ではありません。
adapter が current Runtime context を渡す場合、validator は `runtimeVersion` と
`runtimeDigest` の一致も要求します。standalone value helper は non-empty かつ
versioned digest の shape だけを保証します。

## Runtime identity

`inspect`、`doctor`、MCP `initialize`、verification evidence は runtime version、runtime digest、protocol version を示します。
`ai-cockpit --version` は短い executable version だけを出します。

## Release acceptance の境界

`tests/release/adopter_acceptance.sh` は maintainer 向けの post-release harness であり、Runtime command ではありません。
public Release binary を download して pin し、isolated directory で adopter lifecycle を実行し、`acceptance.json` と `SHA256SUMS` を生成します。
workspace build や local target binary で代用してはならず、acceptance failure が公開済み Release truth を変更することもありません。

lifecycle は省略できません。verification 前に `finalize-plan` を実行し、通常の Work Item は archive 後に
`finalize` と `finalize-verify` を通過して head が `deleted` であることを確認してから structured `close`
を行います。歴史 shared-worktree または direct-merge receipt は Git facts の検証後に、文書化された
低 assurance の retained 例外を使えます。retained が新しい Work Item の close を認可することはありません。

acceptance receipt には各 isolated root の typed before/after manifest も記録されます。`HOME` と `XDG_CONFIG_HOME` の
`allowedPrefixes` は空で、変更されてはいけません。Runtime が書き込めるのは `TMPDIR` と `CARGO_HOME` だけで、allowlist は
`<TMPDIR>/**` と `<CARGO_HOME>/**` に限定されます。cleanup の結果は `cleanup.json` と `cleanupState`/`cleanupError` に記録され、
cleanup failure は acceptance を失敗させますが、公開済み Release truth を unpublish または書き換えません。

`tests/conformance/final_replacement_acceptance.sh` は source repository の最終置換 boundary です。installed Runtime identity、固定した
reference oracle、conformance/adversarial/performance gate、コピーなし検査を記録し、`acceptance.json` と `SHA256SUMS` を生成します。
