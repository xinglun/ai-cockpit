---
author: AI Cockpit maintainers
title: "Troubleshooting and Recovery"
description: "Safe next actions for common AI Cockpit stop states."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-512-reference-docs-batch-33
capabilityClaims:
  - recovery
---

# Troubleshooting and recovery

| Observation | Meaning | Safe next action |
| --- | --- | --- |
| `state: unattached` | No valid `.ai/cockpit.toml` exists. | Review the target, then run `attach --repo <path>`. |
| `calibration_required` | The profile was detected but not human-confirmed. | Review the detected command and run `profile confirm`. |
| Preflight `yellow` | Evidence is missing or requires confirmation. | Read blockers and safe actions; repair the Contract or obtain the required decision. |
| Preflight `red` | Scope, authority, protocol, or repository state is invalid. | Stop. Fix the named repository fact or authority before editing. |
| `finish` says receipt missing/stale | No passed current-snapshot Work Item verification exists. | Run `verify --work-item <id>` after the final edit; do not bypass the check. |
| A `finish_ready` Work Item becomes stale before archive | The repository changed after its verification snapshot was bound. | Do not edit the summary or receipt. Preserve the historical bytes and create a new authorized Work Item from the current snapshot. |
| A stale predecessor must no longer be retried | Its evidence is historical and a bound successor already owns the work. | Record an identity-bound `supersede` recovery decision, then archive and close the predecessor as historical; do not rewrite or re-verify its old evidence. |
| `archive` or `close` fails | Governance is not green or an archive identity is invalid. | Preserve active records, repair evidence, then rerun the failed lifecycle step. |
| `close` says retained resources require cleanup | The receipt is a legacy shared-primary record or an ordinary retained linked resource. | Confirm `provider=local` and the primary-checkout facts; if they cannot be derived, run `work-item finalize-recovery-plan` and record the explicit historical recovery receipt. Never change `retained` to `deleted` by hand. |
| Verification reruns instead of reusing | One or more identity bindings changed or reuse is not authorized. | Treat the rerun as the safe behavior; inspect the receipt reason. |
| MCP says repository binding is required | The server was started without a repository-bound adapter. | Configure `mcp --repo <path>` and keep the path explicit. |
| Release asset or tag is absent | Public distribution evidence is not available yet. | Stop installation and wait for the immutable Release and matching assets. |

Never delete `.ai` records, receipts, or `index.pending` to make status look
clean. Missing, malformed, stale, or contradictory evidence is intentionally
fail-closed.

## Installation and toolchain boundaries

If `attach`, `profile confirm`, or `agent doctor` stops, inspect the named
repository fact and rerun the same command with the explicit `--repo`. The
Runtime does not install or switch a project's JDK, Gradle, Xcode, CocoaPods,
Node, or other external toolchain. A missing project command is an adopter
configuration issue, not a reason to weaken the Contract or substitute a
workspace binary.

If a repository has an active Work Item, finish/archive it before an upgrade or
new Work Item. If a linked worktree, remote default branch, or finalization
receipt is missing, stop and preserve the records; recovery must use an
identity-bound successor/retry path. The reference source's Make/Python
wizard commands are not Rust Runtime commands. Use the installed CLI and the
repository's own declared verification commands.

An already `finish_ready` Work Item has no implicit rewind operation. This is
intentional: a rewind would make the state history ambiguous. Use a successor
Work Item for the changed snapshot and reference the old receipt as historical
evidence.
