---
author: AI Cockpit maintainers
title: "Getting started"
description: "Install the shared Runtime and attach the first repository safely."
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# Getting started

Use this route for a new adopter repository:

1. Follow [Release and distribution](../release/distribution.md) to install an immutable public Release and verify its digest.
2. Run `ai-cockpit inspect --repo /path/to/repository`, then `ai-cockpit attach --repo /path/to/repository`.
3. Run `ai-cockpit status --repo /path/to/repository` and `ai-cockpit doctor --repo /path/to/repository`.
4. Install an Agent adapter only when needed; `attach` does not edit Agent or global MCP configuration.
5. Create a `not_ready` skeleton with `ai-cockpit work-item new --repo /path/to/repository --id <id> --mode code`.
6. Review the read-only profile candidate and complete [First calibration](first-calibration.md).
7. Complete the external review/security/CI decisions in [Adopter configuration](adopter-configuration.md).
8. Run the complete Runtime-native [First Work Item](first-work-item.md).

Installation is a shared Runtime operation. Repository attachment is explicit and
creates repository-local `.ai/` state; one Runtime can serve many repositories without
sharing their Work Items, evidence, or active context.

## Reader routes

- [30-second start](30-second-start.md) — inspect, attach, status, and doctor.
- [Installation](installation.md) — immutable public Runtime and repository separation.
- [Repository profile calibration](calibration.md) — confirm one project-owned quality command.
- [Standard adoption guide](standard-adoption-guide.md) — the complete adoption sequence.
- [Security and release verification](security-release-verification.md) — supply-chain and external evidence boundaries.
- Examples: [Android](examples/android.md), [iOS](examples/ios.md), and [Java](examples/java.md).

After the first Work Item, continue with [Features](../features/README.md) and
[Operations](../operations/README.md).

[Documentation home](../README.md) | [中文](README.zh-CN.md) | [日本語](README.ja.md)
