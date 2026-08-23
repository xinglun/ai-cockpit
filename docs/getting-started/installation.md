---
author: AI Cockpit maintainers
title: "Install AI Cockpit"
description: "Install and verify the shared Runtime without attaching a repository implicitly."
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
---

# Install AI Cockpit

AI Cockpit is one shared external Runtime, not a governance tree copied into
every project. Follow [Release and distribution](../release/distribution.md) to
select an immutable public Release, download the artifact for the exact target,
and verify its SHA-256 before installing it.

Confirm the installed executable separately:

```bash
ai-cockpit --version
```

Installation alone does not create `.ai/`, choose project quality commands,
install an Agent adapter, prove hosted CI, or make a repository production
ready. Those are separate, reviewable repository actions.

After installation, use the read-only-first route:

```bash
repo=/path/to/repository
ai-cockpit inspect --repo "$repo"
ai-cockpit attach --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

Review all reported facts. Then continue with [First calibration](first-calibration.md)
and [Adopter configuration](adopter-configuration.md). A private mirror or
local source checkout is not public Release evidence; see
[Strict installation security](installation-security.md).

[Getting started](README.md) | [中文](installation.zh-CN.md) | [日本語](installation.ja.md)
