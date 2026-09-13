---
author: AI Cockpit maintainers
title: "WI-829 — 明示的な release trigger 修復"
description: "release publication を identity-bound、dispatch-only にする。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-829-release-trigger-repair
lastVerifiedBy: WI-829-release-trigger-repair
---

[English](WI-829-release-trigger-repair.md) · [简体中文](WI-829-release-trigger-repair.zh-CN.md)

# WI-829 — 明示的な release trigger 修復

## Intent と boundary

この Work Item は、governance Work Item identity を持つ明示的な
`workflow_dispatch` だけで publication を開始するようにします。annotated tag は
不変の入力として先に作成して push し、tag の push だけでは未解決の release route
を開始しません。

Runtime protocol、product behavior、historical Work Item migration、既存 Release や
tag の書き換え、user-global configuration はこの Work Item の範囲外です。

## Acceptance

- 欠落、形式不正、または曖昧な release identity は compilation や publication 前に拒否される。
- dispatch は高価な release job の前に immutable tag、source commit、Work Item、Contract を検証する。
- candidate/public installation、upgrade、handoff、attestation、close barrier は必須のままにする。
- policy と三言語の release documentation は同じ dispatch-only boundary を説明する。

## Verification

- `bash tests/release/workflow_policy.sh .github/workflows/release.yml`
- `bash tests/release/action_runtime_policy.sh .github/workflows/release.yml`
- `bash tests/release/version_consistency.sh --repo <repo>`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh <repo>`
- `cargo fmt --all --check`
- reviewed PR の hosted CI と、明示的な v0.2.92 publication dispatch 一回。

## Recovery policy

不変の tag と public Release は書き換えません。失敗した dispatch は最初の invalid
phase からだけ retry し、input、Contract、helper、configuration identity が一致する
evidence のみ再利用します。
