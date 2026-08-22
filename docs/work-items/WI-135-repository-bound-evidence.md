---
author: AI Cockpit maintainers
workItemId: WI-135-repository-bound-evidence
title: Repository-bound retention and close evidence
description: Bind retention metadata and close receipts to the repository and Work Item at every lifecycle boundary.
audience:
  - adopter
  - contributor
  - maintainer
status: implementation
authority: canonical
lastVerifiedBy: WI-135-repository-bound-evidence
---

# WI-135 — Repository-bound retention and close evidence

## Intent

Prevent a copied, malformed, or foreign retention policy or close receipt from being
accepted as current repository truth.

## Boundaries

- Retention policy schema version, repository identity, Work Item identity, timestamps,
  and retention values are validated before use.
- Embedded verification retention and the repository-local retention policy agree when
  both are present.
- Close receipts write and require the repository identity; missing or foreign receipts
  cannot promote an archived Work Item to `closed` or appear as a valid human decision.
- Historical evidence bytes remain immutable and are never rewritten by this WI.

## Acceptance

- Valid retention and close records remain readable.
- Foreign, missing, malformed, unknown-field, schema-mismatched, and cross-repository
  records fail closed in Outcome, MCP, finish, archive, close, status, and purge paths.
- Regression tests cover the repository and Work Item bindings and preserve legacy
  historical projection.

## Verification

The archived verification evidence and close decision are linked after the lifecycle is
complete. This WI does not introduce Task Report or recovery-state features.
