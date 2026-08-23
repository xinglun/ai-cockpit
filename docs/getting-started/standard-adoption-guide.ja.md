---
author: AI Cockpit maintainers
title: "Standard adoption guide"
description: "Verified Runtime から最初の closed Work Item までの reader-first route。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# Standard adoption guide

次の stage を順に完了します。各 stage は独立した evidence boundary です。

1. Immutable な public Runtime を[install](installation.ja.md)し、正確な artifact を verify する。
2. 対象 repository を明示的に[inspect/attach](30-second-start.ja.md)する。
3. Owner-approved quality command 1 件で[最初の calibration](first-calibration.ja.md)を完了する。
4. [Adopter configuration](adopter-configuration.ja.md)の review、security、recovery、CI owner checklist を完了する。
5. 必要なら repository-local Agent adapter を明示的に install し、`agent doctor` で verify する。
6. 専用 branch/worktree と reviewed PR で[最初の Work Item](first-work-item.ja.md)を実行する。
7. Archive 前に human Outcome を表示し、merge 後に exact resource cleanup を verify して structured human close decision を記録する。

Installation、attach、profile confirmation、implementation、provider review、close を一つの暗黙
approval にまとめてはいけません。ある boundary の pass は次を証明しません。Unknown または
contradictory evidence は依存する claim だけを stop し、owner と recovery condition を示します。

Release trust と private mirror limit は[Security と Release verification](security-release-verification.ja.md)
を参照してください。Platform example は project fact を発明せず Unknown を保つ方法を示します。

[Getting started](README.ja.md) | [English](standard-adoption-guide.md) | [中文](standard-adoption-guide.zh-CN.md)
