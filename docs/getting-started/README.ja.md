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
6. [Features](../features/README.ja.md) と [Operations](../operations/README.ja.md) に進みます。

Install は共有 Runtime の操作で、repository attach は明示的な操作です。attach は repository-local
`.ai/` を作成します。1 つの Runtime は複数 repository を扱えますが、Work Item、evidence、active context は共有しません。

[Documentation home](../README.ja.md) | [English](README.md) | [中文](README.zh-CN.md)
