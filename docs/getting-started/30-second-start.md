---
author: AI Cockpit maintainers
title: "30-second start"
description: "The shortest safe path from an installed Runtime to an attached repository."
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# 30-second start

Use an immutable published Runtime that you already verified. Then inspect the
repository before allowing the first repository-local write:

```bash
repo=/path/to/repository
ai-cockpit inspect --repo "$repo"
ai-cockpit attach --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

`inspect` is the read-only first look. `attach` creates only repository-owned
`.ai/` state; it does not install Agent instructions or edit global MCP
configuration. Stop if the repository is not the intended Git checkout, the
worktree has unexplained changes, or `doctor` is not `ok`.

Next, follow [First calibration](first-calibration.md), then run the
[first Work Item](first-work-item.md). For binary installation and digest
checks, start with [Installation](installation.md).

[Getting started](README.md) | [中文](30-second-start.zh-CN.md) | [日本語](30-second-start.ja.md)
