---
author: AI Cockpit maintainers
title: "WI-239 — v0.2.31 public Release adopter acceptance"
workItemId: WI-239-release-v0-2-31-adopter-acceptance
description: "Validate the immutable v0.2.31 Release against a fresh isolated adopter and bind the installed Runtime to this repository."
audience:
  - maintainer
  - reviewer
  - adopter
status: current
lastVerifiedBy: WI-239-release-v0-2-31-adopter-acceptance
authority: canonical
---

# WI-239 — v0.2.31 public Release adopter acceptance

This Work Item is the post-release acceptance boundary for v0.2.31. It uses
only the public Release archive for Runtime operations; source checkout and
workspace binaries are not Runtime fallbacks.

## Acceptance boundary

- The public tag is non-draft and non-prerelease, and the archive, manifest,
  and checksums agree on the immutable Release identity.
- A fresh adopter is created in isolated HOME, XDG_CONFIG_HOME, TMPDIR, and
  CARGO_HOME roots. Attach, profile confirmation, Agent doctor, repository
  identity, and isolation checks pass.
- `first-adopter-smoke` remains `not_ready`; the scaffold does not invent
  intent, scope, acceptance criteria, authority, approval, or completion.
- The second verification reuses evidence without spawning a new process, and
  the lifecycle reaches close with a structured human decision receipt.
- The temporary acceptance run root is validated and removed on success.
- The installed v0.2.31 binary passes explicit repository-bound inspect,
  status, doctor, and Agent doctor checks for this repository.

## Evidence

The acceptance receipt, runtime identity, release manifest, isolation manifests,
verification reuse outputs, lifecycle evidence, and installed-runtime checks
are archived under `.ai/evidence/WI-239-release-v0-2-31-adopter-acceptance/`.

## References

- [Release distribution](../release/distribution.md)
- [Outcome report](../reference/outcome-report.md)
- [Agent workflow](../reference/agent-workflow.md)
