---
author: AI Cockpit maintainers
title: "排查与恢复"
description: "常见 AI Cockpit 停止状态及安全下一步。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
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
| `finish_ready` Work Item 在 archive 前变成 stale | verification 绑定的 snapshot 之后 repository 发生了变化。 | 不要编辑 summary 或 receipt。保留历史 bytes，基于当前 snapshot 创建新的授权 Work Item。 |
| stale predecessor 不应再重试 | 其 evidence 已是历史记录，且已有绑定 successor 接管工作。 | 记录绑定 identity 的 `supersede` recovery decision，再把 predecessor 作为历史项 archive/close；不要重写或重新验证旧 evidence。 |
| `archive`/`close` 失败 | governance 非 green 或 archive identity 非法。 | 保留 active 记录，修复 evidence 后重试失败步骤。 |
| Verification 重新执行而非 reuse | identity binding 变化或 reuse 未授权。 | 把 rerun 当作安全行为，检查 receipt reason。 |
| MCP 要求 repository binding | server 未用 repository-bound adapter 启动。 | 配置 `mcp --repo <path>` 并保持路径显式。 |
| Release asset/tag 不存在 | 公开分发证据尚未就绪。 | 停止安装，等待不可变 Release 和匹配制品。 |

不要删除 `.ai` 记录、receipt 或 `index.pending` 来让状态看起来干净。缺失、malformed、过期或
矛盾的 evidence 会按设计 fail closed。

已经处于 `finish_ready` 的 Work Item 没有隐式 rewind 操作。这是有意的：
rewind 会使状态历史含义不明确。应为变化后的 snapshot 创建 successor Work
Item，并把旧 receipt 作为历史 evidence 引用。
