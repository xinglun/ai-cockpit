---
title: "WI-626 — v0.2.85 release と adopter acceptance"
description: "失敗した v0.2.84 境界を保持し、監査可能な後継 Release を公開する。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-626-release-v0-2-85
lastVerifiedBy: WI-626-release-v0-2-85
terminalArchive: .ai/work-items/archive/WI-626-release-v0-2-85.contract.json
terminalVerification: .ai/evidence/WI-626-release-v0-2-85.verification.json
terminalFinalization: .ai/decisions/WI-626-release-v0-2-85.finalize.json
terminalDecision: .ai/decisions/WI-626-release-v0-2-85.close.json
---

[English](WI-626-release-v0-2-85.md) · [简体中文](WI-626-release-v0-2-85.zh-CN.md)

# WI-626 — v0.2.85 release と adopter acceptance

## 目的

失敗した immutable な `v0.2.84` 公開境界を保持したうえで、レビュー済み Runtime を `v0.2.85` として公開し、
immutable な Release artifact だけで公開インストールと upgrade を検証する。

## 境界

この Work Item は version metadata、release/distribution 文書、失敗 Release の recovery projection、
Release acceptance interface を対象とする。対象リポジトリの変更、reference scaffold/Python/Make のコピー、
global Agent/MCP configuration の変更は行わない。

## 受け入れ条件

1. Workspace package と `Cargo.lock` が `0.2.85` になる。
2. Release CI が annotated `v0.2.85` tag、archive、checksum、SBOM/provenance、Formula、Runtime identity を公開する。
3. 公開後の adopter/N-1 harness は immutable な `v0.2.85` と `v0.2.83` artifact だけを使い、隔離と run-root cleanup を証明する。
4. English、中文、日本語の release/versioning/parity 記録が `v0.2.85` と `v0.2.83` 境界を示す。
5. terminal Outcome は人間向けに表示され、status、unknowns、evidence、human decision、next action を記録する。

## 検証

merge 前に workspace test と release/documentation policy check を実行する。公開後は immutable adopter/N-1 acceptance
harness を実行して download した Runtime identity を記録し、source や workspace binary を Release の代替にしない。
