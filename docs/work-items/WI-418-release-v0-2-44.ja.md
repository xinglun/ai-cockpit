---
author: AI Cockpit maintainers
title: WI-418 — v0.2.44 release
description: lockfile-aware Cargo 検証選択を含む reviewed Runtime を公開する。
workItemId: WI-418-release-v0-2-44
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-418-release-v0-2-44
terminalArchive: .ai/work-items/archive/WI-418-release-v0-2-44.contract.json
terminalVerification: .ai/evidence/WI-418-release-v0-2-44.verification.json
terminalFinalization: .ai/decisions/WI-418-release-v0-2-44.finalize.json
terminalDecision: .ai/decisions/WI-418-release-v0-2-44.close.json
---

# WI-418 — v0.2.44 release

[English](WI-418-release-v0-2-44.md) · [简体中文](WI-418-release-v0-2-44.zh-CN.md)

## Intent

lockfile-aware Cargo 検証 command の修正後に reviewed な `main` を v0.2.44 として公開し、
Release identity と三言語 documentation を同期する。

## Scope と boundary

この Work Item は patch release と strict release source route の検証だけを扱う。governance
semantics、reference/V1 Runtime や installer のコピー、global Agent/MCP configuration、
adopter application source は変更しない。public-artifact adopter acceptance は release 後の
別 Work Item で実施する。

## Acceptance

- Cargo metadata と lockfile を v0.2.43 から v0.2.44 へ一つだけ patch 更新し、tag/Release を再利用しない。
- reviewed workflow が exact commit、target archive、SBOM、manifest、Formula、checksum、
  provenance、immutable tag/Release identity を bind する。
- release、installation、versioning、parity の guidance を English、Simplified Chinese、日本語で同期する。
- 後続の隔離 Work Item は immutable な public v0.2.44 artifact だけで adopter acceptance を行う。
- reviewed merge、finalization、close、同期、exact cleanup の後に `main` が `ready_on_base` になる。

## Verification boundary

release 前は宣言した strict source/release gate を使い、staged candidate や source build を
public adopter evidence として扱わない。
