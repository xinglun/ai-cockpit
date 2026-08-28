---
author: AI Cockpit maintainers
title: "WI-369 — merge 後 CI transition gate"
description: "review 済み merge から close までの過渡状態と、古い未 close Work Item を区別し、gate を弱めない。"
workItemId: WI-369-post-merge-ci-transition-gate
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-369-post-merge-ci-transition-gate
capabilityClaims:
  - governance_integrity
  - reference_parity
---

# WI-369 — merge 後 CI transition gate

[English](WI-369-post-merge-ci-transition-gate.md) · [简体中文](WI-369-post-merge-ci-transition-gate.zh-CN.md)

## Intent と境界

review 済み merge の直後に default-branch CI が実行される一方、provider の finalization と
authoritative close receipt は merge 後の cleanup で記録されます。この Work Item は、その
順序による誤った `missing_terminal_decision` failure を除去します。ただし close 欠落を
advisory にはしません。

許される唯一の transition は、設定された default branch への実際の GitHub `push` で、
`HEAD` が正確な二親の merge であり、その merge が対象 Work Item の archive Contract を
新規追加する場合です。gate は `awaiting_merge_close` と報告し、次の通常の default-branch
commit では finalization と close を必須にします。

変更範囲は repository gate、CI 呼び出しの説明、regression fixture、三言語の文書/parity
record に限定します。Rust Runtime lifecycle、release artifact、provider API、global Agent/
MCP configuration、source Python/Make/V1 runtime は対象外です。

## Acceptance

- 条件を満たす merge は明示的な `awaiting_merge_close` transition として扱われ、誤った
  `missing_terminal_decision` finding を出しません。
- close のない次の通常 default-branch commit は fail-closed のままです。
- direct commit、壊れた/無関係な merge、古い未 close Work Item、parity 欠落、GitHub context の
  欠落/矛盾は引き続き blocking です。
- 判定は Git history と標準の不変 GitHub context から決定的に行い、bypass flag や process-global
  current repository state は導入しません。
- regression test と三言語文書は同じ bounded transition と eventual finalize/close を説明します。
- merge、close、正確な branch/worktree cleanup の前に、installed Runtime verification と可視の
  human Outcome を完了します。

## Verification record

regression suite は archive Work Item を新規追加する reviewed merge を構成して transition を
検証し、その後に通常 commit を追加して同じ未 close Work Item が gate を block することを検証
します。既存の negative fixture も gate test corpus に残します。

GitHub workflow は `GITHUB_EVENT_NAME`、`GITHUB_REF`、`GITHUB_SHA` を継承します。これらは event
識別専用であり、Contract、evidence、PR、close の検証を置き換えません。
