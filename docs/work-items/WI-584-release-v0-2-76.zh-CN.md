---
author: AI Cockpit maintainers
title: "WI-584——v0.2.76 发布与对象工程恢复交接"
description: "发布并验证归档 Work Item 重验证所需的 Runtime。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-584-release-v0-2-76
lastVerifiedBy: WI-584-release-v0-2-76
terminalArchive: .ai/work-items/archive/WI-584-release-v0-2-76.contract.json
terminalVerification: .ai/evidence/WI-584-release-v0-2-76.verification.json
terminalFinalization: .ai/decisions/WI-584-release-v0-2-76.finalize.json
terminalDecision: .ai/decisions/WI-584-release-v0-2-76.close.json
---

[English](WI-584-release-v0-2-76.md) · [日本語](WI-584-release-v0-2-76.ja.md)

# WI-584——v0.2.76 发布与对象工程恢复交接

## 目标

从已审查并同步的默认分支发布具有 identity binding 的 v0.2.76 Runtime
基线。该版本是对象工程执行 append-only 归档 Contract 重验证与后继关闭验收的
Runtime 依赖；本 Work Item 不操作对象工程。

## 范围与边界

- 将 Cargo 元数据、lockfile 和三语发布/版本说明同步到 v0.2.76，同时保留
  v0.2.75 作为前一个公开基线。
- 生成并验证绑定 identity 的发布 archive、manifest、checksums、SBOM、
  provenance、attestation 和 Runtime identity。
- 在隔离目录中只用下载的不可变制品执行 public adopter 与 N-1 验收，包含禁止
  写入目录和临时运行目录清理证明。
- 把对象工程恢复明确记录为外部只读交接依赖；此处绝不修改其 `.ai/`、源码、分支
  或 evidence。

Runtime 行为、对象工程、全局 Agent/MCP 配置、参考源复制、失败 tag 历史及无关历史
记录均不在本 Work Item 范围内。

## 验收

1. Cargo 元数据、lockfile 和发布/版本页面标识 v0.2.76，并保留 v0.2.75 作为前一个
   公开基线。
2. Release CI 为 v0.2.76 生成绑定 identity 的五目标制品和供应链 receipt。
3. public adopter 与 N-1 验收只使用下载的 v0.2.76 制品，证明隔离和临时目录清理，
   并用同一 binary 验证本仓库。
4. 不修改 Runtime 行为、对象工程、全局配置、失败 tag 历史或无关 evidence。
5. 为对象工程团队记录公开 Runtime identity 与准确命令交接；不重写或伪造历史证据。

## 验证边界

Contract 的 acceptance 以编写语言为权威；本地化页面只改变呈现。对象工程恢复是外部
只读交接，本 Work Item 不宣称其已完成。

## 验证

- `tests/release/version_consistency.sh`
- `tests/release/workflow_policy.sh`
- `tests/release/action_runtime_policy.sh`
- `tests/release/source_archive_policy_test.sh`
- `tests/release/adopter_acceptance_test.sh`
- `tests/release/adopter_upgrade_acceptance_test.sh`
- `tests/docs/documentation_acceptance.sh`
- `tests/docs/parity_status_check.sh`
- `cargo test --locked --workspace`
- `git diff --check`
