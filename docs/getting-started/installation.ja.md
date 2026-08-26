---
author: AI Cockpit maintainers
title: "AI Cockpit をインストールする"
description: "Repository を暗黙に attach せず、共有 Runtime を install・verify する。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
---

# AI Cockpit をインストールする

AI Cockpit は各 project に governance tree をコピーする方式ではなく、外部の共有 Runtime
です。[Release と配布](../release/distribution.ja.md)に従って immutable な public Release
を選び、正確な target の artifact を取得し、install 前に SHA-256 を検証します。

Install 済み executable は別に確認します。

```bash
ai-cockpit --version
```

Install だけでは `.ai/` の作成、project quality command の選択、Agent adapter の導入、
hosted CI の証明、production readiness の判断は行いません。これらは独立した reviewable
な repository 操作です。

この Rust Runtime は reference template の 10 段階 Interactive Installer Wizard を意図的に
提供しません。Install は immutable Release の境界であり、repository onboarding は `inspect`、
`attach`、profile の proposal/confirmation、`doctor` を使って明示的かつ非暗黙に行います。
Provider や Agent adapter は独自の conversation UI を提供できますが、これらの repository-bound
operation を呼び出す必要があり、preview や prompt だけで approval を作ることはできません。

Install 後は read-only-first route を使います。

```bash
repo=/path/to/repository
ai-cockpit inspect --repo "$repo"
ai-cockpit attach --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

報告された facts を review し、[最初の calibration](first-calibration.ja.md)と
[Adopter configuration](adopter-configuration.ja.md)へ進みます。private mirror や
local source checkout は public Release evidence ではありません。
[厳格な installation security](installation-security.ja.md)も確認してください。

[Getting started](README.ja.md) | [English](installation.md) | [中文](installation.zh-CN.md)
