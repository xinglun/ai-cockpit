---
author: AI Cockpit maintainers
title: "WI-412 — v0.2.42 release preparation"
description: "WI-411 後の reviewed Runtime を公開し、公開 adopter acceptance 用の clean base を残す。"
workItemId: WI-412-release-v0-2-42
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-412-release-v0-2-42
capabilityClaims: [release_distribution, repository_isolation]
---

# WI-412 — v0.2.42 release preparation

[English](WI-412-release-v0-2-42.md) · [简体中文](WI-412-release-v0-2-42.zh-CN.md)

## Intent

WI-411 後の reviewed `main` から v0.2.42 を公開し、別 Work Item で行う
immutable public adopter acceptance のための clean な reviewed base を残す。

## Boundary

この Work Item は patch version、三言語の release/versioning guidance、strict
release source route、reviewed lifecycle を扱う。Runtime governance semantics、
historical evidence、global Agent/MCP configuration、adopter application source は変更しない。
公開 artifact の adopter acceptance は別の post-release Work Item で行い、ここでは主張しない。

## Acceptance

1. Cargo metadata と lockfile を v0.2.41 から v0.2.42 へ一度だけ進め、既存 tag/Release を再利用しない。
2. reviewed workflow が exact reviewed commit、target archive、SBOM、manifest、Formula、SHA256SUMS、
   provenance、immutable tag/Release identity を bind する。
3. 現行 release、install、versioning、parity 文書を English・简体中文・日本語で同期し、過去の Release は明示的に historical とする。
4. post-release acceptance は別の isolated Work Item で immutable public v0.2.42 artifact のみを使い、source/workspace fallback を許可しない。
5. reviewed merge、finalization、close、default branch 同期、exact branch/worktree cleanup 後に `main` が `ready_on_base` になる。

## Verification boundary

pre-release は宣言済み strict source/release gate を使う。staged candidate や source build を public adopter evidence として扱わない。失敗は可視のまま保持し、公開済み Release truth を書き換えない。
