---
author: AI Cockpit maintainers
title: "WI-43 — Runtime 互換性と Repository Migration Protocol"
description: "Runtime-only upgrade と明示的な repository migration の実装境界と利用手順。"
audience:
  - maintainer
  - adopter
status: current
authority: canonical
lastVerifiedBy: implementation-acceptance
capabilityClaims:
  - runtime_upgrade_boundary
  - repository_migration
---

# WI-43 — Runtime 互換性と Repository Migration Protocol

## 目的

共有 Runtime をアップグレードしても repository の governance state を黙って変更しないこと。
Runtime-only upgrade は `.ai/` を変更せず、repository schema の変更は明示的に review、承認し、receipt に bind します。

## 利用手順

```bash
ai-cockpit compatibility --repo /path/to/repository
ai-cockpit migrate plan --repo /path/to/repository
ai-cockpit migrate apply --repo /path/to/repository --approved
```

互換性 state は次のとおりです。

- `COMPATIBLE`: 通常の lifecycle、Agent、MCP、verification を実行できる；
- `MIGRATION_REQUIRED`: inspect と read-only plan だけを許可し、state/evidence を書く操作を停止する；
- `INCOMPATIBLE`: 保存された schema を理解する Runtime が入るまで fail closed で停止する。

現在の Repository Protocol は version 1、repository schema の target は version 2 です。
旧 schema は legacy state として読み取り、`status`、`attach`、通常の Runtime 呼び出しで自動更新しません。

## Receipt と保持規則

適用した migration は `.ai/migrations/<migration-id>.json` に source/target schema、前後 digest、Runtime version、
Runtime digest、変更ファイル、result を記録します。変更対象は versioned protocol file と migration record だけです。
Archive Work Item、evidence、decision、knowledge、その他の履歴は変更しません。Runtime に global current repository や
global Work Item state はありません。

## Acceptance

- 旧 schema は version 1 として扱われ `MIGRATION_REQUIRED` になる；
- `migrate plan` は read-only で human approval を明示する；
- `--approved` なしの `migrate apply` は失敗し bytes を変更しない；
- 承認済み migration は `COMPATIBLE` となり Runtime-bound receipt を作る；
- 二重適用は拒否される；
- historical evidence と archive Work Item の bytes は不変である；
- すべての repository command は明示的な `--repo` を要求する。
