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
  どれかが失敗したら recovery のため未完了のままにします。

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

## 安全境界

規則は language-neutral かつ repository-local に保ちます。secret や machine
credential を含めず、user-global Agent/MCP configuration を変更せず、managed Agent
prompt を governance authority として扱いません。V1 Runtime code、schema、installer、
template implementation を本 repository にコピーしません。
