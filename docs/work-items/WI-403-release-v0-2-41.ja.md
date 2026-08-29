---
author: AI Cockpit maintainers
title: "WI-403 — v0.2.41 release publication and adopter acceptance"
description: "性能 batch 後の Runtime を公開し、immutable な public artifact を current repository と新規 adopter で検証します。"
workItemId: WI-403-release-v0-2-41
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-403-release-v0-2-41
capabilityClaims: [release_distribution, adopter_acceptance, runtime_installation]
---

# WI-403 — v0.2.41 release publication and adopter acceptance

[English](WI-403-release-v0-2-41.md) · [简体中文](WI-403-release-v0-2-41.zh-CN.md)

## Intent

Rust performance batch 後、review 済みで同期された `main` から v0.2.41 を公開し、immutable な public artifact が current repository と新規 adopter を governance できることを証明します。

## Boundary

対象は patch version の release、release/distribution documentation、immutable artifact の install、post-release adopter acceptance です。governance semantics、reference parity implementation、business code、global Agent/MCP configuration は変更しません。source build、workspace binary、moving branch の fallback は受け入れません。

## Acceptance

1. Cargo metadata と lockfile を v0.2.40 から v0.2.41 へ正確に更新します。
2. reviewed main から公開 Release を作り、archive、SBOM、provenance、manifest、SHA-256 identity を一致させます。
3. download した public binary を明示的な repository context で install し、`inspect`、`status`、`doctor` が healthy であることを確認します。
4. isolated な新規 adopter で attach、scaffold、lifecycle、evidence reuse、cleanup を完了し、`first-adopter-smoke` は `not_ready` のままにします。
5. Runtime/repository identity、artifact digest、isolation manifest、acceptance output、cleanup truth を evidence として保持します。
6. 英語・簡体字中国語・日本語の release/parity docs を同期し、exact cleanup 後の main が `ready_on_base` になります。

## Verification boundary

Release publication と adopter acceptance は別の truth です。post-release acceptance が失敗した場合は `releasePublished: true` と `adopterAcceptance: failed` を記録し、公開済み Release や historical evidence を書き換えません。download した binary の Runtime identity を release evidence に bind する authority は acceptance harness だけです。
