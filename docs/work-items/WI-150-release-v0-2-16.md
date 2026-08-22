---
author: AI Cockpit maintainers
title: "WI-150 — v0.2.16 release baseline"
description: "Prepare the v0.2.16 immutable Runtime release and keep source, documentation, and release identity aligned."
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-150-release-v0-2-16
workItemId: WI-150-release-v0-2-16
---

# WI-150 — v0.2.16 release baseline

WI-150 aligned the workspace metadata, lockfile, release documentation, and
release-policy checks for the v0.2.16 Runtime. It deliberately kept the CI
Cargo checks as a shadow comparison while the Runtime verification route
continues to converge; it did not change governance semantics or repository
history.

The immutable public Release is [v0.2.16](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.16),
bound to tag commit `521177b`. The complete release workflow, including build,
manifest, checksums, SBOM, provenance, smoke, adopter, and N-1 acceptance, is
recorded in [workflow run 32602194567](https://github.com/xinglun/ai-cockpit/actions/runs/32602194567).

The Work Item's local verification evidence is
`.ai/evidence/WI-150-release-v0-2-16.verification.json`. Public publication
and installed-runtime acceptance are separate post-release evidence; they are
projected by WI-151 rather than rewriting this archived record.
