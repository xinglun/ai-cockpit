---
author: AI Cockpit 维护者
title: WI-407——Knowledge 投影物化边界
description: 使派生 Knowledge 投影显式、确定且保持仓库本地隔离。
workItemId: WI-407-knowledge-materialization-boundary
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-407-knowledge-materialization-boundary
terminalArchive: .ai/work-items/archive/WI-407-knowledge-materialization-boundary.contract.json
terminalVerification: .ai/evidence/WI-407-knowledge-materialization-boundary.verification.json
---

# WI-407——Knowledge 投影物化边界

## 意图

使 Knowledge 目录、索引、来源 digest 和刷新时机显式且可验证，同时不引入第二个治理权威。

## 范围

- CLI 与 MCP Knowledge 查询都报告仓库本地派生写入边界。
- 过期或损坏时，legacy 与 v2 投影保持确定性并可重建。
- Contract、evidence、archive 和 decision 继续作为权威来源。
- 英文、简体中文、日文文档同步说明同一边界。

## 证据

- 归档 Contract：`.ai/work-items/archive/WI-407-knowledge-materialization-boundary.contract.json`
- Verification：`.ai/evidence/WI-407-knowledge-materialization-boundary.verification.json`
- Pull Request：[ #372 ](https://github.com/xinglun/ai-cockpit/pull/372)

## 边界

Knowledge 是仓库本地的派生投影。显式查询可能物化或重建 `.ai/knowledge/`，但不会授权变更，也不会改变治理权威记录。生命周期命令不会静默物化它。
