---
author: AI Cockpit maintainers
title: "文档权威边界"
description: "面向人和 Agent 的读者优先文档归属。"
audience: [user, agent, maintainer]
status: current
authority: canonical
lastVerifiedBy: WI-548-reference-file-comparison-batch-38
---

# 文档权威边界

规范的 Agent 读取集属于仓库本地：`.ai/README.md`、`.ai/glossary.md`、`AGENTS.md` 以及当前绑定仓库的机器可读 `.ai` 记录。先读 `docs/current/README.md`，再按需要阅读 `docs/getting-started/README.md` 的采用流程和 `docs/reference/README.md` 的命令与语义。各语言页面相互链接；翻译只是展示，不是第二套 policy。

current 与 reference 页面说明受支持行为。`docs/archive/**` 下的历史材料仅提供上下文，除非人类明确写入 Work Item Contract，否则不产生当前 authority。源模板计划、Python 脚本、Make 目标和生成报告是比较 evidence，不是本 Rust 仓库的使用指令。

文档检查会校验 frontmatter、链接、语言对应页、parity 行和终态 evidence，但不会静默晋级草稿或推断治理决定。说明边界或限制时，必须明确对应 Runtime 命令、Contract 字段或 evidence 引用；不能宣称对象工程继承源专属 installer、provider policy 或 wire 格式。

Agent 应在行动前查询 Runtime 状态（`inspect`、`status`、`doctor`），以当前 Work Item Contract 为 authority，并在交付时展示可见的 human Outcome。所有 attach 的对象工程共享这条路线，但每个 `--repo` 的事实和决定仍完全隔离。
