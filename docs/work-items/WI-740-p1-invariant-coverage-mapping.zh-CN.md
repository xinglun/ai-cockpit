---
author: AI Cockpit maintainers
title: "WI-740——P1 协作不变量覆盖映射(重新交付)"
description: "在多代理WI编号冲突后重新交付WI-681的不变量覆盖映射，修正不变量8的判断，并反映不变量5与9已被独立解决的现状。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-740-p1-invariant-coverage-mapping
status: implemented
authority: authorized
lastVerifiedBy: WI-740-p1-invariant-coverage-mapping
terminalArchive: .ai/work-items/archive/WI-740-p1-invariant-coverage-mapping.contract.json
terminalVerification: .ai/evidence/WI-740-p1-invariant-coverage-mapping.verification.json
terminalFinalization: .ai/decisions/WI-740-p1-invariant-coverage-mapping.finalize.json
terminalDecision: .ai/decisions/WI-740-p1-invariant-coverage-mapping.close.json
---

[English](WI-740-p1-invariant-coverage-mapping.md) · [日本語](WI-740-p1-invariant-coverage-mapping.ja.md)

# WI-740——P1 协作不变量覆盖映射(重新交付)

## 意图

重新交付最初以 WI-681 尝试的协作不变量覆盖映射。WI-681 未合并即被关闭,
原因是一次真实的多代理 WI 编号冲突(另一个并行运行的代理独立地将相同的
短编号 `WI-681` 用于一个无关的文档晋级 Work Item,并率先通过 PR #677
合并)。本次重新交付同时修正了一处判断:不变量8(跨会话切换的授权适用
性)原本被标记为"未发现自动化测试",但更仔细阅读后发现
`crates/cockpit-repository/tests/preflight_review.rs::bound_human_review_receipt_allows_checkpoint_but_not_stale_reuse`
已经证明了其核心主张。原草稿正确指出的缺口不变量5与9,已分别由 WI-682
(PR #679)和 WI-710(PR #702)在本次重新交付之前独立关闭,因此本页反映
这一现状,而非过时的缺口清单。依据仓库所有者的明确授权,继续推进 AI
Cockpit 协作语言专项。

## 边界

这是仅限文档的 Work Item。新增
`docs/reference/collaboration-invariant-coverage.md`(+ zh-CN/ja)、
`docs/reference/README.md`(+ zh-CN/ja)中的一条索引条目、本记录页面,以及
自身在 reference-parity 中的登记。不修改任何测试文件、任何 CLI/MCP 行为,
也不修改任何 Contract/Outcome schema。编写不变量7的测试明确不在本次范围
内,已在交付文档中列为有边界的后续 Work Item。

## 验收与生命周期

- 不变量1、2、3、4、5、6、8、9、10 均陈述为已自动化(完全或部分),每一条
  都引用了在交付时直接阅读当前测试源码所确认的测试文件与函数。不变量7
  陈述为唯一剩余的已命名缺口。
- 文档明确说明本页为何取代 WI-681,以及自该草稿以来发生了什么变化。
- 遵循 `start → preflight → checkpoint → verify → finish → archive → close`;
  保留 `user_visible_benefit_not_declared`。
- 在精确审查的 head 上,`bash tests/docs/documentation_acceptance.sh` 通过。

## 证据

- archive:`.ai/work-items/archive/WI-740-p1-invariant-coverage-mapping.contract.json`
- verification:`.ai/evidence/WI-740-p1-invariant-coverage-mapping.verification.json`
- finalization:`.ai/decisions/WI-740-p1-invariant-coverage-mapping.finalize.json`(待合并后生成)
- close:`.ai/decisions/WI-740-p1-invariant-coverage-mapping.close.json`(待合并后生成)
