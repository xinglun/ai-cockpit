---
author: AI Cockpit maintainers
title: "WI-830 — release dispatch 構文修復"
description: "publication 前に検出された release close expression を修復する。"
audience: [maintainer, reviewer]
status: in_progress
authority: authorized
workItemId: WI-830-release-dispatch-syntax
lastVerifiedBy: WI-830-release-dispatch-syntax
---

[English](WI-830-release-dispatch-syntax.md) · [简体中文](WI-830-release-dispatch-syntax.zh-CN.md)

# WI-830 — release dispatch 構文修復

## Intent と boundary

WI-830 は dispatch-only release close condition に余分にあった閉じ括弧を修復します。
この不具合は GitHub が run を作成する前に拒否しました。同じ malformed expression に
対する local regression も追加します。

Runtime protocol、product behavior、release acceptance の実行、immutable tag や
Release の書き換え、historical migration、user-global configuration は範囲外です。

## Acceptance

- GitHub が `workflow_dispatch` release workflow を受け付ける。
- local release policy が malformed close expression を拒否し、balanced expression を受け付ける。
- この修復で product artifact、tag、Release を書き換えない。
- publication retry 前に hosted checks が成功する。

## Verification

- `bash tests/ci/release_gate_policy_test.sh`
- `bash tests/release/workflow_policy.sh .github/workflows/release.yml`
- `bash tests/release/action_runtime_policy.sh .github/workflows/release.yml`
- `cargo fmt --all --check`
- hosted PR checks と明示的な publication dispatch 一回。
