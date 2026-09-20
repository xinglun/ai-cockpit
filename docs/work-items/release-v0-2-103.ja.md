---
author: AI Cockpit maintainers
workItemId: release-v0-2-103
title: verification receipt と配布ポリシー修正後の v0.2.103 リリース
description: review 済みソース、不変公開 artifact、adopter acceptance、正確な resource cleanup の証拠がそろった後だけ公開する。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: release-v0-2-103
---

[English](release-v0-2-103.md) · [简体中文](release-v0-2-103.zh-CN.md)

# v0.2.103 リリース

この WI は、有界 verification receipt 処理と Apple Silicon 専用 Homebrew 配布ポリシーを含む Runtime リリースを準備する。review 済み merge、不変 tag、公開 Release、downloaded-artifact checks、fresh install、upgrade、provider cleanup の証拠が完了するまで、公開成功を主張しない。

## 境界

- このリリースには、Sentinel が Issue #902 の acceptance を replay する前に必要な Runtime 側修正を含む。
- Intel Homebrew はサポート対象の配布または acceptance target ではない。standalone x86_64 macOS archive は引き続きサポートする。
- Sentinel その他の adopter repository は変更しない。
- 直接の証拠なしに、性能向上や第三者チャット画面での Outcome 表示を主張しない。
