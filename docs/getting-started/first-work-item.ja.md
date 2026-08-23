---
author: AI Cockpit maintainers
title: "最初の Work Item"
description: "Authorized Contract から reviewed close までの完全な Runtime-native route。"
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - cli_lifecycle
---

# 最初の Work Item

一つの Work Item に一つの専用 branch/worktree と一つの pull request を使います。Repository
で discovered remote default branch の最新 commit から始め、repository-bound command は
常に repository を明示します。

```bash
repo=/path/to/repository
id=WI-001-example-change
ai-cockpit start --repo "$repo" --id "$id" --intent "Bounded example change を行う。" --goal "Example の reviewed evidence を届ける。" --scope 'docs/**' --out-of-scope 'src/**' --risk normal --authority authorized --acceptance "Documented example と registered check が pass する。" --required-evidence verification
```

Generated human-owned Contract を review します。Actual source、scope、out-of-scope、acceptance、
verification、authority、remote、default branch、base revision が必要です。Generated Summary、
evidence、Outcome、archive、decision receipt は手で編集しません。

## Implementation 前に実際の review resource を bind する

Initial governance bytes を commit、専用 branch を push し、merge せず draft pull request を
作ります。Actual provider/Git facts を読み、PR URL、branch、worktree、remote、base branch を
発明しません。その facts だけを temporary `ResourceFinalizationContext` に入れます。

```json
{
  "branch": "feature/example-change",
  "worktree": "/absolute/path/to/worktree",
  "baseBranch": "main",
  "baseRemote": "origin",
  "provider": "github",
  "pullRequest": "https://github.com/owner/repository/pull/123"
}
```

Preflight 前に reviewed context を bind します。

```bash
ai-cockpit work-item finalize-plan --repo "$repo" --id "$id" --input /tmp/WI-001.finalize-context.json
ai-cockpit preflight --repo "$repo" --contract .ai/work-items/active/WI-001-example-change.contract.json
```

Preflight が `not_ready` または `needs_human_confirmation` なら停止して review を人へ表示します。
`verification_pending` は declared evidence の収集だけに進めます。一度だけ serial checkpoint を
記録し、Contract scope だけを実装します。

```bash
ai-cockpit checkpoint --repo "$repo" --id "$id"
ai-cockpit verify --repo "$repo" --work-item "$id" --command cargo --args test,--workspace --workers 1
ai-cockpit finish --repo "$repo" --id "$id"
```

Contract の project command を使い、Cargo は example に限ります。Final edit 後の verification
は同じ Work Item/snapshot に対して fresh でなければなりません。

## Visible Outcome を届けてから archive する

人向け handoff を独立した visible message として render します。

```bash
AI_COCKPIT_LANGUAGE=ja ai-cockpit work-item outcome --repo "$repo" --id "$id"
```

Handoff は `Outcome: 🟢`、`Outcome: 🟡`、または `Outcome: 🔴` で始まり、status、unknown、
evidence、human decision、next action を含みます。Current green Outcome だけが進めます。
JSON lookup や folded tool result は handoff ではありません。

```bash
ai-cockpit archive --repo "$repo" --id "$id"
```

## Merge と cleanup を通して finalize する

Archive bundle を先に単独で commit/push します。その push 後に provider PR を再取得し、clean
worktree を要求します。次に repository ID、Work Item、Runtime version/digest、archived Contract digest、exact PR、
branch、worktree、resource context に bind した provider-derived receipt を得ます。Merge 前の strict
receipt は reason `awaiting_merge_close`、unmerged PR、present branch、clean worktree、
`failureCodes: ["unmerged_pull_request"]` の blocked state で、retained success ではありません。

```bash
ai-cockpit work-item finalize --repo "$repo" --id "$id" --input /tmp/WI-001.premerge-finalize-receipt.json
```

Runtime が canonical finalization receipt を書きます。次の governance commit ではその receipt だけを
commit/push し、source、documentation、archive、その他の governance change を同じ head advance に
混ぜません。hosted checks を要求して reviewed pull request を
merge します。Local `main` への shortcut merge や cleanup evidence 前の provider branch deletion
は禁止です。Provider-derived merge-observation と exact cleanup receipt を追加の
Merge 後は `--repo` を surviving かつ fast-forward synchronized な default-branch checkout
へ切り替えます。削除済み feature worktree は command root にできません。Provider-derived
merge-observation と exact cleanup receipt を追加の `work-item finalize` call で append し、
immutable linear chain を保ちます。

```bash
repo=/path/to/synchronized-default-branch-worktree
ai-cockpit work-item finalize --repo "$repo" --id "$id" --input /tmp/WI-001.merge-observation-receipt.json
ai-cockpit work-item finalize --repo "$repo" --id "$id" --input /tmp/WI-001.cleanup-receipt.json
```

Unique terminal head を verify します。

```bash
ai-cockpit work-item finalize-verify --repo "$repo" --id "$id"
```

Default branch が同期済み、merged head が bound、worktree が clean、exact owned local/remote branch
が deleted のときだけ、authorized person が structured close decision を記録できます。

```bash
decision_time=$(date -u +%Y-%m-%dT%H:%M:%SZ)
ai-cockpit close --repo "$repo" --id "$id" --human-decision approved --actor human:repository-owner --authority-source repository-review-policy --reason "Reviewed evidence と exact cleanup が完了した。" --evidence-ref ".ai/evidence/WI-001-example-change.verification.json" --policy-ref "repository-review-policy" --decided-at "$decision_time" --resume-condition none
```

Failed/unknown transition は evidence と recovery condition を保持して open のままにします。
Lifecycle を green に見せるため record を削除・書換してはいけません。
[Agent workflow reference](../reference/agent-workflow.ja.md)は上記 receipt file が使う
provider/resource evidence boundary を定義します。

[Standard adoption guide](standard-adoption-guide.ja.md) | [English](first-work-item.md) | [中文](first-work-item.zh-CN.md)
