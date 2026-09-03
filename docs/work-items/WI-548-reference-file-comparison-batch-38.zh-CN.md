---
author: AI Cockpit maintainers
title: "WI-548 — 治理与边界脚本比较批次 38"
description: "逐个比较 13 个固定参考脚本，记录 Rust 原生或外部边界，不复制源实现。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
workItemId: WI-548-reference-file-comparison-batch-38
lastVerifiedBy: WI-548-reference-file-comparison-batch-38
terminalArchive: .ai/work-items/archive/WI-548-reference-file-comparison-batch-38.contract.json
terminalVerification: .ai/evidence/WI-548-reference-file-comparison-batch-38.verification.json
terminalFinalization: .ai/decisions/WI-548-reference-file-comparison-batch-38.finalize.json
terminalDecision: .ai/decisions/WI-548-reference-file-comparison-batch-38.close.json
---

# WI-548 — 治理与边界脚本比较批次 38

## 目标

在固定本地提交 `fde3380f81fea5fd2e288f7a8849f737dc074060` 上逐个阅读 13 个维护中的参考脚本，记录共享 Rust Runtime 与对象工程的语义对应和不声明边界。本批不复制 Python 模块、Make 编排、provider 状态或源 JSON wire 格式。

## 文件级结果

清单 `tests/conformance/reference_file_inventory.json` 为 13 个路径记录了分类、对应文件和理由；其中 detached uninstaller 与全局 disable/enable 明确为 `reference-only`，其余为 `implemented-different-by-design`。详细逐项表格见三语参考比较页。

## 发现与对象工程继承

本批未发现可移植实现遗漏。detached uninstaller 与全局 disable/enable 是有意保留的 source/provider 边界，不是缺少的 Runtime 能力。每个 attach 的对象工程继承同一共享 binary、显式 `--repo`、隔离 Contract/evidence/knowledge 和 human Outcome 规则，不继承源安装状态、Python registry 或对象工程专属 policy。

## 验收

- 清单在固定提交上准确记录本批 13 个现行路径，并为每项提供理由及对应或明确边界。
- 本批不再有 `deferred-next-batch` 或 `migrate-gap`；retired 历史保持追加式。
- 英文、简体中文、日文比较页与本 Work Item 表述一致。
- 清单、文档、格式、lint 和 workspace verification 在收尾前全部通过。
