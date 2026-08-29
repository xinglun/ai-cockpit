---
author: AI Cockpit maintainers
title: "WI-376 — v0.2.39 发布 adopter 验收"
description: "用不可变公开 Release 验证当前仓库与全新独立 adopter。"
workItemId: WI-376-release-adopter-acceptance
audience: [maintainer, reviewer]
status: completed
authority: human-authorized
lastVerifiedBy: WI-376-release-adopter-acceptance
capabilityClaims: [release_acceptance, repository_isolation, evidence_reuse]
---

# WI-376 — v0.2.39 发布 adopter 验收

[English](WI-376-release-adopter-acceptance.md) · [日本語](WI-376-release-adopter-acceptance.ja.md)

## 目标

证明不可变的 v0.2.39 公开 Release 可以同时治理当前仓库和全新的独立
adopter，且仓库状态不共享、不依赖源码 checkout。

## 范围与边界

- 校验公开下载归档、binary 摘要、manifest 与校验和。
- 校验 v0.2.39 在当前仓库中的 Runtime 能力继承。
- attach 全新 adopter，检查脚手架，并完成带有效证据和结构化关闭决定的
  Work Item 生命周期。
- 证明精确证据复用、快照变化后的重新执行、全局目录隔离；保存可审计验收
  回执后清理临时状态。

Runtime 新功能、源码或 workspace binary fallback、第二种技术栈，以及全局
Agent/MCP 配置不在本 Work Item 范围内。

## 验收标准

1. 下载的 v0.2.39 归档和 binary 与 `release-manifest.json`、`SHA256SUMS`
   一致。
2. 当前仓库为 `COMPATIBLE` 且 `ready_on_base`；`doctor` 正常，
   `runtimeCodeInRepository` 为 false，Agent doctor 为 `VERIFIED`。
3. 全新 adopter 拥有不同的 `repositoryId`，只生成最小仓库脚手架。
4. 新 Work Item 骨架保持 `not_ready`；Runtime 不擅自填写人工意图、范围、
   验收标准和授权。
5. adopter 生命周期生成绑定仓库、快照、Work Item、Runtime 身份和关闭决定的
   schema-2 证据。
6. 完全相同的重复验证不执行节点；快照变化后重新执行验证。
7. 禁止写入的全局目录保持不变，Runtime 写入仅在隔离目录内。
8. 验收产物包含 Runtime 身份、JSON 输出、复用/隔离证明、生命周期证据和校验
   和；完成后删除临时 adopter 与运行根目录。

## 验证边界

测试对象只有已发布的 Release。验收回执属于发布后的证据，不会修改不可变的
Release truth。

## 结果

已将 v0.2.39 公开归档和 binary 与 release manifest、checksums 进行校验。
当前仓库继承 Runtime 0.2.39，`inspect`、`status`、`doctor` 和 Agent doctor
均健康。全新的独立 adopter 获得了不同的 repository identity，保留
`first-adopter-smoke` 为 `not_ready`，并完成 schema-2 verification/finish/archive/
finalize/close 生命周期。完整 Release 与 adopter 回执保存在
`release-adopter-acceptance-artifacts/`；固定 adopter 路径和隔离的重试目录已在
采集后删除。
