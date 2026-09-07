---
author: AI Cockpit maintainers
title: "WI-627——参考源逐文件台账重新基线"
description: "将逐文件比较台账绑定到最新本地参考 checkout，同时保留审计历史。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-627-reference-rebaseline
status: implemented
authority: canonical
lastVerifiedBy: WI-627-reference-rebaseline
---

# WI-627——参考源逐文件台账重新基线

## 目标

将比较台账重新绑定到最新本地参考源提交
`a9224aed77b5c317b53c4551a9eec306d91ee330`，同时保留历史决定，并把所有
发生变化或新出现的路径交给后续语义批次。这是台账和文档校正，不是把源工程实现移植进 Rust。

## 结果

当前台账包含 5,175 个 tracked path：4,262 个 generated-history、426 个
implemented-different-by-design、1 个 implemented-equivalent、8 个
not-applicable、124 个 reference-only、354 个 deferred-next-batch，
`migrate-gap` 为 0。本次没有退休路径。Rust 基线为
`98f12b18b978db509fc884a8a6225afeb7f10df5`，使用公开 Runtime v0.2.85，
二进制摘要为 `sha256:ece00d0b596c4674eaf37225e95a83aacec66a4c33f0bb89857f5dcaecde3a50`。

校验器先检查完整的当前路径集合，再只在历史批次归属检查中排除源内容发生变化的路径。这样合法的
重新基线不会被误报为旧批次缺记录，同时仍要求每个当前路径都有分类。

## 边界与对象工程继承

参考 checkout 仅使用本地固定提交；不复制源 Python、Shell、Make、provider 配置或源 JSON wire 格式。
已 attach 的对象工程继续继承一份共享 Runtime、显式 `--repo`、隔离的 Contract/evidence/knowledge 和
面向人的 Outcome；它们的仓库状态与本比较台账保持隔离。

## 验证

```text
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/docs/reference_comparison_metadata_test.py
```

另见：[English](WI-627-reference-rebaseline.md) · [日本語](WI-627-reference-rebaseline.ja.md)。
