---
author: AI Cockpit maintainers
workItemId: WI-925-current-performance
title: 現行版 object repository の performance measurement
description: object repository を変更せずに現行 Runtime と開発サイクルを再測定する。
audience: [adopter, contributor, maintainer]
status: implemented
authority: human:xinglun
lastVerifiedBy: WI-925-current-performance
terminalArchive: .ai/work-items/archive/WI-925-current-performance.contract.json
terminalVerification: .ai/evidence/WI-925-current-performance.verification.json
terminalDecision: .ai/decisions/WI-925-current-performance.close.json
---

[English](WI-925-current-performance.md) · [简体中文](WI-925-current-performance.zh-CN.md)

# WI-925 — 現行版 object repository の performance measurement

この Work Item は現行版の証拠を作成するもので、最適化の効果を前提にしない。
インストール済み Runtime と同じ Rust/Cargo toolchain を使い、ORG-X、sentinel、
goods-garden、ai-investigation-orchestrator の隔離 view を read-only で観測する。
それぞれの main/default branch と既存 working tree の bytes は変更しない。

## Acceptance boundary

- 各 release-grade operation は 100 個以上の valid warm sample を持つ。99 個以下は
  diagnostic-only とし、release-grade comparator は拒否する。
- raw sample、p50/p95/p99、Runtime/binary/repository/toolchain/environment identity、
  execution/reuse counter、unavailable または invalidation reason を保持する。
- Runtime latency と Contract→reviewable PR、verification→finish、merge 後 cleanup
  cost は分けて報告する。lifecycle data がない場合は `unknown` とし、zero にしない。
- paired comparison が凍結した noise budget を越えた場合だけ speedup を報告し、それ
  以外は `within_noise` または `unknown` のままにする。

## Verification boundary

`CARGO_INCREMENTAL=0` と共有 verification target directory を使う。collection 前に
format、scope、projection の安価な check を行う。各 object repository の branch、HEAD、
working tree を前後で確認し、raw capture を Work Item evidence に保存する。この Work
Item は object repository を変更せず、Runtime code を最適化せず、release を公開しない。

development-cycle report は、実測した interval と Runtime が観測できない metric
（agent operation count や provider cleanup timestamp など）を分ける。最終 human
Outcome では、まだ証明されていない benefit を unknown として明示する。
