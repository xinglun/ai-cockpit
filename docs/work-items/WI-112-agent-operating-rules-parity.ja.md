---
author: AI Cockpit maintainers
title: "WI-112 Agent operating rules parity"
description: "今後の Rust Work Item が継承する参照元の適用可能な Agent workflow 規則。"
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - agent_workflow_boundaries
---

# WI-112: Agent operating rules parity

## 目的

参照元にある Agent workflow、Outcome、review、release、安全境界のうち有用なものを
今後の Work Item が継承できるようにし、shared Rust Runtime と repository-local state
という本 project のモデルは維持します。

## 範囲

`AGENTS.md`、`.ai/README.md`、三言語の Agent workflow reference、reference index、
本 Work Item record を更新します。参照元の規則を継承、Rust project 向け適用、template
専用の除外に分類します。Runtime code、Protocol schema、global Agent/MCP configuration、
packaging、release asset は変更しません。

## 受入れ

- remote/default branch と immutable な公開 Release の境界を明記します。
- Contract、glossary、scope、Summary、evidence、checks、Outcome、問題解決、並行互換性、
  merge 後の closure 規則を今後の Work Item 向けに記録します。
- human Outcome は可視の `🔴`、`🟡`、`🟢` marker と fail-closed の進行条件を保持します。
- English、中文、日本語の reference page と Work Item record を同期し、link を検証します。
- 参照元固有の `make ai-*`、`contractVersion: 2`、V1 assumption は本 Rust project から
  明示的に除外します。

## 検証

```text
bash tests/docs/documentation_acceptance.sh
git diff --check
```

## Outcome

Status: **local implementation 完了。Runtime-bound lifecycle と documentation check が通過しました。**
