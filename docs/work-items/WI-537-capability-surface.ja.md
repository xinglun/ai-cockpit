---
title: "型付き MCP capability surface"
workItemId: WI-537-capability-surface
status: 進行中
authority: Work Item Contract と archive 済み Runtime evidence
---

# WI-537 — 型付き MCP capability surface

AI Cockpit は MCP tool を discoverable で repository-bound な interface として
人と Agent に提供します。`tools/list` は各 tool の引数を記述し、`tools/call` は
dispatch 前に欠落・不正型・競合・unknown の引数を拒否します。CLI と三言語の
reference は同じ discovery 手順と人向け Outcome handoff を説明します。

範囲は MCP capability の記述/検証と文書に限定し、lifecycle mutation、global
Agent/MCP 設定、自動的な host conversation 投稿は追加しません。

Work Item の close 後、verification と terminal lifecycle の記録を[reference
parity registry](../reference/reference-parity.ja.md)から参照します。
