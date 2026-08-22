---
author: AI Cockpit maintainers
title: "WI-151——v0.2.16 发布后自治理验收"
description: "只使用不可变公开 v0.2.16 binary，验证安装后 AI Cockpit 可以治理本仓库。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-151-post-release-v0-2-16-self-governance
workItemId: WI-151-post-release-v0-2-16-self-governance
---

# WI-151——v0.2.16 发布后自治理验收

WI-151 是发布后的验收边界。它下载公开的 v0.2.16 aarch64 macOS archive，校验 checksum
和 archive layout，并在不使用源码或 workspace fallback 的情况下安装解压后的 binary。

安装 binary 身份为：

- version：`0.2.16`
- binary SHA-256：`0e9e9e85f3a96d22702cf95edab928bd2307c4636e53836bee46ca4e8cabf796`
- repositoryId：`sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b`

在显式 `--repo` 下，`inspect`、`status`、`doctor`、`agent doctor` 和全量 workspace verification
均通过。面向人的 Outcome 以 English、简体中文和日本語渲染，并显示可见的 `🟢` 标记和结构化
Human Decision。验收证据为 `.ai/evidence/WI-151-post-release-v0-2-16-self-governance.verification.json`；
决定记录为 `.ai/decisions/WI-151-post-release-v0-2-16-self-governance.close.json`。

发布 workflow 和公开 artifact 仍是发布事实的权威证据；本 Work Item 记录安装 Runtime 后的 adopter 结果。
