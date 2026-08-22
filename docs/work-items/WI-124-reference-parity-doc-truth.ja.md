---
author: AI Cockpit maintainers
workItemId: WI-124-reference-parity-doc-truth
title: Reference parity、documentation truth、release consistency
description: reader route、parity matrix、operator baseline を current Runtime に揃える。
audience:
  - adopter
  - maintainer
status: implementation
authority: canonical
lastVerifiedBy: WI-124-reference-parity-doc-truth
---

# WI-124 — Reference parity、documentation truth、release consistency

## Intent

English、Simplified Chinese、日本語の公開 document が current Rust Runtime の truth を
説明するようにします。Operations route は完全な governed lifecycle を示し、parity
matrix は現行の WI-121/WI-122/WI-123 boundary を含み、release consistency check は
Cargo metadata から current baseline version を解決します。

## Scope

- root README の lifecycle route と language link。
- 三言語の reference-parity page と current implementation baseline。
- 三言語の Contract/Summary field mapping page と Rust boundary。
- 三言語の operations page と version drift check。
- documentation と release consistency の regression script。
- この Work Item の三言語 document と Runtime が生成する receipt。

Field mapping は current typed Rust Protocol の documentation projection であり、新しい
schema を追加したり未実装の reference field を主張したりしません。Rust Runtime behavior、
Agent/MCP configuration、historical Work Item bytes は対象外です。

## Acceptance

1. 三言語の root README が `inspect → attach → start → preflight → checkpoint → verify → finish → archive → close` を示し、gate の意味を説明する。
2. Reference parity が WI-121、WI-122、WI-123 を current の Implemented boundary として evidence/document link 付きで示す。
3. Contract/Summary field page が三言語で current Rust field を mapping し、`Implemented`、`Partial`、`External` boundary を明示する。
4. Operations page が current adopter target を説明し、release version を hard-code しない。release script は Cargo metadata から version を解決する。
5. Documentation と version consistency script が lifecycle marker、parity status、baseline target、field mapping、stale version の drift で失敗する。
6. Runtime feature と global configuration は変更しない。

## Verification

```bash
bash tests/docs/documentation_acceptance.sh
bash tests/release/version_consistency.sh --repo .
git diff --check
```

最終 human Outcome は traffic-light marker、unknowns、evidence、human decision、next action
を含む別の visible handoff として届けます。
