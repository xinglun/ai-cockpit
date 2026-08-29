---
author: AI Cockpit maintainers
title: Upgrade
description: shared Runtime と repository attachment を更新する境界。project readiness とは別です。
audience:
  - adopter
  - maintainer
status: current
authority: translation
canonical: docs/reference/upgrade.md
lastVerifiedBy: WI-378-reference-documentation-batch-17
capabilityClaims:
  - runtime_upgrade
---

# Upgrade

[English](upgrade.md) · [简体中文](upgrade.zh-CN.md) · [日本語](upgrade.ja.md)

Installed Runtime の upgrade と repository schema migration は別の操作です。Runtime-only upgrade は通常、マシンの shared binary だけを変更し、repository の `.ai/` bytes は変更しません。Migration は plan、backup/rollback evidence、human decision を伴う明示的な reviewed repository Work Item です。

## Runtime upgrade

インストール前に immutable public Release archive を使い、manifest、SHA-256、runtime identity を検証します。新 binary が doctor と release acceptance を通るまで、rollback 用に現在の Runtime を残します。インストール後も各 repository には明示的な attach と request-scoped command が必要です。

```sh
ai-cockpit inspect --repo /path/to/project
ai-cockpit compatibility --repo /path/to/project
ai-cockpit doctor --repo /path/to/project
```

Runtime は commit、push、PR の open/merge、global Agent/MCP configuration の編集を行いません。managed adapter を変更する場合は target repository の別の明示的な `agent install` Work Item とします。

## Repository migration

まず `ai-cockpit migrate plan --repo <path>` を実行し、command が要求する明示的な approval 付きで reviewed plan だけを適用します。Migration は Contract、evidence、decision、knowledge、archive history を保全し、Runtime version の変更だけを理由に古い evidence を書き換えません。未完了/非互換の場合、read-only diagnostics は使えますが、stateful lifecycle write は fail closed です。

Reference source の installer、`Makefile.ai`、Python module、provider marker file は Rust repository にコピーしません。semantic boundary は shared external Runtime と isolated repository Protocol です。
