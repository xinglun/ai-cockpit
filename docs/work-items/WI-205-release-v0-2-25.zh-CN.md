---
author: AI Cockpit maintainers
title: "WI-205——v0.2.25 发布与 transition 兼容性恢复"
description: "在前驱 base identity 漂移后，从同步的默认分支重新建立 v0.2.25 发布边界。"
audience:
  - maintainer
  - adopter
workItemId: WI-205-release-v0-2-25
status: in_progress
authority: canonical
lastVerifiedBy: WI-205-release-v0-2-25
---

# WI-205——v0.2.25 发布与 transition 兼容性恢复

WI-205 是 WI-204 的 successor。前驱从仍开放的前驱分支启动，无法真实绑定
pull request 的 base，因此保留其 archive 与失败的 finalization 尝试为不可变历史。
本 Work Item 在 verification 与 archive 之前记录同步的 `origin/main` base，
然后完成 v0.2.25 公开发布边界。

adopter 验收只接受不可变的 v0.2.25 Release 资产。回执必须绑定 Release identity、
下载 binary 与 N-1 证据、隔离 root manifest、清理证明、append-only transition
和终态 human decision。v0.2.24 仍是发布前失败历史。

文档入口：[English](WI-205-release-v0-2-25.md) · [日本語](WI-205-release-v0-2-25.ja.md)

## 验收边界

1. v0.2.25 版本、文档和 parity 一致。
2. 不可变公开 Release 具有完整 manifest、checksum、archive、SBOM、Formula
   和 provenance evidence。
3. 下载的 v0.2.25 在无源码 fallback 的隔离环境中通过 adopter 与
   v0.2.23→v0.2.25 N-1 验收。
4. 安装的 v0.2.25 接受并记录 append-only finalization transition。
5. WI-204 recovery、base 与 Runtime identity、evidence reuse、isolation、cleanup
   和三语 Outcome 可审计。
