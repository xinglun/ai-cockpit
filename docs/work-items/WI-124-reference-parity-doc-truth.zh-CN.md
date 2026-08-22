---
author: AI Cockpit maintainers
workItemId: WI-124-reference-parity-doc-truth
title: 参考源对齐、文档事实与发布一致性
description: 让读者路线、参考源矩阵和运维基线与当前 Runtime 保持一致。
audience:
  - adopter
  - maintainer
status: implementation
authority: canonical
lastVerifiedBy: WI-124-reference-parity-doc-truth
---

# WI-124——参考源对齐、文档事实与发布一致性

## 意图

让 English、简体中文和日本語文档准确描述当前 Rust Runtime。运维入口必须展示完整
治理生命周期，参考源矩阵必须包含当前 WI-121/WI-122/WI-123 边界，发布一致性检查必须
从 Cargo metadata 推导当前基线版本。

## 范围

- 根 README 的生命周期路线和语言链接。
- 三语 reference-parity 页面及当前实现基线。
- 三语 Contract/Summary 字段映射页面，明确 Rust 边界。
- 三语 operations 页面和版本漂移检查。
- 文档与发布一致性回归脚本。
- 本 Work Item 的三语文档和 Runtime 生成的 receipts。

字段映射只投影当前 typed Rust Protocol，不新增 schema，也不宣称未实现的参考源字段。Rust
Runtime 行为、Agent/MCP 配置和历史 Work Item bytes 不在范围内。

## 验收

1. 三语根 README 都展示 `inspect → attach → start → preflight → checkpoint → verify → finish → archive → close`，并解释各门的语义。
2. Reference parity 将 WI-121、WI-122、WI-123 标为当前已实现边界，并提供证据和文档链接。
3. Contract/Summary 字段页面在三种语言中映射当前 Rust 字段，并明确标记 `Implemented`、`Partial`、`External` 边界。
4. Operations 页面描述当前 adopter target，不写死发布版本；发布脚本从 Cargo metadata 推导版本。
5. 文档和版本一致性脚本会在生命周期标记、parity 状态、基线 target、字段映射或过期版本漂移时失败。
6. 不修改 Runtime 功能和全局配置。

## 验证

```bash
bash tests/docs/documentation_acceptance.sh
bash tests/release/version_consistency.sh --repo .
git diff --check
```

最终 human Outcome 必须单独交付，并包含交通灯标记、unknowns、evidence、人类决定和下一步。
