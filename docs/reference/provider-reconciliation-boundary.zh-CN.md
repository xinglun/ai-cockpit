---
author: AI Cockpit maintainers
title: "Provider 对账边界"
description: "历史 provider 清单是证据上下文，不是当前 provider 事实。"
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# Provider 对账边界

[English](provider-reconciliation-boundary.md) · [日本語](provider-reconciliation-boundary.ja.md)

`open-pr-issue-reconciliation-662.*` 和 `pre-release-documentation-alignment.json`
等参考文件，是源仓库历史版本的 provider/审查评估材料。它们记录过去某个版本
观察到的状态，不能证明当前仓库、当前 GitHub PR、发布或企业批准。

AI Cockpit 保持 provider 责任边界：

- Runtime 可以要求、绑定、展示和归档委托证据；
- GitHub/Hosted CI、审查者、分支保护、发布和企业留存由外部系统负责；
- 过期或缺失的对账是 unknown，不能授权 merge、release 或 close；
- 新 provider 观察必须针对当前仓库和 Work Item 身份重新采集，并带有自己的
  digest、时间和来源。

因此，逐文件台账将源 JSON/Markdown 记录标为 `reference-only`。它们不会复制到
`.ai/`，不会并入当前 status，也不会覆盖仓库本地 Contract 或 Runtime 证据。
目标工程边界请参见[发布分发](../release/distribution.zh-CN.md)和[参考源对齐](reference-parity.zh-CN.md)。

## 显式生命周期布局证据

Schema-v3 资源收尾使用 `postMergeSync.lifecycleLayout`（`separate` 或 `combined`）和 `postMergeSync.documentation`（`projected` 或 `absent`）；schema-v1/v2 保持原规则。`combined` 绑定真实 parent→commit 的 finish/archive 组合，不补造中间提交。Runtime 联合校验规范 Summary、Outcome、report、events、archive manifest 和 verification 绑定。冻结 report 可保留历史 active 路径。原 verification 和失败 attempt 保留原字节及 Runtime 身份；新增 attempt 必须绑定实际执行。

原始 PR 身份不可变；sync PR 绑定实际 base、head、merge，base 必须包含 source merge。Runtime 证明无冲突的 Git 合并结果，拒绝冲突和私加合并内容，不假定 merge tree 等于 sync head tree。同步分支仅允许本 WI 的生命周期记录及所需文档变化。

`absent` 是显式证据而非文档豁免：source head、生命周期 parent、archive、sync base/head 和 merge 均不得存在该 WI 页面、parity 行或文档 active 引用，且 Contract 必须允许缺席。历史 evidence/report 引用仍合法。provider 身份、新鲜度、消歧、同一 Git common directory 中保留的 control checkout、Retained → Deleted、finalize-verify 和独立真人 close 决定仍为硬门槛。
