---
author: AI Cockpit maintainers
title: WI-404 — Release documentation terminal promotion
description: Promote completed Work Item documentation only after immutable lifecycle evidence is present.
workItemId: WI-404-release-docs-terminal-promotion
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-404-release-docs-terminal-promotion
---

# WI-404 — Release documentation terminal promotion

This Work Item repairs the documentation projection exposed by the v0.2.41
release quality gate. It promotes the completed WI-402 documentation and its
tri-language parity rows without rewriting any immutable `.ai` evidence or
decision bytes.

## Boundary

- Update only the WI-402 tri-language Work Item pages and the tri-language
  reference parity rows.
- Keep archive, verification, finalization, and close records as immutable
  evidence references.
- Do not change Runtime semantics or publish a Release.

## Verification

The installed Runtime records the repository-bound verification evidence. The
documentation, promotion, parity, inventory, and full workspace checks must
pass before review.
