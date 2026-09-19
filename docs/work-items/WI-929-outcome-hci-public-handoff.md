---
author: AI Cockpit maintainers
workItemId: WI-929-outcome-hci-public-handoff
title: Public complete Outcome handoff for archived Work Items
description: Keep the complete language-selected human Outcome and ordered assistant-message events available to CLI and MCP consumers without claiming host display that has not been confirmed.
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:xinglun
lastVerifiedBy: WI-929-outcome-hci-public-handoff
---

# WI-929 — Public complete Outcome handoff for archived Work Items

This Work Item closes the remaining Outcome/HCI handoff gap after the archived
Outcome delivery work. An archive response must carry the same complete,
validated human Outcome that the human handoff renders, including its language,
Work Item identity, delivery identity, and ordered assistant-message events.
Consumers that read only stdout, JSON, or MCP content must not lose the body
that was previously visible only on stderr or behind a summary view.

The active conversation language selects the presentation language (`en`, `zh`,
or `ja`). Contract text, evidence facts, and receipt identities remain unchanged;
only Runtime presentation is localized. A missing host adapter remains
`full_handoff_only` with display confirmation unknown. `display_confirmed` is
reported only from actual per-message receipts, never from a capability flag or
an agent-authored boolean.

Interrupted delivery reuses the same prepared body and identity-bound accepted
receipt progress. It does not re-run verification or archive. When the host has
no idempotency or display-confirmation API, duplicate or display state remains
an explicit limitation. The controlled command-host test proves the adapter
protocol and exact event payloads; it is not evidence that a third-party Codex or
Claude conversation displayed the messages.

## Acceptance and evidence

- CLI archive and `archive --json` expose the exact full human handoff body and
  ordered `assistantMessageEvents` from one validated `OutcomeDelivery`.
- MCP `delivery=true` exposes the same body, identity, language, ordered
  segments, and events; parity tests compare the body and event sequence.
- English, Simplified Chinese, and Japanese preserve the same facts and next
  action while selecting the active conversation language.
- No-host, interrupted, non-contiguous, and consecutive-Work-Item cases remain
  fail-closed and do not create new verification or archive executions.
- The final Outcome keeps user benefit unknown unless evidence declares it.

This Work Item does not create a generic chat platform, alter lifecycle or
authorization rules, modify object repositories, or publish a release.
