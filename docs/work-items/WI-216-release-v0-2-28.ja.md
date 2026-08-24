---
author: AI Cockpit maintainers
title: "WI-216 — v0.2.28 immutable Release と adopter acceptance"
description: "merge 済みの reference comparison baseline から v0.2.28 を公開し、インストール済み Runtime で公開 artifact を受入れます。"
audience:
  - maintainer
  - adopter
  - reviewer
workItemId: WI-216-release-v0-2-28
status: recovered
authority: canonical
lastVerifiedBy: WI-216-release-v0-2-28
---

# WI-216 — v0.2.28 immutable Release と adopter acceptance

この Work Item は最初の reference-source file comparison batch の merge 後に
patch Release を公開します。比較 baseline は既に merge 済みであり、この境界は
次の immutable Runtime artifact と公開受入れだけを扱います。

## Acceptance

1. workspace package version、lockfile、Release/architecture docs、三言語 route が v0.2.28 を一貫して示します。
2. tag は reviewed な default branch の merge descendant からだけ作成し、source、tag、manifest、checksum、provenance gate を通過します。
3. public adopter と N-1 acceptance は immutable な download artifact だけを使い、source checkout、workspace binary、local `target` fallback を禁止します。
4. acceptance receipt は repository/runtime identity、isolation manifest、cleanup state、evidence reuse、visible localized Outcome を束縛します。
5. post-release version consistency、adopter acceptance、upgrade acceptance が Release truth を書き換えずに成功します。

## Out of scope

次の reference-source file comparison batch は別の境界です。この Work Item は Runtime
feature を追加せず、reference implementation code を copy せず、user-global Agent/MCP
configuration を変更しません。

## Evidence boundary

公開 Release、download archive、manifest、checksum、attestation、adopter receipt は
immutable な external evidence です。post-release failure は
`releasePublished: true` と `adopterAcceptance: failed` を記録し、Release truth を書き換えません。
