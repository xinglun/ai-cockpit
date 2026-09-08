---
author: AI Cockpit maintainers
title: アーキテクチャ責任・依存関係マップ（2026-09）
description: 観察、ガバナンス、ライフサイクル、証拠、実行、投影の現在の責任境界と、P1-B から P3 への調査境界をソース引用付きで記録する。
audience:
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human:repository-owner
workItemId: WI-691-p0-responsibility-map
lastVerifiedBy: WI-691-p0-responsibility-map
---

# アーキテクチャ責任・依存関係マップ（2026-09）

これは 2026-09 アーキテクチャ最適化の P0 事実マップである。目標実装ではなく、現在の
責任と呼び出し境界を記録する。North Star は **Calibrated Human-Agent Trust** のままで
ある。本 Work Item は文書だけを変更し、Rust の挙動、wire format、ガバナンス規則、
`.ai/` 記録プロトコルは変更しない。

## 入口と現在の呼び出し経路

CLI は `crates/cockpit-cli/src/main.rs:619-831` でコマンドと Runtime adapter を選び、
Work Item 操作と human Outcome 表示を `1208-1380` で再度 dispatch する。MCP は
`crates/cockpit-mcp/src/lib.rs:7-21` でツールを定義し、`574-674` で検証・dispatch し、
`1005-1055` で同じ Outcome input と renderer を使う。adapter はガバナンス事実の権威
ではない。

```text
CLI / MCP
  -> cockpit-repository operation
     -> cockpit-git RepositorySnapshot と repository-local record
     -> typed cockpit-protocol fact と governance validator
     -> command が必要なら cockpit-verification の計画/実行
     -> lifecycle/evidence receipt と status/Outcome projection
  -> CLI / MCP JSON または human rendering
```

`RuntimeContext` と `RepositoryContext` は `crates/cockpit-protocol/src/lib.rs:120-131`。
repository crate は `crates/cockpit-repository/src/lib.rs:48-92` で
execution_context、evidence_store、lifecycle、outcome_render、project_governance、
status_projection を公開している。これは crate 内の module 境界であり、全責任が純化済み
という意味ではない。

## 責任表

| 責任 | 権威ある事実 | 許される I/O | 検証・判断・表示の所有者 |
|---|---|---|---|
| CLI/MCP 入口 | parsed argument と Runtime identity；CLI `619-831`、MCP `574-674` | adapter の入出力だけ。独自の repository authority は持たない | repository operation が判断し、adapter が serialize/language を選ぶ |
| Repository observation | `RepositorySnapshot`：`cockpit-git/lib.rs:212-231,268-300`；identity/digest：`repository/lib.rs:536-552,2443-2663` | observation boundary の Git subprocess と repository file read | `observe`/`observe_cached`：`repository/lib.rs:12084-12231`；consumer が freshness を検証 |
| Runtime/repository identity | `RuntimeContext`、`RepositoryContext`、`.ai/cockpit.toml`、`.ai/project.json` | status projection と attach の repository read | protocol type と identity check。human authorization は推論しない |
| Contract、policy、project declaration | `Contract`、`GovernancePolicyDocument`、`ProjectGovernanceProjection`：`protocol/lib.rs:2603-2645,609-627,322-335` | `project_governance.rs:53-127,241-317` が declaration を読む。policy 解決は `repository/lib.rs:2742-2990` | strict parse、identity/snapshot binding、unknown は project_governance |
| Governance validation/decision | Contract/Summary evidence、policy、snapshot、Runtime identity：`repository/lib.rs:3359-3555,4395-4450` | decision helper が repository record を読む。`governance_controls.rs:1038-1184` が projection を検証 | `required_verification_checks` 等は pure (`governance_controls.rs:33-72`)。preflight/decision entry が receipt を記録 |
| Lifecycle coordination | Contract、Summary、checkpoint/verification/finalization/close record | `lifecycle.rs:324-467,469-560,883-905,1087-1165`；archive/close は `repository/lib.rs:4504-4752,7353-7817` | 順序と gate は lifecycle/repository。storage は authorization を与えない |
| Evidence storage/history | reusable receipt、repository/profile/node binding、delegated evidence/validity | nofollow read/write：`evidence_store.rs:36-39,225-280`；protocol type：`protocol/lib.rs:927-960` | receipt validation は evidence/protocol。receipt は governance decision ではない |
| Physical execution/scheduling | verification graph/plan、`PhysicalExecution`、`ExecutionResult`、Work Item receipt | process、worker、resource budget、single-flight：`cockpit-verification/lib.rs:1206-1441,1468-1525,1595-1833` | execution は成功/失敗だけを返し、repository が適用性と authorization を別検証 |
| Status/Outcome projection | `OutcomeState`、`TaskOutcomeReport`、`WorkItemStatusSnapshot`、history/freshness：`protocol/lib.rs:3236-3505` | config/profile、1つの Git snapshot、record：`status_projection.rs:3-90` | status_projection が machine status、outcome_v2 が Outcome を組み立てる。projection は権限を与えない |
| Human Outcome rendering | 検証済み `OutcomeRenderInput` と language | `outcome_render.rs:70-76` の renderer は repository path を受けず input だけを format | render が表示境界。ただし input assembly は同じ module に残る |
| Persistence/recovery | atomic JSON、lifecycle lock、archive manifest、finalization/close record | `repository/lib.rs:12033-12081`；finalization `5246-7140`；recovery `status_projection.rs:464-585` | authoritative record と recovery check を明示し、projection は再構成可能な view とする |

## 現在の混在・重複・依存方向

1. `project_governance_projection`、`project_governance_unknowns`、
   `project_success_criteria` はそれぞれ snapshot digest と repository ID を計算する
   (`project_governance.rs:288-317,320-383,385-401`)。これは P1-B の request-scoped
   observation context の候補であり、atomic snapshot の証明ではない。
2. `create_work_item_scaffold` は Git discovery、snapshot、profile read、scaffold facts
   の導出、Contract/Summary write を一つの lifecycle operation に含む
   (`lifecycle.rs:324-467`)。`preflight_work_item_internal` も Contract/snapshot の
   read と decision の評価・保存を行う (`lifecycle.rs:883-905` 以降)。pure governance
   function ではない。
3. `governance_controls` は主に validator だが、`record_work_item_governance_controls`
   は設計上 Summary を write する (`governance_controls.rs:1186-1250`)。read-only check
   と write boundary を混同してはならない。
4. `render_human_outcome` は pure だが、`outcome_render_input_from_outcome` は root を
   `build_outcome_render_input` に渡し、archive と close decision を読ませる
   (`outcome_render.rs:14-76`)。同 module は lifecycle Summary と human decision も読む
   (`666-705,816-875`)。従って P1-A は renderer pure 化までで、assembly の filesystem
   分離は未完了である。
5. repository submodule は `super::*` で root の `ObserverError`、`repository_id`、
   `snapshot_digest` を使う。現在は crate 内の一方向依存であり、この P0 から新 crate や
   circular Cargo dependency を導入する理由はない。先に共有 helper の依存を狭めるべきである。

status path は status projection 全体で Git snapshot を一度だけ取得し、readiness の
二重 snapshot を避けている (`status_projection.rs:50-90`)。これは再利用すべき仕組みだが、
execution や persistence の境界を越えて snapshot の有効期間を広げる理由にはならない。

## 再利用できる既存機構

- `RepositorySnapshot`/`GitRepository::snapshot`：`cockpit-git/lib.rs:212-231,268-300`。
- 最小の Runtime binding である `RuntimeContext`：`cockpit-protocol/src/lib.rs:120-124`。
- typed input を受け、command execution をしない `required_verification_checks` と
  `validate_checkpoint_evidence_bindings`：`governance_controls.rs:33-72`。
- governance から分離済みの capability-scoped nofollow receipt store：
  `evidence_store.rs:36-39,225-280`。
- lifecycle lock、single-file atomic replacement、pending-index detection：
  `repository/lib.rs:12041-12081`、`evidence_store.rs:71-100`。
- 共通 CLI/MCP renderer の `OutcomeRenderInput` は「一度 assembly、複数 display」の方向に
  適しているが、現在の assembly は外へ移す余地がある：`outcome_render.rs:14-76`。
- physical execution は独自 identity/result digest を持ち、Work Item receipt を別に bind する：
  `cockpit-verification/lib.rs:1261-1441`。

## 後続 Work Item の限定された調査

### P1-B — 明示的な observation context

問題は複数の入口が snapshot を自分で取得し、identity/digest も再計算するため、同一
request が異なる観察時点の facts を混ぜ得ること。目標は edit 前、execution 後、
persistence 前など実際の phase ごとに context を作り下流へ渡すこと。実際の変更境界を
またいだ共有や global current-repository cache は行わない。リスクは concurrent change、
unknown 結果、digest semantics を隠すこと。検証は request 内 read count と、観察/実行中の
file/config/repository identity 変更に対する stop/retry/unknown を含む。

### P2-A — lifecycle、evidence、execution、projection の所有権

module は存在するが、scaffold/preflight/archive/close が read、governance check、write を
完全な use case の中でまだ混ぜている。Observation は facts、Governance は decision、
Lifecycle は順序、Evidence は保存、Execution は command、Projection は表示を担当する
よう、一つの use case ごとに移行する。Port は実際の substitution/fault injection 境界に
限る。API、`.ai/` layout、error、history read を維持し、pure governance の I/O 無しを
dependency test と既存 integration test で確認する。

### P2-B — state type と合法な遷移

protocol には `OutcomeState`、verification stage、evidence validity、finalization state、
`HumanDecision` が既にある (`protocol/lib.rs:368-415,559-568,927-960,962-1082,3236-3505`)。
一方 lifecycle、evidence applicability、governance、human assurance、history は strings/
optional fields で接続される。既存 enum を優先し、missing/invalid/expired/revoked/
not-applicable が本当に潰れている箇所だけ narrow type を追加する。JSON 互換と旧 evidence
の read-only replay を守り、合法組合せ/不正遷移を table-test する。

### P2-C — multi-file consistency、concurrency、recovery

既存の lifecycle lock、atomic single-file replacement、pending-index detection、archive
manifest、finalization receipt、close decision check は確認できるが、P0 はそれだけで
transaction だとは判断しない。finish/archive/close/recovery ごとに commit record、write
order、operation identity、conflict boundary、recovery entry を特定し、authoritative record
だけから projection を再構築する。各 write 後の中断、write/space failure、repeat、二つの
process、projection の欠落/破損を fault injection し、未完了を完了表示しないことを確認する。

### P3 — physical execution と governance binding

`PhysicalSingleFlightCoordinator`、physical execution identity/result、Work Item receipt は
`cockpit-verification` (`1206-1441,1468-1525`)、governance decision/policy gate は
`cockpit-repository` (`repository/lib.rs:2934-3131,3359-3555`) にある。ただし reuse
eligibility は `execution_context.rs:14-230` にあり、owner は分かれている。cache hit、
execution success、Work Item evidence binding、governance permission を別々の facts と
して維持する。single-flight の拡張は concurrency/repository isolation を変えるため、
pure refactor に混ぜない。同一/異なる key、failure propagation、cancel、resource peak、
execution count、receipt binding、独立した governance decision を検証する。

## P0 結論と未知点

typed protocol、Git snapshot、制限された evidence store、lifecycle lock、bounded execution、
filesystem-free final renderer という有用な境界は既にある。未解決なのは end-to-end の
observation context owner、各 multi-file lifecycle operation の commit record、verification
reuse と physical execution の production boundary である。これは P1-B、P2-A/P2-C、P3 の
調査入力であり、P0 は実装を行わない。

