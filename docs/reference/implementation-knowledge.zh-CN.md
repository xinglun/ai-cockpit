---
author: AI Cockpit 维护者
title: 实现知识
description: 面向已完成 Work Item 的确定性、证据绑定知识记录。
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: translation
canonical: docs/reference/implementation-knowledge.md
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - evidence_bound_knowledge
---

# 实现知识

[English](implementation-knowledge.md) · [简体中文](implementation-knowledge.zh-CN.md) · [日本語](implementation-knowledge.ja.md)

实现知识是已验证并归档 Work Item 的派生投影，不是 Agent 记忆、第二事实源或设计权威。
Contract、验证证据、归档和最终 Outcome 仍是权威记录。

## 查询

```text
ai-cockpit knowledge query --repo /path/to/repository \
  --topic <topic> --component <component> \
  --state verified --work-item-id <id>
```

Runtime 对提供的条件采用 AND 语义，返回稳定且绑定仓库的记录。`--v2` 返回包含 truth state、confidence、证据引用、unknowns 和 snapshot digest 的 `KnowledgeV2Record`。
显式查询可能在仓库本地的 `.ai/knowledge/` 下物化或重建派生索引；响应会报告
`projection.materialization`、`projection.path` 和
`projection.writeBoundary=repository-local-derived`。这次写入不会授权新变更，也不会修改 Contract、evidence、archive 或 decision 权威记录。

生命周期命令不会静默物化 Knowledge。派生索引缺失、损坏、过期或不完整时，只有显式查询路径会从归档来源重建并重新验证，或明确返回 partial/unknown。
source digest 仅用于缓存校验；归档记录仍是事实来源。

## 与参考源的明确差异

参考文档还描述日期、合并提交、`latestKnownRecord` 和显式 supersession 过滤器。当前 Rust 投影只公开上面的仓库绑定过滤器；这些维度不会被静默推断，也不属于本版本 CLI/MCP contract。未来增加时必须有独立 Contract、Schema、测试和三语文档。

Knowledge 不是语义搜索、向量检索、模糊推荐、RAG，也不保证新仓库具有相同实现。空结果不证明主题从未处理；日期、supersession 和收益只有在证据中明确记录时才显示。

## 共享 Runtime 与对象工程继承

安装的 Runtime 可以共享，但每次查询都必须带显式 `--repo`。索引、记录、证据和 adapter 状态保留在该仓库的 `.ai/` 中；对象工程继承的是只读证据边界，不是参考仓库的生成记录或 Python/Make 命令。
