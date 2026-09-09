---
author: AI Cockpit maintainers
title: "WI-719 — P1 大容量変更ファイルのストリーミング hash 実験"
description: "証拠と fail-closed ガバナンスを弱めずに大容量変更ファイルの streaming hash を計測し、効果がある場合だけ採用を検討する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-719-p1-streaming-hash
status: measured_declined
authority: human:repository-owner
lastVerifiedBy: WI-719-p1-streaming-hash
terminalArchive: .ai/work-items/archive/WI-719-p1-streaming-hash.contract.json
terminalVerification: .ai/evidence/WI-719-p1-streaming-hash.verification.json
terminalFinalization: .ai/decisions/WI-719-p1-streaming-hash.finalize.json
terminalDecision: .ai/decisions/WI-719-p1-streaming-hash.close.json
---

[English](WI-719-p1-streaming-hash.md) · [简体中文](WI-719-p1-streaming-hash.zh-CN.md)

# WI-719 — P1 大容量変更ファイルのストリーミング hash 実験

## 結論

streaming hash candidate は計測後に不採用とした。candidate は rollback 済みで、production の挙動は変わっていない。North Star は Calibrated Human-Agent Trust のままである。これは証拠に基づく見送りであり、安全性の否定ではない。

## 仮説と境界

candidate は変更ファイルの全体 `fs::read` を chunked hash と上限付き text capture に置き換える案だった。完全な read が成功した場合だけ digest を確定し、既存の raw digest byte、`MAX_CHANGE_TEXT_BYTES` 境界、fail-closed read failure を保持する設計である。実験範囲は `crates/cockpit-git` に限定し、cache、IncrementalMerkle、認可、証拠 binding、recovery、Outcome、release acceptance は変更していない。

## ペア計測

baseline と candidate は同じ base revision `caa6ddffcc1847c9a1161e1d8aa414f1acabd11e` から別々に build し、Runtime identity を個別に binding した。repository snapshot、macOS arm64/10 CPU、tracked file 10,331、tracked bytes 80,274,774、historical Work Item 621、通常パスの 16 MiB 変更ファイル 1 個を揃えた。各 run は raw order、first measurement、OS-cache warmup 1 回、warm 20 samples、nearest-rank を保存し、p99 は unavailable とした。

| command | baseline warm p50/p95 ms | candidate warm p50/p95 ms | candidate 差分 |
| --- | ---: | ---: | ---: |
| status | 2,895.183 / 3,346.259 | 2,965.996 / 3,043.153 | +2.445% / −9.057% |
| inspect | 133.806 / 143.756 | 134.867 / 139.398 | +0.793% / −3.036% |
| doctor | 52.129 / 54.041 | 52.776 / 54.804 | +1.241% / +1.412% |
| observe | 188.036 / 204.221 | 186.403 / 207.658 | −0.869% / +1.684% |

Contract は `status` の p50 と p95 の双方に 5% 以上の改善を要求していた。candidate は p50 を満たさず、budget gate は `budget_exceeded:status:2965.996>2739.035` を報告した。この host では信頼できる filesystem type/comparison key も取得できず、comparator は fail closed した。Runtime 内部の read bytes、hashed bytes、Git call、child process、peak memory は unavailable 理由付きで記録し、ゼロにはしていない。

raw evidence は `.ai/evidence/external/WI-719-p1-streaming-hash.baseline.large-file.json` と `.ai/evidence/external/WI-719-p1-streaming-hash.candidate.large-file.json`、binding decision と gate 結果は `.ai/evidence/external/WI-719-p1-streaming-hash.experiment-summary.json` に保存した。

## 正確性とガバナンス

rollback 前の candidate focused test は大容量 capture boundary と raw digest byte について pass した。Runtime/binary identity だけを除いて正規化した source baseline と candidate の `inspect`、`status`、`doctor`、`observe` 出力、error、exit code は一致した。最終 tree では既存の repository と IncrementalMerkle test を保持して pass させ、production optimization は採用せず、性能向上も主張しない。

## 次の対応

次の性能 WI は、計測済みの大規模 history `status` bottleneck を優先する。信頼できる resource または latency benefit が別の隔離計測で示されない限り、大容量 streaming hash は再開しない。resident MCP、polling、parallel read、coordinator、cache、in-process Git、PGO は本 WI の範囲外である。

## 検証の制限

documentation acceptance と Work Item status consistency は pass した。repository-wide の
`promote_closed_work_item.py --check-all` は、並行 WI-717 の projection に必要な通常の Markdown ファイルがないため fail closed した。独立した WI-717 の問題は
`.ai/evidence/external/WI-719-p1-streaming-hash.validation-limitation.json` に保存し、本 WI では変更していない。
