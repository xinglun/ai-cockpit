---
author: AI Cockpit maintainers
title: "WI-286 — Rust Agent Risk と checkpoint evidence の境界"
workItemId: WI-286-agent-risk-checkpoint-evidence
description: "Reference の Agent Risk と checkpoint 制御を、typed な request-scoped Rust lifecycle 境界へ移行する。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-286-agent-risk-checkpoint-evidence
terminalArchive: .ai/work-items/archive/WI-286-agent-risk-checkpoint-evidence.contract.json
terminalVerification: .ai/evidence/WI-286-agent-risk-checkpoint-evidence.verification.json
terminalFinalization: .ai/decisions/WI-286-agent-risk-checkpoint-evidence.finalize.bd7963be356babe9075d0f5451851b1cb12d4361b64918feb8bd1072ef85db94.json
terminalDecision: .ai/decisions/WI-286-agent-risk-checkpoint-evidence.close.json
authority: canonical
---

# WI-286 — Rust Agent Risk と checkpoint evidence の境界

この bounded parity batch は、Python script、Make target、provider-global
設定をコピーせず、reference の Agent Risk と checkpoint semantics を Rust
Runtime に移行します。対象は typed strict な
`checkpointPolicy`/`checkpointEvidence`、intent/scenario route enforcement、
required verification declaration、合法な unknown path、append-only の
Contract amendment revalidation です。

`before_edit` は不変です。verification 開始後の Contract amendment は前後の
hash、理由、無効化した check を記録し、resume history は古い checkpoint
evidence を stale とします。terminal transition 前には fresh preflight と
verification が必要です。`light`、`standard`、`strict`、`release` は
Verification strength profile であり、Evidence Assurance を意味しません。

Contract の acceptance text は原言語を保持します。Human Outcome は固定表示
label のみを localize し、governance fact を翻訳しません。CI integration、
planner/performance、release harness、大規模 module 分割は別の bounded batch
です。

## Reference 対応

| Reference の責任 | Rust boundary |
| --- | --- |
| `ai_check_agent_risk.py` | `validate_agent_risk_controls` と lifecycle gate の共有 |
| `ai_checkpoint.py` | typed `CheckpointPolicy`、`CheckpointEvidence`、`revalidate_contract_amendment` |
| intent/scenario route binding | command 実行前の `resolve_verification_route` |
| static Agent-rule parity | Rust `agent_rule_parity` regression test |

## 受入境界

- malformed、unknown-field、duplicate、foreign、stale、contradictory、symlink
  checkpoint input は fail closed する。
- required verification gate の欠落または失敗は finish/archive に進めない。
- amendment と resume history は stale evidence を再利用できない。
- adopter repository は明示的な repository context と isolation を維持する。
- 英語、簡体字中国語、日本語の文書で semantic（wire byte ではない）parity
  boundary を説明する。
