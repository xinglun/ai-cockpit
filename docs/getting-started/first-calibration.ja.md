---
author: AI Cockpit maintainers
title: "最初の calibration"
description: "Repository quality command を最初に review・confirm する route。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - configuration
---

# 最初の calibration

Installation、repository inspection、attach、正常な `doctor` result の後だけに実行します。
まず formal profile を変えずに candidate を得ます。

```bash
repo=/path/to/repository
ai-cockpit profile propose --repo "$repo"
```

Candidate を repository-owned documentation と hosted CI に照合します。Project owner が
executable、arguments、working directory、toolchain、environment、service、credential、
coverage の Unknown をすべて解決します。

Owner が承認した正確な command だけを confirm します。承認済み command が
`cargo test --workspace` の場合の例です。

```bash
ai-cockpit profile confirm --repo "$repo" --program cargo --args test,--workspace
ai-cockpit doctor --repo "$repo"
```

Local command の pass は bounded local evidence で、branch protection、provider、production、
enterprise evidence ではありません。Candidate が誤っている、または必要 fact が Unknown
なら confirm せず、repository-owned decision を修正して read-only proposal を再実行します。

[Adopter configuration](adopter-configuration.ja.md)を終え、
[最初の Work Item](first-work-item.ja.md)へ進みます。

[Calibration](calibration.ja.md) | [English](first-calibration.md) | [中文](first-calibration.zh-CN.md)
