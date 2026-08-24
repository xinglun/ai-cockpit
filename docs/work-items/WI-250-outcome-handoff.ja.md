---
author: AI Cockpit maintainers
title: "WI-250 — Lifecycle Outcome の直接 handoff"
workItemId: WI-250-outcome-handoff
description: "JSON interface を壊さず lifecycle command が検証済み Human Outcome を直接表示します。"
audience:
  - adopter
  - maintainer
status: current
lastVerifiedBy: WI-250-outcome-handoff
authority: canonical
---

# WI-250 — Lifecycle Outcome の直接 handoff

lifecycle mutation は従来、stdout JSON 内の `outcome` だけを返していました。この
record は機械には安定していますが、埋め込み Agent や terminal が人間向け handoff
を tool output 内に折りたたむ可能性がありました。WI-250 は CLI boundary に直接かつ
後方互換の handoff を追加します。

## Behavior

- `finish`、`archive`、`close` は既存の parse 可能な stdout JSON を維持し、既定では
  同じ Runtime-validated localize 済み Human Outcome を stderr に render します。
- `--json` は stderr handoff だけを抑止し、機械専用 mode を保ちます。
- block された `finish` は永続化済みの赤または黄の Outcome を render し、その後で
  元の nonzero error を返します。presentation が lifecycle gate を迂回しません。
- renderer は固定の `Outcome: 🔴/🟡/🟢` marker と Unknowns、Human decisions、
  Evidence、Next action section を維持し、structured close decision も同じ projection
  で可視化します。

## Boundary

CLI は host application に会話 panel の表示・展開を強制できません。host は stderr
を提示する必要があり、人は `work-item outcome` で durable handoff を決定的に再生
できます。OutcomeV2、archive truth、MCP、既存の historical Work Item bytes は変更しません。

## Verification

CLI integration test は 3 言語、stdout compatibility、machine-only suppression、
structured close decision、blocked fail-closed behavior を検証します。documentation
acceptance、parity/governance gate、Rustfmt、Clippy、locked workspace suite も必須です。
