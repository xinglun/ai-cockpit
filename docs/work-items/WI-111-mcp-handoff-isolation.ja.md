---
author: AI Cockpit maintainers
title: "WI-111 MCP human handoff と release isolation evidence"
description: "repository-bound Outcome delivery と typed post-release isolation manifest。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: mcp-isolation-regression
capabilityClaims:
  - mcp_human_outcome_handoff
  - typed_release_isolation_evidence
---

# WI-111: MCP human handoff と release isolation evidence

## 目的

人間向け Outcome を Agent の正式な handoff にし、release adopter の isolation evidence で file、directory、
symlink、metadata、digest の変更を検出できるようにします。cleanup と repository binding は弱めません。

## 範囲

repository service が一つの human Outcome renderer を持ち、CLI と MCP が `outcome_v2` の検証後に同じ renderer
を呼び出します。MCP には明示的な repository-bound `work_item_outcome` tool を追加し、`work_item_get` は raw
machine record lookup のままです。tool は安定した `structuredContent.outcome` と可視の localized `humanHandoff`
を返します。Contract source text は保持し、human decision を推測しません。

release adopter と upgrade harness は typed isolation manifest を共有します。各 manifest は relative path、entry
type、mode/size/mtime metadata、regular file または symlink target の SHA-256 digest を記録します。HOME と
XDG_CONFIG_HOME は write forbidden root、TMPDIR と CARGO_HOME は明示的に分類された Runtime write root です。
receipt は before/after manifest digest と検証済み temporary root cleanup を bind します。

## 受入れ

- CLI と MCP は同じ renderer を使い、status marker、unknown、evidence、structured human decision projection、
  次の action を表示します。
- English、中文、日本語の MCP handoff を検証し、Contract の受入れ基準は原文のまま保持します。
- manifest regression は file content、directory、symlink target、metadata の変更と、残留 root のない cleanup
  を検証します。
- public v0.2.7 adopter acceptance は `isolation.json` schema 2、typed manifest、`cleanup.json`、directory
  `SHA256SUMS` を含めて通過します。
- repository-local Agent instruction に handoff と isolation boundary を記述し、global Agent/MCP configuration
  は変更しません。

## 検証

```text
cargo test --locked -p cockpit-mcp --test rpc -- --test-threads=1
cargo test --locked -p cockpit-cli --test intelligence --test outcome_human_decision -- --test-threads=1
bash tests/release/isolation_manifest_test.sh
bash tests/release/adopter_acceptance_test.sh
bash tests/release/adopter_upgrade_acceptance_test.sh
bash tests/docs/documentation_acceptance.sh
```

Public acceptance は download した v0.2.7 binary だけを使い、source や workspace binary に fallback しません。
strict typed verification evidence、foreign-runtime policy、historical evidence projection、external immutable
audit retention は後続の独立 Work Item で扱います。

## Outcome

状態：**local implementation 完了。MCP、CLI、documentation、manifest、public adopter acceptance の focused check
が通過しました。**
