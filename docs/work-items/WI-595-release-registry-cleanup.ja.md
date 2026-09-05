---
author: AI Cockpit maintainers
title: "WI-595 — release registry cleanup"
description: "WI-594 close 後に stale な pending-parity projection を削除します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-595-release-registry-cleanup
lastVerifiedBy: WI-595-release-registry-cleanup
---

[English](WI-595-release-registry-cleanup.md) · [简体中文](WI-595-release-registry-cleanup.zh-CN.md)

# WI-595 — release registry cleanup

## Objective

`docs/reference/pending-parity-registry.json` から close 済み WI-594 の stale
entry を削除し、三言語 parity projection を現在の Runtime record と一致させます。
過去の `.ai/` bytes は immutable のまま保持します。

## Boundary

pending registry、parity projection、WI-594/WI-595 の readable documentation
だけを変更します。Runtime behavior、release artifact、object repository、global
Agent/MCP configuration は対象外です。

## Verification

明示的な repository context で JSON parser、`tests/docs/parity_status_check.sh`、
tag-mode governance integrity、documentation acceptance、status consistency を実行します。
