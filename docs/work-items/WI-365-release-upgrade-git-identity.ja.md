---
author: AI Cockpit maintainers
title: "WI-365 — release upgrade Git identity"
workItemId: WI-365-release-upgrade-git-identity
description: "隔離 CI 環境で public-to-staged N-1 acceptance の commit を決定的にする。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-365-release-upgrade-git-identity
terminalArchive: .ai/work-items/archive/WI-365-release-upgrade-git-identity.contract.json
terminalVerification: .ai/evidence/WI-365-release-upgrade-git-identity.verification.json
terminalFinalization: .ai/decisions/WI-365-release-upgrade-git-identity.finalize.json
terminalDecision: .ai/decisions/WI-365-release-upgrade-git-identity.close.json
capabilityClaims: [release_distribution, adopter_acceptance]
---

# WI-365 — release upgrade Git identity

[English](WI-365-release-upgrade-git-identity.md) · [简体中文](WI-365-release-upgrade-git-identity.zh-CN.md)

## Intent

clean な CI runner で cloned control repository に Git identity がなく、N-1
adopter acceptance の commit が失敗した問題を根治します。

## Scope と boundary

- script が commit するすべての harness repository に deterministic な
  repository-local Git identity を設定します。.git/config のみを書き換え、
  global Git configuration は変更しません。
- isolated な `HOME`/`XDG_CONFIG_HOME` と `GIT_CONFIG_GLOBAL=/dev/null` で、
  初期 repository と clone 後 control repository の commit を検証する regression
  を追加します。
- immutable artifact、isolation、cleanup、fail-closed acceptance の既存境界を維持します。

Runtime semantics、hosted workflow policy、global Git/Agent configuration、無関係な
release behavior は対象外です。

## Acceptance

1. upgrade harness のすべての commit path が明示的な repository-local identity を持ち、
   global Git configuration を変更しません。
2. global configuration を無効化した regression が、初期 repository と clone 後 control
   repository の両方の commit 成功を証明します。
3. success/failure path が acceptance truth と cleanup evidence を保持し、検証済みの
   temporary run root だけを削除します。
4. release shell tests、documentation checks、workspace quality checks が pass します。

## Verification boundary

installed Runtime が Contract、preflight、checkpoint、verification、finish、archive、
finalization、close evidence を記録します。公開 Release と N-1 acceptance は immutable
な external release evidence であり、post-release acceptance failure が publish truth を
書き換えることはありません。
