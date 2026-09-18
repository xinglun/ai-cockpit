---
author: AI Cockpit maintainers
workItemId: WI-892-release-v0-2-97
title: Release v0.2.97 after immutable v0.2.96 candidate rejection
description: Correct the workspace version identity and publish the repaired release candidate.
audience: [maintainer, reviewer, adopter]
status: implemented
authority: user:release-after-all-work-items
lastVerifiedBy: WI-892-release-v0-2-97
terminalArchive: .ai/work-items/archive/WI-892-release-v0-2-97.contract.json
terminalVerification: .ai/evidence/WI-892-release-v0-2-97.verification.json
terminalFinalization: .ai/decisions/WI-892-release-v0-2-97.finalize.json
terminalDecision: .ai/decisions/WI-892-release-v0-2-97.close.json
---

# WI-892 — Release v0.2.97 after immutable v0.2.96 candidate rejection

The v0.2.96 tag is retained as an immutable failed candidate because its
source workspace still identified version 0.2.95. This Work Item updates the
workspace identity and publishes the next unused version without moving or
deleting the failed tag.

## Acceptance boundary

- Set the workspace and generated archives to version 0.2.97.
- Keep v0.2.96 immutable and record the failed aggregate evidence.
- Publish only after reviewed checks, manifest/checksum/SBOM binding, and
  downloaded fresh-install and v0.2.93 upgrade acceptance pass in isolated
  roots.
- Keep object repositories and Outcome/HCI/performance scope unchanged.

## Verification plan

Run format and version checks first, then the declared workspace verification,
the release workflow, and downloaded-artifact acceptance. Preserve every
failed-run receipt; do not move tags or rerun unrelated Work Items.
