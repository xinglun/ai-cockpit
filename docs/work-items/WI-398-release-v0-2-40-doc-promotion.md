---
author: AI Cockpit maintainers
title: "WI-398 — v0.2.40 documentation promotion"
description: "Promote the closed v0.2.40 release-preparation documentation from immutable Runtime evidence."
workItemId: WI-398-release-v0-2-40-doc-promotion
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-398-release-v0-2-40-doc-promotion
capabilityClaims: [documentation_governance, release_distribution]
---

# WI-398 — v0.2.40 documentation promotion

[简体中文](WI-398-release-v0-2-40-doc-promotion.zh-CN.md) · [日本語](WI-398-release-v0-2-40-doc-promotion.ja.md)

## Intent

Promote the closed WI-397 documentation to an auditable terminal projection
before the v0.2.40 release tag is created. The promotion consumes immutable
archive, verification, finalization, and close records; it does not rewrite
those records.

## Boundary

This Work Item updates only the tri-language WI-397 document and parity ledger
status and terminal links, plus the WI-397 close/finalization receipts needed
for the reviewed delivery. Runtime behavior, release implementation, and
public adopter acceptance remain outside this boundary.

## Verification

The promotion script, documentation acceptance, status consistency, governance
integrity, and diff checks must pass before merge. Public binary and adopter
acceptance are owned by the successor release-adopter Work Item.
