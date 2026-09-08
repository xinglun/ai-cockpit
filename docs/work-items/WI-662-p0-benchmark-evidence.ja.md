---
author: AI Cockpit maintainers
title: WI-662 — P0 benchmark evidence
description: Runtime 最適化の前に、取得順序を保持し証拠に結び付いた性能 benchmark を確立する。
audience: [maintainer, reviewer, adopter]
workItemId: WI-662-p0-benchmark-evidence
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-662-p0-benchmark-evidence
terminalArchive: .ai/work-items/archive/WI-662-p0-benchmark-evidence.contract.json
terminalVerification: .ai/evidence/WI-662-p0-benchmark-evidence.verification.json
terminalFinalization: .ai/decisions/WI-662-p0-benchmark-evidence.finalize.json
terminalDecision: .ai/decisions/WI-662-p0-benchmark-evidence.close.json
---

# WI-662 — P0 benchmark evidence

[English](WI-662-p0-benchmark-evidence.md) · [简体中文](WI-662-p0-benchmark-evidence.zh-CN.md)

## Intent

Calibrated Human-Agent Trust、evidence validity、repository isolation、authorization boundary、recovery capability を維持しながら、request 受付から信頼できる governance decision までの latency、CPU、I/O、memory cost を調べるため、監査可能で比較可能な AI Cockpit 性能測定を確立する。

## Boundary

この Work Item は benchmark、comparator、test fixture、性能説明、Work Item record、reference parity、plan のみを変更する。cold/warm grouping を修正し schema 2 measurement evidence を確立するが、production Runtime governance behavior は変更せず、P1-P3 最適化、resident cache、in-process Git、PGO、incremental digest、新しい coordinator は導入しない。

## Acceptance

- 固定列 `120, 20, 22, 21` を取得順のまま先に分け、最初の測定値を `120` とする。
- raw sample、warmup 数、sample 数、quantile method、最初の CLI、OS cache warmup 後の独立 CLI、resident MCP の境界を明記し、sample 不足時に信頼できる p95/p99 を主張しない。
- baseline/candidate は異なる Runtime identity を使えるが、各 evidence、repository snapshot、environment の完全性と比較可能性を検証し、既存 schema 1 の negative gate coverage を保持する。
- phase timing、scenario matrix、実 read/hash bytes、Git call 数、process 数、peak memory を取得できない場合は unavailable と明示し、zero にはしない。

## Evidence boundary

この文書は Work Item と measurement contract を登録するだけで、性能向上を主張しない。sanity output は evidence structure の確認専用であり、release acceptance には同一 environment、同一 scenario、十分な sample、既定の published binary が必要である。

## Verification

Contract と plan に定義した focused tests、documentation acceptance、parity check、governance gate、`git diff --check` を実行する。完了前に verified または closed と表示しない。
