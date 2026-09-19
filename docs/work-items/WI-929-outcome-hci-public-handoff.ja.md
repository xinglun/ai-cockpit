---
author: AI Cockpit maintainers
workItemId: WI-929-outcome-hci-public-handoff
title: アーカイブ済み Work Item の完全な Outcome 会話引き渡し
description: ホスト表示を未確認のまま成功と主張せず、CLI と MCP の利用者が完全な言語選択済み人間向け Outcome と順序付き assistant message event を取得できるようにする。
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:xinglun
lastVerifiedBy: WI-929-outcome-hci-public-handoff
---

# WI-929 — アーカイブ済み Work Item の完全な Outcome 会話引き渡し

この Work Item は、アーカイブ後の Outcome を HCI に渡す最後の欠落を収束させます。
アーカイブ応答は、検証済みの同一の完全な人間向け Outcome を、言語、Work Item
identity、delivery identity、順序付き assistant message event とともに返さなければ
なりません。stdout、JSON、または MCP content だけを読む consumer でも、stderr や
summary view にだけ残った本文を失ってはなりません。

現在の会話言語が表示言語（`en`、`zh`、`ja`）を選択します。Contract 原文、evidence
fact、receipt identity は変更せず、Runtime の表示だけをローカライズします。host
adapter が無い場合は `full_handoff_only` で display confirmation は unknown のままです。
`display_confirmed` は実際の message ごとの receipt からだけ導出し、capability や
Agent が入力した boolean から推測しません。

中断時は同じ本文と identity-bound な accepted receipt の進捗を再利用し、verification
や archive を再実行しません。idempotency または display-confirmation API が無い host
では、重複リスクと表示状態を明示的な制限として残します。制御された command-host
test は adapter protocol と event payload の正確性を証明するだけで、第三者の Codex や
Claude の会話画面で実際に表示されたことは証明しません。

## 受入れと evidence

- CLI の通常 archive と `archive --json` は、同じ検証済み `OutcomeDelivery` から完全な
  human handoff 本文と順序付き `assistantMessageEvents` を返します。
- MCP の `delivery=true` は同じ本文、identity、言語、順序付き segment、event を返し、
  parity test が本文と event sequence を比較します。
- 英語、簡体字中国語、日本語で事実と次の action を保ち、現在の会話言語だけを表示に
  反映します。
- host 不在、中断、非連続 progress、連続した二つの Work Item でも fail-closed を保ち、
  新しい verification または archive execution を発生させません。
- evidence に宣言が無い場合、最終 Outcome の user benefit は unknown のままです。

この Work Item は汎用 chat platform、lifecycle や authorization rule の変更、object
repository の変更、release publication を含みません。
