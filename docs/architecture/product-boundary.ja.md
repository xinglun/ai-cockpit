---
author: AI Cockpit maintainers
title: "製品境界"
description: "AI Cockpit の責任範囲と adopter/provider に残る責任。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - product_boundary
---

# 製品境界

## 製品 identity

AI Cockpit は AI 支援開発のための Repository Governance Layer です。North Star
は calibrated human-agent trust、核心ルールは Evidence over Self-Declaration
です。

統治チェーンは次のとおりです。

```text
Evidence → Governance Decision → Human Control
```

## 対象範囲

- deterministic な repository observation
- bounded な Work Item Contract
- scope、authority、evidence、lifecycle decision
- fail-closed verification planning と evidence reuse
- repository 内の facts、decisions、evidence、knowledge projection
- CLI と read/verify MCP adapter

## 明示的な対象外

AI Cockpit は Agent Runtime、Workflow Engine、Security Sandbox、一般的な
prompt-injection detector、identity provider、compliance certificate、または
human review の代替ではありません。Provider identity、branch protection、
production isolation、署名、SBOM、provenance、enterprise policy は外部
evidence または adopter の責任です。

## アーキテクチャ制約

- binary path から Runtime root を推測しない。
- runtime code を対象 repository にコピーしない。
- Repository Protocol version と Runtime version を分離する。
- MCP と CLI は同じ application service を呼び、統治ルールを所有しない。
- human decision は workflow の問いを解決できるが、未検証 check を pass にしない。
