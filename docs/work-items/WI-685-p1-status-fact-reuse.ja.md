---
author: AI Cockpit maintainers
title: "WI-685 — P1 status 事実の再利用"
description: "多数の履歴 Work Item を持つ repository で、1 回の status 観測内に immutable な finalization transition 事実を再利用します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-685-p1-status-fact-reuse
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-685-p1-status-fact-reuse
terminalArchive: .ai/work-items/archive/WI-685-p1-status-fact-reuse.contract.json
terminalVerification: .ai/evidence/WI-685-p1-status-fact-reuse.verification.json
terminalFinalization: .ai/decisions/WI-685-p1-status-fact-reuse.finalize.json
terminalDecision: .ai/decisions/WI-685-p1-status-fact-reuse.close.json
---

[English](WI-685-p1-status-fact-reuse.md) · [简体中文](WI-685-p1-status-fact-reuse.zh-CN.md)

# WI-685 — P1 status 事実の再利用

## Intent

履歴 Work Item が多い repository の `status` で発生する finalization
transition の重複 scan と parse を削減しつつ、Calibrated Human-Agent Trust、
repository/WI isolation、evidence binding、authorization boundary、recovery を維持します。

## Boundary

候補は request-scoped です。`status_with_runtime` が現在の観測だけに束縛された
immutable `FinalizationTransitionIndex` を 1 つ作り、historical finalization
projection に渡します。path、content digest、parsed value、fail-closed な
read/parse/digest error を保持します。cross-request cache は導入せず、snapshot
boundary、evidence、Runtime、MCP、doctor、scheduler、IncrementalMerkle、large-file、
parallel-read、P3 path は変更しません。

shared resolver は indexed candidate を消費する前に canonical receipt を読み、
既存の canonical error precedence を保ちます。malformed、missing、fork、stale、
digest mismatch、symlink、path anomaly は引き続き fail-closed です。

## Hypothesis and evidence

対象 repository には canonical finalization receipt が 395 件、transition file が
156 件あります。変更前は old-runtime receipt ごとに decisions directory を再 scan
し、1 回の status observation で 396 回の directory enumeration になっていました。
候補は canonical inventory と transition index の 2 回だけを列挙します。

計数証拠は
`.ai/evidence/external/WI-685-p1-status-fact-reuse.observation-counts.json` にあります。

## Measurement Contract and result

受入れ前に Contract を更新・再検証しました。warm independent CLI sample は 20 件以上、
`status` p50 は 10%以上、p95 は 5%以上改善し、non-target p95 の回帰は 25% と 20ms の
両方を超えないことを条件にしました。first measurement、OS cache warmup、sample count、
percentile reliability、environment、raw samples、unavailable metrics を記録し、true cold
cache や resident MCP は主張しません。

| Paired run | status baseline p50/p95 ms | status candidate p50/p95 ms | delta |
| --- | ---: | ---: | ---: |
| baseline → candidate | 2058.544 / 2165.179 | 1565.100 / 1596.771 | -23.98% / -26.25% |
| candidate → baseline | 2051.134 / 2104.867 | 1559.946 / 1653.997 | -23.94% / -21.43% |

同一時間帯の 2 回では inspect、doctor、observe も non-target budget 内でした。別の
pre-box round40 candidate run は unrelated path と同時に遅くなりました。host load を取得できない
ため、その unpaired result は adverse evidence として保存し、受入れの根拠には使いません。

raw evidence と summary は `.ai/evidence/external/WI-685-p1-status-fact-reuse.*.json` にあります。

## Correctness and isolation

- focused transition suite 29 件、status projection suite 11 件が pass しました。
- TDD regression test で canonical error precedence の変更を先に検出し、canonical
  receipt を先に観測する resolver refactor 後に pass しました。
- clean、single-file change、malformed transition の CLI status で exit code は同じ 0、
  `.compatibility.runtimeDigest` だけを除いた output SHA-256 は baseline と candidate で一致しました。
- 独立した 2 observation で fixture は各 1 回 parse され、index は observation 終了後に破棄されます。

詳細は `output-parity.json`、raw status JSON、`preflight-negative-tests.json` を参照してください。

## Limits and governance state

resident MCP と残りの scenario matrix は未測定です。Runtime phase timing、actual read/hashed
bytes、internal Git calls、runtime child-process count、cache invalidation events、peak memory
はこの platform では unavailable であり、0 には置き換えていません。host CPU contention を
測定できないため round40 は別分類で保持します。

この Work Item は `in_progress` です。Runtime verify、hosted PR review、merge、archive、close、
documentation promotion が残っており、PR や最終 governance decision はまだ主張していません。
