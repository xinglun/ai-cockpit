---
author: AI Cockpit maintainers
title: "性能诊断"
description: "针对单个仓库 Work Item 的证据型治理成本诊断。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# 性能诊断

[English](performance-diagnosis.md) · [日本語](performance-diagnosis.ja.md)

性能诊断用于解释已测量的治理成本，不改变治理结论。Runtime 的 request-scoped
`diagnose` 输出和 verification cost 观察可以针对一个仓库及可选 Work Item 报告
快照工作、读取/哈希文件、验证运行、执行/复用节点、worker/进程数量、耗时和有限的
瓶颈提示。

报告必须区分：

- 执行和复用是物理观察；每个 Work Item 仍取得自己独立、身份绑定的证据收据；
- 本地进程耗时不能证明 provider 等待、人工等待、token 用量、发布耗时或 adopter
  加速；
- 格式错误、跨 Work Item、身份不一致或不完整的观察保持 unknown/partial，不能降低
  必要验证路线；
- 只有仓库、Runtime、profile、policy、command、stage 和输入身份均匹配时才能比较。

源 JSONL parser 及其报告 wire shape 不是 Runtime 协议要求。AI Cockpit 不臆造 P95、
provider 等待或企业性能声明。参见[治理成本指标](governance-cost-metrics.zh-CN.md)
和[治理配置](governance-profiles.zh-CN.md)。

相同的 advisory 边界适用于每个显式 `--repo` 的 adopter：性能事实是本地遥测，
不是全局项目状态，也不是跳过检查的权限。
