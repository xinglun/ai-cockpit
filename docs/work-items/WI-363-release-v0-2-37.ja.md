---
author: AI Cockpit maintainers
title: "WI-363 — v0.2.37 release と install 済み binary の受入れ"
workItemId: WI-363-release-v0-2-37
description: "release-adopter cleanup の merge 後に immutable release を公開し、隔離 adopter flow で public binary を検証する。"
audience: [adopter, maintainer, reviewer]
status: recovered
authority: canonical
lastVerifiedBy: WI-363-release-v0-2-37
capabilityClaims: [release_distribution, adopter_acceptance]
---

# WI-363 — v0.2.37 release と install 済み binary の受入れ

[English](WI-363-release-v0-2-37.md) · [简体中文](WI-363-release-v0-2-37.zh-CN.md)

## Intent

release-adopter cleanup を merge し、synchronized な `main` から v0.2.37 を公開する。
その後、immutable な public artifact だけを install し、この repository で検証する。

## Scope と boundary

- workspace package、lockfile、現在の三言語 release/versioning 文書を v0.2.37 に揃える。
- reviewed hosted release workflow、public artifact、checksum、SBOM/provenance、adopter
  acceptance、N-1 upgrade evidence を使う。
- checksum 検証済みの public macOS ARM64 binary を install し、明示的 repository で health check を行う。
- 未公開の v0.2.36 staged acceptance failure は履歴として保持する。

Runtime behavior の変更、歴史 evidence の書換え、global Agent/MCP 設定、source-build fallback、
第二 technology-stack adopter はこの Work Item の範囲外である。

## Acceptance

1. Cargo metadata と lockfile が一貫して 0.2.37 を報告する。
2. immutable tag 前に CI と release policy check が成功する。
3. public artifact は GitHub から download して checksum に bind し、source/workspace binary に fallback しない。
4. public adopter と N-1 acceptance が監査可能な receipt を出し、Runtime/repository identity、隔離、lifecycle evidence、temporary root cleanup を証明する。
5. install 済み public binary が明示的 `--repo` で `inspect`、`status`、`doctor`、`agent doctor` を通過する。
6. merge、finalization、close、branch/worktree の正確な cleanup 後、repository が synchronized default branch で ready になる。

## Verification boundary

Runtime lifecycle が Contract、checkpoint、verification、archive、finalization、close evidence を記録する。
Hosted workflow receipt と post-release adopter receipt が public artifact claim の authority であり、
v0.2.36 の失敗 bytes は変更しない。
