---
author: AI Cockpit maintainers
title: "Troubleshooting and Recovery"
description: "Safe next actions for common AI Cockpit stop states."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
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
| `archive` or `close` fails | Governance is not green or an archive identity is invalid. | Preserve active records, repair evidence, then rerun the failed lifecycle step. |
| Verification reruns instead of reusing | One or more identity bindings changed or reuse is not authorized. | Treat the rerun as the safe behavior; inspect the receipt reason. |
| MCP says repository binding is required | The server was started without a repository-bound adapter. | Configure `mcp --repo <path>` and keep the path explicit. |
| Release asset or tag is absent | Public distribution evidence is not available yet. | Stop installation and wait for the immutable Release and matching assets. |

Never delete `.ai` records, receipts, or `index.pending` to make status look
clean. Missing, malformed, stale, or contradictory evidence is intentionally
fail-closed.

An already `finish_ready` Work Item has no implicit rewind operation. This is
intentional: a rewind would make the state history ambiguous. Use a successor
Work Item for the changed snapshot and reference the old receipt as historical
evidence.
