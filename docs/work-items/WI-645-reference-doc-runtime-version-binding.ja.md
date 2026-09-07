---
author: AI Cockpit maintainers
title: WI-645 — reference comparison Runtime version binding
description: 現在の reference-comparison Runtime projection を machine-readable metadata と同期させる。
audience: [maintainer, reviewer, adopter]
workItemId: WI-645-reference-doc-runtime-version-binding
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-645-reference-doc-runtime-version-binding
---

# WI-645 — reference comparison Runtime version binding

[English](WI-645-reference-doc-runtime-version-binding.md) · [简体中文](WI-645-reference-doc-runtime-version-binding.zh-CN.md)

## Intent

6 つの三言語 reference-comparison page の current snapshot に表示する Runtime
version を修正し、人間向け projection が `reference-comparison-metadata.json`
に bind される regression assertion を追加します。

## Boundary

この Work Item は current documentation projection と static metadata check
だけを変更します。Runtime behavior、Protocol bytes、immutable history、pinned
reference checkout、adopter repository は変更しません。

## Acceptance and verification

- 6 page の current snapshot Runtime version と binary digest が metadata sidecar と完全一致し、各一回だけ現れる。
- metadata test は stale current-snapshot version で失敗し、current metadata で成功する。
- 三言語 documentation、inventory、governance integrity、workspace check が historical record を変更せず成功する。

Work Item closure 後に archived page から authoritative Contract、verification、finalization、decision record を参照します。
