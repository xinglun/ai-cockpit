---
author: AI Cockpit maintainers
title: WI-404 — Release documentation terminal promotion
description: 不変な lifecycle evidence が揃った後だけ、完了した Work Item の文書を終状態へ昇格します。
workItemId: WI-404-release-docs-terminal-promotion
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-404-release-docs-terminal-promotion
---

# WI-404 — Release documentation terminal promotion

この Work Item は v0.2.41 の Release quality gate が見つけた documentation
projection の不整合を修正します。完了済み WI-402 の三言語文書と parity 行だけを
昇格し、不変な `.ai` evidence や decision bytes は書き換えません。

## Boundary

- WI-402 の三言語 Work Item ページと三言語 reference parity 行だけを更新します。
- archive、verification、finalization、close は不変の evidence reference として保持します。
- Runtime semantics の変更や Release 公開は行いません。

## Verification

インストール済み Runtime が repository-bound verification evidence を記録します。
文書、promotion、parity、inventory、workspace 全体のチェックを review 前に通過させます。
