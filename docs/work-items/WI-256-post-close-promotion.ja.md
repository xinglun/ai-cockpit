---
author: AI Cockpit maintainers
title: "WI-256 — Typed post-close documentation promotion"
workItemId: WI-256-post-close-promotion
description: "close 後の documentation promotion を再現可能・identity-bound・fail-closed にします。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-256-post-close-promotion
authority: canonical
---

# WI-256 — Typed post-close documentation promotion

WI-256 は WI-255 で判明した workflow gap を修正します。structured close は有効でも、三言語の
documentation projection が忘れやすい手動 command に依存していました。本 WI は repository-owned
typed plan/apply wrapper を追加します。Markdown の動作を Runtime Core に移さず、immutable な
`.ai` lifecycle bytes も書き換えません。

## Acceptance boundary

- plan は repository identity、同期済み `origin/main`、approved close、sequence-2 finalization、
  archive/evidence identity、6 つの exact controlled documentation path の before/after digest を
  bind します。
- stale、foreign、malformed、symlink、dirty、partial、unexpected state は write 前に fail closed。
  current plan の再 apply は deterministic no-op です。
- WI-255 の English、Simplified Chinese、Japanese projection を `.ai` archive、evidence、
  finalization、close bytes を変更せず `Implemented` にします。
- AGENTS と三言語の workflow/command reference は
  `close → visible Outcome → post-close plan/apply → check-all → terminal CI` を要求します。
- wrapper、promoter、documentation、manifest、governance、format、clippy、locked workspace
  checks が installed Runtime で通過します。

## Verification scenarios

Contract は valid plan/apply/idempotent rerun、typed identity/staleness rejection、
dirty/unexpected/partial projection rejection、actionable terminal-CI handoff を cover します。
wrapper test は isolated Git fixture を使用し、immutable `.ai` digest が変わらないことを確認します。

## References

- [Agent workflow](../reference/agent-workflow.ja.md)
- [Commands](../reference/commands.ja.md)
- [Reference parity](../reference/reference-parity.ja.md)
