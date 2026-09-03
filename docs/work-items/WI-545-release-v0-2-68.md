---
author: AI Cockpit maintainers
title: "WI-545 — v0.2.68 release and public-artifact acceptance"
description: "Publish the next verified Runtime release and bind public installation evidence."
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
workItemId: WI-545-release-v0-2-68
lastVerifiedBy: WI-545-release-v0-2-68
terminalArchive: .ai/work-items/archive/WI-545-release-v0-2-68.contract.json
terminalVerification: .ai/evidence/WI-545-release-v0-2-68.verification.json
terminalFinalization: .ai/decisions/WI-545-release-v0-2-68.finalize.json
terminalDecision: .ai/decisions/WI-545-release-v0-2-68.close.json
---

[简体中文](WI-545-release-v0-2-68.zh-CN.md) · [日本語](WI-545-release-v0-2-68.ja.md)

# WI-545 — v0.2.68 release and public-artifact acceptance

## Intent and goal

Publish v0.2.68 from the reviewed, synchronized default branch and prove that
the immutable public artifact can be installed and accepted without source or
workspace-binary fallback.

## Scope

- Advance Cargo package identity and the tri-language release/versioning pages.
- Register the release in the reference-parity ledger and preserve the
  release Work Item evidence path.
- Run pre-release quality/policy checks and, after publication, public artifact,
  adopter, N-1, installed-runtime, and cleanup acceptance.

## Acceptance boundary

The public Release, archive/SBOM/provenance digests, and installed binary must
agree with the immutable tag. `first-adopter-smoke` remains `not_ready` until
human-owned Contract fields are supplied. Installation never attaches a
repository and this Work Item does not modify an object repository or global
Agent/MCP configuration.

## Verification

The active Contract is authoritative for commands and evidence. The terminal
handoff must include a visible human Outcome with status, unknowns, evidence,
human decision, and next action, followed by documentation promotion and exact
branch/worktree cleanup.
