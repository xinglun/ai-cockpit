---
author: AI Cockpit maintainers
title: "WI-248 — Recovery decision read-side recovery"
workItemId: WI-248-recovery-read-side-recovery
description: "Redeliver strict current recovery-decision read validation from the current default branch without importing the failed predecessor lifecycle."
audience:
  - adopter
  - maintainer
status: current
lastVerifiedBy: WI-248-recovery-read-side-recovery
authority: canonical
---

# WI-248 — Recovery decision read-side recovery

WI-242 produced a verified immutable archive and canonical pre-merge
finalization on an older base, but hosted quality rejected its missing
tri-language parity registration. The draft PR remains failed delivery truth.
WI-248 records the successor decision, starts from `origin/main@7d1bd78`, and
replays only the reviewed Rust implementation and regression tests from
`a3846e5`; no WI-242 archive, evidence, finalization, or old lifecycle commit
is imported.

## Current read boundary

- Recording still validates repository, Runtime, predecessor artifact, decision,
  timestamp, and successor identity before writing an append-only receipt.
- Outcome and archive consumers repeat those checks for every current recovery
  candidate and validate the regular-file and digest-bound filename boundary.
- A foreign, stale, tampered, malformed, or ambiguous candidate fails closed
  through stable `recovery_decision_invalid:<code>` diagnostics. It cannot move
  active artifacts or make Outcome green.
- A valid successor or supersede decision must bind an existing successor
  Contract back to the same repository, predecessor identity, and predecessor
  Contract digest.

## Historical boundary

Archived historical records remain readable under their historical projection.
The current-read validator does not rewrite immutable bytes or turn legacy
Runtime identity into a new current failure. WI-242 stays on PR #192 as the
preserved failed predecessor; `.ai/decisions/WI-242-recovery-read-side.recovery.json`
binds the handoff to WI-248.

## Verification

TDD regressions cover valid current recovery, forged repository/Runtime and
predecessor bindings, predecessor/successor tampering after record, invalid
candidate filenames, unchanged active artifacts on rejection, and historical
archive compatibility. Documentation, parity, governance, formatting, Clippy,
focused repository suites, and the locked workspace suite remain required.
