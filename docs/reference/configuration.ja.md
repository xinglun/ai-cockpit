---
author: AI Cockpit maintainers
title: "Configuration reference"
description: "Repository-owned TOML configuration、profile state、generated Work Item file。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - configuration
---

# Configuration reference

Repository configuration format は TOML です。JSON には変更しません。

## `.ai/cockpit.toml`

`attach` は次の最小 file を作ります。

```toml
protocol_version = 1
repository_id = "sha256:<64 lowercase hexadecimal characters>"
```

`repository_id` は最初の attach 時に生成され、以後の request は repository-owned file から読み取ります。
absolute path の hash ではないため、attach 済み repository を移動しても evidence は別 repository になりません。
runtime は両方を validate し identity mismatch を拒否します。runtime source や V1 file を `.ai/` に copy しません。

## `.ai/project.json`

`attach` は `state: "calibration_required"` の attached profile を作ります。`profile confirm` 後に profile version が
増え、選択した quality command が verified として記録されます。wrapper は `profileVersion`、`repositoryId`、`state`、
`profileDigest`、`tests`、`buildSystems` を持ちます。unknown profile field は拒否されます。

## Work Item record

`start` は `.ai/work-items/active/` に次を生成します。

- `<id>.contract.json` — intent、scope、authority、acceptance、required evidence、base revision、profile digest、repository snapshot digest。
- `<id>.summary.json` — lifecycle state と checkpoint count。

`verify --work-item <id>` は `.ai/evidence/<id>.verification.json` を書きます。`finish` は outcome、`archive` は archive manifest、
`close` は human decision を記録します。green に見せるため手編集してはいけません。

Cross-process reusable evidence は runtime が `.ai/evidence/reuse/` で管理します。schema、identity binding、resource limit は
[Protocol v1](../protocol/v1/specification.ja.md) を参照してください。
