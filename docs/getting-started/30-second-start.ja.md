---
author: AI Cockpit maintainers
title: "30 秒で開始"
description: "Install 済み Runtime から repository attach までの最短で安全な route。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# 30 秒で開始

検証済みの immutable な public Runtime を使います。最初の repository-local write を
許可する前に、対象 repository を read-only で確認します。

```bash
repo=/path/to/repository
ai-cockpit inspect --repo "$repo"
ai-cockpit attach --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

`inspect` が read-only の最初の確認です。`attach` が作るのは repository-owned な
`.ai/` state だけで、Agent instruction や global MCP configuration は変更しません。
意図した Git checkout ではない、説明できない worktree change がある、または
`doctor` が `ok` でない場合は停止します。

次に[最初の calibration](first-calibration.ja.md)を終え、
[最初の Work Item](first-work-item.ja.md)へ進みます。binary install と digest verify は
[Installation](installation.ja.md)を参照してください。

[Getting started](README.ja.md) | [English](30-second-start.md) | [中文](30-second-start.zh-CN.md)
