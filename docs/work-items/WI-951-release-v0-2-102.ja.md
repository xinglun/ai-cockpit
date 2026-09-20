---
author: AI Cockpit maintainers
workItemId: WI-951-release-v0-2-102
title: cleanup 完了と verification snapshot lifecycle 修正後の v0.2.102 リリース
description: review 済みソース、不変公開 artifact、adopter acceptance、正確な resource cleanup の証拠がそろった後だけ公開する。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-951-release-v0-2-102
---

[English](WI-951-release-v0-2-102.md) · [简体中文](WI-951-release-v0-2-102.zh-CN.md)

# WI-951 — v0.2.102 リリース

この WI は、review 済みの direct-merge finalization 修正と repository の正確な cleanup の後に Runtime を公開する。不変 tag、公開 Release、downloaded artifact の検証、fresh install、upgrade、provider cleanup の証拠が完了するまで、公開成功を主張しない。

## 境界

- downstream acceptance replay に必要な verification snapshot lifecycle 修正を含む。
- Sentinel その他の adopter repository は変更しない。
- 直接の証拠なしに、性能向上や第三者チャット画面での Outcome 表示を主張しない。
