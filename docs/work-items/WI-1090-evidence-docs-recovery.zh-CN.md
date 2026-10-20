---
author: AI Cockpit maintainers
title: "WI-1090 — 证据位置与文档恢复"
description: "证据位置与文档恢复"
audience:
  - maintainer
  - reviewer
authority: explicit-user-authorization
workItemId: WI-1090-evidence-docs-recovery
status: in_progress
lastVerifiedBy: WI-1090-evidence-docs-recovery
---

# WI-1090-evidence-docs-recovery — 证据位置与文档恢复

本项为原生选定后继，修复 PR1050 实际出现的 Windows checkout 与文档失败。范围严格为八份 Agent 副本保真迁移、两个 locator 文件、六份前项文档投影及本项三语页面；不改变 Rust、测试、workflow、全局配置或前项生成历史。其余十九项长路径候选保持原路径与字节。

前项 `WI-1090-authority-source-material-review-replacement` 已归档，七项原生检查通过，托管 CI `39344530319` 失败；close 待处理。追加式选择记录为 `.ai/decisions/WI-1090-authority-source-material-review-replacement.recovery.json`。原 M6 manifest 保留逻辑键；八份迁移副本必须通过 `.ai/evidence/WI-1090-authority-source-material-review-replacement/m6-short-20261020/path-map.actual.json` 显式解析并核对字节、哈希及 Git blob。

本页及三份完整 ID parity 行在自身首次验证收据之前登记提交。必要检查为 `bash tests/docs/documentation_acceptance.sh`、`bash tests/docs/parity_status_check.sh`，另须实际迁移完整性审计。当前结果和生命周期以原生证据为准；登记本身不宣称检查通过或交付完成。

仅在 Runtime 实际准入时，允许自身正常 checkpoint、验证、controls、finish 与 archive。活动 Contract `.ai/work-items/active/WI-1090-evidence-docs-recovery.contract.json` 与归档 Contract `.ai/work-items/archive/WI-1090-evidence-docs-recovery.contract.json` 是生命周期替代位置，不表示同时存在。自身验证为 `.ai/evidence/WI-1090-evidence-docs-recovery.verification.json`；未来 finalize、close 记录分别为 `.ai/decisions/WI-1090-evidence-docs-recovery.finalize.json`、`.ai/decisions/WI-1090-evidence-docs-recovery.close.json`。

未授权任一工作项 close、合并、删除分支/工作树/Provider、发布/tag 或 Runtime 采用。PR1050 保留；独立 Draft PR 和实际 CI 是单独交付证据。材料扫描 Unknown 与 self_declared 保证限制继续明示。本项不组合 WI1089 与 WI1090，不操作 Sentinel，不证明真实消费者恢复。
