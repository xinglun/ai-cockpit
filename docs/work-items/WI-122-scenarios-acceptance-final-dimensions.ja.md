---
author: AI Cockpit maintainers
title: "WI-122 — Scenario・Acceptance・最終 dimensions の control"
description: "Contract/Summary governance projection の bounded validation と明示的な記録を追加する。"
audience:
  - adopter
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: work-item-acceptance
capabilityClaims:
  - scenario_coverage
  - acceptance_evidence
  - final_dimensions
---

# WI-122 — Scenario・Acceptance・最終 dimensions の control

WI-122 は read-only の Contract/Summary validator と bounded な `controls`
writer を追加します。高リスクの scenario coverage は fail-closed です。
番号のない legacy Acceptance は互換性を保ち、番号付き Acceptance は stable
ID と item ごとの evidence を使います。Intent alignment の unknown は明示的に
保持されます。

最終 acceptance receipt は参照源と同じ 20 dimensions を使用します。Runtime は
receipt の shape、identity、decision、GO prerequisite を検証しますが、provider・
enterprise・adopter evidence を合成しません。任意の `fourPillarProjection` は
明示された表示用 projection であり、`4D` field ではありません。

機能は `work-item validate`、`work-item controls`、repository-bound MCP tool
`work_item_validate` から利用できます。すべての command は明示的な repository
binding を要求します。
