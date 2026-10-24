---
author: AI Cockpit maintainers
title: "Contract と Summary の fields"
description: "Work Item Contract と Summary を current Rust Runtime に対応付ける field reference。"
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - contract_field_mapping
---

# Contract と Summary の fields

このページは reference source の Contract と Summary の概念を current Rust Runtime に
対応付けます。これは field mapping であり、別の schema でも、reference の全 field が
実装済みだという宣言でもありません。Runtime の repository protocol state は `.ai/` に
保存し、実行ファイルは governed repository の外に一つだけ install します。

Status の意味：

- **Implemented** — current Runtime の boundary が読み取り、書き込み、または検証し、repository-local の意味が定義されている。
- **Partial** — 表現または読み取りはできるが、reference の広い semantics は保証しない。
- **External** — Agent host、provider、organization、または別システムの責任であり、Runtime は evidence を bind/display するだけで生成しない。

## 理由付き Contract amendment

Active Contract は実装中に更新できる計画です。`ContractAmendmentRequest` は一意な
`changeId`、現在の `expectedContractDigest`、空でない変更理由 `reason`、順序付きの変更を
要求します。操作は `add`、`set`、`clear`、`remove`、`replace`、`reorder` です。collection
操作は対象 element を検証し、`reorder` は全要素の完全な permutation を要求します。batch は
atomic で、変更後の Contract 全体を再検証します。古い digest は conflict とし、暗黙に
rebase しません。同一 request の再送は元の receipt を返し、同じ ID を別 request に使うと拒否します。

field registry が編集を許可するのは human-owned な計画 field だけです。repository/Work Item
identity、base/snapshot facts、lifecycle、Runtime-observed facts、immutable receipt/evidence は
保護対象です。authority、risk、verification strength、gate の変更は sensitive で、既存の policy
と reauthorization を再実行します。理由は permission ではありません。確定履歴は old/new value、
digest、snapshot、無効化された check を bind して追記専用で保存します。`work-item amendments`
は読み取り専用で、amendment 後は preflight と影響する verification が新たに必要です。

## Work Item Contract（`*.contract.json`）

| Field | Rust Runtime mapping | Status |
| --- | --- | --- |
| `protocolVersion` | Repository Protocol version。現在は `1`。 | Implemented |
| `contractVersion` | typed Contract V2 の optional opt-in。historical protocol record は読み取り可能。 | Implemented |
| `repositoryId` | attach した repository から導く identity。isolation に必須。 | Implemented |
| `workItemId`、`mode`、`state`、`createdAt` | Work Item identity と lifecycle metadata。 | Implemented |
| `intent`、`goal` | human-owned purpose。`intent` は legacy text または structured `businessGoal`、`userGoal`、`problem`、`constraints`、`nonGoals`、`rationale` を取れる。 | Implemented |
| `scope`、`outOfScope` | repository-relative implementation boundary。不安全または曖昧な path は fail-closed。 | Implemented |
| `risk`、`authority` | preflight が使う declaration。repository record は人の identity を認証しない。 | Implemented / External identity boundary |
| `acceptanceCriteria` | human-owned acceptance。`A1:` のような番号付き criteria は Summary evidence に bind できる。 | Implemented |
| `requiredEvidenceClasses` | lifecycle completion に必要な evidence class。 | Implemented |
| `requiredRuntimeCapabilities` | Contract amendment が追加する Runtime 管理の minimum capability marker。generic amendment から保護され、未対応の旧 strict-schema Runtime は Contract を拒否します。 | Implemented |
| `sources` | legacy string または typed `{path, reason}` reference。 | Implemented |
| `verification` | legacy verification string または typed `{check, required}` declaration。fresh execution の代わりにはならない。 | Implemented |
| `baseRevision` | Work Item の開始 revision。snapshot から導く。 | Implemented |
| `projectProfileDigest`、`repositorySnapshotDigest` | project profile と repository snapshot の content binding。 | Implemented |
| `baseCommit`、`baselineDirtyPaths` | V2 lineage と開始時に観測した既存 dirty path の fingerprint。両方ある場合 `baseCommit` は `baseRevision` と一致する必要がある。 | Implemented |
| `archiveSequence`、`resumeHistory` | 正の archive 順序と、closed predecessor を連続して記録する lineage。 | Implemented |
| `synchronizationCheckpoint`、`synchronizationHistory` | 明示的に authorized された base synchronization と digest-bound rebase history。不完全な entry は fail-closed。 | Implemented |
| `guidelines`、`preReviewWarnings`、`acceptance` | human-authored guidance、review warning、optional alias。`acceptance` は `acceptanceCriteria` と一致する必要がある。 | Implemented |
| `authorityEvidence`、`restrictedWriteApproval`、`destructiveChangePolicy.approvalEvidence` | typed repository-local provenance と approval payload。V2 は malformed/unknown nested field を拒否し、legacy provider extension は読み取り可能。 | Implemented / External identity boundary |
| `problemStatement`、`riskAssessment`、`agentCapability`、`executionDecision` | strict typed な optional V2 safety/review input。non-continue decision は preflight を止める。 | Implemented |
| `destructiveChangePolicy`、`rollbackNote`、`unknowns`、`notCodable` | explicit safety、recovery、unresolved state declaration。 | Implemented |
| `scenarioCoverage` | optional high-risk scenario projection。required/unverified scenario は checkpoint 前に fail-closed。 | Implemented |
| `concurrencyBoundary` | parallel Work Item の Contract-owned path boundary と slot authorization。 | Implemented |
| `checkpointPolicy` | typed な `light`/`standard`/`strict`/`release` profile と明示的な required stages/checks。strict/release は Agent Risk の 6 gate（`aiWorkItem`、`aiScope`、`aiAgentRisk`、`aiSummary`、`aiStatus`、`aiStatusCheck`）も要求する。unknown field と重複要求は fail closed。Verification strength であり Evidence Assurance ではない。 | Implemented |
| `humanDecisionPoints`、`documentationImpact`、`performanceImpact`、`governanceProfile` など | current typed validator が behavior を定義する場合だけ意味を持つ additive value。generic field は approval ではない。 | Partial |
| `governanceProfile.materialInspectionReview` | `unknownCause=readable_committed_rust_syntax_unknown` となる、完全で読み取り可能かつ provenance が有効な commit 済み Rust 変更だけを対象にする、`self_declared` 技術 review の strict な任意 opt-in です。request と receipt は正確な Contract/profile、base、source snapshot、manifest、policy、analyzer、および各 path/hunk/blob に bind されます。decision は scanner Unknown を machine Clean と再ラベルせず、Finding やその他の Unknown も解消しません。Runtime は reviewer identity を認証しないため、人間、provider、release の approval ではありません。 | Implemented / External identity boundary |

`authority: authorized` は repository-local declaration です。enterprise identity、provider verification、organization policy、approval authenticity は外部 evidence であり、Contract bytes から推測しません。

## Change Summary（`*.summary.json`）

| Field | Rust Runtime mapping | Status |
| --- | --- | --- |
| `workItemId`、`repositoryId`、`mode`、`state` | Contract/repository binding と serial lifecycle state。 | Implemented |
| `changedPaths` | scope と archive check に使う snapshot-observed paths。 | Implemented |
| `checkpointCount` | current lifecycle の exactly-one checkpoint gate。 | Implemented |
| `checkpointEvidence` | repository、Work Item、snapshot、Contract に strict bind された `before_edit`/`before_finish` record。Contract amendment revalidation は append-only、verification 開始後の amendment は既存 required check を無効化し、resume history は stale record の再利用を拒否する。 | Implemented |
| `preflightState`、`preflightAt`、`preflightContractDigest`、`preflightDecisionDigest`、`preflightRepositorySnapshotDigest` | repository-bound preflight decision と freshness binding。 | Implemented |
| `scenarioCoverage` | Contract と照合する Summary scenario status、evidence、reason。 | Implemented |
| `acceptanceEvidence` | stable acceptance ID と explicit evidence、intent alignment の mapping。 | Implemented |
| `intentAlignment` | optional resolved/unresolved projection。欠落は unknown のまま。 | Implemented |
| `finalDimensions` | exact twenty dimensions の receipt。decision は `GO`、`CONDITIONAL_GO`、`NO_GO`。`fourPillarProjection` は表示用。 | Implemented |
| `verification` | Runtime execution receipt は `.ai/evidence/` に書き、path の存在だけでは満たさない。 | Implemented |
| `materialReviewReceipt`、`materialReviewReceiptHistory` | pointer は typed receipt を一つの正確な request に bind します。history は置き換えられた pointer 値を保持し、古い immutable sidecar の bytes は変わりません。raw scanner Unknown は表示上も保持されます。decision が解消できるのは許可された syntax Unknown だけです。reviewable Unknown が残っている場合だけ stale receipt が blocking になります。missing または invalid receipt はその Unknown を解消せず、Unknown が残っていない場合に stale blocker は追加されません。Finding と他の Unknown は引き続き blocking です。 | Implemented |
| `outcome`、archive manifest、human decision | `.ai/work-items/archive/` と `.ai/decisions/` の Runtime-generated terminal projection。 | Implemented |
| `reviewReadiness`、`residualRisks`、`knownGaps`、`followUps`、`documentationAlignment` | reference として有用だが、current Runtime の universal typed Summary contract ではない。 | Partial |
| provider、enterprise、hosted-CI、attestation、SBOM、organization approval | delegated evidence として import/link できるが、Runtime は provider authority を生成しない。 | External |

## Boundary

Runtime は Contract の source language を保持し、governance fact を machine-translate しません。
Outcome localization は label と presentation だけを変えます。missing、stale、contradictory、malformed、
identity-mismatched field は適用される gate に従って yellow または red のままで、documentation projection で green にはなりません。

[Reference source parity](reference-parity.ja.md) と [Commands](commands.ja.md) も参照してください。

## Contract review の境界

現在の Rust 境界は、ガバナンス評価の前に任意の `scenarioCoverage` リストの
形を検証します。各項目には `scenario`、boolean の `required`、対応する
status、evidence リストが必要です。`verified` には evidence、
`not_applicable` には reason が必要で、重複名や未知の nested field は
fail closed になります。これは構造検証だけです。シナリオを必須にするかは
risk policy が決め、Runtime がシナリオ、期待結果、verification plan を作る
ことはありません。

`acceptanceCriteria` は空でない人間の宣言でなければなりません。`A<n>:` の
番号付き criteria は Summary の evidence mapping を opt-in する形式として
残し、番号なしの criteria は legacy/原文の宣言として読み取り可能です。
`concurrencyBoundary` も schema、正の容量、空でない理由を検証してから
parallel slot を使います。これらの検証は verification tier を assurance
に変換せず、slot 宣言を権限決定にも変換しません。


## 完全な開始計画と明示的な委任開発レビュー

CLI `start --scenario-coverage <json-file>` は既存の型
`ScenarioCoverageEntry` の配列を受け取る。MCP `work_item_start.scenarioCoverage` も同じ型と
リポジトリ検証を使う。初期状態は unverified、evidence は空で、最初の preflight 前に保存する。
不正入力ではライフサイクル記録を作らない。省略時は従来の形を維持する。

CLI `start --development-review <json-file>` と MCP `developmentReview` は
`governanceProfile.developmentReview` の明示的な Work Item ポリシーを選ぶ。既定は strict のまま。
選択を含む正確な基準 Contract に対する正式なネイティブ所有者レビューが必要。
actor や出典文字列、任意の承認ファイルだけでは権限にならない。

```json
{"schemaVersion":1,"reviewerActor":"agent:reviewer","authoritySource":"user:explicit-bounded-delegation","allowedChanges":["append_scenarios","fill_verification_plan","append_verification"],"preapprovedCommands":["cargo test --locked"]}
```

typed amendment の任意 `developmentReview` は AuthorityEvidence を再利用する。
明示 agent／出典、`assurance=self_declared`、唯一の操作 `review_development_amendment` が必要。
許可は三種類のみ：必須・未検証・証拠なしで正確な宣言済み／事前承認コマンドを使うシナリオの追加、
既存の空の verificationPlan への同種コマンドの補完、有限集合の正確な事前承認コマンドの追加。
既存の順序と検査を保つ。scope、権限、リスク、ポリシー、受入基準、任意コマンド変更は従来の人間確認／拒否に従う。

Runtime は既存ロック内で別の `delegatedReview` を prepared／committed 履歴に保存し、
所有者基準と回执、ポリシー、repo／WI、要求、前後 Contract、ソース snapshot、Runtime を結び付ける。
欠落・改名・改変・別項目・不完全な記録は拒否する。元の所有者記録は不変で、agent レビューは
`decisionEvidence` にならず、close、Finding、失敗検証、強い人間／provider ポリシー、宿主／GitHub 承認を満たさない。

これはローカル出典であり外部本人認証ではない。全ローカル記録の書換えに対する保証もない。
共有 Runtime と本番設定は変更しない。本番利用には別途明示選択と新しい所有者レビューが必要で、agent の技術判断は誤り得る。
WI 間の期限／取消と外部 B は対象外。隔離テストでの確認回数削減は実際の利益測定ではない。


## 範囲を限定したタスク認可

完全なネイティブ `governanceProfile.taskAuthorization` を明示して選択します。従来モードと strict モードは変更しません。CLI `start --task-authorization <policy.json>` または MCP `work_item_start.taskAuthorization` で型付き宣言を保存します。新しい所有者判断が必要な場合は staged 起項（MCP `prepare:false`、CLI は `--prepare` を省略）でシナリオ・コマンド・入力参照を先に保存し、その後 preflight、実際のレビュー、checkpoint を行います。MCP の既存既定値 `prepare:true` は互換です。

ポリシーは `schemaVersion:1`、明示的な `agentActor`、`authoritySource`、有限の `allowedActions` とその部分集合 `closeoutActions`、任意の RFC3339 `expiresAt` を含みます。公開操作には `publicTarget` の `provider`、`repository`、`baseBranch` が必要です。ネイティブ所有者判断は repository・WI・目標・ファイル・操作・検証基準を束縛します。caller ファイルや actor 文字列は人間／外部の本人認証ではなく、ホストや provider の権限を与えません。

対応操作は型付き caller（`schemaVersion`、`actor`、`authoritySource`、`assurance:"self_declared"`）を CLI `--task-caller <caller.json>` または MCP `taskCaller` で渡します。Runtime は同じ WI ロック内で再確認し、検証の実プロセス起動とキャッシュ変化時の再実行にも適用します。caller のない呼出しは設定済み actor を継承しません。未対応の Task モード旧 closeout 登録は明示的に拒否します。資源対象は現在、所有者が認めた repository/base に一致する GitHub PR URL に限定し、新しい資源は archive 後に元の Contract を変更せず束縛します。

セッション再開では単一 WI の CLI `work-item status --json --authorization-request <request.json>` または MCP `work_item_status.authorizationRequest` を利用します。要求は `schemaVersion:1`、`caller`、`action`、正確な相対 `paths`、任意の `publicTarget` を含み、全 WI 問合せとは併用できません。応答は認可・証拠の新鮮度・現在の操作准入・`nextAction`（継続、再検証、現在の gate 解決、個別操作の入力・段階確認（`check_operation_boundary`）、host 境界確認、停止）を分離します。問合せ結果は再利用可能な許可証ではありません。範囲内の通常 commit は認可を維持しますが、証拠は再検証が必要になり得ます。範囲・権限・目標・受入基準やコマンド変更は既存の所有者／限定的な機械レビュー規則に従います。

認可は交付完了・撤回・期限の最初の事象で終了します。controls の `taskAuthorizationTermination` に `schemaVersion:1`、現在の `grantId`、`decision:"withdrawn"` または `"completed"`、明示 caller、reason を記録します。不変の記録は冪等で、権限を狭めるだけであり、認可復活や業務成功を作りません。各境界で期限を確認し、残り時間が不変の宣言済み予算より短い場合は子プロセスを開始しません。実行中の操作は安全な境界で観測し、盲目的にロールバックしません。

archive 後は宣言済み残余交付／資源操作だけが可能です。別途有効なネイティブの人間 close 後は、事前認可済み closeout 公開と補助資源整理だけを許し、ソース作業を再開できません。交付完了時は completed を記録し、撤回や期限で停止します。人間 close は別判断です。本 WI はこの機能で自己承認せず、共有 Runtime・外部 B・本番権限を有効化しません。
