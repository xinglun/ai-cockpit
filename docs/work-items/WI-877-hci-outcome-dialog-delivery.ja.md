---
author: AI Cockpit maintainers
workItemId: WI-877-hci-outcome-dialog-delivery
title: Conversation-facing archived Outcome delivery
description: 完全で identity-bound な assistant-message event を archive Outcome の各 segment に公開し、host capability の境界を正直に保持する。
audience: [adopter, contributor, maintainer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-877-hci-outcome-dialog-delivery
terminalArchive: .ai/work-items/archive/WI-877-hci-outcome-dialog-delivery.contract.json
terminalVerification: .ai/evidence/WI-877-hci-outcome-dialog-delivery.verification.json
terminalDecision: .ai/decisions/WI-877-hci-outcome-dialog-delivery.close.json
---

# WI-877 — Conversation-facing archived Outcome delivery

この Work Item は archive 後に残る HCI handoff の gap を閉じる。Runtime は
`OutcomeDelivery` を唯一の検証済み fact source とし、順序付き
`assistantMessageEvents` を公開する。conversation layer は各 complete segment
を逐語的に独立した assistant message として転送できる。

## Acceptance boundary

- 通常および対応する historical archive path は、valid な CLI JSON と MCP delivery output に full body を保持する。
- 各 conversation event は同じ prepared payload から Work Item、delivery、archive、language、順序、segment digest、body を保持する。
- host acceptance/display は message ごとの receipt だけから導く。host API がなければ `full_handoff_only` と display `unknown` のままとし、Codex/Claude connector は主張しない。
- 中断した delivery は同じ identity-bound accepted progress を再利用し、verification や archive を再実行しない。連続した Work Item は分離される。

この Work Item は authorization、verification、release 公開、Issue #851 recovery semantics、object repository を変更しない。
