---
author: AI Cockpit maintainers
title: "WI-139F — Runtime v0.2.13 release acceptance"
description: "Release the merged recovery and adopter-acceptance controls as an immutable public Runtime."
audience:
  - maintainer
status: active
authority: repository-local
lastVerifiedBy: pending-release-evidence
workItemId: WI-139F-runtime-v0-2-13
---

# WI-139F — Runtime v0.2.13 release acceptance

This Work Item publishes the current merged Runtime as `v0.2.13`. Completion
requires immutable public Release artifacts, public fresh-adopter acceptance,
N-1 upgrade acceptance from `v0.2.12`, and installation checks on this
repository. Acceptance must use downloaded public binaries only; source builds
are not a substitute for Release evidence.

The release receipt binds the tag, archive digest, binary digest, platform,
Runtime identity, adopter repository identity, isolation manifests, cleanup
result, and lifecycle evidence. Post-release failure records failed acceptance
without changing the published Release truth.
