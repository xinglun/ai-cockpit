---
author: AI Cockpit maintainers
title: Work Item ライフサイクルのクローズ
description: archive、merge、正確な cleanup 後に reviewed Work Item を安全に閉じる。
audience: [adopter, maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/reference/work-item-lifecycle-closure.md
lastVerifiedBy: WI-512-reference-docs-batch-33
---

# Work Item ライフサイクルのクローズ

[English](work-item-lifecycle-closure.md) · [简体中文](work-item-lifecycle-closure.zh-CN.md) · [日本語](work-item-lifecycle-closure.ja.md)

クローズは `start → preflight → checkpoint → verify → finish → archive` 後の最終 handoff であり、
branch deletion の近道ではありません。Runtime は reviewed PR、正確な Work Item head、archived
Contract/Summary/evidence、同期済み base、clean worktree、remote branch 不在をすべて検証します。

## 通常の route

```text
verify → finish/archive → push → reviewed PR と hosted checks → merge
→ finalize → finalize-verify → close → synchronize and clean
```

close は PR state、branch/head identity、base fast-forward、archive/decision receipt、clean
worktree、remote branch 不在を順に検証し、その後に対象の local Work Item branch だけを削除します。
provider の auto-delete でこの証明を迂回してはいけません。

`ready_on_base` は呼び出し元 checkout が clean で同期済み default branch 上にある状態です。
`closed_but_current_worktree_detached` は close 済みだが別の検証済み worktree が base を所有する
状態なので、表示された base worktree から継続し、detached checkout を ready と扱いません。

## recovery と歴史境界

欠落、stale、foreign、矛盾した事実は fail closed で retry identity を保持します。provider anomaly
や stacked-PR recovery は別の明示的 receipt を使い、immutable archive を書き換えず、open PR を
merged に変えません。reference source の `make` と Python orchestration は Rust Runtime command
ではありません。Rust は明示的な `--repo` と repository-local evidence で同じ review、archive、
正確な cleanup の意図を保ちます。

### 歴史 archive の quarantine

明示的に承認された `supersede` recovery では、旧版の Task Outcome Markdown など任意の
artifact が archive manifest の digest と一致しなくなった場合でも、immutable な archived
predecessor を close できます。Recovery receipt は `predecessorArchiveManifestDigest` で
archive manifest の正確な bytes を bind しなければなりません。Runtime は artifact や
manifest を書き換えず、close receipt に `historical_low` の
`historicalArchiveIntegrity` を記録し、歴史 Work Item を current green verification ではなく
yellow として投影します。必須の Contract/Summary/Outcome bytes、identity、events、その他の
artifact integrity は引き続き fail closed です。manifest の欠落、foreign、malformed、symlink、
または異なる digest ではこの quarantine path を使用できません。

## successor と historical recovery

blocked な predecessor の Contract、Summary、Outcome、evidence は immutable な
historical record として保持します。作業を引き継ぐ場合は、`work-item recover` の
identity-bound retry/successor receipt を使い、元の決定や証拠を上書きしません。
legacy finalization は read-only の recovery plan/receipt で扱います。

Provider PR が存在しない過去の direct merge では、実在する merge commit、parents、base
revision、repository identity を記録し、historical/low-assurance と明示できます。架空の
PR 番号を補ったり、過去の receipt を書き換えたりしません。共有主 worktree の
`retained` disposition も実際の resource 状態として保持します。

旧版の shared-primary receipt に `historical` field がない場合でも、`finalize-verify` は
`provider=local`、Contract/receipt identity、primary checkout の topology、retained された
clean resource を確認した後だけ、狭い read-only projection を行います。その
`historical_low` 結果は `close` で利用できます。事実が欠落または矛盾する場合は明示的な
recovery plan/receipt が必要で、archive や predecessor bytes の copy/書き換えは行いません。

stacked PR や post-archive の provider anomaly は通常の close の近道ではありません。
別の human-authorized append-only evidence が必要であり、open または検証不能な PR を
merged と扱いません。reference source の Make/Python recovery command は Rust Runtime
command ではありません。
