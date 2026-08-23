---
author: AI Cockpit maintainers
title: "Repository profile calibration"
description: "Confirm project-owned quality commands without guessing repository facts."
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - configuration
---

# Repository profile calibration

Attachment detects possible build systems but does not decide which command is
the repository's quality baseline. Inspect the current profile and ask the
Runtime for a read-only candidate:

```bash
repo=/path/to/repository
ai-cockpit status --repo "$repo"
ai-cockpit profile propose --repo "$repo"
```

The proposal is not an applied change. A repository owner must confirm the
working directory, executable, arguments, toolchain, credentials boundary,
coverage, and hosted CI counterpart. Do not infer a command only because a
manifest, project file, or wrapper exists.

After review, confirm one exact project-owned command. This example is for a
Rust repository whose owner selected `cargo test --workspace`:

```bash
ai-cockpit profile confirm --repo "$repo" --program cargo --args test,--workspace
ai-cockpit status --repo "$repo"
```

Another stack uses its own approved program and arguments. Calibration does not
install a toolchain, authenticate a provider, or prove hosted CI. Unknown facts
remain Unknown and block any claim that depends on them.

[First calibration](first-calibration.md) | [中文](calibration.zh-CN.md) | [日本語](calibration.ja.md)
