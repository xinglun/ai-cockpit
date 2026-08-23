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

Merge は Work Item の close ではありません。hosted check が通った後、正確な
branch と worktree は別の resource-finalization 境界で処理します。

```text
finalize-plan → finalize → finalize-verify → close
```

これは WI-160 の policy baseline であり、Runtime `0.2.17` が提供する command
ではありません。Runtime 統合は後続の明示的な Work Item で保留します。この文書と
static gate は CLI が既に実装済みだとは主張しません。

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
