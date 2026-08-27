---
author: AI Cockpit maintainers
workItemId: WI-333-comprehension-validation
title: "WI-333：参考源理解验证协议与参与者记录"
description: "逐一比对固定理解验证研究文件并记录不可移植的目标边界。"
audience:
  - adopter
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-333-comprehension-validation
capabilityClaims:
  - reference_parity
---

# WI-333：参考源理解验证协议与参与者记录

## 目的

逐一比对固定参考源文件并登记可审计分类。本 Work Item 只建立目标边界，不开展参与者研究，
也不转移人体参与者证据。

## 范围与决定

下面 12 个固定源路径全部为 `reference-only`。它们的流程、匿名标识、版本、回答、样本计数和
研究结论属于参考源，不能移植到本工程。目标对应物说明自己的读者路线、Agent workflow、Contract、
Outcome 和 Runtime evidence 边界。不复制源响应或结果字节，也不从源研究推导目标的理解、发布、
安全、安保或企业结论。

| 固定源路径 | 目标对应物 | 决定 |
| --- | --- | --- |
| `docs/reference/comprehension-validation-protocol.md` | `docs/README.md`、Agent workflow、Outcome report | `reference-only`；外部资格、同意和访谈协议 |
| `docs/reference/comprehension-validation-protocol.zh-CN.md` | 中文 README、Agent workflow、Outcome report | `reference-only`；不暗示目标参与者研究 |
| `docs/reference/comprehension-validation-protocol.ja.md` | 日文 README、Agent workflow、Outcome report | `reference-only`；源伦理不是 Runtime policy |
| `docs/reference/comprehension-validation-response.schema.json` | `.ai/README.md`、Outcome report | `reference-only`；不是 Runtime Contract/evidence schema |
| `docs/reference/comprehension-validation-responses/peter_01.en.json` | 英文 README、human-benefit report | `reference-only`；源历史响应 |
| `docs/reference/comprehension-validation-responses/peter_02.en.json` | 英文 README、human-benefit report | `reference-only`；不导入参与者数据 |
| `docs/reference/comprehension-validation-responses/tanaka_01.ja.json` | 日文 README、human-benefit report | `reference-only`；不是 adopter evidence |
| `docs/reference/comprehension-validation-responses/tanaka_02.ja.json` | 日文 README、human-benefit report | `reference-only`；绑定源版本 |
| `docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json` | 中文 README、human-benefit report | `reference-only`；不声称目标母语评分 |
| `docs/reference/comprehension-validation-responses/xiaoli_02.zh-CN.json` | 中文 README、human-benefit report | `reference-only`；不复制原始文本 |
| `docs/reference/comprehension-validation-results.json` | 三语 human-benefit report、comparison | `reference-only`；样本/结果绑定源版本 |
| `docs/reference/comprehension-validation-results.md` | 英文 human-benefit、Outcome report | `reference-only`；源限制不是目标证据 |

## 验收

- Inventory 正好包含 12 条 WI-333 记录，全部为 `reference-only`，且对应物和理由非空。
- 没有 deferred 或 migrate 记录，也不把参与者响应/结果复制进目标 evidence。
- 三语 comparison、parity、Work Item 文档说明相同边界。
- 文档和 inventory 检查通过；使用已安装 Runtime 产生当前证据，并完成审查 PR、合并、关闭和清理。

## 对象工程/采用方边界

采用方继承目标的文档路线、Contract、evidence 和 Agent workflow，不能继承另一仓库的人体参与者
记录。未来研究必须先由人拥有独立 Contract，明确同意、保留、隐私和 evidence。

语言版本：[English](WI-333-comprehension-validation.md) · [日本語](WI-333-comprehension-validation.ja.md)
