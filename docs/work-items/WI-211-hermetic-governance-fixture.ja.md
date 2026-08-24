---
author: AI Cockpit maintainers
title: "WI-211 — governance fixture の event context 分離"
description: "release workflow の GitHub event variables が governance regression fixture を汚染しないようにする。"
audience:
  - maintainer
  - reviewer
workItemId: WI-211-hermetic-governance-fixture
status: recovered
authority: canonical
lastVerifiedBy: WI-211-hermetic-governance-fixture
---

# WI-211 — governance fixture の event context 分離

Release workflow は source quality job 全体に GitHub event variables を export します。
以前の governance regression test はその値を通常の fixture に漏らしていたため、local では
成功し release-tag CI では失敗する可能性がありました。本 Work Item では各 fixture の event
context を明示します。

## Acceptance

1. `tests/ci/governance_integrity_gate_test.sh` が通常環境と release-tag 環境変数の両方で成功する。
2. 通常 fixture は release context を明示的に解除し、実際の `release-tag-*` fixture は厳格な tag context を使う。
3. 両環境で同じ deterministic findings と exit status が保たれる。
4. immutable な v0.2.26 の公開履歴を移動・書き換えず、source fallback にも使わない。

## Out of scope

Runtime governance semantics、public Release asset、reference source の file-by-file parity、
user-global Agent/MCP configuration は対象外です。

## Verification

GitHub event variables がない環境と、`GITHUB_EVENT_NAME=push`、
`GITHUB_REF=refs/tags/<tag>`、対応する `GITHUB_SHA` を設定した環境で regression を実行します。
続いて repository gate manifest と workspace tests を実行します。

## Evidence boundary

この修正は source-test evidence です。失敗した v0.2.26 の公開を成功 Release に変えるものではなく、
その履歴は immutable のまま保持します。次の公開には新しい tag を使います。
