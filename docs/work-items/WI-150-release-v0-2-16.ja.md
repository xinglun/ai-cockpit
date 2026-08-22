---
author: AI Cockpit maintainers
title: "WI-150 — v0.2.16 Release baseline"
description: "v0.2.16 immutable Runtime Release の source、documentation、identity を整合させる。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-150-release-v0-2-16
workItemId: WI-150-release-v0-2-16
---

# WI-150 — v0.2.16 Release baseline

WI-150 は v0.2.16 Runtime の workspace metadata、lockfile、Release 文書、Release policy
check を整合させました。Runtime の検証 route が収束する間も、既存の Cargo check は CI の
shadow comparison として残しています。この Work Item は governance semantics を変更せず、
過去の record も書き換えません。

immutable な公開 Release は [v0.2.16](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.16) で、
tag commit `521177b` に bind されています。build、manifest、checksum、SBOM、provenance、smoke、
adopter、N-1 acceptance を含む Release workflow は
[workflow run 32602194567](https://github.com/xinglun/ai-cockpit/actions/runs/32602194567) に記録されています。

この Work Item の local verification evidence は `.ai/evidence/WI-150-release-v0-2-16.verification.json` です。
公開後の publication と installed Runtime acceptance は別の evidence であり、WI-151 が投影します。
この archived record は書き換えません。
