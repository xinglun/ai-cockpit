---
author: AI Cockpit maintainers
title: "WI-160 — Resource finalization と branch/worktree closure の baseline"
description: "review 済み PR の merge 後に必要な resource-finalization 境界を定義し、static に保護します。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-160-resource-finalization-baseline
workItemId: WI-160-resource-finalization-baseline
---

# WI-160 — Resource finalization と branch/worktree closure の baseline

## Intent

Merge と Work Item の close は別の事実です。この Work Item は、正確な branch や
worktree が dirty、識別不能、Decision なしで保持、または残存している間に、review
済み PR を完全に close 済みとして扱うことを防ぎます。

## Boundary

Policy baseline は次の通りです。

```text
finalize-plan → finalize → finalize-verify → close
```

`finalize-plan` は正確な branch、worktree、provider PR、merge head、remote、default
branch、cleanup 意図を記録し、削除はしません。`finalize` は identity、protection、
dirty state の確認後に、その正確な merge 済み resource だけを処理します。
`finalize-verify` は同期済み default branch、関係する worktree の clean 状態、正確な
local/remote branch 削除を証明します。

観測失敗や provider error は `unknown` として Work Item を recovery のため open に
保ちます。`retain` は owner、理由、scope、期限または review 条件を持つ明示的で限定
された Human Decision の場合だけ許可し、cleanup 成功へ黙って変換しません。finalize
成功前の `close` は禁止です。

Runtime `0.2.17` はこれらを CLI command として提供します。WI-159 が typed Runtime
command と receipt 統合を実装し、WI-161 が archived bytes を書き換えない historical
evidence close compatibility を追加します。

Verification: `.ai/evidence/WI-160-resource-finalization-baseline.verification.json`。
Archive: `.ai/work-items/archive/WI-160-resource-finalization-baseline.archive.json`。
Decision: `.ai/decisions/WI-160-resource-finalization-baseline.close.json`。

## In scope

- 三言語の `agent-workflow` と `reference-parity` の契約文。
- `tests/workflow/resource_finalization_policy.sh` と test wrapper による static/regression gate。
- 境界と Runtime evidence compatibility を説明する三言語 Work Item 文書。

## Out of scope

- Runtime source と `crates/**` の変更。
- provider 側 branch 削除、GitHub workflow、global Agent/MCP configuration。
- 既存 branch と worktree の削除または変更。

## Acceptance

1. 三言語 workflow page が `finalize-plan`、`finalize`、`finalize-verify` を要求し、
   `unknown`/`retain` を保持し、silent deletion と cleanup 前の close を禁止し、Runtime
   Runtime command boundary を明示する。
2. 三言語 parity page が同じ Implemented boundary と historical evidence compatibility を記載する。
3. repository の static gate が通り、任意の言語 page から closure rule を削ると test が失敗する。
4. 変更は `docs/` と `tests/` に限定され、Runtime source と生成された governance receipt は
   手編集しない。

## Verification

`tests/workflow/resource_finalization_policy_test.sh` と documentation acceptance gate を
実行します。Runtime lifecycle evidence が Contract と verification receipt を bind し、
CLI 統合は WI-159、historical evidence close compatibility は WI-161 で扱います。
