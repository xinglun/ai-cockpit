---
author: AI Cockpit maintainers
title: "WI-773 — P1 status canonical fact 再利用"
description: "status の canonical finalization receipt 事実再利用を測定し、事前登録した効果を満たさない候補を却下する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-773-p1-status-canonical-fact-reuse
lastVerifiedBy: WI-773-p1-status-canonical-fact-reuse
terminalArchive: .ai/work-items/archive/WI-773-p1-status-canonical-fact-reuse.contract.json
terminalVerification: .ai/evidence/WI-773-p1-status-canonical-fact-reuse.verification.json
terminalFinalization: .ai/decisions/WI-773-p1-status-canonical-fact-reuse.finalize.308053f58465bd212e3edb34f296a7628cb59422d0c3ccc2af112e3c75168201.json
terminalDecision: .ai/decisions/WI-773-p1-status-canonical-fact-reuse.close.json
---

# WI-773 — P1 status canonical fact 再利用

## Contract と境界

この Work Item は remote `origin/main` の revision `5a24d4c0df865ece469822dbdc0dcc36eda07d85`
から開始した。仮説は、`status` の historical finalization projection が canonical
`*.finalize.json` を inventory の外側で既に観測した後、同じ receipt を再読込・再解析している
というものだった。候補の再利用は一つの不変な status observation 内に限定し、request 間、repository 間、
変更前後、governance rule、その他の性能変更は対象外とした。

Contract には、候補を受け入れる条件として warm 独立 CLI `status` の p50 と p95 がともに
少なくとも 5% 改善し、governance output が同一で、非対象 budget を超えないことを事前登録した。

## 測定と判断

paired raw evidence は次の二つである。

- `tests/performance/fixtures/WI-773-status-canonical-fact-reuse-baseline.json`
- `tests/performance/fixtures/WI-773-status-canonical-fact-reuse-candidate.json`

両方とも同じ detached clean fixture、repository identity、HEAD、445 件の canonical
finalization receipt、651 件の archived Work Item contract を使用した。harness は raw 順序、
最初の測定値、独立 CLI の warmup 一回、warm 20 samples、nearest-rank 分位点、scenario facts、
unavailable reason を保持する。fixture の分類は `many-historical-wi` である。

| path | warm p50 | warm p95 | 判定 |
| --- | ---: | ---: | --- |
| baseline `status` | 1062.470 ms | 1095.271 ms | 参照 |
| candidate `status` | 1089.892 ms | 1723.202 ms | 却下 |

この paired run では candidate の p50 は約 2.58% 遅く、p95 は約 57.3% 遅かったため、Contract
の閾値を満たさない。candidate の production 変更は撤回し、governance behavior、evidence semantics、
repository isolation、authorization boundary、recovery behavior は変更していない。

この macOS host では filesystem type を信頼して取得できず、`stat -f %T` は `/` を返した。そのため
comparison key と filesystem comparability は unavailable と記録した。Runtime が公開しない phase timing、
read bytes、hashed bytes、Git call 数、child process 数、peak memory、cache invalidation reason は
unavailable のままとし、ゼロで埋めていない。resident MCP と concurrent validation はこの CLI harness の対象外である。

## Correctness と検証

candidate probe は一つの observation 内の fact path を使い、canonical head read wrapper が二度目に呼ばれない
ことを確認した。probe 自体は通過したが、paired end-to-end 結果が閾値未達だったため候補は却下した。保持する tree
には元の実装だけがあり、candidate helper は残していない。

保持した raw record には P0 gate を実行した。gate は unavailable filesystem comparison と、二回目の candidate
`status` p95 budget 超過を fail closed で扱った。これは測定制約と候補却下の証拠であり、gate を弱める理由ではない。
次の Work Item は新しい bottleneck profile 後に、主要な実測段階だけを独立して対象にする。

## Outcome

Outcome: 🟡 candidate 却下、測定 evidence は保持。issue 数は 1（効果不足と tail の不安定さ）。blocker はこの仮説から
受け入れ可能な production optimization が得られなかったこと。解決済みなのは、重複 read 仮説を分離・測定し、
fail-closed で却下したこと。risk は現在の status path が変わらず従来コストを保持すること。verification は raw paired
benchmark、P0 gate output、focused probe、撤回後の repository test。次は P0 scenario matrix から新しい独立 bottleneck
を選び、新しい Contract と evidence なしに本候補を復活させない。
