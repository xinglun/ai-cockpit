---
author: AI Cockpit maintainers
title: "WI-343——参考 inventory 基础对账"
workItemId: WI-343-reference-inventory-foundation-reconciliation
description: "将已完成比较的五个参考路径登记到机器 inventory，不改变 Runtime 行为，也不复制源工具。"
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-343-reference-inventory-foundation-reconciliation
capabilityClaims:
  - reference_parity
---

# WI-343——参考 inventory 基础对账

## 意图与边界

WI-339 已逐文件比较五个固定参考路径，但生成的 inventory 仍把它们标为
`deferred-next-batch`。本 Work Item 修正该台账缺口，使机器 inventory、三语比较页和
parity 登记表达相同的已审阅决定。

范围仅包括 inventory 生成器/manifest、比较与 parity 文档以及本 Work Item 记录。
Runtime 行为、源 Python/Make 工具、provider 集成、不可变历史 evidence、全局
Agent/MCP 配置和其他 deferred 路径均不在范围内。

## 逐文件决定

| 固定参考路径 | 分类 | 有界目标决定 |
| --- | --- | --- |
| `docs/reference/cross-wi-integration.md` | `reference-only` | 源聚合报告仅为 advisory；目标以每个 Work Item 的 archive、parity ledger 和面向人的 Outcome 作为审计边界。 |
| `docs/reference/dependabot-intake.md` | `not-applicable` | Dependabot bot 分支接入属于 provider 专属能力；通用 delegated evidence 与依赖事实仍由外部/仓库负责。 |
| `docs/reference/deprecated-assets-registry.json` | `reference-only` | 源清理 registry 不是可移植 Runtime 协议；目标边界是显式 lifecycle 与 resource finalization。 |
| `docs/reference/deprecated-assets.md` | `reference-only` | 源过时链说明保留为参考文档；Rust 不声称复制 `check-deprecated-assets` 命令。 |
| `docs/reference/derived-artifacts.md` | `implemented-different-by-design` | typed Contract/evidence/archive/status/Outcome projection 保持事实与视图分离；derived view 不能授权决定。 |

这是语义/文档 parity，不是源命令或 JSON-wire parity。对账器不复制源实现，也不自行生成治理决定。

## 验收

- 五个路径在固定 inventory 中各出现一次，分类与上表一致，且没有
  `deferred-next-batch` 或 `migrate-gap`。
- 在固定 source/target commit 上，inventory 生成与 `--check` 确定性通过。
- 英文、简体中文、日文比较/parity 页面表达相同的五项决定和当前计数。
- Runtime 行为、源工具、不可变 evidence、provider/全局配置保持不变。
- 声明的文档、inventory、治理和锁定 workspace 检查通过。

## 验证命令

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit a533d49dfa848d95742833f8cd1b5f7e1bb897d5 --check
bash tests/docs/documentation_acceptance.sh
bash tests/docs/getting_started_semantic.sh
python3 tests/ci/governance_integrity_gate.py --repo . --report target/wi343-governance-integrity.json
cargo test --locked --workspace
```

[English](WI-343-reference-inventory-foundation-reconciliation.md) ·
[日本語](WI-343-reference-inventory-foundation-reconciliation.ja.md)
