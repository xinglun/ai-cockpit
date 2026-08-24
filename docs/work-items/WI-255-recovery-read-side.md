---
author: AI Cockpit maintainers
title: "WI-255 — Recovery-decision read-side validation"
workItemId: WI-255-recovery-read-side
description: "Revalidate current recovery decisions before Outcome or archive consumption while preserving immutable historical records."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-255-recovery-read-side
terminalArchive: .ai/work-items/archive/WI-255-recovery-read-side.contract.json
terminalVerification: .ai/evidence/WI-255-recovery-read-side.verification.json
terminalFinalization: .ai/decisions/WI-255-recovery-read-side.finalize.70b8faaab38e83dcd7d4fe55892abfe4c553ec1efb369bf81c2e259a9fe8566b.json
terminalDecision: .ai/decisions/WI-255-recovery-read-side.close.json
authority: canonical
---

# WI-255 — Recovery-decision read-side validation

WI-255 rebuilds the current recovery read-side from the synchronized default
branch. It selectively carries the code, tests, and user-facing boundary that
were reviewed in unmerged PRs #192 and #202; no WI-242 or WI-248 lifecycle byte
is copied, rewritten, or presented as current evidence.

## Acceptance boundary

- Every current recovery candidate is a bounded regular non-symlink JSON file.
  Duplicate keys, malformed or oversized input, and a digest-suffixed filename
  that does not match the canonical JSON digest fail closed.
- Before Outcome or superseded archive consumes a candidate, the repository,
  Work Item, current Runtime, predecessor Contract/Summary/Outcome/Events,
  timestamp, decision shape, and successor Contract binding are revalidated.
- One invalid current candidate cannot be skipped in favor of an older valid
  candidate. Equally timed valid candidates use deterministic path ordering.
- Failure uses the stable `recovery_decision_invalid` boundary, yields a red
  current Outcome, and cannot move active artifacts.
- Historical immutable archives retain their recorded Runtime identity and
  projection; the current-read rule does not retroactively reclassify them.

## Verification scenarios

The Contract requires five scenarios: valid current recovery, forged current
recovery, invalid current candidate files, deterministic candidate selection,
and historical archive compatibility. Focused repository tests cover each
scenario with real filesystem artifacts, followed by repository, documentation,
governance, clippy, and full-workspace checks.

## Lifecycle projection

This row remains conditional until verified close. Its future evidence paths
are `.ai/work-items/archive/WI-255-recovery-read-side.contract.json`,
`.ai/evidence/WI-255-recovery-read-side.verification.json`,
`.ai/decisions/WI-255-recovery-read-side.finalize.json`, and
`.ai/decisions/WI-255-recovery-read-side.close.json`.

## References

- [Agent workflow](../reference/agent-workflow.md)
- [Commands](../reference/commands.md)
- [Reference parity](../reference/reference-parity.md)
