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
- 正規の delivery 順序は latest remote default base → 専用 branch/worktree →
  implement → finish/archive → push → reviewed PR → merge → close → synchronize
  and clean です。PR review 前に feature branch を local `main` へ merge せず、merge
  前に branch を削除せず、provider の自動削除で finalization を迂回しません。remote
  step が失敗したら retry checkout と identity を保持します。reviewed merge、default
  branch 同期、正確な cleanup が完了して初めて `ready_on_base` であり、detached
  worktree は ready ではありません。
- Work Item ごとに一つの Contract、専用 branch/worktree、一つの PR を使います。
  scope、evidence ownership、repository context、serialized projection が分離し、
  Runtime が compatible と判定した独立 Work Item だけを並行できます。
- `start` または `work-item new` の前に top-level `status.readiness` projection を確認します。close decision が有効でない archived Work Item、`.ai` 以外の既存変更、detached HEAD、または検出された remote default base と HEAD の不一致がある場合、通常の新しい Work Item は拒否されます。正の readiness claim は `readyOnBase` だけで、remote metadata が欠落または曖昧なら `unknown` とし、暗黙に green へ昇格しません。recovery successor は predecessor の明示的な継続です。
- 通常の `start` と `work-item new` は、専用の linked worktree と non-default branch から実行しなければなりません。repository の primary worktree は同期済み default branch 専用です。実装をここに bind すると、finalization で branch/worktree の正確な削除を証明できません。Runtime は Work Item を書く前に primary worktree と既知の default branch を拒否します。明確な remote default base がない linked worktree も拒否されます。これは fail-closed の topology check であり、provider bypass ではありません。
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
- Recovery receipt は append-only chain です。canonical `<id>.recovery.json` が先行する retry の場合、CI は有効な digest-suffixed `<id>.recovery.<digest>.json` successor/supersession receipt を解決し、選択した path を各 parity projection に bind します。candidate が invalid または ambiguous なら fail-closed のままとし、gate は retry を terminal successor として扱いません。新しい検証後に retry の predecessor Contract、Summary、Outcome、Events digest が archive の記録と一致しなくなった場合、CI はその retry を消費済みの履歴 evidence として扱い、実際の finalization decision へ進みます。binding が一致し、Summary に blocked が明示された retry は現在の recovery 境界として残ります。
- manifest がまだ `archived` の Work Item でも、元の archive 後に有効な append-only `supersede` recovery decision を追加できます（provider の PR base と凍結された Contract base を一致させられない場合など）。Runtime は明示的な recovery 経路を通じて predecessor を close できますが、archive manifest やアーカイブ artifact bytes は書き換えません。resource context を持つ通常の archived Work Item に有効な provider finalization receipt がない場合、Outcome は yellow/not-ready のままです。無効な recovery candidate がこの finalization gate を回避することはありません。
- review 済みの governance 修正により archived Contract を正当に変更した場合は、歴史 evidence を編集したり古い Contract digest で通常の `work-item recover` を再利用したりせず、`work-item revalidate-archived` を使用します。この command は現在の archive manifest/Contract と完全な historical verification evidence を bind した append-only の `contract_amendment_revalidation` successor decision を記録し、predecessor が pending close の間に `not_ready` successor を作成します。新しい intent、scope、verification、finalization、human close は successor が担当し、repository-bound な terminal evidence が揃ったときだけ predecessor の履歴 close を許可します。歴史が欠落または矛盾する場合は fail-closed のままで、predecessor bytes は書き換えません。
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
- Rust の green terminal は reference の `status=completed` と
  `humanStatusColor=green` に相当し、さらに `state=Verified`、`decisionState=green`、
  current binding、直接の human-visible delivery を要求します。handoff には issue
  count、blocker/停止理由、解決済み issue、risk、verification、impact、next action を
  含めます。事実には evidence を付け、根拠のない benefit は inference と明示します。
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

Hosted quality boundary は動的で収束します。同じ Pull Request の実行は concurrency group に束ね、
superseded run は cancel しますが、`main` と release truth はこの PR policy で cancel しません。
最初の route-planning job が changed paths、stage、risk から `light`、`standard`、`strict` を選びます。
documentation-only の `light` route は Windows と V1-oracle job を開始しませんが、選択された profile の
検査を弱めるものではありません。repository gate の前に、malformed/stale な active Summary、
不正な checkpoint 数、または green preflight のない `finish_ready` を route が拒否します。失敗は
一つの安定した root code と remediation で示され、fixture の意図的な negative output は複数の独立
failure として数えられません。adopter project は自身の Repository Context で同じ境界を継承し、
project 間で Work Item、Evidence、failure state を共有しません。

実装前の Contract review は、宣言された intent、scenario の形、acceptance
declaration、parallel boundary も検証します。壊れた scenario list、重複した
scenario、空の acceptance、無効な slot boundary は review finding であり、
Agent が推測で修復する事実ではありません。scenario coverage の必須性は
risk policy が決め、人間の宣言と fresh evidence が揃うまで Runtime は
yellow/red を維持します。

Agent Risk と checkpoint は同じ Rust lifecycle validator で強制されます。typed
required verification declaration は preflight、verify、finish、archive、close
で共有され、missing、duplicate、failed、invalidated gate は presentation field
によって許可にはなりません。typed `checkpointPolicy` は Verification strength
（`light`、`standard`、`strict`、`release`）と required stages/checks だけを選び、
Evidence Assurance を意味しません。
`work-item revalidate-amendment --repo <repo> --id <id> --reason <text>` は
Contract amendment evidence を append し、`before_edit` を置換しません。
verification 開始後は既存 required check を無効化し、fresh preflight と
verification を要求します。resume history と checkpoint timestamp も bind され、
stale predecessor evidence は current Work Item を authorize できません。
typed checkpoint evidence 導入前に作成された repository では、amendment が保存済みの
legacy checkpoint identity fields から typed `before_edit` entry を決定的に昇格できます。
これは intent、authority、verification を推測するものではありません。

Checkpoint snapshot には時間的な意味があります。有効な `before_edit` または
amendment entry はその認可境界の repository state を記録するため、認可された
編集と fresh preflight の後は現在の snapshot より古くなり得ます。終端 evidence
である `before_finish` だけは現在の Contract、repository identity、snapshot に
一致しなければならず、stale、foreign、malformed、duplicate、symlink entry は
fail closed します。

最終化も snapshot に依存します。最後の verification 後は、Runtime が生成した
finish/outcome/archive record を commit する前に `finish` と `archive` を実行します。
`.ai/` だけの commit でも snapshot identity は変わるため、verification と archive の間に
commit すると receipt は stale になります。現在の Work Item で再検証し、gate を迂回しては
いけません。archive 成功後にだけ archive record を commit し、その後 provider finalization
と hosted checks を行います。

## Resource finalization の境界

新しい Work Item は `close` の前に provider 側の branch と worktree の cleanup を
完了しなければなりません。`retained`、`blocked`、`unknown` の finalization result は
terminal success ではありません。Runtime は legacy library entry point と Runtime-bound
CLI の両方でこの順序を拒否します。旧 Runtime が作った immutable な履歴については、
`work-item finalize` が close 後に identity-bound な deleted transition を 1 件だけ
append できます。これは限定された reconciliation であり、closed root digest を束縛し、
元の close bytes を保持し、PR の merge、branch の削除、worktree の除去を証明しなければ
なりません。新しい Work Item を認可したり、通常の cleanup-before-close ルールを弱めたり
するものではありません。

Finalization evidence は append-only chain です。canonical `<id>.finalize.json` は不変の chain root であり、後続の provider observation は predecessor digest と sequence を束縛した `<id>.finalize.<digest>.json` に保存されます。archived Contract は `baseRevision` を凍結し、canonical/transition receipt の `pullRequest.baseRevision` は record 時と `finalize-verify` 時の両方で完全一致しなければなりません。archive 前の rebase では active Contract の binding と review を更新し、archive 後の rebase は禁止して record を書き換えず fail-closed recovery を行います。`finalize-verify` と `close` は一意な線形 head を要求し、stale predecessor、fork、malformed record、symlink、base mismatch、identity drift は fail closed になります。pre-merge blocked root は連続する merge observation（`retained`）と cleanup（`deleted`）transition で進みます。canonical governance receipt の commit により PR head が進む場合、最初の unmerged-to-merged observation だけが `governanceAppendRevision` を宣言できます。PR、branch、worktree の各 head は同時に変わり、Git は旧 head が新 head の ancestor であることを証明します。この append 区間には、同一 Work Item の通常 finalization receipt と、Runtime が生成した完全な post-finalize evidence bundle だけを追加できます。bundle の path は `.ai/evidence/<id>/quality-route-post-finalize.json` と `.ai/evidence/<id>/repository-gates-post-finalize.json` に限定されます。受理される各 path は Git の `A`-only change で、tree entry は `100644` regular blob でなければなりません。両 evidence file は固定 schema に従い、archived Contract、PR base、bounded head、route receipt digest、manifest digest、selected profile、および passed required gates を束縛しなければなりません。これらは束縛済み observation であって、それ自体が authority ではなく、区間には引き続き finalization receipt の追加が必要です。bundle の欠落、別 Work Item または filename、malformed/duplicate-key JSON、binding mismatch、削除、変更、rename、symlink、無関係な変更、非 merge または後続の head drift は拒否されます。archive bytes は書き換えません。cleanup は受理済み head を保持します。

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

実運用では最低 2 つの commit を使います。最初の commit で三言語の conditional parity row と
documentation registration を先に記録し、その commit が feature branch から参照できることを
確認してから verification を実行し Runtime evidence を追加します。row registration と
verification evidence を同じ commit にまとめてはいけません。順序を誤った場合は immutable
delivery を保持し、履歴を書き換えず明示的な recovery successor を作成します。

Pull-request merge ref は default branch と feature snapshot を結合した tree であり、feature
snapshot の replay ではありません。default branch が後続の authoritative lifecycle decision
を追加する場合、各 parity row は pre-merge receipt を保持しながら、その decision も列挙する
必要があります。push head が green でも後続 close path の欠落は fail closed です。Runtime
recovery successor は predecessor bytes を保持し、delivery を昇格する前に正確な
base-plus-feature topology を検証します。

Recovery evidence は記録時だけでなく読み取り時にも再検証します。current recovery
candidate が Outcome projection または superseded archive behavior に影響する前に、Runtime
は regular-file/filename 境界、repository と current Runtime identity、predecessor の
Contract/Summary/Outcome/Events digest、timestamp、decision shape、正確な successor Contract
binding を再確認します。malformed、foreign、stale、tampered、ambiguous な candidate は安定した
`recovery_decision_invalid` 境界となり、active artifact を移動できません。historical archive
の immutable bytes と historical projection は保持され、この current-read rule が遡及的に
書き換えたり再分類したりすることはありません。

新しい Runtime が作成する successor には `predecessorWorkItemId`、
`predecessorContractDigest`、`recoveryDecisionPath` が必要です。これらの field より前に
作成された historical successor は、recovery receipt が predecessor/successor/repository の
digest を正確に bind し、successor に検証済み archive、strict verification evidence、confirmed
structured close が揃う場合だけ互換になります。新しい append-only supersede receipt には
`successorBindingMode: legacy_terminal_evidence` を記録します。欠落、foreign、stale、malformed、
symlink、または不完全な terminal evidence は `recovery_decision_invalid` のままです。これは
明示的に限定された historical compatibility projection であり、無条件の green ではなく、
predecessor bytes も書き換えません。

`retry` は明示的な lifecycle transition であり、green の宣言ではありません。失敗した
gate が active item を `finish_ready` に残した場合、Runtime は現在の Summary だけを合法な
`checkpointed` retry point に戻し、一時的な失敗 projection を消去します。blocked Outcome
と predecessor digest は append-only recovery receipt が参照し続け、新しい current Outcome
には fresh な `verify` と `finish` が必要です。それ以外の lifecycle state からの retry は
拒否されます。superseded manifest はコピーされた report digest を直接 bind するため、
生成された task-report digest を埋める目的で歴史的 Outcome bytes を書き換えません。

`finish.lifecycle` failure の後に人が認可した retry を行う場合、Runtime は明示的な
`recoveryRetryPending` marker を設定できます。この marker は記録済み recovery receipt
に対する fresh な verification を一度だけ許可するもので、green の preflight を合成しません。
置換 report と event binding が成功した後にだけ `finish` が marker を消去するため、途中までの
retry は可視のまま recovery できます。

Merge は Work Item の close ではありません。hosted check が通った後、正確な
branch と worktree は別の resource-finalization 境界で処理します。

```text
finalize-plan → finalize → finalize-verify → close
```

これは Runtime が提供する command です。すべて `--repo` を明示し、型付きで
identity-bound な context/receipt を要求します。暗黙の削除は行いません。Work Item は
verification 後にだけ archive でき、`finalize-verify` が identity-bound な `Deleted`
receipt を受理した後にだけ close できます。`Retained` は中間の merge observation または
旧 record の legacy fact に限られ、新しい close を認可しません。旧い close 済み record
だけは上記の限定的な deleted reconciliation を append できます。Archived verification
evidence は不変の historical truth として保持され、Runtime upgrade 後に current result として
再検証されません。新しい finalization receipt は close を実行する Runtime に必ず bind されます。
`pending:<stable-reference>` provider sentinel も `unknown` と同じ provisional であり、`finish` や `archive` を通過できません。verification と terminal lifecycle evidence を記録する前に、正確な reviewed resource context に置き換えてください。

structured close の後には、controlled documentation projection と default-branch
terminal check が必要です。

```text
close → promote closed docs → terminal CI
```

synchronized detached closure context で `python3
tests/docs/promote_closed_work_item.py --repo <repo> --work-item <id>` を実行し、
続けて同じ helper の `--check-all` を実行します。helper は regular non-symlink の
archive、verification、linear finalization、sequence-2 deleted、merge、structured
close identity を先に検証します。write boundary は exact 3 Work Item documents の
machine-owned lifecycle frontmatter と、3 reference-parity documents の exact Work
Item row だけです。body prose や `.ai` lifecycle truth は rewrite しません。invalid
input は write 前に fail closed となり、stale projection は quality gate に失敗します。
これは explicit repository workflow helper であり、Runtime Core による自動 Markdown
mutation ではありません。

有界な terminal exception があります。documentation-promotion Work Item の archived
Contract が、自身の三言語ページと三つの parity ledger を含む正確な docs-only scope を
持つ場合、その Work Item 自体が projection boundary です。close 後も pre-archive の
conditional row は意図的に残り、`--check-all` は terminal evidence を検証しますが、
同じ Work Item を書き換えるための再帰的 successor は作成しません。混在、wildcard、または
不正な scope は通常どおり fail closed です。

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
  Decision です。保持した resource を cleanup 成功に黙って変換せず、新しい `close` を
  認可しません。旧い close 済み record だけが上記の限定的な deleted reconciliation を
  受けられます。
- `finalize-verify` が `Deleted` で成功する前に `close` してはいけません。失敗時は retry
  identity と可視の yellow/red Outcome を保持します。

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
