---
author: AI Cockpit maintainers
workItemId: WI-903-precheckpoint-contract-scope
title: checkpoint 前 Contract scope 追加
description: 必須の初期 projection が漏れた場合に、checkpoint または verification 前の active Work Item が Runtime 経由で scope を追加できるようにする。
audience: [maintainer, reviewer, contributor]
status: implemented
authority: human:xinglun
lastVerifiedBy: WI-903-precheckpoint-contract-scope
terminalArchive: .ai/work-items/archive/WI-903-precheckpoint-contract-scope.contract.json
terminalVerification: .ai/evidence/WI-903-precheckpoint-contract-scope.verification.json
terminalDecision: .ai/decisions/WI-903-precheckpoint-contract-scope.close.json
---

# WI-903 — checkpoint 前 Contract scope 追加

本 WI は WI-902 の開始時に見つかった lifecycle の行き詰まりを修正する。
Runtime は checkpoint 前に宣言済みの文書 projection を要求していた一方、
checkpoint が存在しない段階では唯一の scope 追加入口を拒否していた。
修正は最初の checkpoint と verification result より前の追加 scope に限定する。
identity、authority、base revision、mode、既存の acceptance criteria、および
checkpoint 後の evidence boundary は変更できない。

## 受け入れと証拠

- fixture は checkpoint 前に不足 path を追加し、通常の preflight/checkpoint を通過する。
- checkpoint 後 amendment の revalidation は変わらない。
- 不変 Contract field の変更は引き続き拒否される。
