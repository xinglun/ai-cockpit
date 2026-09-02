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
lastVerifiedBy: WI-512-reference-docs-batch-33
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

repository migration や managed-file replacement の前に、active Work Item を意図せず変更しない
ことを確認します。Migration plan には対象 path、schema/version transition、backup location、
rollback condition、human decision を明記します。plan が欠落、malformed、conflict、stale の
場合は write を停止します。Runtime upgrade だけで新しい project profile を有効化したり、
repository を ready と判断したりはしません。

project-owned または diverged な governance file を検出した場合は現在の bytes を保持し、
review 用の conflict report を出します。ファイルの上書きや generated evidence の手編集は禁止です。
Cursor rule を含む managed Agent adapter は ownership と detach を持つ明示的な repository-local
install であり、Runtime upgrade が暗黙に注入するものではありません。

active Work Item がある、remote の既定 branch を確立できない、managed file が diverged、target が downgrade、または conflict report が欠落/不正の場合は write 前に停止します。conflict を解消するか明確な base evidence を用意して retry してください。`--upgrade-with-active` は意図的で別途 review された recovery scenario の場合だけ使います。

## Repository migration

まず `ai-cockpit migrate plan --repo <path>` を実行し、command が要求する明示的な approval 付きで reviewed plan だけを適用します。Migration は Contract、evidence、decision、knowledge、archive history を保全し、Runtime version の変更だけを理由に古い evidence を書き換えません。未完了/非互換の場合、read-only diagnostics は使えますが、stateful lifecycle write は fail closed です。

Reference source の installer、`Makefile.ai`、Python module、provider marker file は Rust repository にコピーしません。semantic boundary は shared external Runtime と isolated repository Protocol です。

したがって reference source の installer/Make command は説明用の reference material です。
installed binary、immutable Release evidence、明示的な `--repo` command を使用してください。
