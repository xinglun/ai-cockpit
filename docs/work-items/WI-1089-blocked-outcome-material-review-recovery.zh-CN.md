---
author: AI Cockpit maintainers
title: "WI-1089 — 阻塞 Outcome 的材料审阅恢复"
description: "失败 Outcome 与敏感 Contract 修改后的有限恢复。"
audience:
  - maintainer
  - reviewer
authority: explicit-user-authorization
workItemId: WI-1089-blocked-outcome-material-review-recovery
status: in_progress
lastVerifiedBy: WI-1089-blocked-outcome-material-review-recovery
---

# WI-1089 — 阻塞 Outcome 的材料审阅恢复

敏感 Contract 修改后，持久化的失败 Outcome 仍为红色。当前 preflight 与身份绑定的真人
`confirm_review` 只确认当前 Contract，不接受材料扫描的残余风险。只有源代码为干净、
已提交的 Rust，扫描结果恰为一个允许审阅的语法 Unknown、没有 Finding，且当前请求与
Contract 的材料审阅政策精确匹配时，Runtime 才能提供
`record_material_review_decision`。材料决定独立作出，仍需对应审阅权限。

有效的材料收据可在正常前置条件下开启新的验证。真实通过的验证收据写入前，当前失败
Outcome 仍为红色；随后当前投影才会更新，追加式失败事件仍可读取。在此之前 finish 与
archive 仍被阻止。脏源码、不可用的
材料投影、Finding、矛盾证据、过期或外来审阅输入及错误的恢复决定均继续被拒绝。

当前实现范围仅含 repository 状态投影、repository/CLI/MCP 回归测试和本 WI 三语
文档。reference-parity 登记需要单独的 Contract 范围决定。
