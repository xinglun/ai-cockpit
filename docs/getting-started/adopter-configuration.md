---
author: AI Cockpit maintainers
title: "Adopter configuration"
description: "Repository-owned review, security, recovery, profile, and CI decisions required for adoption."
audience:
  - adopter
  - security
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# Adopter configuration

AI Cockpit provides repository-local governance mechanics. It does not choose
the people, provider identities, security contacts, or organization policy for
an adopter. Complete this checklist in a separate, reviewed Work Item:

- protect the discovered remote default branch and require the repository's
  approved review policy;
- configure CODEOWNERS or an equivalent provider rule with the real owner;
- publish the repository's private vulnerability-reporting route, supported
  versions, response expectations, and disclosure policy in `SECURITY.md`;
- name the recovery and incident owner, including a safe stop/resume route;
- confirm the project quality command and its coverage boundary;
- configure hosted CI to run the repository-owned gates and retain provider
  evidence without placing secrets in Work Item records;
- document which identity, approval, signing, provenance, and retention claims
  remain external.

Use Runtime facts to inspect the repository, but do not treat them as provider
proof:

```bash
repo=/path/to/repository
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
ai-cockpit agent doctor --repo "$repo" --json
```

A green local result does not prove that branch protection or required reviews
are enabled. Missing or contradictory external evidence remains Unknown and
requires the responsible person or provider to resolve it.

[Standard adoption guide](standard-adoption-guide.md) | [中文](adopter-configuration.zh-CN.md) | [日本語](adopter-configuration.ja.md)
