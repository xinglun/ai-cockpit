---
author: AI Cockpit maintainers
title: "多语言语义一致性"
description: "语言投影保持治理事实一致，不翻译权威 Contract 原文。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# 多语言语义一致性

[English](multilingual-semantic-parity.md) · [日本語](multilingual-semantic-parity.ja.md)

英语、简体中文和日语是同一份仓库绑定 Runtime 事实的展示投影。三种语言
中的固定标题、状态标签、停止/下一步提示、风险信号、限制和人工决定字段，
必须表达相同含义。

CLI 测试覆盖三种语言的稳定 marker 和摘要。语言投影不得：

- 把黄色或红色证据变成绿色；
- 臆造批准、收益、能力或 provider/企业声明；
- 遗漏阻塞项、未知项、必要检查、安全警告或恢复动作；
- 翻译或改写 acceptance criteria、intent、scope 等人类拥有的 Contract 值。

Contract 值保留其编写语言，并明确标注为原文。只有 Runtime 自有的展示文本
进行本地化。Agent adapter 可以提供额外的非权威翻译，但原值及其 digest/引用
始终是治理来源。

这是语义一致性，不是源 wire 或 Python comparator 的兼容承诺。每个投影仍使用
显式 `--repo` 和仓库本地证据；语言选择不能改变治理决定。
