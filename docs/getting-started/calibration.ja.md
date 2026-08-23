---
author: AI Cockpit maintainers
title: "Repository profile calibration"
description: "Repository facts を推測せず、project-owned quality command を確認する。"
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - configuration
---

# Repository profile calibration

Attach は候補 build system を検出できますが、repository の quality baseline command を
決定しません。現在の profile と read-only candidate を確認します。

```bash
repo=/path/to/repository
ai-cockpit status --repo "$repo"
ai-cockpit profile propose --repo "$repo"
```

Proposal は適用済み change ではありません。Repository owner が working directory、
executable、arguments、toolchain、credential boundary、coverage、hosted CI counterpart を
確認します。manifest、project file、wrapper があるだけで command を推測しないでください。

Review 後に正確な project-owned command を確認します。次は owner が
`cargo test --workspace` を選んだ Rust repository だけの例です。

```bash
ai-cockpit profile confirm --repo "$repo" --program cargo --args test,--workspace
ai-cockpit status --repo "$repo"
```

別 stack では承認された program/arguments を使います。Calibration は toolchain install、
provider authentication、hosted CI proof を行いません。Unknown は Unknown のまま保持し、
依存する claim を block します。

[最初の calibration](first-calibration.ja.md) | [English](calibration.md) | [中文](calibration.zh-CN.md)
