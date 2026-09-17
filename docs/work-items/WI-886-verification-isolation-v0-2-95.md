---
author: AI Cockpit maintainers
workItemId: WI-886-verification-isolation-v0-2-95
title: Repair release acceptance isolation and publish v0.2.95
description: Preserve explicit verification target isolation and prove candidate and downloaded-artifact acceptance after the immutable v0.2.94 acceptance failure.
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
predecessorWorkItemId: WI-883-release-v0-2-94-current-main
lastVerifiedBy: WI-886-verification-isolation-v0-2-95
---

# WI-886 — Repair release acceptance isolation and publish v0.2.95

This successor preserves the immutable v0.2.94 tag and its failed staged
acceptance evidence. It repairs the verification environment boundary so a
caller-provided isolated `CARGO_TARGET_DIR` is honored, while the documented
HOME fallback and `CARGO_INCREMENTAL=0` remain unchanged. Publication of
v0.2.95 is the final step after reviewed merge, candidate acceptance, and
downloaded-artifact installation/upgrade acceptance.

The result must report isolation, cleanup, verification, release identity, and
remaining unknowns separately. It must not infer a successful public release
from a candidate or source checkout.
