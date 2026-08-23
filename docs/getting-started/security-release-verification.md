---
author: AI Cockpit maintainers
title: "Security and release verification"
description: "What AI Cockpit release evidence proves and where external responsibility begins."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
---

# Security and release verification

Use [Release and distribution](../release/distribution.md) for the executable
commands and current immutable baseline. Keep these evidence types separate:

| Evidence | Supports | Does not prove |
| --- | --- | --- |
| Stable provider Release | The named assets are publicly available | Their digest or source is correct |
| Git tag | An immutable source reference exists | A stable provider Release exists |
| `SHA256SUMS` and manifest | The selected artifact matches published bytes and metadata | Who approved the release |
| Provider attestation | The provider statement binds the artifact subject | Enterprise compliance or safe execution |
| SBOM | Components are inventoried | Absence of vulnerabilities or build provenance |
| Adopter acceptance receipt | The pinned public binary completed the bounded harness | Every target, stack, or organization policy passed |

Missing, stale, foreign, or contradictory evidence is not a pass. The Runtime
records repository and executable identity, but external providers and people
remain responsible for publication, identity, branch protection, private
mirrors, incident policy, and enterprise assurance.

For ordinary adoption, verify the public artifact first, then attach the target
repository. For maintainer-side post-release checks, use only the published
binary and preserve failed acceptance as failed historical evidence; never
replace it with a workspace build.

[Strict installation security](installation-security.md) | [中文](security-release-verification.zh-CN.md) | [日本語](security-release-verification.ja.md)
