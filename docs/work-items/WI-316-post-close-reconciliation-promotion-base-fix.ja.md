---
author: AI Cockpit maintainers
title: "WI-316 — post-close reconciliation promotion base fix"
workItemId: WI-316-post-close-reconciliation-promotion-base-fix
description: "W315 の履歴を書き換えず、recovered promotion correction を最新 remote default base に再 bind する。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-316-post-close-reconciliation-promotion-base-fix
---

# WI-316 — post-close reconciliation promotion base fix

## Intent と boundary

W315 は immutable な archived delivery です。Contract の `baseRevision` が古い branch head
を指していたため、hosted CI は評価前に拒否しました。本 bounded successor は最新の
`origin/main` から開始し、実際の CI base を bind して、履歴を書き換えずに reviewed な
W314/W315 correction を再配信します。

## Scope と acceptance

- Contract は hosted CI が使用する最新 remote default revision を記録します。
- 有効な successor/supersede recovery は歴史的な promotion exception とし、retry、malformed、
  foreign recovery は fail-closed のままです。
- W315 archive と predecessor evidence は byte-for-byte 不変です。
- English、Simplified Chinese、Japanese の Work Item/parity projection を verification 前に同期します。

## Verification

promotion/documentation regression、`cargo fmt`、warning deny の clippy、locked workspace test、
および正確な reviewed branch の hosted CI を実行します。Governance interface は installed Runtime を使用します。

## Related history

- W315: hosted base-revision gate に拒否された immutable delivery。
- W316: remote base binding を修正する bounded successor。

[English](WI-316-post-close-reconciliation-promotion-base-fix.md) ·
[简体中文](WI-316-post-close-reconciliation-promotion-base-fix.zh-CN.md)
