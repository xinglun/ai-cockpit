---
author: AI Cockpit maintainers
title: "WI-435 — local reference inventory rebaseline"
workItemId: WI-435-reference-inventory-rebaseline-local
description: "維持される local semantic reference に file-level ledger を再バインドし、変更された source の判断を静かに昇格しない。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-435-reference-inventory-rebaseline-local
terminalArchive: .ai/work-items/archive/WI-435-reference-inventory-rebaseline-local.contract.json
terminalVerification: .ai/evidence/WI-435-reference-inventory-rebaseline-local.verification.json
terminalFinalization: .ai/decisions/WI-435-reference-inventory-rebaseline-local.finalize.json
terminalDecision: .ai/decisions/WI-435-reference-inventory-rebaseline-local.close.json
---

# WI-435 — local reference inventory rebaseline

この Work Item は、`AI_COCKPIT_REFERENCE_ROOT` で選択する maintainer 提供の local checkout に
file-level comparison ledger を明示的に再バインドします。source は commit
`fde3380f81fea5fd2e288f7a8849f737dc074060` に固定し、public reference repository は必要としません。
これは inventory と documentation の変更であり、semantic comparison batch や source content の copy ではありません。

[English](WI-435-reference-inventory-rebaseline-local.md) · [简体中文](WI-435-reference-inventory-rebaseline-local.zh-CN.md)

## Scope と安全境界

- current の 4,450 tracked path、160 changed path、以前の ledger から retired になった 669 path を記録します。
- 以前の各判断を historical として保持し、changed non-history record は file-by-file review まで
  `deferred-next-batch` のままにします。
- previous source commit と manifest digest を復元可能にし、machine ledger、lock、tests、三言語 documentation を一致させます。
- reference file の copy、Rust Runtime behavior の変更、CI policy の変更、source update からの governance decision 推論は行いません。

現在の ledger は 3,681 generated-history、223 implemented-different-by-design、1 implemented-equivalent、
4 not-applicable、62 reference-only、479 deferred-next-batch です。retired path は historical metadata であり、current parity claim ではありません。

## Verification boundary

local-source policy、旧 ledger regression、current ledger regression、documentation/parity check、workspace test が
すべて成功した場合だけ rebaseline を受け入れます。変更または削除された source path は ledger から見える必要があり、
checkout の欠落、moving commit、public-network fallback は失敗です。
