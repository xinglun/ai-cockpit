---
author: AI Cockpit maintainers
title: "WI-288 — Provider finalization correction"
workItemId: WI-288-provider-finalization-correction
description: "実際の Provider PR identity が確定した後、先行実装を不変の recovery linkage とともに再配信する。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-288-provider-finalization-correction
authority: canonical
---

# WI-288 — Provider finalization correction

## 目的

WI-287 は Provider context に placeholder の PR URL が入ったため、fail-closed
で不変の履歴として保持した。この successor は predecessor bytes と Runtime
機能を変更せず、実際の GitHub PR identity が確定してから同じ実装を再配信する。

## 境界

- WI-287 archive と recovery decision をそのまま保持する。
- verify 前にこの Contract の `resourceContext` を実際の PR に束縛する。
- インストール済み Runtime と hosted checks で再検証する。
- Provider finalization を記録・検証し、構造化 decision で close し、merged
  branch/worktree だけを正確に削除する。

## adopter との一致

対象工程にも提供される explicit repository context、fail-closed unknown、
human-visible Outcome を同じ境界で検証する。ローカル記録から Provider approval
を推測しない。

## 検証

`cargo test --locked --workspace`、conformance/documentation acceptance、PR
hosted checks、Provider finalization verify、close 後の status/doctor を実行する。
