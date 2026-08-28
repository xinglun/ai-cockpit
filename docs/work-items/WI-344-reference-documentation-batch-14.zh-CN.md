---
author: AI Cockpit maintainers
title: "WI-344——参考文档第 14 批"
workItemId: WI-344-reference-documentation-batch-14
description: "逐一比较五个固定参考验收/恢复文档，记录有界的 Rust 对应物，不导入源项目历史。"
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-344-reference-documentation-batch-14
terminalArchive: .ai/work-items/archive/WI-344-reference-documentation-batch-14.contract.json
terminalVerification: .ai/evidence/WI-344-reference-documentation-batch-14.verification.json
terminalFinalization: .ai/decisions/WI-344-reference-documentation-batch-14.finalize.json
terminalDecision: .ai/decisions/WI-344-reference-documentation-batch-14.close.json
capabilityClaims:
  - reference_parity
---

# WI-344——参考文档第 14 批

## 意图与边界

本 Work Item 逐一比较固定参考源中的五个下一批文档：恢复可用性、最终 North
Star 验收、源 WIII 修复审计和源完整修复基线，并记录它们是由 Rust 原生读者/Runtime
边界承接，还是源项目专属历史、不可导入。

范围仅包括 inventory 生成器/manifest、三语 comparison/parity 页面和本 Work Item
记录。Runtime 行为、源 Python/Make 工具、provider/全局 Agent 配置、不可变历史
evidence 以及 release/adopter 执行均不在范围内。

## 逐文件决定

| 固定参考路径 | 分类 | 有界目标决定 |
| --- | --- | --- |
| `docs/reference/failure-recovery-usability.md` | `implemented-different-by-design` | 仓库绑定的恢复、失败 gate/recovery condition、Task Outcome 和面向人的 handoff 提供当前边界。源九场景 Python 报告 wire shape 不复制；配套脚本/测试另行排期。 |
| `docs/reference/final-north-star-acceptance.json` | `implemented-different-by-design` | 目标 final-replacement acceptance 路线和精确 dimension/parity 记录保留证据及外部 adopter/provider 限制，不导入源 decision bytes。 |
| `docs/reference/final-north-star-acceptance.md` | `implemented-different-by-design` | Design Philosophy、Product Boundary、Outcome 和 final-replacement acceptance 保留 North Star；本地检查不能替代外部 evidence。 |
| `docs/reference/final-wiii-remediation-closure-audit.md` | `reference-only` | 源专属 WIII PR 身份、reviewer 和历史关闭声明不是目标证据。Rust 保留自己的 Work Item intelligence 与并行边界文档。 |
| `docs/reference/full-remediation-acceptance.md` | `reference-only` | 源 WI-01–WI-19 修复顺序属于内部历史。目标保留自己的 evidence-bound acceptance 路线，不发布源进度或 Release 声明。 |

这是语义/文档 parity，不是源命令或 JSON-wire parity。对象工程边界仍是一个共享
Runtime、按 repository 隔离的状态以及独立生成的 evidence。

## 验收与验证

- 五个路径在固定 inventory 中各出现一次，分类如上，且没有 deferred 或 migrate-gap。
- 英文、简体中文、日文 comparison/parity 页面表达相同决定和当前计数。
- 不复制或把源实现、内部进度历史、provider 身份或外部 evidence 提升为当前能力。
- inventory、文档、治理和锁定 workspace 检查通过。

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit a533d49dfa848d95742833f8cd1b5f7e1bb897d5 --check
bash tests/docs/documentation_acceptance.sh
bash tests/docs/getting_started_semantic.sh
python3 tests/ci/governance_integrity_gate.py --repo . --report target/wi344-governance-integrity.json
cargo test --locked --workspace
```

[English](WI-344-reference-documentation-batch-14.md) ·
[日本語](WI-344-reference-documentation-batch-14.ja.md)
