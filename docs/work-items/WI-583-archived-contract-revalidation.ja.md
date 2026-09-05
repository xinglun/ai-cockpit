---
author: AI Cockpit maintainers
title: "WI-583 — Archived Contract 再検証と successor close"
description: "履歴検証後に正当に変更された archived Work Item のための append-only recovery path。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-583-archived-contract-revalidation
lastVerifiedBy: WI-583-archived-contract-revalidation
---

[English](WI-583-archived-contract-revalidation.md) · [简体中文](WI-583-archived-contract-revalidation.zh-CN.md)

# WI-583 — Archived Contract 再検証と successor close

## 目的

Work Item の archive 後に Contract が正当に修正された場合も、元の archive と
evidence bytes を保持し、successor で現在の再検証を記録し、人間の明示的な
権限で close できる recovery path を提供します。provider 結果は捏造しません。

## 境界

Runtime と repository-bound CLI が対象です。
`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` は外部 read-only
adopter であり、この WI では変更しません。source template の wire format、release、
CI policy 再設計、provider 操作は対象外です。

## 設計

`work-item revalidate-archived --repo <repository> --id <predecessor> --successor <successor>`
は archived Contract、archive manifest、historical verification evidence を検証して
append-only recovery decision を記録します。現在の Contract digest、元 evidence
digest、repository identity、manifest、人間の権限を束縛して successor の骨格を作成します。
Successor が通常の lifecycle と新しい検証を完了してから predecessor を close できます。

Historical evidence は書き換えず、現在の green evidence に昇格させません。欠落、破損、
stale、foreign、symlink、矛盾する evidence は fail-closed です。

## 受入れ

1. archive 後の Contract 修正と不変の元 verification を再現する fixture。
2. predecessor が pending close の間に successor revalidation を作成・検証できること。
3. successor が `start → preflight → checkpoint → verify → finish → archive → finalize → finalize-verify → close` を完了すること。
4. successor 検証後だけ predecessor を close し、履歴/現在の evidence と lineage を記録すること。
5. 改ざん、欠落、破損、foreign、stale、symlink evidence を書き込みなしで拒否すること。
6. 三言語の command/workflow docs が append-only と historical evidence 境界を説明すること。

## 検証

Contract は Rust protocol/repository/CLI の focused test、workspace test、fmt、clippy、
文書 quality gate を宣言します。Evidence は明示的な repository context を付けた
installed Runtime だけが生成します。
