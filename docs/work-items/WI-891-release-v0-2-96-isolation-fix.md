---
author: AI Cockpit maintainers
workItemId: WI-891-release-v0-2-96-isolation-fix
title: Release v0.2.96 verification isolation fix
description: Repair explicit verification target-directory propagation exposed by the failed immutable v0.2.95 candidate.
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-891-release-v0-2-96-isolation-fix
---

# WI-891 — Release v0.2.96 verification isolation fix

This successor preserves the immutable v0.2.95 failed-candidate history and
repairs the Runtime environment boundary before publishing v0.2.96. The
change must keep an explicit `CARGO_TARGET_DIR` supplied by an adopter
acceptance root when verification uses a pinned executable identity.

## Acceptance boundary

- Keep v0.2.95 immutable and do not modify object repositories.
- Review and merge the Runtime fix through one PR with hosted checks.
- Publish v0.2.96 only after candidate and downloaded-artifact installation and
  N-1 upgrade acceptance prove HOME, XDG_CONFIG_HOME, CARGO_HOME, and target
  roots remain isolated and are cleaned.
- Preserve unknown host-display and performance claims; this WI only repairs
  the release isolation blocker.

## Verification plan

Run the targeted repository environment tests with `CARGO_INCREMENTAL=0` and
the shared verification target, then the declared release and documentation
gates. Keep the failed v0.2.95 workflow receipt as external evidence and do
not rerun validation merely to deliver a message.

