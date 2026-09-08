---
author: AI Cockpit maintainers
title: "协作不变量覆盖情况"
description: "将十条协作语言语义不变量映射到既有的自动化测试覆盖，引用确切的测试文件与函数，并准确说明尚存在的缺口。"
audience: [maintainer, reviewer, contributor]
status: current
authority: canonical
lastVerifiedBy: WI-681-p1-invariant-coverage-mapping
---

# 协作不变量覆盖情况

本页回答:对于 WI-679 协作语言契约(`docs/reference/collaboration-language-contract.zh-CN.md`,
在本 Work Item 提交时尚未进入默认分支;见 PR #675)中陈述的
十条语义不变量,今天是否已经有自动化测试在强制执行?如果有,是哪一个?
其目的是防止重复建设测试基础设施(优先复用),并如实说明哪些不变量尚无
自动化交叉检查。

以下每一条引用都是在撰写本页时直接阅读当前测试源码得到的,没有任何一条
仅凭测试名称推断。

## 覆盖情况表

| # | 不变量 | 今天是否自动化? | 测试 | 断言内容 |
| --- | --- | --- | --- | --- |
| 1 | 验证通过 ≠ 全面受理/合并授权 | **是** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json` | 让同一个 Work Item 经历 `finish`(绿)→`archive`(黄,终结尚未完成)→记录一次 `Deleted` 终结 →`close`(需要显式 `--human-decision`),并在每一步断言交接文本与稳定 JSON 都不会把这些状态坍缩成单一的"已完成"。 |
| 2 | 空记录 ≠ 无风险 | **是** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields` | 渲染一个风险/测试削弱字段为空的 Outcome,断言人类可读文本从不会因数据缺失而声称"未发现风险"或等价的正面结论。 |
| 3 | 未知事实不能由表达层补全 | **是(部分)** | `crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | 遍历 `tests/adversarial/manifest.json` 中的语义案例(目前 15 条)并通过 `evaluate()` 断言评估结果绑定于案例数据而非措辞。仅覆盖该清单中已收录的对抗性措辞,并非穷尽。 |
| 4 | 历史记录不能无依据地变成当前的通过/失败 | **是** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_preserves_historical_and_superseded_distinctions`、`::archived_report_tamper_is_red_and_not_reprojected_as_verified` | 断言 `runtime_historical` 记录保留其历史措辞(不会出现本应用于当前失败的"Repair the missing evidence"文案),以及被篡改的归档报告会渲染为红色,而不是被静默地重新投影为已验证。 |
| 5 | 同一事实在 CLI/MCP/摘要/完整报告中保持一致 | **部分——真实缺口** | `crates/cockpit-mcp/tests/rpc.rs::mcp_work_item_outcome_returns_explicit_human_handoff_with_cli_parity`、`::mcp_blocked_outcome_exposes_the_same_recovery_facts_as_cli` | 这些测试是在进程内直接调用 `cockpit_mcp::handle_request_for_repo()`(库函数调用),并将其输出与固定的预期字符串比较。这证明了 MCP handler *自身*输出的内部一致性、且与文档记载的 CLI 措辞一致,但从未在同一测试中真正启动 `ai-cockpit` CLI 二进制并对两者的实时输出做差异比较。**目前不存在真正的、跨进程的 CLI 子进程与 MCP handler 一致性测试。** |
| 6 | 更换语言不改变事实/授权范围/后果 | **是** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json`;`crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | CLI 测试依次以 `AI_COCKPIT_LANGUAGE=en/zh-CN/ja` 启动真实二进制,断言机器可读的 stdout JSON 字段在各语言下逐字节相同,只有面向人的 stderr 文本本地化。对抗性语料测试进一步检查每个语义案例在每种语言下的 5 种措辞变体评估结果相同。 |
| 7 | 展示的下一步与当前 Runtime 状态/策略一致 | **间接** | 与第1条相同的生命周期测试,加上 `crates/cockpit-repository/tests/status_projection.rs::status_projection_distinguishes_archived_from_valid_closed_decision` | 这些测试在每一次真实状态转换时断言与下一步相关的字段(生命周期阶段、阻断项),但没有测试专门针对广泛的状态矩阵,把渲染出的"下一步"语句与独立计算出的预期动作逐一比对。 |
| 8 | 既有授权是否适用由规则与记录决定,不因会话切换而改变 | **未发现自动化测试** | — | 没有测试模拟两个不同的调用方/会话,基于身份绑定摘要复用(或应当拒绝复用)一份 Receipt/决定。这是一个真实缺口;相关已记载/已观测行为见 `docs/reference/collaboration-scenario-matrix.json` 的 SCN-010/SCN-011/SCN-022。 |
| 9 | 每个需要人工决定的问题都指明对象/影响/恢复条件 | **未发现专门测试** | — | `humanDecisionRequest` 的结构由生产代码(`preflight`)生成,并记载于 `docs/reference/agent-workflow.md`,但未发现有测试断言这四个必需字段(what/why/options/question/resumeCondition)始终同时非空。 |
| 10 | 摘要可以省略细节但不能隐藏阻断/关键未知项/必要决定 | **是(部分)** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields`(同文件中的相邻断言) | 直接覆盖了"空不等于正面结论"的一半;未发现有单独测试专门断言一个*已填充*的阻断/未知项/决定不会被摘要压缩丢弃,该情形目前只是被第1条的生命周期测试顺带覆盖。 |

## 如何阅读本表

- **是**表示本仓库自身测试套件中已存在一个通过的测试,确实按不变量所述强制执行,本页引用了确切的函数。
- **部分**表示存在真实覆盖,但未触及不变量所述的全部情形(见该行说明)。
- **未发现自动化测试**就是字面意思:在 `tests/` 及各 crate 的 `tests/` 目录中搜索未找到。这并不意味着 Runtime 一定违反该不变量——其中几条也由上述测试间接覆盖到的生产代码在结构上加以保护——只是说明目前没有测试会在回归时捕获它。

## 已知缺口与建议的后续 Work Item

1. **不变量5(跨入口一致性)**:新增一个集成测试,在同一测试中把真实 `ai-cockpit` CLI 二进制作为子进程启动(复用 `outcome_handoff.rs` 已有的方式),并调用 `cockpit_mcp::handle_request_for_repo()`(复用 `rpc.rs` 已有的方式)针对同一个仓库 fixture,然后断言两者在所有稳定字段上一致。这只是组合两个已被验证过的模式,而不是引入新的测试基础设施。
2. **不变量8(跨会话的授权复用)**:新增一个测试,先记录一份决定回执,再模拟第二个"会话"(针对同一仓库的一次全新进程调用)尝试复用:(a)未发生变化——应当复用;(b)Contract/快照摘要发生变化——应当要求全新决定。`crates/cockpit-repository/tests/` 中已有 `attach()` + `start_work_item_with_options()` 的 fixture 模式可供搭建。
3. **不变量9(`humanDecisionRequest` 完整性)**:新增一条聚焦断言(可放在 `crates/cockpit-repository/tests/` 现有的 preflight 测试旁边),验证每一个 `needs_human_confirmation` 的 preflight 结果,其 `humanDecisionRequest` 的 `whatHappened`/`whyItMatters`/`options`/`question`/`resumeCondition` 均非空。
4. **不变量7(下一步的正确性)**:这是最难被通用化测试的一条,因为"正确"取决于 `collaboration-scenario-matrix.json` 中的完整状态矩阵。一个务实的下一步是,针对该矩阵中 `observed` 的一个子集场景,断言下一步字段,而不是尝试建立一个通用的判定器。

以上四条均未在本 Work Item 中实现;每一条都是有边界、可独立交付的后续工作,复用既有测试模式而非新增一套,符合本专项自身"复用既有测试设施"的方向。
