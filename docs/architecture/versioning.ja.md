---
author: AI Cockpit maintainers
title: "バージョニング"
description: "Runtime と Repository Protocol の version identity と migration boundary。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - versioning
---

# バージョニング

Runtime version と Repository Protocol version は独立しています。

```text
ai-cockpit --version
0.1.0

repository:
protocol_version = 1
```

CLI version は executable package を示し、protocol version は repository storage contract
を示します。Runtime version、runtime digest、protocol version は `inspect`、`doctor`、MCP
`initialize`、verification evidence などの identity-bearing surface で一緒に提供されます。
`--version` は短い package-version command であり、完全な identity envelope を返す約束ではありません。

Runtime upgrade は Protocol 1 を継続サポートしたまま capability を追加できます。Protocol 1 →
Protocol 2 だけが repository migration です。Historical Work Item は decision boundary で使った
Project Profile digest と protocol version を保持します。Major migration は個別に review する
Work Item とし、旧 evidence を保持します。
