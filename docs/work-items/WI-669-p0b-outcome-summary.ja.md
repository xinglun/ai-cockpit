---
author: AI Cockpit maintainers
title: "WI-669 — P0-B Outcome summary と完全な evidence view"
description: "CLI と MCP に決定的な reader-first Outcome summary と明示的な完全 evidence view を提供します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-669-p0b-outcome-summary
lastVerifiedBy: WI-669-p0b-outcome-summary
---

[English](WI-669-p0b-outcome-summary.md) · [简体中文](WI-669-p0b-outcome-summary.zh-CN.md)

# WI-669 — P0-B Outcome summary と完全な evidence view

## Intent

既定の human Outcome の読解コストを下げながら、verification、lifecycle、
human decision、evidence、不確実性の校正境界を保ちます。summary は CLI と
MCP で共有する決定的な規則で生成し、完全な audit view は明示的に表示できます。

## Boundary

この Work Item は表示層だけを変更します。machine JSON、validation rule、
authorization semantics、exit code、永続化形式、historical evidence は変更しません。
測定済みの認知効果や実ユーザー研究も主張しません。

## Evidence

- Archive: `.ai/work-items/archive/WI-669-p0b-outcome-summary.contract.json`
- Verification: `.ai/evidence/WI-669-p0b-outcome-summary.verification.json`
- Hosted delivery: [PR #668](https://github.com/xinglun/ai-cockpit/pull/668)

## Current state

reviewed PR、provider finalization、明示的な human close decision が完了するまで、
この Work Item は In progress のままです。
