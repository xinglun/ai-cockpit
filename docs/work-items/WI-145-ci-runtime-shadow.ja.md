---
author: AI Cockpit maintainers
workItemId: WI-145-ci-runtime-shadow
title: CI Runtime verification shadow
description: Cargo gate を削除せずに Phase 1 Runtime verification を CI に追加します。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-145-ci-runtime-shadow
---

# WI-145 — CI Runtime verification shadow

この Work Item は CI に post-release の immutable Runtime shadow lane を追加します。
Phase 1 では既存 Cargo quality check を独立した基準として残し、Phase 2 の比較と
Phase 3 の YAML policy convergence は後続の境界とします。

実装 Evidence: `.ai/evidence/WI-145-ci-runtime-shadow.verification.json`。
クローズ決定: `.ai/decisions/WI-145-ci-runtime-shadow.close.json`。
