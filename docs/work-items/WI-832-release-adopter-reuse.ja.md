---
author: AI Cockpit maintainers
title: "WI-832 — adopter verification reuse"
description: "staged と upgrade の adopter acceptance を Runtime の package route に一致させる。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-832-release-adopter-reuse
lastVerifiedBy: WI-832-release-adopter-reuse
---

[English](WI-832-release-adopter-reuse.md) · [简体中文](WI-832-release-adopter-reuse.zh-CN.md)

# WI-832 — adopter verification reuse

## Intent

adopter harness は成功した verification を再利用できることを示す必要がある。そのため
profile confirmation は Runtime の workspace package route が実行する正確な command、
`cargo test --locked --package adopter` を記録する。

## Boundary

staged と N−1 upgrade acceptance は同じ package 単位の command を使う。回帰検査は最初の
receipt を保持し、二回目の verification が spawned process 0 件を報告することを確認する。
Runtime の reuse protocol、release identity、無関係な cleanup は変更しない。

## Verification

- adopter と upgrade harness の static check が通過する。
- 実際の v0.2.92 staged candidate で二回目の verification は `nodesReused: 1`、
  `processesSpawned: 0` を報告する。

## Out of scope

Runtime reuse protocol、product build、Release tag と asset、historical Work Item、
無関係な cleanup。
