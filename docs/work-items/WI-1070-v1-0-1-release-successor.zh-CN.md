---
author: AI Cockpit maintainers
title: "WI-1070 — v1.0.1 稳定版发布后继项"
description: "修复获批的 WI-1068 parity 行，补齐三语 Work Item 文档，并保留分阶段的 v1.0.1 发布验收。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-1070-v1-0-1-release-successor
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1070-v1-0-1-release-successor
contractDigest: sha256:b92b2eecb57e68f7943b54f67bca249158d31c55824146c47827cb8ea43f2657
---

[English](WI-1070-v1-0-1-release-successor.md) · [日本語](WI-1070-v1-0-1-release-successor.ja.md)

# WI-1070 — v1.0.1 稳定版发布后继项

本页是当前 [WI-1070 Contract](../../.ai/work-items/active/WI-1070-v1-0-1-release-successor.contract.json) 的读者版投影，绑定 `sha256:b92b2eecb57e68f7943b54f67bca249158d31c55824146c47827cb8ea43f2657`。状态、证据、准入和生命周期决定以 Runtime 记录为准。

## 目标与阶段顺序

目标是提供准确的英文、简体中文和日文发布参考，并提供经验证的 v1.0.1 稳定版制品、安装和 N-1 升级。这些目前只是预期收益；对应证据通过前不得报告为已交付。

源码阶段的 finish 门槛包括三语文档、canonical promotion 检查、文档验收、严格源码质量路线、8 项 `cognitive_benefit` 集成测试，以及 successor 精确源码上的完整 `cargo test --locked --workspace -- --quiet`。集成测试须用同一个私有不可变 `CARGO_BIN_EXE_ai-cockpit` 副本运行 Python 与 Rust，并保留完整 JSON/Markdown 相等性及二进制摘要/路径断言。workspace 验证使用 `RUST_TEST_THREADS=2`、`CARGO_BUILD_JOBS=1`、`CARGO_PROFILE_TEST_DEBUG=0`、`CARGO_INCREMENTAL=0`、一个 Runtime worker 和有限的 900 秒显式 timeout ceiling。Contract 将该 ceiling 绑定到五个支持的验证阶段（`task`、`pre_ci`、`pr`、`merge`、`release`）；完整 workspace 命令本身仍在 task stage 执行。未指定 timeout 时仍为默认 300 秒。推送后，PR 精确 head 必须先通过必需 CI 与 release-plan 检查，才能标记为可合并或正常合并。只有正常合并且 Runtime 新鲜准许发布后，才能一次性创建 v1.0.1 tag；随后现有工作流才构建并验收候选包。四目标候选安装/冒烟和从稳定版 v1.0.0 开始的 Linux x86_64 分阶段升级必须先于公开发布。官方公开制品、公开 Linux 安装/N-1、Apple Silicon macOS CLI/MCP 验收及最终化/关闭均属于后续门槛。

## 范围与边界

完整源码范围为以下六个文档文件和现有 cognitive-benefit 集成测试：

- `docs/reference/reference-parity.md`
- `docs/reference/reference-parity.zh-CN.md`
- `docs/reference/reference-parity.ja.md`
- `docs/work-items/WI-1070-v1-0-1-release-successor.md`
- `docs/work-items/WI-1070-v1-0-1-release-successor.zh-CN.md`
- `docs/work-items/WI-1070-v1-0-1-release-successor.ja.md`
- `crates/cockpit-cli/tests/cognitive_benefit.rs`

生产源码和 CI policy 不在范围内。唯一的测试改动限于 A6 中 `crates/cockpit-cli/tests/cognitive_benefit.rs` 的私有二进制隔离；仅在对当前 Contract 完成新鲜 preflight 并取得明确真人确认后实施，且不改断言。保留所有 WI-1068 历史和 WI-1069 记录。不得覆盖 tag、Release 或不可变制品。稳定版 v1.0.0 是 N-1 前序版本；创建前必须确认 v1.0.1 tag 和 Release 均未使用。

## Contract 验收标准

- **A1** 原样保留 WI-1068 Contract、Summary、Outcome、events、archive、verification evidence 和此前所有成功/失败结果；追加 successor 证据，不重标历史结果。
- **A2** 使用 canonical Runtime promotion projection，仅修复英文、简体中文、日文 reference-parity 文档中的三条获批 WI-1068 行；保留无关行。
- **A3** 添加本组三语 WI-1070 页面以及三份 parity 文档中的 WI-1070 行，使其绑定最终 Contract，并保留 WI-1069 记录和 worktree。
- **A4** 在精确 successor base/head 上通过 documentation acceptance、canonical promotion `--check-all` 和严格质量路线，不留 stale 或 pending parity 条目。
- **A5** 生成新的 Runtime release plan，绑定 PR #1022 及其精确合并源码、稳定版 v1.0.0 N-1、尚未使用的 v1.0.1 tag/Release 名称、provider immutability 元数据和 workflow gates。
- **A6** 在 `RUST_TEST_THREADS=2` 下，用同一个私有不可变 `CARGO_BIN_EXE_ai-cockpit` 副本运行 Python 与 Rust，完整保留 JSON/Markdown 相等性及 `runtimeBinaryDigest`/路径断言，并使完整 8 项 `cognitive_benefit` 集成测试全部通过。随后以 `RUST_TEST_THREADS=2`、`CARGO_BUILD_JOBS=1`、`CARGO_PROFILE_TEST_DEBUG=0`、`CARGO_INCREMENTAL=0`、一个 Runtime worker 和有限的 900 秒显式 timeout ceiling；该 Work Item 的 `modify_source` 验证策略绑定 `task`、`pre_ci`、`pr`、`merge`、`release` 五个阶段，完整 workspace 命令在 task stage 执行。通过精确接受源码上的 `cargo test --locked --workspace -- --quiet`。省略 timeout 时仍使用默认 300 秒。保留此前所有失败作为历史证据。
- **A7** PR 标记可合并或正常合并前，PR 精确 head 必须通过所有必需 GitHub Actions、Rust Contract、repository-quality、package-coverage、Windows Runtime 和 behavioral-oracle gates。head 改变后需重新跑精确 head CI；保留 PR #1022 早前的 revision-binding 失败。
- **A8** 正常合并且 Runtime 新鲜准许发布后，创建一次不可变 v1.0.1 tag。公开 Release 前，四目标 `aarch64-apple-darwin`、`x86_64-unknown-linux-gnu`、`aarch64-unknown-linux-gnu`、`x86_64-pc-windows-msvc` 候选安装/冒烟及从 v1.0.0 开始的 Linux x86_64 N-1 均须通过。
- **A9** 发布后核验官方 manifest、SHA256SUMS 和 Linux x86_64 制品身份/摘要；公开安装和从 v1.0.0 开始的 N-1 升级必须通过。保留失败下载，不覆盖不可变制品。
- **A10** 在本机 Apple Silicon 上安装官方 macOS 制品并核验 archive、manifest、checksum、已安装 SHA-256、版本、绝对 CLI 路径、MCP initialize 和只读请求；不得用本地构建替代。
- **A11** 只有所有必需发布/平台步骤成功且 Runtime 准许 cleanup 后，才记录精确 branch/worktree finalization 与 lifecycle 回执并关闭。WI-1068 历史保持不变。
- **A12** 在对应文档及发布/安装/升级证据通过前，多语言文档和经过验证的 v1.0.1 稳定版仍是预期收益。

本页没有声称候选版、公开发布、安装、N-1、macOS/MCP 或关闭已完成。此处不声称已创建 tag、公开发布或完成用户可见交付。
