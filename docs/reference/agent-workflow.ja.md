---
author: AI Cockpit maintainers
title: "Agent ワークフローとレビュー境界"
description: "今後の AI Cockpit Work Item が継承する repository-local の運用規則。"
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - agent_workflow_boundaries
---

# Agent ワークフローとレビュー境界

これは参照元の運用規則を本プロジェクトに適用した投影です。ガバナンス上の
意図を保ちつつ、インストール済み Rust Runtime と本 repository の Protocol
語彙を使用します。

## 継承する規則

- repository が検出した remote の default branch の最新 commit から開始し、
  remote、default branch、base revision を Work Item Contract に記録します。
- Work Item ごとに一つの Contract、専用 branch/worktree、一つの PR を使います。
  scope、evidence ownership、repository context、serialized projection が分離し、
  Runtime が compatible と判定した独立 Work Item だけを並行できます。
- 変更前に `.ai/README.md` と `.ai/glossary.md` を読み、`inspect`、`status`、
  `doctor` を実行します。宣言した scope 外を変更せず、test と evidence を保持し、
  Summary を更新し、Contract の project checks を実行します。
- `preflight` が `not_ready` または `needs_human_confirmation` を返した場合は停止し、
  Preflight Review を人に提示します。advisory の成功終了は実装の許可を意味しません。
- scaffold の intent、goal、scope、out-of-scope、acceptance、authority のいずれかが空なら、
  Runtime は `yellow` と `reviewState: needs_human_confirmation` を返し、ready として扱いません。
  `verification_pending` の yellow は Contract が宣言した evidence の収集だけに使え、
  `needs_human_confirmation` は checkpoint を越えられません。
- 事前 Contract review は repository、Work Item、Contract digest、snapshot digest を bind します。
  いずれかが変わった場合は checkpoint 前に preflight をやり直します。
- `reviewState` が `needs_human_confirmation` の場合、preflight は what happened、why it matters、
  options、recommendation、question、resume condition を含む構造化 `humanDecisionRequest` も返します。
  これは人向けの request であり approval ではありません。Agent は request を権限として扱わず、
  Contract を amend して preflight を再実行します。
- 人は repository-local の `decisionEvidence` projection だけを使って、この限定された review を記録できます。
  strict receipt は `decisionId`、Work Item、repository、Contract digest、preflight decision digest、
  snapshot digest、actor、timestamp、reason を bind します。有効な receipt は checkpoint transition
  だけを許可し、test、scenario、verification、release の完了を証明しません。欠落、stale、foreign、
  malformed、symlink の receipt は停止したままです。
- review receipt は append-only です。Contract または repository snapshot が変わった場合、新しい receipt は digest suffix の decision path に保存され、以前の receipt は historical evidence として残り上書きされません。`work-item recover` は predecessor の Contract/Summary/Outcome/event digest と current Runtime に bind した strict な `retry`、`successor`、または `supersede` decision を記録します。`supersede` には bind 済み successor が必要で、predecessor を明示的な履歴終端状態として archive し、元の bytes を保持します。これは verification を自動で green にせず、predecessor を書き換えません。superseded は現在の成功・失敗ではなく、後続処理は successor が担います。
- 実装後にしか実行できない high-risk の必須 scenario は、Contract の `scenarioCoverage` で
  `unverified` のままにできますが、空でない `expected`（または `expectedResult`）と具体的な
  `verificationPlan` の両方が必要です。これは実装計画の evidence であり完了 evidence ではありません。
  Summary scenario guard と `finish` は引き続き実行済み evidence を要求します。
- 人間向け Outcome は独立して提示し、`Outcome: 🟢`、`Outcome: 🟡`、または
  `Outcome: 🔴` で始め、unknown、evidence、human decision、次の action を含めます。
  欠落、折りたたみ表示のみ、stale、contradictory、malformed の Outcome は
  fail closed とし、進行を許可しません。
  top-level の `finish`、`archive`、`close` は stdout JSON を維持し、既定では
  この handoff を stderr に出力します。`--json` は機械専用 mode です。block された
  `finish` は永続化済みの赤/黄 Outcome を出力した後も元の nonzero failure を返します。
  CLI は host の会話 panel を強制展開できないため、host は stderr を提示するか
  `work-item outcome` を再生する必要があります。
- 問題が現在の Work Item の範囲内なら、その Contract を amend/revalidate して
  現在の Work Item で修正します。scope、authority、base が本当に異なる場合、
  独立変更の場合、安全な in-scope 修正が不可能な場合、失敗した delivery の再実施、
  または人の明示的な指示がある場合だけ successor を作成します。
- install と upgrade の acceptance は immutable な公開 Release tag と download
  binary を使います。merge 後の closure では archive evidence、decision、merged PR
  head、同期済み default branch、clean worktree、正確な branch 削除を検証します。
  archive evidence は immutable な archive manifest に対して検証し、merge による
  current repository snapshot の変更だけを理由に stale と再分類しません。どれかが
  失敗したら recovery のため未完了のままにします。

## 本プロジェクトでの適用

参照元にある `make ai-*` コマンドと `contractVersion: 2` template Protocol は、
本プロジェクトの command や schema 要件ではありません。本 Rust project は
インストール済み shared Runtime と次の明示的な lifecycle を使用します。

```text
start → preflight → checkpoint → verify → finish → archive → close
```

すべての repository-bound command に `--repo` を付けます。Runtime に global な
current repository、Work Item、project profile はありません。Contract の criteria
は原文を保持し、人間向け presentation 層だけを localize します。

## Resource finalization の境界

Finalization evidence は append-only chain です。canonical `<id>.finalize.json` は不変の chain root であり、後続の provider observation は predecessor digest と sequence を束縛した `<id>.finalize.<digest>.json` に保存されます。`finalize-verify` と `close` は一意な線形 head を要求し、stale predecessor、fork、malformed record、symlink、identity drift は fail closed になります。pre-merge blocked root は連続する merge observation（`retained`）と cleanup（`deleted`）transition で進みます。canonical governance receipt の commit により PR head が進む場合、最初の unmerged-to-merged observation だけが `governanceAppendRevision` を宣言できます。PR、branch、worktree の各 head は同時に変わり、Git は旧 head が新 head の ancestor であることを証明します。この append 区間には、同一 Work Item の通常 finalization receipt と、Runtime が生成した完全な post-finalize evidence bundle だけを追加できます。bundle の path は `.ai/evidence/<id>/quality-route-post-finalize.json` と `.ai/evidence/<id>/repository-gates-post-finalize.json` に限定されます。受理される各 path は Git の `A`-only change で、tree entry は `100644` regular blob でなければなりません。両 evidence file は固定 schema に従い、archived Contract、PR base、bounded head、route receipt digest、manifest digest、selected profile、および passed required gates を束縛しなければなりません。これらは束縛済み observation であって、それ自体が authority ではなく、区間には引き続き finalization receipt の追加が必要です。bundle の欠落、別 Work Item または filename、malformed/duplicate-key JSON、binding mismatch、削除、変更、rename、symlink、無関係な変更、非 merge または後続の head drift は拒否されます。archive bytes は書き換えません。cleanup は受理済み head を保持します。

## Pending parity 登録

`docs/reference/pending-parity-registry.json` は、同じ scope の PR に三言語 parity row を
安全に追加できない archived code Work Item のための厳密に型付けされた一時 bridge です。
parity evidence ではなく、Implemented を意味しません。各 entry は repository、完全な
Work Item ID、GitHub PR、Contract base、canonical finalization head、正確な
archive/evidence/finalize path、3 つの正確な `In progress` row、RFC 3339 created time を
束縛します。`headRevision` は canonical receipt の PR、branch、worktree head と一致し、
`registryBaseRevision` は registry だけを変更する 1 commit の直接の親を別に束縛します。

通常の archive、verification、finalization 検証が常に先に実行されます。正確な feature
branch または pull-request entry だけが 3 つの `missing_parity_entry` を
`pending_parity_registration` に置き換えられます。unknown/duplicate field、foreign
identity、unsafe/symlink path、missing/mismatched record、別 ancestor、registry 以外の
append、partial parity、malformed JSON は fail closed です。default branch、merge 後、
またはいずれかの parity row が存在すると entry は `stale_pending_parity_registration`
になります。後続 change は 3 言語 row を原子的に追加して entry を削除し、predecessor
の `.ai` record を書き換えません。

parity を変更する Work Item は別の self-contained route を使用します。Contract の
scope/acceptance または active Summary の changed paths が
`docs/reference/reference-parity*` か parity registration を所有すると明示した場合、light
governance gate は verification 前に 3 つの lifecycle-bound row を要求します。standard と
strict は同じ static check を継承します。通常の code Work Item は `active_non_parity` と分類
され、documentation scope を強制されません。各 row は将来の archived Contract、verification、
canonical finalize、close path を列挙し、条件付き status
`In progress → verified close 後 Implemented` を使います。Git は row commit が verification
evidence の追加より厳密に前であることを証明しなければなりません。missing、partial、wrong
status、foreign path、post-archive-only row は fail closed です。同じ row を変更せずに active、
awaiting merge/close、closed の state を表現でき、archived evidence を書き換えません。この
route は pending registry の default-branch stale rule を緩和しません。

Pull-request merge ref は default branch と feature snapshot を結合した tree であり、feature
snapshot の replay ではありません。default branch が後続の authoritative lifecycle decision
を追加する場合、各 parity row は pre-merge receipt を保持しながら、その decision も列挙する
必要があります。push head が green でも後続 close path の欠落は fail closed です。Runtime
recovery successor は predecessor bytes を保持し、delivery を昇格する前に正確な
base-plus-feature topology を検証します。

Merge は Work Item の close ではありません。hosted check が通った後、正確な
branch と worktree は別の resource-finalization 境界で処理します。

```text
finalize-plan → finalize → finalize-verify → close
```

これは Runtime が提供する command です。すべて `--repo` を明示し、型付きで
identity-bound な context/receipt を要求します。暗黙の削除は行いません。Work Item は
verification 後にだけ archive でき、`finalize-verify` が `Deleted` または明示的に認可された
`Retained` receipt を受理した後にだけ close できます。Archived verification evidence は
不変の historical truth として保持され、Runtime upgrade 後に current result として再検証
されません。一方、新しい finalization receipt は close を実行する Runtime に必ず bind
されます。

## Release tag の transition 順序

PR の merge と有効な pre-merge finalization receipt の commit が完了してから Release
tag を作成します。Source quality は、不変の tag 上で receipt が identity-bound であり、
receipt に記録された PR head が tag commit の ancestor であることを Git が証明できる
場合に限り、`awaiting_merge_close` 境界として扱います。これは Work Item の close や
cleanup の免除ではありません。公開後の binary で `finalize`、`finalize-verify`、構造化
human `close` を続けて実行します。通常 branch、証明できない tag、malformed receipt は
引き続き fail-closed です。

- `finalize-plan` は正確な Work Item branch/worktree、provider PR、merge head、
  remote、default branch、cleanup 計画を記録します。branch や worktree を削除しません。
- `finalize` は PR、head、dirty state、protection の確認が通った後だけ、正確な
  merge 済み branch/worktree を処理します。branch の silent deletionは禁止です。
- `finalize-verify` は同期済み default branch、関係する worktree の clean 状態、
  正確な local/remote branch 削除を証明します。provider error、identity mismatch、
  観測不完全は `unknown` として Work Item を recovery のため open に保ち、続行の
  許可にはしません。
- `retain` は owner、理由、scope、期限または review 条件を持つ明示的な Human
  Decision です。保持した resource を cleanup 成功に黙って変換しません。組織 policy
  が限定的な retain path を明示的に許可しない限り、`close` は block されます。
- `finalize-verify` の成功（または別途認可され監査可能な retain path の受理）より前に
  `close` してはいけません。失敗時は retry identity と可視の yellow/red Outcome を保持します。

## Agent provider surface

Adapter はこれらの規則を repository-local に投影する薄い層であり、別の
policy engine ではありません。`agent install` は明示的かつ ownership 付きで
実行します。新しい Cursor の install は provider-native な
`.cursor/rules/ai-cockpit.mdc` を使用します。既に managed な
`.cursor/rules/ai-cockpit.md` がある repository では、upgrade で rename や
user file の上書きを行わず、legacy target を維持します。Runtime は
`AGENTS.md`、`CLAUDE.md`、`GEMINI.md`、Cursor rule、global provider/MCP
configuration を自動 install しません。

生成される managed section は、上記の Contract-first、pause、Summary、可視の
Outcome、closure の意味をそのまま伝えます。これは advisory な discovery
guidance であり、現在の governance state は常に明示的な Runtime query から
取得します。provider の prompt が authority を付与することはありません。

## 安全境界

規則は language-neutral かつ repository-local に保ちます。secret や machine
credential を含めず、user-global Agent/MCP configuration を変更せず、managed Agent
prompt を governance authority として扱いません。V1 Runtime code、schema、installer、
template implementation を本 repository にコピーしません。
人から明示的に依頼されない限り、ユーザーの変更を revert しません。既定の指示読込集合は
`.ai/README.md`、`.ai/glossary.md`、`AGENTS.md`、現在の machine-readable governance
records です。`docs/archive/**` と reference material は、人または Contract が明示的に
含めない限り、過去/参考情報であり現在の指示権限を持ちません。status、receipt、archive
などの生成ファイルは Runtime が生成し、手編集しません。
参照元の hosted-verification snapshot 例外に相当する command は本 Rust project には
ありません。未公開の local snapshot を reviewed branch/PR workflow の代わりに push
しないでください。
