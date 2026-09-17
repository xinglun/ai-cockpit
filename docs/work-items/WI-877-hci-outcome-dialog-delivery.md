---
author: AI Cockpit maintainers
workItemId: WI-877-hci-outcome-dialog-delivery
title: Conversation-facing archived Outcome delivery
description: Expose one complete, identity-bound assistant-message event per archived Outcome segment while preserving honest host capability limits.
audience: [adopter, contributor, maintainer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-877-hci-outcome-dialog-delivery
---

# WI-877 — Conversation-facing archived Outcome delivery

This Work Item closes the remaining HCI handoff gap after archive. The Runtime
keeps `OutcomeDelivery` as the single validated fact source and exposes ordered
`assistantMessageEvents` so a conversation layer can forward each complete
segment verbatim as an independent assistant message.

## Acceptance boundary

- Normal and supported historical archive paths retain the complete full body
  in valid CLI JSON and MCP delivery output.
- Each conversation event preserves Work Item, delivery, archive, language,
  ordering, segment digest, and body facts from the same prepared payload.
- Host acceptance/display comes only from per-message receipts. A missing host
  API remains `full_handoff_only` with display `unknown`; no Codex or Claude
  connector is claimed.
- Interrupted delivery reuses the same identity-bound accepted progress and
  does not rerun verification or archive. Consecutive Work Items stay distinct.

The Work Item does not change authorization, verification, release publication,
Issue #851 recovery semantics, or object repositories.
