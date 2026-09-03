---
title: "Typed MCP capability surface"
workItemId: WI-537-capability-surface
author: AI Cockpit maintainers
description: "Typed, fail-closed MCP capability discovery and usage guidance."
audience:
  - adopter
  - maintainer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-537-capability-surface
terminalArchive: .ai/work-items/archive/WI-537-capability-surface.contract.json
terminalVerification: .ai/evidence/WI-537-capability-surface.verification.json
terminalFinalization: .ai/decisions/WI-537-capability-surface.finalize.json
terminalDecision: .ai/decisions/WI-537-capability-surface.close.json
---

# WI-537 — Typed MCP capability surface

AI Cockpit exposes its MCP tools as a discoverable, repository-bound interface
for people and Agents. `tools/list` describes each tool's arguments, and
`tools/call` rejects missing, malformed, conflicting, or unknown arguments
before dispatch. The CLI and tri-lingual reference pages describe the same
discovery and human Outcome handoff.

Scope is limited to MCP capability description/validation and documentation;
it does not add lifecycle mutations, configure global Agent/MCP settings, or
automatically post to a host conversation.

Verification and terminal lifecycle records are linked from the [reference
parity registry](../reference/reference-parity.md) after the Work Item is
closed.
