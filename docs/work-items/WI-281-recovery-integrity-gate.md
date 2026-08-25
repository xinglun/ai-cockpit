---
author: AI Cockpit maintainers
title: "WI-281 — recovery integrity gate"
workItemId: WI-281-recovery-integrity-gate
description: "Make CI resolve append-only recovery heads and require complete current-cycle Work Item projections."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-281-recovery-integrity-gate
authority: canonical
---

# WI-281 — recovery integrity gate

This Work Item closes the hosted-governance gap found when an immutable
predecessor has a canonical retry plus a digest-suffixed successor or
supersession receipt. The gate must select the valid recovery head
deterministically, keep invalid candidates fail-closed, and require the
tri-language Work Item and parity projections that the current release cycle
declares.
