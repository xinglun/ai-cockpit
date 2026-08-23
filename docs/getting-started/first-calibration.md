---
author: AI Cockpit maintainers
title: "First calibration"
description: "A reviewable first confirmation of one repository quality command."
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - configuration
---

# First calibration

Run this only after installation, repository inspection, attachment, and a clean
`doctor` result. First obtain a candidate without changing the formal profile:

```bash
repo=/path/to/repository
ai-cockpit profile propose --repo "$repo"
```

Review the candidate against repository-owned documentation and hosted CI. Ask
the project owner to resolve every unknown executable, argument, working
directory, toolchain, environment, service, credential, and coverage fact.

Confirm only the exact command the owner approved. For example, when the
approved command is `cargo test --workspace`:

```bash
ai-cockpit profile confirm --repo "$repo" --program cargo --args test,--workspace
ai-cockpit doctor --repo "$repo"
```

A passing local command is bounded local evidence. It is not branch-protection,
provider, production, or enterprise evidence. If the candidate is wrong or a
required fact is Unknown, do not confirm it; correct the repository-owned
decision and rerun the read-only proposal.

Continue with [Adopter configuration](adopter-configuration.md), then the
[first Work Item](first-work-item.md).

[Calibration](calibration.md) | [中文](first-calibration.zh-CN.md) | [日本語](first-calibration.ja.md)
