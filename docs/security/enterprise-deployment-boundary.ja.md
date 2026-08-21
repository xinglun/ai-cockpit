---
author: AI Cockpit maintainers
title: "Enterprise Deployment Boundary"
description: "Shared Runtime と repository-local Protocol の提供範囲、enterprise が外部に用意する範囲。"
audience:
  - adopter
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - enterprise_deployment_boundary
---

# Enterprise Deployment Boundary

machine または toolchain ごとに shared `ai-cockpit` Runtime を 1 つ install します。各 repository は明示的に
attach し、repository identity、Contract、evidence、knowledge、adapter は repository ごとの `.ai/` に分離します。

AI Cockpit は typed contract、bounded verification、fail-closed reuse、repository-local record、export 可能な
audit event を提供します。global identity provider、OS sandbox、branch protection、production change control、
secret manager、enterprise SIEM、WORM retention、signature service、SBOM generator、provenance authority、
organization-wide approval directory は提供しません。

Adopter は delegated evidence で external control を Work Item に bind し、data classification、retention、disposal、
export rule を定義します。external evidence が存在し valid に bind されていなければ、local green decision は external
control の proof ではありません。
