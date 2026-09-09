---
author: AI Cockpit maintainers
title: "协作不变量覆盖情况"
description: "将十条协作语言语义不变量映射到既有的自动化测试覆盖，引用确切的测试文件与函数，并准确说明剩余边界。"
audience: [maintainer, reviewer, contributor]
status: current
authority: canonical
lastVerifiedBy: WI-753-p1-runtime-consistency
---

# 协作不变量覆盖情况

本页回答:对于 WI-679 协作语言契约(`docs/reference/collaboration-language-contract.md`,
已通过 PR #675 合并)中陈述的十条语义不变量,今天是否已经有自动化测试在
强制执行?如果有,是哪一个?其目的是防止重复建设测试基础设施(优先复用),
并如实说明哪些不变量尚无自动化交叉检查。

以下每一条引用都是在撰写本页时直接阅读当前测试源码得到的,没有任何一条
仅凭测试名称推断。

**本页取代此前的 WI-681 草稿。** WI-681 未合并即被关闭,原因是另一个并行
运行的代理独立使用了相同的短编号(`WI-681-wi674-doc-promotion`,已通过
PR #677 合并)——这是一次真实的多代理 WI 编号冲突,而非内容缺陷。本次
重新交付(WI-740)同时修正了该草稿中的一处判断:不变量8原本被标记为
"未发现自动化测试",但更仔细阅读后发现已有测试覆盖其核心主张(见下方第8
行)。WI-681 指出的缺口不变量5与9,已分别由 WI-682(PR #679)与 WI-710
(PR #702)独立关闭,因此本页也将其标记为"是"。

## 覆盖情况表

| # | 不变量 | 今天是否自动化? | 测试 | 断言内容 |
| --- | --- | --- | --- | --- |
| 1 | 验证通过 ≠ 全面受理/合并授权 | **是** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json` | 让同一个 Work Item 经历 `finish`(绿)→`archive`(黄,终结尚未完成)→记录一次 `Deleted` 终结 →`close`(需要显式 `--human-decision`),并在每一步断言交接文本与稳定 JSON 都不会把这些状态坍缩成单一的"已完成"。 |
| 2 | 空记录 ≠ 无风险 | **是** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields` | 渲染一个风险/测试削弱字段为空的 Outcome,断言人类可读文本从不会因数据缺失而声称"未发现风险"或等价的正面结论。 |
| 3 | 未知事实不能由表达层补全 | **是(部分)** | `crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | 遍历 `tests/adversarial/manifest.json` 中的语义案例(目前 15 条)并通过 `evaluate()` 断言评估结果绑定于案例数据而非措辞。仅覆盖该清单中已收录的对抗性措辞,并非穷尽。 |
| 4 | 历史记录不能无依据地变成当前的通过/失败 | **是** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_preserves_historical_and_superseded_distinctions`、`::archived_report_tamper_is_red_and_not_reprojected_as_verified` | 断言 `runtime_historical` 记录保留其历史措辞(不会出现本应用于当前失败的"Repair the missing evidence"文案),以及被篡改的归档报告会渲染为红色,而不是被静默地重新投影为已验证。 |
| 5 | 同一事实在 CLI/MCP/摘要/完整报告中保持一致 | **是** | `crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs::cli_subprocess_and_mcp_handler_agree_on_the_same_outcome`(由 WI-682、PR #679 新增) | 为 CLI 一侧启动真实 `ai-cockpit` 二进制子进程,为 MCP 一侧在进程内调用 `cockpit_mcp::handle_request_for_repo()`,针对同一仓库 fixture 与同一 Work Item,断言 CLI 的 `work-item outcome --json` 输出与 MCP `work_item_outcome` 工具的 `structuredContent.outcome` 完全相等(使用与被测二进制完全匹配的 `RuntimeContext`)。这正是此前 `crates/cockpit-mcp/tests/rpc.rs::*_with_cli_parity` 系列测试(仅比较两次进程内调用)未能提供的真正跨进程检查。 |
| 6 | 更换语言不改变事实/授权范围/后果 | **是** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json`;`crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | CLI 测试依次以 `AI_COCKPIT_LANGUAGE=en/zh-CN/ja` 启动真实二进制,断言机器可读的 stdout JSON 字段在各语言下逐字节相同,只有面向人的 stderr 文本本地化。对抗性语料测试进一步检查每个语义案例在每种语言下的 5 种措辞变体评估结果相同。 |
| 7 | 展示的下一步与当前 Runtime 状态/策略一致 | **是(有界)** | `crates/cockpit-repository/tests/scenario_matrix_next_action.rs`(WI-741、PR #713); `crates/cockpit-cli/tests/collaboration_consistency.rs::displayed_option_state_and_runtime_transition_stay_consistent_through_resume`(WI-753) | WI-741 将展示的下一步消息绑定到实际观测的场景矩阵。WI-753 进一步断言展示的人工决定选项与 checkpoint、中断、恢复过程中的每个 Runtime 转换一致。覆盖是有界的,不构成所有状态的通用判定器。 |
| 8 | 既有授权是否适用由规则与记录决定,不因会话切换而改变 | **是** | `crates/cockpit-repository/tests/preflight_review.rs::bound_human_review_receipt_allows_checkpoint_but_not_stale_reuse` | 记录一份决定回执,确认 `preflight` 转为 `human_decision_recorded` 且 `checkpoint` 在快照未变时成功(复用有效);随后改变仓库内容,断言 `preflight` 回到 `needs_human_confirmation` 且 `checkpoint` 被拒绝(快照变化使此前的决定失效,必须重新决定)。这直接证明了该不变量所述的"依规则与记录判断"这一核心主张;该测试早于本 Work Item 就已存在,此前的 WI-681 草稿误将其判定为缺口。若要针对*不同调用者身份*(而非同一调用者跨快照变化)专门建测试,是一个更窄的可选后续,而非当前对不变量核心主张的缺失检查。 |
| 9 | 每个需要人工决定的问题都指明对象/影响/恢复条件 | **是** | `crates/cockpit-repository/tests/contract_preflight.rs::assert_human_decision_request_is_complete`,由 `::scaffold_preflight_is_not_ready_and_records_human_review_requirements` 与 `::high_risk_scenario_coverage_stops_at_preflight_for_human_review` 调用(由 WI-710、PR #702 新增) | 断言 `what_happened`、`why_it_matters`、`question`、`resume_condition`、`options`、`recommended_option`、`recommendation_reason` 均非空,`recommended_option` 指向某个已提供的选项,且每个选项的 `id`/`label`/`effect` 均非空——针对两个独立触发的真实 `needs_human_confirmation` 场景进行检查。 |
| 10 | 摘要可以省略细节但不能隐藏阻断/关键未知项/必要决定 | **是(部分)** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields`(同文件中的相邻断言) | 直接覆盖了"空不等于正面结论"的一半;未发现有单独测试专门断言一个*已填充*的阻断/未知项/决定不会被摘要压缩丢弃,该情形目前只是被第1条的生命周期测试顺带覆盖。 |

## 如何阅读本表

- **是**表示本仓库自身测试套件中已存在一个通过的测试,确实按不变量所述强制执行,本页引用了确切的函数。
- **部分**表示存在真实覆盖,但未触及不变量所述的全部情形(见该行说明)。
- **间接**(仅不变量7)表示存在相关的状态转换断言,但没有测试直接、端到端地断言该具体主张。

## 剩余边界

不变量7不再是名义上的零覆盖缺口:WI-741 与 WI-753 已提供绑定实际观测
Runtime 状态的有界可执行检查。这些检查不是所有状态/选项组合的通用判定器。
第五节的独立引接完整性检查仍单独规划,以便证明无需会话历史即可重建状态,
避免与这里的状态/转换一致性检查混为一谈。
