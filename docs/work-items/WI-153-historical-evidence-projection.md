---
author: AI Cockpit maintainers
title: "WI-153 — Historical evidence projection"
description: "Project valid archived evidence from older Runtime identities as historical without weakening active fail-closed validation."
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-153-historical-evidence-projection
workItemId: WI-153-historical-evidence-projection
---

# WI-153 — Historical evidence projection

WI-153 preserves immutable archived evidence while distinguishing an older,
otherwise valid Runtime identity from a current verification failure. Archived
v2 evidence produced by an earlier Runtime is shown as historical yellow with
`historical_evidence_not_revalidated`; malformed, tampered, or identity-invalid
evidence remains fail-closed. Active Work Items continue to reject a foreign
Runtime identity as red.

The tri-language parity index was also corrected to include WI-147 through
WI-152. Existing Work Item archive and evidence bytes were not rewritten.

Evidence: `.ai/evidence/WI-153-historical-evidence-projection.verification.json`.
Decision: `.ai/decisions/WI-153-historical-evidence-projection.close.json`.
