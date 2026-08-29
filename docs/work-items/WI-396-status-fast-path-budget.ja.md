---
workItemId: WI-396-status-fast-path-budget
title: "status fast path と厳格な performance budget"
author: AI Cockpit maintainers
description: "clean snapshot の冗長 subprocess を削減し、性能主張を identity-bound と fail-closed に保つ。"
type: implementation
audience: [adopter, contributor, maintainer, reviewer]
authority: human-authorized
status: in_progress
lastVerifiedBy: WI-396-status-fast-path-budget
---

# WI-396 — status fast path と厳格な performance budget

[English](WI-396-status-fast-path-budget.md) · [简体中文](WI-396-status-fast-path-budget.zh-CN.md)

## Intent

WI-395 後の Rust performance convergence を継続する。clean repository
snapshot は同値な diff が空であることを示すため、冗長な Git subprocess を
省略できる。dirty または不確実な入力では完全な patch inspection と同じ
governance facts を維持する。

## Boundary

benchmark は宣言したローカル platform の release/installed Runtime を対象と
する。status `<50 ms` と medium observation `<100 ms` は明示的な目標であり、
未達は bounded gap または failed budget として記録し、verification を弱めて
隠してはならない。Runtime と repository identity を常に記録する。

Runtime は共有 external binary のままで、adopter は明示的な `--repo` で
bind し、repository ごとに独立した `.ai/` state を保持する。global cache、
current repository、provider/enterprise 性能主張、reference installer/Make/
Python/V1 のコピーは導入しない。

## Verification

locked workspace tests、Git snapshot regression、performance fixture、
identity-bound regression gate、documentation gates、`git diff --check` を
実行する。最終 evidence には command、sample/median、`gitCalls`、Runtime
digest、repository identity を含める。
