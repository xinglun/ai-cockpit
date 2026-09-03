---
title: "Typed MCP capability surface"
workItemId: WI-537-capability-surface
status: in-progress
authority: Work Item contract and archived Runtime evidence
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
