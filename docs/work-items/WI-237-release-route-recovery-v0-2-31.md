---
author: AI Cockpit maintainers
title: "WI-237 — release route recovery and v0.2.31 publication"
workItemId: WI-237-release-route-recovery-v0-2-31
description: "Repair the clean-batch release route and publish the next immutable patch release without rewriting v0.2.30 history."
audience:
  - maintainer
  - reviewer
  - adopter
status: current
authority: canonical
lastVerifiedBy: WI-237-release-route-recovery-v0-2-31
---

# WI-237 — release route recovery and v0.2.31 publication

This Work Item repairs the release quality route exposed by the clean-batch
boundary: a repository with no active Work Item directory is valid, but the
release workflow must not fail while discovering contracts. The immutable
v0.2.30 tag and its failed publication attempt remain historical facts and are
not rewritten. The corrected route publishes v0.2.31, after which a successor
Work Item owns public adopter and N-1 acceptance.

## Acceptance boundary

- Release route planning handles an absent `.ai/work-items/active` directory
  deterministically, with a regression covering the zero-active boundary.
- Package metadata, lockfile, release docs, and tri-language parity identify
  v0.2.31; v0.2.30 remains preserved as failed immutable history.
- Hosted release checks publish only immutable v0.2.31 artifacts.
- Public v0.2.31 artifact identity, installed-runtime checks, and isolated
  adopter/upgrade acceptance are handed to a successor Work Item.

## References

- [Release and Distribution](../release/distribution.md)
- [Versioning](../architecture/versioning.md)
- [Reference parity ledger](../reference/reference-parity.md)
