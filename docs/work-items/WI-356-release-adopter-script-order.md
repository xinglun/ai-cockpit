---
author: AI Cockpit maintainers
title: "WI-356 — release adopter acceptance lifecycle ordering"
workItemId: WI-356-release-adopter-script-order
description: "Keep the published-artifact adopter harness aligned with the Runtime lifecycle entry gate."
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-357-release-adopter-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-356-release-adopter-script-order.contract.json
terminalVerification: .ai/evidence/WI-356-release-adopter-script-order.verification.json
---

# WI-356 — release adopter acceptance lifecycle ordering

[简体中文](WI-356-release-adopter-script-order.zh-CN.md) · [日本語](WI-356-release-adopter-script-order.ja.md)

## Intent and boundary

The release adopter harness must attach the fresh repository, install its
explicit Agent adapter, commit that governance state, and only then create the
first Work Item scaffold. This preserves the Runtime's fail-closed clean-entry
rule and keeps the acceptance proof reproducible.

The change is limited to the staged adopter harness and its static regression.
Runtime behavior, public release artifacts, global Agent/MCP configuration, and
the upgrade harness are outside this Work Item.

The regression deliberately checks the ordering boundary: adapter installation
must be committed before `work-item new` is invoked, so a clean repository is
present at lifecycle entry.

## Verification and delivery boundary

The harness static checks pass, including success and failure cleanup
assertions. The archived Contract, verification evidence, and provider
finalization/close receipts remain the authoritative lifecycle records; the
pre-merge parity row is intentionally promoted to Implemented only after the
reviewed PR is merged and closed.
