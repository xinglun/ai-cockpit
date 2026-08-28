---
title: How to read Cockpit status
description: Turn generated status and Outcome into a bounded human decision.
author: AI Cockpit maintainers
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-346-reference-governance-profiles-status
capabilityClaims:
  - human_outcome_handoff
---

# How to read Cockpit status

[English](how-to-read-cockpit-status.md) · [简体中文](how-to-read-cockpit-status.zh-CN.md) · [日本語](how-to-read-cockpit-status.ja.md)

This page is for anyone reviewing a Work Item, including a non-technical
approver. It explains how to turn generated facts into a bounded decision; it
does not make the decision for you.

## Start with the visible handoff

Read the request-scoped status first, then replay the human handoff:

```sh
ai-cockpit status --repo <repository> --id <work-item>
ai-cockpit work-item outcome --repo <repository> --id <work-item>
```

The second command prints a report beginning with `Outcome: 🔴`, `Outcome: 🟡`,
or `Outcome: 🟢`. The CLI and repository-bound MCP `work_item_outcome` expose
the same human-facing projection. A host may decide how to display a message,
but a folded log or raw `work_item_get` response is not a substitute for this
handoff. Use `--json` only when a machine needs the stable `OutcomeV2` object.

## Read in this order

1. **Task Result and marker** — identify the Work Item and the decision signal.
2. **What was completed** — read the Runtime summary and any delivered-change
   claims.
3. **Problems found and stops triggered** — identify failed gates or reasons
   that work was paused.
4. **Problems resolved and risks** — distinguish recorded resolutions from
   remaining risks and warnings.
5. **Unknowns** — treat every unknown as an unresolved question, not as a
   hidden pass.
6. **Human decisions** — confirm the structured actor, authority source,
   reason, evidence/policy references, time, and resume condition when present.
7. **Verification and evidence** — inspect the named, repository-bound
   receipts and their freshness before taking action.
8. **Impact and next action** — an undeclared user benefit remains unknown; the
   next action must follow the stated recovery or review condition.
9. **Acceptance criteria (Contract language)** — read the owner's original
   Contract text. It is preserved verbatim for auditability.

The reference source calls some of these fields `Key Conclusion`,
`Recommendation`, `Decision Drivers`, `Evidence`, and `Scenario Coverage`.
The Rust Runtime keeps the same reading intent but presents typed Outcome
sections and a separate status projection; this is semantic parity, not a
source JSON wire contract.

## Meaning of the colors

| Marker | Meaning | Safe action |
| --- | --- | --- |
| 🟢 Green | Current, identity-bound evidence is sufficient for review. | Review the named evidence and obtain the decision required by your process. It is not merge or release authorization. |
| 🟡 Yellow | Evidence is incomplete, partial, historical, or a human decision is still required. | Investigate, collect the missing evidence, or record an explicit decision. Keep the Work Item in its current safe state. |
| 🔴 Red | A required control failed, authority/scope is invalid, or evidence is contradictory or malformed. | Stop. Follow the stated recovery condition; do not guess or hand-edit generated records. |
| `unknown` field | A fact or projection could not be trusted or has not been declared. | Ask for clarification or a fresh, bound receipt. It never silently becomes green. |

Colors are semantic signals, not scores. A green Outcome means that its
current evidence can be reviewed; it does not authorize a merge, release,
publication, security claim, or enterprise assurance. A yellow or red result is
not repaired by rerunning an unrelated command.

## Stop conditions and evidence boundaries

Stale, malformed, symlinked, cross-Work-Item, cross-repository, or
snapshot-mismatched status/evidence must be regenerated through the Runtime.
Never edit a generated Contract, Summary, Outcome, receipt, archive, or
decision to make a color change. Historical evidence remains immutable; a
current result requires fresh verification under the current Runtime.

Local verification, hosted CI, provider attestations, SBOM/provenance, and
enterprise approvals are different evidence boundaries. The report must show
which boundary produced each receipt. A local green check cannot be relabeled
as provider or enterprise assurance.

## Language and adopter inheritance

Runtime-generated headings, markers, statuses, summaries, unknown codes, and
recovery hints follow `AI_COCKPIT_LANGUAGE` (or the adapter's selected
language). Contract intent, scope, and acceptance criteria stay in their
original language; automatic translation must not alter governance facts. An
Agent conversation should render the handoff in the user's language and keep
the original Contract text available.

Every adopter repository follows the same route with an explicit `--repo`.
The shared Runtime has no current project or global Work Item, so one
repository's status cannot authorize or describe another repository's work.

## Next references

- [Governance profiles](governance-profiles.md) explains proportional quality routing.
- [Human-facing Outcome](outcome-report.md) defines the complete handoff and machine boundary.
- [Command reference](commands.md) lists the lifecycle commands and explicit bindings.
- [Troubleshooting and recovery](troubleshooting.md) explains how to resume after a stop.
