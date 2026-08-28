---
author: AI Cockpit maintainers
title: "WI-359 — 同期済み main から v0.2.36 を公開"
workItemId: WI-359-release-v0-2-36
description: "完全に同期した default branch から cleanup fix を公開し、実際の public artifact を検証する。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-359-release-v0-2-36
capabilityClaims: [release_distribution, adopter_acceptance]
---

# WI-359 — 同期済み main から v0.2.36 を公開

[English](WI-359-release-v0-2-36.md) · [简体中文](WI-359-release-v0-2-36.zh-CN.md)

## Intent

review 済み・merge 済み・同期済みの default branch から cleanup fix を v0.2.36 として公開します。v0.2.35 の失敗公開は immutable history として保持します。

## Scope

- workspace package、lockfile、三言語の release/versioning 文書を v0.2.36 に揃える。
- WI-358 の finalize と close を含む同期済み main にだけ tag を付ける。
- hosted release workflow の実際の public artifact、checksum、SBOM、provenance、adopter、cleanup evidence を使う。
- macOS ARM64 の正確な public binary を install し、明示した repository で health check を行う。

## Boundary

v0.2.35 を移動・削除・再標識せず、失敗した workflow の事実を書き換えません。runtime behavior、global Agent/MCP 設定を変更せず、release acceptance に source-build fallback を使いません。

## Acceptance

1. 全 workspace package と `Cargo.lock` が 0.2.36 で、version consistency を通過する。
2. review 済み merge と同期済み default branch の確認後だけ v0.2.36 を tag する。
3. public workflow が strict source quality、全 target build、artifact binding、temporary-root cleanup proof 付き adopter acceptance を通過する。
4. download した public binary の checksum/digest が release manifest と一致し、install 済み binary が 0.2.36 を報告し、明示的な `--repo` で inspect/status/doctor/agent doctor が通る。
5. v0.2.36 タグ作成前に、マージ済みのすべての配布ブランチと worktree を同期して正確にクリーンアップし、残存するマージ済みブランチや checkout がない。
6. v0.2.35 は失敗公開履歴として残り、Release として扱わない。

## Verification

Runtime lifecycle evidence、review 済み PR、hosted release workflow、public Release manifest/checksum、installed binary digest、adopter acceptance receipt が authoritative record です。
