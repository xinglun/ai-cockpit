---
author: AI Cockpit maintainers
title: "Getting started"
description: "共有 Runtime を install し、最初の repository を安全に attach する route。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# Getting started

新しい adopter repository では次の route を使います。

1. [Release と配布](../release/distribution.ja.md)に従い、immutable な public Release を install して digest を検証します。
2. `ai-cockpit inspect --repo /path/to/repository`、続けて `ai-cockpit attach --repo /path/to/repository` を実行します。
3. `ai-cockpit status --repo /path/to/repository` と `ai-cockpit doctor --repo /path/to/repository` を実行します。
4. 必要な場合だけ Agent adapter を install します。`attach` は Agent file や global MCP configuration を変更しません。
5. `ai-cockpit work-item new --repo /path/to/repository --id <id> --mode code` で `not_ready` skeleton を作成します。
6. Read-only profile candidate を review し、[最初の calibration](first-calibration.ja.md)を完了します。
7. [Adopter configuration](adopter-configuration.ja.md)で external review、security、CI decision を完了します。
8. 完全な Runtime-native [最初の Work Item](first-work-item.ja.md)を実行します。

Install は共有 Runtime の操作で、repository attach は明示的な操作です。attach は repository-local
`.ai/` を作成します。1 つの Runtime は複数 repository を扱えますが、Work Item、evidence、active context は共有しません。

## Reader routes

- [30 秒で開始](30-second-start.ja.md) — inspect、attach、status、doctor。
- [Installation](installation.ja.md) — immutable public Runtime と repository の分離。
- [Repository profile calibration](calibration.ja.md) — project-owned quality command 1 件の確認。
- [Standard adoption guide](standard-adoption-guide.ja.md) — 完全な adoption sequence。
- [Security と Release verification](security-release-verification.ja.md) — supply-chain/external evidence boundary。
- Examples: [Android](examples/android.ja.md)、[iOS](examples/ios.ja.md)、[Java](examples/java.ja.md)。

最初の Work Item 後は [Features](../features/README.ja.md) と
[Operations](../operations/README.ja.md) に進みます。

[Documentation home](../README.ja.md) | [English](README.md) | [中文](README.zh-CN.md)
