---
author: AI Cockpit maintainers
title: "WI-271 — WI-270 finalization recovery"
workItemId: WI-271-finalization-recovery
description: "Recover WI-270 without rewriting its immutable archive and bind reviewed PR context before verification and archive."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-271-finalization-recovery
authority: canonical
---

# WI-271 — WI-270 finalization recovery

## Intent

WI-270 completed the first bounded reference Contract comparison, but its
archive was created after `finalize-plan` had been bound to a provisional
`pullRequest: pending` context. Hosted governance correctly rejected the
missing valid finalization boundary. This successor preserves every WI-270
byte and completes the reviewed delivery with the actual PR context bound
before verification and archive.

## Scope

- Preserve the exact WI-270 archive, evidence, preflight receipts, docs, and
  inventory changes; do not rewrite or delete predecessor bytes.
- Record the Runtime-valid WI-270 successor recovery decision.
- Mark WI-270 recovered and register WI-271 in all three parity ledgers before
  WI-271 archive evidence is created.
- Create the reviewed PR, run `finalize-plan` with its exact URL, then execute
  the installed Runtime lifecycle and hosted checks.
- Complete merge observation, exact branch/worktree cleanup, finalization
  verification, structured close, and a visible human Outcome.

## Boundary

This is a narrow lifecycle recovery. It does not compare another reference
slice, rewrite historical evidence, refactor the Runtime's large source files,
or change global Agent/MCP configuration. Architecture cleanup is deferred
until the reference-comparison batches are complete and will be separately
bounded and verified.

## Verification

- installed Runtime with explicit `--repo`
- governance integrity, parity, inventory, and documentation checks
- hosted quality, Windows, and reference-oracle checks
- finalization and exact cleanup receipts
- visible `Outcome: 🟢`, `Outcome: 🟡`, or `Outcome: 🔴` containing status,
  unknowns, evidence, human decision, and next action

## Terminal evidence

- Archive: `.ai/work-items/archive/WI-271-finalization-recovery.contract.json`
- Verification: `.ai/evidence/WI-271-finalization-recovery.verification.json`
- Recovery: `.ai/decisions/WI-270-reference-contract-batch.recovery.json`
- Finalization: `.ai/decisions/WI-271-finalization-recovery.finalize.json`,
  `.ai/decisions/WI-271-finalization-recovery.finalize.e1afe79cf257e0675123913123a2eca1aba0b7bf7ffa85893d0054409b76a258.json`,
  `.ai/decisions/WI-271-finalization-recovery.finalize.3fbc88f554e5c352127cb3872f4e082effd03d1ce8534bd87796be2862252152.json`
- Close: `.ai/decisions/WI-271-finalization-recovery.close.json`
- Reviewed resources: PR [#224](https://github.com/xinglun/ai-cockpit/pull/224) and
  merge-observation PR [#225](https://github.com/xinglun/ai-cockpit/pull/225), both
  merged with hosted checks passing.
