---
author: AI Cockpit maintainers
title: "WI-415 — v0.2.43 release"
description: "WI-414 後の reviewed Runtime を v0.2.43 として公開し、次の public-artifact acceptance baseline を確立します。"
workItemId: WI-415-release-v0-2-43
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-415-release-v0-2-43
capabilityClaims: [release_distribution, repository_isolation]
sourceCommit: 107dfab6e6e331041a73fce7406f573bfbd7610c
canonical: docs/work-items/WI-415-release-v0-2-43.md
---

# WI-415 — v0.2.43 release

[English](WI-415-release-v0-2-43.md) · [简体中文](WI-415-release-v0-2-43.zh-CN.md)

## Intent

reviewed な WI-414 後の `main` から v0.2.43 を公開し、別 Work Item で行う
immutable public adopter acceptance のために clean な reviewed base を残します。

## Boundary

この Work Item は patch version、三言語の release/installation/versioning/parity 文書、
strict release source route の検証だけを扱います。Runtime governance semantics、
historical evidence、global Agent/MCP configuration、adopter application source は変更しません。
public-artifact acceptance は post-release の別 Work Item です。

## Acceptance

1. Cargo metadata と lockfile を v0.2.42 から v0.2.43 へ一度だけ進め、既存 tag/Release を再利用しません。
2. reviewed release workflow が reviewed commit、target archive、SBOM、manifest、Formula、SHA256SUMS、provenance、immutable tag/Release identity を正確に bind します。
3. English、简体中文、日本語の current release/installation/versioning/parity 文書を同期し、過去の release は historical として明示します。
4. post-release acceptance は isolated な別 Work Item で immutable public v0.2.43 artifact のみを使い、source/workspace fallback を許可しません。
5. reviewed merge、finalization、close、default branch 同期、正確な branch/worktree cleanup 後に `main` が `ready_on_base` になります。

## Verification boundary

pre-release は Contract が宣言する strict source/release gate を使います。staged candidate や
source build を public adopter evidence として提示しません。release/cleanup の失敗は可視のまま保持し、
published Release truth を書き換えません。

[English](WI-415-release-v0-2-43.md) · [简体中文](WI-415-release-v0-2-43.zh-CN.md)
