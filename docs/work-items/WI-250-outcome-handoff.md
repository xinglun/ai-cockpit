---
author: AI Cockpit maintainers
title: "WI-250 — Direct lifecycle Outcome handoff"
workItemId: WI-250-outcome-handoff
description: "Make lifecycle commands surface the verified human Outcome without breaking their JSON interface."
audience:
  - adopter
  - maintainer
status: recovered
lastVerifiedBy: WI-250-outcome-handoff
authority: canonical
---

# WI-250 — Direct lifecycle Outcome handoff

Lifecycle mutations previously returned an `outcome` only inside stdout JSON.
That record was stable for machines, but an embedding Agent or terminal could
leave the human handoff folded inside tool output. WI-250 adds a direct,
backward-compatible handoff at the CLI boundary.

## Behavior

- `finish`, `archive`, and `close` keep their existing parseable stdout JSON
  and render the same Runtime-validated, localized human Outcome on stderr by
  default.
- `--json` suppresses only the stderr handoff, preserving a machine-only mode.
- A blocked `finish` renders the persisted red or yellow Outcome and then
  returns the original nonzero error. Presentation never bypasses a lifecycle
  gate.
- The renderer retains the fixed `Outcome: 🔴/🟡/🟢` marker and
  Unknowns, Human decisions, Evidence, and Next action sections. A structured
  close decision becomes visible through the same projection.

## Boundary

The CLI cannot force a host application to open or expand a conversation
panel. A host must surface stderr, while a person can deterministically replay
the durable handoff with `work-item outcome`. OutcomeV2, archive truth, MCP,
and existing historical Work Item bytes are unchanged.

## Verification

CLI integration tests cover all three languages, stdout compatibility,
machine-only suppression, structured close decisions, and blocked fail-closed
behavior. Documentation acceptance, parity and governance gates, Rustfmt,
Clippy, and the locked workspace suite remain required.
