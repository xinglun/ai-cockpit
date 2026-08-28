---
author: AI Cockpit maintainers
title: "WI-364 — primary worktree release recovery"
workItemId: WI-364-primary-worktree-release
description: "通常の Work Item が repository の primary worktree に bind することを防ぎ、専用 checkout から v0.2.37 を再 delivery する。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-364-primary-worktree-release
capabilityClaims: [lifecycle_entry, release_distribution]
---

# WI-364 — primary worktree release recovery

[English](WI-364-primary-worktree-release.md) · [简体中文](WI-364-primary-worktree-release.zh-CN.md)

## Intent

WI-363 で露呈した release delivery boundary を根治します。通常の Work Item は
repository の primary worktree または default branch に bind せず、専用 Work Item
worktree から v0.2.37 を再 delivery します。predecessor の immutable recovery evidence は保持します。

## Scope と boundary

- 現在の checkout が Git primary worktree または既知の default branch の場合、Contract を書く前に通常の `start` と `work-item new` を拒否します。
- remote default base が欠落または曖昧な linked worktree を拒否し、linked worktree のない local calibration repository は `status: unknown` のままにします。
- primary、default、dedicated、曖昧な metadata の各 topology を対象とする CLI regression を追加します。
- canonical な三言語 workflow、command、parity 文書に topology 要件と WI-363 recovery boundary を記載します。
- この専用 worktree から immutable な v0.2.37 artifact、adopter、N-1、finalization、close、正確な cleanup acceptance を完了します。

WI-363 の archive/evidence/decision bytes、release artifact semantics、global Agent/MCP configuration、無関係な Runtime behavior の変更は対象外です。

## Acceptance

1. 通常の `start` と `work-item new` は primary worktree と default branch で fail closed し、専用 linked worktree は許可されます。
2. remote default metadata が欠落または曖昧な場合、linked worktree は authorize されず、false-green Contract を書きません。
3. topology regression は全ケースをカバーし、拒否された entry に Work Item file を残しません。
4. 三言語の workflow、command、parity 文書がルールを説明し predecessor recovery boundary にリンクします。
5. 公開 v0.2.37 artifact は source/workspace fallback なしで checksum-bound download され、adopter と N-1 receipt が isolation と cleanup を証明します。
6. reviewed merge、finalization、close、正確な branch/worktree cleanup 後、同期済み `main` が ready on base になります。

## Verification boundary

installed Runtime が Contract amendment、preflight、checkpoint、verification、finish、archive、finalization、close evidence を記録します。Hosted CI と公開 artifact acceptance が release claim の根拠です。WI-363 の archive と recovery bytes は historical immutable のままで、書き換えません。
