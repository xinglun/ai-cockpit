---
author: AI Cockpit maintainers
title: "排查与恢复"
description: "常见 AI Cockpit 停止状态及安全下一步。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-512-reference-docs-batch-33
capabilityClaims:
  - recovery
---

# 排查与恢复

| 现象 | 含义 | 安全下一步 |
| --- | --- | --- |
| `state: unattached` | 没有有效 `.ai/cockpit.toml`。 | 检查目标后运行 `attach --repo <path>`。 |
| `calibration_required` | profile 已检测但未由人确认。 | 检查命令后运行 `profile confirm`。 |
| Preflight `yellow` | 证据缺失或需要确认。 | 阅读 blockers/safe actions，修复 Contract 或取得所需决定。 |
| Preflight `red` | scope、authority、protocol 或 repository 状态非法。 | 停止；修复指定事实或权限。 |
| `finish` 报 receipt missing/stale | 没有当前 snapshot 的 passed Work Item verification。 | 最终编辑后运行 `verify --work-item <id>`，不能绕过检查。 |
| `finish_ready` Work Item 在 archive 前变成 stale | verification 绑定的 snapshot 之后 repository 发生了变化。 | 不要编辑 summary 或 receipt。如果该 Work Item 仍负责这项工作，使用绑定 identity 的原生 retry，然后重新运行 preflight、verification 并显式 finish。如果 Runtime 拒绝 retry，或已有授权 successor 接管，保留旧 receipt 并沿用绑定的 successor 路径。 |
| stale predecessor 不应再重试 | 其 evidence 已是历史记录，且已有绑定 successor 接管工作。 | 记录绑定 identity 的 `supersede` recovery decision，再把 predecessor 作为历史项 archive/close；不要重写或重新验证旧 evidence。 |
| `archive`/`close` 失败 | governance 非 green 或 archive identity 非法。 | 保留 active 记录，修复 evidence 后重试失败步骤。 |
| `close` 提示 retained resources require cleanup | receipt 可能是旧版共享主 checkout 记录，也可能是普通 retained linked resource。 | 确认 `provider=local` 及主 checkout 事实；无法确定时运行 `work-item finalize-recovery-plan` 并记录显式 historical recovery receipt。不要手动把 `retained` 改为 `deleted`。 |
| 历史 direct-merge 提示 base 与 Contract 不一致 | bundled merge 的真实第一 parent 可能不同于 Work Item 冻结时的 Contract base。 | 重新运行 `work-item finalize-recovery-plan --merge-commit <sha>`；保留 `pullRequest.baseRevision` 为真实第一 parent，保留 `historical.contractBaseRevision` 为归档 Contract base，并复制准确的 `resourceContext`/branch/worktree identity。不要修改任一历史事实或虚构 PR。 |
| 历史 direct-merge 提示 `resourceContext.<field>` 不匹配 | receipt identity 中有字段与 Contract 或 plan 输出不同。 | 以 `finalize-recovery-plan` 输出的完整 `suggestedReceipt` 为起点，不要重建协议字段；只填写列出的人工字段，保持 context/branch/worktree 原值后重试。拒绝时 Runtime 不会写入半成品记录。 |
| `finalize` 或 `finalize-recovery` 拒绝 Runtime 自己的 direct-merge 计划 | 计划生成器与应用校验器发生了契约分歧。 | 将计划和失败输入保留在 `.ai/` 之外，不要修改生成记录或虚构 PR，并报告 Runtime 版本、摘要和完整错误。完整计划收据必须能通过两个应用入口往返验证。 |
| successor 材料计划列出 predecessor 曾审阅的 Unknown | 该上下文只说明精确的历史匹配；它不是继承的决定，也不满足 successor 请求。 | 按正常流程审阅当前请求。变化或新增的路径、blob、hunk、cause、Contract scope、scanner、policy 或 profile 仍未解决。 |
| Verification 重新执行而非 reuse | identity binding 变化或 reuse 未授权。 | 把 rerun 当作安全行为，检查 receipt reason。 |
| MCP 要求 repository binding | server 未用 repository-bound adapter 启动。 | 配置 `mcp --repo <path>` 并保持路径显式。 |
| Release asset/tag 不存在 | 公开分发证据尚未就绪。 | 停止安装，等待不可变 Release 和匹配制品。 |

不要删除 `.ai` 记录、receipt 或 `index.pending` 来让状态看起来干净。缺失、malformed、过期或
矛盾的 evidence 会按设计 fail closed。

## 安装与工具链边界

如果 `attach`、`profile confirm` 或 `agent doctor` 停止，先检查它指出的仓库事实，再使用显式 `--repo` 重跑同一命令。Runtime 不会安装或切换项目的 JDK、Gradle、Xcode、CocoaPods、Node 或其他外部工具链。项目命令缺失属于 adopter 配置问题，不能因此削弱 Contract 或改用 workspace binary。

仓库存在 active Work Item 时，应在升级或创建新 Work Item 前先 finish/archive。如果 linked worktree、远端默认分支或 finalization receipt 缺失，应停止并保留记录；恢复必须使用绑定 identity 的 successor/retry 路径。参考源的 Make/Python wizard 命令不是 Rust Runtime 命令；请使用已安装 CLI 和当前仓库声明的 verification commands。

已经处于 `finish_ready` 的 Work Item 不能仅通过记录 verification 来刷新为新的 finish。
如果变化仍属于原 Work Item，使用已准入且绑定 identity 的 retry，重新运行 fresh
preflight 和 verification，再显式 finish。如果 retry 被拒绝或绑定 successor 已接管，
保留 predecessor evidence 并沿用既有 successor 路径。
