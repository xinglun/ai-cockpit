---
author: AI Cockpit maintainers
title: "WI-367——v0.2.38 公开 Release adopter 验收"
workItemId: WI-367-release-adopter-v0-2-38
description: "使用不可变的公开 v0.2.38 产物，在隔离的新 adopter 工程中完成验收并保留可重复的证据基线。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-367-release-adopter-v0-2-38
terminalArchive: .ai/work-items/archive/WI-367-release-adopter-v0-2-38.contract.json
terminalVerification: .ai/evidence/WI-367-release-adopter-v0-2-38.verification.json
terminalFinalization: .ai/decisions/WI-367-release-adopter-v0-2-38.finalize.json
terminalDecision: .ai/decisions/WI-367-release-adopter-v0-2-38.close.json
capabilityClaims: [release_distribution, adopter_acceptance, repository_isolation]
---

# WI-367——v0.2.38 公开 Release adopter 验收

[English](WI-367-release-adopter-v0-2-38.md) · [日本語](WI-367-release-adopter-v0-2-38.ja.md)

## 意图

只使用不可变的公开 v0.2.38 Release binary 治理一个全新的 adopter 工程，证明发布验收与隔离边界，并为后续版本保留可重复基线。

## 范围与边界

- 运行公开 adopter 验收与升级脚本，不使用源码、workspace binary 或
  `cargo build`/`cargo run` 回退。
- 保留 Runtime identity、发布元数据、evidence reuse、完整 Work Item
  lifecycle、隔离 manifest 与清理证明。
- 确保 manifest helper 兼容 macOS Bash 3.2 与 Linux Bash。

Runtime 实现、CI workflow policy、全局 Agent/MCP 配置、历史 evidence 改写和
adopter 业务代码不属于本 WI。

## 验收

1. 记录并相互校验公开 v0.2.38 发布元数据、archive digest 和 binary digest。
2. 新 adopter 工程只由下载的 v0.2.38 binary attach 与治理。
3. 以 Runtime identity 记录 repository identity、Work Item lifecycle、evidence reuse 和 close decision。
4. HOME 与 XDG_CONFIG_HOME 保持不变；Runtime 写入目录隔离，临时运行根目录在成功与失败时均被删除。
5. 不使用源码 checkout、workspace binary 或 Cargo 回退。
6. acceptance receipt 与 checksum 可复现并适合作为后续发布基线。
7. 公开脚本在 macOS Bash 3.2 与 Linux Bash 上均不会发生 manifest deadlock。

## Evidence 与结果

- 公开 Release：[v0.2.38](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.38)
- 发布 workflow：[33195494850](https://github.com/xinglun/ai-cockpit/actions/runs/33195494850)
- 验收 evidence：`.ai/evidence/WI-367-release-adopter-v0-2-38/acceptance.json`
- Runtime verification：`.ai/evidence/WI-367-release-adopter-v0-2-38.verification.json`
- 隔离与清理 evidence：`.ai/evidence/WI-367-release-adopter-v0-2-38/isolation.json` 与 `cleanup.json`

公开 workflow 与本地不可变产物运行均通过。首次运行发现的 macOS portability
问题已在 `tests/release/isolation_manifest.sh` 修复，并由回归测试覆盖。
