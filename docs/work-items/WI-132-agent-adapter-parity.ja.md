---
author: AI Cockpit maintainers
workItemId: WI-132-agent-adapter-parity
title: Agent adapter と provider surface の parity
description: Rust Runtime の境界を明示したまま、参照元の Contract-first と可視 Outcome 規則を repository-local Agent adapter に伝える。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-133-docs-truth
---

# WI-132 — Agent adapter と provider surface の parity

## Intent

install 済み Agent が参照元と同じ安全な運用境界を受け取れるようにします。
Python/Make の runtime をコピーせず、adapter は明示的な discovery projection とし、
現在の governance state は shared Rust Runtime が所有します。

## Boundaries

- 新規 Cursor install は provider-native な `.cursor/rules/ai-cockpit.mdc` を使う。
- 既存の managed `.cursor/rules/ai-cockpit.md` は保持し、安全に読取・所有・復旧できるようにする。
- managed section に Contract-first、unknown、preflight の human pause、Summary、可視 Outcome、merge 後の closure を含める。
- glossary と reference workflow を英語・日本語・簡体字中国語で更新する。
- provider/global configuration、Core protocol を変更せず、V1 runtime code/schema/installer、Python module、Make command をコピーしない。

## Acceptance

- provider detection/install/doctor/repair/detach は repository-bound、deterministic、isolated、fail-closed である。
- 新規 install の Cursor target は `.mdc`、managed legacy `.md` は unsafe migration なしに利用できる。
- `not_ready` または `needs_human_confirmation` では人へ pause し、Contract の decision を発明せず、archive/closure 前に可視 Outcome を提示する。
- glossary と三言語の workflow/parity 文書が Rust adaptation boundary と provider policy を説明する。

## Verification

Focused Agent/CLI test、workspace check、clippy、documentation acceptance の結果は
archived Contract、verification evidence、close decision、Runtime evidence を参照してください。
証跡は `.ai/evidence/WI-132-agent-adapter-parity.verification.json` と
`.ai/decisions/WI-132-agent-adapter-parity.close.json` です。
