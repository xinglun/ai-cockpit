---
author: AI Cockpit maintainers
title: "WI-195 — Governance integrity recovery gate"
description: "dynamic governance inventory を recovery-aware にし、public adopter isolation receipt を harden します。"
audience:
  - maintainer
  - reviewer
workItemId: WI-195-governance-recovery-gate
status: historical
authority: canonical
lastVerifiedBy: WI-196-governance-recovery-gate-retry
---

# WI-195 — Governance integrity recovery gate

これは current batch で使用した corrective Work Item です。dynamic governance gate は有効な
superseded predecessor を `recovered` history として受け入れ、malformed、foreign、missing
recovery は fail-closed のままにします。public adopter と N-1 acceptance harness は source
repository identity を bind し、すべての receipt write を検査し、identity-safe validation
後だけ temporary run root を削除します。

Recovery は approval、verification、merge authorization ではありません。blocked predecessor
bytes は immutable な red のまま保持し、successor は Contract、evidence、hosted PR、closure
lifecycle を独立して完了する必要があります。

finish evidence 記録後に同じ scope の parity correction が見つかりました。WI-195 は
immutable な recovered history として保持し、fresh delivery は WI-196 で継続します。
修正済み Release と immutable public-artifact acceptance の後、reference source の
file-by-file 比較を次の batch として開始します。

[English](WI-195-governance-recovery-gate.md) ·
[简体中文](WI-195-governance-recovery-gate.zh-CN.md)
