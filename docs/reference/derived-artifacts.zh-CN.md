---
author: AI Cockpit maintainers
title: "派生工件与权威边界"
description: "说明 Rust Runtime 的投影如何保持可观察但不成为治理权威。"
audience: [user, agent, maintainer]
status: current
authority: canonical
lastVerifiedBy: WI-548-reference-file-comparison-batch-38
---

# 派生工件与权威边界

AI Cockpit 区分仓库事实和由事实派生的视图。Contract、repository
snapshot、verification receipt、decision 与 archive manifest，只有在类型、身份和摘要绑定通过校验时才具有权威性。status、summary、Outcome handoff 与 knowledge index 是面向人和 Agent 的派生投影；投影不能授权变更，也不能替代源记录。

参考模板用 Python registry 校验生成事实及其输入。Rust Runtime 保留可移植规则——输入必须显式、来源必须可追溯、派生必须确定、身份必须 fail-closed——但不复制该 registry 或其 JSON wire 格式。仓库本地 Knowledge 同样只是读取/派生视图，不能替代 Contract、Evidence 或 human Decision。

审计时先读源记录，再核对投影：

1. `ai-cockpit inspect --repo <repo>` 建立 snapshot 与 changed paths。
2. `ai-cockpit status --repo <repo>` 展示生命周期事实和 readiness。
3. `ai-cockpit work-item outcome --repo <repo> --id <id>` 输出面向人的 handoff；它不是新的决策。

若投影与源记录不一致，Runtime 会报告绑定或新鲜度问题并停止。Agent 不得手改 generated status、Outcome、knowledge、evidence 或 archive；应在明确的人类 authority 下修改所属 Contract 或执行所属 Runtime 操作。

每个 attach 的对象工程通过共享 binary 与显式 `--repo` 继承同一边界，但不会继承源 Python registry 或源专属生成文件 policy。
